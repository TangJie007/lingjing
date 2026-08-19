<script setup lang="ts">
import { useToast } from "../composables/useToast";

const { state } = useToast();
</script>

<template>
  <div class="toast-pop" :class="{ show: state.visible }" role="status" aria-live="polite">
    <span class="dot" />
    <span>{{ state.message }}</span>
  </div>
</template>
