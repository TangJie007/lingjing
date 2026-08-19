import type { WallpaperItem } from "../data/catalog";

/** 统一的卡片副标题格式（本地 / 在线 / 收藏） */
export function wallpaperMeta(item: WallpaperItem): string {
  if (item.missing) return "源文件缺失";
  if (item.source === "online") {
    const heat = item.heat && item.heat !== "在线" ? ` · ${item.heat}` : "";
    return `${item.category} · ${item.size}${heat}`;
  }
  const res = item.type === "image" && item.size === "740K" ? "2K" : item.size;
  return `${item.category} · ${res}`;
}
