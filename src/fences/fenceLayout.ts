/** Fence layout persisted in app data (Rust), with one-time localStorage migration. */

export const APP_ORDER_KEY = "lingscape-fence-app-order";
export const IMAGE_ORDER_KEY = "lingscape-fence-image-order";
export const DOC_ORDER_KEY = "lingscape-fence-document-order";
export const FOLDER_ORDER_KEY = "lingscape-fence-folder-order";
export const MEDIA_ORDER_KEY = "lingscape-fence-media-order";
export const ARCHIVE_ORDER_KEY = "lingscape-fence-archive-order";
export const CATEGORY_KEY = "lingscape-fence-category";

export interface FenceLayout {
  appOrder: string[];
  imageOrder: string[];
  documentOrder: string[];
  folderOrder: string[];
  mediaOrder: string[];
  archiveOrder: string[];
  categories: Record<string, string>;
}

const ORDER_FIELD: Record<string, keyof FenceLayout> = {
  [APP_ORDER_KEY]: "appOrder",
  [IMAGE_ORDER_KEY]: "imageOrder",
  [DOC_ORDER_KEY]: "documentOrder",
  [FOLDER_ORDER_KEY]: "folderOrder",
  [MEDIA_ORDER_KEY]: "mediaOrder",
  [ARCHIVE_ORDER_KEY]: "archiveOrder",
};

const LOCAL_KEYS = [
  APP_ORDER_KEY,
  IMAGE_ORDER_KEY,
  DOC_ORDER_KEY,
  FOLDER_ORDER_KEY,
  MEDIA_ORDER_KEY,
  ARCHIVE_ORDER_KEY,
  CATEGORY_KEY,
];

function emptyLayout(): FenceLayout {
  return {
    appOrder: [],
    imageOrder: [],
    documentOrder: [],
    folderOrder: [],
    mediaOrder: [],
    archiveOrder: [],
    categories: {},
  };
}

function layoutHasData(layout: FenceLayout): boolean {
  return (
    layout.appOrder.length > 0 ||
    layout.imageOrder.length > 0 ||
    layout.documentOrder.length > 0 ||
    layout.folderOrder.length > 0 ||
    layout.mediaOrder.length > 0 ||
    layout.archiveOrder.length > 0 ||
    Object.keys(layout.categories).length > 0
  );
}

function readLocalOrder(key: string): string[] {
  try {
    const saved = JSON.parse(localStorage.getItem(key) || "[]") as unknown[];
    return saved.filter((p): p is string => typeof p === "string" && !p.startsWith("::"));
  } catch {
    return [];
  }
}

function readLocalCategories(): Record<string, string> {
  try {
    const raw = JSON.parse(localStorage.getItem(CATEGORY_KEY) || "{}");
    return raw && typeof raw === "object" ? (raw as Record<string, string>) : {};
  } catch {
    return {};
  }
}

function migrateFromLocalStorage(base: FenceLayout): FenceLayout {
  if (layoutHasData(base)) return base;

  const layout = emptyLayout();
  layout.appOrder = readLocalOrder(APP_ORDER_KEY);
  layout.imageOrder = readLocalOrder(IMAGE_ORDER_KEY);
  layout.documentOrder = readLocalOrder(DOC_ORDER_KEY);
  layout.folderOrder = readLocalOrder(FOLDER_ORDER_KEY);
  layout.mediaOrder = readLocalOrder(MEDIA_ORDER_KEY);
  layout.archiveOrder = readLocalOrder(ARCHIVE_ORDER_KEY);
  layout.categories = readLocalCategories();

  if (!layoutHasData(layout)) return base;

  for (const key of LOCAL_KEYS) {
    try {
      localStorage.removeItem(key);
    } catch {
      /* ignore */
    }
  }
  return layout;
}

let cache: FenceLayout | null = null;
let saveTimer: number | null = null;

export function getFenceLayout(): FenceLayout {
  return cache ?? emptyLayout();
}

export async function initFenceLayout(): Promise<void> {
  if (!window.__TAURI__) {
    cache = migrateFromLocalStorage(emptyLayout());
    return;
  }
  try {
    const rust = await window.__TAURI__.core.invoke<FenceLayout>("load_fence_layout");
    const migrated = migrateFromLocalStorage(rust || emptyLayout());
    cache = migrated;
    if (JSON.stringify(migrated) !== JSON.stringify(rust || emptyLayout())) {
      await persistFenceLayoutNow();
    }
  } catch {
    cache = migrateFromLocalStorage(emptyLayout());
  }
}

async function persistFenceLayoutNow() {
  if (!window.__TAURI__ || !cache) return;
  try {
    await window.__TAURI__.core.invoke("save_fence_layout", { layout: cache });
  } catch (err) {
    console.warn("save_fence_layout failed", err);
  }
}

export function scheduleFenceLayoutSave() {
  if (saveTimer !== null) window.clearTimeout(saveTimer);
  saveTimer = window.setTimeout(() => {
    saveTimer = null;
    void persistFenceLayoutNow();
  }, 300);
}

export function saveFenceOrder(orderKey: string, paths: string[]) {
  const field = ORDER_FIELD[orderKey];
  if (!field || !cache) return;
  (cache[field] as string[]) = paths;
  scheduleFenceLayoutSave();
}

export function saveFenceCategory(path: string, category: string) {
  if (!cache) return;
  cache.categories[path] = category;
  scheduleFenceLayoutSave();
}

/** Keep in-memory order after rename so refresh doesn't treat the item as new. */
export function migrateFencePath(oldPath: string, newPath: string) {
  if (!cache || !oldPath || !newPath || samePath(oldPath, newPath)) return;
  const fields: (keyof FenceLayout)[] = [
    "appOrder",
    "imageOrder",
    "documentOrder",
    "folderOrder",
    "mediaOrder",
    "archiveOrder",
  ];
  for (const field of fields) {
    const arr = cache[field] as string[];
    const idx = arr.findIndex((p) => samePath(p, oldPath));
    if (idx >= 0) arr[idx] = newPath;
  }
  const catKey = Object.keys(cache.categories).find((p) => samePath(p, oldPath));
  if (catKey) {
    const cat = cache.categories[catKey];
    delete cache.categories[catKey];
    cache.categories[newPath] = cat;
  }
}

export function removeFencePath(path: string) {
  if (!cache || !path) return;
  const fields: (keyof FenceLayout)[] = [
    "appOrder",
    "imageOrder",
    "documentOrder",
    "folderOrder",
    "mediaOrder",
    "archiveOrder",
  ];
  for (const field of fields) {
    const arr = cache[field] as string[];
    for (let i = arr.length - 1; i >= 0; i--) {
      if (samePath(arr[i], path)) arr.splice(i, 1);
    }
  }
  for (const key of Object.keys(cache.categories)) {
    if (samePath(key, path)) delete cache.categories[key];
  }
}

export function samePath(a: string, b: string): boolean {
  return normalizePath(a) === normalizePath(b);
}

function normalizePath(p: string): string {
  return p.replace(/\//g, "\\").toLowerCase();
}

export function loadFenceOrder(orderKey: string): string[] {
  const field = ORDER_FIELD[orderKey];
  if (!field) return [];
  return [...((getFenceLayout()[field] as string[]) || [])];
}

export function loadFenceCategories(): Record<string, string> {
  return { ...getFenceLayout().categories };
}

/** Re-read layout from Rust (e.g. after backend migrated a path). */
export async function reloadFenceLayout(): Promise<void> {
  if (!window.__TAURI__) return;
  try {
    const rust = await window.__TAURI__.core.invoke<FenceLayout>("load_fence_layout");
    cache = rust || emptyLayout();
  } catch {
    /* keep existing cache */
  }
}
