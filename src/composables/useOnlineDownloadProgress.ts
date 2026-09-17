import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { isLocalOnlineDownloaded } from "./useLocalOnlineDownloads";

export type OnlineDownloadPhase =
  | "resolving"
  | "downloading"
  | "done"
  | "cached"
  | "idle"
  | "cancelled"
  | "failed";

export type DownloadOutcome = "success" | "failed" | "cancelled";

export interface OnlineDownloadProgressEvent {
  id: string;
  downloaded: number;
  total?: number | null;
  phase: OnlineDownloadPhase;
}

export interface DownloadJob {
  id: string;
  label: string;
  downloaded: number;
  total: number | null;
  phase: OnlineDownloadPhase;
  forButton: boolean;
  /** Set when the job has finished (kept in today's history). */
  outcome?: DownloadOutcome;
  error?: string;
  updatedAt: number;
}

interface DayStore {
  date: string;
  records: DownloadJob[];
}

const STORAGE_KEY = "lingscape.download.tray.v1";

const jobs = ref<DownloadJob[]>([]);

/** @deprecated single-job view kept for detail button helpers */
const state = computed(() => {
  const active =
    jobs.value.find((j) => isActivePhase(j.phase)) ?? jobs.value[0];
  if (!active) {
    return {
      active: false,
      forButton: false,
      label: "",
      id: "",
      downloaded: 0,
      total: null as number | null,
      phase: "idle" as OnlineDownloadPhase,
    };
  }
  return {
    active: isActivePhase(active.phase),
    forButton: active.forButton,
    label: active.label,
    id: active.id,
    downloaded: active.downloaded,
    total: active.total,
    phase: active.phase,
  };
});

let unlisten: UnlistenFn | null = null;
let listenPromise: Promise<void> | null = null;

function isActivePhase(phase: OnlineDownloadPhase): boolean {
  return phase === "resolving" || phase === "downloading";
}

function todayKey(d = new Date()): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

function isFinishedJob(job: DownloadJob): boolean {
  return (
    job.outcome != null ||
    job.phase === "done" ||
    job.phase === "cached" ||
    job.phase === "cancelled" ||
    job.phase === "failed"
  );
}

function readStoredDate(): string | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as DayStore;
    return typeof parsed?.date === "string" ? parsed.date : null;
  } catch {
    return null;
  }
}

/** Drop finished rows from a previous calendar day (keep in-flight jobs). */
function pruneIfNewDay() {
  const today = todayKey();
  const stored = readStoredDate();
  if (stored && stored !== today) {
    localStorage.removeItem(STORAGE_KEY);
    jobs.value = jobs.value.filter((j) => isActivePhase(j.phase) && !j.outcome);
  }
}

function persistFinishedJobs() {
  try {
    const today = todayKey();
    const records = jobs.value
      .filter(isFinishedJob)
      .map((j) => ({
        ...j,
        // Active flags should never stick on history rows.
        forButton: false,
        phase:
          j.outcome === "failed"
            ? ("failed" as const)
            : j.outcome === "cancelled"
              ? ("cancelled" as const)
              : j.phase === "cached"
                ? ("cached" as const)
                : ("done" as const),
      }));
    const store: DayStore = { date: today, records };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(store));
  } catch (e) {
    console.warn("[download] persist failed", e);
  }
}

function hydrateFromStorage() {
  pruneIfNewDay();
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return;
    const parsed = JSON.parse(raw) as DayStore;
    if (!parsed || parsed.date !== todayKey() || !Array.isArray(parsed.records)) {
      return;
    }
    const records = parsed.records.filter(
      (r): r is DownloadJob =>
        !!r &&
        typeof r === "object" &&
        typeof (r as DownloadJob).id === "string" &&
        typeof (r as DownloadJob).label === "string",
    );
    if (records.length === 0 || jobs.value.length > 0) return;
    jobs.value = records.map((r) => ({
      ...r,
      forButton: false,
      updatedAt: typeof r.updatedAt === "number" ? r.updatedAt : Date.now(),
    }));
  } catch {
    /* ignore */
  }
}

hydrateFromStorage();

function upsertJob(job: DownloadJob) {
  const idx = jobs.value.findIndex((j) => j.id === job.id);
  if (idx >= 0) {
    const copy = jobs.value.slice();
    copy[idx] = job;
    jobs.value = copy;
  } else {
    jobs.value = [...jobs.value, job];
  }
}

function patchJob(id: string, patch: Partial<DownloadJob>) {
  const idx = jobs.value.findIndex((j) => j.id === id);
  if (idx < 0) return;
  const copy = jobs.value.slice();
  copy[idx] = { ...copy[idx]!, ...patch, updatedAt: Date.now() };
  jobs.value = copy;
}

/** Update an in-flight job's byte progress (plugin-http driven downloads). */
export function reportOnlineDownloadProgress(
  id: string,
  downloaded: number,
  total?: number | null,
  phase: OnlineDownloadPhase = "downloading",
) {
  const jobId = String(id);
  const idx = jobs.value.findIndex((j) => j.id === jobId);
  if (idx < 0) return;
  const cur = jobs.value[idx]!;
  if (!isActivePhase(cur.phase) && cur.outcome) return;
  const copy = jobs.value.slice();
  copy[idx] = {
    ...cur,
    downloaded,
    total:
      typeof total === "number" && total > 0
        ? total
        : cur.total,
    phase,
    updatedAt: Date.now(),
  };
  jobs.value = copy;
}

const httpAbortControllers = new Map<string, AbortController>();

export function registerHttpDownloadAbort(id: string, ac: AbortController) {
  httpAbortControllers.set(String(id), ac);
}

export function clearHttpDownloadAbort(id: string) {
  httpAbortControllers.delete(String(id));
}

function abortHttpDownload(id: string) {
  const ac = httpAbortControllers.get(String(id));
  if (ac) {
    ac.abort();
    httpAbortControllers.delete(String(id));
  }
}

/** Archive a finished row that reuses the wallpaper id so a new attempt can start. */
function archiveFinishedIfNeeded(jobId: string) {
  const idx = jobs.value.findIndex((j) => j.id === jobId && isFinishedJob(j));
  if (idx < 0) return;
  const cur = jobs.value[idx]!;
  const copy = jobs.value.slice();
  copy[idx] = {
    ...cur,
    id: `${cur.id}#${cur.updatedAt || Date.now()}`,
    forButton: false,
  };
  jobs.value = copy;
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
          const id = String(p.id || "");
          if (!id) return;
          const idx = jobs.value.findIndex((j) => j.id === id);
          if (idx < 0) return;
          const cur = jobs.value[idx]!;
          if (!isActivePhase(cur.phase) && cur.outcome) return;
          const next: DownloadJob = {
            ...cur,
            downloaded: p.downloaded ?? cur.downloaded,
            total:
              typeof p.total === "number" && p.total > 0 ? p.total : cur.total,
            phase: p.phase || "downloading",
            updatedAt: Date.now(),
          };
          if (p.phase === "cached" && !next.total) {
            next.downloaded = 1;
            next.total = 1;
          }
          if (p.phase === "done" || p.phase === "cached") {
            next.outcome = "success";
            next.phase = p.phase;
          }
          const copy = jobs.value.slice();
          copy[idx] = next;
          jobs.value = copy;
          if (next.outcome) persistFinishedJobs();
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
  await ensureListening();
  pruneIfNewDay();
  const jobId = id ? String(id) : `job-${Date.now()}`;
  archiveFinishedIfNeeded(jobId);
  const job: DownloadJob = {
    id: jobId,
    label: label.trim() || "正在下载",
    downloaded: 0,
    total: null,
    phase: "resolving",
    forButton: options?.forButton === true,
    updatedAt: Date.now(),
  };
  upsertJob(job);
}

export function finishOnlineDownloadJob(
  id: string,
  result: { success: boolean; cancelled?: boolean; error?: string },
) {
  const jobId = String(id);
  const idx = jobs.value.findIndex((j) => j.id === jobId);
  if (idx < 0) return;

  const outcome: DownloadOutcome = result.cancelled
    ? "cancelled"
    : result.success
      ? "success"
      : "failed";
  const cur = jobs.value[idx]!;
  const copy = jobs.value.slice();
  copy[idx] = {
    ...cur,
    forButton: false,
    outcome,
    error: outcome === "failed" ? (result.error || "下载失败").trim() : undefined,
    phase:
      outcome === "cancelled"
        ? "cancelled"
        : outcome === "failed"
          ? "failed"
          : cur.phase === "cached"
            ? "cached"
            : "done",
    updatedAt: Date.now(),
  };
  jobs.value = copy;
  persistFinishedJobs();
}

/** @deprecated Prefer finishOnlineDownloadJob */
export function endOnlineDownloadProgress(delayMs = 0, id?: string) {
  const targetId = id || jobs.value[jobs.value.length - 1]?.id;
  if (!targetId) return;
  const job = jobs.value.find((j) => j.id === targetId);
  if (!job) return;
  if (job.outcome) return;
  if (delayMs > 0) {
    window.setTimeout(() => {
      finishOnlineDownloadJob(targetId, { success: true });
    }, delayMs);
  } else {
    finishOnlineDownloadJob(targetId, { success: true });
  }
}

/** @deprecated Prefer finishOnlineDownloadJob */
export function endOnlineDownloadJob(id: string, delayMs = 0) {
  endOnlineDownloadProgress(delayMs, id);
}

export async function cancelOnlineDownload(id: string): Promise<boolean> {
  const jobId = String(id);
  patchJob(jobId, { phase: "cancelled", outcome: "cancelled", forButton: false });
  persistFinishedJobs();
  abortHttpDownload(jobId);
  try {
    return await invoke<boolean>("cancel_online_download", { id: jobId });
  } catch (e) {
    console.warn("[download] cancel failed", e);
    return false;
  }
}

export function useOnlineDownloadProgress() {
  const activeJobs = computed(() =>
    jobs.value.filter((j) => isActivePhase(j.phase) && !j.outcome),
  );

  /** Today's records: active first, then finished newest-first. */
  const todayJobs = computed(() => {
    const active = jobs.value.filter((j) => isActivePhase(j.phase) && !j.outcome);
    const finished = jobs.value
      .filter((j) => isFinishedJob(j))
      .slice()
      .sort((a, b) => (b.updatedAt || 0) - (a.updatedAt || 0));
    return [...active, ...finished];
  });

  const activeCount = computed(() => activeJobs.value.length);

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

  function jobRatio(job: DownloadJob): number | null {
    if (job.outcome === "success" || job.phase === "done" || job.phase === "cached") {
      return 1;
    }
    if (!job.total || job.total <= 0) return null;
    return Math.min(1, job.downloaded / job.total);
  }

  function jobPercent(job: DownloadJob): string | null {
    const r = jobRatio(job);
    if (r == null) return null;
    return `${Math.round(r * 100)}%`;
  }

  function jobStatusText(job: DownloadJob): string {
    if (job.outcome === "cancelled" || job.phase === "cancelled") return "已取消";
    if (job.outcome === "failed" || job.phase === "failed") {
      return job.error ? `失败：${job.error}` : "失败";
    }
    if (job.outcome === "success" || job.phase === "done" || job.phase === "cached") {
      return "成功";
    }
    if (job.phase === "resolving") return "准备中…";
    const pct = jobPercent(job);
    if (pct) return pct;
    if (job.downloaded > 0) return formatBytes(job.downloaded);
    return "下载中…";
  }

  function isDownloadingItem(itemId?: string | null) {
    if (!itemId) return false;
    return jobs.value.some(
      (j) =>
        j.id === String(itemId) &&
        j.forButton &&
        isActivePhase(j.phase) &&
        !j.outcome,
    );
  }

  function isDownloadDisabled(itemId?: string | null) {
    return isDownloadingItem(itemId) || isLocalOnlineDownloaded(itemId);
  }

  function downloadButtonLabel(itemId?: string | null, idle = "↓ 下载") {
    if (isLocalOnlineDownloaded(itemId) && !isDownloadingItem(itemId)) {
      return "↓ 已下载";
    }
    if (!isDownloadingItem(itemId)) return idle;
    const job = jobs.value.find((j) => j.id === String(itemId) && !j.outcome);
    if (!job) return "↓ 下载中…";
    const pct = jobPercent(job);
    if (pct) return `↓ ${pct}`;
    if (job.phase === "resolving") return "↓ 准备中…";
    if (job.phase === "done" || job.phase === "cached") return "↓ 完成";
    if (job.downloaded > 0) return `↓ ${formatBytes(job.downloaded)}`;
    return "↓ 下载中…";
  }

  return {
    state,
    jobs,
    activeJobs,
    todayJobs,
    activeCount,
    ratio,
    percentLabel,
    jobRatio,
    jobPercent,
    jobStatusText,
    isDownloadingItem,
    isDownloadDisabled,
    downloadButtonLabel,
    cancelOnlineDownload,
    pruneIfNewDay,
  };
}
