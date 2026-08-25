import { onMounted, onUnmounted, ref } from "vue";
import type { DesktopItem, ShellMenuEntry } from "./types";

/**
 * Prefer the HTML shell menu. Native TrackPopupMenu on the embedded fence HWND
 * often returns Ok without a visible popup, which previously skipped fallback.
 */
export function useShellContextMenu() {
  const open = ref(false);
  const entries = ref<ShellMenuEntry[]>([]);
  const path = ref("");
  const style = ref({ left: "0px", top: "0px" });
  let lastOpenAt = 0;
  let generation = 0;

  function clearUi() {
    open.value = false;
    entries.value = [];
  }

  function hide() {
    clearUi();
    path.value = "";
    generation += 1;
  }

  function place(x: number, y: number) {
    const menu = document.getElementById("shell-ctx");
    const pad = 8;
    const w = menu?.offsetWidth || 220;
    const h = menu?.offsetHeight || 240;
    let left = x;
    let top = y;
    if (left + w > window.innerWidth - pad) {
      left = Math.max(pad, window.innerWidth - w - pad);
    }
    if (top + h > window.innerHeight - pad) {
      top = Math.max(pad, window.innerHeight - h - pad);
    }
    style.value = { left: `${left}px`, top: `${top}px` };
  }

  async function runCommand(commandId: number) {
    const p = path.value;
    hide();
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

  async function show(item: DesktopItem | null, x: number, y: number) {
    if (!window.__TAURI__) return;
    const now = Date.now();
    if (now - lastOpenAt < 250) return;
    lastOpenAt = now;

    const targetPath = item?.path || "";
    const myGen = ++generation;
    clearUi();
    path.value = targetPath;

    try {
      const list = await window.__TAURI__.core.invoke<ShellMenuEntry[]>(
        "list_desktop_shell_context_menu",
        { path: targetPath },
      );
      if (myGen !== generation) return;
      entries.value = list || [];
      if (!entries.value.length) {
        console.warn("shell context menu returned no entries", { path: targetPath });
      }
      path.value = targetPath;
      open.value = true;
      style.value = { left: `${x}px`, top: `${y}px` };
      requestAnimationFrame(() => {
        if (myGen === generation) place(x, y);
      });
    } catch (e) {
      console.warn("shell context menu failed", e);
      if (myGen === generation) clearUi();
    }
  }

  function onDocPointerDown(e: PointerEvent) {
    if (!open.value) return;
    const menu = document.getElementById("shell-ctx");
    if (menu && menu.contains(e.target as Node)) return;
    // Ignore the same right-button press that opened the menu.
    if (e.button === 2) return;
    hide();
  }

  function onWindowBlur() {
    if (open.value) {
      clearUi();
      path.value = "";
    }
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") hide();
  }

  onMounted(() => {
    document.addEventListener("pointerdown", onDocPointerDown);
    window.addEventListener("blur", onWindowBlur);
    window.addEventListener("resize", hide);
    document.addEventListener("keydown", onKeyDown);
  });

  onUnmounted(() => {
    document.removeEventListener("pointerdown", onDocPointerDown);
    window.removeEventListener("blur", onWindowBlur);
    window.removeEventListener("resize", hide);
    document.removeEventListener("keydown", onKeyDown);
  });

  return { open, entries, path, style, show, hide, runCommand };
}
