<script setup lang="ts">
import { computed } from "vue";
import type { WallpaperItem } from "../data/catalog";

const props = defineProps<{
  items: WallpaperItem[];
  selectedId?: string | null;
}>();
const emit = defineEmits<{
  (e: "select", item: WallpaperItem): void;
  (e: "set", item: WallpaperItem): void;
  (e: "import"): void;
}>();

const empty = computed(() => props.items.length === 0);

function meta(item: WallpaperItem) {
  return `${item.category} · ${item.size}`;
}
</script>

<template>
  <div class="main">
    <div class="fav-title" style="display:flex;align-items:center;justify-content:space-between;gap:12px;">
      <span>本地库</span>
      <button
        type="button"
        class="pill imp"
        style="cursor:pointer;"
        @click="emit('import')"
      >＋ 导入</button>
    </div>
    <div class="fav-sub">拖放文件到窗口或点击导入 · 支持 mp4 / webm / gif / webp / jpg / png</div>

    <div v-if="empty" class="placeholder">
      <p>拖放或点击导入壁纸</p>
    </div>

    <div v-else class="grid">
      <div
        v-for="(item, idx) in items"
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
    </div>
  </div>
</template>
