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
import FenceToast from "./components/FenceToast.vue";
import RenameDialog from "./components/RenameDialog.vue";
import ShellMenuEntries from "./components/ShellMenuEntries.vue";
import ShellMenuLoading from "./components/ShellMenuLoading.vue";
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
import { initFenceLayout } from "./fenceLayout";
import { friendlyError, showFenceToast } from "./fenceUi";
import type { DesktopItem, FenceGroupKey, FenceGroupState } from "./types";
import { cellPreviewDataUrl, useShellFileDrag } from "./useShellFileDrag";
import { useExternalFileDrop } from "./useExternalFileDrop";
import { markIconDragEnd, markIconDragStart } from "./iconOpen";
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

const {
  entries: shellEntries,
  loading: shellLoading,
  error: shellError,
  menuOpen: shellMenuOpen,
  prepare: prepareShellMenu,
  loadSubmenu: loadShellSubmenu,
  runCommand: runShellCommand,
} = useShellContextMenu();

const shellDrag = useShellFileDrag({
  onUiReset: () => {
    dragging.value = false;
    fenceDraggingKey.value = null;
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
  markIconDragStart();
  dragging.value = true;
  fenceDraggingKey.value = key === "app" ? null : key;
  const el = evt.item as HTMLElement;
  shellDrag.begin(el?.dataset?.path || "", cellPreviewDataUrl(el));
}

function onDragEnd(key: FenceGroupKey) {
  dragging.value = false;
  fenceDraggingKey.value = null;
  shellDrag.end();
  markIconDragEnd();
  persist(key);
  persistAll();
  flushPending();
}

function onDragMove(_evt: unknown, originalEvent?: Event) {
  const oe = originalEvent as MouseEvent | undefined;
  shellDrag.setModifiers(!!oe?.shiftKey, !!oe?.ctrlKey);
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

onMounted(async () => {
  window.__fenceApply = applyFenceItems;
  await initFenceLayout();
  await externalDrop.start();
  if (!window.__TAURI__) return;
  try {
    applyFenceItems(await window.__TAURI__.core.invoke<DesktopItem[]>("list_desktop_items"));
  } catch (e) {
    showFenceToast(friendlyError(e));
  }
  try {
    await window.__TAURI__.event.listen<DesktopItem[]>("fence-items", (e) =>
      applyFenceItems(e.payload || []),
    );
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
        :class="{ 'icon-dragging': dragging, 'file-drop-hover': fileDropHover }"
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
