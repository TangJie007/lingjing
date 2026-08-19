<script setup lang="ts">
import { computed, inject, ref, type Ref } from "vue";
import TopBar from "./TopBar.vue";
import MediaThumb from "./MediaThumb.vue";
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
    emptyText: "没有匹配的壁纸",
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

function meta(item: WallpaperItem) {
  if (item.source === "online") {
    return `${item.category} · ${item.size}`;
  }
  const res = item.type === "image" && item.size === "740K" ? "2K" : "4K";
  return `${item.category} · ${res}`;
}
</script>

<template>
  <div class="main">
    <TopBar />
    <div class="cats">
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
    <p v-if="loading" class="grid-status">正在加载在线壁纸…</p>
    <div v-else class="grid">
      <div
        v-for="(item, idx) in list"
        :key="item.id"
        class="card"
        :class="{ selected: props.selectedId === item.id }"
        :style="{ animationDelay: `${Math.min(idx, 5) * 60}ms` }"
        role="button"
        tabindex="0"
        :aria-label="`${item.name}，设为壁纸`"
        @click="emit('select', item)"
        @keydown="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); emit('select', item); } }"
      >
        <div class="thumb">
          <MediaThumb :item="item" />
          <span v-if="item.type === 'video' || item.type === 'gif'" class="badge">LIVE</span>
          <span v-if="item.source === 'online'" class="badge online-badge">在线</span>
          <span class="vol">{{ item.size }}</span>
          <div class="hover-acts">
            <span class="ha-btn preview" @click.stop="emit('select', item)">▶ 预览</span>
            <span class="ha-btn apply" @click.stop="emit('set', item)">设为壁纸</span>
          </div>
        </div>
        <div class="info">
          <div class="t">{{ item.name }}</div>
          <div class="m">{{ meta(item) }}</div>
        </div>
      </div>
      <p v-if="list.length === 0" class="grid-empty">{{ emptyText }}</p>
    </div>
  </div>
</template>

<style scoped>
.grid-status,
.grid-empty {
  grid-column: 1 / -1;
  text-align: center;
  color: var(--text-3);
  padding: 40px 0;
  font-size: 13px;
}
.online-badge {
  left: auto;
  right: 8px;
  top: auto;
  bottom: 8px;
  background: rgba(79, 70, 229, 0.85);
}
</style>
