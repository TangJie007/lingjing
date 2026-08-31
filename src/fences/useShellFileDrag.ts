/** Shell OLE drag-out while Sortable is active (into Explorer folders / other apps). */

import { clearIconDragArm } from "./iconDragCursor";
import { friendlyError, showFenceToast } from "./fenceUi";

function cancelHtmlDragArtifacts() {
  // Force-end Sortable / pointer capture left behind when the mouse is released
  // over another window (Explorer) after OLE handoff.
  for (const type of ["pointercancel", "pointerup"] as const) {
    document.dispatchEvent(
      new PointerEvent(type, {
        bubbles: true,
        cancelable: true,
        button: 0,
        buttons: 0,
      }),
    );
  }
  document.dispatchEvent(
    new MouseEvent("mouseup", { bubbles: true, cancelable: true, button: 0 }),
  );
  // Strip leftover Sortable / drag classes so cursor is no longer "grabbing".
  const sticky = [
    "is-dragging",
    "is-dragging-source",
    "drag-ghost",
    "sortable-drag",
    "sortable-ghost",
    "sortable-chosen",
    "sortable-fallback",
  ];
  document.querySelectorAll(sticky.map((c) => `.${c}`).join(",")).forEach((el) => {
    el.classList.remove(...sticky);
    if (el instanceof HTMLElement) {
      el.style.removeProperty("display");
      el.style.removeProperty("opacity");
      el.style.removeProperty("transform");
      el.style.removeProperty("position");
      el.style.removeProperty("top");
      el.style.removeProperty("left");
      el.style.removeProperty("width");
      el.style.removeProperty("height");
      el.style.removeProperty("z-index");
      el.style.removeProperty("pointer-events");
    }
  });
  document.querySelectorAll(".sortable-fallback, .drag-ghost").forEach((el) => {
    el.remove();
  });
  document.getElementById("stage")?.classList.remove("icon-dragging", "icon-drag-armed");
  clearIconDragArm();
}

export function useShellFileDrag(opts?: {
  /** Called when handing off to / finishing OLE drag — reset Vue drag UI here. */
  onUiReset?: () => void;
}) {
  let activePath = "";
  let previewDataUrl: string | null = null;
  let shiftKey = false;
  let ctrlKey = false;
  let foreignHits = 0;
  let lastProbe = 0;
  let inFlight = false;
  let pending = false;
  let suspended = false;
  let session = 0;
  let pollTimer: number | null = null;

  function stopPoll() {
    if (pollTimer != null) {
      window.clearInterval(pollTimer);
      pollTimer = null;
    }
  }

  function resetUi() {
    cancelHtmlDragArtifacts();
    opts?.onUiReset?.();
  }

  function begin(path: string, preview: string | null) {
    activePath = path;
    previewDataUrl = preview;
    shiftKey = false;
    ctrlKey = false;
    foreignHits = 0;
    lastProbe = 0;
    inFlight = false;
    pending = false;
    suspended = false;
    session += 1;
    stopPoll();
    // Cursor leaves the WebView when over Explorer — Sortable "move" stops.
    // Poll independently so we still hand off to OLE drag-out.
    pollTimer = window.setInterval(() => {
      void probe();
    }, 50);
  }

  function end() {
    stopPoll();
    activePath = "";
    pending = false;
    inFlight = false;
    foreignHits = 0;
    suspended = false;
    session += 1;
  }

  /** Pause OLE handoff (e.g. while hovering a folder drop target). */
  function setSuspended(next: boolean) {
    suspended = next;
    if (next) foreignHits = 0;
  }

  function setModifiers(shift: boolean, ctrl: boolean) {
    shiftKey = shift;
    ctrlKey = ctrl;
  }

  function isPending() {
    return pending;
  }

  async function probe() {
    if (!activePath || pending || inFlight || suspended || !window.__TAURI__) return;
    const now = performance.now();
    if (now - lastProbe < 40) return;
    lastProbe = now;

    const mySession = session;
    inFlight = true;
    try {
      const foreign = await window.__TAURI__.core.invoke<boolean>(
        "is_desktop_drag_over_foreign",
      );
      if (mySession !== session || !activePath || pending || suspended) return;
      if (!foreign) {
        foreignHits = 0;
        return;
      }
      foreignHits += 1;
      if (foreignHits < 2) return;

      pending = true;
      const path = activePath;
      const mode = ctrlKey && !shiftKey ? "copy" : "move";
      const preview = previewDataUrl;
      stopPoll();
      end();

      // Soft-cancel Sortable before OLE takes the mouse (physical button still down).
      cancelHtmlDragArtifacts();
      opts?.onUiReset?.();

      try {
        await window.__TAURI__.core.invoke("start_desktop_file_drag", {
          path,
          mode,
          previewDataUrl: preview,
        });
      } catch (err) {
        const msg = friendlyError(err);
        if (!msg.includes("鼠标已松开")) {
          showFenceToast(msg);
        }
      } finally {
        // OLE drag ends when the user releases over Explorer — fence never sees
        // that mouseup, so force-clear grabbing cursor / Sortable leftovers.
        resetUi();
      }
    } catch (err) {
      showFenceToast(friendlyError(err));
      foreignHits = 0;
      resetUi();
    } finally {
      inFlight = false;
      pending = false;
    }
  }

  return { begin, end, setModifiers, setSuspended, isPending, probe };
}

export function cellPreviewDataUrl(el: HTMLElement | null): string | null {
  const img = el?.querySelector("img");
  const src = img?.src || "";
  return src.startsWith("data:image/png;base64,") ? src : null;
}
