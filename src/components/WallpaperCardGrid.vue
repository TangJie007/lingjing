<script setup lang="ts">
import MediaThumb from "./MediaThumb.vue";
import type { WallpaperItem } from "../data/catalog";
import { wallpaperMeta } from "../composables/wallpaperMeta";

withDefaults(
  defineProps<{
    items: WallpaperItem[];
    selectedId?: string | null;
    meta?: (item: WallpaperItem) => string;
  }>(),
  {
    meta: wallpaperMeta,
  },
);

const emit = defineEmits<{
  (e: "select", item: WallpaperItem): void;
  (e: "preview", item: WallpaperItem): void;
  (e: "set", item: WallpaperItem): void;
}>();
</script>

<template>
  <div class="grid">
    <div
      v-for="(item, idx) in items"
      :key="item.id"
      class="card"
      :class="{ selected: selectedId === item.id, missing: item.missing }"
      :style="{ animationDelay: `${Math.min(idx, 5) * 60}ms` }"
      role="button"
      tabindex="0"
      :aria-label="`${item.name}，设为壁纸`"
      @click="emit('select', item)"
      @keydown="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); emit('select', item); } }"
    >
      <div class="thumb">
        <MediaThumb :item="item" />
        <slot name="badge" :item="item">
          <span v-if="item.type === 'video' || item.type === 'gif'" class="badge">LIVE</span>
        </slot>
        <span class="vol">{{ item.size }}</span>
        <slot name="overlay" :item="item" />
        <div class="hover-acts">
          <span class="ha-btn preview" @click.stop="emit('preview', item)">▶ 预览</span>
          <slot name="apply" :item="item">
            <span
              v-if="!item.missing"
              class="ha-btn apply"
              @click.stop="emit('set', item)"
            >设为壁纸</span>
            <span v-else class="ha-btn apply disabled" title="源文件已缺失">无法设壁纸</span>
          </slot>
        </div>
      </div>
      <div class="info">
        <div class="t">{{ item.name }}</div>
        <div class="m">{{ meta(item) }}</div>
      </div>
    </div>
  </div>
</template>
