<script setup lang="ts">
import { useI18n } from 'vue-i18n'
const { t } = useI18n()
</script>

<template>
  <div class="page-ai-create">
    <div class="create-header">
      <h1>{{ t('ai.title') }}</h1>
      <span class="subtitle">{{ t('ai.subtitle') }}</span>
    </div>
    <div class="empty-guide">
      <div class="guide-visual">✨</div>
      <h3>{{ t('ai.emptyGuide') }}</h3>
      <p>{{ t('ai.emptyGuideCta') }}</p>
    </div>
  </div>
</template>

<style scoped>
.page-ai-create { display: flex; flex-direction: column; gap: var(--spacing-6); max-width: 860px; }
.create-header { display: flex; align-items: baseline; gap: var(--spacing-3); }
.create-header h1 {
  font-family: var(--font-display);
  font-size: var(--text-2xl);
  font-weight: 600;
  color: var(--color-text-primary);
}
.subtitle { font-size: var(--text-sm); color: var(--color-text-secondary); }
.empty-guide {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-16) var(--spacing-4);
  text-align: center;
}
.guide-visual {
  width: 120px;
  height: 120px;
  border-radius: var(--radius-xl);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border-subtle);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 40px;
  margin-bottom: var(--spacing-5);
}
.empty-guide h3 {
  font-family: var(--font-display);
  font-size: var(--text-lg);
  font-weight: 600;
  color: var(--color-text-primary);
  margin-bottom: var(--spacing-2);
}
.empty-guide p {
  font-size: var(--text-sm);
  color: var(--color-text-secondary);
  max-width: 400px;
}
</style>
