<script setup lang="ts">
import { computed } from "vue";
import WallpaperCardGrid from "../components/WallpaperCardGrid.vue";
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
</script>

<template>
  <div class="main">
    <div class="fav-title">我的收藏</div>
    <div class="fav-sub">已收藏 {{ favs.length }} 张壁纸 · 本地保存，无需登录</div>

    <div v-if="favs.length === 0" class="placeholder">
      <p>还没有收藏，在壁纸卡片上点 ❤️ 收藏吧</p>
    </div>

    <WallpaperCardGrid
      v-else
      :items="favs"
      :selected-id="selectedId"
      @select="emit('select', $event)"
      @set="emit('set', $event)"
    />
  </div>
</template>
