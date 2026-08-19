<script setup lang="ts">
import { computed, onMounted, onUnmounted, provide, ref, watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import WinBar from "./components/WinBar.vue";
import IconRail from "./components/IconRail.vue";
import WallpaperGrid from "./components/WallpaperGrid.vue";
import DetailDrawer from "./components/DetailDrawer.vue";
import PlaybackBar, { type LoopMode } from "./components/PlaybackBar.vue";
import SettingsView from "./components/SettingsView.vue";
import FavoritesView from "./components/FavoritesView.vue";
import LocalLibraryView from "./components/LocalLibraryView.vue";
import Toast from "./components/Toast.vue";
import { showToast } from "./composables/useToast";
import { CATALOG, type WallpaperItem } from "./data/catalog";
import {
  enginePause,
  enginePlay,
  engineSetVolume,
  importMedia,
  listLibrary,
  loadFavoriteIds,
  onEngineState,
  setFavoriteRemote,
  setWallpaper,
  type EngineState,
} from "./composables/useEngine";

const drawerItem = ref<WallpaperItem | null>(CATALOG[0] ?? null);
const drawerOpen = ref(true);
const selectedId = ref<string | null>(CATALOG[0]?.id ?? null);
const current = ref<WallpaperItem | null>(CATALOG[0] ?? null);
const theme = ref<"light" | "dark">("light");
const localItems = ref<WallpaperItem[]>([]);
const engine = ref<EngineState | null>(null);
const loopMode = ref<LoopMode>("list");

const search = ref("");
const sort = ref("最热");
provide("topbarSearch", search);
provide("topbarSort", sort);

const activeNav = ref("online");

const playQueue = computed(() => {
  const locals = localItems.value;
  const samples = CATALOG.filter((i) => !!i.mediaSrc);
  // Prefer current context: local page → local only; else samples + local
  if (activeNav.value === "local") return locals.length ? locals : samples;
  return [...samples, ...locals];
});

const favoriteItems = computed(() => {
  const out: WallpaperItem[] = [];
  for (const i of CATALOG) if (i.favorite) out.push(i);
  for (const i of localItems.value) if (i.favorite) out.push(i);
  return out;
});

async function applyFavorites(ids: string[]) {
  const set = new Set(ids);
  for (const i of CATALOG) i.favorite = set.has(String(i.id));
  for (const i of localItems.value) i.favorite = set.has(String(i.id));
}

async function refreshLibrary() {
  try {
    localItems.value = await listLibrary();
    const { ids, isNew } = await loadFavoriteIds();
    if (isNew) {
      const seeds = CATALOG.filter((i) => i.favorite).map((i) => String(i.id));
      for (const id of seeds) {
        await setFavoriteRemote(id, true);
      }
      await applyFavorites(seeds);
    } else {
      await applyFavorites(ids);
    }
  } catch (e) {
    console.warn(e);
  }
}

function onSelect(item: WallpaperItem) {
  selectedId.value = item.id;
  drawerItem.value = item;
  drawerOpen.value = true;
  current.value = item;
}

async function onSet(item: WallpaperItem) {
  if (!item.mediaSrc) {
    showToast("该资源暂无可用媒体");
    return;
  }
  try {
    const state = await setWallpaper(item);
    engine.value = state;
    current.value = item;
    selectedId.value = item.id;
    showToast(`壁纸「${item.name}」已成功应用到桌面`);
  } catch (e) {
    showToast(e instanceof Error ? e.message : String(e));
  }
}

async function onFavorite(item: WallpaperItem) {
  const next = !item.favorite;
  item.favorite = next;
  try {
    const ids = await setFavoriteRemote(String(item.id), next);
    await applyFavorites(ids);
    showToast(next ? "已收藏" : "已取消收藏");
  } catch (e) {
    item.favorite = !next;
    showToast(e instanceof Error ? e.message : String(e));
  }
}

function onNav(key: string) {
  activeNav.value = key;
  if (key === "online" || key === "favorite" || key === "local") {
    if (drawerItem.value) drawerOpen.value = true;
  } else {
    drawerOpen.value = false;
  }
}

function toggleTheme() {
  theme.value = theme.value === "light" ? "dark" : "light";
}

async function runImport(paths?: string[] | null) {
  let selected = paths ?? null;
  if (!selected) {
    const picked = await open({
      multiple: true,
      filters: [
        {
          name: "Wallpaper",
          extensions: ["mp4", "webm", "gif", "webp", "jpg", "jpeg", "png"],
        },
      ],
    });
    if (!picked) return;
    selected = Array.isArray(picked) ? picked : [picked];
  }
  if (!selected.length) return;
  try {
    const { items: imported, errors } = await importMedia(selected);
    await refreshLibrary();
    if (imported.length) {
      showToast(`已导入 ${imported.length} 个文件`);
      activeNav.value = "local";
    } else {
      showToast("没有成功导入的文件");
    }
    if (errors.length) {
      showToast(`部分失败：${errors.slice(0, 2).join("；")}${errors.length > 2 ? "…" : ""}`);
    }
  } catch (e) {
    showToast(e instanceof Error ? e.message : String(e));
  }
}

function onDrop(e: DragEvent) {
  e.preventDefault();
  const files = e.dataTransfer?.files;
  if (!files?.length) return;
  const paths: string[] = [];
  for (const f of Array.from(files)) {
    const p = (f as File & { path?: string }).path;
    if (p) paths.push(p);
  }
  if (!paths.length) {
    showToast("请使用导入按钮选择本地文件");
    return;
  }
  void runImport(paths);
}

function pickNext(dir: 1 | -1) {
  const q = playQueue.value;
  if (!q.length) return null;
  const curId = current.value?.id;
  let idx = q.findIndex((i) => i.id === curId);
  if (idx < 0) idx = 0;
  if (loopMode.value === "random") {
    if (q.length === 1) return q[0]!;
    let next = idx;
    while (next === idx) next = Math.floor(Math.random() * q.length);
    return q[next]!;
  }
  if (loopMode.value === "single" && dir === 1 && current.value) {
    // next button still advances; ended-media handled separately
  }
  const nextIdx = (idx + dir + q.length) % q.length;
  return q[nextIdx]!;
}

async function onPrev() {
  const item = pickNext(-1);
  if (item) await onSet(item);
}
async function onNext() {
  const item = pickNext(1);
  if (item) await onSet(item);
}

async function onPlay() {
  try {
    engine.value = await enginePlay();
  } catch (e) {
    showToast(e instanceof Error ? e.message : String(e));
  }
}
async function onPause() {
  try {
    engine.value = await enginePause();
  } catch (e) {
    showToast(e instanceof Error ? e.message : String(e));
  }
}
async function onVolume(v: number, muted: boolean) {
  try {
    engine.value = await engineSetVolume(v, muted);
  } catch {
    /* ignore for silent media */
  }
}

function onLoop(mode: LoopMode) {
  loopMode.value = mode;
}

function openDetail() {
  if (!current.value) return;
  drawerItem.value = current.value;
  drawerOpen.value = true;
  selectedId.value = current.value.id;
}

let unlisten: (() => void) | undefined;
let endedWatch: number | undefined;

onMounted(async () => {
  await refreshLibrary();
  unlisten = await onEngineState((s) => {
    engine.value = s;
    if (s.error) showToast(s.error);
    // auto-advance on natural end for list/random
    if (
      s.duration > 0 &&
      s.currentTime >= s.duration - 0.35 &&
      !s.playing &&
      current.value &&
      loopMode.value !== "single"
    ) {
      void onNext();
    } else if (
      s.duration > 0 &&
      s.currentTime >= s.duration - 0.35 &&
      loopMode.value === "single" &&
      current.value
    ) {
      void onSet(current.value);
    }
  });
});

onUnmounted(() => {
  unlisten?.();
  if (endedWatch) window.clearInterval(endedWatch);
});

watch(theme, (v) => document.documentElement.setAttribute("data-theme", v), { immediate: true });
</script>

<template>
  <div
    class="window"
    @dragover.prevent
    @drop="onDrop"
  >
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
        :items="favoriteItems"
        :selected-id="selectedId"
        @select="onSelect"
        @set="onSet"
      />
      <LocalLibraryView
        v-else-if="activeNav === 'local'"
        :items="localItems"
        :selected-id="selectedId"
        @select="onSelect"
        @set="onSet"
        @import="runImport()"
      />
      <SettingsView v-else-if="activeNav === 'settings'" />
      <div v-else class="main">
        <div class="placeholder">
          <h3>关于灵镜</h3>
          <p>动态壁纸客户端 · Phase 2 引擎已接入</p>
        </div>
      </div>

      <DetailDrawer
        :item="drawerItem"
        :open="drawerOpen && (activeNav === 'online' || activeNav === 'favorite' || activeNav === 'local')"
        @close="drawerOpen = false"
        @set="onSet"
        @favorite="onFavorite"
      />
    </div>

    <PlaybackBar
      :current="current"
      :engine="engine"
      :queue="playQueue"
      @import="runImport()"
      @open-detail="openDetail"
      @play="onPlay"
      @pause="onPause"
      @prev="onPrev"
      @next="onNext"
      @volume="onVolume"
      @loop="onLoop"
    />
  </div>

  <Toast />
</template>
