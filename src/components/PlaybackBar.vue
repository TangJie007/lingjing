<script setup lang="ts">
import { ref } from "vue";
import { showToast } from "../composables/useToast";
import type { WallpaperItem } from "../data/catalog";

defineProps<{ current: WallpaperItem | null }>();
const emit = defineEmits<{ (e: "import"): void }>();

const playing = ref(false);
const volume = ref(70);
const muted = ref(false);
const loop = ref<"list" | "single" | "random">("list");

function togglePlay() {
  playing.value = !playing.value;
}
function cycleLoop() {
  loop.value = loop.value === "list" ? "single" : loop.value === "single" ? "random" : "list";
}
const loopIcon = () =>
  loop.value === "single" ? "🔂" : loop.value === "random" ? "🔀" : "🔁";

function doImport() {
  emit("import");
  showToast("导入面板开发中");
}
</script>

<template>
  <footer
    class="playbar flex h-14 flex-shrink-0 items-center gap-3 border-t border-[var(--border)] bg-[var(--surface-2)] px-5"
  >
    <div class="flex w-44 items-center gap-2 overflow-hidden">
      <div
        v-if="current"
        class="pb-thumb h-8 w-[46px] flex-shrink-0 rounded-md"
        :style="{ background: current.thumb }"
      />
      <div class="min-w-0">
        <div class="t truncate text-[13px] font-semibold text-[var(--text)]">
          {{ current ? `${current.name} · 正在播放` : "未选择壁纸" }}
        </div>
        <div class="m text-[10px] text-[var(--text-3)]">
          {{ current ? current.type.toUpperCase() : "—" }}
        </div>
      </div>
    </div>

    <div class="flex items-center gap-1.5">
      <button class="pb-btn" title="上一个" aria-label="上一个">⟨</button>
      <button
        class="pb-btn play"
        :title="playing ? '暂停' : '播放'"
        aria-label="播放或暂停"
        @click="togglePlay"
      >{{ playing ? "❚❚" : "▶" }}</button>
      <button class="pb-btn" title="下一个" aria-label="下一个">⟩</button>
    </div>

    <div class="flex-1 min-w-0">
      <div class="bar h-1 w-full overflow-hidden rounded-full bg-[var(--border)]">
        <i
          class="block h-full w-full origin-left rounded-full bg-[var(--primary)]"
          :class="{ paused: !playing }"
        />
      </div>
    </div>

    <div class="flex items-center gap-1.5">
      <button
        class="pb-btn"
        :title="muted ? '取消静音' : '静音'"
        aria-label="静音"
        @click="muted = !muted"
      >{{ muted ? "🔇" : "🔊" }}</button>
      <input
        type="range"
        min="0"
        max="100"
        :value="volume"
        class="h-1 w-20 cursor-pointer appearance-none rounded-full bg-[var(--border)]"
        :aria-label="`音量 ${volume}`"
        @input="(e) => (volume = +(e.target as HTMLInputElement).value)"
      />
      <button
        class="pb-btn"
        :title="`循环：${loop}`"
        aria-label="切换循环模式"
        @click="cycleLoop"
      >{{ loopIcon() }}</button>
    </div>

    <button
      class="pill imp ml-2"
      title="导入壁纸"
      aria-label="导入壁纸"
      @click="doImport"
    >＋ 导入</button>
  </footer>
</template>

<style scoped>
.playbar {
  app-region: no-drag;
}

.pb-thumb {
  background-size: cover;
}

.t {
  font-size: 13px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.bar i {
  transform-origin: left;
  animation: prog 18s linear infinite;
}
.bar i.paused {
  animation-play-state: paused;
}

.pb-btn {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border);
  background: var(--surface);
  cursor: pointer;
  color: var(--text-2);
  font-size: 13px;
  transition: transform var(--dur-fast) var(--ease),
    background var(--dur-fast) var(--ease),
    color var(--dur-fast) var(--ease),
    border-color var(--dur-fast) var(--ease);
}
.pb-btn:hover {
  color: var(--text);
  border-color: var(--border-strong);
  transform: scale(1.08);
}
.pb-btn:active {
  transform: scale(0.9);
}
.pb-btn.play {
  background: var(--primary);
  color: #fff;
  border-color: var(--primary);
  width: 38px;
  height: 38px;
}

.pill {
  font-size: 12px;
  padding: 6px 12px;
  border-radius: var(--r-pill);
  border: 1px solid var(--border);
  background: var(--surface);
  color: var(--text-2);
  cursor: pointer;
  transition: transform var(--dur-fast) var(--ease),
    color var(--dur-fast) var(--ease),
    border-color var(--dur-fast) var(--ease);
}
.pill:hover {
  border-color: var(--border-strong);
  color: var(--text);
  transform: translateY(-1px);
}
.pill:active {
  transform: scale(0.94);
}
.pill.imp {
  color: var(--primary);
  border-color: var(--primary-soft);
  background: var(--primary-soft);
}

input[type="range"]::-webkit-slider-thumb {
  appearance: none;
  height: 12px;
  width: 12px;
  border-radius: 999px;
  background: var(--primary);
}
</style>
