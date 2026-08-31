<script setup lang="ts">
import { computed } from "vue";
import {
  ContextMenuItem,
  ContextMenuPortal,
  ContextMenuSeparator,
  ContextMenuSub,
  ContextMenuSubContent,
  ContextMenuSubTrigger,
} from "reka-ui";
import type { ShellMenuEntry } from "../types";
import { shellMenuIcon } from "../shellMenuIcons";
import ShellMenuLoading from "./ShellMenuLoading.vue";

const props = defineProps<{
  entries: ShellMenuEntry[];
  showMoreOptions?: boolean;
}>();

const emit = defineEmits<{
  command: [entry: ShellMenuEntry];
  submenu: [menuPath: number[]];
  moreOptions: [];
}>();

const pinEntries = computed(() => props.entries.filter((e) => e.pin));
const bodyEntries = computed(() => props.entries.filter((e) => !e.pin));

function onSelect(entry: ShellMenuEntry) {
  if (entry.disabled || entry.id == null || entry.id === 0) return;
  emit("command", entry);
}

function iconOf(entry: ShellMenuEntry) {
  return shellMenuIcon(entry);
}
</script>

<template>
  <div class="shell-ctx-layout">
    <div v-if="pinEntries.length" class="shell-ctx-pins">
      <ContextMenuItem
        v-for="(entry, idx) in pinEntries"
        :key="'pin-' + idx"
        class="pin"
        :class="{ destructive: entry.destructive }"
        :title="entry.label || ''"
        :disabled="!!entry.disabled"
        @select="onSelect(entry)"
      >
        <img v-if="iconOf(entry)" :src="iconOf(entry)!" alt="" />
        <span v-else class="pin-fallback">{{ (entry.label || "?").slice(0, 1) }}</span>
      </ContextMenuItem>
    </div>

    <div class="shell-ctx-scroll">
      <template v-for="(entry, idx) in bodyEntries" :key="idx">
        <ContextMenuSeparator v-if="entry.separator" class="sep" />
        <ContextMenuSub
          v-else-if="entry.children"
          @update:open="$event && emit('submenu', entry.menuPath || [])"
        >
          <ContextMenuSubTrigger class="item has-sub" :disabled="!!entry.disabled">
            <span class="ico">
              <img v-if="iconOf(entry)" :src="iconOf(entry)!" alt="" />
            </span>
            <span class="lbl">{{ entry.label || "" }}</span>
            <span class="arrow">›</span>
          </ContextMenuSubTrigger>
          <ContextMenuPortal>
            <ContextMenuSubContent class="shell-ctx-sub" :side-offset="2">
              <ShellMenuLoading v-if="entry.loading" compact />
              <ShellMenuEntries
                v-else-if="entry.children.length"
                :entries="entry.children"
                @command="emit('command', $event)"
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
          :class="{ destructive: entry.destructive }"
          :disabled="!!entry.disabled"
          @select="onSelect(entry)"
        >
          <span class="ico">
            <img v-if="iconOf(entry)" :src="iconOf(entry)!" alt="" />
          </span>
          <span class="lbl">{{ entry.label || "" }}</span>
        </ContextMenuItem>
      </template>
      <template v-if="props.showMoreOptions">
        <ContextMenuSeparator class="sep" />
        <ContextMenuItem class="item" @select="emit('moreOptions')">
          <span class="ico"></span>
          <span class="lbl">显示更多选项</span>
        </ContextMenuItem>
      </template>
    </div>
  </div>
</template>
