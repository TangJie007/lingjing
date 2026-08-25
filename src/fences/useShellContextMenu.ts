import { ref } from "vue";
import type { ShellMenuEntry } from "./types";

/**
 * Shell context menu data layer for reka-ui ContextMenu.
 * UI open/close + positioning are owned by ContextMenuTrigger/Content.
 */
export function useShellContextMenu() {
  const entries = ref<ShellMenuEntry[]>([]);
  const path = ref("");
  const loading = ref(false);
  const error = ref("");
  let generation = 0;
  let lastPrepareAt = 0;
  let lastPreparePath = "\0";

  function clear() {
    entries.value = [];
    path.value = "";
    loading.value = false;
    error.value = "";
  }

  function onOpenChange(open: boolean) {
    if (!open) {
      generation += 1;
      clear();
    }
  }

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
        console.warn("shell context menu returned no entries", {
          path: targetPath,
        });
      }
    } catch (e) {
      console.warn("shell context menu failed", e);
      if (myGen === generation) {
        entries.value = [];
        error.value = String(e);
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
    if (!entry || entry.loading || entry.children?.length) return;
    const myGen = generation;
    entry.loading = true;
    try {
      const list = await window.__TAURI__?.core.invoke<ShellMenuEntry[]>(
        "list_desktop_shell_context_submenu",
        { path: path.value, menuPath },
      );
      if (myGen === generation) entry.children = list || [];
    } catch (e) {
      console.warn("load shell submenu failed", e);
      if (myGen === generation) {
        entry.children = [{ label: "加载超时", disabled: true }];
      }
    } finally {
      if (myGen === generation) entry.loading = false;
    }
  }

  async function runCommand(commandId: number, menuPath: number[] = []) {
    const p = path.value;
    generation += 1;
    clear();
    if (!window.__TAURI__) return;
    try {
      await window.__TAURI__.core.invoke("invoke_desktop_shell_context_command", {
        path: p || "",
        commandId,
        menuPath,
      });
    } catch (e) {
      console.warn("invoke shell command failed", e);
    }
  }

  return {
    entries,
    path,
    loading,
    error,
    prepare,
    loadSubmenu,
    runCommand,
    onOpenChange,
  };
}
