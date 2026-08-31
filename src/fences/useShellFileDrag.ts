/** Shell OLE drag-out while Sortable is active (into Explorer folders / other apps). */

import { clearIconDragArm } from "./iconDragCursor";
import { friendlyError, showFenceToast } from "./fenceUi";

const PROBE_INTERVAL_MS = 120;
const PROBE_MIN_GAP_MS = 100;
/** Skip huge image data-URLs as OLE preview — decode/copy cost dominates. */
const MAX_PREVIEW_CHARS = 48_000;

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

  const sticky = [
    "is-dragging",
    "is-dragging-source",
    "drag-ghost",
    "sortable-drag",
    "sortable-ghost",
    "sortable-chosen",
    "sortable-fallback",
  ];
  const root = document.getElementById("stage") || document.body;
  root.querySelectorAll(sticky.map((c) => `.${c}`).join(",")).forEach((el) => {
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
  document.querySelectorAll("body > .sortable-fallback, body > .drag-ghost").forEach((el) => {
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

  function startPoll() {
    stopPoll();
    if (!activePath || suspended) return;
    pollTimer = window.setInterval(() => {
      void probe();
    }, PROBE_INTERVAL_MS);
  }

  function resetUi() {
    cancelHtmlDragArtifacts();
    opts?.onUiReset?.();
  }

  function begin(path: string, preview: string | null) {
    activePath = path;
    previewDataUrl =
      preview && preview.length <= MAX_PREVIEW_CHARS ? preview : null;
    shiftKey = false;
    ctrlKey = false;
    foreignHits = 0;
    lastProbe = 0;
    inFlight = false;
    pending = false;
    suspended = false;
    session += 1;
    startPoll();
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
    if (suspended === next) return;
    suspended = next;
    if (next) {
      foreignHits = 0;
      stopPoll();
    } else if (activePath) {
      startPoll();
    }
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
    if (now - lastProbe < PROBE_MIN_GAP_MS) return;
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
  if (!src.startsWith("data:image/png;base64,")) return null;
  if (src.length > MAX_PREVIEW_CHARS) return null;
  return src;
}
