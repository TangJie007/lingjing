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
import arrowRightIcon from "../../assets/svg/arrow-right-bold.svg";
import ShellMenuLoading from "./ShellMenuLoading.vue";

const props = defineProps<{
  entries: ShellMenuEntry[];
}>();

const emit = defineEmits<{
  command: [entry: ShellMenuEntry];
  submenu: [menuPath: number[]];
}>();

const pinEntries = computed(() => props.entries.filter((e) => e.pin));
const bodyEntries = computed(() => props.entries.filter((e) => !e.pin));

function isProperties(entry: ShellMenuEntry) {
  const label = (entry.label || "").replace(/&/g, "");
  return label.includes("属性") || /properties/i.test(label);
}

function onSelect(entry: ShellMenuEntry) {
  if (entry.disabled) return;
  // Shell sometimes reports 属性 with id 0; still run it.
  if ((entry.id == null || entry.id === 0) && !isProperties(entry)) return;
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
          v-else-if="entry.children && !isProperties(entry)"
          @update:open="$event && emit('submenu', entry.menuPath || [])"
        >
          <ContextMenuSubTrigger class="item has-sub" :disabled="!!entry.disabled">
            <span class="ico">
              <img v-if="iconOf(entry)" :src="iconOf(entry)!" alt="" />
            </span>
            <span class="lbl">{{ entry.label || "" }}</span>
            <img class="arrow" :src="arrowRightIcon" alt="" />
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
    </div>
  </div>
</template>
