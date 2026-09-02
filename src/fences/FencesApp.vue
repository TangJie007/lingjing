<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref } from "vue";
import type { SortableEvent } from "sortablejs";
import {
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuPortal,
  ContextMenuRoot,
  ContextMenuTrigger,
} from "reka-ui";
import FenceGroup from "./components/FenceGroup.vue";
import { FILE_DRAG_DELAY_MS } from "./iconDragCursor";
import FenceToast from "./components/FenceToast.vue";
import RenameDialog from "./components/RenameDialog.vue";
import ShellMenuEntries from "./components/ShellMenuEntries.vue";
import ShellMenuLoading from "./components/ShellMenuLoading.vue";
import DynamicIsland from "./components/DynamicIsland.vue";
import {
  APP_ORDER_KEY,
  ARCHIVE_ORDER_KEY,
  DOC_ORDER_KEY,
  FOLDER_ORDER_KEY,
  IMAGE_ORDER_KEY,
  MEDIA_ORDER_KEY,
  emptyTextFor,
  loadOrder,
  partitionApps,
  partitionByCategory,
  persistGroupOrder,
  saveCategoryOverride,
} from "./helpers";
import {
  initFenceLayout,
  loadFencesCollapsed,
  saveFencesCollapsed,
} from "./fenceLayout";
import { forgetIcon, mergeGroupItems, pruneIconCache } from "./fenceItems";
import { friendlyError, showFenceToast } from "./fenceUi";
import type { DesktopItem, FenceGroupKey, FenceGroupState } from "./types";
import { cellPreviewDataUrl, useShellFileDrag } from "./useShellFileDrag";
import { useExternalFileDrop } from "./useExternalFileDrop";
import { beginIconOpen, markIconDragEnd, markIconDragStart } from "./iconOpen";
import { useShellContextMenu } from "./useShellContextMenu";

const groups = reactive<Record<FenceGroupKey, FenceGroupState>>({
  app: { key: "app", orderKey: APP_ORDER_KEY, items: [], emptyText: "", native: true },
  image: {
    key: "image",
    orderKey: IMAGE_ORDER_KEY,
    items: [],
    emptyText: "暂无图片",
    native: false,
    title: "图片",
  },
  document: {
    key: "document",
    orderKey: DOC_ORDER_KEY,
    items: [],
    emptyText: "暂无文档",
    native: false,
    title: "文档",
  },
  folder: {
    key: "folder",
    orderKey: FOLDER_ORDER_KEY,
    items: [],
    emptyText: "暂无文件夹",
    native: false,
    compact: true,
    title: "文件夹",
  },
  media: {
    key: "media",
    orderKey: MEDIA_ORDER_KEY,
    items: [],
    emptyText: "暂无媒体",
    native: false,
    compact: true,
    title: "媒体",
  },
  archive: {
    key: "archive",
    orderKey: ARCHIVE_ORDER_KEY,
    items: [],
    emptyText: "暂无压缩包",
    native: false,
    compact: true,
    title: "压缩包",
  },
});

const fileKeys: Exclude<FenceGroupKey, "app">[] = [
  "image",
  "document",
  "folder",
  "media",
  "archive",
];

const hostId: Record<Exclude<FenceGroupKey, "app">, string> = {
  image: "images",
  document: "documents",
  folder: "folders",
  media: "media",
  archive: "archives",
};

const appDragGroup = { name: "apps", pull: true, put: ["apps"] };
const fileDragGroup = { name: "files", pull: true, put: ["files", "apps"] };

const pendingFenceItems = ref<DesktopItem[] | null>(null);
const dragging = ref(false);
const fenceDraggingKey = ref<FenceGroupKey | null>(null);
const fileDropHover = ref(false);
/** Drop-on-folder / recycle while Sortable-dragging. */
const pendingSpecialDrop = ref<
  | { src: string; kind: "folder"; folder: string }
  | { src: string; kind: "recycle" }
  | null
>(null);
const dragSourcePath = ref("");
const dirtyGroups = new Set<FenceGroupKey>();
let lastDragPointer = { x: 0, y: 0 };
let folderDropEl: HTMLElement | null = null;
let dragPointerListening = false;
const fencesCollapsed = ref(false);
const fenceItemCount = computed(() =>
  (Object.keys(groups) as FenceGroupKey[]).reduce(
    (n, k) => n + groups[k].items.filter((i) => !i.builtin).length,
    0,
  ),
);

function toggleFencesCollapsed() {
  fencesCollapsed.value = !fencesCollapsed.value;
  saveFencesCollapsed(fencesCollapsed.value);
}

async function openMainSettings() {
  if (!window.__TAURI__) return;
  try {
    await window.__TAURI__.core.invoke("show_main_settings");
  } catch (e) {
    showFenceToast(friendlyError(e));
  }
}

async function disableDesktopOrganize() {
  if (!window.__TAURI__) return;
  try {
    await window.__TAURI__.core.invoke("set_desktop_organize", { enabled: false });
  } catch (e) {
    showFenceToast(friendlyError(e));
  }
}

const {
  entries: shellEntries,
  loading: shellLoading,
  error: shellError,
  menuOpen: shellMenuOpen,
  preload: preloadShellMenuData,
  prepare: prepareShellMenu,
  invalidateMenuCache: invalidateShellMenuCache,
  loadSubmenu: loadShellSubmenu,
  runCommand: runShellCommand,
} = useShellContextMenu();

const shellDrag = useShellFileDrag({
  onUiReset: () => {
    stopDragPointerListen();
    dragging.value = false;
    fenceDraggingKey.value = null;
    pendingSpecialDrop.value = null;
    dragSourcePath.value = "";
    clearFolderDropHighlight();
    markIconDragEnd();
    flushPending();
  },
});
const externalDrop = useExternalFileDrop({
  onHover: (active) => {
    fileDropHover.value = active;
  },
});

function persist(key: FenceGroupKey) {
  persistGroupOrder(groups[key].orderKey, groups[key].items);
}

function persistDirty() {
  for (const k of dirtyGroups) persist(k);
  dirtyGroups.clear();
}

function markDirty(key: FenceGroupKey) {
  dirtyGroups.add(key);
}

function onAdded(key: FenceGroupKey, evt: SortableEvent) {
  const path = (evt.item as HTMLElement)?.dataset?.path;
  if (!path) return;
  saveCategoryOverride(path, key);
  const item = groups[key].items.find((i) => i.path === path);
  if (item) item.kind = key;
  markDirty(key);
  persist(key);
}

function onSorted(key: FenceGroupKey) {
  markDirty(key);
  persist(key);
}

function clearFolderDropHighlight() {
  if (folderDropEl) {
    folderDropEl.classList.remove("folder-drop-over");
    folderDropEl = null;
  }
}

function setFolderDropHighlight(cell: HTMLElement | null) {
  if (folderDropEl === cell) return;
  if (folderDropEl) folderDropEl.classList.remove("folder-drop-over");
  folderDropEl = cell;
  cell?.classList.add("folder-drop-over");
}

function cellUnderPoint(
  x: number,
  y: number,
  dragged?: HTMLElement | null,
): HTMLElement | null {
  const under = document.elementFromPoint(x, y) as HTMLElement | null;
  const cell = under?.closest?.(".cell") as HTMLElement | null;
  if (!cell || cell === dragged) return null;
  return cell;
}

function specialDropFromCell(
  cell: HTMLElement | null,
  srcPath: string,
):
  | { cell: HTMLElement; kind: "folder"; folder: string }
  | { cell: HTMLElement; kind: "recycle" }
  | null {
  if (!cell || !srcPath) return null;
  const path = cell.dataset?.path || "";
  if (!path || path === srcPath) return null;
  if (cell.dataset?.recycle === "1") {
    return { cell, kind: "recycle" };
  }
  if (cell.dataset?.isDir === "1") {
    return { cell, kind: "folder", folder: path };
  }
  return null;
}

function findSpecialDrop(
  evt: { dragged?: HTMLElement; related?: HTMLElement },
  originalEvent?: Event,
): ReturnType<typeof specialDropFromCell> {
  const dragged = evt.dragged;
  const srcPath =
    dragSourcePath.value || dragged?.dataset?.path || "";
  const related = evt.related?.closest?.(".cell") as HTMLElement | null;
  const fromRelated = specialDropFromCell(related, srcPath);
  if (fromRelated && related !== dragged) return fromRelated;

  const fromTarget = (originalEvent?.target as HTMLElement | null)?.closest?.(
    ".cell",
  ) as HTMLElement | null;
  const fromEvt = specialDropFromCell(fromTarget, srcPath);
  if (fromEvt && fromTarget !== dragged) return fromEvt;

  // Cross-group (files → apps recycle): Sortable may not report related.
  return specialDropFromCell(
    cellUnderPoint(lastDragPointer.x, lastDragPointer.y, dragged),
    srcPath,
  );
}

function applySpecialDropTarget(
  drop: ReturnType<typeof specialDropFromCell>,
  srcPath: string,
): boolean {
  if (!drop || !srcPath) {
    setFolderDropHighlight(null);
    pendingSpecialDrop.value = null;
    shellDrag.setSuspended(false);
    return false;
  }
  setFolderDropHighlight(drop.cell);
  pendingSpecialDrop.value =
    drop.kind === "recycle"
      ? { src: srcPath, kind: "recycle" }
      : { src: srcPath, kind: "folder", folder: drop.folder };
  shellDrag.setSuspended(true);
  return true;
}

function onDragPointerMove(e: PointerEvent) {
  if (!dragging.value) return;
  if (!Number.isFinite(e.clientX) || !Number.isFinite(e.clientY)) return;
  lastDragPointer = { x: e.clientX, y: e.clientY };
  shellDrag.setModifiers(!!e.shiftKey, !!e.ctrlKey);
  const srcPath = dragSourcePath.value;
  if (!srcPath) return;
  const drop = specialDropFromCell(
    cellUnderPoint(e.clientX, e.clientY),
    srcPath,
  );
  if (applySpecialDropTarget(drop, srcPath)) return;
  // Outside the fence stage (or over another window with capture) — probe early.
  const stage = document.getElementById("stage");
  const r = stage?.getBoundingClientRect();
  if (
    r &&
    e.clientX >= r.left &&
    e.clientX <= r.right &&
    e.clientY >= r.top &&
    e.clientY <= r.bottom
  ) {
    return;
  }
  void shellDrag.probe();
}

function startDragPointerListen() {
  if (dragPointerListening) return;
  dragPointerListening = true;
  window.addEventListener("pointermove", onDragPointerMove, true);
}

function stopDragPointerListen() {
  if (!dragPointerListening) return;
  dragPointerListening = false;
  window.removeEventListener("pointermove", onDragPointerMove, true);
}

function removeItemLocally(path: string) {
  forgetIcon(path);
  for (const k of Object.keys(groups) as FenceGroupKey[]) {
    const list = groups[k].items;
    const idx = list.findIndex((i) => i.path === path);
    if (idx >= 0) {
      list.splice(idx, 1);
      markDirty(k);
    }
  }
}

function onDragStart(key: FenceGroupKey, evt: SortableEvent) {
  markIconDragStart();
  dragging.value = true;
  fenceDraggingKey.value = key === "app" ? null : key;
  dirtyGroups.clear();
  markDirty(key);
  pendingSpecialDrop.value = null;
  clearFolderDropHighlight();
  const el = evt.item as HTMLElement;
  const path = el?.dataset?.path || "";
  dragSourcePath.value = path;
  shellDrag.setSuspended(false);
  shellDrag.begin(path, cellPreviewDataUrl(el));
  startDragPointerListen();
}

async function onDragEnd(key: FenceGroupKey) {
  stopDragPointerListen();

  // OLE took over (or is taking over) — don't tear down the handoff or treat as fence drop.
  if (shellDrag.isPending()) {
    pendingSpecialDrop.value = null;
    dragSourcePath.value = "";
    clearFolderDropHighlight();
    dragging.value = false;
    fenceDraggingKey.value = null;
    markIconDragEnd();
    return;
  }

  const pending = pendingSpecialDrop.value;
  const src = dragSourcePath.value || pending?.src || "";
  let drop:
    | { kind: "folder"; folder: string }
    | { kind: "recycle" }
    | null = pending
      ? pending.kind === "recycle"
        ? { kind: "recycle" }
        : { kind: "folder", folder: pending.folder }
      : null;

  const under = specialDropFromCell(
    cellUnderPoint(lastDragPointer.x, lastDragPointer.y),
    src,
  );
  if (under) {
    drop =
      under.kind === "recycle"
        ? { kind: "recycle" }
        : { kind: "folder", folder: under.folder };
  }

  pendingSpecialDrop.value = null;
  dragSourcePath.value = "";
  clearFolderDropHighlight();
  dragging.value = false;
  fenceDraggingKey.value = null;
  shellDrag.setSuspended(true);
  shellDrag.end();
  markIconDragEnd();

  if (src && drop && window.__TAURI__) {
    markDirty(key);
    removeItemLocally(src);
    persistDirty();
    try {
      if (drop.kind === "recycle") {
        await window.__TAURI__.core.invoke("delete_desktop_item", {
          path: src,
        });
      } else {
        await window.__TAURI__.core.invoke("move_desktop_item_into_folder", {
          path: src,
          folderPath: drop.folder,
        });
      }
    } catch (e) {
      showFenceToast(friendlyError(e));
      try {
        applyFenceItems(
          await window.__TAURI__.core.invoke<DesktopItem[]>("list_desktop_items"),
        );
      } catch {
        /* ignore */
      }
    }
    flushPending();
    return;
  }

  markDirty(key);
  persistDirty();
  flushPending();
}

function onDragMove(
  evt: { dragged?: HTMLElement; related?: HTMLElement },
  originalEvent?: Event,
) {
  const oe = originalEvent as MouseEvent | undefined;
  if (oe && Number.isFinite(oe.clientX) && Number.isFinite(oe.clientY)) {
    lastDragPointer = { x: oe.clientX, y: oe.clientY };
  }
  shellDrag.setModifiers(!!oe?.shiftKey, !!oe?.ctrlKey);

  const srcPath =
    dragSourcePath.value || evt.dragged?.dataset?.path || "";
  const drop = findSpecialDrop(evt, originalEvent);
  if (applySpecialDropTarget(drop, srcPath)) {
    return false;
  }
  return true;
}

function flushPending() {
  if (!pendingFenceItems.value || dragging.value || shellDrag.isPending()) return;
  const items = pendingFenceItems.value;
  pendingFenceItems.value = null;
  render(items);
}

function render(items: DesktopItem[] | null | undefined) {
  const list = items || [];
  pruneIconCache(list.map((i) => i.path));
  const apps = partitionApps(list);
  const buckets = partitionByCategory(list);
  groups.app.items = mergeGroupItems(groups.app.items, apps);
  groups.image.items = mergeGroupItems(
    groups.image.items,
    loadOrder(IMAGE_ORDER_KEY, buckets.image),
  );
  groups.document.items = mergeGroupItems(
    groups.document.items,
    loadOrder(DOC_ORDER_KEY, buckets.document),
  );
  groups.folder.items = mergeGroupItems(
    groups.folder.items,
    loadOrder(FOLDER_ORDER_KEY, buckets.folder),
  );
  groups.media.items = mergeGroupItems(
    groups.media.items,
    loadOrder(MEDIA_ORDER_KEY, buckets.media),
  );
  groups.archive.items = mergeGroupItems(
    groups.archive.items,
    loadOrder(ARCHIVE_ORDER_KEY, buckets.archive),
  );
}

function applyFenceItems(items: DesktopItem[] | null | undefined) {
  if (dragging.value || shellDrag.isPending()) {
    pendingFenceItems.value = items || [];
    return;
  }
  pendingFenceItems.value = null;
  render(items);
}

async function openItem(path: string) {
  if (!window.__TAURI__) return;
  if (!beginIconOpen()) return;
  try {
    await window.__TAURI__.core.invoke("open_desktop_item", { path });
  } catch (e) {
    showFenceToast(friendlyError(e));
  }
}

function pathFromEvent(e: Event): string {
  const t = e.target as HTMLElement | null;
  const cell = t?.closest?.(".cell") as HTMLElement | null;
  return cell?.dataset?.path || "";
}

/**
 * Icons → item Shell menu.
 * Stage / #apps / fence blank → desktop blank menu (supports Paste).
 */
function onStageContextMenu(e: MouseEvent) {
  const path = pathFromEvent(e);
  // Do not preventDefault — ContextMenuTrigger needs the event to open.
  void prepareShellMenu(path);
}

function onStagePointerDown(e: PointerEvent) {
  if (e.button !== 2) return;
  void prepareShellMenu(pathFromEvent(e));
}

function preloadShellMenu(path: string) {
  if (!path || dragging.value || shellDrag.isPending()) return;
  void preloadShellMenuData(path);
}

onMounted(async () => {
  window.__fenceApply = applyFenceItems;
  await initFenceLayout();
  fencesCollapsed.value = loadFencesCollapsed();
  await externalDrop.start();
  if (!window.__TAURI__) return;
  try {
    applyFenceItems(await window.__TAURI__.core.invoke<DesktopItem[]>("list_desktop_items"));
  } catch (e) {
    showFenceToast(friendlyError(e));
  }
  try {
    await window.__TAURI__.event.listen<DesktopItem[]>("fence-items", (e) => {
      invalidateShellMenuCache();
      applyFenceItems(e.payload || []);
    });
  } catch (e) {
    console.warn("listen fence-items failed", e);
  }
  try {
    await window.__TAURI__.event.listen<string>("fence-toast", (e) => {
      if (e.payload) showFenceToast(friendlyError(e.payload));
    });
  } catch (e) {
    console.warn("listen fence-toast failed", e);
  }
});

onUnmounted(() => {
  stopDragPointerListen();
  externalDrop.stop();
  if (window.__fenceApply === applyFenceItems) delete window.__fenceApply;
});
</script>

<template>
  <ContextMenuRoot v-model:open="shellMenuOpen">
    <ContextMenuTrigger as-child>
      <div
        id="stage"
        class="stage"
        :class="{
          'icon-dragging': dragging,
          'file-drop-hover': fileDropHover,
          'fences-collapsed': fencesCollapsed,
        }"
        @pointerdown.capture="onStagePointerDown"
        @contextmenu.capture="onStageContextMenu"
      >
        <DynamicIsland
          :fences-collapsed="fencesCollapsed"
          :item-count="fenceItemCount"
          @toggle-fences="toggleFencesCollapsed"
          @open-settings="openMainSettings"
          @disable-organize="disableDesktopOrganize"
        />

        <FenceGroup
          group-key="app"
          v-model:items="groups.app.items"
          native
          host-id="apps"
          :drag-group="appDragGroup"
          :folder-move-guard="onDragMove"
          @open="openItem"
          @preload="preloadShellMenu"
          @sorted="onSorted('app')"
          @added="onAdded('app', $event)"
          @start="onDragStart('app', $event)"
          @end="onDragEnd('app')"
        />

        <div id="files">
          <section
            v-for="key in fileKeys"
            :key="key"
            class="fence"
            :class="{
              compact: groups[key].compact,
              'fence-dragging': fenceDraggingKey === key,
            }"
            :data-kind="key"
          >
            <div class="fence-title">{{ groups[key].title }}</div>
            <FenceGroup
              :group-key="key"
              v-model:items="groups[key].items"
              :empty-text="emptyTextFor(key)"
              :host-id="hostId[key]"
              :drag-group="fileDragGroup"
              :drag-delay="FILE_DRAG_DELAY_MS"
              :folder-move-guard="onDragMove"
              @open="openItem"
              @preload="preloadShellMenu"
              @sorted="onSorted(key)"
              @added="onAdded(key, $event)"
              @start="onDragStart(key, $event)"
              @end="onDragEnd(key)"
            />
          </section>
        </div>
      </div>
    </ContextMenuTrigger>

    <ContextMenuPortal>
      <ContextMenuContent
        v-if="shellLoading || !!shellError || shellEntries.length > 0"
        class="shell-ctx"
        :collision-padding="8"
      >
        <ShellMenuLoading v-if="shellLoading && !shellEntries.length" />
        <ContextMenuItem v-else-if="shellError" class="item shell-ctx-error" disabled>
          <span class="lbl">{{ shellError }}</span>
        </ContextMenuItem>
        <ShellMenuEntries
          v-else-if="shellEntries.length"
          :entries="shellEntries"
          @command="runShellCommand"
          @submenu="loadShellSubmenu"
        />
      </ContextMenuContent>
    </ContextMenuPortal>
  </ContextMenuRoot>
  <FenceToast />
  <RenameDialog />
</template>
