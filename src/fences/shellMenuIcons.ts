import cutIcon from "../assets/svg/jianqie.svg";
import copyIcon from "../assets/svg/copy.svg";
import shareIcon from "../assets/svg/share.svg";
import deleteIcon from "../assets/svg/delete.svg";
import renameIcon from "../assets/svg/rename.svg";
import refreshIcon from "../assets/svg/refresh.svg";
import type { ShellMenuEntry } from "./types";

/** Match Rust `shell_menu::ids` builtins. */
export const BUILTIN_CUT = 0xf0000000 + 7;
export const BUILTIN_COPY = 0xf0000000 + 8;
export const BUILTIN_DELETE = 0xf0000000 + 10;
export const BUILTIN_RENAME = 0xf0000000 + 11;
export const BUILTIN_REFRESH = 0xf0000000 + 13;
export const BUILTIN_SHARE = 0xf0000000 + 21;

export type ShellPinKind = "cut" | "copy" | "rename" | "share" | "delete" | "refresh";

const ICON_BY_KIND: Record<ShellPinKind, string> = {
  cut: cutIcon,
  copy: copyIcon,
  rename: renameIcon,
  share: shareIcon,
  delete: deleteIcon,
  refresh: refreshIcon,
};

export function shellPinKind(entry: ShellMenuEntry): ShellPinKind | null {
  const id = entry.id ?? 0;
  if (id === BUILTIN_CUT) return "cut";
  if (id === BUILTIN_COPY) return "copy";
  if (id === BUILTIN_RENAME) return "rename";
  if (id === BUILTIN_DELETE) return "delete";
  if (id === BUILTIN_REFRESH) return "refresh";
  if (id === BUILTIN_SHARE) return "share";

  const label = (entry.label || "").trim();
  if (label.includes("剪切") || /^cut$/i.test(label)) return "cut";
  if (label.includes("复制") || /^copy$/i.test(label)) return "copy";
  if (label.includes("重命名") || /^rename$/i.test(label)) return "rename";
  if (label.includes("共享") || label.includes("分享") || /share/i.test(label)) {
    return "share";
  }
  if (label.includes("删除") || /^delete$/i.test(label)) return "delete";
  if (label.includes("刷新") || /^refresh$/i.test(label)) return "refresh";
  return null;
}

/** Prefer project SVG assets for known actions. */
export function shellMenuIcon(entry: ShellMenuEntry): string | null {
  const kind = shellPinKind(entry);
  if (kind) return ICON_BY_KIND[kind];
  return entry.icon || null;
}
