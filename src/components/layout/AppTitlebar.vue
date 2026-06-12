<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { getCurrentWindow } from '@tauri-apps/api/window'

const { t } = useI18n()
const router = useRouter()
const win = getCurrentWindow()

defineEmits<{ toggleSearch: [] }>()

function minimize() { win.minimize() }
function toggleMaximize() { win.toggleMaximize() }
function closeWindow() { win.close() }

function openSettings() {
  router.push('/settings')
}
</script>

<template>
  <header class="titlebar">
    <div class="titlebar-brand">
      <svg class="brand-icon-svg" viewBox="0 0 48 48" width="22" height="22" fill="none" xmlns="http://www.w3.org/2000/svg">
        <defs>
          <filter id="logo-glow" x="-20%" y="-20%" width="140%" height="140%">
            <feGaussianBlur stdDeviation="1.5" result="blur" />
            <feMerge>
              <feMergeNode in="blur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
          <linearGradient id="logo-fill" x1="4" y1="36" x2="44" y2="8" gradientUnits="userSpaceOnUse">
            <stop offset="0%" stop-color="#fbbf24" />
            <stop offset="25%" stop-color="#a3e635" />
            <stop offset="55%" stop-color="#34d399" />
            <stop offset="85%" stop-color="#22d3ee" />
            <stop offset="100%" stop-color="#67e8f9" />
          </linearGradient>
          <radialGradient id="logo-highlight" cx="16" cy="14" r="18" gradientUnits="userSpaceOnUse">
            <stop offset="0%" stop-color="white" stop-opacity="0.35" />
            <stop offset="60%" stop-color="white" stop-opacity="0.08" />
            <stop offset="100%" stop-color="white" stop-opacity="0" />
          </radialGradient>
        </defs>
        <path
          d="M 24 3 C 14 3, 5 12, 5 24 C 5 35, 12 43, 20 45 C 28 47, 38 42, 41 33 C 44 24, 42 14, 34 8 C 30 5, 27 3, 24 3 Z"
          fill="url(#logo-fill)"
          filter="url(#logo-glow)"
        />
        <path
          d="M 24 6 C 28 6, 32 9, 35 15 C 38 21, 37 29, 33 35 C 30 39, 25 41, 20 40"
          fill="none"
          stroke="white"
          stroke-width="0.8"
          stroke-linecap="round"
          opacity="0.25"
        />
        <path
          d="M 24 3 C 14 3, 5 12, 5 24 C 5 35, 12 43, 20 45 C 28 47, 38 42, 41 33 C 44 24, 42 14, 34 8 C 30 5, 27 3, 24 3 Z"
          fill="url(#logo-highlight)"
        />
      </svg>
      <span class="brand-name">{{ t('app.name') }}</span>
      <span class="brand-sub">LingScape</span>
    </div>

    <div class="titlebar-spacer" />

    <div class="titlebar-actions">
      <button class="tb-action-btn" :title="t('library.search')" @click="$emit('toggleSearch')">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
          <circle cx="7" cy="7" r="4.5" />
          <line x1="10.2" y1="10.2" x2="14" y2="14" />
        </svg>
      </button>
      <button class="tb-action-btn" :title="t('nav.settings')" @click="openSettings">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="8" cy="8" r="2.5" />
          <path d="M8 1v1.5M8 13.5V15M1 8h1.5M13.5 8H15M3.05 3.05l1.06 1.06M11.89 11.89l1.06 1.06M3.05 12.95l1.06-1.06M11.89 4.11l1.06-1.06" />
        </svg>
      </button>
    </div>

    <div class="titlebar-controls">
      <button class="tb-btn minimize" :title="t('titlebar.minimize')" @click="minimize">
        <svg viewBox="0 0 14 14" fill="none">
          <line x1="2" y1="7" x2="12" y2="7" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
        </svg>
      </button>
      <button class="tb-btn maximize" :title="t('titlebar.maximize')" @click="toggleMaximize">
        <svg viewBox="0 0 14 14" fill="none">
          <rect x="2" y="2" width="9" height="9" rx="1.5" stroke="currentColor" stroke-width="1.5" />
        </svg>
      </button>
      <button class="tb-btn close" :title="t('titlebar.close')" @click="closeWindow">
        <svg viewBox="0 0 14 14" fill="none">
          <line x1="3" y1="3" x2="11" y2="11" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
          <line x1="11" y1="3" x2="3" y2="11" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
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
  flex-shrink: 0;
  background: oklch(99% 0.004 163 / 0.6);
  backdrop-filter: blur(20px) saturate(1.3);
  border-radius: var(--radius-full);
  border: 1px solid oklch(95% 0.005 163 / 0.5);
  transition: all 0.25s ease;
}
.titlebar-brand:hover {
  background: oklch(99% 0.004 163 / 0.8);
  border-color: oklch(90% 0.008 163 / 0.6);
}

.brand-icon-svg { flex-shrink: 0; }

.brand-name {
  font-family: var(--font-display);
  font-size: var(--text-base);
  font-weight: 600;
  letter-spacing: -0.01em;
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
  padding: 3px;
  background: oklch(99% 0.004 163 / 0.5);
  backdrop-filter: blur(20px) saturate(1.3);
  border-radius: var(--radius-full);
  border: 1px solid oklch(95% 0.005 163 / 0.4);
  -webkit-app-region: no-drag;
  transition: all 0.25s ease;
}
.titlebar-controls:hover {
  background: oklch(99% 0.004 163 / 0.75);
  border-color: oklch(90% 0.008 163 / 0.5);
  box-shadow: var(--shadow-sm);
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
.tb-btn svg { width: 14px; height: 14px; transition: all 0.12s ease; }
.tb-btn:hover { background: oklch(92% 0.008 163 / 0.6); color: oklch(30% 0.01 250); }
.tb-btn:active { transform: scale(0.9); }
.tb-btn.minimize:hover { background: oklch(92% 0.03 163 / 0.5); color: oklch(45% 0.08 163); }
.tb-btn.maximize:hover { background: oklch(92% 0.03 230 / 0.5); color: oklch(45% 0.08 230); }
.tb-btn.close:hover {
  background: oklch(60% 0.2 20 / 0.9);
  color: white;
  box-shadow: 0 2px 8px oklch(60% 0.2 20 / 0.3);
}
.tb-btn.close:hover svg { transform: rotate(90deg); }
</style>
