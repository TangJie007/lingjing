<script setup lang="ts">
import { useI18n } from 'vue-i18n'
const { t } = useI18n()
</script>

<template>
  <div class="page-api">
    <h1>{{ t('apiKeys.title') }}</h1>
    <p class="empty-text">{{ t('common.empty') }}</p>
  </div>
</template>

<style scoped>
.page-api { max-width: 680px; }
.page-api h1 {
  font-family: var(--font-display);
  font-size: var(--text-2xl);
  font-weight: 600;
  color: var(--color-text-primary);
  margin-bottom: var(--spacing-4);
}
.empty-text { color: var(--color-text-tertiary); font-size: var(--text-sm); }
</style>
