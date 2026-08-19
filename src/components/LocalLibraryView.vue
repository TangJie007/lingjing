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
  (e: "import"): void;
  (e: "remove", item: WallpaperItem): void;
}>();

const empty = computed(() => props.items.length === 0);

function meta(item: WallpaperItem) {
  if (item.missing) return "源文件缺失";
  return `${item.category} · ${item.size}`;
}

function confirmRemove(item: WallpaperItem, ev: Event) {
  ev.stopPropagation();
  if (window.confirm(`从本地库移除「${item.name}」？物理文件也会被删除。`)) {
    emit("remove", item);
  }
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
        :class="{ selected: props.selectedId === item.id, missing: item.missing }"
        :style="{ animationDelay: `${Math.min(idx, 5) * 60}ms` }"
        role="button"
        tabindex="0"
        :aria-label="`${item.name}，设为壁纸`"
        @click="emit('select', item)"
        @keydown="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); emit('select', item); } }"
      >
        <div class="thumb">
          <MediaThumb :item="item" />
          <span v-if="item.missing" class="badge missing-badge">缺失</span>
          <span v-else-if="item.type === 'video' || item.type === 'gif'" class="badge">LIVE</span>
          <span class="vol">{{ item.size }}</span>
          <span
            class="del"
            role="button"
            tabindex="0"
            aria-label="删除本地项"
            @click.stop="confirmRemove(item, $event)"
            @keydown.stop="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); confirmRemove(item, e); } }"
          >🗑</span>
          <div class="hover-acts">
            <span class="ha-btn preview" @click.stop="emit('select', item)">▶ 预览</span>
            <span
              v-if="!item.missing"
              class="ha-btn apply"
              @click.stop="emit('set', item)"
            >设为壁纸</span>
            <span v-else class="ha-btn apply disabled" title="源文件已缺失">无法设壁纸</span>
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

<style scoped>
.thumb {
  position: relative;
}
.del {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 28px;
  height: 28px;
  border-radius: 999px;
  background: rgba(15, 17, 24, 0.65);
  color: #fff;
  font-size: 13px;
  display: grid;
  place-items: center;
  cursor: pointer;
  opacity: 0;
  transition: opacity var(--dur-fast) var(--ease), transform var(--dur-fast) var(--ease);
  z-index: 4;
}
.card:hover .del,
.card:focus-within .del,
.del:focus-visible {
  opacity: 1;
}
.del:hover { transform: scale(1.05); background: var(--error); }
.card.missing .thumb-bg { opacity: 0.45; filter: grayscale(0.85); }
.card.missing .t { color: var(--text-3); }
.missing-badge { background: var(--error) !important; }
.ha-btn.disabled {
  opacity: 0.55;
  cursor: not-allowed;
  pointer-events: none;
}
</style>
