<script setup lang="ts">
import { computed, onUnmounted, ref } from "vue";
import { showToast } from "../composables/useToast";
import { useAuth } from "../composables/useAuth";

const emit = defineEmits<{ (e: "success"): void }>();

const { user, isLoggedIn, authLoading, login, sendLoginCode, loginWithCode, logout } = useAuth();

type LoginMode = "password" | "code";
const mode = ref<LoginMode>("password");

const account = ref("");
const password = ref("");
const email = ref("");
const code = ref("");
const codeCooldown = ref(0);
let codeTimer: number | null = null;

const displayName = computed(
  () => user.value?.nickname || user.value?.username || user.value?.email || "已登录",
);

function startCooldown(seconds = 60) {
  codeCooldown.value = seconds;
  if (codeTimer !== null) window.clearInterval(codeTimer);
  codeTimer = window.setInterval(() => {
    codeCooldown.value -= 1;
    if (codeCooldown.value <= 0 && codeTimer !== null) {
      window.clearInterval(codeTimer);
      codeTimer = null;
    }
  }, 1000);
}

async function onPasswordLogin() {
  if (!account.value.trim() || !password.value) {
    showToast("请输入账号和密码");
    return;
  }
  try {
    await login(account.value, password.value);
    password.value = "";
    showToast(`欢迎回来，${displayName.value}`);
    emit("success");
  } catch (e) {
    showToast(e instanceof Error ? e.message : String(e));
  }
}

async function onSendCode() {
  if (!email.value.trim()) {
    showToast("请输入邮箱");
    return;
  }
  if (codeCooldown.value > 0) return;
  try {
    await sendLoginCode(email.value);
    showToast("验证码已发送，请查收邮箱");
    startCooldown();
  } catch (e) {
    showToast(e instanceof Error ? e.message : String(e));
  }
}

async function onCodeLogin() {
  if (!email.value.trim() || !code.value.trim()) {
    showToast("请输入邮箱和验证码");
    return;
  }
  try {
    await loginWithCode(email.value, code.value);
    code.value = "";
    showToast(`欢迎回来，${displayName.value}`);
    emit("success");
  } catch (e) {
    showToast(e instanceof Error ? e.message : String(e));
  }
}

async function onLogout() {
  await logout();
  showToast("已退出登录");
}

onUnmounted(() => {
  if (codeTimer !== null) window.clearInterval(codeTimer);
});
</script>

<template>
  <div class="login-panel">
    <template v-if="isLoggedIn">
      <div class="user-card">
        <div class="avatar">{{ displayName.slice(0, 1).toUpperCase() }}</div>
        <div class="meta">
          <div class="name">{{ displayName }}</div>
          <div v-if="user?.email" class="email">{{ user.email }}</div>
        </div>
        <button type="button" class="btn-out" :disabled="authLoading" @click="onLogout">退出</button>
      </div>
    </template>
    <template v-else>
      <p class="hint">登录后可同步社区点赞状态；未登录也可正常浏览在线壁纸。</p>

      <div class="tabs" role="tablist">
        <button
          type="button"
          class="tab"
          :class="{ on: mode === 'password' }"
          role="tab"
          :aria-selected="mode === 'password'"
          @click="mode = 'password'"
        >密码登录</button>
        <button
          type="button"
          class="tab"
          :class="{ on: mode === 'code' }"
          role="tab"
          :aria-selected="mode === 'code'"
          @click="mode = 'code'"
        >验证码登录</button>
      </div>

      <template v-if="mode === 'password'">
        <label class="field">
          <span>账号</span>
          <input
            v-model="account"
            type="text"
            autocomplete="username"
            placeholder="邮箱或用户名"
            @keydown.enter="onPasswordLogin"
          />
        </label>
        <label class="field">
          <span>密码</span>
          <input
            v-model="password"
            type="password"
            autocomplete="current-password"
            placeholder="请输入密码"
            @keydown.enter="onPasswordLogin"
          />
        </label>
        <button type="button" class="btn-in" :disabled="authLoading" @click="onPasswordLogin">
          {{ authLoading ? "登录中…" : "登录" }}
        </button>
      </template>

      <template v-else>
        <label class="field">
          <span>邮箱</span>
          <input v-model="email" type="email" autocomplete="username" placeholder="user@example.com" />
        </label>
        <label class="field">
          <span>验证码</span>
          <div class="code-row">
            <input
              v-model="code"
              type="text"
              inputmode="numeric"
              autocomplete="one-time-code"
              placeholder="6 位验证码"
              maxlength="6"
              @keydown.enter="onCodeLogin"
            />
            <button
              type="button"
              class="btn-code"
              :disabled="authLoading || codeCooldown > 0"
              @click="onSendCode"
            >
              {{ codeCooldown > 0 ? `${codeCooldown}s` : "获取验证码" }}
            </button>
          </div>
        </label>
        <button type="button" class="btn-in" :disabled="authLoading" @click="onCodeLogin">
          {{ authLoading ? "登录中…" : "登录" }}
        </button>
      </template>
    </template>
  </div>
</template>

<style scoped>
.login-panel {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.hint {
  font-size: 12px;
  color: var(--text-2);
  line-height: 1.55;
  margin: 0;
}
.tabs {
  display: flex;
  gap: 6px;
  padding: 3px;
  background: var(--surface-2);
  border-radius: var(--r-md);
  border: 1px solid var(--border);
}
.tab {
  flex: 1;
  border: none;
  background: transparent;
  color: var(--text-2);
  font-size: 12px;
  font-weight: 600;
  padding: 7px 10px;
  border-radius: calc(var(--r-md) - 2px);
  cursor: pointer;
}
.tab.on {
  background: var(--surface);
  color: var(--primary);
  box-shadow: var(--sh-sm);
}
.field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
  color: var(--text-2);
}
.field input {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--r-md);
  padding: 8px 10px;
  font: inherit;
  color: var(--text);
}
.field input:focus {
  border-color: var(--primary);
  box-shadow: 0 0 0 3px var(--primary-soft);
  outline: none;
}
.code-row {
  display: flex;
  gap: 8px;
}
.code-row input {
  flex: 1;
  min-width: 0;
}
.btn-code {
  flex-shrink: 0;
  border: 1px solid var(--border);
  border-radius: var(--r-md);
  padding: 0 10px;
  font-size: 12px;
  font-weight: 600;
  background: var(--surface-2);
  color: var(--text);
  cursor: pointer;
  white-space: nowrap;
}
.btn-code:disabled {
  opacity: 0.55;
  cursor: default;
}
.btn-in,
.btn-out {
  border: 1px solid var(--border);
  border-radius: var(--r-md);
  padding: 8px 14px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}
.btn-in {
  width: 100%;
  background: var(--primary);
  color: #fff;
  border-color: var(--primary);
  align-self: stretch;
}
.btn-in:hover { background: var(--primary-hover); }
.btn-in:disabled { opacity: 0.6; cursor: default; }
.btn-out {
  background: var(--surface);
  color: var(--text);
}
.btn-out:hover { background: var(--surface-2); }
.user-card {
  display: flex;
  align-items: center;
  gap: 12px;
}
.avatar {
  width: 40px;
  height: 40px;
  border-radius: 999px;
  background: var(--primary-soft);
  color: var(--primary);
  display: grid;
  place-items: center;
  font-weight: 700;
  flex-shrink: 0;
}
.meta { flex: 1; min-width: 0; }
.name { font-size: 14px; font-weight: 600; }
.email { font-size: 12px; color: var(--text-2); overflow: hidden; text-overflow: ellipsis; }
</style>
