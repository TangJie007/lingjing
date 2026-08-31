/** Shell OLE drag-out while Sortable is active (foreign window hover). */

import { friendlyError, showFenceToast } from "./fenceUi";

export function useShellFileDrag() {
  let activePath = "";
  let previewDataUrl: string | null = null;
  let shiftKey = false;
  let foreignHits = 0;
  let lastProbe = 0;
  let inFlight = false;
  let pending = false;

  function begin(path: string, preview: string | null) {
    activePath = path;
    previewDataUrl = preview;
    shiftKey = false;
    foreignHits = 0;
    lastProbe = 0;
    inFlight = false;
    pending = false;
  }

  function end() {
    activePath = "";
    pending = false;
    inFlight = false;
  }

  function setShift(v: boolean) {
    shiftKey = v;
  }

  function isPending() {
    return pending;
  }

  async function probe() {
    if (!activePath || pending || inFlight || !window.__TAURI__) return;
    const now = performance.now();
    if (now - lastProbe < 45) return;
    lastProbe = now;

    inFlight = true;
    try {
      const foreign = await window.__TAURI__.core.invoke<boolean>(
        "is_desktop_drag_over_foreign",
      );
      if (!activePath || pending) return;
      if (!foreign) {
        foreignHits = 0;
        return;
      }
      foreignHits += 1;
      if (foreignHits < 2) return;

      pending = true;
      const path = activePath;
      const mode = shiftKey ? "move" : "copy";
      const preview = previewDataUrl;
      end();

      // End Sortable fallback drag.
      document.dispatchEvent(
        new PointerEvent("pointerup", { bubbles: true, cancelable: true, button: 0 }),
      );

      try {
        await window.__TAURI__.core.invoke("start_desktop_file_drag", {
          path,
          mode,
          previewDataUrl: preview,
        });
      } catch (err) {
        showFenceToast(friendlyError(err));
      }
    } catch (err) {
      showFenceToast(friendlyError(err));
      foreignHits = 0;
    } finally {
      inFlight = false;
    }
  }

  return { begin, end, setShift, isPending, probe };
}

export function cellPreviewDataUrl(el: HTMLElement | null): string | null {
  const img = el?.querySelector("img");
  const src = img?.src || "";
  return src.startsWith("data:image/png;base64,") ? src : null;
}
