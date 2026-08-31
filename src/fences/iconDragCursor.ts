/** Grab cursor after long-press, before Sortable drag starts. */

export const ICON_DRAG_ARM_MS = 220;

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

export function onIconPointerDown(canDrag: boolean) {
  if (!canDrag) return;
  clearIconDragArm();
  armTimer = window.setTimeout(() => {
    armTimer = null;
    stageEl()?.classList.add("icon-drag-armed");
  }, ICON_DRAG_ARM_MS);
}

export function onIconPointerUp() {
  clearIconDragArm();
}
