<script setup lang="ts">
import { nextTick, provide, ref, useTemplateRef } from "vue";
import IconRail from "./components/IconRail.vue";
import TopBar from "./components/TopBar.vue";
import WallpaperGrid from "./components/WallpaperGrid.vue";
import DetailDrawer from "./components/DetailDrawer.vue";
import PlaybackBar from "./components/PlaybackBar.vue";
import type { WallpaperItem } from "./data/catalog";

const drawerItem = ref<WallpaperItem | null>(null);
const drawerOpen = ref(false);
const selected = ref<WallpaperItem | null>(null);
const current = ref<WallpaperItem | null>(null);
const theme = ref<"frost-light" | "frost-dark">("frost-light");

// Top-bar search drives the grid filter (A4). Lifted to App so TopBar can
// write and WallpaperGrid can read without prop-drilling.
const search = ref("");
provide("topbarSearch", search);

const menu = ref<{ item: WallpaperItem; x: number; y: number } | null>(null);
// Keyboard cursor for the right-click menu (A8). Wraps so ↑↓ stays inside bounds.
const menuFocus = ref(0);
const menuEl = useTemplateRef<HTMLElement>("menuEl");

const MENU_ACTIONS = ["set", "favorite", "share", "details"] as const;

function onSelect(item: WallpaperItem) {
  selected.value = item;
  drawerItem.value = item;
  drawerOpen.value = true;
}
function onSet(item: WallpaperItem) {
  current.value = item;
  drawerOpen.value = false;
}
function onContext(item: WallpaperItem, x: number, y: number) {
  menu.value = { item, x, y };
  menuFocus.value = 0;
  nextTick(() => {
    // Auto-focus the first item so ↑↓ / Enter / Esc work without an extra Tab.
    const first = menuEl.value?.querySelector<HTMLButtonElement>(".ctx-item");
    first?.focus();
  });
}
function closeMenu() {
  menu.value = null;
}
function onMenuKey(e: KeyboardEvent) {
  if (!menu.value) return;
  if (e.key === "Escape") {
    e.preventDefault();
    closeMenu();
    return;
  }
  if (e.key === "ArrowDown" || e.key === "ArrowUp") {
    e.preventDefault();
    const dir = e.key === "ArrowDown" ? 1 : -1;
    menuFocus.value = (menuFocus.value + dir + MENU_ACTIONS.length) % MENU_ACTIONS.length;
    const items = menuEl.value?.querySelectorAll<HTMLButtonElement>(".ctx-item");
    items?.[menuFocus.value]?.focus();
  }
}
function runMenuAction(action: (typeof MENU_ACTIONS)[number]) {
  if (!menu.value) return;
  const it = menu.value.item;
  switch (action) {
    case "set":
      onSet(it);
      break;
    case "favorite":
      it.favorite = !it.favorite;
      break;
    case "share":
      // Phase 2: integrate share channel; Phase 1 no-op.
      break;
    case "details":
      onSelect(it);
      break;
  }
  closeMenu();
}
function toggleTheme() {
  theme.value = theme.value === "frost-light" ? "frost-dark" : "frost-light";
  document.documentElement.setAttribute("data-theme", theme.value);
}
function importLocal() {
  // Phase 2 接入真实导入；Phase 1 占位
}
</script>

<template>
  <div
    class="flex h-full flex-col overflow-hidden rounded-2xl border border-[var(--border)] bg-[var(--bg)] shadow-[0_24px_64px_rgba(0,0,0,0.12)] animate-fade-in"
  >
    <div class="flex min-h-0 flex-1">
      <IconRail @nav="() => {}" />
      <div class="flex min-w-0 flex-1 flex-col">
        <TopBar @theme="toggleTheme" />
        <WallpaperGrid
          @select="onSelect"
          @set="onSet"
          @context="onContext"
        />
      </div>
      <DetailDrawer
        :item="drawerItem"
        :open="drawerOpen"
        @close="drawerOpen = false"
        @set="onSet"
        @favorite="(i) => (i.favorite = !i.favorite)"
      />
    </div>
    <PlaybackBar :current="current" @import="importLocal" />

    <!-- 右键菜单（键盘可达：A8） -->
    <div
      v-if="menu"
      ref="menuEl"
      role="menu"
      tabindex="-1"
      class="fixed z-50 min-w-[140px] overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--bg-elevated)] py-1 text-[13px] shadow-lg"
      :style="{ left: menu.x + 'px', top: menu.y + 'px' }"
      @mouseleave="closeMenu"
      @keydown="onMenuKey"
    >
      <button class="ctx-item" role="menuitem" @click="runMenuAction('set')">设为壁纸</button>
      <button class="ctx-item" role="menuitem" @click="runMenuAction('favorite')">收藏</button>
      <button class="ctx-item" role="menuitem" @click="runMenuAction('share')">分享</button>
      <button class="ctx-item" role="menuitem" @click="runMenuAction('details')">详情</button>
    </div>
  </div>
</template>

<style scoped>
.ctx-item {
  display: block;
  width: 100%;
  padding: 7px 14px;
  text-align: left;
  color: var(--text);
  transition: background 0.12s;
}
.ctx-item:hover {
  background: rgba(31, 35, 41, 0.06);
}
</style>
