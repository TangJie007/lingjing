<script setup lang="ts">
defineProps<{
  label?: string;
}>();

const emit = defineEmits<{ (e: "click"): void }>();
</script>

<template>
  <button
    type="button"
    class="modal-close"
    :aria-label="label || '关闭'"
    :title="label || '关闭'"
    @click.stop="emit('click')"
  >
    <svg class="modal-close-icon" viewBox="0 0 12 12" aria-hidden="true">
      <path
        d="M2.2 2.2l7.6 7.6M9.8 2.2L2.2 9.8"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
      />
    </svg>
  </button>
</template>
