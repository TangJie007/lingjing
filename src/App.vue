<script setup lang="ts">
import { computed, onMounted, onUnmounted, provide, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWindow } from "@tauri-apps/api/window";
import WinBar from "./components/WinBar.vue";
import IconRail from "./components/IconRail.vue";
import DetailDrawer from "./components/DetailDrawer.vue";
import Toast from "./components/Toast.vue";
import LoginModal from "./components/LoginModal.vue";
import FirstRunAutostartModal from "./components/FirstRunAutostartModal.vue";
import { showToast } from "./composables/useToast";
import { soundOn } from "./composables/useAudio";
import { CATALOG, type WallpaperItem } from "./data/catalog";
import {
  enginePause,
  enginePlay,
  importMedia,
  listLibrary,
  loadFavoriteIds,
  onEngineState,
  onPauseRecommend,
  removeLibraryItem,
  setFavoriteRemote,
  exportWallpaper,
  setWallpaper,
  type EngineState,
  type PauseRecommendPayload,
} from "./composables/useEngine";
import { forgetVideoPoster, videoPosterKey } from "./composables/useVideoPoster";
import { loadSettings, useSettings, hasVersionRecord, completeFirstRun } from "./composables/useSettings";
import { useAuth } from "./composables/useAuth";
import { fetchOnlineCategories, fetchOnlineWallpapers, type OnlineCategory } from "./composables/useLingjingApi";

const route = useRoute();
const router = useRouter();

const drawerItem = ref<WallpaperItem | null>(null);
const drawerOpen = ref(false);
const selectedId = ref<string | null>(null);
const current = ref<WallpaperItem | null>(null);
const localItems = ref<WallpaperItem[]>([]);
const engine = ref<EngineState | null>(null);
type LoopMode = "list" | "single" | "random";
const loopMode = ref<LoopMode>("single");

const search = ref("");
const sort = ref("最热");
provide("topbarSearch", search);
provide("topbarSort", sort);

const loginOpen = ref(false);
const firstRunOpen = ref(false);
const firstRunSaving = ref(false);

const settings = useSettings();
const { isLoggedIn, refreshMe, user } = useAuth();

const userLabel = computed(
  () => user.value?.nickname || user.value?.username || user.value?.email || "已登录",
);

const onlineItems = ref<WallpaperItem[]>([]);
const onlineCategories = ref<OnlineCategory[]>([]);
const onlineCategoryId = ref<number | null>(null);
const onlineLoading = ref(false);
const onlineFetchError = ref("");

const onlineGridItems = computed(() => {
  if (!settings.value.onlineEnabled) {
    return CATALOG.filter((i) => !!i.mediaSrc);
  }
  return onlineItems.value;
});

const onlineGridLoading = computed(
  () => settings.value.onlineEnabled && onlineLoading.value,
);

const onlineEmptyText = computed(() => {
  if (!settings.value.onlineEnabled) return "没有匹配的壁纸";
  if (onlineFetchError.value) return onlineFetchError.value;
  return "暂无在线壁纸，请确认 API 服务已启动";
});

const drawerVisible = computed(
  () => drawerOpen.value && route.meta.showDrawer === true,
);

const playQueue = computed(() => {
  const locals = localItems.value;
  const samples = settings.value.onlineEnabled
    ? onlineItems.value.filter((i) => !!i.mediaSrc)
    : CATALOG.filter((i) => !!i.mediaSrc);
  if (route.name === "local") return locals.length ? locals : samples;
  return [...samples, ...locals];
});

const favoriteItems = computed(() => {
  const out: WallpaperItem[] = [];
  if (settings.value.onlineEnabled) {
    for (const i of onlineItems.value) if (i.favorite) out.push(i);
  } else {
    for (const i of CATALOG) if (i.favorite) out.push(i);
  }
  for (const i of localItems.value) if (i.favorite) out.push(i);
  return out;
});

const routeViewProps = computed(() => {
  switch (route.name) {
    case "online":
      return {
        selectedId: selectedId.value,
        items: onlineGridItems.value,
        favorites: favoriteItems.value,
        loading: onlineGridLoading.value,
        emptyText: onlineEmptyText.value,
        onlineEnabled: settings.value.onlineEnabled,
        categories: onlineCategories.value,
        categoryId: onlineCategoryId.value,
      };
    case "local":
      return {
        selectedId: selectedId.value,
        items: localItems.value,
      };
    default:
      return {};
  }
});

function allWallpapers(): WallpaperItem[] {
  const samples = settings.value.onlineEnabled
    ? onlineItems.value.filter((i) => !!i.mediaSrc)
    : CATALOG.filter((i) => !!i.mediaSrc);
  return [...samples, ...localItems.value];
}

function findWallpaper(id: string): WallpaperItem | null {
  return allWallpapers().find((i) => i.id === id) ?? null;
}

provide("findWallpaper", findWallpaper);

function syncRouteSideEffects(name: typeof route.name) {
  if (name === "online") void refreshOnline();
  const keepDrawer = name === "online" || name === "local";
  if (!keepDrawer) drawerOpen.value = false;
}

function syncLoopModeFromSettings() {
  const m = settings.value.loopMode;
  if (m === "list" || m === "single" || m === "random") {
    loopMode.value = m;
  }
}

watch(
  () => settings.value.loopMode,
  (m) => {
    if (m === "list" || m === "single" || m === "random") {
      loopMode.value = m;
    }
  },
);

watch(loopMode, (m) => {
  if (settings.value.loopMode !== m) {
    settings.value.loopMode = m;
  }
});

watch(
  () => settings.value.soundOn,
  (v) => {
    soundOn.value = v;
  },
  { immediate: true },
);

watch(
  () => route.name,
  (name) => {
    syncRouteSideEffects(name);
  },
  { immediate: true },
);

async function applyFavorites(ids: string[]) {
  const set = new Set(ids);
  if (settings.value.onlineEnabled) {
    for (const i of onlineItems.value) i.favorite = set.has(String(i.id));
  } else {
    for (const i of CATALOG) i.favorite = set.has(String(i.id));
  }
  for (const i of localItems.value) i.favorite = set.has(String(i.id));
}

async function refreshOnline() {
  if (route.name !== "online" || !settings.value.onlineEnabled) {
    if (!settings.value.onlineEnabled) {
      onlineItems.value = [];
      onlineCategories.value = [];
      onlineCategoryId.value = null;
      onlineFetchError.value = "";
    }
    return;
  }
  onlineLoading.value = true;
  onlineFetchError.value = "";
  try {
    const [cats, wallpaperResult] = await Promise.all([
      fetchOnlineCategories().catch(() => [] as OnlineCategory[]),
      fetchOnlineWallpapers({
        pageSize: 48,
        categoryId:
          onlineCategoryId.value != null ? String(onlineCategoryId.value) : "",
      }),
    ]);
    onlineCategories.value = cats;
    onlineItems.value = wallpaperResult.items;
    const { ids } = await loadFavoriteIds();
    for (const i of onlineItems.value) {
      i.favorite = ids.includes(String(i.id));
    }
  } catch (e) {
    onlineItems.value = [];
    const msg = e instanceof Error ? e.message : String(e);
    onlineFetchError.value = msg;
    if (route.name === "online") showToast(msg);
  } finally {
    onlineLoading.value = false;
  }
}

async function onOnlineCategoryChange(categoryId: number | null) {
  if (onlineCategoryId.value === categoryId) return;
  onlineCategoryId.value = categoryId;
  if (route.name !== "online" || !settings.value.onlineEnabled) return;
  onlineLoading.value = true;
  onlineFetchError.value = "";
  try {
    const { items } = await fetchOnlineWallpapers({
      pageSize: 48,
      categoryId: categoryId != null ? String(categoryId) : "",
    });
    onlineItems.value = items;
    const { ids } = await loadFavoriteIds();
    for (const i of onlineItems.value) {
      i.favorite = ids.includes(String(i.id));
    }
  } catch (e) {
    onlineItems.value = [];
    const msg = e instanceof Error ? e.message : String(e);
    onlineFetchError.value = msg;
    showToast(msg);
  } finally {
    onlineLoading.value = false;
  }
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
}

function openWallpaperDetail(item: WallpaperItem) {
  selectedId.value = item.id;
  drawerItem.value = item;
  drawerOpen.value = false;
  void router.push({ name: "wallpaper-detail", params: { id: item.id } });
}

function onPreview(item: WallpaperItem) {
  openWallpaperDetail(item);
}

function onOpenDetail(item: WallpaperItem) {
  openWallpaperDetail(item);
}

async function onSet(item: WallpaperItem) {
  if (item.missing) {
    showToast("源文件已缺失，请重新导入或删除该项");
    return;
  }
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
    const msg = e instanceof Error ? e.message : String(e);
    showToast(`设壁纸失败：${msg}`);
  }
}

async function onDownload(item: WallpaperItem) {
  if (!item.mediaSrc) {
    showToast("该资源无法下载");
    return;
  }
  const raw = item.mediaSrc.split("?")[0] ?? item.mediaSrc;
  const ext = raw.includes(".") ? raw.split(".").pop() ?? "bin" : "bin";
  const safeName = item.name.replace(/[\\/:*?"<>|]/g, "_");
  try {
    const saved = await exportWallpaper(item.mediaSrc, `${safeName}.${ext}`);
    if (saved) showToast(`已保存：${saved}`);
    else showToast("已取消保存");
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

async function onRemoveLocal(item: WallpaperItem) {
  try {
    const wasCurrent = current.value?.id === item.id || engine.value?.mediaId === item.id;
    const newState = await removeLibraryItem(item.id);
    engine.value = newState;
    forgetVideoPoster(videoPosterKey(item));
    localItems.value = localItems.value.filter((i) => i.id !== item.id);
    const { ids } = await loadFavoriteIds();
    await applyFavorites(ids);
    if (wasCurrent) {
      current.value = localItems.value[0] ?? CATALOG[0] ?? null;
    } else if (current.value?.id === item.id) {
      current.value = localItems.value[0] ?? CATALOG[0] ?? null;
    }
    showToast("删除成功");
  } catch (e) {
    showToast(e instanceof Error ? e.message : String(e));
  }
}

function openLogin() {
  loginOpen.value = true;
}

function onLoginSuccess() {
  loginOpen.value = false;
  if (route.name === "online") void refreshOnline();
}

async function runImport(paths?: string[] | null) {
  let selected = paths ?? null;
  if (!selected) {
    const picked = await open({
      multiple: true,
      filters: [
        {
          name: "Video",
          extensions: ["mp4", "webm"],
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
      showToast("导入成功");
      await router.push({ name: "local" });
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
    if (!p) continue;
    if (!/\.(mp4|webm)$/i.test(p)) continue;
    paths.push(p);
  }
  if (!paths.length) {
    showToast("仅支持导入 mp4 / webm 视频");
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
  const nextIdx = (idx + dir + q.length) % q.length;
  return q[nextIdx]!;
}

async function onNext() {
  const item = pickNext(1);
  if (item) await onSet(item);
}

let unlisten: (() => void) | undefined;
let unlistenPause: (() => void) | undefined;
let unlistenNavigateSettings: (() => void) | undefined;
let lastEngineErrorToast = "";

function isBenignEngineError(msg: string) {
  const m = msg.toLowerCase();
  return m.includes("aborterror") || m.includes("interrupted by a new load");
}

function toastEngineError(msg: string) {
  if (!msg || isBenignEngineError(msg) || msg === lastEngineErrorToast) return;
  lastEngineErrorToast = msg;
  showToast(msg);
}
let lastRecommend = { reason: "", at: 0 };
let lastUserAction = 0;

async function applyPauseRecommend(p: PauseRecommendPayload) {
  const now = Date.now();
  if (p.reason === "resume") {
    // Wallpaper recovery is handled in Rust; avoid toast spam.
    lastRecommend = { reason: p.reason, at: now };
    return;
  }
  if (p.reason === lastRecommend.reason && now - lastRecommend.at < 1500) return;
  lastRecommend = { reason: p.reason, at: now };
  if (p.action === "pause") {
    if (now - lastUserAction < 1200) return;
    try {
      engine.value = await enginePause(false);
      const label =
        p.reason === "fullscreen"
          ? "全屏应用"
          : p.reason === "battery"
            ? "电池模式"
            : p.reason === "rdp"
              ? "远程桌面"
              : "自动";
      showToast(`已自动暂停：${label}`);
    } catch {
      /* ignore */
    }
  } else if (p.action === "play") {
    if (!engine.value?.mediaId) return;
    if (engine.value?.userPaused) return;
    try {
      engine.value = await enginePlay();
      if (p.reason !== "settings") showToast("已自动恢复播放");
    } catch {
      /* ignore */
    }
  }
}

watch(
  () => [settings.value.onlineEnabled, settings.value.apiBaseUrl, isLoggedIn.value] as const,
  ([enabled]) => {
    if (enabled && route.name === "online") void refreshOnline();
  },
);

onMounted(async () => {
  document.documentElement.setAttribute("data-theme", "light");
  await loadSettings();
  try {
    const seen = await hasVersionRecord();
    if (!seen) firstRunOpen.value = true;
  } catch {
    firstRunOpen.value = true;
  }
  syncLoopModeFromSettings();
  await refreshMe().catch(() => undefined);
  await refreshLibrary();
  unlisten = await onEngineState((s) => {
    engine.value = s;
    if (s.error) toastEngineError(s.error);
    else lastEngineErrorToast = "";
    // Single loop is handled by <video loop> in wallpaper.html.
    // Do NOT call onSet here — that reloads the whole engine and spams toast.
    if (
      s.duration > 0 &&
      s.currentTime >= s.duration - 0.35 &&
      !s.playing &&
      current.value &&
      loopMode.value !== "single"
    ) {
      void onNext();
    }
  });
  unlistenPause = await onPauseRecommend(applyPauseRecommend);

  try {
    const { listen } = await import("@tauri-apps/api/event");
    unlistenNavigateSettings = await listen("navigate-settings", () => {
      void router.push({ name: "settings" });
    });
  } catch (e) {
    console.warn("listen navigate-settings failed", e);
  }

  // Optional: open hidden on launch
  try {
    const args = (await getCurrentWindow().listen("tauri://launched", () => {})) as unknown;
    void args;
  } catch {
    /* ignore */
  }
});

onUnmounted(() => {
  unlisten?.();
  unlistenPause?.();
  unlistenNavigateSettings?.();
});

async function onFirstRunConfirm(autostart: boolean) {
  if (firstRunSaving.value) return;
  firstRunSaving.value = true;
  try {
    await completeFirstRun(autostart);
    settings.value.autostart = autostart;
    firstRunOpen.value = false;
  } catch (e) {
    console.warn("complete_first_run failed", e);
    showToast("保存首次设置失败，请稍后重试");
  } finally {
    firstRunSaving.value = false;
  }
}
</script>

<template>
  <div
    class="window"
    @dragover.prevent
    @drop="onDrop"
  >
    <WinBar
      :online-enabled="settings.onlineEnabled"
      :logged-in="isLoggedIn"
      :user-label="userLabel"
      @login="openLogin"
    />

    <div class="app-body">
      <IconRail />

      <RouterView v-slot="{ Component }">
        <component
          :is="Component"
          v-bind="routeViewProps"
          @select="onSelect"
          @preview="onPreview"
          @set="onSet"
          @favorite="onFavorite"
          @download="onDownload"
          @import="runImport()"
          @remove="onRemoveLocal"
          @category-change="onOnlineCategoryChange"
        />
      </RouterView>

      <DetailDrawer
        :item="drawerItem"
        :open="drawerVisible"
        @close="drawerOpen = false"
        @open-detail="onOpenDetail"
        @set="onSet"
        @favorite="onFavorite"
        @download="onDownload"
      />
    </div>
  </div>

  <Toast />
  <FirstRunAutostartModal :open="firstRunOpen" :saving="firstRunSaving" @confirm="onFirstRunConfirm" />
  <LoginModal :open="loginOpen" @close="loginOpen = false" @success="onLoginSuccess" />
</template>
