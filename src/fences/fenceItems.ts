/** Diff / icon-cache helpers for fence list updates. */

import type { DesktopItem } from "./types";

const iconByPath = new Map<string, string>();

function rememberIcon(item: DesktopItem) {
  if (item.icon) iconByPath.set(item.path, item.icon);
}

function withCachedIcon(item: DesktopItem): DesktopItem {
  if (item.icon) {
    rememberIcon(item);
    return item;
  }
  const cached = iconByPath.get(item.path);
  return cached ? { ...item, icon: cached } : item;
}

/** Prune icon cache entries that no longer appear on the desktop. */
export function pruneIconCache(livePaths: Iterable<string>) {
  const live = new Set(livePaths);
  for (const key of iconByPath.keys()) {
    if (!live.has(key)) iconByPath.delete(key);
  }
}

/**
 * Merge incoming list onto previous group items.
 * Reuses previous object identity when path/name/kind/icon are unchanged
 * so Vue can skip re-rendering those cells.
 */
export function mergeGroupItems(
  prev: DesktopItem[],
  next: DesktopItem[],
): DesktopItem[] {
  if (prev === next) return prev;
  if (
    prev.length === next.length &&
    prev.every((p, i) => p.path === next[i]?.path)
  ) {
    let changed = false;
    const out = prev.map((p, i) => {
      const n = withCachedIcon(next[i]);
      if (
        p.name === n.name &&
        p.kind === n.kind &&
        p.isDir === n.isDir &&
        !!p.builtin === !!n.builtin &&
        (p.icon || "") === (n.icon || "")
      ) {
        return p;
      }
      changed = true;
      return n;
    });
    return changed ? out : prev;
  }

  const prevMap = new Map(prev.map((p) => [p.path, p]));
  return next.map((raw) => {
    const n = withCachedIcon(raw);
    const old = prevMap.get(n.path);
    if (
      old &&
      old.name === n.name &&
      old.kind === n.kind &&
      old.isDir === n.isDir &&
      !!old.builtin === !!n.builtin &&
      (old.icon || "") === (n.icon || "")
    ) {
      return old;
    }
    if (old?.icon && !n.icon) return { ...n, icon: old.icon };
    return n;
  });
}

export function forgetIcon(path: string) {
  iconByPath.delete(path);
}
