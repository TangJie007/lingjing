<script setup lang="ts">
import { computed, ref } from "vue";
import type { WallpaperItem } from "../data/catalog";
import { resolveMediaUri } from "../composables/useEngine";
import PreviewPlaybackBar from "./PreviewPlaybackBar.vue";

const props = defineProps<{ item: WallpaperItem }>();

const videoRef = ref<HTMLVideoElement | null>(null);

const uri = computed(() => {
  if (props.item.missing || !props.item.mediaSrc) return null;
  return resolveMediaUri(props.item);
});

const isVideo = computed(() => props.item.type === "video" && !!uri.value);
const isImage = computed(
  () => (props.item.type === "image" || props.item.type === "gif") && !!uri.value,
);

function onCanPlay() {
  const v = videoRef.value;
  if (!v) return;
  void v.play().catch(() => undefined);
}
</script>

<template>
  <div class="detail-media">
    <div
      class="detail-media-bg"
      :class="{ 'is-video': isVideo }"
      :style="{ background: item.thumb }"
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
        @canplay="onCanPlay"
      />
      <img v-else-if="isImage" class="detail-media-el" :src="uri!" alt="" />
      <PreviewPlaybackBar v-if="isVideo" :video="videoRef" class="detail-playback-overlay" />
    </div>
  </div>
</template>

<style scoped>
.detail-media {
  position: relative;
  border-radius: var(--r-md);
  overflow: hidden;
  background: #0e1014;
}
.detail-media-bg {
  aspect-ratio: 16 / 9;
  max-height: min(56vh, 520px);
  position: relative;
  overflow: hidden;
}
.detail-playback-overlay {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 2;
}
.detail-media-el {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: contain;
  display: block;
  background: #0e1014;
}
</style>
