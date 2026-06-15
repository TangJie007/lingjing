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

  // V2 新增设置 (SET-006~010)
  const restoreOnExit = ref(true)
  const confirmOnExit = ref(false)
  const hoverPreview = ref(true)
  const rotateEnabled = ref(false)
  const rotateInterval = ref('30min')
  const rotateSource = ref('all')
  const rotateOrder = ref('sequential')
  const peekHotkey = ref('Win+Shift+P')
  const peekDuration = ref('3s')
  const allMonitorsSame = ref(true)

  // 持久化 — 监听所有设置变化
  watch(
    [locale, autoStart, videoFps, scalingMode, pauseOnFullscreen, cacheLimitGB,
     defaultStep1Model, defaultStep2Model, defaultGenerationCount, defaultResolution,
     restoreOnExit, confirmOnExit, hoverPreview, rotateEnabled, rotateInterval, rotateSource, rotateOrder],
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
          restoreOnExit: restoreOnExit.value,
          confirmOnExit: confirmOnExit.value,
          hoverPreview: hoverPreview.value,
          rotateEnabled: rotateEnabled.value,
          rotateInterval: rotateInterval.value,
          rotateSource: rotateSource.value,
          rotateOrder: rotateOrder.value,
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
      if (data.restoreOnExit !== undefined) restoreOnExit.value = data.restoreOnExit
      if (data.confirmOnExit !== undefined) confirmOnExit.value = data.confirmOnExit
      if (data.hoverPreview !== undefined) hoverPreview.value = data.hoverPreview
      if (data.rotateEnabled !== undefined) rotateEnabled.value = data.rotateEnabled
      if (data.rotateInterval) rotateInterval.value = data.rotateInterval
      if (data.rotateSource) rotateSource.value = data.rotateSource
      if (data.rotateOrder) rotateOrder.value = data.rotateOrder
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
    restoreOnExit,
    confirmOnExit,
    hoverPreview,
    rotateEnabled,
    rotateInterval,
    rotateSource,
    rotateOrder,
    peekHotkey,
    peekDuration,
    allMonitorsSame,
    loadSettings,
  }
})
