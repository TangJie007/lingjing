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
  <div class="flex h-full min-h-0 flex-col">
    <div class="flex items-center justify-between px-4 pb-2.5 pt-3.5 text-xs tracking-wide text-[var(--text-dim)]">
      <span>壁纸库</span>
      <button class="flex h-6 w-6 items-center justify-center rounded-lg bg-accent-soft text-base leading-none text-accent transition-colors hover:bg-[rgba(124,92,255,0.3)]" title="添加壁纸" @click="addWallpaper">＋</button>
    </div>
    <div class="flex min-h-0 flex-1 flex-col gap-1.5 overflow-y-auto px-2.5 pb-3">
      <button
        v-for="item in items"
        :key="item.id"
        class="item flex items-center gap-2.5 rounded-xl p-2 text-left transition-colors hover:bg-white/5"
        :class="item.active ? 'bg-accent-soft animate-pop' : ''"
        @click="select(item)"
      >
        <span class="thumb h-7 w-10 flex-shrink-0 rounded-md" :data-type="item.type" />
        <span class="flex min-w-0 flex-col">
          <span class="truncate text-[13px]">{{ item.name }}</span>
          <span class="text-[10px] text-[var(--text-dim)]">{{ item.type.toUpperCase() }}</span>
        </span>
      </button>
      <p v-if="items.length === 0" class="py-6 text-center text-xs text-[var(--text-dim)]">暂无壁纸，点击 ＋ 添加</p>
    </div>
  </div>
</template>

<style scoped>
.thumb {
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
</style>
