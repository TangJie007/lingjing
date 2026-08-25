<script setup lang="ts">
import { computed, onMounted, onUnmounted, provide, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWindow } from "@tauri-apps/api/window";
import WinBar from "./components/WinBar.vue";
import IconRail from "./components/IconRail.vue";
import DetailDrawer from "./components/DetailDrawer.vue";
import PlaybackBar, { type LoopMode } from "./components/PlaybackBar.vue";
import Toast from "./components/Toast.vue";
import LoginModal from "./components/LoginModal.vue";
import FirstRunAutostartModal from "./components/FirstRunAutostartModal.vue";
import { showToast } from "./composables/useToast";
import { soundOn } from "./composables/useAudio";
import { CATALOG, type WallpaperItem } from "./data/catalog";
import {
  enginePause,
  enginePlay,
  engineSetVolume,
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
import { loadSettings, useSettings, hasVersionRecord, completeFirstRun } from "./composables/useSettings";
import { useAuth } from "./composables/useAuth";
import { fetchOnlineWallpapers } from "./composables/useLingjingApi";

const route = useRoute();
const router = useRouter();

const drawerItem = ref<WallpaperItem | null>(CATALOG[0] ?? null);
const drawerOpen = ref(true);
const selectedId = ref<string | null>(CATALOG[0]?.id ?? null);
const current = ref<WallpaperItem | null>(CATALOG[0] ?? null);
const localItems = ref<WallpaperItem[]>([]);
const engine = ref<EngineState | null>(null);
const loopMode = ref<LoopMode>("list");

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
        loading: onlineGridLoading.value,
        emptyText: onlineEmptyText.value,
      };
    case "favorite":
      return {
        selectedId: selectedId.value,
        items: favoriteItems.value,
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

function syncRouteSideEffects(name: typeof route.name) {
  if (name === "online") void refreshOnline();
  if (name === "online" || name === "favorite" || name === "local") {
    if (drawerItem.value) drawerOpen.value = true;
  } else {
    drawerOpen.value = false;
  }
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
  () => settings.value.onlineEnabled,
  (enabled) => {
    if (!enabled && route.name === "online") {
      void router.replace({ name: "local" });
    }
  },
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
      onlineFetchError.value = "";
    }
    return;
  }
  onlineLoading.value = true;
  onlineFetchError.value = "";
  try {
    const { items } = await fetchOnlineWallpapers({ pageSize: 48 });
    onlineItems.value = items;
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

async function onShare(item: WallpaperItem) {
  const text = [item.name, item.author ? `by ${item.author}` : "", item.mediaSrc ?? ""]
    .filter(Boolean)
    .join("\n");
  try {
    await navigator.clipboard.writeText(text);
    showToast("已复制到剪贴板");
  } catch {
    showToast("复制失败，请检查剪贴板权限");
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
    localItems.value = localItems.value.filter((i) => i.id !== item.id);
    const { ids } = await loadFavoriteIds();
    await applyFavorites(ids);
    if (wasCurrent) {
      current.value = localItems.value[0] ?? CATALOG[0] ?? null;
      showToast(`已移除并停止桌面壁纸「${item.name}」`);
    } else {
      if (current.value?.id === item.id) {
        current.value = localItems.value[0] ?? CATALOG[0] ?? null;
      }
      showToast(`已从本地库移除「${item.name}」`);
    }
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
  lastUserAction = Date.now();
  try {
    engine.value = await enginePlay();
  } catch (e) {
    showToast(e instanceof Error ? e.message : String(e));
  }
}
async function onPause() {
  lastUserAction = Date.now();
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
let unlistenPause: (() => void) | undefined;
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
      const label = p.reason === "fullscreen" ? "全屏应用" : p.reason === "battery" ? "电池模式" : p.reason === "rdp" ? "远程桌面" : "自动";
      showToast(`已自动暂停：${label}`);
    } catch {
      /* ignore */
    }
  } else if (p.action === "play") {
    if (!engine.value?.mediaId) return;
    if (engine.value?.userPaused) return;
    try {
      engine.value = await enginePlay();
      showToast("已自动恢复播放");
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
  if (!settings.value.onlineEnabled && route.name === "online") {
    await router.replace({ name: "local" });
  }
  syncLoopModeFromSettings();
  await refreshMe().catch(() => undefined);
  await refreshLibrary();
  unlisten = await onEngineState((s) => {
    engine.value = s;
    if (s.error) toastEngineError(s.error);
    else lastEngineErrorToast = "";
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
  unlistenPause = await onPauseRecommend(applyPauseRecommend);

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
});

function onPlayWrapped() {
  lastUserAction = Date.now();
  return onPlay();
}
function onPauseWrapped() {
  lastUserAction = Date.now();
  return onPause();
}

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
      <IconRail :online-enabled="settings.onlineEnabled" />

      <RouterView v-slot="{ Component }">
        <component
          :is="Component"
          v-bind="routeViewProps"
          @select="onSelect"
          @set="onSet"
          @import="runImport()"
          @remove="onRemoveLocal"
        />
      </RouterView>

      <DetailDrawer
        :item="drawerItem"
        :open="drawerVisible"
        @close="drawerOpen = false"
        @set="onSet"
        @favorite="onFavorite"
        @share="onShare"
        @download="onDownload"
      />
    </div>

    <PlaybackBar
      :current="current"
      :engine="engine"
      :queue="playQueue"
      :loop-mode="loopMode"
      @import="runImport()"
      @open-detail="openDetail"
      @play="onPlayWrapped"
      @pause="onPauseWrapped"
      @prev="onPrev"
      @next="onNext"
      @volume="onVolume"
      @loop="onLoop"
    />
  </div>

  <Toast />
  <FirstRunAutostartModal :open="firstRunOpen" :saving="firstRunSaving" @confirm="onFirstRunConfirm" />
  <LoginModal :open="loginOpen" @close="loginOpen = false" @success="onLoginSuccess" />
</template>
