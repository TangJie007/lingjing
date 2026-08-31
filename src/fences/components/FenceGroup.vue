<script setup lang="ts">
import { VueDraggable } from "vue-draggable-plus";
import type { GroupOptions, MoveEvent, SortableEvent } from "sortablejs";
import FenceCell from "./FenceCell.vue";
import type { DesktopItem, FenceGroupKey } from "../types";
import { ICON_DRAG_ARM_MS } from "../iconDragCursor";

const props = defineProps<{
  groupKey: FenceGroupKey;
  items: DesktopItem[];
  native?: boolean;
  emptyText?: string;
  hostId?: string;
  dragGroup: string | GroupOptions;
  /** Return false to cancel Sortable reorder (e.g. drop into folder). */
  folderMoveGuard?: (evt: MoveEvent, originalEvent: Event) => boolean;
}>();

const emit = defineEmits<{
  "update:items": [items: DesktopItem[]];
  open: [path: string];
  sorted: [];
  added: [evt: SortableEvent];
  start: [evt: SortableEvent];
  end: [evt: SortableEvent];
}>();

function onUpdate(list: DesktopItem[]) {
  emit("update:items", list);
  emit("sorted");
}

function onMove(evt: MoveEvent, originalEvent: Event) {
  return props.folderMoveGuard?.(evt, originalEvent) !== false;
}
</script>

<template>
  <VueDraggable
    :model-value="items"
    class="drag-host"
    :class="{ 'fence-grid': !native }"
    :id="hostId"
    :animation="90"
    :group="dragGroup"
    filter=".no-drag"
    :prevent-on-filter="true"
    :force-fallback="true"
    :delay="ICON_DRAG_ARM_MS"
    :delay-on-touch-only="false"
    :fallback-tolerance="6"
    ghost-class="is-dragging-source"
    drag-class="is-dragging"
    fallback-class="drag-ghost"
    :fallback-on-body="true"
    :swap-threshold="0.65"
    :empty-insert-threshold="48"
    :on-move="onMove"
    @update:model-value="onUpdate"
    @add="emit('added', $event)"
    @start="emit('start', $event)"
    @end="emit('end', $event)"
  >
    <FenceCell
      v-for="item in items"
      :key="item.path"
      :item="item"
      :native="native"
      @open="emit('open', item.path)"
    />
    <div v-if="!items.length && emptyText" class="fence-empty no-drag">
      {{ emptyText }}
    </div>
  </VueDraggable>
</template>
