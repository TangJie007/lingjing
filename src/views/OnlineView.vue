<script setup lang="ts">
import { computed, inject, ref, type Ref } from "vue";
import TopBar from "../components/TopBar.vue";
import WallpaperCardGrid from "../components/WallpaperCardGrid.vue";
import { CATALOG, type WallpaperItem } from "../data/catalog";

const props = withDefaults(
  defineProps<{
    selectedId?: string | null;
    items?: WallpaperItem[];
    loading?: boolean;
    emptyText?: string;
  }>(),
  {
    loading: false,
    emptyText: "暂无在线壁纸，请确认 API 服务已启动",
  },
);

const emit = defineEmits<{
  (e: "select", item: WallpaperItem): void;
  (e: "set", item: WallpaperItem): void;
}>();

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

const list = computed(() => {
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

const totalCount = computed(() => (props.items ?? CATALOG).length);
</script>

<template>
  <div class="main">
    <div class="fav-title">在线壁纸</div>
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

    <div v-if="loading" class="placeholder">
      <p>正在加载在线壁纸…</p>
    </div>
    <div v-else-if="list.length === 0" class="placeholder">
      <p>{{ emptyText }}</p>
    </div>
    <WallpaperCardGrid
      v-else
      :items="list"
      :selected-id="selectedId"
      @select="emit('select', $event)"
      @set="emit('set', $event)"
    />
  </div>
</template>
