<script setup lang="ts">
import { ref } from "vue";
import TopBar from "./components/TopBar.vue";
import WallpaperList, { type WallpaperItem } from "./components/WallpaperList.vue";
import PreviewStage from "./components/PreviewStage.vue";

const preview = ref<InstanceType<typeof PreviewStage> | null>(null);

function onSelect(item: WallpaperItem) {
  preview.value?.onSelect(item);
}
</script>

<template>
  <div class="flex h-full flex-col overflow-hidden rounded-2xl border border-[var(--border)] bg-[var(--bg)] shadow-[0_24px_64px_rgba(0,0,0,0.45)] backdrop-blur-2xl backdrop-saturate-140 animate-fade-in">
    <TopBar />
    <div class="flex min-h-0 flex-1">
      <aside class="flex w-[260px] flex-shrink-0 flex-col border-r border-[var(--border)] bg-[var(--bg-elevated)]">
        <WallpaperList @select="onSelect" />
      </aside>
      <main class="flex min-w-0 flex-1">
        <PreviewStage ref="preview" />
      </main>
    </div>
  </div>
</template>
