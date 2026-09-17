<script setup lang="ts">
import { computed } from "vue";
import { useOnlineDownloadProgress } from "../composables/useOnlineDownloadProgress";

const { state, ratio, percentLabel, sizeLabel } = useOnlineDownloadProgress();

const barStyle = computed(() => {
  if (ratio.value == null) return undefined;
  return { transform: `scaleX(${ratio.value})` };
});
</script>

<template>
  <div
    class="dl-progress"
    :class="{ show: state.active, indeterminate: ratio == null && state.phase === 'downloading' }"
    role="status"
    aria-live="polite"
    :aria-busy="state.active"
  >
    <div class="dl-progress-meta">
      <span class="dl-progress-label">{{ state.label }}</span>
      <span class="dl-progress-pct">{{ percentLabel ?? sizeLabel }}</span>
    </div>
    <div class="dl-progress-track" aria-hidden="true">
      <div class="dl-progress-fill" :class="{ pulse: ratio == null }" :style="barStyle" />
    </div>
    <div v-if="percentLabel" class="dl-progress-sub">{{ sizeLabel }}</div>
  </div>
</template>
