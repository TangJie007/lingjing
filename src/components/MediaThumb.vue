<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { WallpaperItem } from "../data/catalog";
import { resolveMediaUri } from "../composables/useEngine";
import {
  cacheVideoPoster,
  captureVideoFrame,
  getCachedVideoPoster,
  videoPosterKey,
} from "../composables/useVideoPoster";

const props = withDefaults(
  defineProps<{ item: WallpaperItem; mode?: "thumb" | "preview" }>(),
  { mode: "thumb" },
);

const uri = computed(() => {
  if (props.item.missing || !props.item.mediaSrc) return null;
  return resolveMediaUri(props.item);
});

const isPreview = computed(() => props.mode === "preview");
const showVideo = computed(() => props.item.type === "video" && !!uri.value);
const showImage = computed(
  () => (props.item.type === "image" || props.item.type === "gif") && !!uri.value,
);
const posterKey = computed(() => videoPosterKey(props.item));

const mediaReady = ref(false);
const posterUrl = ref<string | null>(null);
const videoRef = ref<HTMLVideoElement | null>(null);
let seekPending = false;

function applyCachedPoster() {
  const cached = getCachedVideoPoster(posterKey.value);
  if (!cached) return false;
  posterUrl.value = cached;
  mediaReady.value = true;
  return true;
}

watch(
  () => [posterKey.value, props.mode, showVideo.value] as const,
  () => {
    seekPending = false;
    if (isPreview.value || !showVideo.value) {
      mediaReady.value = false;
      posterUrl.value = null;
      return;
    }
    // 命中 base64 缓存则直接用，避免列表刷新时闪空再截帧
    if (applyCachedPoster()) return;
    mediaReady.value = false;
    posterUrl.value = null;
  },
  { immediate: true },
);

function seekThumbFrame() {
  if (seekPending || posterUrl.value) return;
  const v = videoRef.value;
  if (!v) return;
  seekPending = true;
  const duration = v.duration;
  if (Number.isFinite(duration) && duration > 0) {
    v.currentTime = Math.min(1, Math.max(0.05, duration * 0.05));
  } else {
    v.currentTime = 0.1;
  }
}

function onVideoMeta() {
  if (isPreview.value || posterUrl.value) return;
  seekThumbFrame();
}

function onThumbSeeked() {
  if (isPreview.value) return;
  seekPending = false;
  mediaReady.value = true;
  if (posterUrl.value) return;
  const v = videoRef.value;
  if (!v) return;
  const dataUrl = captureVideoFrame(v);
  if (dataUrl) {
    cacheVideoPoster(posterKey.value, dataUrl);
    posterUrl.value = dataUrl;
  }
}

function onPreviewCanPlay() {
  if (!isPreview.value) return;
  const v = videoRef.value;
  if (!v) return;
  mediaReady.value = true;
  const p = v.play();
  if (p?.catch) {
    p.catch((e: unknown) => {
      const msg = e instanceof Error ? e.message : String(e);
      if (!/aborterror|interrupted by a new load/i.test(msg)) {
        console.warn("[MediaThumb] preview play failed", e);
      }
    });
  }
}

function onImgLoad() {
  mediaReady.value = true;
}
</script>

<template>
  <div
    class="thumb-bg"
    :class="{
      'has-media': mediaReady,
      'is-video-preview': isPreview && showVideo,
    }"
    :style="{ background: item.thumb }"
  >
    <video
      v-if="showVideo && (isPreview || !posterUrl)"
      ref="videoRef"
      class="thumb-media thumb-video"
      :src="uri!"
      muted
      playsinline
      :loop="isPreview"
      :preload="isPreview ? 'auto' : 'metadata'"
      disablePictureInPicture
      disableremoteplayback
      controlslist="nodownload noplaybackrate noremoteplayback"
      @loadedmetadata="onVideoMeta"
      @seeked="onThumbSeeked"
      @canplay="onPreviewCanPlay"
      @error="seekPending = false"
    />
    <img
      v-else-if="showVideo && posterUrl"
      class="thumb-media"
      :src="posterUrl"
      alt=""
      @load="onImgLoad"
    />
    <img
      v-else-if="showImage"
      class="thumb-media"
      :src="uri!"
      alt=""
      :loading="isPreview ? 'eager' : 'lazy'"
      @load="onImgLoad"
    />
  </div>
</template>

<style scoped>
.thumb-media {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
  opacity: 0;
  transition: opacity var(--dur-base) var(--ease);
  pointer-events: none;
}
.thumb-bg.has-media .thumb-media {
  opacity: 1;
}
.thumb-bg.is-video-preview .thumb-media {
  transform: translateZ(0);
  backface-visibility: hidden;
}
.thumb-video {
  -webkit-appearance: none;
  appearance: none;
}
.thumb-video::-webkit-media-controls {
  display: none !important;
}
.thumb-video::-webkit-media-controls-enclosure {
  display: none !important;
}
.thumb-video::-webkit-media-controls-start-playback-button {
  display: none !important;
  -webkit-appearance: none;
  opacity: 0;
  pointer-events: none;
}
.thumb-bg:not(.has-media) .thumb-video {
  visibility: hidden;
}
</style>
