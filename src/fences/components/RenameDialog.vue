<script setup lang="ts">
import {
  cancelRenameDialog,
  confirmRenameDialog,
  useRenameDialog,
} from "../useRenameDialog";

const { open, value } = useRenameDialog();
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="rename-dialog-mask"
      @mousedown.self="cancelRenameDialog"
    >
      <div
        class="rename-dialog"
        role="dialog"
        aria-modal="true"
        aria-label="重命名"
      >
        <div class="rename-dialog-title">重命名</div>
        <input
          v-model="value"
          class="rename-dialog-input"
          type="text"
          spellcheck="false"
          @keydown.enter.prevent="confirmRenameDialog"
          @keydown.escape.prevent="cancelRenameDialog"
        />
        <div class="rename-dialog-actions">
          <button type="button" class="rename-dialog-btn" @click="cancelRenameDialog">
            取消
          </button>
          <button type="button" class="rename-dialog-btn primary" @click="confirmRenameDialog">
            确定
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.rename-dialog-mask {
  position: fixed;
  inset: 0;
  z-index: 10000;
  display: grid;
  place-items: center;
  background: rgba(0, 0, 0, 0.42);
  backdrop-filter: blur(4px);
}

.rename-dialog {
  width: 360px;
  max-width: calc(100vw - 32px);
  padding: 16px 18px 14px;
  border-radius: 10px;
  background: rgba(32, 32, 34, 0.96);
  border: 1px solid rgba(255, 255, 255, 0.12);
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.45);
  color: #f5f5f7;
}

.rename-dialog-title {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 12px;
}

.rename-dialog-input {
  width: 100%;
  box-sizing: border-box;
  height: 34px;
  padding: 0 10px;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.18);
  background: rgba(0, 0, 0, 0.28);
  color: inherit;
  font: inherit;
  outline: none;
}

.rename-dialog-input:focus {
  border-color: rgba(10, 132, 255, 0.85);
  box-shadow: 0 0 0 2px rgba(10, 132, 255, 0.25);
}

.rename-dialog-input:disabled {
  opacity: 0.6;
}

.rename-dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 14px;
}

.rename-dialog-btn {
  min-width: 72px;
  height: 30px;
  padding: 0 14px;
  border-radius: 6px;
  border: 1px solid rgba(255, 255, 255, 0.14);
  background: rgba(255, 255, 255, 0.06);
  color: inherit;
  font: inherit;
  font-size: 13px;
  cursor: default;
}

.rename-dialog-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.1);
}

.rename-dialog-btn.primary {
  border-color: rgba(10, 132, 255, 0.55);
  background: rgba(10, 132, 255, 0.88);
  color: #fff;
}

.rename-dialog-btn.primary:hover:not(:disabled) {
  background: rgba(10, 132, 255, 1);
}

.rename-dialog-btn:disabled {
  opacity: 0.55;
  pointer-events: none;
}
</style>
