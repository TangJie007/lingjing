<script setup lang="ts">
import { computed, ref, watch } from "vue";
import MediaThumb from "./MediaThumb.vue";
import type { WallpaperItem } from "../data/catalog";
import { useOnlineDownloadProgress } from "../composables/useOnlineDownloadProgress";

const props = defineProps<{ item: WallpaperItem | null; open: boolean }>();
const emit = defineEmits<{
  (e: "close"): void;
  (e: "open-detail", item: WallpaperItem): void;
  (e: "set", item: WallpaperItem): void;
  (e: "favorite", item: WallpaperItem): void;
  (e: "download", item: WallpaperItem): void;
}>();

const { isDownloadingItem, isDownloadDisabled, downloadButtonLabel } = useOnlineDownloadProgress();
const downloading = computed(() => isDownloadingItem(props.item?.id));
const downloadDisabled = computed(() => isDownloadDisabled(props.item?.id));
const downloadLabel = computed(() => downloadButtonLabel(props.item?.id));

const COUNTDOWN = 30;
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

watch(downloading, (busy) => {
  if (busy) stopCountdown();
  else if (props.open && props.item) startCountdown();
});

const tags = computed(() => props.item?.tags ?? ["#4K"]);

function onDownloadClick() {
  if (!props.item || downloadDisabled.value) return;
  emit("download", props.item);
}
</script>

<template>
  <div class="detail" :class="{ open: open && item, closed: !(open && item) }">
    <div v-if="item" class="detail-inner">
      <div
        class="d-prev"
        role="button"
        tabindex="0"
        title="查看详情"
        @click="item && emit('open-detail', item)"
        @keydown="(e) => { if ((e.key === 'Enter' || e.key === ' ') && item) { e.preventDefault(); emit('open-detail', item); } }"
      >
        <MediaThumb :item="item" mode="preview" />
        <div class="wheel" :class="{ run: wheelRun }">
          <svg class="wheel-svg" viewBox="0 0 50 50" aria-hidden="true">
            <circle class="wheel-bg" cx="25" cy="25" r="22" />
            <circle class="wheel-ring" cx="25" cy="25" r="22" />
          </svg>
          <div class="wheel-num" aria-live="polite">
            <div class="n">{{ remain }}</div>
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
        <div v-if="item.source !== 'local'" class="act-row">
          <div
            class="btn-ghost"
            :class="{ liked: item.favorite }"
            role="button"
            tabindex="0"
            :aria-label="item.favorite ? '已收藏' : '收藏'"
            @click="emit('favorite', item)"
          >{{ item.favorite ? "♥ 收藏" : "♡ 收藏" }}</div>
          <div
            class="btn-ghost"
            :class="{ busy: downloading, disabled: downloadDisabled && !downloading }"
            role="button"
            tabindex="0"
            aria-label="下载"
            :aria-busy="downloading"
            :aria-disabled="downloadDisabled"
            @click="onDownloadClick"
          >{{ downloadLabel }}</div>
        </div>
        <div class="btn-apply" role="button" tabindex="0" aria-label="设为壁纸" @click="emit('set', item)">设为壁纸</div>
        <div class="d-note">30 秒未操作将自动收起 · 点击预览查看详情</div>
      </div>
    </div>
  </div>
</template>
