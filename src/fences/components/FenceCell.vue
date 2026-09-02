<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { iconFor, isRecycleBinPath } from "../helpers";
import { onIconPointerDown, onIconPointerUp } from "../iconDragCursor";
import {
  getSelectedIconPath,
  onSelectedIconPathChange,
  setSelectedIconPath,
  tryIconDoubleClick,
} from "../iconOpen";
import type { DesktopItem } from "../types";

const props = defineProps<{
  item: DesktopItem;
  native?: boolean;
  /** Match Sortable delay; files use 0. */
  dragDelay?: number;
}>();

const emit = defineEmits<{
  open: [];
  preload: [];
}>();

const canDrag = computed(() => !props.item.builtin);
const isRecycle = computed(() => isRecycleBinPath(props.item.path));
const lastClickAt = ref(0);
const pressed = ref(false);
const opening = ref(false);
const selected = ref(getSelectedIconPath() === props.item.path);
let openingTimer: number | null = null;
let unsubSelected: (() => void) | null = null;

onMounted(() => {
  unsubSelected = onSelectedIconPathChange(() => {
    selected.value = getSelectedIconPath() === props.item.path;
  });
});

onUnmounted(() => {
  unsubSelected?.();
  if (openingTimer !== null) window.clearTimeout(openingTimer);
});

function flashOpening() {
  opening.value = true;
  if (openingTimer !== null) window.clearTimeout(openingTimer);
  openingTimer = window.setTimeout(() => {
    opening.value = false;
    openingTimer = null;
  }, 220);
}

function onActivate(e: MouseEvent) {
  if (e.button !== 0) return;
  setSelectedIconPath(props.item.path);
  tryIconDoubleClick(lastClickAt, () => {
    flashOpening();
    emit("open");
  });
}

function onPointerDown(e: PointerEvent) {
  if (e.button !== 0) return;
  pressed.value = true;
  onIconPointerDown(canDrag.value, props.dragDelay);
}

function onPointerUp() {
  pressed.value = false;
  onIconPointerUp();
}
</script>

<template>
  <div
    class="cell"
    :class="{
      native: !!native,
      draggable: canDrag,
      'no-drag': !canDrag,
      'is-folder': !!item.isDir && !item.builtin,
      'is-recycle': isRecycle,
      'is-selected': selected,
      'is-pressed': pressed,
      'is-opening': opening,
    }"
    :data-path="item.path"
    :data-is-dir="item.isDir && !item.builtin ? '1' : '0'"
    :data-recycle="isRecycle ? '1' : '0'"
    :title="item.path"
    @click="onActivate"
    @pointerenter="emit('preload')"
    @pointerdown="onPointerDown"
    @pointerup="onPointerUp"
    @pointercancel="onPointerUp"
    @pointerleave="pressed = false"
  >
    <div class="icon" :class="{ folder: item.isDir && !item.builtin }">
      <img v-if="item.icon" :src="item.icon" alt="" draggable="false" />
      <template v-else>{{ iconFor(item) }}</template>
    </div>
    <div class="name">{{ item.name }}</div>
  </div>
</template>
