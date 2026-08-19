<script setup lang="ts">
import { computed } from "vue";
import MediaThumb from "./MediaThumb.vue";
import type { WallpaperItem } from "../data/catalog";

const props = defineProps<{
  items: WallpaperItem[];
  selectedId?: string | null;
}>();
const emit = defineEmits<{
  (e: "select", item: WallpaperItem): void;
  (e: "set", item: WallpaperItem): void;
}>();

const favs = computed(() => props.items);

function meta(item: WallpaperItem) {
  const res = item.type === "image" && item.size === "740K" ? "2K" : "4K";
  return `${item.category} · ${res}`;
}
</script>

<template>
  <div class="main">
    <div class="fav-title">我的收藏</div>
    <div class="fav-sub">已收藏 {{ favs.length }} 张壁纸 · 本地保存，无需登录</div>

    <div v-if="favs.length === 0" class="placeholder">
      <p>还没有收藏，去发现页点 ❤️ 收藏吧~</p>
    </div>

    <div v-else class="grid">
      <div
        v-for="(item, idx) in favs"
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
    </div>
  </div>
</template>
