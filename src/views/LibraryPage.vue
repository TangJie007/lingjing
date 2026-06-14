<script setup lang="ts">
// 壁纸库页面 (WL-001, WL-002)
// 网格展示、浏览、应用、删除、导出、AI 生成历史
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useWallpaperStore } from '@/stores/wallpaper'
import type { WallpaperItem } from '@/stores/wallpaper'
import WallpaperCard from '@/components/wallpaper/WallpaperCard.vue'
import Toast from '@/components/common/Toast.vue'

const { t } = useI18n()
const wallpaperStore = useWallpaperStore()
const toastRef = ref<InstanceType<typeof Toast>>()

const activeCategory = ref('all')
const sortBy = ref<'newest' | 'oldest' | 'name'>('newest')
const liveOnly = ref(false)
const importing = ref(false)

const categories = computed(() => [
  { key: 'all', label: t('library.all') },
  { key: 'ai', label: t('library.aiGenerated') },
  { key: 'local', label: t('library.localImport') },
  { key: 'video', label: t('library.liveWallpaper') },
])

// 过滤和排序
const filteredWallpapers = computed(() => {
  let list = [...wallpaperStore.wallpapers]

  // 分类过滤
  if (activeCategory.value === 'ai') {
    list = list.filter((w) => w.source === 'ai')
  } else if (activeCategory.value === 'local') {
    list = list.filter((w) => w.source === 'local')
  } else if (activeCategory.value === 'video') {
    list = list.filter((w) => w.mediaType === 'video')
  }

  // 动态壁纸筛选
  if (liveOnly.value) {
    list = list.filter((w) => w.mediaType === 'video')
  }

  // 排序
  if (sortBy.value === 'newest') {
    list.sort((a, b) => parseInt(b.createdAt) - parseInt(a.createdAt))
  } else if (sortBy.value === 'oldest') {
    list.sort((a, b) => parseInt(a.createdAt) - parseInt(b.createdAt))
  } else if (sortBy.value === 'name') {
    list.sort((a, b) => a.filename.localeCompare(b.filename))
  }

  return list
})

const operatingId = ref<string | null>(null)

// 应用壁纸（乐观更新，后台完成切换，不阻塞 UI）
function applyWallpaper(id: string) {
  wallpaperStore.setWallpaper(id).then(() => {
    toastRef.value?.show('success', '✅ ' + t('toast.wallpaperSet'))
  }).catch((e) => {
    const message = e instanceof Error ? e.message : String(e)
    console.error('应用壁纸失败:', e)
    toastRef.value?.show('error', '❌ ' + (message || t('toast.generateFailed')))
  })
}

// 删除壁纸（Tauri 原生对话框，window.confirm 在 WebView 中不可靠）
async function deleteWallpaper(id: string) {
  if (operatingId.value) return
  try {
    const { ask } = await import('@tauri-apps/plugin-dialog')
    const confirmed = await ask(t('library.deleteBtn') + '?', {
      title: '灵境 LingScape',
      kind: 'warning',
    })
    if (!confirmed) return

    operatingId.value = id
    await wallpaperStore.deleteWallpaper(id)
    toastRef.value?.show('info', t('toast.wallpaperDeleted'))
  } catch (e) {
    const message = e instanceof Error ? e.message : String(e)
    console.error('删除壁纸失败:', e)
    toastRef.value?.show('error', '❌ ' + (message || t('toast.generateFailed')))
  } finally {
    operatingId.value = null
  }
}

// 导出壁纸
async function exportWallpaper(id: string) {
  const wp = wallpaperStore.wallpapers.find((w) => w.id === id)
  if (!wp) return
  try {
    await wallpaperStore.exportWallpaper(id, wp.filename)
  } catch {
    // 降级
  }
}

// 导入本地壁纸（使用 Tauri dialog 插件获取真实文件路径）
async function importWallpaper() {
  importing.value = true
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const result = await open({
      multiple: true,
      filters: [{
        name: '图片/视频',
        extensions: ['jpg', 'jpeg', 'png', 'webp', 'mp4', 'webm', 'gif'],
      }],
    })
    if (result) {
      const files = Array.isArray(result) ? result : [result]
      for (const filePath of files) {
        if (filePath) {
          await wallpaperStore.importWallpaper(filePath as string)
        }
      }
    }
  } catch (e) {
    console.error('导入失败:', e)
  } finally {
    importing.value = false
  }
}

onMounted(() => {
  wallpaperStore.loadWallpapers()
})
</script>

<template>
  <div class="page-library">
    <!-- 分类筛选 -->
    <div class="lib-categories">
      <button
        v-for="cat in categories"
        :key="cat.key"
        type="button"
        class="lib-cat-pill"
        :class="{ active: activeCategory === cat.key }"
        @click="activeCategory = cat.key"
      >
        {{ cat.label }}
      </button>
    </div>

    <!-- 操作栏 -->
    <div class="lib-meta-bar">
      <div class="lib-stats">
        <span>{{ t('library.totalCount', { count: filteredWallpapers.length }) }}</span>
      </div>
      <div class="lib-controls">
        <!-- 排序 -->
        <button type="button" class="lib-control-btn" @click="sortBy = sortBy === 'newest' ? 'oldest' : sortBy === 'oldest' ? 'name' : 'newest'">
          <span>{{
            sortBy === 'newest' ? t('library.sortNewest') :
            sortBy === 'oldest' ? t('library.sortOldest') :
            t('library.sortDefault')
          }}</span>
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="6 9 12 15 18 9" />
          </svg>
        </button>
        <!-- 动态壁纸筛选 -->
        <label class="lib-control-check">
          <input v-model="liveOnly" type="checkbox" />
          <span class="check-dot" />
          <span>{{ t('library.liveWallpaper') }}</span>
        </label>
        <!-- 导入按钮 -->
        <button type="button" class="lib-control-btn lib-import-btn" :disabled="importing" @click="importWallpaper">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" />
            <polyline points="7,10 12,15 17,10" />
            <line x1="12" y1="15" x2="12" y2="3" />
          </svg>
          <span>{{ importing ? t('common.loading') : t('library.localImport') }}</span>
        </button>
      </div>
    </div>

    <!-- 加载中 -->
    <div v-if="wallpaperStore.loading" class="empty-state">
      <div class="gen-spinner" style="margin-bottom: var(--spacing-4)" />
      <div class="empty-state-desc">{{ t('common.loading') }}</div>
    </div>

    <!-- 空状态 -->
    <div v-else-if="filteredWallpapers.length === 0" class="empty-state">
      <div class="empty-state-visual">
        <svg width="120" height="120" viewBox="0 0 120 120" fill="none" xmlns="http://www.w3.org/2000/svg">
          <defs>
            <filter id="empty-glow" x="-30%" y="-30%" width="160%" height="160%">
              <feGaussianBlur stdDeviation="4" result="blur" />
              <feMerge><feMergeNode in="blur" /><feMergeNode in="SourceGraphic" /></feMerge>
            </filter>
            <linearGradient id="empty-fill" x1="10" y1="90" x2="110" y2="20" gradientUnits="userSpaceOnUse">
              <stop offset="0%" stop-color="#fbbf24" />
              <stop offset="25%" stop-color="#a3e635" />
              <stop offset="55%" stop-color="#34d399" />
              <stop offset="85%" stop-color="#22d3ee" />
              <stop offset="100%" stop-color="#67e8f9" />
            </linearGradient>
            <radialGradient id="empty-hi" cx="40" cy="35" r="45" gradientUnits="userSpaceOnUse">
              <stop offset="0%" stop-color="white" stop-opacity="0.3" />
              <stop offset="100%" stop-color="white" stop-opacity="0" />
            </radialGradient>
          </defs>
          <path
            d="M 60 8 C 38 8, 12 30, 12 60 C 12 88, 30 108, 50 113 C 70 118, 95 105, 103 83 C 111 61, 105 35, 86 22 C 76 15, 68 8, 60 8 Z"
            fill="url(#empty-fill)"
            filter="url(#empty-glow)"
            opacity="0.35"
          />
          <path
            d="M 60 8 C 38 8, 12 30, 12 60 C 12 88, 30 108, 50 113 C 70 118, 95 105, 103 83 C 111 61, 105 35, 86 22 C 76 15, 68 8, 60 8 Z"
            fill="url(#empty-hi)"
          />
        </svg>
      </div>
      <div class="empty-state-title">{{ t('library.emptyTitle') }}</div>
      <div class="empty-state-desc">{{ t('library.emptyDesc') }}</div>
      <button type="button" class="btn-generate" @click="importWallpaper">
        <span class="gen-icon">📁</span>
        {{ t('library.localImport') }}
      </button>
    </div>

    <!-- 壁纸网格 -->
    <div v-else class="lib-grid">
      <WallpaperCard
        v-for="wp in filteredWallpapers"
        :key="wp.id"
        :wallpaper="wp"
        :is-active="wp.id === wallpaperStore.currentWallpaperId"
        :busy="operatingId === wp.id"
        @apply="applyWallpaper"
        @delete="deleteWallpaper"
        @export="exportWallpaper"
      />
    </div>
    <Toast ref="toastRef" />
  </div>
</template>

<style scoped>
.lib-import-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  color: var(--color-primary);
  border-color: var(--color-primary-surface);
  font-weight: 500;
}
.lib-import-btn:hover {
  background: var(--color-primary-surface);
  border-color: var(--color-primary);
}
.lib-import-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
