<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { WallpaperItem } from "../data/catalog";

const props = defineProps<{ item: WallpaperItem | null; open: boolean }>();
const emit = defineEmits<{
  (e: "close"): void;
  (e: "set", item: WallpaperItem): void;
  (e: "favorite", item: WallpaperItem): void;
  (e: "share", item: WallpaperItem): void;
  (e: "download", item: WallpaperItem): void;
}>();

const COUNTDOWN = 60;
const remain = ref(COUNTDOWN);
const wheelRun = ref(false);
let timer: number | null = null;

function startCountdown() {
  stopCountdown();
  remain.value = COUNTDOWN;
  wheelRun.value = false;
  requestAnimationFrame(() => {
    wheelRun.value = true;
  });
  timer = window.setInterval(() => {
    remain.value -= 1;
    if (remain.value <= 0) {
      stopCountdown();
      emit("close");
    }
  }, 1000);
}
function stopCountdown() {
  if (timer !== null) {
    clearInterval(timer);
    timer = null;
  }
  wheelRun.value = false;
}

watch(
  () => [props.open, props.item?.id] as const,
  ([o]) => {
    if (o && props.item) startCountdown();
    else stopCountdown();
  },
  { immediate: true },
);

const tags = computed(() => props.item?.tags ?? ["#4K"]);
</script>

<template>
  <div class="detail" :class="{ open: open && item, closed: !(open && item) }">
    <div v-if="item" class="detail-inner">
      <div class="d-prev">
        <div class="thumb-bg" :style="{ background: item.thumb }" />
        <div class="wheel" :class="{ run: wheelRun }">
          <svg class="wheel-svg" viewBox="0 0 50 50" aria-hidden="true">
            <circle class="wheel-bg" cx="25" cy="25" r="22" />
            <circle class="wheel-ring" cx="25" cy="25" r="22" />
          </svg>
          <div class="wheel-num" aria-live="polite">
            <div class="n">{{ remain }}</div>
            <div class="l">秒后取消</div>
          </div>
        </div>
      </div>
      <div class="d-body">
        <h3>{{ item.name }}</h3>
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
            :aria-label="item.favorite ? '已收藏' : '收藏'"
            @click="emit('favorite', item)"
          >{{ item.favorite ? "♥ 收藏" : "♡ 收藏" }}</div>
          <div class="btn-ghost" role="button" tabindex="0" aria-label="分享" @click="emit('share', item)">↗ 分享</div>
          <div class="btn-ghost" role="button" tabindex="0" aria-label="下载" @click="emit('download', item)">↓ 下载</div>
        </div>
        <div class="btn-apply" role="button" tabindex="0" aria-label="设为壁纸" @click="emit('set', item)">设为壁纸</div>
        <div class="d-note">60 秒未操作将自动收起预览</div>
      </div>
    </div>
  </div>
</template>
