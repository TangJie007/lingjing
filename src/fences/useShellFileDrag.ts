/** Shell OLE drag-out while Sortable is active (into Explorer folders / other apps).
 *
 * Lifecycle and pitfalls: docs/fence-ole-drag-out.md
 * — arm directly (no cancel-then-arm); native watch is required when focus is on Explorer.
 */

import { clearIconDragArm } from "./iconDragCursor";
import { friendlyError, showFenceToast } from "./fenceUi";

/** Poll while cursor may leave the webview (no pointer events over Explorer). */
const PROBE_INTERVAL_MS = 32;
/** Skip huge image data-URLs as OLE preview — decode/copy cost dominates. */
const MAX_PREVIEW_CHARS = 48_000;

const LOG = "[shell-drag]";

function log(...args: unknown[]) {
  console.info(LOG, ...args);
}

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
  let inFlight = false;
  /** True once we commit to OLE — blocks Sortable end() from aborting handoff. */
  let handoff = false;
  /** end() during an in-flight probe — finish the probe before clearing. */
  let endAfterProbe = false;
  let suspended = false;
  let session = 0;
  let pollTimer: number | null = null;
  let unlistenHandoff: (() => void) | undefined;
  let armSeq = 0;

  function stopPoll() {
    if (pollTimer != null) {
      window.clearInterval(pollTimer);
    }
    pollTimer = null;
  }

  function startPoll() {
    stopPoll();
    if (!activePath || suspended || handoff) return;
    pollTimer = window.setInterval(() => {
      void probe();
    }, PROBE_INTERVAL_MS);
  }

  async function cancelNativeWatch() {
    if (!window.__TAURI__) return;
    try {
      await window.__TAURI__.core.invoke("cancel_desktop_outgoing_drag_watch");
    } catch {
      /* ignore */
    }
  }

  function armNativeWatch() {
    if (!window.__TAURI__ || !activePath || suspended || handoff) return;
    const path = activePath;
    const preview = previewDataUrl;
    const seq = ++armSeq;
    log("arm request", path);
    void window.__TAURI__.core
      .invoke("arm_desktop_outgoing_drag_watch", {
        path,
        previewDataUrl: preview,
      })
      .then((gen: unknown) => {
        if (seq !== armSeq) return;
        log("armed watch gen=", gen, path);
      })
      .catch((err: unknown) => {
        console.warn(LOG, "arm failed", err);
      });
  }

  function clearSession(cancelWatch = true) {
    stopPoll();
    if (cancelWatch) void cancelNativeWatch();
    activePath = "";
    inFlight = false;
    endAfterProbe = false;
    suspended = false;
    session += 1;
    armSeq += 1;
  }

  function resetUi() {
    cancelHtmlDragArtifacts();
    opts?.onUiReset?.();
  }

  function onNativeHandoff() {
    if (handoff) return;
    log("native handoff");
    handoff = true;
    endAfterProbe = false;
    stopPoll();
    activePath = "";
    // End Sortable while LBUTTON is still physically down (DoDragDrop needs it).
    cancelHtmlDragArtifacts();
    opts?.onUiReset?.();
  }

  function onNativeDone() {
    log("native done");
    inFlight = false;
    handoff = false;
    clearSession(false);
  }

  async function ensureNativeListeners() {
    if (!window.__TAURI__ || unlistenHandoff) return;
    try {
      unlistenHandoff = await window.__TAURI__.event.listen(
        "desktop-outgoing-drag-handoff",
        () => onNativeHandoff(),
      );
      await window.__TAURI__.event.listen(
        "desktop-outgoing-drag-done",
        () => onNativeDone(),
      );
      log("native listeners ready");
    } catch (e) {
      console.warn("desktop outgoing drag listen failed", e);
    }
  }

  function begin(path: string, preview: string | null) {
    activePath = path;
    previewDataUrl =
      preview && preview.length <= MAX_PREVIEW_CHARS ? preview : null;
    inFlight = false;
    handoff = false;
    endAfterProbe = false;
    suspended = false;
    session += 1;
    log("begin", path);
    void ensureNativeListeners();
    // arm() itself invalidates any prior watch — do NOT cancel-then-arm (race).
    armNativeWatch();
    startPoll();
  }

  function end() {
    // Sortable mouseup often races OLE handoff — don't abort mid-flight.
    if (handoff) {
      log("end ignored (handoff)");
      return;
    }
    log("end", { path: activePath, inFlight });
    stopPoll();
    if (inFlight) {
      // Keep activePath/session so a probe that already saw "foreign" can hand off.
      endAfterProbe = true;
      void cancelNativeWatch();
      return;
    }
    clearSession(true);
  }

  /** Pause OLE handoff (e.g. while hovering a folder drop target). */
  function setSuspended(next: boolean) {
    if (handoff) return;
    if (suspended === next) return;
    suspended = next;
    log("suspended=", next);
    if (next) {
      stopPoll();
      void cancelNativeWatch();
    } else if (activePath) {
      armNativeWatch();
      startPoll();
    }
  }

  /** Kept for callers; OLE now advertises COPY|MOVE and the target chooses. */
  function setModifiers(_shift: boolean, _ctrl: boolean) {}

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
      if (foreign) log("probe foreign=", foreign);
      // Single-threaded: set handoff before any further await so nested
      // Sortable end() → end() cannot abort this handoff.
      if (mySession !== session || handoff || suspended) {
        if (endAfterProbe) clearSession(true);
        return;
      }
      if (!foreign || !activePath) {
        if (endAfterProbe) clearSession(true);
        return;
      }

      handoff = true;
      endAfterProbe = false;
      const path = activePath;
      const preview = previewDataUrl;
      stopPoll();
      armSeq += 1;
      void cancelNativeWatch();
      activePath = "";
      log("probe → OLE", path);

      // End Sortable while LBUTTON is still physically down (DoDragDrop needs it).
      cancelHtmlDragArtifacts();
      opts?.onUiReset?.();

      try {
        // Native side advertises COPY|MOVE; Explorer / upload UIs pick the effect.
        const started = await window.__TAURI__.core.invoke<boolean>(
          "try_start_desktop_file_drag_if_foreign",
          {
            path,
            previewDataUrl: preview,
          },
        );
        log("OLE started=", started);
        if (!started) {
          // Cursor left foreign window before OLE could start.
          resetUi();
        }
        // When started, native drag already finished; UI was reset above.
      } catch (err) {
        const msg = friendlyError(err);
        log("OLE error", msg);
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
