<script setup lang="ts">
// 状态栏 — API 状态 + 生成历史
import { useI18n } from 'vue-i18n'
import { useApiKeysStore } from '@/stores/api-keys'

const { t } = useI18n()
const apiKeysStore = useApiKeysStore()
</script>

<template>
  <footer class="statusbar">
    <div class="status-item">
      <span class="status-dot" :class="{ offline: !apiKeysStore.hasConfiguredKey }" />
      <span>{{ apiKeysStore.hasConfiguredKey ? t('status.apiConnected') : t('status.apiDisconnected') }}</span>
    </div>
    <span class="status-sep" />
    <div class="status-item">
      <span>{{ t('app.name') }} v1.0.0</span>
    </div>
  </footer>
</template>

<style scoped>
.statusbar {
  height: var(--statusbar-height);
  display: flex;
  align-items: center;
  padding: 0 var(--spacing-4);
  background: var(--color-bg-deep);
  border-top: 1px solid var(--color-border-subtle);
  font-size: var(--text-xs);
  color: var(--color-text-tertiary);
  gap: var(--spacing-4);
  flex-shrink: 0;
}
.status-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-1);
}
.status-sep {
  width: 1px;
  height: 12px;
  background: var(--color-border-subtle);
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
