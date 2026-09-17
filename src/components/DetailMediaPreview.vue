<script setup lang="ts">
import { computed, ref, watch } from "vue";
import VueSkeletonLoader from "vue3-skeleton-loader";
import "vue3-skeleton-loader/dist/index.css";
import type { WallpaperItem } from "../data/catalog";
import { resolveMediaUri } from "../composables/useEngine";
import PreviewPlaybackBar from "./PreviewPlaybackBar.vue";

const props = withDefaults(
  defineProps<{ item: WallpaperItem; fill?: boolean }>(),
  { fill: false },
);

const videoRef = ref<HTMLVideoElement | null>(null);
const mediaRatio = ref(16 / 9);
const mediaReady = ref(false);

const uri = computed(() => {
  if (props.item.missing || !props.item.mediaSrc) return null;
  return resolveMediaUri(props.item);
});

const isVideo = computed(() => props.item.type === "video" && !!uri.value);
const isImage = computed(
  () => (props.item.type === "image" || props.item.type === "gif") && !!uri.value,
);
const isLoading = computed(() => !!uri.value && !mediaReady.value);

const frameStyle = computed(() => {
  if (props.fill) return {};
  const r = mediaRatio.value > 0 ? mediaRatio.value : 16 / 9;
  return {
    aspectRatio: String(r),
    width: `min(100%, calc(min(56vh, 520px) * ${r}))`,
  };
});

function applyRatio(w: number, h: number) {
  if (w > 0 && h > 0) mediaRatio.value = w / h;
}

function markReady() {
  mediaReady.value = true;
}

function onCanPlay() {
  const v = videoRef.value;
  if (!v) return;
  applyRatio(v.videoWidth, v.videoHeight);
  markReady();
  void v.play().catch(() => undefined);
}

function onLoadedMeta() {
  const v = videoRef.value;
  if (!v) return;
  applyRatio(v.videoWidth, v.videoHeight);
}

function onImgLoad(e: Event) {
  const img = e.target as HTMLImageElement;
  applyRatio(img.naturalWidth, img.naturalHeight);
  markReady();
}

watch(
  () => [props.item.id, uri.value] as const,
  () => {
    mediaRatio.value = 16 / 9;
    mediaReady.value = !uri.value;
  },
  { immediate: true },
);
</script>

<template>
  <div class="detail-media" :class="{ fill }">
    <div
      class="detail-media-bg"
      :class="{ 'is-video': isVideo, 'is-loading': isLoading }"
      :style="[{ background: isLoading ? undefined : '' }, frameStyle]"
    >
      <VueSkeletonLoader
        v-if="isLoading"
        class="detail-skeleton"
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
        v-if="isVideo"
        ref="videoRef"
        class="detail-media-el"
        :class="{ ready: mediaReady }"
        :src="uri!"
        muted
        playsinline
        loop
        preload="auto"
        @loadedmetadata="onLoadedMeta"
        @canplay="onCanPlay"
        @error="markReady"
      />
      <img
        v-else-if="isImage"
        class="detail-media-el"
        :class="{ ready: mediaReady }"
        :src="uri!"
        alt=""
        @load="onImgLoad"
        @error="markReady"
      />
      <PreviewPlaybackBar
        v-if="isVideo && mediaReady"
        :video="videoRef"
        class="detail-playback-overlay"
      />
    </div>
  </div>
</template>

<style scoped>
.detail-media {
  position: relative;
  width: 100%;
  display: flex;
  justify-content: center;
  border-radius: var(--r-md);
  overflow: hidden;
  isolation: isolate;
  background: #0e1014;
}
.detail-media.fill {
  flex: 1;
  min-width: 0;
  min-height: 0;
  height: 100%;
  align-self: stretch;
}
.detail-media.fill .detail-media-bg {
  width: 100%;
  height: 100%;
  max-width: none;
  max-height: none;
  aspect-ratio: auto;
}
.detail-media-bg {
  position: relative;
  max-width: 100%;
  max-height: min(56vh, 520px);
  overflow: hidden;
  border-radius: inherit;
  flex-shrink: 0;
}
.detail-media-bg.is-loading {
  background: #e8eaef;
}
.detail-skeleton {
  position: absolute !important;
  inset: 0;
  width: 100% !important;
  height: 100% !important;
  z-index: 1;
  display: block;
}
.detail-skeleton :deep(.vue-skeleton-loader-bone),
.detail-skeleton :deep(.v-skeleton-loader-image) {
  width: 100% !important;
  height: 100% !important;
  margin: 0 !important;
  border-radius: 0 !important;
}
.detail-playback-overlay {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 2;
  border-bottom-left-radius: inherit;
  border-bottom-right-radius: inherit;
  overflow: hidden;
}
.detail-media-el {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: contain;
  display: block;
  background: transparent;
  border-radius: inherit;
  opacity: 0;
  transition: opacity 180ms ease;
}
.detail-media-el.ready {
  opacity: 1;
}
.detail-media.fill .detail-media-el {
  object-fit: contain;
}
</style>
