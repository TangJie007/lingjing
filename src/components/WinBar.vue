<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";

defineProps<{
  onlineEnabled?: boolean;
  loggedIn?: boolean;
  userLabel?: string;
}>();

const emit = defineEmits<{ (e: "login"): void }>();

async function onDrag(e: MouseEvent) {
  if (e.button === 0) await invoke("start_drag");
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
    <div class="win-dots" aria-hidden="true">
      <span class="a" />
      <span />
      <span />
    </div>
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
      @click="emit('login')"
      @keydown="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); emit('login'); } }"
    >{{ loggedIn ? userLabel : "登录" }}</span>
    <span class="win-act" role="button" tabindex="0" aria-label="最小化" title="最小化" @click="minimize">—</span>
    <span class="win-act" role="button" tabindex="0" aria-label="最大化" title="最大化" @click="toggleMaximize">▢</span>
    <span class="win-act close" role="button" tabindex="0" aria-label="关闭" title="关闭" @click="close">✕</span>
  </div>
</template>
