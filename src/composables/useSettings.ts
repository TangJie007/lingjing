import { invoke } from "@tauri-apps/api/core";
import { ref, watch } from "vue";

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
  autostart: true,
  hideIconsOnDoubleClick: false,
  pauseOnFullscreen: true,
  pauseOnBattery: true,
  pauseOnRdp: true,
  soundOn: true,
  defaultVolume: 0.8,
  importCopyToData: true,
  libraryDirOverride: null,
  loopMode: "list",
  onlineEnabled: false,
  desktopOrganizeEnabled: false,
  apiBaseUrl: "http://localhost:3002",
};

const settings = ref<AppSettings>({ ...DEFAULT_SETTINGS });
let loaded = false;
let writeTimer: number | null = null;

export function useSettings() {
  return settings;
}

export async function loadSettings(): Promise<AppSettings> {
  try {
    const remote = await invoke<AppSettings>("load_settings");
    settings.value = { ...DEFAULT_SETTINGS, ...remote };
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
