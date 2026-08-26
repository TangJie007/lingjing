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
  command: [id: number, menuPath: number[]];
  submenu: [menuPath: number[]];
}>();

function onSelect(entry: ShellMenuEntry) {
  if (entry.disabled || entry.id == null || entry.id === 0) return;
  emit("command", entry.id, entry.menuPath || []);
}
</script>

<template>
  <template v-for="(entry, idx) in entries" :key="idx">
    <ContextMenuSeparator v-if="entry.separator" class="sep" />
    <ContextMenuSub
      v-else-if="entry.children"
      @update:open="$event && emit('submenu', entry.menuPath || [])"
    >
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
          <ContextMenuItem v-if="entry.loading" class="item" disabled>
            <span class="lbl">加载中…</span>
          </ContextMenuItem>
          <ShellMenuEntries
            v-else-if="entry.children.length"
            :entries="entry.children"
            @command="(id, path) => emit('command', id, path)"
            @submenu="emit('submenu', $event)"
          />
          <ContextMenuItem v-else class="item" disabled>
            <span class="lbl">无可用命令</span>
          </ContextMenuItem>
        </ContextMenuSubContent>
      </ContextMenuPortal>
    </ContextMenuSub>
    <ContextMenuItem
      v-else
      class="item"
        :disabled="!!entry.disabled"
      @select="onSelect(entry)"
    >
      <span class="ico">
        <img v-if="entry.icon" :src="entry.icon" alt="" />
      </span>
      <span class="lbl">{{ entry.label || "" }}</span>
    </ContextMenuItem>
  </template>
</template>
