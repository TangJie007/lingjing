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
  <header class="topbar" @mousedown="onDrag">
    <div class="brand">
      <span class="dot" />
      <span class="title">灵境 LingScape</span>
      <span class="subtitle">动态壁纸</span>
    </div>
    <div class="actions" @mousedown.stop>
      <button class="win-btn" title="最小化" @click="minimize">—</button>
      <button class="win-btn" title="最大化" @click="toggleMaximize">▢</button>
      <button class="win-btn close" title="关闭" @click="close">✕</button>
    </div>
  </header>
</template>

<style scoped>
.topbar {
  height: 44px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px 0 16px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-elevated);
  app-region: drag;
}

.brand {
  display: flex;
  align-items: center;
  gap: 8px;
}

.dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: linear-gradient(135deg, #7c5cff, #4ad6ff);
  box-shadow: 0 0 10px rgba(124, 92, 255, 0.7);
}

.title {
  font-weight: 600;
  font-size: 14px;
  letter-spacing: 0.2px;
}

.subtitle {
  font-size: 11px;
  color: var(--text-dim);
}

.actions {
  display: flex;
  gap: 4px;
  app-region: no-drag;
}

.win-btn {
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
