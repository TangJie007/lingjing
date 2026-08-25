<script setup lang="ts">
import { computed } from "vue";
import { iconFor } from "../helpers";
import type { DesktopItem } from "../types";

const props = defineProps<{
  item: DesktopItem;
  native?: boolean;
}>();

const emit = defineEmits<{
  open: [];
  context: [e: MouseEvent];
}>();

const canDrag = computed(() => !props.item.builtin);
</script>

<template>
  <div
    class="cell"
    :class="{ native: !!native, draggable: canDrag, 'no-drag': !canDrag }"
    :data-path="item.path"
    :title="item.path"
    @dblclick="emit('open')"
    @contextmenu.prevent.stop="emit('context', $event)"
    @pointerdown="
      (e: PointerEvent) => {
        if (e.button !== 2) return;
        e.preventDefault();
        e.stopPropagation();
        emit('context', e);
      }
    "
  >
    <div class="icon" :class="{ folder: item.isDir && !item.builtin }">
      <img v-if="item.icon" :src="item.icon" alt="" draggable="false" />
      <template v-else>{{ iconFor(item) }}</template>
    </div>
    <div class="name">{{ item.name }}</div>
  </div>
</template>
