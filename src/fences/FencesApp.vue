<script setup lang="ts">
import { onMounted, onUnmounted, reactive, ref } from "vue";
import type { SortableEvent } from "sortablejs";
import {
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuPortal,
  ContextMenuRoot,
  ContextMenuTrigger,
} from "reka-ui";
import FenceGroup from "./components/FenceGroup.vue";
import ShellMenuEntries from "./components/ShellMenuEntries.vue";
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
import type { DesktopItem, FenceGroupKey, FenceGroupState } from "./types";
import { cellPreviewDataUrl, useShellFileDrag } from "./useShellFileDrag";
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

const {
  entries: shellEntries,
  loading: shellLoading,
  error: shellError,
  prepare: prepareShellMenu,
  loadSubmenu: loadShellSubmenu,
  runCommand: runShellCommand,
  onOpenChange: onShellOpenChange,
} = useShellContextMenu();

const shellDrag = useShellFileDrag();

function persist(key: FenceGroupKey) {
  persistGroupOrder(groups[key].orderKey, groups[key].items);
}

function persistAll() {
  for (const k of Object.keys(groups) as FenceGroupKey[]) persist(k);
}

function onAdded(key: FenceGroupKey, evt: SortableEvent) {
  const path = (evt.item as HTMLElement)?.dataset?.path;
  if (!path) return;
  saveCategoryOverride(path, key);
  const item = groups[key].items.find((i) => i.path === path);
  if (item) item.kind = key;
  persist(key);
}

function onSorted(key: FenceGroupKey) {
  persist(key);
}

function onDragStart(key: FenceGroupKey, evt: SortableEvent) {
  dragging.value = true;
  fenceDraggingKey.value = key === "app" ? null : key;
  const el = evt.item as HTMLElement;
  shellDrag.begin(el?.dataset?.path || "", cellPreviewDataUrl(el));
}

function onDragEnd(key: FenceGroupKey) {
  dragging.value = false;
  fenceDraggingKey.value = null;
  shellDrag.end();
  persist(key);
  persistAll();
  flushPending();
}

function onDragMove(_evt: unknown, originalEvent?: Event) {
  const oe = originalEvent as MouseEvent | undefined;
  shellDrag.setShift(!!oe?.shiftKey);
  void shellDrag.probe();
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
  groups.app.items = partitionApps(list);
  const buckets = partitionByCategory(list);
  groups.image.items = loadOrder(IMAGE_ORDER_KEY, buckets.image);
  groups.document.items = loadOrder(DOC_ORDER_KEY, buckets.document);
  groups.folder.items = loadOrder(FOLDER_ORDER_KEY, buckets.folder);
  groups.media.items = loadOrder(MEDIA_ORDER_KEY, buckets.media);
  groups.archive.items = loadOrder(ARCHIVE_ORDER_KEY, buckets.archive);
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
  try {
    await window.__TAURI__.core.invoke("open_desktop_item", { path });
  } catch (e) {
    console.warn("open failed", e);
  }
}

function pathFromEvent(e: Event): string {
  const t = e.target as HTMLElement | null;
  const cell = t?.closest?.(".cell") as HTMLElement | null;
  return cell?.dataset?.path || "";
}

/** Load the isolated custom menu for icons and no menu on blank fence space. */
function onStageContextMenu(e: MouseEvent) {
  const target = e.target as HTMLElement | null;
  const path = pathFromEvent(e);
  const isBlankFenceArea =
    !!target?.closest?.(".fence") && !path;
  if (isBlankFenceArea) {
    e.preventDefault();
    e.stopImmediatePropagation();
    return;
  }
  void prepareShellMenu(path);
}

onMounted(async () => {
  window.__fenceApply = applyFenceItems;
  if (!window.__TAURI__) return;
  try {
    applyFenceItems(await window.__TAURI__.core.invoke<DesktopItem[]>("list_desktop_items"));
  } catch (e) {
    console.warn("list_desktop_items failed", e);
  }
  try {
    await window.__TAURI__.event.listen<DesktopItem[]>("fence-items", (e) =>
      applyFenceItems(e.payload || []),
    );
  } catch (e) {
    console.warn("listen fence-items failed", e);
  }
});

onUnmounted(() => {
  if (window.__fenceApply === applyFenceItems) delete window.__fenceApply;
});
</script>

<template>
  <ContextMenuRoot @update:open="onShellOpenChange">
    <ContextMenuTrigger as-child>
      <div
        id="stage"
        @contextmenu.capture="onStageContextMenu"
      >
        <FenceGroup
          group-key="app"
          v-model:items="groups.app.items"
          native
          host-id="apps"
          :drag-group="appDragGroup"
          @open="openItem"
          @sorted="onSorted('app')"
          @added="onAdded('app', $event)"
          @start="onDragStart('app', $event)"
          @end="onDragEnd('app')"
          @move="onDragMove"
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
              @open="openItem"
              @sorted="onSorted(key)"
              @added="onAdded(key, $event)"
              @start="onDragStart(key, $event)"
              @end="onDragEnd(key)"
              @move="onDragMove"
            />
          </section>
        </div>
      </div>
    </ContextMenuTrigger>

    <ContextMenuPortal>
      <ContextMenuContent class="shell-ctx" :collision-padding="8">
        <ContextMenuItem v-if="shellLoading && !shellEntries.length" class="item" disabled>
          <span class="lbl">加载中…</span>
        </ContextMenuItem>
        <ContextMenuItem v-else-if="shellError" class="item" disabled>
          <span class="lbl">{{ shellError }}</span>
        </ContextMenuItem>
        <ShellMenuEntries
          v-else
          :entries="shellEntries"
          @command="runShellCommand"
          @submenu="loadShellSubmenu"
        />
      </ContextMenuContent>
    </ContextMenuPortal>
  </ContextMenuRoot>
</template>
