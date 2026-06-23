<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useDesktopOrganizerStore } from '@/stores/desktop-organizer'
import Toast from '@/components/common/Toast.vue'
import { ref } from 'vue'

const { t } = useI18n()
const store = useDesktopOrganizerStore()
const toastRef = ref<InstanceType<typeof Toast>>()

async function organize() {
  try {
    const result = await store.organizeDesktop()
    toastRef.value?.show(
      'success',
      t('desktop.organizeSuccess', { count: result.arranged }),
    )
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    toastRef.value?.show('error', t('desktop.organizeFailed', { msg }))
  }
}
</script>

<template>
  <div class="organizer-simple">
    <div class="organizer-card">
      <div class="organizer-icon">🗂️</div>
      <h1 class="organizer-title">{{ t('desktop.title') }}</h1>
      <p class="organizer-desc">{{ t('desktop.desc') }}</p>
      <button
        class="organizer-btn"
        :disabled="store.organizing"
        @click="organize"
      >
        <span v-if="store.organizing" class="btn-spinner" />
        <span>{{ store.organizing ? t('desktop.organizing') : t('desktop.organizeBtn') }}</span>
      </button>
      <p v-if="store.lastResult" class="organizer-result">
        {{ t('desktop.lastResult', { count: store.lastResult.arranged }) }}
      </p>
    </div>
    <Toast ref="toastRef" />
  </div>
</template>

<style scoped>
.organizer-simple {
  min-height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px 24px;
}
.organizer-card {
  text-align: center;
  max-width: 360px;
  width: 100%;
}
.organizer-icon {
  font-size: 48px;
  margin-bottom: 16px;
  opacity: 0.85;
}
.organizer-title {
  font-size: var(--text-xl);
  font-weight: 700;
  color: var(--color-text-primary);
  margin-bottom: 8px;
  font-family: var(--font-heading);
}
.organizer-desc {
  font-size: var(--text-sm);
  color: var(--color-text-tertiary);
  margin-bottom: 28px;
  line-height: 1.6;
}
.organizer-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  padding: 14px 24px;
  border: none;
  border-radius: var(--radius-lg);
  background: var(--gradient-primary);
  color: white;
  font-size: 16px;
  font-weight: 600;
  font-family: var(--font-ui);
  cursor: pointer;
  transition: filter 0.15s;
}
.organizer-btn:hover:not(:disabled) { filter: brightness(1.05); }
.organizer-btn:disabled { opacity: 0.7; cursor: not-allowed; }
.organizer-result {
  margin-top: 16px;
  font-size: var(--text-sm);
  color: var(--color-text-secondary);
}
.btn-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid rgba(255,255,255,0.4);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }
</style>
