<script setup lang="ts">
import { computed, ref, watch } from "vue";
import VueSkeletonLoader from "vue3-skeleton-loader";
import "vue3-skeleton-loader/dist/index.css";
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

const urlThumb = computed(() => {
  const t = props.item.thumb?.trim();
  if (!t) return null;
  if (/^(https?:|data:|blob:|asset:)/i.test(t)) return t;
  return null;
});

const isPreview = computed(() => props.mode === "preview");
/** Card list: prefer remote thumbnailUrl over decoding the full video. */
const showRemoteThumb = computed(() => !isPreview.value && !!urlThumb.value);
const showVideo = computed(
  () => props.item.type === "video" && !!uri.value && !showRemoteThumb.value,
);
const showImage = computed(() => {
  if (showRemoteThumb.value) return true;
  return (props.item.type === "image" || props.item.type === "gif") && !!uri.value;
});
const imageSrc = computed(() => {
  if (showRemoteThumb.value) return urlThumb.value;
  return uri.value;
});
const posterKey = computed(() => videoPosterKey(props.item));

const mediaReady = ref(false);
const posterUrl = ref<string | null>(null);
const videoRef = ref<HTMLVideoElement | null>(null);
const isLoading = computed(() => {
  if (showRemoteThumb.value) return !mediaReady.value;
  return !!uri.value && !mediaReady.value;
});
let seekPending = false;

function applyCachedPoster() {
  const cached = getCachedVideoPoster(posterKey.value);
  if (!cached) return false;
  posterUrl.value = cached;
  mediaReady.value = true;
  return true;
}

watch(
  () => [posterKey.value, props.mode, showVideo.value, showRemoteThumb.value, urlThumb.value] as const,
  () => {
    seekPending = false;
    if (showRemoteThumb.value) {
      mediaReady.value = false;
      posterUrl.value = null;
      return;
    }
    if (isPreview.value || !showVideo.value) {
      mediaReady.value = false;
      posterUrl.value = null;
      return;
    }
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
      'is-loading': isLoading,
      'is-video-preview': isPreview && showVideo,
    }"
    :style="
      isLoading || urlThumb
        ? undefined
        : { background: item.thumb }
    "
  >
    <VueSkeletonLoader
      v-if="isLoading"
      class="thumb-skeleton"
      type="image"
      animation="wave"
      width="100%"
      height="100%"
      border-radius="0"
      base-color="#E8EAEF"
      highlight-color="#F7F8FA"
      duration="1.4s"
    />
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
      v-else-if="showImage && imageSrc"
      class="thumb-media"
      :src="imageSrc"
      alt=""
      :loading="isPreview ? 'eager' : 'lazy'"
      @load="onImgLoad"
      @error="mediaReady = true"
    />
  </div>
</template>

<style scoped>
.thumb-bg {
  position: absolute;
  inset: 0;
  overflow: hidden;
}
.thumb-skeleton {
  position: absolute !important;
  inset: 0;
  width: 100% !important;
  height: 100% !important;
  display: block;
}
.thumb-skeleton :deep(.vue-skeleton-loader-bone),
.thumb-skeleton :deep(.v-skeleton-loader-image) {
  width: 100% !important;
  height: 100% !important;
  margin: 0 !important;
  border-radius: 0 !important;
}
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
  z-index: 1;
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
