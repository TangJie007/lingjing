<script setup lang="ts">
// OB-004: 顶部常驻 API 配置引导条
// 当用户跳过了 API 配置时显示，引导用户去设置页配置 API Key
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

const { t } = useI18n()
const router = useRouter()

function goToSettings() {
  router.push('/settings')
}
</script>

<template>
  <div class="api-banner">
    <div class="api-banner-icon">🔑</div>
    <span class="api-banner-text">{{ t('banner.apiNotConfigured') }}</span>
    <button class="api-banner-btn" @click="goToSettings">
      {{ t('banner.configureNow') }}
    </button>
  </div>
</template>

<style scoped>
.api-banner {
  display: flex;
  align-items: center;
  gap: var(--spacing-3);
  padding: var(--spacing-2) var(--spacing-4);
  background: linear-gradient(135deg, oklch(96% 0.02 90 / 0.9), oklch(97% 0.01 163 / 0.9));
  border-bottom: 1px solid oklch(90% 0.04 90 / 0.3);
  font-size: var(--text-sm);
  color: var(--color-text-primary);
  flex-shrink: 0;
}
.api-banner-icon {
  font-size: var(--text-base);
  flex-shrink: 0;
}
.api-banner-text {
  flex: 1;
  color: var(--color-text-secondary);
}
.api-banner-btn {
  padding: var(--spacing-1) var(--spacing-3);
  border-radius: var(--radius-sm);
  background: var(--gradient-primary);
  color: white;
  border: none;
  font-family: var(--font-ui);
  font-size: var(--text-xs);
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.2s ease;
}
.api-banner-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(0, 131, 54, 0.3);
}
</style>
