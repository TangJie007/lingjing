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
  >
    <div class="icon" :class="{ folder: item.isDir && !item.builtin }">
      <img v-if="item.icon" :src="item.icon" alt="" draggable="false" />
      <template v-else>{{ iconFor(item) }}</template>
    </div>
    <div class="name">{{ item.name }}</div>
  </div>
</template>
