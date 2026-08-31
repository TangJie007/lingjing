import { ref, watch } from "vue";
import type { ShellMenuEntry } from "./types";
import {
  BUILTIN_RENAME,
  friendlyError,
  showFenceErrorToast,
} from "./fenceUi";
import { migrateFencePath } from "./fenceLayout";
import { requestRename } from "./useRenameDialog";

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

  async function prepare(targetPath: string) {
    if (!window.__TAURI__) return;

    const now = Date.now();
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

    const myGen = ++generation;
    path.value = targetPath;
    loading.value = true;
    error.value = "";
    // Clear immediately so we never show another item's shell commands.
    entries.value = [];

    try {
      const list = await window.__TAURI__.core.invoke<ShellMenuEntry[]>(
        "list_desktop_shell_context_menu",
        { path: targetPath },
      );
      if (myGen !== generation) return;
      entries.value = list || [];
      if (!entries.value.length) {
        error.value = "暂无可用菜单项";
      }
    } catch (e) {
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

  async function loadSubmenu(menuPath: number[]) {
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
    const myGen = generation;
    entry.loading = true;
    try {
      const list = await window.__TAURI__?.core.invoke<ShellMenuEntry[]>(
        "list_desktop_shell_context_submenu",
        { path: path.value, menuPath },
      );
      if (myGen === generation) entry.children = list || [];
    } catch (e) {
      if (myGen === generation) {
        entry.children = [{ label: friendlyError(e), disabled: true }];
      }
    } finally {
      if (myGen === generation) entry.loading = false;
    }
  }

  async function renameAt(p: string) {
    const next = await requestRename(p);
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
      showFenceToast("已重命名");
    } catch (e) {
      showFenceToast(friendlyError(e));
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
    prepare,
    loadSubmenu,
    runCommand,
  };
}

