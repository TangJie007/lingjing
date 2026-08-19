<script setup lang="ts">
import OnlineLoginPanel from "./OnlineLoginPanel.vue";

defineProps<{ open: boolean }>();
const emit = defineEmits<{ (e: "close"): void; (e: "success"): void }>();
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="lm-mask" @click.self="emit('close')">
      <div class="lm-card" role="dialog" aria-modal="true" aria-label="登录灵境社区">
        <header>
          <h3>登录灵境社区</h3>
          <button type="button" class="lm-close" aria-label="关闭" @click="emit('close')">✕</button>
        </header>
        <OnlineLoginPanel @success="emit('success')" />
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.lm-mask {
  position: fixed;
  inset: 0;
  background: rgba(15, 17, 24, 0.45);
  backdrop-filter: blur(6px);
  display: grid;
  place-items: center;
  z-index: 1000;
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
  margin-bottom: 4px;
}
header h3 {
  font-size: 16px;
  font-weight: 700;
  margin: 0;
}
.lm-close {
  width: 28px;
  height: 28px;
  border: 1px solid var(--border);
  border-radius: var(--r-md);
  background: var(--surface-2);
  color: var(--text-2);
  cursor: pointer;
  font-size: 12px;
  line-height: 1;
}
.lm-close:hover {
  background: var(--surface);
  color: var(--text);
}
.lm-card :deep(.login-panel) {
  margin-top: 12px;
  border: none;
  padding: 0;
  background: transparent;
}
</style>
