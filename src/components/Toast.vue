<script setup lang="ts">
import { useToast } from "../composables/useToast";

const { state } = useToast();
</script>

<template>
  <div
    v-show="state.visible"
    class="toast-pop"
    role="status"
    aria-live="polite"
  >
    <span class="dot" />
    <span>{{ state.message }}</span>
  </div>
</template>

<style scoped>
.toast-pop {
  position: fixed;
  left: 50%;
  bottom: 36px;
  transform: translate(-50%, 20px);
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 13.5px;
  font-weight: 600;
  padding: 12px 20px;
  border-radius: var(--r-md);
  background: var(--text);
  color: var(--bg);
  box-shadow: var(--sh-win);
  opacity: 0;
  pointer-events: none;
  z-index: 50;
  animation: toastIn var(--dur-base) var(--ease) forwards;
}
.toast-pop .dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--success);
}
</style>
