<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { ref } from "vue";

const emit = defineEmits<{ (e: "theme"): void }>();

const search = ref("");

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
  <header
    class="topbar flex h-12 flex-shrink-0 items-center gap-3 border-b border-[var(--border)] bg-[var(--bg-elevated)] px-4"
    @mousedown="onDrag"
  >
    <div class="flex items-center gap-2">
      <span class="text-sm font-semibold tracking-wide text-[var(--text)]">灵境 LingScape</span>
      <span class="text-[11px] text-[var(--text-dim)]">动态壁纸</span>
    </div>

    <div class="actions relative ml-2 flex-1 max-w-md" @mousedown.stop>
      <span class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-[var(--text-dim)]">🔍</span>
      <input
        v-model="search"
        type="text"
        placeholder="搜索壁纸、标签或作者…"
        class="w-full rounded-xl border border-[var(--border)] bg-[var(--bg)] py-1.5 pl-9 pr-3 text-[13px] text-[var(--text)] outline-none transition-colors duration-[var(--dur-fast)] focus:border-[var(--primary)]"
      />
    </div>

    <div class="actions ml-auto flex items-center gap-1" @mousedown.stop>
      <button
        class="win-btn"
        title="切换主题"
        @click="emit('theme')"
      >
        🌗
      </button>
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
  background: rgba(31, 35, 41, 0.08);
  color: var(--text);
}
.win-btn.close:hover {
  background: #e5484d;
  color: #fff;
}
</style>
