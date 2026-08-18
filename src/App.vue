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
  <div class="app-shell">
    <TopBar />
    <div class="app-body">
      <aside class="sidebar">
        <WallpaperList @select="onSelect" />
      </aside>
      <main class="stage">
        <PreviewStage ref="preview" />
      </main>
    </div>
  </div>
</template>

<style scoped>
.app-shell {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg);
  backdrop-filter: blur(28px) saturate(140%);
  border: 1px solid var(--border);
  border-radius: 16px;
  overflow: hidden;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.45);
}

.app-body {
  flex: 1;
  display: flex;
  min-height: 0;
}

.sidebar {
  width: 260px;
  flex-shrink: 0;
  border-right: 1px solid var(--border);
  background: var(--bg-elevated);
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.stage {
  flex: 1;
  min-width: 0;
  display: flex;
}
</style>
