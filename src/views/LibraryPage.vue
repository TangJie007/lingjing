<script setup lang="ts">
// 壁纸库页面 V2 (WL-001, WL-002, WL-004)
// 卡片网格、排序、搜索、标签筛选、收藏
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useWallpaperStore } from '@/stores/wallpaper'
import { useDesktopOrganizerStore } from '@/stores/desktop-organizer'
import WallpaperCard from '@/components/wallpaper/WallpaperCard.vue'
import Toast from '@/components/common/Toast.vue'

const { t } = useI18n()
const wallpaperStore = useWallpaperStore()
const organizerStore = useDesktopOrganizerStore()
const toastRef = ref<InstanceType<typeof Toast>>()

const activeCategory = ref('all')
const sortBy = ref<'newest' | 'oldest' | 'name' | 'size' | 'recent'>('newest')
const viewMode = ref<'grid' | 'list'>('grid')
const cardSize = ref<'sm' | 'md' | 'lg'>('md')
const importing = ref(false)
const operatingId = ref<string | null>(null)

const categories = [
  { key: 'all', label: t('library.all') },
  { key: 'favorites', label: '♥ ' + t('library.favorites') },
  { key: 'ai', label: t('library.aiGenerated') },
  { key: 'local', label: t('library.localImport') },
  { key: 'video', label: t('library.liveWallpaper') },
]

const sortLabels: Record<string, string> = {
  newest: t('library.sortNewest'),
  oldest: t('library.sortOldest'),
  name: t('library.sortName'),
  size: t('library.sortSize'),
  recent: t('library.sortRecent'),
}

// 分类 + 排序
const displayList = computed(() => {
  let list = wallpaperStore.filteredWallpapers

  if (activeCategory.value === 'favorites') {
    list = list.filter((w) => w.favorite)
  } else if (activeCategory.value === 'ai') {
    list = list.filter((w) => w.source === 'ai')
  } else if (activeCategory.value === 'local') {
    list = list.filter((w) => w.source === 'local')
  } else if (activeCategory.value === 'video') {
    list = list.filter((w) => w.mediaType === 'video')
  }

  const sorted = [...list]
  switch (sortBy.value) {
    case 'newest':
      sorted.sort((a, b) => parseInt(b.createdAt) - parseInt(a.createdAt))
      break
    case 'oldest':
      sorted.sort((a, b) => parseInt(a.createdAt) - parseInt(b.createdAt))
      break
    case 'name':
      sorted.sort((a, b) => a.filename.localeCompare(b.filename))
      break
    case 'size':
      sorted.sort((a, b) => b.fileSize - a.fileSize)
      break
    case 'recent':
      sorted.sort((a, b) => parseInt(b.createdAt) - parseInt(a.createdAt))
      break
  }
  return sorted
})

// 一键整理桌面
async function organizeDesktop() {
  try {
    const result = await organizerStore.organizeDesktop()
    toastRef.value?.show(
      'success',
      t('desktop.organizeSuccess', { count: result.arranged }),
    )
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    toastRef.value?.show('error', t('desktop.organizeFailed', { msg }))
  }
}

// 应用壁纸
function applyWallpaper(id: string) {
  wallpaperStore.setWallpaper(id).then(() => {
    toastRef.value?.show('success', '✅ ' + t('toast.wallpaperSet'))
  }).catch((e) => {
    const message = e instanceof Error ? e.message : String(e)
    console.error('应用壁纸失败:', e)
    toastRef.value?.show('error', '❌ ' + (message || t('toast.generateFailed')))
  })
}

// 删除壁纸
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
  } catch { /* 降级 */ }
}

// 导入本地壁纸
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
        if (filePath) await wallpaperStore.importWallpaper(filePath as string)
      }
    }
  } catch (e) {
    console.error('导入失败:', e)
  } finally {
    importing.value = false
  }
}

// 切换收藏
function toggleFavorite(id: string) {
  wallpaperStore.toggleFavorite(id)
}

// 循环排序
function cycleSort() {
  const order: typeof sortBy.value[] = ['newest', 'oldest', 'name', 'size', 'recent']
  const idx = order.indexOf(sortBy.value)
  sortBy.value = order[(idx + 1) % order.length]
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
      <!-- 动态标签 -->
      <button
        v-for="tag in wallpaperStore.allTags"
        :key="tag"
        type="button"
        class="lib-cat-pill tag"
        :class="{ active: wallpaperStore.activeTag === tag }"
        @click="wallpaperStore.activeTag = wallpaperStore.activeTag === tag ? null : tag"
      >
        {{ tag }}
      </button>
    </div>

    <!-- 操作栏 -->
    <div class="lib-meta-bar">
      <div class="lib-stats">
        <span>{{ t('library.totalCount', { count: displayList.length }) }}</span>
      </div>
      <div class="lib-controls">
        <!-- 视图切换 -->
        <button type="button" class="lib-control-btn icon-only" @click="viewMode = viewMode === 'grid' ? 'list' : 'grid'">
          <svg v-if="viewMode === 'grid'" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7" /><rect x="14" y="3" width="7" height="7" /><rect x="3" y="14" width="7" height="7" /><rect x="14" y="14" width="7" height="7" /></svg>
          <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="8" y1="6" x2="21" y2="6" /><line x1="8" y1="12" x2="21" y2="12" /><line x1="8" y1="18" x2="21" y2="18" /><line x1="3" y1="6" x2="3.01" y2="6" /><line x1="3" y1="12" x2="3.01" y2="12" /><line x1="3" y1="18" x2="3.01" y2="18" /></svg>
        </button>
        <!-- 卡片尺寸 -->
        <button type="button" class="lib-control-btn icon-only" @click="cardSize = cardSize === 'md' ? 'lg' : cardSize === 'lg' ? 'sm' : 'md'">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="4" y="4" :width="cardSize === 'sm' ? 6 : cardSize === 'md' ? 8 : 10" :height="cardSize === 'sm' ? 6 : cardSize === 'md' ? 8 : 10" rx="1" /></svg>
        </button>
        <!-- 排序 -->
        <button type="button" class="lib-control-btn" @click="cycleSort">
          <span>{{ sortLabels[sortBy] }}</span>
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="6 9 12 15 18 9" />
          </svg>
        </button>
        <!-- 导入按钮 -->
        <button type="button" class="lib-control-btn lib-import-btn" :disabled="importing" @click="importWallpaper">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" />
            <polyline points="7,10 12,15 17,10" />
            <line x1="12" y1="15" x2="12" y2="3" />
          </svg>
          <span>{{ importing ? t('common.loading') : t('library.localImport') }}</span>
        </button>
        <!-- 一键桌面整理 -->
        <button type="button" class="lib-control-btn lib-organize-btn" :disabled="organizerStore.organizing" @click="organizeDesktop">
          <span v-if="organizerStore.organizing" class="oc-spinner" />
          <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="3" width="7" height="7" rx="1" /><rect x="14" y="3" width="7" height="7" rx="1" />
            <rect x="3" y="14" width="7" height="7" rx="1" /><rect x="14" y="14" width="7" height="7" rx="1" />
          </svg>
          <span>{{ organizerStore.organizing ? t('desktop.organizing') : t('desktop.organizeBtn') }}</span>
        </button>
      </div>
    </div>

    <!-- 加载中 -->
    <div v-if="wallpaperStore.loading" class="empty-state">
      <div class="gen-spinner" style="margin-bottom: var(--spacing-4)" />
      <div class="empty-state-desc">{{ t('common.loading') }}</div>
    </div>

    <!-- 空状态 -->
    <div v-else-if="displayList.length === 0" class="empty-state">
      <div class="empty-state-visual">
        <svg width="120" height="120" viewBox="0 0 120 120" fill="none">
          <defs>
            <filter id="empty-glow" x="-30%" y="-30%" width="160%" height="160%">
              <feGaussianBlur stdDeviation="4" result="blur" />
              <feMerge><feMergeNode in="blur" /><feMergeNode in="SourceGraphic" /></feMerge>
            </filter>
            <linearGradient id="empty-fill" x1="10" y1="90" x2="110" y2="20" gradientUnits="userSpaceOnUse">
              <stop offset="0%" stop-color="#fbbf24" /><stop offset="25%" stop-color="#a3e635" />
              <stop offset="55%" stop-color="#34d399" /><stop offset="85%" stop-color="#22d3ee" />
              <stop offset="100%" stop-color="#67e8f9" />
            </linearGradient>
            <radialGradient id="empty-hi" cx="40" cy="35" r="45" gradientUnits="userSpaceOnUse">
              <stop offset="0%" stop-color="white" stop-opacity="0.3" /><stop offset="100%" stop-color="white" stop-opacity="0" />
            </radialGradient>
          </defs>
          <path d="M 60 8 C 38 8, 12 30, 12 60 C 12 88, 30 108, 50 113 C 70 118, 95 105, 103 83 C 111 61, 105 35, 86 22 C 76 15, 68 8, 60 8 Z" fill="url(#empty-fill)" filter="url(#empty-glow)" opacity="0.35" />
          <path d="M 60 8 C 38 8, 12 30, 12 60 C 12 88, 30 108, 50 113 C 70 118, 95 105, 103 83 C 111 61, 105 35, 86 22 C 76 15, 68 8, 60 8 Z" fill="url(#empty-hi)" />
        </svg>
      </div>
      <div class="empty-state-title">{{ t('library.emptyTitle') }}</div>
      <div class="empty-state-desc">{{ t('library.emptyDesc') }}</div>
      <button type="button" class="btn-generate" @click="importWallpaper">
        <span class="gen-icon">📁</span>
        {{ t('library.localImport') }}
      </button>
    </div>

    <!-- 壁纸网格 / 列表 -->
    <div v-else :class="['lib-grid', `size-${cardSize}`, viewMode === 'list' ? 'list-view' : '']">
      <WallpaperCard
        v-for="wp in displayList"
        :key="wp.id"
        :wallpaper="wp"
        :is-active="wp.id === wallpaperStore.currentWallpaperId"
        @apply="applyWallpaper"
        @delete="deleteWallpaper"
        @export="exportWallpaper"
        @toggle-favorite="toggleFavorite"
      />
    </div>
    <Toast ref="toastRef" />
  </div>
</template>

<style scoped>
/* ── Spinner (shared) ─────────────────────────────────────── */
.oc-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid var(--color-primary-surface);
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  flex-shrink: 0;
}
@keyframes spin { to { transform: rotate(360deg); } }

.lib-cat-pill.tag {
  background: rgba(203, 236, 218, 0.5);
  font-size: 12px;
}

.lib-grid.size-sm { grid-template-columns: repeat(auto-fill, minmax(150px, 1fr)); }
.lib-grid.size-md { grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); }
.lib-grid.size-lg { grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); }
.lib-grid.list-view {
  grid-template-columns: 1fr;
}

.lib-control-btn.icon-only {
  width: 36px;
  padding: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.lib-import-btn,
.lib-organize-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  color: var(--color-primary);
  border-color: var(--color-primary-surface);
  font-weight: 500;
}
.lib-import-btn:hover,
.lib-organize-btn:hover {
  background: var(--color-primary-surface);
  border-color: var(--color-primary);
}
.lib-import-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.lib-organize-btn:disabled {
  opacity: 0.6;
  cursor: wait;
}
</style>
