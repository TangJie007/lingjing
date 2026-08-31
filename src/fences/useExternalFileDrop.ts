/** Accept Explorer (and other app) file drops onto the desktop fence. */

import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

type DropPayload =
  | { type: "enter"; paths: string[] }
  | { type: "over"; paths?: string[] }
  | { type: "drop"; paths: string[] }
  | { type: "leave" };

/**
 * Visual hover + Chromium drop-allowance.
 * Actual placement is handled in Rust (`on_webview_event` → `place_paths_on_desktop`).
 */
export function useExternalFileDrop(opts: {
  onHover?: (active: boolean) => void;
}) {
  let unlisten: (() => void) | null = null;
  let allowDrop: ((e: DragEvent) => void) | null = null;
  let swallowDrop: ((e: DragEvent) => void) | null = null;

  async function start() {
    if (!window.__TAURI__) return;

    // Chromium needs preventDefault on dragover or OS file drops are cancelled.
    allowDrop = (e: DragEvent) => {
      e.preventDefault();
      if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
    };
    swallowDrop = (e: DragEvent) => {
      e.preventDefault();
    };
    window.addEventListener("dragenter", allowDrop);
    window.addEventListener("dragover", allowDrop);
    window.addEventListener("drop", swallowDrop);

    try {
      const win = getCurrentWebviewWindow();
      unlisten = await win.onDragDropEvent((event) => {
        const payload = event.payload as DropPayload;
        if (payload.type === "enter" || payload.type === "over") {
          opts.onHover?.(true);
          return;
        }
        opts.onHover?.(false);
      });
    } catch (e) {
      console.warn("desktop file drop listen failed", e);
    }
  }

  function stop() {
    if (allowDrop) {
      window.removeEventListener("dragenter", allowDrop);
      window.removeEventListener("dragover", allowDrop);
      allowDrop = null;
    }
    if (swallowDrop) {
      window.removeEventListener("drop", swallowDrop);
      swallowDrop = null;
    }
    unlisten?.();
    unlisten = null;
    opts.onHover?.(false);
  }

  return { start, stop };
}
