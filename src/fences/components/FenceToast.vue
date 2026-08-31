<script setup lang="ts">
import { useFenceToast } from "../fenceUi";

const { state } = useFenceToast();
</script>

<template>
  <div class="fence-toast" :class="{ show: state.visible }" role="status" aria-live="polite">
    <span class="dot" />
    <span>{{ state.message }}</span>
  </div>
</template>
