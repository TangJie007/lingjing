<script setup lang="ts">
import {
  ContextMenuItem,
  ContextMenuPortal,
  ContextMenuSeparator,
  ContextMenuSub,
  ContextMenuSubContent,
  ContextMenuSubTrigger,
} from "reka-ui";
import type { ShellMenuEntry } from "../types";

defineProps<{
  entries: ShellMenuEntry[];
}>();

const emit = defineEmits<{
  command: [id: number];
}>();

function onSelect(entry: ShellMenuEntry) {
  if (entry.disabled || entry.id == null) return;
  emit("command", entry.id);
}
</script>

<template>
  <template v-for="(entry, idx) in entries" :key="idx">
    <ContextMenuSeparator v-if="entry.separator" class="sep" />
    <ContextMenuSub v-else-if="entry.children">
      <ContextMenuSubTrigger
        class="item has-sub"
        :disabled="!!entry.disabled"
      >
        <span class="ico">
          <img v-if="entry.icon" :src="entry.icon" alt="" />
        </span>
        <span class="lbl">{{ entry.label || "" }}</span>
        <span class="arrow">›</span>
      </ContextMenuSubTrigger>
      <ContextMenuPortal>
        <ContextMenuSubContent class="shell-ctx-sub" :side-offset="2">
          <ShellMenuEntries
            :entries="entry.children"
            @command="emit('command', $event)"
          />
        </ContextMenuSubContent>
      </ContextMenuPortal>
    </ContextMenuSub>
    <ContextMenuItem
      v-else
      class="item"
      :disabled="!!entry.disabled || entry.id == null"
      @select="onSelect(entry)"
    >
      <span class="ico">
        <img v-if="entry.icon" :src="entry.icon" alt="" />
      </span>
      <span class="lbl">{{ entry.label || "" }}</span>
    </ContextMenuItem>
  </template>
</template>
