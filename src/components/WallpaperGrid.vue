<script setup lang="ts">
import { computed, inject, ref, type Ref } from "vue";
import TopBar from "./TopBar.vue";
import { CATALOG, CATEGORIES, type WallpaperItem } from "../data/catalog";

const props = defineProps<{ selectedId?: number | null }>();
const emit = defineEmits<{
  (e: "select", item: WallpaperItem): void;
  (e: "set", item: WallpaperItem): void;
}>();

const activeCat = ref("全部");
const search = inject<Ref<string>>("topbarSearch", ref(""));
const sort = inject<Ref<string>>("topbarSort", ref("最热"));

const list = computed(() => {
  let r = [...CATALOG];
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
  const res = item.type === "image" && item.size === "740K" ? "2K" : "4K";
  return `${item.category} · ${res}`;
}
</script>

<template>
  <div class="main">
    <TopBar />
    <div class="cats">
      <span
        v-for="c in CATEGORIES"
        :key="c"
        class="chip"
        :class="{ on: activeCat === c }"
        role="button"
        tabindex="0"
        @click="activeCat = c"
        @keydown="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); activeCat = c; } }"
      >{{ c }}</span>
    </div>
    <div class="grid">
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
          <div class="thumb-bg" :style="{ background: item.thumb }" />
          <span v-if="item.type === 'video' || item.type === 'gif'" class="badge">LIVE</span>
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
      <p v-if="list.length === 0" style="grid-column: 1 / -1; text-align: center; color: var(--text-3); padding: 40px 0; font-size: 13px;">
        没有匹配的壁纸
      </p>
    </div>
  </div>
</template>
