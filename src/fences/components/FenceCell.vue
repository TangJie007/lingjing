<script setup lang="ts">
import { computed, ref } from "vue";
import { iconFor } from "../helpers";
import { onIconPointerDown, onIconPointerUp } from "../iconDragCursor";
import { tryIconDoubleClick } from "../iconOpen";
import type { DesktopItem } from "../types";

const props = defineProps<{
  item: DesktopItem;
  native?: boolean;
}>();

const emit = defineEmits<{
  open: [];
}>();

const canDrag = computed(() => !props.item.builtin);
const lastClickAt = ref(0);

function onActivate(e: MouseEvent) {
  if (e.button !== 0) return;
  tryIconDoubleClick(lastClickAt, () => emit("open"));
}

function onPointerDown(e: PointerEvent) {
  if (e.button !== 0) return;
  onIconPointerDown(canDrag.value);
}

function onPointerUp() {
  onIconPointerUp();
}
</script>

<template>
  <div
    class="cell"
    :class="{ native: !!native, draggable: canDrag, 'no-drag': !canDrag }"
    :data-path="item.path"
    :title="item.path"
    @click="onActivate"
    @pointerdown="onPointerDown"
    @pointerup="onPointerUp"
    @pointercancel="onPointerUp"
  >
    <div class="icon" :class="{ folder: item.isDir && !item.builtin }">
      <img v-if="item.icon" :src="item.icon" alt="" draggable="false" />
      <template v-else>{{ iconFor(item) }}</template>
    </div>
    <div class="name">{{ item.name }}</div>
  </div>
</template>
