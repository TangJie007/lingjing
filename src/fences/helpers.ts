import type { DesktopItem, FenceGroupKey } from "./types";
import {
  APP_ORDER_KEY,
  ARCHIVE_ORDER_KEY,
  CATEGORY_KEY,
  DOC_ORDER_KEY,
  FOLDER_ORDER_KEY,
  IMAGE_ORDER_KEY,
  MEDIA_ORDER_KEY,
  loadFenceCategories,
  loadFenceOrder,
  saveFenceCategory,
  saveFenceOrder,
} from "./fenceLayout";

export {
  APP_ORDER_KEY,
  IMAGE_ORDER_KEY,
  DOC_ORDER_KEY,
  FOLDER_ORDER_KEY,
  MEDIA_ORDER_KEY,
  ARCHIVE_ORDER_KEY,
  CATEGORY_KEY,
};

const BUILTIN_ORDER = [
  "20D04FE0-3AEA-1069-A2D8-08002B30309D",
  "645FF040-5081-101B-9F08-00AA002F954E",
  "F02C1A0D-B21F-4110-8426-0A0C959C3602",
];

export function saveOrder(key: string, paths: string[]) {
  saveFenceOrder(key, paths);
}

export function loadOrder(key: string, items: DesktopItem[]): DesktopItem[] {
  try {
    const saved = loadFenceOrder(key);
    const map = new Map(items.map((a) => [a.path, a]));
    const out: DesktopItem[] = [];
    for (const p of saved) {
      if (typeof p !== "string" || p.startsWith("::")) continue;
      if (map.has(p)) {
        out.push(map.get(p)!);
        map.delete(p);
      }
    }
    for (const a of items) {
      if (map.has(a.path)) out.push(a);
    }
    return out;
  } catch {
    return items;
  }
}

export function loadCategoryOverrides(): Record<string, string> {
  return loadFenceCategories();
}

export function saveCategoryOverride(path: string, category: string) {
  saveFenceCategory(path, category);
}

export function effectiveCategory(item: DesktopItem | null | undefined): string {
  if (!item) return "other";
  const overrides = loadCategoryOverrides();
  if (overrides[item.path]) return overrides[item.path];
  return item.kind || "other";
}

export function builtinRank(path: string): number {
  const upper = String(path || "").toUpperCase();
  const idx = BUILTIN_ORDER.findIndex((id) => upper.includes(id));
  return idx >= 0 ? idx : 99;
}

export function partitionByCategory(list: DesktopItem[]) {
  const buckets: Record<Exclude<FenceGroupKey, "app">, DesktopItem[]> = {
    image: [],
    document: [],
    folder: [],
    media: [],
    archive: [],
  };
  for (const item of list) {
    if (item.builtin) continue;
    const cat = effectiveCategory(item);
    if (cat === "app") continue;
    if (cat in buckets) buckets[cat as Exclude<FenceGroupKey, "app">].push(item);
    else buckets.document.push(item);
  }
  return buckets;
}

export function partitionApps(list: DesktopItem[]): DesktopItem[] {
  const builtins: DesktopItem[] = [];
  const regular: DesktopItem[] = [];
  for (const item of list) {
    const cat = effectiveCategory(item);
    if (cat !== "app") continue;
    if (item.builtin) builtins.push(item);
    else regular.push(item);
  }
  builtins.sort((a, b) => builtinRank(a.path) - builtinRank(b.path));
  return [...builtins, ...loadOrder(APP_ORDER_KEY, regular)];
}

export function iconFor(item: DesktopItem): string {
  if (item?.isDir && !item?.builtin) return "📁";
  const ext = (item.name.split(".").pop() || "").toLowerCase();
  const map: Record<string, string> = {
    lnk: "🔗",
    exe: "⚙️",
    txt: "📄",
    pdf: "📕",
    doc: "📘",
    docx: "📘",
    xls: "📗",
    xlsx: "📗",
    ppt: "📙",
    pptx: "📙",
    zip: "🗜️",
    rar: "🗜️",
    png: "🖼️",
    jpg: "🖼️",
    jpeg: "🖼️",
    gif: "🖼️",
    webp: "🖼️",
    mp4: "🎬",
    mp3: "🎵",
  };
  return map[ext] || "📄";
}

export function emptyTextFor(key: FenceGroupKey): string {
  const map: Partial<Record<FenceGroupKey, string>> = {
    image: "暂无图片",
    document: "暂无文档",
    folder: "暂无文件夹",
    media: "暂无媒体",
    archive: "暂无压缩包",
  };
  return map[key] || "";
}

export function persistGroupOrder(orderKey: string, items: DesktopItem[]) {
  saveOrder(
    orderKey,
    items.filter((a) => !a.builtin).map((a) => a.path),
  );
}
