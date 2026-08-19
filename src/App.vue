<script setup lang="ts">
import { provide, ref, watch } from "vue";
import WinBar from "./components/WinBar.vue";
import IconRail from "./components/IconRail.vue";
import WallpaperGrid from "./components/WallpaperGrid.vue";
import DetailDrawer from "./components/DetailDrawer.vue";
import PlaybackBar from "./components/PlaybackBar.vue";
import SettingsView from "./components/SettingsView.vue";
import FavoritesView from "./components/FavoritesView.vue";
import Toast from "./components/Toast.vue";
import { showToast } from "./composables/useToast";
import { CATALOG, type WallpaperItem } from "./data/catalog";

const drawerItem = ref<WallpaperItem | null>(CATALOG[0] ?? null);
const drawerOpen = ref(true);
const selectedId = ref<number | null>(CATALOG[0]?.id ?? null);
const current = ref<WallpaperItem | null>(CATALOG[0] ?? null);
const theme = ref<"light" | "dark">("light");

const search = ref("");
const sort = ref("最热");
provide("topbarSearch", search);
provide("topbarSort", sort);

const activeNav = ref("online");

function onSelect(item: WallpaperItem) {
  selectedId.value = item.id;
  drawerItem.value = item;
  drawerOpen.value = true;
  current.value = item;
}
function onSet(item: WallpaperItem) {
  current.value = item;
  selectedId.value = item.id;
  showToast(`壁纸「${item.name}」已成功应用到桌面`);
}
function onFavorite(item: WallpaperItem) {
  item.favorite = !item.favorite;
  showToast(item.favorite ? "已收藏" : "已取消收藏");
}
function onNav(key: string) {
  activeNav.value = key;
  if (key !== "online" && key !== "favorite") drawerOpen.value = false;
  else if (drawerItem.value) drawerOpen.value = true;
}
function toggleTheme() {
  theme.value = theme.value === "light" ? "dark" : "light";
}

watch(theme, (v) => document.documentElement.setAttribute("data-theme", v), { immediate: true });
</script>

<template>
  <div class="window">
    <WinBar @theme="toggleTheme" />

    <div class="app-body">
      <IconRail :active="activeNav" @nav="onNav" />

      <WallpaperGrid
        v-if="activeNav === 'online'"
        :selected-id="selectedId"
        @select="onSelect"
        @set="onSet"
      />
      <FavoritesView
        v-else-if="activeNav === 'favorite'"
        :selected-id="selectedId"
        @select="onSelect"
        @set="onSet"
      />
      <SettingsView v-else-if="activeNav === 'settings'" />
      <div v-else class="main">
        <div class="placeholder">
          <h3>该页面将在 Phase 2 / 3 接入</h3>
          <p>当前已交付：在线壁纸库、我的收藏、设置。</p>
        </div>
      </div>

      <DetailDrawer
        :item="drawerItem"
        :open="drawerOpen && (activeNav === 'online' || activeNav === 'favorite')"
        @close="drawerOpen = false"
        @set="onSet"
        @favorite="onFavorite"
      />
    </div>

    <PlaybackBar :current="current" @import="() => {}" />
  </div>

  <Toast />
</template>
