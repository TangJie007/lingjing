<script setup lang="ts">
import { computed } from "vue";
import { CATALOG, type WallpaperItem } from "../data/catalog";

const props = defineProps<{ selectedId?: number | null }>();
const emit = defineEmits<{
  (e: "select", item: WallpaperItem): void;
  (e: "set", item: WallpaperItem): void;
  (e: "favorite", item: WallpaperItem): void;
}>();

const favs = computed(() => CATALOG.filter((i) => i.favorite));
const count = computed(() => favs.value.length);

function pick(item: WallpaperItem) {
  emit("select", item);
}
function setWp(item: WallpaperItem) {
  emit("set", item);
}
</script>

<template>
  <div class="flex-1 overflow-y-auto px-7 py-5">
    <h3 style="font-size: 18px; font-weight: 700; margin-bottom: 4px;">我的收藏</h3>
    <div style="font-size: 12.5px; color: var(--text-3); margin-bottom: 18px;">
      已收藏 {{ count }} 张壁纸 · 本地保存，无需登录
    </div>

    <div v-if="count === 0" class="empty">
      还没有收藏，去发现页点 ❤️ 收藏吧~
    </div>

    <div v-else class="grid">
      <div
        v-for="(item, idx) in favs"
        :key="item.id"
        :class="['card', 'group', 'card-in', props.selectedId === item.id ? 'selected' : '']"
        :style="{ animationDelay: `${Math.min(idx, 5) * 60}ms` }"
        role="button"
        tabindex="0"
        :aria-label="`${item.name}，设为壁纸`"
        @click="pick(item)"
        @keydown="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); pick(item); } }"
      >
        <div class="thumb aspect-[16/10] w-full">
          <div class="thumb-bg" :style="{ background: item.thumb }" />
          <span v-if="item.type === 'video' || item.type === 'gif'" class="badge">LIVE</span>
          <span class="vol">{{ item.size }}</span>
          <div class="hover-acts">
            <button class="ha-btn preview" @click.stop="pick(item)">▶ 预览</button>
            <button class="ha-btn apply" @click.stop="setWp(item)">设为壁纸</button>
          </div>
        </div>
        <div class="info">
          <div class="t">{{ item.name }}</div>
          <div class="m">{{ item.category }} · {{ item.type === 'video' ? '4K' : item.type === 'gif' ? 'GIF' : '2K' }}</div>
        </div>
        <span class="liked-corner">❤️</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.empty {
  text-align: center;
  color: var(--text-3);
  font-size: 13px;
  padding: 48px 0;
}
.grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 14px;
  align-content: start;
}
@media (max-width: 680px) {
  .grid {
    grid-template-columns: repeat(2, 1fr);
  }
}
.card {
  border: 1px solid var(--border);
  border-radius: var(--r-md);
  overflow: hidden;
  background: var(--surface);
  cursor: pointer;
  position: relative;
  transition: transform var(--dur-fast) var(--ease),
    box-shadow var(--dur-fast) var(--ease),
    border-color var(--dur-fast) var(--ease);
}
.card:hover {
  transform: translateY(-4px);
  box-shadow: var(--sh-md);
  border-color: var(--border-strong);
}
.card.selected {
  border-color: var(--primary);
  box-shadow: 0 0 0 2px var(--primary-soft);
}
.thumb {
  position: relative;
  overflow: hidden;
}
.thumb-bg {
  position: absolute;
  inset: 0;
  transition: transform var(--dur-base) var(--ease);
}
.card:hover .thumb-bg {
  transform: scale(1.07);
}
.badge {
  position: absolute;
  left: 7px;
  top: 7px;
  font-size: 10px;
  padding: 2px 7px;
  border-radius: var(--r-pill);
  background: rgba(255, 255, 255, 0.9);
  color: var(--text);
  font-weight: 600;
  z-index: 2;
}
.vol {
  position: absolute;
  right: 7px;
  bottom: 7px;
  font-size: 10px;
  padding: 2px 7px;
  border-radius: var(--r-pill);
  background: rgba(17, 24, 39, 0.55);
  color: #fff;
  z-index: 2;
}
.hover-acts {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  gap: 8px;
  padding-bottom: 11px;
  background: linear-gradient(to top, rgba(17, 24, 39, 0.55), transparent 55%);
  opacity: 0;
  transform: translateY(8px);
  transition: opacity var(--dur-fast) var(--ease), transform var(--dur-fast) var(--ease);
  z-index: 3;
}
.card:hover .hover-acts {
  opacity: 1;
  transform: translateY(0);
}
.ha-btn {
  font-size: 11.5px;
  font-weight: 600;
  padding: 6px 12px;
  border-radius: var(--r-pill);
  cursor: pointer;
  border: 1px solid transparent;
  transition: transform var(--dur-fast) var(--ease);
}
.ha-btn.preview {
  background: rgba(255, 255, 255, 0.94);
  color: var(--text);
}
.ha-btn.apply {
  background: var(--primary);
  color: #fff;
}
.ha-btn:hover {
  transform: scale(1.06);
}
.ha-btn:active {
  transform: scale(0.92);
}
.info {
  padding: 9px 11px;
}
.info .t {
  font-size: 13px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.info .m {
  font-size: 11px;
  color: var(--text-3);
  margin-top: 2px;
}
.liked-corner {
  position: absolute;
  right: 2px;
  top: 2px;
  font-size: 12px;
  z-index: 2;
}
.card-in {
  animation: fadeUp var(--dur-slow) var(--ease-out) both;
}
</style>
