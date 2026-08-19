<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import type { EngineState } from "../composables/useEngine";

const props = defineProps<{
  engine: EngineState | null;
}>();

const stage = ref<HTMLDivElement | null>(null);
const videoEl = ref<HTMLVideoElement | null>(null);
const imgEl = ref<HTMLImageElement | null>(null);

let videoBlobUrl: string | null = null;

function isVideo(s: EngineState | null) {
  if (!s?.uri) return false;
  if ((s.mediaType || "").toLowerCase() === "video") return true;
  return /\.(mp4|webm)(\?|$)/i.test(s.uri);
}

function clearVideo() {
  const v = videoEl.value;
  if (v) {
    v.pause();
    v.removeAttribute("src");
    v.load();
  }
  if (imgEl.value) imgEl.value.removeAttribute("src");
  if (videoBlobUrl) {
    URL.revokeObjectURL(videoBlobUrl);
    videoBlobUrl = null;
  }
}

async function apply(s: EngineState) {
  const root = stage.value;
  if (!root) return;
  const uri = s.uri || "";
  if (!uri) {
    clearVideo();
    return;
  }
  if (isVideo(s)) {
    if (imgEl.value) imgEl.value.removeAttribute("src");
    const v = videoEl.value;
    if (v) {
      v.muted = !!s.muted;
      v.volume = typeof s.volume === "number" ? s.volume : 0.8;
      v.loop = false;
      v.src = uri;
      const p = v.play();
      if (p && p.catch) p.catch(() => {});
    }
  } else {
    clearVideo();
    if (imgEl.value) {
      imgEl.value.onerror = () => console.error("[wp-stage] image error", uri);
      imgEl.value.src = uri;
    }
  }
}

function applyPlaying(s: EngineState) {
  if (!isVideo(s)) return;
  const v = videoEl.value;
  if (!v) return;
  if (s.playing) {
    const p = v.play();
    if (p && p.catch) p.catch(() => {});
  } else {
    v.pause();
  }
}

function applyVolume(s: EngineState) {
  if (!videoEl.value) return;
  videoEl.value.muted = !!s.muted;
  videoEl.value.volume = typeof s.volume === "number" ? s.volume : 0.8;
}

let lastUri: string | null = null;
let lastPlay: boolean | null = null;
let lastMuted: boolean | null = null;
let lastVol: number | null = null;

function diffApply(s: EngineState) {
  if ((s.uri || null) !== lastUri) {
    apply(s);
    lastUri = s.uri || null;
  }
  if (s.playing !== lastPlay) {
    applyPlaying(s);
    lastPlay = s.playing;
  }
  if (s.muted !== lastMuted || s.volume !== lastVol) {
    applyVolume(s);
    lastMuted = s.muted;
    lastVol = s.volume;
  }
}

let raf = 0;
function tick() {
  if (props.engine) diffApply(props.engine);
  raf = requestAnimationFrame(tick);
}

onMounted(() => {
  tick();
});
onUnmounted(() => {
  cancelAnimationFrame(raf);
  clearVideo();
});
</script>

<template>
  <div ref="stage" class="wp-stage" aria-hidden="true">
    <video
      ref="videoEl"
      playsinline
      autoplay
      style="display: block"
    ></video>
    <img ref="imgEl" alt="" />
    <div class="wp-overlay"></div>
  </div>
</template>
