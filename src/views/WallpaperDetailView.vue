<script setup lang="ts">
import { computed, inject } from "vue";
import { useRoute, useRouter } from "vue-router";
import DetailMediaPreview from "../components/DetailMediaPreview.vue";
import EmptyState from "../components/EmptyState.vue";
import type { WallpaperItem } from "../data/catalog";

const route = useRoute();
const router = useRouter();

const findWallpaper = inject<(id: string) => WallpaperItem | null>("findWallpaper");

const item = computed(() => {
  const id = String(route.params.id ?? "");
  return findWallpaper?.(id) ?? null;
});

const tags = computed(() => item.value?.tags ?? ["#4K"]);

const emit = defineEmits<{
  (e: "set", item: WallpaperItem): void;
  (e: "favorite", item: WallpaperItem): void;
  (e: "download", item: WallpaperItem): void;
}>();

function goBack() {
  if (window.history.length > 1) router.back();
  else void router.push({ name: "local" });
}
</script>

<template>
  <div class="main detail-page">
    <div class="detail-page-head">
      <button type="button" class="detail-back" @click="goBack">← 返回</button>
    </div>

    <div v-if="!item" class="detail-empty">
      <EmptyState description="未找到该壁纸" />
      <button type="button" class="pill imp" @click="goBack">返回列表</button>
    </div>

    <div v-else class="detail-page-split">
      <div class="detail-page-media">
        <DetailMediaPreview :item="item" fill />
      </div>

      <aside class="detail-page-body">
        <h2>{{ item.name }}</h2>
        <div class="by">by 设计者 · {{ item.author }}</div>
        <div class="d-meta">
          <div>
            <div class="k">分辨率</div>
            <div class="v">自适应</div>
          </div>
          <div>
            <div class="k">大小</div>
            <div class="v">{{ item.size }}</div>
          </div>
          <div>
            <div class="k">热度</div>
            <div class="v">{{ item.heat }}</div>
          </div>
        </div>
        <div class="d-tags">
          <span v-for="t in tags" :key="t" class="tag-demo">{{ t }}</span>
        </div>
        <div class="detail-page-actions">
          <div v-if="item.source !== 'local'" class="act-row">
            <div
              class="btn-ghost"
              :class="{ liked: item.favorite }"
              role="button"
              tabindex="0"
              @click="emit('favorite', item)"
            >
              {{ item.favorite ? "♥ 收藏" : "♡ 收藏" }}
            </div>
            <div class="btn-ghost" role="button" tabindex="0" @click="emit('download', item)">
              ↓ 下载
            </div>
          </div>
          <div
            class="btn-apply"
            role="button"
            tabindex="0"
            @click="emit('set', item)"
          >
            设为壁纸
          </div>
        </div>
      </aside>
    </div>
  </div>
</template>

<style scoped>
.detail-page {
  overflow: hidden;
}
.detail-empty {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
}
.detail-page-split {
  flex: 1;
  min-height: 0;
  display: flex;
  gap: 20px;
  align-items: stretch;
}
.detail-page-media {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
}
.detail-page-body {
  width: 280px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow-y: auto;
  padding: 4px 2px 8px;
}
.detail-page-body h2 {
  margin: 0;
  font-size: 18px;
  font-weight: 700;
  line-height: 1.35;
  word-break: break-word;
}
.detail-page-body .by {
  margin-top: 6px;
  font-size: 12.5px;
  color: var(--text-2);
}
.detail-page-body .d-meta {
  margin: 18px 0 14px;
}
.detail-page-actions {
  margin-top: auto;
  padding-top: 16px;
}
.detail-page-actions .act-row {
  margin-bottom: 10px;
}
.detail-page-actions .btn-ghost,
.detail-page-actions .btn-apply {
  width: 100%;
  justify-content: center;
  box-sizing: border-box;
}
.detail-page-actions .act-row {
  display: flex;
  gap: 8px;
}
.detail-page-actions .act-row .btn-ghost {
  flex: 1;
}
</style>
