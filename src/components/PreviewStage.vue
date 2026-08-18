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
  <div class="preview">
    <div class="canvas">
      <div v-if="current" class="media" :data-type="current.type">
        <span class="placeholder">{{ current.name }}</span>
        <span class="badge">{{ current.type.toUpperCase() }}</span>
      </div>
      <div v-else class="media empty">
        <span class="placeholder">从左侧选择壁纸预览</span>
      </div>
    </div>
    <div class="footer">
      <span class="status">{{ status || " " }}</span>
      <button class="apply" :disabled="!current" @click="applyWallpaper">
        应用为壁纸
      </button>
    </div>
  </div>
</template>

<style scoped>
.preview {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  padding: 16px;
  gap: 14px;
}

.canvas {
  flex: 1;
  border-radius: var(--radius);
  overflow: hidden;
  border: 1px solid var(--border);
  background: radial-gradient(120% 120% at 30% 20%, #23232c, #15151b);
  min-height: 0;
}

.media {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, rgba(124, 92, 255, 0.25), rgba(74, 214, 255, 0.15));
}
.media[data-type="web"] {
  background: linear-gradient(135deg, rgba(255, 124, 174, 0.25), rgba(255, 184, 108, 0.15));
}
.media[data-type="image"] {
  background: linear-gradient(135deg, rgba(67, 233, 123, 0.22), rgba(56, 249, 215, 0.15));
}
.media.empty {
  background: transparent;
}

.placeholder {
  font-size: 15px;
  color: var(--text);
  opacity: 0.85;
}

.badge {
  position: absolute;
  top: 12px;
  right: 12px;
  font-size: 10px;
  padding: 3px 8px;
  border-radius: 999px;
  background: rgba(0, 0, 0, 0.35);
  color: #fff;
  letter-spacing: 0.5px;
}

.footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.status {
  font-size: 12px;
  color: var(--text-dim);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.apply {
  padding: 9px 20px;
  border-radius: 10px;
  font-size: 13px;
  font-weight: 600;
  color: #fff;
  background: linear-gradient(135deg, #7c5cff, #6a4dff);
  box-shadow: 0 6px 18px rgba(124, 92, 255, 0.4);
  transition: transform 0.12s, opacity 0.15s;
}
.apply:hover {
  transform: translateY(-1px);
}
.apply:disabled {
  opacity: 0.4;
  cursor: not-allowed;
  transform: none;
}
</style>
