<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import AppLogo from "./AppLogo.vue";

defineProps<{
  onlineEnabled?: boolean;
  loggedIn?: boolean;
  userLabel?: string;
}>();

const emit = defineEmits<{ (e: "login"): void }>();

async function onDrag(e: MouseEvent) {
  if (e.button !== 0) return;
  const target = e.target as HTMLElement | null;
  // no-drag 区域仍会冒泡到 win-bar；跳过按钮，否则 start_drag 会吞掉 click
  if (target?.closest(".win-act, .app-logo")) return;
  await invoke("start_drag");
}
function minimize() {
  getCurrentWindow().minimize();
}
function toggleMaximize() {
  getCurrentWindow().toggleMaximize();
}
function close() {
  getCurrentWindow().close();
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
    <span class="win-act" role="button" tabindex="0" aria-label="最小化" title="最小化" @mousedown.stop @click="minimize">—</span>
    <span class="win-act" role="button" tabindex="0" aria-label="最大化" title="最大化" @mousedown.stop @click="toggleMaximize">▢</span>
    <span class="win-act close" role="button" tabindex="0" aria-label="关闭" title="关闭" @mousedown.stop @click="close">✕</span>
  </div>
</template>
