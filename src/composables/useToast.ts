// useToast.ts
// 灵镜 · 全局 Toast（单例 ref 管理，自动 2400ms 消失）

import { ref } from "vue";

interface ToastState {
  visible: boolean;
  message: string;
}

const state = ref<ToastState>({ visible: false, message: "" });
let timer: number | null = null;

export function showToast(message: string, durationMs = 2400) {
  state.value.message = message;
  state.value.visible = true;
  if (timer !== null) {
    clearTimeout(timer);
    timer = null;
  }
  timer = window.setTimeout(() => {
    state.value.visible = false;
    timer = null;
  }, durationMs);
}

export function useToast() {
  return { state, showToast };
}
