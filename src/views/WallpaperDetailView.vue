<script setup lang="ts">
import { computed, inject } from "vue";
import { useRoute, useRouter } from "vue-router";
import DetailMediaPreview from "../components/DetailMediaPreview.vue";
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
      <h2 v-if="item">{{ item.name }}</h2>
    </div>

    <div v-if="!item" class="placeholder">
      <p>未找到该壁纸</p>
      <button type="button" class="pill imp" @click="goBack">返回列表</button>
    </div>

    <template v-else>
      <DetailMediaPreview :item="item" />

      <div class="detail-page-body">
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
        <div class="act-row">
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
    </template>
  </div>
</template>
