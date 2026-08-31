import { nextTick, ref } from "vue";

const open = ref(false);
const value = ref("");

let currentPath = "";
let resolver: ((name: string | null) => void) | null = null;

function basename(path: string): string {
  return path.split(/[/\\]/).pop() || "";
}

function selectRange(name: string): [number, number] {
  const dot = name.lastIndexOf(".");
  if (dot > 0) return [0, dot];
  return [0, name.length];
}

export function useRenameDialog() {
  return { open, value };
}

export function requestRename(
  path: string,
  displayName?: string,
): Promise<string | null> {
  if (resolver) {
    resolver(null);
    resolver = null;
  }
  currentPath = path;
  const initial = (displayName || "").trim() || basename(path);
  value.value = initial;
  open.value = true;
  return new Promise((resolve) => {
    resolver = resolve;
    void nextTick(() => {
      const input = document.querySelector<HTMLInputElement>(".rename-dialog-input");
      if (!input) return;
      input.focus();
      const [start, end] = selectRange(value.value);
      input.setSelectionRange(start, end);
    });
  });
}

function finish(name: string | null) {
  open.value = false;
  const resolve = resolver;
  resolver = null;
  currentPath = "";
  resolve?.(name);
}

export function confirmRenameDialog() {
  const trimmed = value.value.trim();
  const base = basename(currentPath);
  const stem = (() => {
    const dot = base.lastIndexOf(".");
    return dot > 0 ? base.slice(0, dot) : base;
  })();
  if (!trimmed || trimmed === base || trimmed === stem) {
    finish(null);
    return;
  }
  finish(trimmed);
}

export function cancelRenameDialog() {
  finish(null);
}
