<script setup lang="ts">
// 自定义标题栏 — 品牌 Logo + 窗口控制按钮
import { useI18n } from 'vue-i18n'
import { getCurrentWindow } from '@tauri-apps/api/window'

const { t } = useI18n()
const win = getCurrentWindow()

function minimize() {
  win.minimize()
}

function toggleMaximize() {
  win.toggleMaximize()
}

function closeWindow() {
  win.close()
}
</script>

<template>
  <header class="titlebar">
    <div class="titlebar-brand">
      <!-- 灵叶 Logo SVG (简化版) -->
      <svg class="brand-icon" viewBox="0 0 24 24" width="22" height="22">
        <defs>
          <linearGradient id="logo-grad" x1="2" y1="18" x2="22" y2="4">
            <stop offset="0%" stop-color="#fbbf24" />
            <stop offset="25%" stop-color="#a3e635" />
            <stop offset="55%" stop-color="#34d399" />
            <stop offset="85%" stop-color="#22d3ee" />
          </linearGradient>
        </defs>
        <path
          d="M12 2C7 2 3 6 3 12c0 5 3 9 7 10 4 1 9-1 10-5 2-4 1-9-3-12-2-1-3-2-5-3z"
          fill="url(#logo-grad)"
          opacity="0.9"
        />
      </svg>
      <span class="brand-name">{{ t('app.name') }}</span>
      <span class="brand-sub">LingScape</span>
    </div>

    <div class="titlebar-controls">
      <button class="tb-btn minimize" :title="t('titlebar.minimize')" @click="minimize">
        <svg viewBox="0 0 14 14"><path d="M2 11h10" stroke="currentColor" stroke-width="1.2" fill="none" /></svg>
      </button>
      <button class="tb-btn maximize" :title="t('titlebar.maximize')" @click="toggleMaximize">
        <svg viewBox="0 0 14 14"><rect x="2" y="2" width="10" height="10" stroke="currentColor" stroke-width="1.2" fill="none" /></svg>
      </button>
      <button class="tb-btn close" :title="t('titlebar.close')" @click="closeWindow">
        <svg viewBox="0 0 14 14">
          <path d="M3 3l8 8M11 3l-8 8" stroke="currentColor" stroke-width="1.2" fill="none" />
        </svg>
      </button>
    </div>
  </header>
</template>

<style scoped>
.titlebar {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: var(--titlebar-height);
  display: flex;
  align-items: center;
  background: transparent;
  -webkit-app-region: drag;
  flex-shrink: 0;
  z-index: 100;
  padding: 0 var(--spacing-4);
}
.titlebar-brand {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  padding: var(--spacing-1) var(--spacing-3) var(--spacing-1) var(--spacing-2);
  background: oklch(99% 0.004 163 / 0.6);
  backdrop-filter: blur(20px) saturate(1.3);
  border-radius: var(--radius-full);
  border: 1px solid oklch(95% 0.005 163 / 0.5);
}
.brand-icon { flex-shrink: 0; }
.brand-name {
  font-family: var(--font-display);
  font-size: var(--text-base);
  font-weight: 600;
  color: var(--color-text-primary);
}
.brand-sub {
  font-size: var(--text-xs);
  color: var(--color-text-tertiary);
  margin-left: var(--spacing-1);
  font-weight: 300;
}
.titlebar-controls {
  display: flex;
  align-items: center;
  gap: 2px;
  margin-left: auto;
  padding: 3px;
  background: oklch(99% 0.004 163 / 0.5);
  backdrop-filter: blur(20px) saturate(1.3);
  border-radius: var(--radius-full);
  border: 1px solid oklch(95% 0.005 163 / 0.4);
  -webkit-app-region: no-drag;
}
.tb-btn {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-full);
  border: none;
  background: transparent;
  color: oklch(50% 0.01 250 / 0.7);
  cursor: pointer;
  transition: all 0.12s ease;
}
.tb-btn:hover { background: oklch(92% 0.008 163 / 0.6); color: oklch(30% 0.01 250); }
.tb-btn.close:hover { background: oklch(60% 0.2 20 / 0.9); color: white; }
</style>
