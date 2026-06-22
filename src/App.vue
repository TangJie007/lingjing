<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAppStore } from '@/stores/app'
import { useWallpaperStore } from '@/stores/wallpaper'
import { useApiKeysStore } from '@/stores/api-keys'
import { useTray } from '@/composables/useTray'
import { useWindow } from '@/composables/useWindow'
import HazyBackground from '@/components/layout/HazyBackground.vue'
import AppTitlebar from '@/components/layout/AppTitlebar.vue'
import FabCreate from '@/components/layout/FabCreate.vue'
import SearchOverlay from '@/components/layout/SearchOverlay.vue'
import AiCreatePanel from '@/components/ai/AiCreatePanel.vue'
import ApiConfigBanner from '@/components/layout/ApiConfigBanner.vue'
import Toast from '@/components/common/Toast.vue'

const appStore = useAppStore()
const wallpaperStore = useWallpaperStore()
const apiKeysStore = useApiKeysStore()
const router = useRouter()
const route = useRoute()
const toastRef = ref<InstanceType<typeof Toast>>()

const searchOpen = ref(false)
const aiPanelOpen = ref(false)
const contentScrollRef = ref<HTMLElement | null>(null)
const contentScrolled = ref(false)

function onContentScroll() {
  contentScrolled.value = (contentScrollRef.value?.scrollTop ?? 0) > 2
}

const isOnboarding = computed(() => route.path === '/onboarding')
const isDesktopPlayer = computed(() => route.path === '/desktop-player')
const isFenceOverlay = computed(() => route.path === '/fence-overlay')
const showChrome = computed(() => !isOnboarding.value && !isDesktopPlayer.value && !isFenceOverlay.value)

// 系统托盘事件处理
useTray((action) => {
  switch (action) {
    case 'pause':
      wallpaperStore.toggleVideoPlay()
      toastRef.value?.show(
        'info',
        wallpaperStore.isVideoPlaying ? '视频壁纸已恢复' : '视频壁纸已暂停'
      )
      break
    case 'next':
      // 切换到下一张壁纸
      if (wallpaperStore.wallpapers.length > 1) {
        const currentIdx = wallpaperStore.wallpapers.findIndex(
          (w) => w.id === wallpaperStore.currentWallpaperId
        )
        const nextIdx = (currentIdx + 1) % wallpaperStore.wallpapers.length
        const next = wallpaperStore.wallpapers[nextIdx]
        if (next) {
          wallpaperStore.setWallpaper(next.id)
          toastRef.value?.show('info', '已切换到下一张壁纸')
        }
      }
      break
  }
})

useWindow()

// 全屏检测轮询
let fullscreenTimer: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  fullscreenTimer = setInterval(() => {
    wallpaperStore.checkFullscreen()
  }, 3000)
})

onUnmounted(() => {
  if (fullscreenTimer) {
    clearInterval(fullscreenTimer)
    fullscreenTimer = null
  }
})

function toggleAiPanel() {
  aiPanelOpen.value = !aiPanelOpen.value
}

watch(
  () => route.path,
  () => nextTick(onContentScroll)
)

watch(
  () => route.query.ai,
  (value) => {
    if (value === 'open') {
      aiPanelOpen.value = true
      // 保留 preset 参数给 AiCreatePanel 使用
      const preset = route.query.preset as string | undefined
      if (preset) {
        // 通过 store 或组件通信传递预设 prompt
        aiPreset.value = preset
      }
      router.replace({ path: route.path, query: {} })
    }
  },
  { immediate: true }
)

const aiPreset = ref<string | null>(null)

// 预设 prompt 映射
const presetPrompts: Record<string, string> = {
  preset1: '赛博朋克城市夜景，霓虹灯流光溢彩，雨后的街道倒映着光芒',
  preset2: '宫崎骏风格夏日田园，蓝天白云绿草地，温暖的阳光洒落',
  preset3: '极简抽象渐变，柔和的几何形态，静谧氛围',
}

onMounted(() => {
  appStore.loadSavedLocale()

  if (appStore.isFirstLaunch && route.path !== '/onboarding' && route.path !== '/desktop-player') {
    router.replace('/onboarding')
  }
})
</script>

<template>
  <!-- 桌面播放器 / 栅格叠加层 — 无 chrome，全屏 -->
  <template v-if="isDesktopPlayer || isFenceOverlay">
    <router-view />
  </template>

  <!-- 主界面 -->
  <template v-else>
    <HazyBackground v-if="showChrome" />

    <template v-if="isOnboarding">
      <router-view />
    </template>

    <div v-else class="app-shell">
      <AppTitlebar :scrolled="contentScrolled" @toggle-search="searchOpen = !searchOpen" />
      <SearchOverlay v-model:open="searchOpen" />
      <ApiConfigBanner
        v-if="showChrome && !appStore.isFirstLaunch && !apiKeysStore.hasConfiguredKey"
      />
      <div class="main-layout">
        <div class="content-area">
          <div ref="contentScrollRef" class="content-scroll" @scroll="onContentScroll">
            <router-view />
          </div>
        </div>
      </div>

      <FabCreate :active="aiPanelOpen" @click="toggleAiPanel" />
      <AiCreatePanel v-model:open="aiPanelOpen" :preset-prompt="presetPrompts[aiPreset ?? ''] ?? ''" />
      <Toast ref="toastRef" />
    </div>
  </template>
</template>
