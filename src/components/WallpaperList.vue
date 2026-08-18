<script setup lang="ts">
import { ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";

export interface WallpaperItem {
  id: number;
  name: string;
  path: string;
  type: "video" | "image" | "web";
  active: boolean;
}

const items = ref<WallpaperItem[]>([
  { id: 1, name: "极光之夜", path: "builtin://aurora", type: "video", active: true },
  { id: 2, name: "深海粒子", path: "builtin://particles", type: "web", active: false },
  { id: 3, name: "城市霓虹", path: "builtin://neon", type: "image", active: false },
]);

const emit = defineEmits<{ (e: "select", item: WallpaperItem): void }>();

function select(item: WallpaperItem) {
  items.value.forEach((i) => (i.active = i.id === item.id));
  emit("select", item);
}

async function addWallpaper() {
  const selected = await open({
    multiple: false,
    filters: [
      { name: "媒体文件", extensions: ["mp4", "webm", "mov", "png", "jpg", "jpeg", "gif", "html"] },
    ],
  });
  if (typeof selected === "string") {
    const ext = selected.split(".").pop()?.toLowerCase() ?? "";
    const type: WallpaperItem["type"] =
      ["mp4", "webm", "mov"].includes(ext) ? "video"
      : ["html"].includes(ext) ? "web"
      : "image";
    const item: WallpaperItem = {
      id: Date.now(),
      name: selected.split(/[\\/]/).pop() ?? selected,
      path: selected,
      type,
      active: false,
    };
    items.value.push(item);
    select(item);
  }
}
</script>

<template>
  <div class="list">
    <div class="list-head">
      <span>壁纸库</span>
      <button class="add" title="添加壁纸" @click="addWallpaper">＋</button>
    </div>
    <div class="scroll">
      <button
        v-for="item in items"
        :key="item.id"
        class="item"
        :class="{ active: item.active }"
        @click="select(item)"
      >
        <span class="thumb" :data-type="item.type" />
        <span class="meta">
          <span class="name">{{ item.name }}</span>
          <span class="type">{{ item.type.toUpperCase() }}</span>
        </span>
      </button>
      <p v-if="items.length === 0" class="empty">暂无壁纸，点击 ＋ 添加</p>
    </div>
  </div>
</template>

<style scoped>
.list {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

.list-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px 10px;
  font-size: 12px;
  color: var(--text-dim);
  letter-spacing: 0.4px;
}

.add {
  width: 24px;
  height: 24px;
  border-radius: 8px;
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 16px;
  line-height: 1;
  transition: background 0.15s;
}
.add:hover {
  background: rgba(124, 92, 255, 0.3);
}

.scroll {
  flex: 1;
  overflow-y: auto;
  padding: 0 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px;
  border-radius: 10px;
  text-align: left;
  transition: background 0.15s;
}
.item:hover {
  background: rgba(255, 255, 255, 0.05);
}
.item.active {
  background: var(--accent-soft);
}

.thumb {
  width: 40px;
  height: 28px;
  border-radius: 6px;
  flex-shrink: 0;
  background: linear-gradient(135deg, #2a2a35, #3a3a48);
}
.thumb[data-type="video"] {
  background: linear-gradient(135deg, #7c5cff, #4ad6ff);
}
.thumb[data-type="web"] {
  background: linear-gradient(135deg, #ff7cae, #ffb86c);
}
.thumb[data-type="image"] {
  background: linear-gradient(135deg, #43e97b, #38f9d7);
}

.meta {
  display: flex;
  flex-direction: column;
  min-width: 0;
}
.name {
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.type {
  font-size: 10px;
  color: var(--text-dim);
}

.empty {
  text-align: center;
  font-size: 12px;
  color: var(--text-dim);
  padding: 24px 0;
}
</style>
