<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";

async function onDrag(e: MouseEvent) {
  // 仅在左键按下空白处时启动系统拖拽
  if (e.button === 0) {
    await invoke("start_drag");
  }
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
  <header class="topbar flex h-11 flex-shrink-0 items-center justify-between border-b border-[var(--border)] bg-[var(--bg-elevated)] px-4 py-3" @mousedown="onDrag">
    <div class="flex items-center gap-2">
      <span class="h-2.5 w-2.5 rounded-full bg-gradient-to-br from-accent to-[#4ad6ff] shadow-[0_0_10px_rgba(124,92,255,0.7)]" />
      <span class="text-sm font-semibold tracking-wide">灵境 LingScape</span>
      <span class="text-[11px] text-[var(--text-dim)]">动态壁纸</span>
    </div>
    <div class="actions flex gap-1" @mousedown.stop>
      <button class="win-btn" title="最小化" @click="minimize">—</button>
      <button class="win-btn" title="最大化" @click="toggleMaximize">▢</button>
      <button class="win-btn close" title="关闭" @click="close">✕</button>
    </div>
  </header>
</template>

<style scoped>
.topbar {
  app-region: drag;
}
.actions {
  app-region: no-drag;
}
.win-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 28px;
  border-radius: 8px;
  font-size: 13px;
  color: var(--text-dim);
  transition: background 0.15s, color 0.15s;
}
.win-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: var(--text);
}
.win-btn.close:hover {
  background: #e5484d;
  color: #fff;
}
</style>
