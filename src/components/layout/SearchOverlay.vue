<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'

const open = defineModel<boolean>('open', { default: false })
const { t } = useI18n()
const inputRef = ref<HTMLInputElement>()
const query = ref('')

watch(open, async (visible) => {
  if (visible) {
    await nextTick()
    inputRef.value?.focus()
  } else {
    query.value = ''
  }
})

function close() {
  open.value = false
}
</script>

<template>
  <div class="search-overlay" :class="{ visible: open }">
    <div class="search-overlay-inner">
      <svg class="search-overlay-icon" width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
        <circle cx="9" cy="9" r="5.5" />
        <line x1="12.5" y1="12.5" x2="17" y2="17" />
      </svg>
      <input
        ref="inputRef"
        v-model="query"
        class="search-overlay-input"
        type="text"
        :placeholder="t('library.search')"
        autocomplete="off"
        @keydown.esc="close"
      />
      <button class="search-overlay-close" :title="t('common.close')" @click="close">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
          <line x1="3" y1="3" x2="13" y2="13" />
          <line x1="13" y1="3" x2="3" y2="13" />
        </svg>
      </button>
    </div>
  </div>
</template>
