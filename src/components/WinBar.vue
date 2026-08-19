<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import AppLogo from "./AppLogo.vue";
import { showToast } from "../composables/useToast";

defineProps<{
  onlineEnabled?: boolean;
  loggedIn?: boolean;
  userLabel?: string;
}>();

const emit = defineEmits<{ (e: "login"): void }>();

async function onDrag(e: MouseEvent) {
  if (e.button !== 0) return;
  const target = e.target as HTMLElement | null;
  if (target?.closest(".win-act, .app-logo")) return;
  await invoke("start_drag");
}

async function minimize() {
  try {
    await invoke("minimize_main");
  } catch (e) {
    showToast(e instanceof Error ? e.message : String(e));
  }
}

async function hideToTray() {
  try {
    await invoke("hide_main");
    showToast("已隐藏到系统托盘，点击托盘图标可恢复");
  } catch (e) {
    showToast(e instanceof Error ? e.message : String(e));
  }
}
</script>

<template>
  <div class="win-bar" @mousedown="onDrag">
    <AppLogo class="win-logo" :size="22" decorative />
    <span class="win-name">灵镜 LINGJING</span>
    <span class="win-sub">动态壁纸</span>
    <span class="win-spacer" />
    <span
      v-if="onlineEnabled"
      class="win-act login"
      :class="{ 'is-logged-in': loggedIn }"
      role="button"
      tabindex="0"
      :aria-label="loggedIn ? `已登录：${userLabel}` : '登录灵境社区'"
      @mousedown.stop
      @click.stop="emit('login')"
      @keydown="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); emit('login'); } }"
    >{{ loggedIn ? userLabel : "登录" }}</span>
    <span
      class="win-act"
      role="button"
      tabindex="0"
      aria-label="最小化"
      title="最小化"
      @mousedown.stop
      @click.stop="minimize"
    >—</span>
    <span
      class="win-act close"
      role="button"
      tabindex="0"
      aria-label="隐藏到托盘"
      title="隐藏到托盘"
      @mousedown.stop
      @click.stop="hideToTray"
    >✕</span>
  </div>
</template>
