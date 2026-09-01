<script setup lang="ts">
import { computed, inject, ref, watch, type Ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import TopBar from "../components/TopBar.vue";
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

const categories = computed(() => {
  const src = props.items ?? CATALOG;
  const set = new Set<string>(["全部"]);
  for (const i of src) {
    if (i.category) set.add(i.category);
  }
  return [...set];
});

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

const totalCount = computed(() => (props.items ?? CATALOG).length);
</script>

<template>
  <div class="main">
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

    <template v-if="tab === 'discover'">
      <div class="fav-sub">
        已加载 {{ totalCount }} 张 · 浏览灵境社区动态壁纸 · 登录后同步点赞
      </div>

      <TopBar />

      <div v-if="categories.length > 1" class="cats">
        <span
          v-for="c in categories"
          :key="c"
          class="chip"
          :class="{ on: activeCat === c }"
          role="button"
          tabindex="0"
          @click="activeCat = c"
          @keydown="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); activeCat = c; } }"
        >{{ c }}</span>
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
