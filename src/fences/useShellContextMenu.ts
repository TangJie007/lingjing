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
  let generation = 0;
  let lastPrepareAt = 0;
  let lastPreparePath = "\0";

  function clear() {
    entries.value = [];
    path.value = "";
    loading.value = false;
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
      if (myGen === generation) entries.value = [];
    } finally {
      if (myGen === generation) loading.value = false;
    }
  }

  async function runCommand(commandId: number) {
    const p = path.value;
    generation += 1;
    clear();
    if (!window.__TAURI__) return;
    try {
      await window.__TAURI__.core.invoke("invoke_desktop_shell_context_command", {
        path: p || "",
        commandId,
      });
    } catch (e) {
      console.warn("invoke shell command failed", e);
    }
  }

  return { entries, path, loading, prepare, runCommand, onOpenChange };
}
