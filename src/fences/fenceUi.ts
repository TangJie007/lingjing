import { ref } from "vue";

interface ToastState {
  visible: boolean;
  message: string;
}

const state = ref<ToastState>({ visible: false, message: "" });
let timer: number | null = null;

export function showFenceToast(message: string, durationMs = 2400) {
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

/** Longer toast for failures so they can be read / compared with terminal logs. */
export function showFenceErrorToast(message: string, durationMs = 8000) {
  console.error("[fence]", message);
  showFenceToast(message, durationMs);
}

export function useFenceToast() {
  return { state, showFenceToast };
}

export function friendlyError(e: unknown): string {
  const msg = String(e).replace(/^Error:\s*/i, "").trim();
  if (/超时|timeout/i.test(msg)) return "操作超时，请重试";
  if (/not allowed|forbidden|denied by acl|permission/i.test(msg) && !/access/i.test(msg)) {
    return "操作未被允许，请检查应用权限配置";
  }
  if (/command .+ not found|unknown command/i.test(msg)) {
    return "功能未就绪，请重新启动应用";
  }
  if (/不存在|not found/i.test(msg)) return "文件不存在或已被移动";
  if (/拒绝|access|denied|permission/i.test(msg)) return "无权访问该文件";
  if (/非法字符|invalid/i.test(msg)) return "名称包含非法字符";
  if (/已存在|exists/i.test(msg)) return "目标名称已存在";
  // Keep share / shell errors readable (still cap runaway PowerShell dumps).
  if (/共享|share|powershell|未找到共享/i.test(msg)) {
    return msg.length > 240 ? `${msg.slice(0, 237)}…` : msg;
  }
  return msg.length > 96 ? `${msg.slice(0, 93)}…` : msg;
}

export const BUILTIN_DELETE = 0xf0000000 + 10;
export const BUILTIN_RENAME = 0xf0000000 + 11;

export function confirmDelete(name: string): boolean {
  const label = name.trim() || "此项目";
  return window.confirm(`确定将「${label}」移到回收站吗？\n\n此操作可在回收站中恢复。`);
}
