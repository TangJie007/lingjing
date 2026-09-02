import { clearIconDragArm } from "./iconDragCursor";

let dragActive = false;
let dragEndedAt = 0;
let lastOpenAt = 0;
let selectedPath: string | null = null;
const selectedListeners = new Set<() => void>();

const DBLCLICK_MS = 480;
/** Ignore repeated open invokes (double-fire / menu + icon). */
export const OPEN_COOLDOWN_MS = 450;

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

/** Returns false when an open was attempted too soon after the previous one. */
export function beginIconOpen(): boolean {
  const now = performance.now();
  if (now - lastOpenAt < OPEN_COOLDOWN_MS) return false;
  lastOpenAt = now;
  return true;
}

export function getSelectedIconPath() {
  return selectedPath;
}

export function setSelectedIconPath(path: string | null) {
  if (selectedPath === path) {
    selectedListeners.forEach((l) => l());
    return;
  }
  selectedPath = path;
  selectedListeners.forEach((l) => l());
}

export function onSelectedIconPathChange(cb: () => void) {
  selectedListeners.add(cb);
  return () => {
    selectedListeners.delete(cb);
  };
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
