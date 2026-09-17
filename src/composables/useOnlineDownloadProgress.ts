import { computed, ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type OnlineDownloadPhase =
  | "resolving"
  | "downloading"
  | "done"
  | "cached"
  | "idle";

export interface OnlineDownloadProgressEvent {
  id: string;
  downloaded: number;
  total?: number | null;
  phase: OnlineDownloadPhase;
}

interface ProgressState {
  active: boolean;
  label: string;
  id: string;
  downloaded: number;
  total: number | null;
  phase: OnlineDownloadPhase;
}

const state = ref<ProgressState>({
  active: false,
  label: "",
  id: "",
  downloaded: 0,
  total: null,
  phase: "idle",
});

let unlisten: UnlistenFn | null = null;
let listenPromise: Promise<void> | null = null;
let hideTimer: number | null = null;

function clearHideTimer() {
  if (hideTimer !== null) {
    clearTimeout(hideTimer);
    hideTimer = null;
  }
}

export function formatBytes(n: number): string {
  if (!Number.isFinite(n) || n <= 0) return "0 B";
  if (n >= 1_073_741_824) return `${(n / 1_073_741_824).toFixed(1)} GB`;
  if (n >= 1_048_576) return `${(n / 1_048_576).toFixed(1)} MB`;
  if (n >= 1024) return `${Math.round(n / 1024)} KB`;
  return `${n} B`;
}

async function ensureListening() {
  if (unlisten || listenPromise) return listenPromise;
  listenPromise = (async () => {
    try {
      unlisten = await listen<OnlineDownloadProgressEvent>(
        "online-download-progress",
        (ev) => {
          const p = ev.payload;
          if (!state.value.active) return;
          if (state.value.id && p.id && state.value.id !== p.id) return;
          state.value = {
            ...state.value,
            id: p.id || state.value.id,
            downloaded: p.downloaded ?? 0,
            total: typeof p.total === "number" && p.total > 0 ? p.total : state.value.total,
            phase: p.phase || "downloading",
          };
          if (p.phase === "done" || p.phase === "cached") {
            state.value.phase = p.phase;
            if (p.phase === "cached" && !state.value.total) {
              state.value.downloaded = 1;
              state.value.total = 1;
            }
          }
        },
      );
    } catch (e) {
      console.warn("[download] listen failed", e);
      listenPromise = null;
    }
  })();
  return listenPromise;
}

/** Show the floating progress panel for an upcoming cache/download. */
export async function beginOnlineDownloadProgress(label: string, id?: string) {
  clearHideTimer();
  await ensureListening();
  state.value = {
    active: true,
    label: label.trim() || "正在下载",
    id: id ? String(id) : "",
    downloaded: 0,
    total: null,
    phase: "resolving",
  };
}

/** Hide the panel (optionally after a short delay so 100% is visible). */
export function endOnlineDownloadProgress(delayMs = 0) {
  clearHideTimer();
  if (delayMs <= 0) {
    state.value = { ...state.value, active: false, phase: "idle" };
    return;
  }
  hideTimer = window.setTimeout(() => {
    state.value = { ...state.value, active: false, phase: "idle" };
    hideTimer = null;
  }, delayMs);
}

export function useOnlineDownloadProgress() {
  const ratio = computed(() => {
    const total = state.value.total;
    if (!total || total <= 0) return null;
    return Math.min(1, state.value.downloaded / total);
  });

  const percentLabel = computed(() => {
    const r = ratio.value;
    if (r == null) return null;
    return `${Math.round(r * 100)}%`;
  });

  const sizeLabel = computed(() => {
    const { downloaded, total, phase } = state.value;
    if (phase === "resolving") return "准备中…";
    if (phase === "cached") return "已缓存";
    if (total && total > 0) {
      return `${formatBytes(downloaded)} / ${formatBytes(total)}`;
    }
    if (downloaded > 0) return formatBytes(downloaded);
    return "连接中…";
  });

  return { state, ratio, percentLabel, sizeLabel };
}
