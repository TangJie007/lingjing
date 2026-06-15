<script setup lang="ts">
// 壁纸详情页 (WL-003)
// 大图预览 + 元数据 + AI 方案信息 + 动态播放控制
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useWallpaperStore } from '@/stores/wallpaper'
import type { WallpaperItem } from '@/stores/wallpaper'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const wallpaperStore = useWallpaperStore()

const wallpaper = computed<WallpaperItem | null>(() =>
  wallpaperStore.wallpapers.find((w) => w.id === route.params.id) ?? null
)

const isVideo = computed(() => wallpaper.value?.mediaType === 'video')
const isGif = computed(() => wallpaper.value?.mediaType === 'gif')
const isAi = computed(() => wallpaper.value?.source === 'ai')
const isActive = computed(() => wallpaper.value?.id === wallpaperStore.currentWallpaperId)

const sizeText = computed(() => {
  const size = wallpaper.value?.fileSize ?? 0
  if (size > 1024 * 1024) return `${(size / (1024 * 1024)).toFixed(1)} MB`
  if (size > 1024) return `${(size / 1024).toFixed(0)} KB`
  return `${size} B`
})

const dateText = computed(() => {
  try {
    if (!wallpaper.value) return ''
    const ts = parseInt(wallpaper.value.createdAt) * 1000
    return new Date(ts).toLocaleString('zh-CN')
  } catch {
    return ''
  }
})

function goBack() {
  router.back()
}

function applyWallpaper() {
  if (!wallpaper.value) return
  wallpaperStore.setWallpaper(wallpaper.value.id)
}

function copyPrompt() {
  if (wallpaper.value?.prompt) {
    navigator.clipboard.writeText(wallpaper.value.prompt)
  }
}

function regenerate() {
  if (!wallpaper.value) return
  router.push({ path: '/library', query: { ai: 'open', regenerate: wallpaper.value.id } })
}

// 键盘快捷键
function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    goBack()
  } else if (e.key === ' ') {
    e.preventDefault()
    // toggle play/pause
  } else if (e.key === 'ArrowLeft' || e.key === 'ArrowRight') {
    navigateWallpaper(e.key === 'ArrowRight' ? 1 : -1)
  }
}

function navigateWallpaper(direction: number) {
  const list = wallpaperStore.wallpapers
  const idx = list.findIndex((w) => w.id === route.params.id)
  if (idx === -1) return
  const nextIdx = (idx + direction + list.length) % list.length
  router.replace(`/wallpaper/${list[nextIdx].id}`)
}

onMounted(() => {
  window.addEventListener('keydown', onKeydown)
  if (!wallpaperStore.wallpapers.length) {
    wallpaperStore.loadWallpapers()
  }
})

onUnmounted(() => {
  window.removeEventListener('keydown', onKeydown)
})
</script>

<template>
  <div class="detail-page">
    <!-- 顶部导航 -->
    <div class="detail-header">
      <button class="back-btn" @click="goBack">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="15,18 9,12 15,6" />
        </svg>
        <span>{{ t('library.title') }}</span>
      </button>
      <div class="detail-nav-hint">
        <span>← → {{ t('detail.navigate') }} · Esc {{ t('detail.close') }}</span>
      </div>
    </div>

    <div v-if="!wallpaper" class="detail-empty">
      <p>{{ t('detail.notFound') }}</p>
    </div>

    <div v-else class="detail-body">
      <!-- 左侧：大图预览 -->
      <div class="detail-preview">
        <template v-if="isVideo">
          <video
            :src="wallpaper.path"
            class="preview-media"
            muted
            loop
            autoplay
            playsinline
            controls
          />
        </template>
        <template v-else-if="isGif">
          <img :src="wallpaper.path" :alt="wallpaper.filename" class="preview-media" />
        </template>
        <template v-else>
          <img :src="wallpaper.path" :alt="wallpaper.filename" class="preview-media" />
        </template>
      </div>

      <!-- 右侧：信息面板 -->
      <div class="detail-info">
        <h2 class="detail-name">{{ wallpaper.filename }}</h2>

        <!-- 基础信息 -->
        <div class="info-section">
          <h3 class="info-title">{{ t('detail.basicInfo') }}</h3>
          <div class="info-grid">
            <div class="info-item">
              <span class="info-label">{{ t('detail.type') }}</span>
              <span class="info-value">
                {{ isAi ? t('library.aiGenerated') : t('library.localImport') }}
                · {{ isVideo ? t('library.video') : isGif ? 'GIF' : t('detail.image') }}
              </span>
            </div>
            <div class="info-item">
              <span class="info-label">{{ t('detail.resolution') }}</span>
              <span class="info-value">{{ wallpaper.width }} × {{ wallpaper.height }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">{{ t('detail.fileSize') }}</span>
              <span class="info-value">{{ sizeText }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">{{ t('detail.createdAt') }}</span>
              <span class="info-value">{{ dateText }}</span>
            </div>
            <div v-if="isAi && wallpaper.model" class="info-item">
              <span class="info-label">{{ t('detail.aiModel') }}</span>
              <span class="info-value">{{ wallpaper.model }}</span>
            </div>
          </div>
        </div>

        <!-- AI 方案信息 -->
        <div v-if="isAi" class="info-section">
          <h3 class="info-title">{{ t('detail.aiPlan') }}</h3>
          <div v-if="wallpaper.prompt" class="info-block">
            <span class="info-label">{{ t('detail.originalDesc') }}</span>
            <p class="info-text">{{ wallpaper.prompt }}</p>
          </div>
          <div v-if="wallpaper.plan" class="info-block">
            <span class="info-label">{{ t('detail.optimizedPrompt') }}</span>
            <p class="info-text mono">{{ wallpaper.plan }}</p>
          </div>
          <div class="info-actions">
            <button v-if="wallpaper.prompt" class="action-btn" @click="copyPrompt">
              {{ t('detail.copyPrompt') }}
            </button>
            <button class="action-btn primary" @click="regenerate">
              {{ t('detail.regenerate') }}
            </button>
          </div>
        </div>

        <!-- 操作按钮 -->
        <div class="detail-actions">
          <button
            class="apply-btn"
            :class="{ disabled: isActive }"
            :disabled="isActive"
            @click="applyWallpaper"
          >
            {{ isActive ? t('library.inUse') : t('library.applyBtn') }}
          </button>
          <button class="action-btn" @click="wallpaperStore.exportWallpaper(wallpaper.id, wallpaper.filename)">
            {{ t('library.exportBtn') }}
          </button>
          <button class="action-btn danger" @click="wallpaperStore.deleteWallpaper(wallpaper.id); goBack()">
            {{ t('library.deleteBtn') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.detail-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.detail-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 24px;
  border-bottom: 1px solid var(--color-border-subtle);
  flex-shrink: 0;
}

.back-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  background: none;
  border: none;
  color: var(--color-text-secondary);
  font-family: var(--font-ui);
  font-size: 14px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 8px;
  transition: background 0.15s;
}
.back-btn:hover {
  background: var(--color-bg-surface);
}

.detail-nav-hint {
  font-size: 12px;
  color: var(--color-text-tertiary);
  font-family: var(--font-ui);
}

.detail-empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-tertiary);
}

.detail-body {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.detail-preview {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.03);
  padding: 24px;
}

.preview-media {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  border-radius: 12px;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.08);
}

.detail-info {
  width: 380px;
  flex-shrink: 0;
  overflow-y: auto;
  padding: 24px;
  border-left: 1px solid var(--color-border-subtle);
}

.detail-name {
  font-size: 20px;
  font-weight: 700;
  color: var(--color-text-primary);
  margin-bottom: 24px;
  word-break: break-all;
}

.info-section {
  margin-bottom: 24px;
}

.info-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--color-border-subtle);
}

.info-grid {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.info-item {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
}

.info-label {
  font-size: 13px;
  color: var(--color-text-tertiary);
  flex-shrink: 0;
}

.info-value {
  font-size: 13px;
  color: var(--color-text-primary);
  text-align: right;
}

.info-block {
  margin-bottom: 12px;
}

.info-text {
  font-size: 13px;
  color: var(--color-text-primary);
  line-height: 1.6;
  margin-top: 4px;
  padding: 10px 12px;
  background: var(--color-bg-surface);
  border-radius: 8px;
}

.info-text.mono {
  font-family: var(--font-mono);
  font-size: 12px;
}

.info-actions {
  display: flex;
  gap: 8px;
  margin-top: 8px;
}

.action-btn {
  padding: 6px 14px;
  border-radius: 8px;
  border: 1px solid var(--color-border-subtle);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
  font-family: var(--font-ui);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s;
}
.action-btn:hover {
  background: var(--color-bg-surface);
}
.action-btn.primary {
  background: var(--gradient-primary);
  color: white;
  border: none;
}
.action-btn.danger {
  color: #e53e3e;
  border-color: rgba(229, 62, 62, 0.3);
}

.detail-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-top: 16px;
  border-top: 1px solid var(--color-border-subtle);
}

.apply-btn {
  width: 100%;
  padding: 12px;
  border-radius: 12px;
  border: none;
  background: var(--gradient-primary);
  color: white;
  font-family: var(--font-ui);
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: opacity 0.15s;
}
.apply-btn:hover:not(.disabled) {
  opacity: 0.9;
}
.apply-btn.disabled {
  opacity: 0.5;
  cursor: default;
}
</style>
