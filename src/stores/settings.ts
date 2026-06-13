// 通用设置状态 (SET-001~004)
import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { DEFAULT_LOCALE } from '@/config/constants'

export const useSettingsStore = defineStore('settings', () => {
  // 通用 (SET-001)
  const autoStart = ref(false)
  const locale = ref(localStorage.getItem('lingscape-locale') || DEFAULT_LOCALE)

  // API 默认值 (SET-002)
  const defaultStep1Model = ref('doubao-2.0-lite-32k')
  const defaultStep2Model = ref('Seedream 4.0')
  const defaultGenerationCount = ref(3)
  const defaultResolution = ref('1080P')

  // 壁纸播放 (SET-003)
  const videoFps = ref(30)
  const scalingMode = ref<'fill' | 'fit' | 'stretch' | 'tile'>('fill')

  // 性能与存储 (SET-004)
  const pauseOnFullscreen = ref(true)
  const cacheLimitGB = ref(5)

  // 持久化 — 监听所有设置变化
  watch(
    [locale, autoStart, videoFps, scalingMode, pauseOnFullscreen, cacheLimitGB,
     defaultStep1Model, defaultStep2Model, defaultGenerationCount, defaultResolution],
    () => {
      localStorage.setItem(
        'lingscape-settings',
        JSON.stringify({
          locale: locale.value,
          autoStart: autoStart.value,
          videoFps: videoFps.value,
          scalingMode: scalingMode.value,
          pauseOnFullscreen: pauseOnFullscreen.value,
          cacheLimitGB: cacheLimitGB.value,
          defaultStep1Model: defaultStep1Model.value,
          defaultStep2Model: defaultStep2Model.value,
          defaultGenerationCount: defaultGenerationCount.value,
          defaultResolution: defaultResolution.value,
        })
      )
    },
    { deep: true }
  )

  function loadSettings() {
    const raw = localStorage.getItem('lingscape-settings')
    if (!raw) return
    try {
      const data = JSON.parse(raw)
      if (data.locale) locale.value = data.locale
      if (data.autoStart !== undefined) autoStart.value = data.autoStart
      if (data.videoFps) videoFps.value = data.videoFps
      if (data.scalingMode) scalingMode.value = data.scalingMode
      if (data.pauseOnFullscreen !== undefined) pauseOnFullscreen.value = data.pauseOnFullscreen
      if (data.cacheLimitGB !== undefined) cacheLimitGB.value = data.cacheLimitGB
      if (data.defaultStep1Model) defaultStep1Model.value = data.defaultStep1Model
      if (data.defaultStep2Model) defaultStep2Model.value = data.defaultStep2Model
      if (data.defaultGenerationCount) defaultGenerationCount.value = data.defaultGenerationCount
      if (data.defaultResolution) defaultResolution.value = data.defaultResolution
    } catch {
      // ignore corrupted data
    }
  }

  return {
    autoStart,
    locale,
    defaultStep1Model,
    defaultStep2Model,
    defaultGenerationCount,
    defaultResolution,
    videoFps,
    scalingMode,
    pauseOnFullscreen,
    cacheLimitGB,
    loadSettings,
  }
})
