<script setup lang="ts">
// 侧边栏导航 (220px)
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'
import { computed } from 'vue'
import { useApiKeysStore } from '@/stores/api-keys'

const { t } = useI18n()
const route = useRoute()
const apiKeysStore = useApiKeysStore()

const navItems = computed(() => [
  { path: '/ai-create', icon: '✨', label: t('nav.aiCreate') },
  { path: '/library', icon: '🖼️', label: t('nav.library') },
  { path: '/api-keys', icon: '🔑', label: t('nav.apiKeys') },
  { path: '/settings', icon: '⚙️', label: t('nav.settings') },
])

function isActive(path: string) {
  return route.path === path
}
</script>

<template>
  <aside class="sidebar">
    <div class="nav-section-label">{{ t('app.name') }}</div>
    <router-link
      v-for="item in navItems"
      :key="item.path"
      :to="item.path"
      class="nav-item"
      :class="{ active: isActive(item.path) }"
    >
      <span class="nav-icon">{{ item.icon }}</span>
      <span>{{ item.label }}</span>
    </router-link>

    <div class="nav-spacer" />

    <!-- API 连接状态 -->
    <div class="api-status">
      <div class="status-row">
        <span class="status-dot" :class="{ offline: !apiKeysStore.hasConfiguredKey }" />
        <span>{{ apiKeysStore.hasConfiguredKey ? t('status.apiConnected') : t('status.apiDisconnected') }}</span>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: var(--sidebar-width);
  background: oklch(98% 0.005 163 / 0.78);
  backdrop-filter: blur(28px) saturate(1.3);
  border-right: 1px solid oklch(92% 0.005 163 / 0.5);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  padding: var(--spacing-3);
  gap: var(--spacing-1);
  padding-top: calc(var(--titlebar-height) + var(--spacing-3));
}
.nav-section-label {
  font-size: var(--text-xs);
  font-weight: 500;
  color: var(--color-text-tertiary);
  padding: var(--spacing-4) var(--spacing-3) var(--spacing-2);
  letter-spacing: 0.05em;
  text-transform: uppercase;
}
.nav-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-3);
  padding: var(--spacing-2) var(--spacing-3);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.12s ease;
  color: var(--color-text-secondary);
  font-size: var(--text-sm);
  text-decoration: none;
  border: 1px solid transparent;
  position: relative;
}
.nav-item:hover {
  background: var(--color-bg-surface);
  color: var(--color-text-primary);
}
.nav-item.active {
  background: var(--color-primary-surface);
  color: var(--color-primary-light);
  border-color: oklch(85% 0.03 163);
  font-weight: 500;
}
.nav-item.active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 18px;
  border-radius: 0 var(--radius-full) var(--radius-full) 0;
  background: var(--gradient-primary);
}
.nav-icon { width: 18px; text-align: center; flex-shrink: 0; }
.nav-spacer { flex: 1; }
.api-status {
  margin: var(--spacing-2);
  padding: var(--spacing-3);
  background: var(--color-bg-surface);
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border-subtle);
}
.status-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  font-size: var(--text-xs);
  color: var(--color-text-secondary);
}
.status-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--color-success);
  animation: pulse-glow 2s ease-in-out infinite;
}
.status-dot.offline { background: var(--color-text-tertiary); animation: none; }
</style>
