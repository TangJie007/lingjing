import type { WallpaperItem } from "../data/catalog";

/** key → base64 dataURL */
const posterCache = new Map<string, string>();

function fileNameFromItem(item: Pick<WallpaperItem, "name" | "mediaSrc">): string {
  const src = (item.mediaSrc || "").trim();
  if (src) {
    const base = src.replace(/\\/g, "/").split("/").pop();
    if (base) return base;
  }
  return (item.name || "unknown").trim() || "unknown";
}

/** 下标：有导入时间用时间，否则用文件名 */
export function videoPosterKey(
  item: Pick<WallpaperItem, "importedAt" | "name" | "mediaSrc">,
): string {
  const at = item.importedAt;
  if (at != null && String(at).length > 0) {
    return `t:${at}`;
  }
  return `f:${fileNameFromItem(item)}`;
}

export function getCachedVideoPoster(key: string): string | null {
  if (!key) return null;
  return posterCache.get(key) ?? null;
}

export function cacheVideoPoster(key: string, dataUrl: string) {
  if (!key || !dataUrl) return;
  posterCache.set(key, dataUrl);
}

export function forgetVideoPoster(key: string) {
  if (!key) return;
  posterCache.delete(key);
}

export function captureVideoFrame(video: HTMLVideoElement): string | null {
  try {
    const w = video.videoWidth;
    const h = video.videoHeight;
    if (!w || !h) return null;
    const canvas = document.createElement("canvas");
    const scale = Math.min(1, 640 / w);
    canvas.width = Math.max(1, Math.round(w * scale));
    canvas.height = Math.max(1, Math.round(h * scale));
    const ctx = canvas.getContext("2d");
    if (!ctx) return null;
    ctx.drawImage(video, 0, 0, canvas.width, canvas.height);
    return canvas.toDataURL("image/jpeg", 0.82);
  } catch {
    return null;
  }
}

export function isBenignPlayError(err: unknown) {
  const name = err instanceof Error ? err.name : "";
  const msg = err instanceof Error ? err.message : String(err || "");
  const hay = `${name} ${msg}`.toLowerCase();
  return hay.includes("aborterror") || hay.includes("interrupted by a new load");
}
