import { ref, watch } from "vue";
import type { ShellMenuEntry } from "./types";
import {
  BUILTIN_RENAME,
  friendlyError,
  showFenceErrorToast,
  showFenceToast,
} from "./fenceUi";
import { migrateFencePath, samePath } from "./fenceLayout";
import { requestRename } from "./useRenameDialog";

const FILE_MENU_CACHE_TTL_MS = 8000;
const MAX_MENU_CACHE = 64;

type MenuCacheEntry = {
  at: number;
  entries: ShellMenuEntry[];
};

/**
 * Shell context menu data layer for reka-ui ContextMenu.
 * UI open/close + positioning are owned by ContextMenuTrigger/Content.
 */
export function useShellContextMenu() {
  const entries = ref<ShellMenuEntry[]>([]);
  const path = ref("");
  const loading = ref(false);
  const error = ref("");
  const menuOpen = ref(false);
  let generation = 0;
  let lastPrepareAt = 0;
  let lastPreparePath = "\0";
  const menuCache = new Map<string, MenuCacheEntry>();
  const pendingLoads = new Map<string, Promise<ShellMenuEntry[]>>();
  const pendingSubmenuLoads = new Map<string, Promise<ShellMenuEntry[]>>();

  /**
   * Hover preload slot — kept separate from the visible menu state.
   * A new hover replaces the slot; a right-click adopts it into the live menu.
   */
  type PreloadSlot = {
    path: string;
    gen: number;
    entries: ShellMenuEntry[] | null;
    rootDone: boolean;
    /** Menu generation once adopted by a right-click; null while hovering. */
    adoptedGen: number | null;
  };
  let preloadGen = 0;
  let preloadSlot: PreloadSlot | null = null;

  function cacheKey(targetPath: string): string {
    return targetPath.trim();
  }

  function cloneEntries(list: ShellMenuEntry[]): ShellMenuEntry[] {
    return list.map((entry) => ({
      ...entry,
      menuPath: entry.menuPath ? [...entry.menuPath] : entry.menuPath,
      children: entry.children ? cloneEntries(entry.children) : entry.children,
    }));
  }

  function cachedEntries(key: string): ShellMenuEntry[] | null {
    // Blank desktop menu never uses the root cache — always fetch fresh
    // so Paste / clipboard-sensitive items stay current; only submenu preload applies.
    if (!key) return null;
    const hit = menuCache.get(key);
    if (!hit) return null;
    if (Date.now() - hit.at > FILE_MENU_CACHE_TTL_MS) {
      menuCache.delete(key);
      return null;
    }
    return cloneEntries(hit.entries);
  }

  function rememberEntries(key: string, list: ShellMenuEntry[]) {
    if (!key) return;
    menuCache.set(key, { at: Date.now(), entries: cloneEntries(list) });
    while (menuCache.size > MAX_MENU_CACHE) {
      const oldest = menuCache.keys().next().value;
      if (oldest == null) break;
      menuCache.delete(oldest);
    }
  }

  function invalidateMenuCache(targetPath?: string) {
    if (targetPath == null) {
      menuCache.clear();
      pendingSubmenuLoads.clear();
      return;
    }
    const key = cacheKey(targetPath);
    menuCache.delete(key);
    for (const loadKey of [...pendingSubmenuLoads.keys()]) {
      if (loadKey.startsWith(`${key}|`) || loadKey.startsWith("|")) {
        pendingSubmenuLoads.delete(loadKey);
      }
    }
  }

  function writeSubmenuToCache(key: string, menuPath: number[], list: ShellMenuEntry[]) {
    if (!key) return;
    const cached = menuCache.get(key);
    if (!cached) return;
    const entry = findSubmenu(cached.entries, menuPath);
    if (entry) {
      entry.children = cloneEntries(list);
      cached.at = Date.now();
    }
  }

  function clear() {
    entries.value = [];
    path.value = "";
    loading.value = false;
    error.value = "";
  }

  watch(menuOpen, (open) => {
    if (!open) {
      generation += 1;
      clear();
    }
  });

  // Empty shell while still "open" leaves a black strip — force-dismiss.
  watch([entries, loading, error, menuOpen], () => {
    if (
      menuOpen.value &&
      !loading.value &&
      !error.value &&
      entries.value.length === 0
    ) {
      menuOpen.value = false;
    }
  });

  async function preload(targetPath: string) {
    if (!window.__TAURI__ || !targetPath) return;
    const key = cacheKey(targetPath);
    const cached = cachedEntries(key);
    if (cached) return;

    const myGen = ++preloadGen;
    const slot: PreloadSlot = {
      path: targetPath,
      gen: myGen,
      entries: null,
      rootDone: false,
      adoptedGen: null,
    };
    // New hover discards any previous in-flight preload or its result.
    preloadSlot = slot;

    try {
      let load = pendingLoads.get(key);
      if (!load) {
        load = window.__TAURI__.core.invoke<ShellMenuEntry[]>(
          "list_desktop_shell_context_menu",
          { path: targetPath },
        );
        pendingLoads.set(key, load);
      }
      const list = await load;
      pendingLoads.delete(key);
      if (slot.gen !== myGen) return;
      const next = list || [];
      rememberEntries(key, next);
      slot.entries = next;
      slot.rootDone = true;
      void preloadSlotSubmenus(slot, key);
    } catch {
      pendingLoads.delete(key);
    }
  }

  /** Fetch submenus into the preload slot only while it is still current. */
  async function preloadSlotSubmenus(slot: PreloadSlot, key: string) {
    const submenuPaths = (slot.entries || [])
      .filter((entry) => entry.children)
      .map((entry) => entry.menuPath || []);
    for (const menuPath of submenuPaths) {
      if (slot !== preloadSlot || slot.gen !== preloadGen) return;
      await preloadSlotSubmenu(slot, key, menuPath);
    }
  }

  async function preloadSlotSubmenu(
    slot: PreloadSlot,
    key: string,
    menuPath: number[],
  ) {
    const loadKey = `${key}|${menuPath.join("/")}`;
    try {
      let load = pendingSubmenuLoads.get(loadKey);
      if (!load) {
        load = window.__TAURI__!.core.invoke<ShellMenuEntry[]>(
          "list_desktop_shell_context_submenu",
          { path: slot.path, menuPath },
        );
        pendingSubmenuLoads.set(loadKey, load);
      }
      const list = await load;
      pendingSubmenuLoads.delete(loadKey);
      if (slot !== preloadSlot || slot.gen !== preloadGen) return;
      const target = findSubmenu(slot.entries || [], menuPath);
      if (target) target.children = list || [];
    } catch {
      pendingSubmenuLoads.delete(loadKey);
    }
  }

  function adoptPreload(targetPath: string): boolean {
    const slot = preloadSlot;
    if (!slot || slot.path !== targetPath || !slot.rootDone || !slot.entries) {
      return false;
    }
    preloadSlot = null;
    // Invalidate any older in-flight prepare so its result cannot overwrite
    // the adopted preload data.
    generation += 1;
    path.value = targetPath;
    error.value = "";
    entries.value = cloneEntries(slot.entries);
    loading.value = false;
    if (!entries.value.length) {
      error.value = "暂无可用菜单项";
    }
    return true;
  }

  async function prepare(targetPath: string) {
    if (!window.__TAURI__) return;

    const now = Date.now();
    const key = cacheKey(targetPath);
    // pointerdown + contextmenu both call prepare for the same gesture
    if (
      targetPath === lastPreparePath &&
      now - lastPrepareAt < 400 &&
      (loading.value || entries.value.length)
    ) {
      return;
    }
    lastPrepareAt = now;
    lastPreparePath = targetPath;

    // Prefer the hover preload result for the clicked icon.
    if (adoptPreload(targetPath)) {
      void preloadSubmenus(generation);
      return;
    }

    const myGen = ++generation;
    path.value = targetPath;
    error.value = "";

    const cached = cachedEntries(key);
    if (cached) {
      loading.value = false;
      entries.value = cached;
      if (!entries.value.length) {
        error.value = "暂无可用菜单项";
      }
      void preloadSubmenus(myGen);
      return;
    }

    loading.value = true;
    // Clear immediately so we never show another item's shell commands.
    entries.value = [];

    try {
      let load = pendingLoads.get(key);
      if (!load) {
        load = window.__TAURI__.core.invoke<ShellMenuEntry[]>(
          "list_desktop_shell_context_menu",
          { path: targetPath },
        );
        pendingLoads.set(key, load);
      }
      const list = await load;
      pendingLoads.delete(key);
      if (myGen !== generation) return;
      const next = list || [];
      rememberEntries(key, next);
      entries.value = cloneEntries(next);
      if (!entries.value.length) {
        error.value = "暂无可用菜单项";
      }
      void preloadSubmenus(myGen);
    } catch (e) {
      pendingLoads.delete(key);
      if (myGen === generation) {
        entries.value = [];
        error.value = friendlyError(e);
      }
    } finally {
      if (myGen === generation) loading.value = false;
    }
  }

  function samePath(a: number[] | undefined, b: number[]) {
    return !!a && a.length === b.length && a.every((n, i) => n === b[i]);
  }

  function findSubmenu(
    list: ShellMenuEntry[],
    menuPath: number[],
  ): ShellMenuEntry | undefined {
    for (const entry of list) {
      if (entry.children && samePath(entry.menuPath, menuPath)) return entry;
      const nested = entry.children && findSubmenu(entry.children, menuPath);
      if (nested) return nested;
    }
  }

  /** Serialize submenu loads so immutable entry patches cannot clobber each other. */
  let submenuLoadChain: Promise<void> = Promise.resolve();

  /** Replace a submenu node immutably so Vue / reka-ui re-render SubContent. */
  function patchSubmenu(
    menuPath: number[],
    patch: (entry: ShellMenuEntry) => void,
  ): boolean {
    const next = cloneEntries(entries.value);
    const entry = findSubmenu(next, menuPath);
    if (!entry) return false;
    patch(entry);
    entries.value = next;
    return true;
  }

  async function loadSubmenu(menuPath: number[]) {
    const run = async () => {
      const entry = findSubmenu(entries.value, menuPath);
      if (!entry || entry.loading) return;
      if (entry.children && entry.children.length > 0) {
        const onlyPlaceholder =
          entry.children.length === 1 &&
          (!!entry.children[0]?.disabled ||
            entry.children[0]?.label === "加载超时" ||
            entry.children[0]?.label === "加载失败" ||
            entry.children[0]?.label === "无可用命令");
        if (!onlyPlaceholder) return;
      }
      // Builtin blank "新建" ships real children with an empty menuPath —
      // never round-trip those through the Shell submenu host.
      if (!menuPath.length) return;

      const myGen = generation;
      const key = cacheKey(path.value);
      const loadKey = `${key}|${menuPath.join("/")}`;
      if (
        !patchSubmenu(menuPath, (e) => {
          e.loading = true;
        })
      ) {
        return;
      }
      try {
        let load = pendingSubmenuLoads.get(loadKey);
        if (!load) {
          load = window.__TAURI__!.core.invoke<ShellMenuEntry[]>(
            "list_desktop_shell_context_submenu",
            { path: path.value, menuPath },
          );
          pendingSubmenuLoads.set(loadKey, load);
        }
        const list = await load;
        pendingSubmenuLoads.delete(loadKey);
        const next = list || [];
        writeSubmenuToCache(key, menuPath, next);
        if (myGen !== generation) return;
        patchSubmenu(menuPath, (e) => {
          e.children = cloneEntries(next);
          e.loading = false;
        });
      } catch (e) {
        pendingSubmenuLoads.delete(loadKey);
        if (myGen !== generation) return;
        patchSubmenu(menuPath, (entry) => {
          entry.children = [{ label: friendlyError(e), disabled: true }];
          entry.loading = false;
        });
      }
    };

    const done = submenuLoadChain.then(run, run);
    submenuLoadChain = done.then(
      () => undefined,
      () => undefined,
    );
    await done;
  }

  async function preloadSubmenus(myGen: number) {
    const submenuPaths = entries.value
      .filter((entry) => entry.children)
      .map((entry) => entry.menuPath || []);
    for (const menuPath of submenuPaths) {
      if (myGen !== generation) return;
      await loadSubmenu(menuPath);
    }
  }

  async function renameAt(p: string) {
    // System namespace icons (`::{CLSID}`) cannot be renamed.
    if (p.trim().startsWith("::")) {
      showFenceToast("系统图标不支持重命名");
      return;
    }
    const displayName = await lookupDesktopItemName(p);
    const next = await requestRename(p, displayName);
    if (!next) return;
    try {
      const newPath = await window.__TAURI__!.core.invoke<string>(
        "rename_desktop_item",
        { path: p, newName: next },
      );
      // Backend layout is updated; sync the frontend cache so the next
      // fence-items refresh keeps the same slot instead of appending.
      migrateFencePath(p, newPath || joinSibling(p, next));
      try {
        const items = await window.__TAURI__!.core.invoke("list_desktop_items");
        window.__fenceApply?.(items as never);
      } catch {
        /* watcher will refresh */
      }
      invalidateMenuCache(p);
      invalidateMenuCache(newPath || joinSibling(p, next));
      showFenceToast("已重命名");
    } catch (e) {
      showFenceToast(friendlyError(e));
    }
  }

  async function lookupDesktopItemName(targetPath: string): Promise<string | undefined> {
    if (!window.__TAURI__) return undefined;
    try {
      const items = await window.__TAURI__.core.invoke<
        Array<{ path?: string; name?: string }>
      >("list_desktop_items");
      const hit = (items || []).find((item) => samePath(item.path || "", targetPath));
      const name = hit?.name?.trim();
      return name || undefined;
    } catch {
      return undefined;
    }
  }

  function joinSibling(path: string, name: string): string {
    const i = Math.max(path.lastIndexOf("\\"), path.lastIndexOf("/"));
    if (i < 0) return name;
    return path.slice(0, i + 1) + name;
  }

  async function runCommand(entry: ShellMenuEntry) {
    const commandId = entry.id ?? 0;
    const menuPath = entry.menuPath || [];
    const p = path.value;

    // Close + clear immediately so the panel never sits empty as a black strip.
    menuOpen.value = false;
    generation += 1;
    clear();
    if (!window.__TAURI__) return;

    if (commandId === BUILTIN_RENAME) {
      if (!p) return;
      await renameAt(p);
      return;
    }

    try {
      await window.__TAURI__.core.invoke("invoke_desktop_shell_context_command", {
        path: p || "",
        commandId,
        menuPath,
      });
      invalidateMenuCache(p);
      // Delete uses the system Recycle Bin dialog — no extra toast.
    } catch (e) {
      const msg = String(e);
      console.error("[shell-menu] command failed", e);
      if (/已取消|取消/.test(msg)) return;
      if (msg.includes("重命名") || msg.includes("BUILTIN_RENAME")) {
        if (!p) return;
        await renameAt(p);
        return;
      }
      showFenceErrorToast(friendlyError(e));
    }
  }

  return {
    entries,
    path,
    loading,
    error,
    menuOpen,
    preload,
    prepare,
    invalidateMenuCache,
    loadSubmenu,
    runCommand,
  };
}

