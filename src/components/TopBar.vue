<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { inject, ref, type Ref } from "vue";

const emit = defineEmits<{ (e: "theme"): void }>();

const sort = inject<Ref<string>>("topbarSort", ref("最热"));

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
    class="topbar flex h-12 flex-shrink-0 items-center gap-3 border-b border-[var(--border)] bg-[var(--surface)] px-5"
    @mousedown="onDrag"
  >
    <div class="actions flex items-center gap-2" @mousedown.stop>
      <span class="text-sm font-semibold tracking-wide text-[var(--text)]">灵镜 LINGJING</span>
      <span class="text-[11px] text-[var(--text-2)]">动态壁纸</span>
    </div>

    <div class="actions search ml-2 flex-1 max-w-md" @mousedown.stop>
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <circle cx="11" cy="11" r="7" />
        <path d="M21 21l-4.3-4.3" />
      </svg>
      <span class="hint">大家都在搜：</span>
      <span class="kw">极光</span>
      <span class="hint">· 二次元 · 赛博城市</span>
    </div>

    <div class="sort" role="tablist" @mousedown.stop>
      <span
        v-for="s in ['最热', '最新']"
        :key="s"
        :class="sort === s ? 'on' : ''"
        role="tab"
        tabindex="0"
        :aria-selected="sort === s"
        @click="sort = s"
        @keydown="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); sort = s; } }"
      >{{ s }}</span>
    </div>

    <div class="actions ml-auto flex items-center gap-1" @mousedown.stop>
      <button
        class="win-btn"
        title="切换主题"
        aria-label="切换主题"
        @click="emit('theme')"
      >🌗</button>
      <button class="win-btn" title="最小化" aria-label="最小化" @click="minimize">—</button>
      <button class="win-btn" title="最大化" aria-label="最大化" @click="toggleMaximize">▢</button>
      <button class="win-btn close" title="关闭" aria-label="关闭" @click="close">✕</button>
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

/* 搜索框：pill 圆角 + 关键词提示 */
.search {
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: var(--r-pill);
  padding: 8px 14px;
  font-size: 13px;
  color: var(--text-3);
  transition: border-color var(--dur-fast) var(--ease),
    box-shadow var(--dur-fast) var(--ease),
    background var(--dur-base) var(--ease);
}
.search:focus-within {
  border-color: var(--primary);
  box-shadow: 0 0 0 3px var(--primary-soft);
}
.search svg {
  width: 16px;
  height: 16px;
  stroke: var(--text-3);
  fill: none;
  stroke-width: 1.8;
}
.search .hint {
  color: var(--text-3);
}
.search .kw {
  color: var(--primary);
  font-weight: 600;
}

/* 排序：独立 pill 胶囊组 */
.sort {
  display: flex;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: var(--r-pill);
  padding: 3px;
}
.sort span {
  font-size: 12.5px;
  padding: 5px 14px;
  border-radius: var(--r-pill);
  color: var(--text-2);
  cursor: pointer;
  transition: background var(--dur-fast) var(--ease),
    color var(--dur-fast) var(--ease),
    transform var(--dur-fast) var(--ease);
}
.sort span:hover {
  color: var(--text);
}
.sort span:active {
  transform: scale(0.94);
}
.sort span.on {
  background: var(--surface);
  color: var(--text);
  font-weight: 600;
  box-shadow: var(--sh-sm);
}

.win-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 28px;
  border-radius: 8px;
  font-size: 13px;
  color: var(--text-2);
  transition: background 0.15s, color 0.15s;
}
.win-btn:hover {
  background: rgba(31, 35, 41, 0.08);
  color: var(--text);
}
.win-btn.close:hover {
  background: var(--error);
  color: #fff;
}
</style>
