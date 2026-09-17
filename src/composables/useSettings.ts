import { invoke } from "@tauri-apps/api/core";
import { ref, watch } from "vue";

/** 开发默认：本地社区 API */
export const DEV_API_BASE_URL = "http://localhost:8000";
/** 正式打包默认：线上社区 API */
export const PROD_API_BASE_URL = "https://36fa666671.eicp.vip";

/** 当前构建环境应对应的默认 API 基址（vite: DEV→本地，PROD→线上） */
export const DEFAULT_API_BASE_URL = import.meta.env.DEV
  ? DEV_API_BASE_URL
  : PROD_API_BASE_URL;

const LEGACY_API_BASE_URLS = new Set([
  "http://localhost:3002",
  "https://localhost:3002",
]);

export interface AppSettings {
  autostart: boolean;
  hideIconsOnDoubleClick: boolean;
  pauseOnFullscreen: boolean;
  pauseOnBattery: boolean;
  pauseOnRdp: boolean;
  soundOn: boolean;
  defaultVolume: number;
  importCopyToData: boolean;
  libraryDirOverride: string | null;
  loopMode: "list" | "single" | "random";
  onlineEnabled: boolean;
  desktopOrganizeEnabled: boolean;
  apiBaseUrl: string;
}

export const DEFAULT_SETTINGS: AppSettings = {
  autostart: false,
  hideIconsOnDoubleClick: false,
  pauseOnFullscreen: true,
  pauseOnBattery: true,
  pauseOnRdp: true,
  soundOn: true,
  defaultVolume: 0,
  importCopyToData: true,
  libraryDirOverride: null,
  loopMode: "single",
  onlineEnabled: false,
  desktopOrganizeEnabled: false,
  apiBaseUrl: DEFAULT_API_BASE_URL,
};

const settings = ref<AppSettings>({ ...DEFAULT_SETTINGS });
let loaded = false;
let writeTimer: number | null = null;

export function useSettings() {
  return settings;
}

/**
 * 空值 / 旧地址 / 另一环境的默认地址 → 当前环境默认。
 * 开发：线上默认也会被切回本地；打包：本地默认会切到线上。
 */
export function normalizeApiBaseUrl(url?: string | null): string {
  const trimmed = (url || "").trim().replace(/\/$/, "");
  if (!trimmed || LEGACY_API_BASE_URLS.has(trimmed)) {
    return DEFAULT_API_BASE_URL;
  }
  if (import.meta.env.DEV) {
    if (
      trimmed === PROD_API_BASE_URL ||
      trimmed === "http://36fa666671.eicp.vip"
    ) {
      return DEV_API_BASE_URL;
    }
  } else if (
    trimmed === DEV_API_BASE_URL ||
    trimmed === "http://127.0.0.1:8000"
  ) {
    return PROD_API_BASE_URL;
  }
  return trimmed;
}

export async function loadSettings(): Promise<AppSettings> {
  try {
    const remote = await invoke<AppSettings>("load_settings");
    settings.value = {
      ...DEFAULT_SETTINGS,
      ...remote,
      apiBaseUrl: normalizeApiBaseUrl(remote.apiBaseUrl),
      // 循环模式 UI 暂时隐藏，统一使用单曲循环
      loopMode: "single",
    };
  } catch {
    settings.value = { ...DEFAULT_SETTINGS };
  }
  if (!loaded) {
    watch(
      settings,
      (next) => {
        if (writeTimer !== null) {
          window.clearTimeout(writeTimer);
        }
        writeTimer = window.setTimeout(() => {
          void persistSettings(next);
        }, 220);
      },
      { deep: true },
    );
    loaded = true;
  }
  return settings.value;
}

async function persistSettings(next: AppSettings) {
  try {
    await invoke("save_settings", { payload: next });
  } catch (e) {
    console.warn("save_settings failed", e);
  }
}

export interface MigrationPlan {
  fromDir: string;
  toDir: string;
  files: Array<{ relPath: string; fromPath: string; toPath: string; size: number }>;
}

export interface MigrationReport {
  copied: number;
  skipped: number;
  failed: number;
  errors: string[];
  cleanedStale?: number;
}

export interface MigrationProgress {
  done: number;
  total: number;
  relPath: string;
}

export async function setLibraryDir(newDir: string): Promise<MigrationPlan> {
  return invoke<MigrationPlan>("set_library_dir", { newDir });
}

export async function runMigration(
  keepOriginals: boolean,
): Promise<MigrationReport> {
  return invoke<MigrationReport>("migrate_library", {
    payload: { keepOriginals },
  });
}

export interface AppPaths {
  appDataDir: string;
  libraryDir: string;
}

export async function getAppPaths(): Promise<AppPaths> {
  return invoke<AppPaths>("get_app_paths");
}

export async function hasVersionRecord(): Promise<boolean> {
  return invoke<boolean>("has_version_record");
}

export async function completeFirstRun(autostart: boolean): Promise<void> {
  await invoke("complete_first_run", { autostart });
}

export interface LastWallpaper {
  id: string;
  title: string;
  mediaType: string;
  uri: string;
  source: string;
}

export async function getLastWallpaper(): Promise<LastWallpaper | null> {
  return invoke<LastWallpaper | null>("get_last_wallpaper");
}
