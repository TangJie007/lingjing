import type { WallpaperItem, WallpaperType } from "../data/catalog";
import { apiFetch } from "./apiFetch";
import { cacheOnlineWallpaper, saveOnlineWallpaper } from "./useEngine";
import { DEFAULT_API_BASE_URL, normalizeApiBaseUrl, useSettings } from "./useSettings";
import { useAuth } from "./useAuth";

/** Raw wallpaper fields (snake_case from API; camelCase tolerated). */
interface ApiWallpaper {
  id: number;
  title: string;
  description?: string | null;
  category_id?: number;
  categoryId?: number;
  category_name?: string | null;
  categoryName?: string | null;
  file_url?: string | null;
  fileUrl?: string | null;
  thumbnail_url?: string | null;
  thumbnailUrl?: string | null;
  stream_path?: string | null;
  streamPath?: string | null;
  download_path?: string | null;
  downloadPath?: string | null;
  mime_type?: string | null;
  mimeType?: string | null;
  file_size?: number | null;
  fileSize?: number | null;
  like_count?: number;
  likeCount?: number;
  liked?: boolean;
  favorited?: boolean;
  tags?: string[];
}

export interface OnlineCategory {
  id: number;
  name: string;
  slug: string;
  sortOrder: number;
  enabled: boolean;
}

interface CategoryListData {
  items: Array<{
    id: number;
    name: string;
    slug: string;
    sort_order?: number;
    sortOrder?: number;
    is_active?: boolean;
    enabled?: boolean;
  }>;
  total: number;
}

interface WallpaperListData {
  items: ApiWallpaper[];
  total: number;
  page: number;
  page_size?: number;
  pageSize?: number;
}

interface ApiEnvelope<T> {
  ok: boolean;
  data?: T;
  error?: string;
  code?: number;
}

function apiBase(): string {
  const settings = useSettings();
  return normalizeApiBaseUrl(settings.value.apiBaseUrl || DEFAULT_API_BASE_URL);
}

function formatSize(bytes?: number | null): string {
  if (!bytes || bytes <= 0) return "—";
  if (bytes >= 1_048_576) return `${(bytes / 1_048_576).toFixed(1)}M`;
  if (bytes >= 1024) return `${Math.round(bytes / 1024)}K`;
  return `${bytes}B`;
}

function mediaTypeFromMime(mime?: string | null, url?: string): WallpaperType {
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

function pickStr(...vals: Array<string | null | undefined>): string | undefined {
  for (const v of vals) {
    const t = v?.trim();
    if (t) return t;
  }
  return undefined;
}

function pickNum(...vals: Array<number | null | undefined>): number | undefined {
  for (const v of vals) {
    if (typeof v === "number" && Number.isFinite(v)) return v;
  }
  return undefined;
}

function resolveMediaSrc(w: ApiWallpaper): string | undefined {
  const direct = pickStr(w.file_url, w.fileUrl);
  if (direct) return direct;
  const stream = pickStr(w.stream_path, w.streamPath);
  if (!stream) return undefined;
  if (/^https?:\/\//i.test(stream)) return stream;
  const base = apiBase().replace(/\/$/, "");
  return stream.startsWith("/") ? `${base}${stream}` : `${base}/${stream}`;
}

function resolveDownloadPath(w: ApiWallpaper): string | undefined {
  const path = pickStr(w.download_path, w.downloadPath);
  if (path) return path;
  return undefined;
}

function absoluteApiPath(path: string): string {
  if (/^https?:\/\//i.test(path)) return path;
  const base = apiBase().replace(/\/$/, "");
  return path.startsWith("/") ? `${base}${path}` : `${base}/${path}`;
}

function guessFileExt(item: WallpaperItem): string | undefined {
  if (item.type === "video") return "mp4";
  if (item.type === "gif") return "gif";
  if (item.type === "image") return "jpg";
  return undefined;
}

export function mapOnlineWallpaper(w: ApiWallpaper): WallpaperItem {
  const mediaSrc = resolveMediaSrc(w);
  const thumbUrl = pickStr(w.thumbnail_url, w.thumbnailUrl);
  const mime = pickStr(w.mime_type, w.mimeType);
  const type = mediaTypeFromMime(mime, mediaSrc || thumbUrl);
  const category = pickStr(w.category_name, w.categoryName) || "在线";
  const likeCount = pickNum(w.like_count, w.likeCount) ?? 0;
  const tags = Array.isArray(w.tags) && w.tags.length
    ? w.tags.map((t) => (t.startsWith("#") ? t : `#${t}`))
    : [`#${category}`];
  const idNum = w.id;
  const downloadPath =
    resolveDownloadPath(w) ||
    (Number.isInteger(idNum) && idNum > 0
      ? `/api/public/wallpapers/${idNum}/download`
      : undefined);
  return {
    id: `online-${w.id}`,
    name: w.title,
    thumb: thumbUrl || thumbFor(type),
    type,
    size: formatSize(pickNum(w.file_size, w.fileSize)),
    category,
    author: pickStr(w.description) || "灵境社区",
    heat: likeCount ? `🔥 ${likeCount}` : "在线",
    favorite: !!w.favorited,
    tags,
    mediaSrc,
    downloadPath,
    source: "online",
  };
}

function onlineWallpaperId(id: string | number): number | null {
  if (typeof id === "number" && Number.isInteger(id) && id > 0) return id;
  const m = String(id).match(/^online-(\d+)$/);
  if (m) return Number(m[1]);
  const n = Number(id);
  return Number.isInteger(n) && n > 0 ? n : null;
}

export async function fetchOnlineCategories(): Promise<OnlineCategory[]> {
  const { authHeaders } = useAuth();
  const res = await apiFetch(`${apiBase()}/api/public/categories`, {
    headers: { ...authHeaders() },
  });
  const body = (await res.json()) as ApiEnvelope<CategoryListData>;
  if (!body.ok || !body.data) {
    throw new Error(body.error || "加载壁纸分类失败");
  }
  return (body.data.items ?? [])
    .map((c) => ({
      id: c.id,
      name: c.name,
      slug: c.slug,
      sortOrder: pickNum(c.sort_order, c.sortOrder) ?? 0,
      enabled: c.is_active ?? c.enabled ?? true,
    }))
    .filter((c) => c.enabled)
    .sort((a, b) => a.sortOrder - b.sortOrder || a.id - b.id);
}

export async function fetchOnlineWallpapers(options?: {
  page?: number;
  pageSize?: number;
  categoryId?: string | number | null;
  sort?: "latest" | "popular" | "featured";
  q?: string;
}): Promise<{ items: WallpaperItem[]; total: number }> {
  const page = options?.page ?? 1;
  const pageSize = options?.pageSize ?? 24;
  const params = new URLSearchParams({
    page: String(page),
    page_size: String(pageSize),
  });
  if (options?.sort) params.set("sort", options.sort);
  if (options?.q?.trim()) params.set("q", options.q.trim());
  const rawCat = options?.categoryId;
  if (rawCat != null && String(rawCat).trim() !== "") {
    const id = Number(rawCat);
    if (Number.isInteger(id) && id > 0) {
      params.set("category_id", String(id));
    }
  }
  const { authHeaders } = useAuth();
  const res = await apiFetch(`${apiBase()}/api/public/wallpapers?${params}`, {
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

/** Member: list favorited wallpapers (JWT required). */
export async function fetchOnlineFavorites(options?: {
  page?: number;
  pageSize?: number;
}): Promise<{ items: WallpaperItem[]; total: number }> {
  const page = options?.page ?? 1;
  const pageSize = options?.pageSize ?? 48;
  const params = new URLSearchParams({
    page: String(page),
    page_size: String(pageSize),
  });
  const { authHeaders } = useAuth();
  const res = await apiFetch(`${apiBase()}/api/wallpapers/favorites?${params}`, {
    headers: { ...authHeaders() },
  });
  const body = (await res.json()) as ApiEnvelope<WallpaperListData>;
  if (!body.ok || !body.data) {
    throw new Error(body.error || "加载社区收藏失败");
  }
  return {
    items: (body.data.items ?? []).map((w) => {
      const item = mapOnlineWallpaper(w);
      item.favorite = true;
      return item;
    }),
    total: body.data.total ?? 0,
  };
}

/** Member: list download history (JWT required). */
export async function fetchOnlineDownloads(options?: {
  page?: number;
  pageSize?: number;
}): Promise<{ items: WallpaperItem[]; total: number }> {
  const page = options?.page ?? 1;
  const pageSize = options?.pageSize ?? 48;
  const params = new URLSearchParams({
    page: String(page),
    page_size: String(pageSize),
  });
  const { authHeaders } = useAuth();
  const res = await apiFetch(`${apiBase()}/api/wallpapers/downloads?${params}`, {
    headers: { ...authHeaders() },
  });
  const body = (await res.json()) as ApiEnvelope<WallpaperListData>;
  if (!body.ok || !body.data) {
    throw new Error(body.error || "加载下载记录失败");
  }
  return {
    items: (body.data.items ?? []).map(mapOnlineWallpaper),
    total: body.data.total ?? 0,
  };
}

/** Member: favorite / unfavorite a wallpaper (JWT required). */
export async function setOnlineFavorite(
  wallpaperId: string | number,
  favorite: boolean,
): Promise<void> {
  const id = onlineWallpaperId(wallpaperId);
  if (id == null) throw new Error("无效的在线壁纸 ID");
  const { authHeaders } = useAuth();
  const res = await apiFetch(`${apiBase()}/api/wallpapers/${id}/favorite`, {
    method: favorite ? "POST" : "DELETE",
    headers: { ...authHeaders() },
  });
  const body = (await res.json()) as ApiEnvelope<unknown>;
  if (!body.ok) {
    throw new Error(body.error || (favorite ? "收藏失败" : "取消收藏失败"));
  }
}

export interface OnlineSignedFileUrl {
  wallpaperId: number;
  url: string;
  expiresIn: number;
  expiresAt?: string;
}

/**
 * GET /api/public/wallpapers/{id}/file-url — R2 presigned download URL.
 * Used before caching online wallpaper to `.onlinefile`.
 */
export async function fetchOnlineFileUrl(
  wallpaperId: string | number,
  expires = 3600,
): Promise<OnlineSignedFileUrl> {
  const id = onlineWallpaperId(wallpaperId);
  if (id == null) throw new Error("无效的在线壁纸 ID");
  const params = new URLSearchParams({ expires: String(expires) });
  const { authHeaders } = useAuth();
  const res = await apiFetch(
    `${apiBase()}/api/public/wallpapers/${id}/file-url?${params}`,
    { headers: { ...authHeaders() } },
  );
  const body = (await res.json()) as ApiEnvelope<{
    wallpaper_id?: number;
    wallpaperId?: number;
    url?: string;
    expires_in?: number;
    expiresIn?: number;
    expires_at?: string;
    expiresAt?: string;
  }>;
  if (!body.ok || !body.data?.url) {
    throw new Error(body.error || "获取下载地址失败");
  }
  const data = body.data;
  const url = data.url;
  if (!url) throw new Error("获取下载地址失败");
  return {
    wallpaperId: pickNum(data.wallpaper_id, data.wallpaperId) ?? id,
    url,
    expiresIn: pickNum(data.expires_in, data.expiresIn) ?? expires,
    expiresAt: pickStr(data.expires_at, data.expiresAt),
  };
}

/**
 * Resolve a fresh download URL via file-url API, then cache into `.onlinefile`.
 * Used when applying wallpaper (does not write member download history).
 * Returns absolute local path.
 */
export async function downloadOnlineWallpaperToCache(
  item: WallpaperItem,
): Promise<string> {
  if (item.source !== "online") {
    throw new Error("仅支持在线壁纸下载");
  }
  const signed = await fetchOnlineFileUrl(item.id, 3600);
  const raw = signed.url.split("?")[0] ?? signed.url;
  const ext = raw.includes(".") ? (raw.split(".").pop() ?? "").toLowerCase() : "";
  // Presigned R2 URL must not carry Authorization — it invalidates the signature.
  return cacheOnlineWallpaper({
    id: String(item.id),
    url: signed.url,
    fileExt: ext && /^[a-z0-9]{1,8}$/i.test(ext) ? ext : guessFileExt(item),
  });
}

/**
 * User-facing download via `/download` (JWT + history + 302 to object URL).
 * Streams into `.onlinefile` via Rust reqwest. Does not use `file-url`.
 */
export async function saveOnlineWallpaperToDisk(
  item: WallpaperItem,
): Promise<string> {
  if (item.source !== "online") {
    throw new Error("仅支持在线壁纸下载");
  }
  const id = onlineWallpaperId(item.id);
  if (id == null) throw new Error("无效的在线壁纸 ID");
  const path =
    pickStr(item.downloadPath) || `/api/public/wallpapers/${id}/download`;
  const params = new URLSearchParams({ expires: "3600" });
  const sep = path.includes("?") ? "&" : "?";
  const url = `${absoluteApiPath(path)}${sep}${params}`;
  const { authHeaders } = useAuth();
  const auth = authHeaders().Authorization;
  const ext = guessFileExt(item) || "bin";
  const safeName = item.name.replace(/[\\/:*?"<>|]/g, "_") || `wallpaper-${id}`;
  return saveOnlineWallpaper({
    id: String(item.id),
    url,
    fileName: `${safeName}.${ext}`,
    title: item.name,
    fileExt: ext,
    authorization: typeof auth === "string" ? auth : undefined,
    recordDownload: true,
  });
}

/**
 * @deprecated Prefer saveOnlineWallpaperToDisk for user downloads.
 * Kept for callers that still cache then export.
 */
export async function exportOnlineWallpaperToCache(
  item: WallpaperItem,
): Promise<string> {
  if (item.source !== "online") {
    throw new Error("仅支持在线壁纸下载");
  }
  const id = onlineWallpaperId(item.id);
  if (id == null) throw new Error("无效的在线壁纸 ID");
  const path =
    pickStr(item.downloadPath) || `/api/public/wallpapers/${id}/download`;
  const params = new URLSearchParams({ expires: "3600" });
  const sep = path.includes("?") ? "&" : "?";
  const url = `${absoluteApiPath(path)}${sep}${params}`;
  const { authHeaders } = useAuth();
  const auth = authHeaders().Authorization;
  return cacheOnlineWallpaper({
    id: String(item.id),
    url,
    authorization: typeof auth === "string" ? auth : undefined,
    fileExt: guessFileExt(item),
    recordDownload: true,
  });
}
