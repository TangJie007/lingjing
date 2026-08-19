<script setup lang="ts">
import { provide, ref, watch } from "vue";
import IconRail from "./components/IconRail.vue";
import TopBar from "./components/TopBar.vue";
import WallpaperGrid from "./components/WallpaperGrid.vue";
import DetailDrawer from "./components/DetailDrawer.vue";
import PlaybackBar from "./components/PlaybackBar.vue";
import SettingsView from "./components/SettingsView.vue";
import FavoritesView from "./components/FavoritesView.vue";
import Toast from "./components/Toast.vue";
import { showToast } from "./composables/useToast";
import type { WallpaperItem } from "./data/catalog";

const drawerItem = ref<WallpaperItem | null>(null);
const drawerOpen = ref(false);
const selectedId = ref<number | null>(null);
const current = ref<WallpaperItem | null>(null);
const theme = ref<"light" | "dark">("light");

// 共享搜索词 + 排序
const search = ref("");
provide("topbarSearch", search);

const activeNav = ref<string>("online");

// 右键菜单（保留 Phase 1 行为，作为可达性备选）
const menu = ref<{ item: WallpaperItem; x: number; y: number } | null>(null);
const menuFocus = ref(0);
const menuEl = ref<HTMLElement | null>(null);

const MENU_ACTIONS = ["set", "favorite", "share", "details"] as const;

function onSelect(item: WallpaperItem) {
  selectedId.value = item.id;
  drawerItem.value = item;
  drawerOpen.value = true;
}
function onSet(item: WallpaperItem) {
  current.value = item;
  drawerOpen.value = false;
  showToast(`壁纸「${item.name}」已成功应用到桌面`);
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
      onFavorite(it);
      break;
    case "share":
      break;
    case "details":
      onSelect(it);
      break;
  }
  closeMenu();
}
function onFavorite(item: WallpaperItem) {
  item.favorite = !item.favorite;
  showToast(item.favorite ? "已收藏" : "已取消收藏");
}
function onNav(key: string) {
  activeNav.value = key;
}
function toggleTheme() {
  theme.value = theme.value === "light" ? "dark" : "light";
  document.documentElement.setAttribute("data-theme", theme.value);
}
function importLocal() {
  /* 触发由 PlaybackBar 内部 toast 完成 */
}

// 初始化主题
watch(
  theme,
  (v) => {
    document.documentElement.setAttribute("data-theme", v);
  },
  { immediate: true },
);
</script>

<template>
  <div
    class="flex h-full flex-col overflow-hidden rounded-2xl border border-[var(--border)] bg-[var(--bg)] shadow-[var(--sh-win)] animate-fade-in"
  >
    <div class="flex min-h-0 flex-1">
      <IconRail :active="activeNav" @nav="onNav" />
      <div class="flex min-w-0 flex-1 flex-col">
        <TopBar @theme="toggleTheme" />
        <main class="flex min-h-0 flex-1 flex-col">
          <WallpaperGrid
            v-if="activeNav === 'online'"
            :selected-id="selectedId"
            @select="onSelect"
            @set="onSet"
            @favorite="onFavorite"
          />
          <FavoritesView
            v-else-if="activeNav === 'favorite'"
            :selected-id="selectedId"
            @select="onSelect"
            @set="onSet"
            @favorite="onFavorite"
          />
          <SettingsView v-else-if="activeNav === 'settings'" />
          <div v-else class="placeholder">
            <h3>该页面将在 Phase 2 / 3 接入</h3>
            <p>当前变更（ui-redesign）已交付：在线、我的收藏、设置。</p>
          </div>
        </main>
      </div>
      <DetailDrawer
        :item="drawerItem"
        :open="drawerOpen"
        @close="drawerOpen = false"
        @set="onSet"
        @favorite="onFavorite"
      />
    </div>
    <PlaybackBar :current="current" @import="importLocal" />

    <!-- 右键菜单（键盘可达） -->
    <div
      v-if="menu"
      ref="menuEl"
      role="menu"
      tabindex="-1"
      class="fixed z-50 min-w-[140px] overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--surface)] py-1 text-[13px] shadow-lg"
      :style="{ left: menu.x + 'px', top: menu.y + 'px' }"
      @mouseleave="closeMenu"
      @keydown="onMenuKey"
    >
      <button class="ctx-item" role="menuitem" @click="runMenuAction('set')">设为壁纸</button>
      <button class="ctx-item" role="menuitem" @click="runMenuAction('favorite')">
        {{ menu.item.favorite ? "取消收藏" : "收藏" }}
      </button>
      <button class="ctx-item" role="menuitem" @click="runMenuAction('share')">分享</button>
      <button class="ctx-item" role="menuitem" @click="runMenuAction('details')">详情</button>
    </div>
  </div>

  <Toast />
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

.placeholder {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--text-3);
  padding: 32px;
  text-align: center;
}
.placeholder h3 {
  font-size: 18px;
  font-weight: 700;
  margin-bottom: 8px;
}
.placeholder p {
  font-size: 13px;
  max-width: 380px;
  line-height: 1.6;
}
</style>
