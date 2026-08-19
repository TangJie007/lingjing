import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { WallpaperItem } from "../data/catalog";

export interface EngineState {
  mediaId: string | null;
  title: string | null;
  mediaType: string | null;
  uri: string | null;
  playing: boolean;
  volume: number;
  muted: boolean;
  currentTime: number;
  duration: number;
  error: string | null;
}

export function resolveMediaUri(item: WallpaperItem): string | null {
  const src = item.mediaSrc?.trim();
  if (!src) return null;
  if (/^(https?:|asset:|file:)/i.test(src)) return src;
  if (/^[a-zA-Z]:[\\/]/.test(src) || src.startsWith("\\\\")) {
    return convertFileSrc(src);
  }
  const path = src.startsWith("/") ? src : `/${src}`;
  try {
    return new URL(path, window.location.origin).href;
  } catch {
    return path;
  }
}

export async function setWallpaper(item: WallpaperItem): Promise<EngineState> {
  const uri = resolveMediaUri(item);
  if (!uri) throw new Error("该资源暂无可用媒体");
  return invoke<EngineState>("set_wallpaper", {
    payload: {
      id: String(item.id),
      title: item.name,
      mediaType: item.type,
      uri,
    },
  });
}

export async function enginePlay() {
  return invoke<EngineState>("engine_play");
}

export async function enginePause() {
  return invoke<EngineState>("engine_pause");
}

export async function engineSetVolume(volume: number, muted: boolean) {
  return invoke<EngineState>("engine_set_volume", {
    payload: { volume, muted },
  });
}

export async function engineGetState() {
  return invoke<EngineState>("engine_get_state");
}

export function onEngineState(cb: (s: EngineState) => void): Promise<UnlistenFn> {
  return listen<EngineState>("engine-state", (ev) => cb(ev.payload));
}

export interface PauseRecommendPayload {
  action: "pause" | "play";
  reason: string;
}

export function onPauseRecommend(
  cb: (p: PauseRecommendPayload) => void,
): Promise<UnlistenFn> {
  return listen<PauseRecommendPayload>("engine-pause-recommend", (ev) => cb(ev.payload));
}

export interface LibraryDto {
  id: string;
  name: string;
  thumb: string;
  type: string;
  size: string;
  category: string;
  author: string;
  heat: string;
  favorite: boolean;
  tags: string[];
  mediaSrc: string;
  path: string;
}

export function libraryToItem(dto: LibraryDto): WallpaperItem {
  return {
    id: dto.id,
    name: dto.name,
    thumb: dto.thumb,
    type: (dto.type as WallpaperItem["type"]) || "image",
    size: dto.size,
    category: dto.category,
    author: dto.author,
    heat: dto.heat,
    favorite: dto.favorite,
    tags: dto.tags ?? ["#本地"],
    mediaSrc: dto.mediaSrc || dto.path,
    source: "local",
  };
}

export async function listLibrary(): Promise<WallpaperItem[]> {
  const rows = await invoke<LibraryDto[]>("list_library");
  return rows.map(libraryToItem);
}

export async function importMedia(
  paths: string[],
): Promise<{ items: WallpaperItem[]; errors: string[] }> {
  const result = await invoke<{ items: LibraryDto[]; errors: string[] }>("import_media", {
    paths,
  });
  return {
    items: (result.items ?? []).map(libraryToItem),
    errors: result.errors ?? [],
  };
}

export async function removeLibraryItem(id: string): Promise<void> {
  await invoke("remove_library_item", { id });
}

export async function loadFavoriteIds(): Promise<{ ids: string[]; isNew: boolean }> {
  const file = await invoke<{ ids: string[]; isNew: boolean }>("load_favorites");
  return { ids: file.ids ?? [], isNew: !!file.isNew };
}

export async function setFavoriteRemote(id: string, favorite: boolean): Promise<string[]> {
  const file = await invoke<{ ids: string[] }>("set_favorite", {
    payload: { id: String(id), favorite },
  });
  return file.ids ?? [];
}
