<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { WallpaperItem } from "./WallpaperList.vue";

const current = ref<WallpaperItem | null>(null);
const status = ref<string>("");

function onSelect(item: WallpaperItem) {
  current.value = item;
  status.value = "";
}

async function applyWallpaper() {
  if (!current.value) return;
  // 目前仅调用示例 Rust 命令验证前后端打通，后续接入真实壁纸设置
  status.value = await invoke("greet", { name: current.value.name });
}

defineExpose({ onSelect });
</script>

<template>
  <div class="flex min-w-0 flex-1 animate-fade-in flex-col gap-3.5 p-4">
    <div
      class="canvas relative min-h-0 flex-1 overflow-hidden rounded-xl border border-[var(--border)]"
      :style="{ background: 'radial-gradient(120% 120% at 30% 20%, #23232c, #15151b)' }"
    >
      <div
        v-if="current"
        class="media relative flex h-full w-full items-center justify-center"
        :data-type="current.type"
      >
        <span class="text-[15px] opacity-85">{{ current.name }}</span>
        <span class="absolute right-3 top-3 rounded-full bg-black/35 px-2 py-0.5 text-[10px] tracking-wide text-white">{{ current.type.toUpperCase() }}</span>
      </div>
      <div v-else class="flex h-full w-full items-center justify-center">
        <span class="text-[15px] text-[var(--text)] opacity-85">从左侧选择壁纸预览</span>
      </div>
    </div>
    <div class="footer flex items-center justify-between gap-3">
      <span class="status truncate text-xs text-[var(--text-dim)]">{{ status || " " }}</span>
      <button
        class="apply rounded-xl bg-gradient-to-br from-accent to-[#6a4dff] px-5 py-2.5 text-[13px] font-semibold text-white shadow-[0_6px_18px_rgba(124,92,255,0.4)] transition-transform hover:-translate-y-px disabled:translate-y-0 disabled:cursor-not-allowed disabled:opacity-40"
        :disabled="!current"
        @click="applyWallpaper"
      >
        应用为壁纸
      </button>
    </div>
  </div>
</template>

<style scoped>
.media {
  background: linear-gradient(135deg, rgba(124, 92, 255, 0.25), rgba(74, 214, 255, 0.15));
}
.media[data-type="web"] {
  background: linear-gradient(135deg, rgba(255, 124, 174, 0.25), rgba(255, 184, 108, 0.15));
}
.media[data-type="image"] {
  background: linear-gradient(135deg, rgba(67, 233, 123, 0.22), rgba(56, 249, 215, 0.15));
}
</style>
