import { clearIconDragArm } from "./iconDragCursor";

let dragActive = false;
let dragEndedAt = 0;

const DBLCLICK_MS = 480;

export function markIconDragStart() {
  dragActive = true;
  clearIconDragArm();
}

export function markIconDragEnd() {
  dragActive = false;
  dragEndedAt = performance.now();
  clearIconDragArm();
}

export function canOpenIcon() {
  if (dragActive) return false;
  return performance.now() - dragEndedAt > 280;
}

export function tryIconDoubleClick(
  lastClickAt: { value: number },
  onOpen: () => void,
): boolean {
  if (!canOpenIcon()) {
    lastClickAt.value = 0;
    return false;
  }
  const now = performance.now();
  if (now - lastClickAt.value < DBLCLICK_MS) {
    lastClickAt.value = 0;
    onOpen();
    return true;
  }
  lastClickAt.value = now;
  return false;
}
