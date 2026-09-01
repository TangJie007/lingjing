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
  (e: "preview", item: WallpaperItem): void;
  (e: "set", item: WallpaperItem): void;
  (e: "import"): void;
  (e: "remove", item: WallpaperItem): void;
}>();

const empty = computed(() => props.items.length === 0);

function confirmRemove(item: WallpaperItem, ev: Event) {
  ev.stopPropagation();
  if (window.confirm(`从本地库移除「${item.name}」？物理文件也会被删除。`)) {
    emit("remove", item);
  }
}
</script>

<template>
  <div class="main">
    <div class="fav-title page-head-row">
      <span>本地库</span>
      <button type="button" class="pill imp" @click="emit('import')">＋ 导入</button>
    </div>
    <div class="fav-sub">拖放文件到窗口或点击导入 · 支持 mp4 / webm / gif / webp / jpg / png</div>

    <div v-if="empty" class="placeholder">
      <p>拖放或点击导入壁纸</p>
    </div>

    <WallpaperCardGrid
      v-else
      :items="items"
      :selected-id="selectedId"
      @select="emit('select', $event)"
      @preview="emit('preview', $event)"
      @set="emit('set', $event)"
    >
      <template #badge="{ item }">
        <span v-if="item.missing" class="badge missing-badge">缺失</span>
        <span v-else-if="item.type === 'video' || item.type === 'gif'" class="badge">LIVE</span>
      </template>
      <template #overlay="{ item }">
        <span
          class="del"
          role="button"
          tabindex="0"
          aria-label="删除本地项"
          @click.stop="confirmRemove(item, $event)"
          @keydown.stop="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); confirmRemove(item, e); } }"
        >🗑</span>
      </template>
    </WallpaperCardGrid>
  </div>
</template>
