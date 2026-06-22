<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { getCurrentWindow } from '@tauri-apps/api/window'
import logoUrl from '@/assets/icons/lingscape-icon.png'

const { t } = useI18n()
const router = useRouter()
const win = getCurrentWindow()

defineProps<{ scrolled?: boolean }>()
defineEmits<{ toggleSearch: [] }>()

function minimize() { win.minimize() }
function toggleMaximize() { win.toggleMaximize() }
function closeWindow() { win.close() }

function openSettings() {
  router.push('/settings')
}
</script>

<template>
  <header class="titlebar" :class="{ 'titlebar--scrolled': scrolled }" data-tauri-drag-region>
    <div class="titlebar-brand" data-tauri-drag-region>
      <img class="brand-icon" :src="logoUrl" width="22" height="22" alt="" />
      <span class="brand-name">{{ t('app.name') }}</span>
      <span class="brand-sub">LingScape</span>
    </div>

    <div class="titlebar-spacer" data-tauri-drag-region />

    <div class="titlebar-actions">
      <button class="tb-action-btn" :title="t('library.search')" @click="$emit('toggleSearch')">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
          <circle cx="7" cy="7" r="4.5" />
          <line x1="10.2" y1="10.2" x2="14" y2="14" />
        </svg>
      </button>
      <button class="tb-action-btn" :title="t('nav.settings')" @click="openSettings">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
          <circle cx="12" cy="12" r="3" />
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
  height: var(--titlebar-height);
  display: flex;
  align-items: center;
  flex-shrink: 0;
  z-index: 100;
  padding: 0 var(--spacing-4);
  background: transparent;
  backdrop-filter: none;
  border-bottom: 1px solid transparent;
  transition: background 0.25s ease, border-color 0.25s ease, backdrop-filter 0.25s ease;
  -webkit-app-region: drag;
}

.titlebar--scrolled {
  background: rgba(249, 252, 250, 0.72);
  backdrop-filter: blur(16px) saturate(1.2);
  border-bottom-color: rgba(203, 236, 218, 0.35);
}

.titlebar-brand {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  padding: var(--spacing-1) var(--spacing-3) var(--spacing-1) var(--spacing-2);
  flex-shrink: 0;
  cursor: default;
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

.brand-icon { flex-shrink: 0; border-radius: var(--radius-sm); }

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
  app-region: no-drag;
  transition: all 0.25s ease;
}
.titlebar-controls:hover {
  background: oklch(99% 0.004 163 / 0.75);
  border-color: oklch(90% 0.008 163 / 0.5);
  box-shadow: var(--shadow-sm);
}

.titlebar-actions {
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.tb-action-btn,
.tb-btn {
  -webkit-app-region: no-drag;
  app-region: no-drag;
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
