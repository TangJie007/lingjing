<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const activeCategory = ref('all')
const liveOnly = ref(false)

const categories = [
  { key: 'all', label: t('library.all') },
  { key: 'recent', label: t('library.sortRecent') },
]

// 暂无壁纸数据 — 展示空状态
const wallpapers: never[] = []
</script>

<template>
  <div class="page-library">
    <div class="lib-categories">
      <template v-for="cat in categories" :key="cat.key">
        <div v-if="cat.divider" class="lib-cat-divider" />
        <button
          v-else
          type="button"
          class="lib-cat-pill"
          :class="{ active: activeCategory === cat.key, accent: cat.accent }"
          @click="activeCategory = cat.key"
        >
          {{ cat.label }}
        </button>
      </template>
    </div>

    <div class="lib-meta-bar">
      <div class="lib-stats">
        <span>{{ t('library.totalCount', { count: wallpapers.length }) }}</span>
      </div>
      <div class="lib-controls">
        <button type="button" class="lib-control-btn">
          <span>{{ t('library.sortDefault') }}</span>
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9" /></svg>
        </button>
        <button type="button" class="lib-control-btn">
          <span>{{ t('library.gridView') }}</span>
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9" /></svg>
        </button>
        <label class="lib-control-check">
          <input v-model="liveOnly" type="checkbox" />
          <span class="check-dot" />
          <span>{{ t('library.liveWallpaper') }}</span>
        </label>
      </div>
    </div>

    <div v-if="wallpapers.length === 0" class="empty-state">
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
      <div class="empty-state-hint">
        <span class="hint-key">{{ t('library.emptyHintFlow') }}</span>
        <span class="hint-sep">·</span>
        <span class="hint-key">{{ t('library.emptyHintSteps') }}</span>
      </div>
    </div>

    <div v-else class="lib-grid" />
  </div>
</template>
