<script setup lang="ts">
import { ref } from "vue";
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

const menu = ref<{ item: WallpaperItem; x: number; y: number } | null>(null);

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

    <!-- 右键菜单 -->
    <div
      v-if="menu"
      class="fixed z-50 min-w-[140px] overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--bg-elevated)] py-1 text-[13px] shadow-lg"
      :style="{ left: menu.x + 'px', top: menu.y + 'px' }"
      @mouseleave="menu = null"
    >
      <button class="ctx-item" @click="onSet(menu.item); menu = null">设为壁纸</button>
      <button class="ctx-item" @click="menu.item.favorite = !menu.item.favorite; menu = null">收藏</button>
      <button class="ctx-item" @click="menu = null">分享</button>
      <button class="ctx-item" @click="onSelect(menu.item); menu = null">详情</button>
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
