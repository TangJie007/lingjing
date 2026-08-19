<script setup lang="ts">
import { ref } from "vue";
import { showToast } from "../composables/useToast";
import type { WallpaperItem } from "../data/catalog";

defineProps<{ current: WallpaperItem | null }>();
const emit = defineEmits<{ (e: "import"): void }>();

const playing = ref(true);

function togglePlay() {
  playing.value = !playing.value;
}
function doImport() {
  emit("import");
  showToast("导入面板开发中");
}
</script>

<template>
  <div class="playbar">
    <div
      class="pb-thumb"
      :style="{ background: current?.thumb ?? 'var(--border)' }"
    />
    <div class="pb-btns">
      <div class="pb-btn" role="button" tabindex="0" aria-label="上一个">⟨</div>
      <div
        class="pb-btn play"
        role="button"
        tabindex="0"
        aria-label="播放或暂停"
        @click="togglePlay"
      >{{ playing ? "❚❚" : "▶" }}</div>
      <div class="pb-btn" role="button" tabindex="0" aria-label="下一个">⟩</div>
    </div>
    <div class="pb-now">
      <div class="t">{{ current ? `${current.name} · 正在播放` : "未选择壁纸" }}</div>
      <div class="bar"><i :class="{ paused: !playing }" /></div>
    </div>
    <div class="pb-right">
      <span class="pill" role="button" tabindex="0" aria-label="音量">🔊</span>
      <span class="pill" role="button" tabindex="0" aria-label="循环">循环</span>
      <span class="pill imp" role="button" tabindex="0" aria-label="导入壁纸" @click="doImport">＋ 导入</span>
    </div>
  </div>
</template>
