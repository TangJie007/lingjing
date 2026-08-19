export type WallpaperType = "video" | "gif" | "image";

export interface WallpaperItem {
  id: number;
  name: string;
  thumb: string;
  type: WallpaperType;
  size: string;
  category: string;
  author: string;
  heat: string;
  favorite: boolean;
  tags: string[];
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

/** 与 lingjing-ui-design.html 主界面网格完全一致 */
export const CATALOG: WallpaperItem[] = [
  {
    id: 1,
    name: "极光幻境 Aurora",
    thumb: "linear-gradient(135deg,#c7d2fe,#4f46e5)",
    type: "video",
    size: "1.2M",
    category: "科技",
    author: "SkyLab",
    heat: "🔥 3.4k",
    favorite: true,
    tags: ["#科技", "#极光", "#4K"],
  },
  {
    id: 2,
    name: "深海蓝调",
    thumb: "linear-gradient(135deg,#a5f3fc,#0891b2)",
    type: "image",
    size: "860K",
    category: "风景",
    author: "BlueStudio",
    heat: "🔥 2.1k",
    favorite: true,
    tags: ["#风景", "#海洋", "#4K"],
  },
  {
    id: 3,
    name: "霓虹少女",
    thumb: "linear-gradient(135deg,#fbcfe8,#db2777)",
    type: "video",
    size: "2.1M",
    category: "动漫",
    author: "Momo",
    heat: "🔥 5.0k",
    favorite: true,
    tags: ["#动漫", "#霓虹", "#4K"],
  },
  {
    id: 4,
    name: "黄昏列车",
    thumb: "linear-gradient(135deg,#fde68a,#d97706)",
    type: "image",
    size: "740K",
    category: "风景",
    author: "铁道猫",
    heat: "🔥 1.6k",
    favorite: false,
    tags: ["#风景", "#列车", "#2K"],
  },
  {
    id: 5,
    name: "赛博城市",
    thumb: "linear-gradient(135deg,#bfdbfe,#2563eb)",
    type: "gif",
    size: "1.5M",
    category: "科技",
    author: "NeoLab",
    heat: "🔥 4.2k",
    favorite: false,
    tags: ["#科技", "#赛博", "#4K"],
  },
  {
    id: 6,
    name: "森林晨雾",
    thumb: "linear-gradient(135deg,#d9f99d,#65a30d)",
    type: "video",
    size: "980K",
    category: "风景",
    author: "青野",
    heat: "🔥 2.8k",
    favorite: false,
    tags: ["#风景", "#森林", "#4K"],
  },
];
