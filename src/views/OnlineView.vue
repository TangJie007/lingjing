<script setup lang="ts">
import { computed, inject, ref, watch, type Ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import WallpaperCardGrid from "../components/WallpaperCardGrid.vue";
import EmptyState from "../components/EmptyState.vue";
import { CATALOG, type WallpaperItem } from "../data/catalog";

const props = withDefaults(
  defineProps<{
    selectedId?: string | null;
    items?: WallpaperItem[];
    favorites?: WallpaperItem[];
    loading?: boolean;
    emptyText?: string;
    onlineEnabled?: boolean;
  }>(),
  {
    loading: false,
    emptyText: "暂无在线壁纸，请确认 API 服务已启动",
    onlineEnabled: false,
    favorites: () => [],
  },
);

const emit = defineEmits<{
  (e: "select", item: WallpaperItem): void;
  (e: "preview", item: WallpaperItem): void;
  (e: "set", item: WallpaperItem): void;
}>();

type OnlineTab = "discover" | "mine";

type FilterKind = "cat" | "sort";

const FILTERS: { key: string; kind: FilterKind }[] = [
  { key: "全部", kind: "cat" },
  { key: "最热", kind: "sort" },
  { key: "最新", kind: "sort" },
  { key: "科技", kind: "cat" },
  { key: "风景", kind: "cat" },
  { key: "动漫", kind: "cat" },
];

const route = useRoute();
const router = useRouter();

const tab = ref<OnlineTab>(
  route.query.tab === "mine" ? "mine" : "discover",
);

watch(
  () => route.query.tab,
  (q) => {
    tab.value = q === "mine" ? "mine" : "discover";
  },
);

function setTab(next: OnlineTab) {
  tab.value = next;
  void router.replace({
    name: "online",
    query: next === "mine" ? { tab: "mine" } : {},
  });
}

const activeCat = ref("全部");
const search = inject<Ref<string>>("topbarSearch", ref(""));
const sort = inject<Ref<string>>("topbarSort", ref("最热"));

function isFilterOn(key: string, kind: FilterKind) {
  return kind === "sort" ? sort.value === key : activeCat.value === key;
}

function setFilter(key: string, kind: FilterKind) {
  if (kind === "sort") sort.value = key;
  else activeCat.value = key;
}

const discoverList = computed(() => {
  let r = [...(props.items ?? CATALOG)];
  if (activeCat.value !== "全部") r = r.filter((i) => i.category === activeCat.value);
  if (search.value.trim()) {
    const q = search.value.trim().toLowerCase();
    r = r.filter(
      (i) =>
        i.name.toLowerCase().includes(q) ||
        i.category.toLowerCase().includes(q) ||
        i.author.toLowerCase().includes(q),
    );
  }
  if (sort.value === "最新") r = [...r].reverse();
  return r;
});

const favoriteList = computed(() => props.favorites ?? []);
</script>

<template>
  <div class="main">
    <div class="online-head">
      <div class="online-tabs" role="tablist" aria-label="在线分区">
        <button
          type="button"
          class="online-tab"
          role="tab"
          :class="{ on: tab === 'discover' }"
          :aria-selected="tab === 'discover'"
          @click="setTab('discover')"
        >
          发现
        </button>
        <button
          type="button"
          class="online-tab"
          role="tab"
          :class="{ on: tab === 'mine' }"
          :aria-selected="tab === 'mine'"
          @click="setTab('mine')"
        >
          我的
        </button>
      </div>

      <div class="search">
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <circle cx="11" cy="11" r="7" />
          <path d="M21 21l-4.3-4.3" />
        </svg>
        <template v-if="!search">
          <span class="hint">大家都在搜：</span>
          <span class="kw">极光</span>
          <span class="hint">· 二次元 · 赛博城市</span>
        </template>
        <input
          v-model="search"
          type="search"
          class="search-input"
          :class="{ 'is-empty': !search }"
          aria-label="搜索壁纸"
        />
      </div>
    </div>

    <template v-if="tab === 'discover'">
      <div class="online-filters" role="toolbar" aria-label="筛选与排序">
        <span
          v-for="f in FILTERS"
          :key="f.key"
          :class="{ on: isFilterOn(f.key, f.kind) }"
          role="button"
          tabindex="0"
          :aria-pressed="isFilterOn(f.key, f.kind)"
          @click="setFilter(f.key, f.kind)"
          @keydown="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); setFilter(f.key, f.kind); } }"
        >{{ f.key }}</span>
      </div>

      <EmptyState v-if="!onlineEnabled" description="在线功能已关闭，可在设置中开启" />
      <EmptyState v-else-if="loading" description="正在加载在线壁纸…" />
      <EmptyState v-else-if="discoverList.length === 0" :description="emptyText" />
      <WallpaperCardGrid
        v-else
        :items="discoverList"
        :selected-id="selectedId"
        @select="emit('select', $event)"
        @preview="emit('preview', $event)"
        @set="emit('set', $event)"
      />
    </template>

    <template v-else>
      <div class="fav-sub">已收藏 {{ favoriteList.length }} 张壁纸 · 本地保存，无需登录</div>

      <EmptyState
        v-if="favoriteList.length === 0"
        description="还没有收藏，在壁纸卡片上点收藏吧"
      />
      <WallpaperCardGrid
        v-else
        :items="favoriteList"
        :selected-id="selectedId"
        @select="emit('select', $event)"
        @preview="emit('preview', $event)"
        @set="emit('set', $event)"
      />
    </template>
  </div>
</template>
