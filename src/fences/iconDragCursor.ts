/** Grab cursor arm delay for app icons (long-press). Files use 0 = immediate. */
export const ICON_DRAG_ARM_MS = 220;
/** Files: Sortable starts on press + small move, no long-press. */
export const FILE_DRAG_DELAY_MS = 0;

let armTimer: number | null = null;

function stageEl() {
  return document.getElementById("stage");
}

export function clearIconDragArm() {
  if (armTimer !== null) {
    window.clearTimeout(armTimer);
    armTimer = null;
  }
  stageEl()?.classList.remove("icon-drag-armed");
}

export function onIconPointerDown(canDrag: boolean, delayMs: number = ICON_DRAG_ARM_MS) {
  if (!canDrag) return;
  clearIconDragArm();
  if (delayMs <= 0) {
    stageEl()?.classList.add("icon-drag-armed");
    return;
  }
  armTimer = window.setTimeout(() => {
    armTimer = null;
    stageEl()?.classList.add("icon-drag-armed");
  }, delayMs);
}

export function onIconPointerUp() {
  clearIconDragArm();
}
