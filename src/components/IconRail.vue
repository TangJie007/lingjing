<script setup lang="ts">
import { ref } from "vue";

interface NavItem {
  key: string;
  label: string;
  icon: string;
}

const items: NavItem[] = [
  { key: "online", label: "在线", icon: "🏠" },
  { key: "local", label: "本地", icon: "💾" },
  { key: "favorite", label: "收藏", icon: "❤️" },
  { key: "settings", label: "设置", icon: "⚙️" },
  { key: "about", label: "关于", icon: "ℹ️" },
];

const active = ref("online");
const emit = defineEmits<{ (e: "nav", key: string): void }>();

function select(key: string) {
  active.value = key;
  emit("nav", key);
}
</script>

<template>
  <aside
    class="flex w-20 flex-shrink-0 flex-col items-center gap-1 border-r border-[var(--border)] bg-[var(--bg-elevated)] py-3"
  >
    <div class="mb-3 flex h-9 w-9 items-center justify-center rounded-xl bg-[var(--primary-soft)] text-lg">
      🌌
    </div>
    <button
      v-for="it in items"
      :key="it.key"
      class="rail-btn group relative flex w-14 flex-col items-center gap-1 rounded-xl py-2 transition-colors duration-[var(--dur-fast)]"
      :class="
        active === it.key
          ? 'bg-[var(--primary-soft)] text-[var(--primary)]'
          : 'text-[var(--text-dim)] hover:bg-black/5'
      "
      @click="select(it.key)"
    >
      <span
        class="text-xl leading-none transition-transform duration-[var(--dur-fast)]"
        :class="active === it.key ? 'icon-spring' : 'group-hover:scale-110'"
        >{{ it.icon }}</span
      >
      <span class="text-[10px] font-medium">{{ it.label }}</span>
      <span
        v-if="active === it.key"
        class="absolute left-0 top-1/2 h-6 w-1 -translate-y-1/2 rounded-r-full bg-[var(--primary)]"
      />
    </button>
  </aside>
</template>

<style scoped>
.rail-btn {
  app-region: no-drag;
}
</style>
