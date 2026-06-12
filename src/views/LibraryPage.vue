<script setup lang="ts">
import { useI18n } from 'vue-i18n'
const { t } = useI18n()
</script>

<template>
  <div class="page-library">
    <h1>{{ t('library.title') }}</h1>
    <p class="empty-text">{{ t('common.empty') }}</p>
  </div>
</template>

<style scoped>
.page-library { max-width: 100%; }
.page-library h1 {
  font-family: var(--font-heading);
  font-size: var(--text-2xl);
  font-weight: 600;
  color: var(--color-text-primary);
  margin-bottom: var(--spacing-4);
}
.empty-text { color: var(--color-text-tertiary); font-size: var(--text-sm); }
</style>
