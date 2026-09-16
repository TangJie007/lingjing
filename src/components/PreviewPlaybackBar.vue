<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from "vue";

const props = defineProps<{
  video: HTMLVideoElement | null;
}>();

const playing = ref(false);
const current = ref(0);
const duration = ref(0);
let raf = 0;

const progress = computed(() =>
  duration.value > 0 ? Math.min(1, Math.max(0, current.value / duration.value)) : 0,
);

function syncFromVideo() {
  const v = props.video;
  if (!v) return;
  playing.value = !v.paused && !v.ended;
  current.value = v.currentTime || 0;
  duration.value = Number.isFinite(v.duration) ? v.duration : 0;
}

function tick() {
  syncFromVideo();
  raf = requestAnimationFrame(tick);
}

function startTick() {
  cancelAnimationFrame(raf);
  raf = requestAnimationFrame(tick);
}

function stopTick() {
  cancelAnimationFrame(raf);
}

function togglePlay() {
  const v = props.video;
  if (!v) return;
  if (v.paused || v.ended) {
    void v.play().catch(() => undefined);
  } else {
    v.pause();
  }
  syncFromVideo();
}

function onSeek(e: Event) {
  const v = props.video;
  if (!v || duration.value <= 0) return;
  const pct = Number((e.target as HTMLInputElement).value) / 100;
  v.currentTime = pct * duration.value;
  syncFromVideo();
}

function fmt(sec: number) {
  if (!Number.isFinite(sec) || sec < 0) return "0:00";
  const m = Math.floor(sec / 60);
  const s = Math.floor(sec % 60);
  return `${m}:${String(s).padStart(2, "0")}`;
}

watch(
  () => props.video,
  (v, prev) => {
    stopTick();
    if (prev) {
      prev.removeEventListener("play", syncFromVideo);
      prev.removeEventListener("pause", syncFromVideo);
      prev.removeEventListener("timeupdate", syncFromVideo);
      prev.removeEventListener("loadedmetadata", syncFromVideo);
      prev.removeEventListener("ended", syncFromVideo);
    }
    if (!v) {
      playing.value = false;
      current.value = 0;
      duration.value = 0;
      return;
    }
    v.addEventListener("play", syncFromVideo);
    v.addEventListener("pause", syncFromVideo);
    v.addEventListener("timeupdate", syncFromVideo);
    v.addEventListener("loadedmetadata", syncFromVideo);
    v.addEventListener("ended", syncFromVideo);
    syncFromVideo();
    startTick();
  },
  { immediate: true },
);

onUnmounted(stopTick);
</script>

<template>
  <div class="preview-playback">
    <button type="button" class="pp-btn" aria-label="播放或暂停" @click="togglePlay">
      {{ playing ? "❚❚" : "▶" }}
    </button>
    <span class="pp-time">{{ fmt(current) }}</span>
    <div class="pp-track">
      <div class="pp-fill" :style="{ width: `${progress * 100}%` }" />
      <input
        type="range"
        min="0"
        max="100"
        step="0.1"
        :value="progress * 100"
        aria-label="预览进度"
        @input="onSeek"
      />
    </div>
    <span class="pp-time">{{ fmt(duration) }}</span>
  </div>
</template>

<style scoped>
.preview-playback {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  background: linear-gradient(to top, rgba(17, 24, 39, 0.72), rgba(17, 24, 39, 0.35));
  backdrop-filter: blur(6px);
  -webkit-backdrop-filter: blur(6px);
  border-bottom-left-radius: var(--r-md, 12px);
  border-bottom-right-radius: var(--r-md, 12px);
}
.pp-btn {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  border: none;
  background: rgba(255, 255, 255, 0.92);
  color: #1f2329;
  font-size: 12px;
  cursor: pointer;
  flex-shrink: 0;
  display: grid;
  place-items: center;
}
.pp-btn:hover {
  background: #fff;
}
.pp-time {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.88);
  font-variant-numeric: tabular-nums;
  flex-shrink: 0;
  min-width: 32px;
}
.pp-track {
  flex: 1;
  position: relative;
  height: 4px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.28);
}
.pp-fill {
  position: absolute;
  inset: 0 auto 0 0;
  background: #fff;
  border-radius: 4px;
  pointer-events: none;
}
.pp-track input[type="range"] {
  position: absolute;
  inset: -8px 0;
  width: 100%;
  height: 20px;
  opacity: 0;
  cursor: pointer;
  margin: 0;
}
</style>
