import type { WallpaperItem, WallpaperType } from "../data/catalog";
import { useSettings } from "./useSettings";
import { useAuth } from "./useAuth";

interface ApiWallpaper {
  id: number;
  title: string;
  description?: string;
  categoryId?: number;
  categoryName?: string;
  fileUrl: string;
  mimeType?: string;
  fileSize?: number;
  likeCount?: number;
  liked?: boolean;
}

interface WallpaperListData {
  items: ApiWallpaper[];
  total: number;
  page: number;
  pageSize: number;
}

interface ApiEnvelope<T> {
  ok: boolean;
  data?: T;
  error?: string;
}

function apiBase(): string {
  const settings = useSettings();
  return (settings.value.apiBaseUrl || "http://localhost:3002").replace(/\/$/, "");
}

function formatSize(bytes?: number): string {
  if (!bytes || bytes <= 0) return "—";
  if (bytes >= 1_048_576) return `${(bytes / 1_048_576).toFixed(1)}M`;
  if (bytes >= 1024) return `${Math.round(bytes / 1024)}K`;
  return `${bytes}B`;
}

function mediaTypeFromMime(mime?: string, url?: string): WallpaperType {
  const m = (mime || "").toLowerCase();
  if (m.startsWith("video/")) return "video";
  if (m === "image/gif") return "gif";
  if (/\.(mp4|webm)(\?|$)/i.test(url || "")) return "video";
  if (/\.gif(\?|$)/i.test(url || "")) return "gif";
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

export function mapOnlineWallpaper(w: ApiWallpaper): WallpaperItem {
  const type = mediaTypeFromMime(w.mimeType, w.fileUrl);
  const category = w.categoryName || "在线";
  return {
    id: `online-${w.id}`,
    name: w.title,
    thumb: thumbFor(type),
    type,
    size: formatSize(w.fileSize),
    category,
    author: w.description?.trim() || "灵境社区",
    heat: w.likeCount ? `🔥 ${w.likeCount}` : "在线",
    favorite: !!w.liked,
    tags: [`#${category}`],
    mediaSrc: w.fileUrl,
    source: "online",
  };
}

export async function fetchOnlineWallpapers(options?: {
  page?: number;
  pageSize?: number;
  categoryId?: string;
}): Promise<{ items: WallpaperItem[]; total: number }> {
  const page = options?.page ?? 1;
  const pageSize = options?.pageSize ?? 24;
  const categoryId = options?.categoryId ?? "";
  const params = new URLSearchParams({
    page: String(page),
    pageSize: String(pageSize),
    categoryId,
  });
  const { authHeaders } = useAuth();
  const res = await fetch(`${apiBase()}/api/lingjing/wallpapers?${params}`, {
    headers: { ...authHeaders() },
  });
  const body = (await res.json()) as ApiEnvelope<WallpaperListData>;
  if (!body.ok || !body.data) {
    throw new Error(body.error || "加载在线壁纸失败");
  }
  return {
    items: (body.data.items ?? []).map(mapOnlineWallpaper),
    total: body.data.total ?? 0,
  };
}
