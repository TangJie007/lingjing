<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '@/stores/settings'

const { t } = useI18n()
const settings = useSettingsStore()
</script>

<template>
  <div class="page-settings">
    <h1>{{ t('settings.title') }}</h1>

    <!-- 通用 -->
    <section class="settings-section">
      <h2 class="settings-section-title">{{ t('settings.general') }}</h2>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ t('settings.autoStart') }}</span>
          <span class="setting-desc">{{ t('settings.autoStartDesc') }}</span>
        </div>
        <button
          class="toggle"
          :class="{ active: settings.autoStart }"
          @click="settings.autoStart = !settings.autoStart"
        >
          <span class="toggle-knob" />
        </button>
      </div>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ t('settings.language') }}</span>
          <span class="setting-desc">{{ t('settings.languageDesc') }}</span>
        </div>
        <select v-model="settings.locale" class="setting-select">
          <option value="zh-CN">{{ t('settings.langZhCN') }}</option>
          <option value="en" disabled>{{ t('settings.langEn') }}</option>
        </select>
      </div>
    </section>

    <!-- 壁纸 -->
    <section class="settings-section">
      <h2 class="settings-section-title">{{ t('settings.wallpaper') }}</h2>
      <div class="setting-row">
        <span class="setting-label">{{ t('settings.fps') }}</span>
        <select v-model.number="settings.videoFps" class="setting-select">
          <option :value="30">30 FPS</option>
          <option :value="60">60 FPS</option>
        </select>
      </div>
      <div class="setting-row">
        <span class="setting-label">{{ t('settings.scaling') }}</span>
        <select v-model="settings.scalingMode" class="setting-select">
          <option value="fill">{{ t('settings.scalingFill') }}</option>
          <option value="fit">{{ t('settings.scalingFit') }}</option>
          <option value="stretch">{{ t('settings.scalingStretch') }}</option>
          <option value="tile">{{ t('settings.scalingTile') }}</option>
        </select>
      </div>
    </section>

    <!-- 性能 -->
    <section class="settings-section">
      <h2 class="settings-section-title">{{ t('settings.performance') }}</h2>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ t('settings.pauseOnFullscreen') }}</span>
          <span class="setting-desc">{{ t('settings.pauseOnFullscreenDesc') }}</span>
        </div>
        <button
          class="toggle"
          :class="{ active: settings.pauseOnFullscreen }"
          @click="settings.pauseOnFullscreen = !settings.pauseOnFullscreen"
        >
          <span class="toggle-knob" />
        </button>
      </div>
    </section>

    <!-- 存储 -->
    <section class="settings-section">
      <h2 class="settings-section-title">{{ t('settings.storage') }}</h2>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ t('settings.cacheLimit') }}</span>
          <span class="setting-desc">{{ t('settings.cacheLimitDesc') }}</span>
        </div>
        <input
          v-model.number="settings.cacheLimitGB"
          type="number"
          min="1"
          max="50"
          class="setting-input"
        />
      </div>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ t('settings.clearCache') }}</span>
          <span class="setting-desc">{{ t('settings.clearCacheDesc') }}</span>
        </div>
        <button class="btn-danger">{{ t('settings.clearCache') }}</button>
      </div>
    </section>

    <!-- 关于 -->
    <section class="settings-section">
      <h2 class="settings-section-title">{{ t('settings.about') }}</h2>
      <div class="setting-row">
        <span class="setting-label">{{ t('settings.version') }}</span>
        <span class="setting-desc">1.0.0</span>
      </div>
      <div class="setting-row">
        <span class="setting-desc">{{ t('settings.copyright') }}</span>
      </div>
    </section>
  </div>
</template>

<style scoped>
.page-settings { display: flex; flex-direction: column; gap: var(--spacing-6); max-width: 640px; }
.page-settings h1 {
  font-family: var(--font-display);
  font-size: var(--text-2xl);
  font-weight: 600;
  color: var(--color-text-primary);
}
.settings-section { display: flex; flex-direction: column; gap: var(--spacing-3); }
.settings-section-title {
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--color-text-primary);
  padding-bottom: var(--spacing-2);
  border-bottom: 1px solid var(--color-border-subtle);
}
.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-2) 0;
}
.setting-info { display: flex; flex-direction: column; gap: 2px; }
.setting-label { font-size: var(--text-sm); color: var(--color-text-primary); font-weight: 500; }
.setting-desc { font-size: var(--text-xs); color: var(--color-text-tertiary); }
.setting-select {
  padding: var(--spacing-1) var(--spacing-3);
  border-radius: var(--radius-sm);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border-subtle);
  color: var(--color-text-primary);
  font-size: var(--text-sm);
  outline: none;
  cursor: pointer;
}
.setting-input {
  padding: var(--spacing-1) var(--spacing-3);
  border-radius: var(--radius-sm);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border-subtle);
  color: var(--color-text-primary);
  font-size: var(--text-sm);
  outline: none;
  width: 80px;
}
.toggle {
  position: relative;
  width: 40px;
  height: 22px;
  border-radius: var(--radius-full);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border-default);
  cursor: pointer;
  transition: all 0.25s ease;
}
.toggle.active {
  background: var(--color-primary);
  border-color: var(--color-primary-dark);
}
.toggle-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--color-text-primary);
  transition: transform 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.toggle.active .toggle-knob { transform: translateX(18px); }
.btn-danger {
  padding: var(--spacing-2) var(--spacing-4);
  border-radius: var(--radius-md);
  border: 1px solid oklch(45% 0.08 20);
  background: oklch(25% 0.04 20);
  color: var(--color-error);
  font-size: var(--text-sm);
  cursor: pointer;
}
</style>
