<script setup lang="ts">
defineProps<{ active?: boolean }>()
defineEmits<{ click: [] }>()
</script>

<template>
  <button
    class="fab-create"
    :class="{ active: active }"
    :title="$t('ai.title')"
    @click="$emit('click')"
  >
    <svg class="fab-icon" width="28" height="28" viewBox="0 0 28 28" fill="none" xmlns="http://www.w3.org/2000/svg">
      <defs>
        <linearGradient id="fab-grad" x1="0" y1="28" x2="28" y2="0" gradientUnits="userSpaceOnUse">
          <stop offset="0%" stop-color="#a3e635" />
          <stop offset="50%" stop-color="#34d399" />
          <stop offset="100%" stop-color="#22d3ee" />
        </linearGradient>
      </defs>
      <path d="M14 2 L14 26 M2 14 L26 14" stroke="url(#fab-grad)" stroke-width="2.5" stroke-linecap="round" />
      <circle cx="14" cy="14" r="12" stroke="url(#fab-grad)" stroke-width="1.2" fill="none" opacity="0.3" />
    </svg>
    <div class="fab-ring" />
    <div class="fab-ring fab-ring-2" />
  </button>
</template>
