<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import {
  formatBytes,
  useOnlineDownloadProgress,
  type DownloadJob,
} from "../composables/useOnlineDownloadProgress";
import iconDownload from "../assets/svg/xiazai.svg";

const open = ref(false);
const root = ref<HTMLElement | null>(null);
const {
  todayJobs,
  activeCount,
  jobRatio,
  jobStatusText,
  cancelOnlineDownload,
  pruneIfNewDay,
} = useOnlineDownloadProgress();

const hasActive = computed(() => activeCount.value > 0);

function isActiveJob(job: DownloadJob) {
  return (
    !job.outcome &&
    (job.phase === "resolving" || job.phase === "downloading")
  );
}

function outcomeClass(job: DownloadJob) {
  if (job.outcome === "success" || job.phase === "done" || job.phase === "cached") {
    return "is-ok";
  }
  if (job.outcome === "failed" || job.phase === "failed") return "is-fail";
  if (job.outcome === "cancelled" || job.phase === "cancelled") return "is-cancel";
  return "";
}

function toggle() {
  pruneIfNewDay();
  open.value = !open.value;
}

function onDocPointerDown(e: PointerEvent) {
  if (!open.value) return;
  const el = root.value;
  if (el && e.target instanceof Node && !el.contains(e.target)) {
    open.value = false;
  }
}

async function onCancel(id: string) {
  await cancelOnlineDownload(id);
}

onMounted(() => {
  pruneIfNewDay();
  document.addEventListener("pointerdown", onDocPointerDown, true);
});
onUnmounted(() => {
  document.removeEventListener("pointerdown", onDocPointerDown, true);
});
</script>

<template>
  <div ref="root" class="win-dl">
    <button
      type="button"
      class="win-act icon-btn win-dl-btn"
      :class="{ 'has-active': hasActive }"
      aria-label="下载记录"
      title="下载记录"
      :aria-expanded="open"
      @mousedown.stop
      @click.stop="toggle"
    >
      <img :src="iconDownload" alt="" class="win-act-icon" draggable="false" />
      <span v-if="hasActive" class="win-dl-badge">{{ activeCount > 9 ? "9+" : activeCount }}</span>
    </button>

    <div v-if="open" class="win-dl-panel" role="dialog" aria-label="今日下载">
      <div class="win-dl-head">今日下载</div>
      <div v-if="todayJobs.length === 0" class="win-dl-empty">今日暂无下载记录</div>
      <ul v-else class="win-dl-list">
        <li
          v-for="job in todayJobs"
          :key="job.id"
          class="win-dl-item"
          :class="outcomeClass(job)"
        >
          <div class="win-dl-row">
            <div class="win-dl-meta">
              <div class="win-dl-title" :title="job.label">{{ job.label }}</div>
              <div class="win-dl-status">
                <template v-if="isActiveJob(job) && jobRatio(job) != null">
                  {{ jobStatusText(job) }}
                  <span class="win-dl-size">
                    · {{ formatBytes(job.downloaded) }}
                    <template v-if="job.total"> / {{ formatBytes(job.total) }}</template>
                  </span>
                </template>
                <template v-else>{{ jobStatusText(job) }}</template>
              </div>
            </div>
            <button
              v-if="isActiveJob(job)"
              type="button"
              class="win-dl-cancel"
              title="取消下载"
              @click.stop="onCancel(job.id)"
            >
              取消
            </button>
            <span v-else class="win-dl-tag">{{
              job.outcome === "failed" || job.phase === "failed"
                ? "失败"
                : job.outcome === "cancelled" || job.phase === "cancelled"
                  ? "取消"
                  : "成功"
            }}</span>
          </div>
          <div v-if="isActiveJob(job)" class="win-dl-track" aria-hidden="true">
            <div
              class="win-dl-fill"
              :class="{ pulse: jobRatio(job) == null && job.phase === 'downloading' }"
              :style="jobRatio(job) != null ? { transform: `scaleX(${jobRatio(job)})` } : undefined"
            />
          </div>
        </li>
      </ul>
    </div>
  </div>
</template>
