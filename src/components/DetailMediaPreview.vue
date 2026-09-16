<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { WallpaperItem } from "../data/catalog";
import { resolveMediaUri } from "../composables/useEngine";
import PreviewPlaybackBar from "./PreviewPlaybackBar.vue";

const props = withDefaults(
  defineProps<{ item: WallpaperItem; fill?: boolean }>(),
  { fill: false },
);

const videoRef = ref<HTMLVideoElement | null>(null);
const mediaRatio = ref(16 / 9);

const uri = computed(() => {
  if (props.item.missing || !props.item.mediaSrc) return null;
  return resolveMediaUri(props.item);
});

const isVideo = computed(() => props.item.type === "video" && !!uri.value);
const isImage = computed(
  () => (props.item.type === "image" || props.item.type === "gif") && !!uri.value,
);

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

function onCanPlay() {
  const v = videoRef.value;
  if (!v) return;
  applyRatio(v.videoWidth, v.videoHeight);
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
}

watch(
  () => props.item.id,
  () => {
    mediaRatio.value = 16 / 9;
  },
);
</script>

<template>
  <div class="detail-media" :class="{ fill }">
    <div
      class="detail-media-bg"
      :class="{ 'is-video': isVideo }"
      :style="[{ background: item.thumb }, frameStyle]"
    >
      <video
        v-if="isVideo"
        ref="videoRef"
        class="detail-media-el"
        :src="uri!"
        muted
        playsinline
        loop
        preload="auto"
        @loadedmetadata="onLoadedMeta"
        @canplay="onCanPlay"
      />
      <img
        v-else-if="isImage"
        class="detail-media-el"
        :src="uri!"
        alt=""
        @load="onImgLoad"
      />
      <PreviewPlaybackBar v-if="isVideo" :video="videoRef" class="detail-playback-overlay" />
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
  background: #0e1014;
  border-radius: inherit;
}
.detail-media.fill .detail-media-el {
  object-fit: contain;
}
</style>
