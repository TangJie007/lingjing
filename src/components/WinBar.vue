<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";

const emit = defineEmits<{ (e: "theme"): void }>();

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
      class="win-act theme"
      role="button"
      tabindex="0"
      aria-label="切换浅色或深色主题"
      @click="emit('theme')"
      @keydown="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); emit('theme'); } }"
    >◐ 主题</span>
    <span class="win-act" role="button" tabindex="0" aria-label="最小化" title="最小化" @click="minimize">—</span>
    <span class="win-act" role="button" tabindex="0" aria-label="最大化" title="最大化" @click="toggleMaximize">▢</span>
    <span class="win-act close" role="button" tabindex="0" aria-label="关闭" title="关闭" @click="close">✕</span>
  </div>
</template>
