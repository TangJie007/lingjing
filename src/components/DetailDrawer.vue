<script setup lang="ts">
import { ref, watch } from "vue";
import { showToast } from "../composables/useToast";
import type { WallpaperItem } from "../data/catalog";

const props = defineProps<{ item: WallpaperItem | null; open: boolean }>();
const emit = defineEmits<{
  (e: "close"): void;
  (e: "set", item: WallpaperItem): void;
  (e: "favorite", item: WallpaperItem): void;
}>();

const COUNTDOWN = 60;
const remain = ref(COUNTDOWN);
let timer: number | null = null;

function startCountdown() {
  stopCountdown();
  remain.value = COUNTDOWN;
  timer = window.setInterval(() => {
    remain.value -= 1;
    if (remain.value <= 0) {
      stopCountdown();
      emit("close");
    }
  }, 1000);
}
function cancelAutoClose() {
  stopCountdown();
  remain.value = 0;
}
function stopCountdown() {
  if (timer !== null) {
    clearInterval(timer);
    timer = null;
  }
}

watch(
  () => props.open,
  (o) => {
    if (o && props.item) startCountdown();
    else stopCountdown();
  },
);

// 50×50 wheel
const DASH = 2 * Math.PI * 22; // 138.23
const offset = () => DASH * (1 - remain.value / COUNTDOWN);

function setWp() {
  if (props.item) {
    emit("set", props.item);
  }
}
function fav() {
  if (props.item) {
    emit("favorite", props.item);
  }
}
function share() {
  showToast("分享面板开发中");
}
function download() {
  showToast("下载功能开发中");
}
</script>

<template>
  <transition name="drawer">
    <aside
      v-if="open && item"
      class="detail flex w-80 flex-shrink-0 flex-col border-l border-[var(--border)] bg-[var(--surface)]"
    >
      <div class="d-prev" :class="{ 'kenburns-on': open }">
        <div class="thumb-bg" :style="{ background: item.thumb }" />
        <div class="wheel" :class="open ? 'run' : ''">
          <svg class="wheel-svg" viewBox="0 0 50 50" aria-hidden="true">
            <circle class="wheel-bg" cx="25" cy="25" r="22" fill="none" stroke="rgba(255,255,255,0.3)" stroke-width="3" />
            <circle
              class="wheel-ring"
              cx="25" cy="25" r="22"
              fill="none" stroke="#fff" stroke-width="3"
              stroke-linecap="round"
              :stroke-dasharray="DASH"
              :stroke-dashoffset="offset()"
            />
          </svg>
          <div class="wheel-num" aria-live="polite">
            <div class="n">{{ remain }}</div>
            <div class="l">秒后取消</div>
          </div>
        </div>
      </div>

      <div class="d-body">
        <h3>{{ item.name }}</h3>
        <div class="by">by 设计者 · LingJing Studio</div>

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
            <div class="v">🔥 3.4k</div>
          </div>
        </div>

        <div class="d-tags">
          <span class="tag-demo">#{{ item.category }}</span>
          <span class="tag-demo">#{{ item.type === 'video' ? '动态' : item.type === 'gif' ? 'GIF' : '静态' }}</span>
          <span class="tag-demo">#4K</span>
        </div>

        <div class="act-row">
          <button
            class="btn-ghost"
            :class="item.favorite ? 'liked' : ''"
            :aria-label="item.favorite ? '已收藏' : '收藏'"
            @click="fav"
          >
            {{ item.favorite ? "♥ 收藏" : "♡ 收藏" }}
          </button>
          <button class="btn-ghost" aria-label="分享" @click="share">↗ 分享</button>
          <button class="btn-ghost" aria-label="下载" @click="download">↓ 下载</button>
        </div>

        <button class="btn-apply" aria-label="设为壁纸" @click="setWp">设为壁纸</button>

        <div class="d-actions-extra">
          <button
            v-if="remain > 0"
            class="cancel-btn"
            title="取消自动收起"
            @click="cancelAutoClose"
          >取消</button>
        </div>

        <div class="d-note">60 秒未操作将自动收起预览</div>
      </div>
    </aside>
  </transition>
</template>

<style scoped>
.drawer-enter-active,
.drawer-leave-active {
  transition: transform var(--dur-base) var(--ease), opacity var(--dur-base) var(--ease);
}
.drawer-enter-from,
.drawer-leave-to {
  transform: translateX(20px);
  opacity: 0;
}

/* 大图 */
.d-prev {
  aspect-ratio: 16 / 10;
  position: relative;
  flex-shrink: 0;
  overflow: hidden;
}
.thumb-bg {
  position: absolute;
  inset: 0;
  transition: transform var(--dur-base) var(--ease);
}
.d-prev.kenburns-on .thumb-bg {
  animation: kenburns 12s var(--ease) infinite alternate;
}

/* 50×50 wheel */
.wheel {
  position: absolute;
  right: 12px;
  bottom: 12px;
  width: 50px;
  height: 50px;
  border-radius: 50%;
  background: rgba(17, 24, 39, 0.35);
  backdrop-filter: blur(4px);
}
.wheel-svg {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  transform: rotate(-90deg);
}
.wheel.run .wheel-ring {
  animation: countdown 60s linear forwards;
}
.wheel-num {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: #fff;
}
.wheel-num .n {
  font-size: 18px;
  font-weight: 700;
  line-height: 1;
}
.wheel-num .l {
  font-size: 9px;
  opacity: 0.85;
  margin-top: 1px;
}

/* 内容区 */
.d-body {
  padding: 18px 20px;
  flex: 1;
  overflow: auto;
}
.d-body h3 {
  font-size: 18px;
  font-weight: 700;
}
.d-body .by {
  font-size: 12.5px;
  color: var(--text-2);
  margin-top: 3px;
}
.d-meta {
  display: flex;
  gap: 18px;
  margin: 16px 0;
}
.d-meta div .k {
  font-size: 11px;
  color: var(--text-3);
}
.d-meta div .v {
  font-size: 13px;
  font-weight: 600;
  margin-top: 2px;
}
.d-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 16px;
}
.tag-demo {
  display: inline-flex;
  align-items: center;
  font-size: 12px;
  padding: 4px 10px;
  border-radius: var(--r-pill);
  background: var(--surface-2);
  border: 1px solid var(--border);
  color: var(--text-2);
}

.act-row {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}
.btn-ghost {
  flex: 1;
  text-align: center;
  font-size: 13px;
  padding: 9px;
  border: 1px solid var(--border);
  border-radius: var(--r-md);
  color: var(--text-2);
  cursor: pointer;
  background: var(--surface);
  transition: background var(--dur-fast) var(--ease),
    color var(--dur-fast) var(--ease),
    border-color var(--dur-fast) var(--ease),
    transform var(--dur-fast) var(--ease);
}
.btn-ghost:hover {
  border-color: var(--border-strong);
  color: var(--text);
  transform: translateY(-1px);
}
.btn-ghost:active {
  transform: scale(0.96);
}
.btn-ghost.liked {
  color: var(--error);
  border-color: #fecaca;
  background: #fef2f2;
}
:root[data-theme="dark"] .btn-ghost.liked {
  background: rgba(248, 113, 113, 0.12);
  border-color: rgba(248, 113, 113, 0.4);
}

.btn-apply {
  width: 100%;
  text-align: center;
  font-size: 14px;
  font-weight: 700;
  padding: 12px;
  border-radius: var(--r-md);
  background: var(--primary);
  color: #fff;
  cursor: pointer;
  box-shadow: var(--sh-md);
  transition: background var(--dur-fast) var(--ease),
    transform var(--dur-fast) var(--ease);
}
.btn-apply:hover {
  background: var(--primary-hover);
  transform: translateY(-1px) scale(1.01);
}
.btn-apply:active {
  transform: scale(0.97);
}

.d-actions-extra {
  display: flex;
  justify-content: center;
  margin-top: 8px;
}
.cancel-btn {
  font-size: 11px;
  color: var(--text-3);
  padding: 4px 12px;
  border-radius: var(--r-pill);
  background: transparent;
  transition: background var(--dur-fast) var(--ease), color var(--dur-fast) var(--ease);
}
.cancel-btn:hover {
  background: rgba(0, 0, 0, 0.05);
  color: var(--text);
}

.d-note {
  font-size: 11px;
  color: var(--text-3);
  margin-top: 12px;
  text-align: center;
}
</style>
