<script setup lang="ts">
// 壁纸卡片组件 — 展示壁纸缩略图 + 始终可见的操作按钮
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { WallpaperItem } from '@/stores/wallpaper'

const props = defineProps<{
  wallpaper: WallpaperItem
  isActive?: boolean
  busy?: boolean
}>()

const emit = defineEmits<{
  apply: [id: string]
  delete: [id: string]
  export: [id: string]
}>()

const { t } = useI18n()
const imgError = ref(false)

const isVideo = computed(() => props.wallpaper.mediaType === 'video')
const isAi = computed(() => props.wallpaper.source === 'ai')

const badgeText = computed(() => {
  if (isVideo.value) return t('library.videoBadge')
  if (isAi.value) return t('library.aiBadge')
  return null
})

const badgeClass = computed(() => {
  if (isVideo.value) return 'video'
  if (isAi.value) return 'ai'
  return ''
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
</script>

<template>
  <div
    class="lib-card"
    :class="{ 'is-active': isActive }"
  >
    <div class="lib-card-img">
      <template v-if="!imgError && !isVideo">
        <img
          :src="wallpaper.path"
          :alt="wallpaper.filename"
          @error="imgError = true"
        />
      </template>
      <template v-else-if="isVideo">
        <div class="video-placeholder">
          <svg width="48" height="48" viewBox="0 0 48 48" fill="none">
            <circle cx="24" cy="24" r="22" stroke="currentColor" stroke-width="1.5" opacity="0.4" />
            <polygon points="19,14 19,34 35,24" fill="currentColor" opacity="0.7" />
          </svg>
          <span class="video-label">{{ wallpaper.format.toUpperCase() }}</span>
        </div>
      </template>
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
      <div v-if="badgeText || isActive" class="lib-card-badges">
        <span v-if="badgeText" class="lib-badge" :class="badgeClass">{{ badgeText }}</span>
        <span v-if="isActive" class="lib-badge active">{{ t('library.inUse') }}</span>
      </div>
    </div>

    <!-- 信息行 -->
    <div class="lib-card-info">
      <div class="lib-card-title-row">
        <span class="lib-card-title">{{ wallpaper.filename }}</span>
        <span v-if="isAi" class="lib-card-tag">AI</span>
      </div>
      <div class="lib-card-meta">
        <span>{{ sizeText }} · {{ dateText }}</span>
      </div>
    </div>

    <!-- 始终可见的操作按钮 -->
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
        :disabled="busy"
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
}
.lib-card-img img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.6s cubic-bezier(0.23, 1, 0.32, 1);
  display: block;
}
.lib-card:hover .lib-card-img img { transform: scale(1.06); }

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
.lib-badge.video {
  background: rgba(0, 1, 2, 0.72);
  color: #d1dce9;
  border: 1px solid rgba(5, 7, 10, 0.3);
}
.lib-badge.ai {
  background: linear-gradient(135deg, rgba(74, 30, 178, 0.9), rgba(93, 17, 86, 0.9));
  color: white;
}

.video-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: oklch(96% 0.003 163);
  color: var(--color-text-tertiary);
  gap: var(--spacing-2);
}
.video-label {
  font-size: var(--text-xs);
  font-family: var(--font-mono);
  color: var(--color-text-tertiary);
}
.img-fallback {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: oklch(97% 0.004 163);
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
.lib-card-tag {
  display: inline-block;
  padding: 2px 10px;
  border-radius: 20px;
  background: rgba(203, 236, 218, 0.8);
  color: #003014;
  font-size: 11px;
  font-family: var(--font-ui);
  font-weight: 500;
  flex-shrink: 0;
  border: 1px solid rgba(145, 186, 164, 0.4);
}
.lib-card-meta {
  font-size: 12px;
  color: var(--color-text-tertiary);
  margin-top: 4px;
  font-family: var(--font-ui);
}

/* 始终可见的操作按钮 */
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
