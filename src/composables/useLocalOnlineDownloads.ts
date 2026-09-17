import { computed, ref } from "vue";
import type { WallpaperItem, WallpaperType } from "../data/catalog";
import {
  listOnlineFileDownloads,
  type OnlineFileDownloadItem,
} from "./useEngine";

/** IDs present in `.onlinefile/list.json` with a valid local file. */
const localDownloadedIds = ref<Set<string>>(new Set());
/** Wallpaper cards for the Online → 已下载 tab. */
const localDownloadedItems = ref<WallpaperItem[]>([]);
let loadPromise: Promise<void> | null = null;

function formatSize(bytes?: number | null): string {
  if (!bytes || bytes <= 0) return "—";
  if (bytes >= 1_048_576) return `${(bytes / 1_048_576).toFixed(1)}M`;
  if (bytes >= 1024) return `${Math.round(bytes / 1024)}K`;
  return `${bytes}B`;
}

function typeFromFile(item: OnlineFileDownloadItem): WallpaperType {
  const mime = (item.mimeType || "").toLowerCase();
  const name = `${item.fileName} ${item.localPath || ""}`.toLowerCase();
  if (mime.startsWith("video/") || /\.(mp4|webm)(\?|$)/i.test(name)) return "video";
  if (mime === "image/gif" || /\.gif(\?|$)/i.test(name)) return "gif";
  return "image";
}

function thumbFor(type: WallpaperType): string {
  switch (type) {
    case "video":
      return "linear-gradient(135deg,#c7d2fe,#4f46e5)";
    case "gif":
      return "linear-gradient(135deg,#bfdbfe,#2563eb)";
    default:
      return "linear-gradient(135deg,#a5f3fc,#0891b2)";
  }
}

export function onlineFileToWallpaperItem(item: OnlineFileDownloadItem): WallpaperItem {
  const type = typeFromFile(item);
  const mediaSrc = (item.localPath || "").trim() || undefined;
  return {
    id: String(item.id),
    name: (item.title || "").trim() || item.fileName || String(item.id),
    thumb: mediaSrc || thumbFor(type),
    type,
    size: formatSize(item.fileSize),
    category: "已下载",
    author: "本地缓存",
    heat: "已下载",
    favorite: false,
    tags: ["#已下载"],
    mediaSrc,
    source: "online",
  };
}

export async function refreshLocalOnlineDownloads() {
  if (loadPromise) return loadPromise;
  loadPromise = (async () => {
    try {
      const items = await listOnlineFileDownloads();
      localDownloadedIds.value = new Set(items.map((i) => String(i.id)));
      localDownloadedItems.value = items.map(onlineFileToWallpaperItem);
    } catch (e) {
      console.warn("[onlinefile] list failed", e);
    } finally {
      loadPromise = null;
    }
  })();
  return loadPromise;
}

export function markLocalOnlineDownloaded(id: string) {
  const next = new Set(localDownloadedIds.value);
  next.add(String(id));
  localDownloadedIds.value = next;
  // Full list refresh keeps cards in sync (path/title/size).
  void refreshLocalOnlineDownloads();
}

export function isLocalOnlineDownloaded(id?: string | null) {
  if (!id) return false;
  return localDownloadedIds.value.has(String(id));
}

export function useLocalOnlineDownloads() {
  const ids = computed(() => localDownloadedIds.value);
  const items = computed(() => localDownloadedItems.value);
  return {
    ids,
    items,
    isDownloaded: isLocalOnlineDownloaded,
    refresh: refreshLocalOnlineDownloads,
    markDownloaded: markLocalOnlineDownloaded,
  };
}
