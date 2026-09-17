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
  /** When true, detail/drawer download buttons bind to this progress. */
  forButton: boolean;
  label: string;
  id: string;
  downloaded: number;
  total: number | null;
  phase: OnlineDownloadPhase;
}

const state = ref<ProgressState>({
  active: false,
  forButton: false,
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

export async function beginOnlineDownloadProgress(
  label: string,
  id?: string,
  options?: { forButton?: boolean },
) {
  clearHideTimer();
  await ensureListening();
  state.value = {
    active: true,
    forButton: options?.forButton === true,
    label: label.trim() || "正在下载",
    id: id ? String(id) : "",
    downloaded: 0,
    total: null,
    phase: "resolving",
  };
}

export function endOnlineDownloadProgress(delayMs = 0) {
  clearHideTimer();
  if (delayMs <= 0) {
    state.value = { ...state.value, active: false, forButton: false, phase: "idle" };
    return;
  }
  hideTimer = window.setTimeout(() => {
    state.value = { ...state.value, active: false, forButton: false, phase: "idle" };
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

  function isDownloadingItem(itemId?: string | null) {
    if (!state.value.active || !state.value.forButton) return false;
    if (!itemId) return false;
    return state.value.id === String(itemId);
  }

  function downloadButtonLabel(itemId?: string | null, idle = "↓ 下载") {
    if (!isDownloadingItem(itemId)) return idle;
    const pct = percentLabel.value;
    if (pct) return `↓ ${pct}`;
    if (state.value.phase === "resolving") return "↓ 准备中…";
    if (state.value.phase === "done" || state.value.phase === "cached") return "↓ 完成";
    if (state.value.downloaded > 0) return `↓ ${formatBytes(state.value.downloaded)}`;
    return "↓ 下载中…";
  }

  return {
    state,
    ratio,
    percentLabel,
    isDownloadingItem,
    downloadButtonLabel,
  };
}
