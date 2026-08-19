<script setup lang="ts">
import { ref, computed } from "vue";
import { showToast } from "../composables/useToast";
import { useAuth } from "../composables/useAuth";

const { user, isLoggedIn, authLoading, login, logout } = useAuth();

const email = ref("");
const password = ref("");

const displayName = computed(
  () => user.value?.nickname || user.value?.username || user.value?.email || "已登录",
);

async function onLogin() {
  if (!email.value.trim() || !password.value) {
    showToast("请输入邮箱和密码");
    return;
  }
  try {
    await login(email.value, password.value);
    password.value = "";
    showToast(`欢迎回来，${displayName.value}`);
  } catch (e) {
    showToast(e instanceof Error ? e.message : String(e));
  }
}

async function onLogout() {
  await logout();
  showToast("已退出登录");
}
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
      <label class="field">
        <span>邮箱</span>
        <input v-model="email" type="email" autocomplete="username" placeholder="user@example.com" />
      </label>
      <label class="field">
        <span>密码</span>
        <input
          v-model="password"
          type="password"
          autocomplete="current-password"
          placeholder="请输入密码"
          @keydown.enter="onLogin"
        />
      </label>
      <button type="button" class="btn-in" :disabled="authLoading" @click="onLogin">
        {{ authLoading ? "登录中…" : "登录" }}
      </button>
    </template>
  </div>
</template>

<style scoped>
.login-panel {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-top: 8px;
  padding: 14px;
  border: 1px solid var(--border);
  border-radius: var(--r-md);
  background: var(--surface-2);
}
.hint {
  font-size: 12px;
  color: var(--text-2);
  line-height: 1.55;
  margin: 0;
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
.btn-in,
.btn-out {
  border: 1px solid var(--border);
  border-radius: var(--r-md);
  padding: 8px 14px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: transform var(--dur-fast) var(--ease-spring);
}
.btn-in {
  background: var(--primary);
  color: #fff;
  border-color: var(--primary);
  align-self: flex-start;
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
