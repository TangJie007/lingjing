import { computed, ref } from "vue";
import { useSettings } from "./useSettings";

const AUTH_STORAGE_KEY = "lingjing.auth.v1";

export interface LingjingUser {
  id?: number;
  username?: string;
  email?: string;
  nickname?: string;
  avatar?: string;
}

interface StoredAuth {
  token: string;
  user: LingjingUser | null;
}

interface ApiEnvelope<T> {
  ok: boolean;
  data?: T;
  error?: string;
}

const token = ref<string | null>(null);
const user = ref<LingjingUser | null>(null);
const authLoading = ref(false);

function loadStoredAuth() {
  try {
    const raw = localStorage.getItem(AUTH_STORAGE_KEY);
    if (!raw) return;
    const parsed = JSON.parse(raw) as StoredAuth;
    if (parsed.token) {
      token.value = parsed.token;
      user.value = parsed.user ?? null;
    }
  } catch {
    localStorage.removeItem(AUTH_STORAGE_KEY);
  }
}

function persistAuth() {
  if (!token.value) {
    localStorage.removeItem(AUTH_STORAGE_KEY);
    return;
  }
  const payload: StoredAuth = { token: token.value, user: user.value };
  localStorage.setItem(AUTH_STORAGE_KEY, JSON.stringify(payload));
}

function apiBase(): string {
  const settings = useSettings();
  return (settings.value.apiBaseUrl || "http://localhost:3002").replace(/\/$/, "");
}

async function parseJson<T>(res: Response): Promise<ApiEnvelope<T>> {
  const body = (await res.json()) as ApiEnvelope<T>;
  if (!res.ok && body.ok !== false) {
    throw new Error(res.statusText || `HTTP ${res.status}`);
  }
  return body;
}

function extractToken(data: Record<string, unknown> | undefined): string | null {
  if (!data) return null;
  const t = data.token ?? data.accessToken ?? data.access_token;
  return typeof t === "string" && t.trim() ? t.trim() : null;
}

function extractUser(data: Record<string, unknown> | undefined): LingjingUser | null {
  if (!data) return null;
  const u = data.user ?? data;
  if (!u || typeof u !== "object") return null;
  const obj = u as Record<string, unknown>;
  return {
    id: typeof obj.id === "number" ? obj.id : undefined,
    username: typeof obj.username === "string" ? obj.username : undefined,
    email: typeof obj.email === "string" ? obj.email : undefined,
    nickname: typeof obj.nickname === "string" ? obj.nickname : undefined,
    avatar: typeof obj.avatar === "string" ? obj.avatar : undefined,
  };
}

async function applyAuthResponse(body: ApiEnvelope<Record<string, unknown>>) {
  if (!body.ok) {
    throw new Error(body.error || "登录失败");
  }
  const nextToken = extractToken(body.data);
  if (!nextToken) {
    throw new Error("登录响应缺少 token");
  }
  token.value = nextToken;
  user.value = extractUser(body.data);
  persistAuth();
  await refreshMe().catch(() => undefined);
}

async function refreshMe(): Promise<LingjingUser | null> {
  if (!token.value) return null;
  const res = await fetch(`${apiBase()}/api/lingjing/auth/me`, {
    headers: { Authorization: `Bearer ${token.value}` },
  });
  const body = await parseJson<Record<string, unknown>>(res);
  if (!body.ok) {
    if (res.status === 401) {
      await logout();
    }
    throw new Error(body.error || "获取用户信息失败");
  }
  user.value = extractUser(body.data) ?? user.value;
  persistAuth();
  return user.value;
}

async function logout(): Promise<void> {
  const t = token.value;
  token.value = null;
  user.value = null;
  persistAuth();
  if (!t) return;
  try {
    await fetch(`${apiBase()}/api/lingjing/auth/logout`, {
      method: "POST",
      headers: { Authorization: `Bearer ${t}` },
    });
  } catch {
    /* ignore */
  }
}

export function useAuth() {
  const isLoggedIn = computed(() => !!token.value);

  async function login(account: string, password: string): Promise<void> {
    authLoading.value = true;
    try {
      const res = await fetch(`${apiBase()}/api/lingjing/auth/login`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ account: account.trim(), password }),
      });
      await applyAuthResponse(await parseJson<Record<string, unknown>>(res));
    } finally {
      authLoading.value = false;
    }
  }

  async function sendLoginCode(email: string): Promise<void> {
    const res = await fetch(`${apiBase()}/api/lingjing/auth/send-login-code`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email: email.trim() }),
    });
    const body = await parseJson<Record<string, unknown>>(res);
    if (!body.ok) {
      throw new Error(body.error || "发送验证码失败");
    }
  }

  async function loginWithCode(email: string, code: string): Promise<void> {
    authLoading.value = true;
    try {
      const res = await fetch(`${apiBase()}/api/lingjing/auth/login-code`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email: email.trim(), code: code.trim() }),
      });
      await applyAuthResponse(await parseJson<Record<string, unknown>>(res));
    } finally {
      authLoading.value = false;
    }
  }

  function authHeaders(): Record<string, string> {
    if (!token.value) return {};
    return { Authorization: `Bearer ${token.value}` };
  }

  return {
    token,
    user,
    authLoading,
    isLoggedIn,
    login,
    sendLoginCode,
    loginWithCode,
    logout,
    refreshMe,
    authHeaders,
    loadStoredAuth,
  };
}

// init once
loadStoredAuth();
