<script setup lang="ts">
defineProps<{
  compact?: boolean;
}>();
</script>

<template>
  <div
    class="shell-ctx-loading"
    :class="{ compact }"
    role="status"
    aria-label="加载中"
  >
    <span class="shell-ctx-spinner" />
  </div>
</template>
