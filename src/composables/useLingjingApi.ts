import type { WallpaperItem, WallpaperType } from "../data/catalog";
import { apiFetch } from "./apiFetch";
import { DEFAULT_API_BASE_URL, normalizeApiBaseUrl, useSettings } from "./useSettings";
import { useAuth } from "./useAuth";

interface ApiWallpaper {
  id: number;
  title: string;
  description?: string;
  categoryId?: number;
  categoryName?: string;
  fileUrl?: string;
  thumbnailUrl?: string;
  streamPath?: string;
  mimeType?: string;
  fileSize?: number;
  likeCount?: number;
  liked?: boolean;
}

export interface OnlineCategory {
  id: number;
  name: string;
  slug: string;
  sortOrder: number;
  enabled: boolean;
}

interface CategoryListData {
  items: OnlineCategory[];
  total: number;
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
  return normalizeApiBaseUrl(settings.value.apiBaseUrl || DEFAULT_API_BASE_URL);
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

function resolveMediaSrc(w: ApiWallpaper): string | undefined {
  const direct = w.fileUrl?.trim();
  if (direct) return direct;
  const stream = w.streamPath?.trim();
  if (!stream) return undefined;
  if (/^https?:\/\//i.test(stream)) return stream;
  const base = apiBase().replace(/\/$/, "");
  return stream.startsWith("/") ? `${base}${stream}` : `${base}/${stream}`;
}

export function mapOnlineWallpaper(w: ApiWallpaper): WallpaperItem {
  const mediaSrc = resolveMediaSrc(w);
  const type = mediaTypeFromMime(w.mimeType, mediaSrc || w.thumbnailUrl);
  const category = w.categoryName || "在线";
  const thumb = w.thumbnailUrl?.trim() || thumbFor(type);
  return {
    id: `online-${w.id}`,
    name: w.title,
    thumb,
    type,
    size: formatSize(w.fileSize),
    category,
    author: w.description?.trim() || "灵境社区",
    heat: w.likeCount ? `🔥 ${w.likeCount}` : "在线",
    favorite: !!w.liked,
    tags: [`#${category}`],
    mediaSrc,
    source: "online",
  };
}

export async function fetchOnlineCategories(): Promise<OnlineCategory[]> {
  const { authHeaders } = useAuth();
  const res = await apiFetch(`${apiBase()}/api/lingjing/wallpaper-categories`, {
    headers: { ...authHeaders() },
  });
  const body = (await res.json()) as ApiEnvelope<CategoryListData>;
  if (!body.ok || !body.data) {
    throw new Error(body.error || "加载壁纸分类失败");
  }
  return (body.data.items ?? [])
    .filter((c) => c.enabled)
    .sort((a, b) => a.sortOrder - b.sortOrder || a.id - b.id);
}

export async function fetchOnlineWallpapers(options?: {
  page?: number;
  pageSize?: number;
  categoryId?: string | number | null;
}): Promise<{ items: WallpaperItem[]; total: number }> {
  const page = options?.page ?? 1;
  const pageSize = options?.pageSize ?? 24;
  const params = new URLSearchParams({
    page: String(page),
    pageSize: String(pageSize),
  });
  const rawCat = options?.categoryId;
  if (rawCat != null && String(rawCat).trim() !== "") {
    const id = Number(rawCat);
    if (Number.isInteger(id) && id > 0) {
      params.set("categoryId", String(id));
    }
  }
  const { authHeaders } = useAuth();
  const res = await apiFetch(`${apiBase()}/api/lingjing/wallpapers?${params}`, {
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
