/** Shell OLE drag-out while Sortable is active (into Explorer folders / other apps). */

import { clearIconDragArm } from "./iconDragCursor";
import { friendlyError, showFenceToast } from "./fenceUi";

/** Poll while cursor may leave the webview (no pointer events over Explorer). */
const PROBE_INTERVAL_MS = 32;
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
  let inFlight = false;
  /** True once we commit to OLE — blocks Sortable end() from aborting handoff. */
  let handoff = false;
  /** end() during an in-flight probe — finish the probe before clearing. */
  let endAfterProbe = false;
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
    if (!activePath || suspended || handoff) return;
    pollTimer = window.setInterval(() => {
      void probe();
    }, PROBE_INTERVAL_MS);
  }

  function clearSession() {
    stopPoll();
    activePath = "";
    inFlight = false;
    endAfterProbe = false;
    suspended = false;
    session += 1;
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
    inFlight = false;
    handoff = false;
    endAfterProbe = false;
    suspended = false;
    session += 1;
    startPoll();
  }

  function end() {
    // Sortable mouseup often races OLE handoff — don't abort mid-flight.
    if (handoff) return;
    stopPoll();
    if (inFlight) {
      // Keep activePath/session so a probe that already saw "foreign" can hand off.
      endAfterProbe = true;
      return;
    }
    clearSession();
  }

  /** Pause OLE handoff (e.g. while hovering a folder drop target). */
  function setSuspended(next: boolean) {
    if (handoff) return;
    if (suspended === next) return;
    suspended = next;
    if (next) {
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
    return handoff;
  }

  async function probe() {
    if (
      !activePath ||
      handoff ||
      inFlight ||
      suspended ||
      !window.__TAURI__
    ) {
      return;
    }

    const mySession = session;
    inFlight = true;
    try {
      const foreign = await window.__TAURI__.core.invoke<boolean>(
        "is_desktop_drag_over_foreign",
      );
      // Single-threaded: set handoff before any further await so nested
      // Sortable end() → end() cannot abort this handoff.
      if (mySession !== session || handoff || suspended) {
        if (endAfterProbe) clearSession();
        return;
      }
      if (!foreign || !activePath) {
        if (endAfterProbe) clearSession();
        return;
      }

      handoff = true;
      endAfterProbe = false;
      const path = activePath;
      const mode = ctrlKey && !shiftKey ? "copy" : "move";
      const preview = previewDataUrl;
      stopPoll();
      activePath = "";

      // End Sortable while LBUTTON is still physically down (DoDragDrop needs it).
      cancelHtmlDragArtifacts();
      opts?.onUiReset?.();

      try {
        const started = await window.__TAURI__.core.invoke<boolean>(
          "try_start_desktop_file_drag_if_foreign",
          {
            path,
            mode,
            previewDataUrl: preview,
          },
        );
        if (!started) {
          // Cursor left foreign window before OLE could start.
          resetUi();
        }
        // When started, native drag already finished; UI was reset above.
      } catch (err) {
        const msg = friendlyError(err);
        if (!msg.includes("鼠标已松开")) {
          showFenceToast(msg);
        }
        resetUi();
      }
    } catch (err) {
      showFenceToast(friendlyError(err));
      resetUi();
    } finally {
      inFlight = false;
      handoff = false;
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
