import { computed, ref } from "vue";
import { apiFetch } from "./apiFetch";
import { DEFAULT_API_BASE_URL, normalizeApiBaseUrl, useSettings } from "./useSettings";

const AUTH_STORAGE_KEY = "lingjing.auth.v1";

export interface LingjingUser {
  id?: number;
  username?: string;
  email?: string;
  nickname?: string;
  avatar?: string;
  points?: number;
  role?: string;
  isActive?: boolean;
}

interface StoredAuth {
  token: string;
  user: LingjingUser | null;
}

interface ApiEnvelope<T> {
  ok: boolean;
  data?: T;
  error?: string;
  code?: number;
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
  return normalizeApiBaseUrl(settings.value.apiBaseUrl || DEFAULT_API_BASE_URL);
}

function authUrl(path: string): string {
  return `${apiBase()}/api/lingjing/auth/${path.replace(/^\//, "")}`;
}

async function parseJson<T>(res: Response): Promise<ApiEnvelope<T>> {
  const body = (await res.json()) as ApiEnvelope<T>;
  if (!res.ok && body.ok !== false) {
    throw new Error(body.error || res.statusText || `HTTP ${res.status}`);
  }
  return body;
}

function envelopeError(body: ApiEnvelope<unknown>, fallback: string): Error {
  return new Error(body.error || fallback);
}

function extractToken(data: Record<string, unknown> | undefined): string | null {
  if (!data) return null;
  const t = data.access_token ?? data.accessToken ?? data.token;
  return typeof t === "string" && t.trim() ? t.trim() : null;
}

function extractUser(data: Record<string, unknown> | undefined): LingjingUser | null {
  if (!data) return null;
  const u = (data.user as Record<string, unknown> | undefined) ?? data;
  if (!u || typeof u !== "object") return null;
  const avatarRaw = u.avatar_url ?? u.avatar;
  const activeRaw = u.is_active ?? u.isActive;
  return {
    id: typeof u.id === "number" ? u.id : undefined,
    username: typeof u.username === "string" ? u.username : undefined,
    email: typeof u.email === "string" ? u.email : undefined,
    nickname: typeof u.nickname === "string" ? u.nickname : undefined,
    avatar: typeof avatarRaw === "string" ? avatarRaw : undefined,
    points: typeof u.points === "number" ? u.points : undefined,
    role: typeof u.role === "string" ? u.role : undefined,
    isActive: typeof activeRaw === "boolean" ? activeRaw : undefined,
  };
}

async function applyAuthResponse(body: ApiEnvelope<Record<string, unknown>>) {
  if (!body.ok) {
    throw envelopeError(body, "登录失败");
  }
  const nextToken = extractToken(body.data);
  if (!nextToken) {
    throw new Error("登录响应缺少 access_token");
  }
  token.value = nextToken;
  user.value = extractUser(body.data);
  persistAuth();
  await refreshMe().catch(() => undefined);
}

async function refreshMe(): Promise<LingjingUser | null> {
  if (!token.value) return null;
  const res = await apiFetch(authUrl("me"), {
    headers: { Authorization: `Bearer ${token.value}` },
  });
  const body = await parseJson<Record<string, unknown>>(res);
  if (!body.ok) {
    if (res.status === 401) {
      await logout();
    }
    throw envelopeError(body, "获取用户信息失败");
  }
  user.value = extractUser(body.data) ?? user.value;
  persistAuth();
  return user.value;
}

/** Probe login state via GET /api/lingjing/auth/session (HTTP 200 + valid flag). */
async function checkSession(): Promise<boolean> {
  if (!token.value) return false;
  try {
    const res = await apiFetch(authUrl("session"), {
      headers: { Authorization: `Bearer ${token.value}` },
    });
    const body = await parseJson<{ valid?: boolean; user?: Record<string, unknown> | null }>(res);
    if (!body.ok || !body.data?.valid) {
      await logout();
      return false;
    }
    if (body.data.user) {
      user.value = extractUser(body.data.user as Record<string, unknown>) ?? user.value;
      persistAuth();
    }
    return true;
  } catch {
    return !!token.value;
  }
}

async function logout(): Promise<void> {
  const t = token.value;
  token.value = null;
  user.value = null;
  persistAuth();
  if (!t) return;
  try {
    await apiFetch(authUrl("logout"), {
      method: "POST",
      headers: { Authorization: `Bearer ${t}` },
    });
  } catch {
    /* ignore */
  }
}

export function useAuth() {
  const isLoggedIn = computed(() => !!token.value);

  async function login(username: string, password: string): Promise<void> {
    authLoading.value = true;
    try {
      const res = await apiFetch(authUrl("login"), {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ username: username.trim(), password }),
      });
      await applyAuthResponse(await parseJson<Record<string, unknown>>(res));
    } finally {
      authLoading.value = false;
    }
  }

  async function sendLoginCode(email: string): Promise<void> {
    const res = await apiFetch(authUrl("send-login-code"), {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email: email.trim() }),
    });
    const body = await parseJson<Record<string, unknown>>(res);
    if (!body.ok) {
      throw envelopeError(body, "发送验证码失败");
    }
  }

  async function loginWithCode(email: string, code: string): Promise<void> {
    authLoading.value = true;
    try {
      const res = await apiFetch(authUrl("login-code"), {
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
    checkSession,
    authHeaders,
    loadStoredAuth,
  };
}

// init once
loadStoredAuth();
