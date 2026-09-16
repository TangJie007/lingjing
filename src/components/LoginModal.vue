<script setup lang="ts">
import { onMounted, onUnmounted, watch } from "vue";
import OnlineLoginPanel from "./OnlineLoginPanel.vue";
import AppLogo from "./AppLogo.vue";
import ModalCloseButton from "./ModalCloseButton.vue";

const props = defineProps<{ open: boolean }>();
const emit = defineEmits<{ (e: "close"): void; (e: "success"): void }>();

function onKeydown(e: KeyboardEvent) {
  if (e.key === "Escape" && props.open) {
    e.preventDefault();
    emit("close");
  }
}

watch(
  () => props.open,
  (open) => {
    if (open) window.addEventListener("keydown", onKeydown);
    else window.removeEventListener("keydown", onKeydown);
  },
);

onMounted(() => {
  if (props.open) window.addEventListener("keydown", onKeydown);
});
onUnmounted(() => {
  window.removeEventListener("keydown", onKeydown);
});
</script>

<template>
  <!-- Already under .window — no Teleport. Absolute overlay covers the shell. -->
  <div v-if="open" class="lm-mask" @click.self="emit('close')">
    <div class="lm-card" role="dialog" aria-modal="true" aria-label="登录灵境社区">
      <header>
        <div class="lm-title">
          <AppLogo :size="28" decorative />
          <h3>登录灵境社区</h3>
        </div>
        <ModalCloseButton @click="emit('close')" />
      </header>
      <OnlineLoginPanel @success="emit('success')" />
    </div>
  </div>
</template>

<style scoped>
.lm-mask {
  position: absolute;
  inset: 0;
  background: rgba(15, 17, 24, 0.45);
  backdrop-filter: blur(6px);
  display: grid;
  place-items: center;
  z-index: 1000;
  border-radius: inherit;
}
.lm-card {
  width: 400px;
  max-width: 92vw;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--r-lg);
  box-shadow: var(--sh-win);
  padding: 20px 22px 22px;
}
header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 4px;
}
.lm-title {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}
header h3 {
  font-size: 16px;
  font-weight: 700;
  margin: 0;
}
.lm-card :deep(.login-panel) {
  margin-top: 12px;
  border: none;
  padding: 0;
  background: transparent;
}
</style>
