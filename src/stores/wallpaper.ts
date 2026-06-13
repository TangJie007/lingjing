// 壁纸状态管理 (WP-001, WP-002, WL-001)
// 管理壁纸库列表、当前壁纸、视频播放状态
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'

const INVOKE_TIMEOUT_MS = 15_000

async function invokeWithTimeout<T>(
  cmd: string,
  args?: Record<string, unknown>
): Promise<T> {
  if (!isTauri()) {
    throw new Error('Tauri API unavailable')
  }

  return Promise.race([
    invoke<T>(cmd, args),
    new Promise<T>((_, reject) => {
      setTimeout(
        () => reject(new Error(`invoke timeout: ${cmd}`)),
        INVOKE_TIMEOUT_MS
      )
    }),
  ])
}
import { useSettingsStore } from './settings'

export interface WallpaperItem {
  id: string
  filename: string
  path: string
  mediaType: 'image' | 'video' | 'gif'
  format: string
  width: number
  height: number
  fileSize: number
  source: 'ai' | 'local'
  createdAt: string
  prompt?: string
  plan?: string
  model?: string
  cost?: number
}

export const useWallpaperStore = defineStore('wallpaper', () => {
  const wallpapers = ref<WallpaperItem[]>([])
  const currentWallpaperId = ref<string | null>(null)
  const isVideoPlaying = ref(false)
  const isFullscreenPaused = ref(false)
  const loading = ref(false)

  // 当前使用中的壁纸
  const currentWallpaper = computed(() =>
    wallpapers.value.find((w) => w.id === currentWallpaperId.value) ?? null
  )

  // 图片壁纸
  const imageWallpapers = computed(() =>
    wallpapers.value.filter((w) => w.mediaType === 'image' || w.mediaType === 'gif')
  )

  // 视频壁纸
  const videoWallpapers = computed(() =>
    wallpapers.value.filter((w) => w.mediaType === 'video')
  )

  // AI 生成的壁纸
  const aiWallpapers = computed(() =>
    wallpapers.value.filter((w) => w.source === 'ai')
  )

  // 库总大小
  const totalSize = computed(() =>
    wallpapers.value.reduce((sum, w) => sum + w.fileSize, 0)
  )

  // 加载壁纸列表
  async function loadWallpapers() {
    loading.value = true
    try {
      wallpapers.value = await invokeWithTimeout<WallpaperItem[]>('list_wallpapers')
    } catch (e) {
      console.error('加载壁纸列表失败:', e)
      wallpapers.value = []
    } finally {
      loading.value = false
    }
  }

  // 导入本地文件
  async function importWallpaper(filePath: string): Promise<WallpaperItem | null> {
    try {
      const entry = await invoke<WallpaperItem>('import_wallpaper', {
        sourcePath: filePath,
      })
      wallpapers.value.unshift(entry)
      return entry
    } catch (e) {
      console.error('导入壁纸失败:', e)
      return null
    }
  }

  // 删除壁纸
  async function deleteWallpaper(id: string) {
    await invokeWithTimeout('delete_wallpaper', { id })
    wallpapers.value = wallpapers.value.filter((w) => w.id !== id)
    if (currentWallpaperId.value === id) {
      currentWallpaperId.value = null
    }
  }

  // 导出壁纸
  async function exportWallpaper(id: string, destPath: string) {
    try {
      await invoke('export_wallpaper', { id, destPath })
    } catch (e) {
      console.error('导出壁纸失败:', e)
      throw e
    }
  }

  // 设置壁纸
  async function setWallpaper(id: string) {
    await invokeWithTimeout('set_wallpaper', { id })
    currentWallpaperId.value = id

    const wp = wallpapers.value.find((w) => w.id === id)
    if (wp && wp.mediaType === 'video') {
      isVideoPlaying.value = true
    }
  }

  // 暂停/恢复视频
  function toggleVideoPlay() {
    isVideoPlaying.value = !isVideoPlaying.value
  }

  function pauseVideo() {
    isVideoPlaying.value = false
  }

  function resumeVideo() {
    isVideoPlaying.value = true
  }

  // 全屏检测
  function setFullscreenPaused(paused: boolean) {
    isFullscreenPaused.value = paused
    if (paused) {
      pauseVideo()
    } else {
      resumeVideo()
    }
  }

  // 检查全屏状态（轮询）
  async function checkFullscreen() {
    try {
      const isFullscreen = await invoke<boolean>('is_fullscreen_app_running')
      const settings = useSettingsStore()
      if (settings.pauseOnFullscreen) {
        setFullscreenPaused(isFullscreen)
      }
    } catch {
      // 静默处理
    }
  }

  // 获取库大小
  async function getLibrarySize(): Promise<number> {
    try {
      return await invoke<number>('get_library_size')
    } catch {
      return 0
    }
  }

  // 清除壁纸库
  async function clearLibrary() {
    try {
      await invoke('clear_library')
      wallpapers.value = []
      currentWallpaperId.value = null
    } catch (e) {
      console.error('清除壁纸库失败:', e)
    }
  }

  // 添加 AI 生成的壁纸（由 AI 生成流程调用）
  async function addAiWallpaper(entry: Omit<WallpaperItem, 'id' | 'createdAt'>) {
    // AI 生成的壁纸直接追加到列表中（后端已保存文件）
    const id = `wp_${Date.now().toString(16)}`
    const item: WallpaperItem = {
      ...entry,
      id,
      createdAt: String(Math.floor(Date.now() / 1000)),
    }
    wallpapers.value.unshift(item)
    return item
  }

  return {
    wallpapers,
    currentWallpaperId,
    currentWallpaper,
    isVideoPlaying,
    isFullscreenPaused,
    loading,
    imageWallpapers,
    videoWallpapers,
    aiWallpapers,
    totalSize,
    loadWallpapers,
    importWallpaper,
    deleteWallpaper,
    exportWallpaper,
    setWallpaper,
    toggleVideoPlay,
    pauseVideo,
    resumeVideo,
    setFullscreenPaused,
    checkFullscreen,
    getLibrarySize,
    clearLibrary,
    addAiWallpaper,
  }
})
