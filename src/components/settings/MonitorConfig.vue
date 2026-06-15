<script setup lang="ts">
// 多显示器壁纸配置 (SET-008)
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '@/stores/settings'
import { useWallpaperStore } from '@/stores/wallpaper'
import ToggleSwitch from '@/components/common/ToggleSwitch.vue'

const { t } = useI18n()
const settings = useSettingsStore()
const wallpaperStore = useWallpaperStore()

interface MonitorInfo {
  index: number
  name: string
  width: number
  height: number
  wallpaperId: string | null
}

const monitors = ref<MonitorInfo[]>([])
const loading = ref(false)

onMounted(async () => {
  loading.value = true
  try {
    // 模拟检测显示器（实际应通过 Rust invoke 获取）
    const { invoke } = await import('@tauri-apps/api/core')
    const screenWidth = window.screen.width
    const screenHeight = window.screen.height
    monitors.value = [
      {
        index: 0,
        name: `${t('settings.monitorMain')} ${screenWidth}×${screenHeight}`,
        width: screenWidth,
        height: screenHeight,
        wallpaperId: wallpaperStore.currentWallpaperId,
      },
    ]
    // 尝试获取真实显示器信息
    try {
      const realMonitors = await invoke<{ index: number; w: number; h: number; x: number; y: number }[]>('enumerate_monitors')
      if (realMonitors && realMonitors.length > 0) {
        monitors.value = realMonitors.map((m, i) => ({
          index: m.index ?? i,
          name: `${t('settings.monitor')} ${i + 1} ${m.w}×${m.h}`,
          width: m.w,
          height: m.h,
          wallpaperId: i === 0 ? wallpaperStore.currentWallpaperId : null,
        }))
      }
    } catch { /* 降级使用 window.screen */ }
  } finally {
    loading.value = false
  }
})

async function setMonitorWallpaper(monitorIndex: number, wallpaperId: string) {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('set_wallpaper', { id: wallpaperId })
    const m = monitors.value.find((m) => m.index === monitorIndex)
    if (m) m.wallpaperId = wallpaperId
  } catch (e) {
    console.error('设置显示器壁纸失败:', e)
  }
}
</script>

<template>
  <section class="settings-section">
    <h2 class="settings-section-title">{{ t('settings.monitorConfig') }}</h2>
    <div v-if="loading" class="setting-row">
      <span class="setting-desc">{{ t('common.loading') }}</span>
    </div>
    <div v-else-if="monitors.length === 0" class="setting-row">
      <span class="setting-desc">{{ t('settings.noMonitor') }}</span>
    </div>
    <template v-else>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ t('settings.allMonitorsSame') }}</span>
          <span class="setting-desc">{{ t('settings.allMonitorsSameDesc') }}</span>
        </div>
        <ToggleSwitch v-model="settings.allMonitorsSame" />
      </div>
      <div v-for="m in monitors" :key="m.index" class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ m.name }}</span>
          <span class="setting-desc">
            {{ m.wallpaperId ? t('library.inUse') : t('settings.noWallpaper') }}
          </span>
        </div>
        <select
          class="setting-select"
          :value="m.wallpaperId ?? ''"
          @change="setMonitorWallpaper(m.index, ($event.target as HTMLSelectElement).value)"
        >
          <option value="">{{ t('settings.selectWallpaper') }}</option>
          <option
            v-for="wp in wallpaperStore.wallpapers"
            :key="wp.id"
            :value="wp.id"
          >
            {{ wp.filename }}
          </option>
        </select>
      </div>
    </template>
  </section>
</template>
