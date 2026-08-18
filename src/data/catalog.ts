export type WallpaperType = "video" | "gif" | "image";

export interface WallpaperItem {
  id: number;
  name: string;
  /** 缩略图渐变（占位，无真实资源） */
  thumb: string;
  type: WallpaperType;
  size: string;
  category: string;
  favorite: boolean;
}

export const CATEGORIES = [
  "全部",
  "推荐",
  "游戏",
  "动漫",
  "风景",
  "动物",
  "科技",
  "趣味",
  "人机交互",
] as const;

export const SORTS = ["最热", "最新"] as const;

const GRADIENTS = [
  "linear-gradient(135deg,#7c5cff,#4ad6ff)",
  "linear-gradient(135deg,#4f46e5,#22d3ee)",
  "linear-gradient(135deg,#ff7cae,#ffb86c)",
  "linear-gradient(135deg,#43e97b,#38f9d7)",
  "linear-gradient(135deg,#fa709a,#fee140)",
  "linear-gradient(135deg,#30cfd0,#330867)",
  "linear-gradient(135deg,#a8edea,#fed6e3)",
  "linear-gradient(135deg,#5ee7df,#b490ca)",
];

export const CATALOG: WallpaperItem[] = Array.from({ length: 24 }, (_, i) => {
  const cats = ["游戏", "动漫", "风景", "动物", "科技", "趣味", "人机交互"];
  const type: WallpaperType =
    i % 3 === 0 ? "video" : i % 3 === 1 ? "gif" : "image";
  return {
    id: i + 1,
    name: `示例壁纸 ${String(i + 1).padStart(2, "0")}`,
    thumb: GRADIENTS[i % GRADIENTS.length],
    type,
    size: `${(Math.random() * 80 + 6).toFixed(1)} MB`,
    category: cats[i % cats.length],
    favorite: i % 5 === 0,
  };
});
