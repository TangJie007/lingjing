<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import Plyr from 'plyr'
import 'plyr/dist/plyr.css'
import { convertFileSrc, isTauri } from '@tauri-apps/api/core'

const props = defineProps<{
  src: string
  poster?: string | null
}>()

const videoRef = ref<HTMLVideoElement | null>(null)
let player: Plyr | null = null

const videoSrc = computed(() => (isTauri() ? convertFileSrc(props.src) : props.src))
const posterSrc = computed(() => {
  if (!props.poster) return undefined
  return isTauri() ? convertFileSrc(props.poster) : props.poster
})

function initPlayer() {
  if (!videoRef.value) return
  player?.destroy()
  player = new Plyr(videoRef.value, {
    controls: ['play-large', 'play', 'progress', 'current-time', 'mute', 'volume', 'fullscreen'],
    loop: { active: true },
    autoplay: true,
    muted: true,
    clickToPlay: true,
    hideControls: true,
    resetOnEnd: false,
  })
}

watch([() => props.src, () => props.poster], () => {
  if (videoRef.value) {
    videoRef.value.load()
    player?.play()
  }
})

onMounted(initPlayer)

onUnmounted(() => {
  player?.destroy()
  player = null
})

function togglePlay() {
  player?.togglePlay()
}

defineExpose({ togglePlay })
</script>

<template>
  <div class="wallpaper-preview-player">
    <video
      ref="videoRef"
      class="preview-video"
      :src="videoSrc"
      :poster="posterSrc"
      playsinline
      muted
      loop
    />
  </div>
</template>

<style scoped>
.wallpaper-preview-player {
  width: 100%;
  height: 100%;
  background: oklch(12% 0.01 163);
}

.wallpaper-preview-player :deep(.plyr) {
  --plyr-color-main: #008336;
  --plyr-video-background: oklch(12% 0.01 163);
  width: 100%;
  height: 100%;
  border-radius: 0;
}

.wallpaper-preview-player :deep(.plyr__video-wrapper) {
  height: 100%;
  background: oklch(12% 0.01 163);
}

.wallpaper-preview-player :deep(.plyr video) {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.wallpaper-preview-player :deep(.plyr__controls) {
  padding: 12px 16px;
  background: linear-gradient(transparent, rgba(0, 0, 0, 0.55));
}
</style>
