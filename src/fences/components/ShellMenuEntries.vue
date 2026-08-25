<script setup lang="ts">
import type { ShellMenuEntry } from "../types";

defineProps<{
  entries: ShellMenuEntry[];
  nested?: boolean;
}>();

const emit = defineEmits<{
  command: [id: number];
}>();

function onItemClick(entry: ShellMenuEntry) {
  if (entry.disabled) return;
  if (entry.children && entry.children.length) return;
  if (entry.id == null) return;
  emit("command", entry.id);
}
</script>

<template>
  <ul v-if="nested" class="sub">
    <template v-for="(entry, idx) in entries" :key="idx">
      <li v-if="entry.separator" class="sep" role="separator" />
      <li
        v-else
        class="item"
        :class="{
          disabled: !!entry.disabled,
          'has-sub': !!(entry.children && entry.children.length),
        }"
        role="menuitem"
        @click.stop.prevent="onItemClick(entry)"
      >
        <span class="ico">
          <img v-if="entry.icon" :src="entry.icon" alt="" />
        </span>
        <span class="lbl">{{ entry.label || "" }}</span>
        <span v-if="entry.children?.length" class="arrow">›</span>
        <ShellMenuEntries
          v-if="entry.children?.length"
          nested
          :entries="entry.children"
          @command="emit('command', $event)"
        />
      </li>
    </template>
  </ul>
  <template v-else>
    <template v-for="(entry, idx) in entries" :key="idx">
      <li v-if="entry.separator" class="sep" role="separator" />
      <li
        v-else
        class="item"
        :class="{
          disabled: !!entry.disabled,
          'has-sub': !!(entry.children && entry.children.length),
        }"
        role="menuitem"
        @click.stop.prevent="onItemClick(entry)"
      >
        <span class="ico">
          <img v-if="entry.icon" :src="entry.icon" alt="" />
        </span>
        <span class="lbl">{{ entry.label || "" }}</span>
        <span v-if="entry.children?.length" class="arrow">›</span>
        <ShellMenuEntries
          v-if="entry.children?.length"
          nested
          :entries="entry.children"
          @command="emit('command', $event)"
        />
      </li>
    </template>
  </template>
</template>
