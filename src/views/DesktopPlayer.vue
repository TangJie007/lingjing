<script setup lang="ts">
// 桌面播放器占位窗口（视频壁纸由后端 MPV + desktop_core 渲染，不经过 WebView）
import { onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { useWallpaperStore } from '@/stores/wallpaper'
import { invoke } from '@tauri-apps/api/core'

const wallpaperStore = useWallpaperStore()

let fullscreenInterval: ReturnType<typeof setInterval> | null = null

function startFullscreenCheck() {
  if (fullscreenInterval) return
  fullscreenInterval = setInterval(async () => {
    await wallpaperStore.checkFullscreen()
  }, 2000)
}

function stopFullscreenCheck() {
  if (fullscreenInterval) {
    clearInterval(fullscreenInterval)
    fullscreenInterval = null
  }
}

onMounted(async () => {
  try {
    const current = await invoke<{ type: string; path: string; id: string } | null>(
      'get_current_wallpaper'
    )
    if (current?.type === 'video') {
      wallpaperStore.resumeVideo()
      startFullscreenCheck()
    }
  } catch {
    // 非 Tauri 环境
  }

  let unlisten: (() => void) | null = null
  try {
    unlisten = await listen<{ type: string; path: string; id: string; engine?: string }>(
      'wallpaper-changed',
      (event) => {
        if (event.payload.type === 'video') {
          wallpaperStore.resumeVideo()
          startFullscreenCheck()
        } else {
          wallpaperStore.pauseVideo()
          stopFullscreenCheck()
        }
      }
    )
  } catch {
    // 事件监听不可用
  }

  onUnmounted(() => {
    if (unlisten) unlisten()
    stopFullscreenCheck()
  })
})
</script>

<template>
  <div class="desktop-player" />
</template>

<style scoped>
.desktop-player {
  position: fixed;
  inset: 0;
  width: 100vw;
  height: 100vh;
  background: transparent;
  pointer-events: none;
}
</style>

<style>
html:has(.desktop-player),
body:has(.desktop-player) {
  margin: 0;
  padding: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
  background: transparent !important;
}
</style>
