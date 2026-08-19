<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { showToast } from "../composables/useToast";
import MediaThumb from "./MediaThumb.vue";
import type { WallpaperItem } from "../data/catalog";
import type { EngineState } from "../composables/useEngine";

const props = defineProps<{
  current: WallpaperItem | null;
  engine: EngineState | null;
  queue: WallpaperItem[];
  loopMode: LoopMode;
}>();
const emit = defineEmits<{
  (e: "import"): void;
  (e: "open-detail"): void;
  (e: "play"): void;
  (e: "pause"): void;
  (e: "prev"): void;
  (e: "next"): void;
  (e: "volume", volume: number, muted: boolean): void;
  (e: "loop", mode: LoopMode): void;
}>();

export type LoopMode = "list" | "single" | "random";

const volOpen = ref(false);
const volume = ref(0.8);
const muted = ref(false);

const playing = computed(() => props.engine?.playing ?? false);
const progress = computed(() => {
  const d = props.engine?.duration ?? 0;
  const t = props.engine?.currentTime ?? 0;
  if (d > 0) return Math.min(1, Math.max(0, t / d));
  // proxy for no-duration media
  return playing.value ? 1 : 0;
});
const hasDuration = computed(() => (props.engine?.duration ?? 0) > 0);

const loopLabel = computed(() => {
  if (props.loopMode === "single") return "🔂";
  if (props.loopMode === "random") return "🔀";
  return "🔁";
});

watch(
  () => props.engine,
  (s) => {
    if (!s) return;
    volume.value = s.volume;
    muted.value = s.muted;
  },
);

function togglePlay() {
  if (!props.current) return;
  if (playing.value) emit("pause");
  else emit("play");
}

function cycleLoop() {
  const order: LoopMode[] = ["list", "single", "random"];
  const i = order.indexOf(props.loopMode);
  const next = order[(i + 1) % order.length]!;
  emit("loop", next);
  showToast(next === "list" ? "列表循环" : next === "single" ? "单曲循环" : "随机播放");
}

function onVolInput(e: Event) {
  const v = Number((e.target as HTMLInputElement).value);
  volume.value = v;
  muted.value = v <= 0;
  emit("volume", volume.value, muted.value);
}

function toggleMute() {
  muted.value = !muted.value;
  emit("volume", volume.value, muted.value);
}

function doImport() {
  emit("import");
}
</script>

<template>
  <div class="playbar">
    <div
      class="pb-thumb"
      role="button"
      tabindex="0"
      aria-label="打开详情"
      :style="{ cursor: current ? 'pointer' : 'default' }"
      @click="current && emit('open-detail')"
      @keydown.enter="current && emit('open-detail')"
    >
      <MediaThumb v-if="current" :item="current" />
    </div>
    <div class="pb-btns">
      <div class="pb-btn" role="button" tabindex="0" aria-label="上一个" @click="emit('prev')">⟨</div>
      <div
        class="pb-btn play"
        role="button"
        tabindex="0"
        aria-label="播放或暂停"
        @click="togglePlay"
      >{{ playing ? "❚❚" : "▶" }}</div>
      <div class="pb-btn" role="button" tabindex="0" aria-label="下一个" @click="emit('next')">⟩</div>
    </div>
    <div class="pb-now">
      <div
        class="t"
        role="button"
        tabindex="0"
        style="cursor: pointer;"
        @click="current && emit('open-detail')"
      >{{ current ? `${current.name} · 正在播放` : "未选择壁纸" }}</div>
      <div class="bar">
        <i
          :class="{ paused: !playing, proxy: !hasDuration }"
          :style="hasDuration ? { transform: `scaleX(${progress})`, animation: 'none' } : undefined"
        />
      </div>
    </div>
    <div class="pb-right" style="position: relative;">
      <span class="pill" role="button" tabindex="0" aria-label="音量" @click="volOpen = !volOpen">{{ muted || volume <= 0 ? "🔇" : "🔊" }}</span>
      <div
        v-if="volOpen"
        style="position:absolute;bottom:44px;right:70px;padding:10px 12px;border:1px solid var(--border);border-radius:12px;background:var(--surface);box-shadow:var(--sh-md);display:flex;align-items:center;gap:8px;z-index:20;"
      >
        <button type="button" class="pill" style="cursor:pointer;" @click="toggleMute">{{ muted ? "取消静音" : "静音" }}</button>
        <input
          type="range"
          min="0"
          max="1"
          step="0.01"
          :value="volume"
          aria-label="音量滑块"
          style="width:100px;"
          @input="onVolInput"
        />
      </div>
      <span class="pill" role="button" tabindex="0" aria-label="循环" @click="cycleLoop">{{ loopLabel }}</span>
      <span class="pill imp" role="button" tabindex="0" aria-label="导入壁纸" @click="doImport">＋ 导入</span>
    </div>
  </div>
</template>
