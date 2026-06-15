<script setup lang="ts">
// 壁纸卡片组件 V2 — 缩略图网格 + hover 动态预览 + 类型角标 + 收藏按钮
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import type { WallpaperItem } from '@/stores/wallpaper'

const props = defineProps<{
  wallpaper: WallpaperItem
  isActive?: boolean
}>()

const emit = defineEmits<{
  apply: [id: string]
  delete: [id: string]
  export: [id: string]
  toggleFavorite: [id: string]
}>()

const { t } = useI18n()
const router = useRouter()
const imgError = ref(false)
const isHovering = ref(false)
const showPreview = ref(false)
let hoverTimer: ReturnType<typeof setTimeout> | null = null

const isVideo = computed(() => props.wallpaper.mediaType === 'video')
const isGif = computed(() => props.wallpaper.mediaType === 'gif')
const isAi = computed(() => props.wallpaper.source === 'ai')
const isFavorited = computed(() => props.wallpaper.favorite === true)

const thumbSrc = computed(() => {
  if (props.wallpaper.thumbPath) return props.wallpaper.thumbPath
  if (props.wallpaper.mediaType === 'image') return props.wallpaper.path
  return null
})

const formatBadge = computed(() => {
  if (isGif.value) return 'GIF'
  if (isVideo.value) return props.wallpaper.format.toUpperCase()
  return null
})

const sizeText = computed(() => {
  const size = props.wallpaper.fileSize
  if (size > 1024 * 1024) return `${(size / (1024 * 1024)).toFixed(1)} MB`
  if (size > 1024) return `${(size / 1024).toFixed(0)} KB`
  return `${size} B`
})

const dateText = computed(() => {
  try {
    const ts = parseInt(props.wallpaper.createdAt) * 1000
    const d = new Date(ts)
    return d.toLocaleDateString('zh-CN', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    })
  } catch {
    return ''
  }
})

function onMouseEnter() {
  isHovering.value = true
  if (isVideo.value || isGif.value) {
    hoverTimer = setTimeout(() => {
      showPreview.value = true
    }, 500)
  }
}

function onMouseLeave() {
  isHovering.value = false
  showPreview.value = false
  if (hoverTimer) {
    clearTimeout(hoverTimer)
    hoverTimer = null
  }
}

function openDetail() {
  router.push(`/wallpaper/${props.wallpaper.id}`)
}
</script>

<template>
  <div
    class="lib-card"
    :class="{ 'is-active': isActive }"
    @mouseenter="onMouseEnter"
    @mouseleave="onMouseLeave"
    @click="openDetail"
  >
    <div class="lib-card-img">
      <!-- 缩略图 / 静态图片 -->
      <template v-if="!imgError && thumbSrc && !showPreview">
        <img
          :src="thumbSrc"
          :alt="wallpaper.filename"
          @error="imgError = true"
        />
      </template>
      <!-- hover 动态预览 -->
      <template v-else-if="showPreview && (isVideo || isGif)">
        <video
          v-if="isVideo"
          :src="wallpaper.path"
          muted
          loop
          autoplay
          playsinline
          class="preview-video"
        />
        <img
          v-else-if="isGif"
          :src="wallpaper.path"
          :alt="wallpaper.filename"
        />
      </template>
      <!-- 无缩略图时的占位 -->
      <template v-else>
        <div class="img-fallback">
          <svg width="40" height="40" viewBox="0 0 40 40" fill="none">
            <rect x="4" y="4" width="32" height="32" rx="4" stroke="currentColor" stroke-width="1.5" opacity="0.3" />
            <circle cx="16" cy="15" r="3" fill="currentColor" opacity="0.2" />
            <path d="M4 28l8-8 6 6 8-8 10 10" stroke="currentColor" stroke-width="1.5" opacity="0.3" fill="none" />
          </svg>
        </div>
      </template>

      <!-- 徽章 -->
      <div class="lib-card-badges">
        <span v-if="isActive" class="lib-badge active">{{ t('library.inUse') }}</span>
        <span v-if="formatBadge" class="lib-badge format">{{ formatBadge }}</span>
        <span v-if="isAi" class="lib-badge ai">AI</span>
      </div>

      <!-- 收藏按钮 -->
      <button
        class="fav-btn"
        :class="{ favorited: isFavorited }"
        @click.stop="emit('toggleFavorite', wallpaper.id)"
      >
        {{ isFavorited ? '♥' : '♡' }}
      </button>
    </div>

    <!-- 信息行 -->
    <div class="lib-card-info">
      <div class="lib-card-title-row">
        <span class="lib-card-title">{{ wallpaper.filename }}</span>
      </div>
      <div class="lib-card-meta">
        <span>{{ sizeText }} · {{ dateText }}</span>
      </div>
    </div>

    <!-- 操作按钮 -->
    <div class="lib-card-actions">
      <button
        class="card-action-btn primary"
        :class="{ disabled: isActive }"
        :disabled="isActive"
        @click.stop="emit('apply', wallpaper.id)"
      >
        {{ isActive ? t('library.inUse') : t('library.applyBtn') }}
      </button>
      <button
        class="card-action-btn"
        @click.stop="emit('delete', wallpaper.id)"
      >
        {{ t('library.deleteBtn') }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.lib-card {
  border-radius: 16px;
  overflow: hidden;
  background: rgba(249, 252, 250, 0.85);
  backdrop-filter: blur(12px) saturate(1.1);
  border: 1px solid rgba(206, 214, 210, 0.6);
  transition: all 0.4s cubic-bezier(0.23, 1, 0.32, 1);
  position: relative;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04), 0 4px 12px rgba(0, 0, 0, 0.03);
  display: flex;
  flex-direction: column;
  cursor: pointer;
}
.lib-card:hover {
  transform: translateY(-4px);
  border-color: rgba(104, 180, 139, 0.5);
  box-shadow:
    0 8px 24px rgba(0, 0, 0, 0.06),
    0 16px 48px rgba(19, 92, 53, 0.1);
}
.lib-card.is-active {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px rgba(0, 131, 54, 0.25);
}

.lib-card-img {
  aspect-ratio: 16/10;
  overflow: hidden;
  position: relative;
  background: oklch(97% 0.004 163);
}
.lib-card-img img,
.preview-video {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.6s cubic-bezier(0.23, 1, 0.32, 1);
  display: block;
}
.lib-card:hover .lib-card-img img,
.lib-card:hover .preview-video {
  transform: scale(1.06);
}

.lib-card-badges {
  position: absolute;
  top: 10px;
  left: 10px;
  display: flex;
  gap: 6px;
  pointer-events: none;
  z-index: 3;
}

.lib-badge {
  padding: 3px 10px;
  border-radius: 20px;
  font-size: 11px;
  font-weight: 600;
  font-family: var(--font-ui);
  letter-spacing: 0.02em;
  backdrop-filter: blur(8px);
}
.lib-badge.active {
  background: var(--gradient-primary);
  color: white;
  box-shadow: 0 2px 8px rgba(0, 131, 54, 0.35);
}
.lib-badge.format {
  background: rgba(0, 1, 2, 0.72);
  color: #d1dce9;
  border: 1px solid rgba(5, 7, 10, 0.3);
}
.lib-badge.ai {
  background: linear-gradient(135deg, rgba(74, 30, 178, 0.9), rgba(93, 17, 86, 0.9));
  color: white;
}

.fav-btn {
  position: absolute;
  top: 10px;
  right: 10px;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  border: none;
  background: rgba(0, 0, 0, 0.35);
  color: white;
  font-size: 16px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  z-index: 3;
  backdrop-filter: blur(4px);
}
.fav-btn:hover {
  background: rgba(0, 0, 0, 0.55);
  transform: scale(1.1);
}
.fav-btn.favorited {
  color: #ff4d6a;
}

.img-fallback {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-tertiary);
}

.lib-card-info { padding: 12px 14px 8px; flex: 1; }
.lib-card-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.lib-card-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: var(--font-ui);
}
.lib-card-meta {
  font-size: 12px;
  color: var(--color-text-tertiary);
  margin-top: 4px;
  font-family: var(--font-ui);
}

.lib-card-actions {
  display: flex;
  gap: 8px;
  padding: 0 14px 14px;
}
.card-action-btn {
  flex: 1;
  padding: 8px 0;
  border-radius: 10px;
  border: 1px solid var(--color-border-subtle);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
  font-family: var(--font-ui);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
  text-align: center;
}
.card-action-btn:hover:not(.disabled) {
  background: var(--color-bg-surface);
  border-color: var(--color-border-default);
}
.card-action-btn.primary {
  background: var(--gradient-primary);
  color: white;
  border: none;
  font-weight: 600;
}
.card-action-btn.primary:hover:not(.disabled) {
  background: var(--gradient-primary-hover);
}
.card-action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.card-action-btn.primary.disabled {
  opacity: 0.6;
  cursor: default;
}
.card-action-btn:active:not(.disabled) { transform: scale(0.97); }
</style>
