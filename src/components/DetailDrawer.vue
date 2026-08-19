<script setup lang="ts">
import { ref, watch } from "vue";
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
// Public — bound to the 取消 button. Stops the auto-collapse timer and freezes
// the ring at its current value; the drawer stays open so the user can keep
// inspecting the wallpaper or trigger an action.
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

const dash = 2 * Math.PI * 26;
const offset = () => dash * (1 - remain.value / COUNTDOWN);

function setWp() {
  if (props.item) emit("set", props.item);
}
function fav() {
  if (props.item) emit("favorite", props.item);
}
</script>

<template>
  <transition name="drawer">
    <aside
      v-if="open && item"
      class="flex w-80 flex-shrink-0 flex-col border-l border-[var(--border)] bg-[var(--bg-elevated)]/80 backdrop-blur-xl"
    >
      <div class="flex items-center justify-between px-4 py-3">
        <span class="text-sm font-semibold text-[var(--text)]">壁纸详情</span>
        <button class="text-[var(--text-dim)] hover:text-[var(--text)]" @click="emit('close')">✕</button>
      </div>

      <div class="px-4">
        <div
          class="aspect-[16/10] w-full overflow-hidden rounded-xl border border-[var(--border)]"
          :style="{ background: item.thumb }"
        />
      </div>

      <div class="flex items-center justify-between px-4 py-2 text-[12px] text-[var(--text-dim)]">
        <span class="truncate font-medium text-[var(--text)]">{{ item.name }}</span>
        <span>{{ item.type.toUpperCase() }} · {{ item.size }}</span>
      </div>

      <!-- 60s 倒计时环 -->
      <div class="flex items-center gap-2 px-4 pb-2">
        <svg width="60" height="60" viewBox="0 0 60 60" class="-rotate-90">
          <circle cx="30" cy="30" r="26" fill="none" stroke="var(--border)" stroke-width="4" />
          <circle
            cx="30" cy="30" r="26" fill="none" stroke="var(--primary)" stroke-width="4"
            stroke-linecap="round" :stroke-dasharray="dash" :stroke-dashoffset="offset()"
          />
        </svg>
        <div class="flex-1 text-[11px] leading-tight text-[var(--text-dim)]">
          <div>预览将在</div>
          <div class="text-[var(--text)]">{{ remain }}s 后自动收起</div>
        </div>
        <button
          v-if="remain > 0"
          class="cancel-btn rounded-lg px-2 py-1 text-[11px] text-[var(--text-dim)] hover:bg-black/5 hover:text-[var(--text)]"
          title="取消自动收起"
          @click="cancelAutoClose"
        >
          取消
        </button>
      </div>

      <div class="mt-1 flex gap-2 px-4 pb-4">
        <button
          class="flex-1 rounded-xl bg-[var(--primary)] py-2 text-[13px] font-semibold text-white transition-transform duration-[var(--dur-fast)] hover:-translate-y-px"
          @click="setWp"
        >
          设为壁纸
        </button>
        <button
          class="rounded-xl border border-[var(--border)] px-3 py-2 text-[13px] text-[var(--text)] hover:bg-black/5"
          @click="fav"
        >
          {{ item.favorite ? "❤️" : "🤍" }}
        </button>
        <button
          class="rounded-xl border border-[var(--border)] px-3 py-2 text-[13px] text-[var(--text)] hover:bg-black/5"
        >
          ⤓
        </button>
        <button
          class="rounded-xl border border-[var(--border)] px-3 py-2 text-[13px] text-[var(--text)] hover:bg-black/5"
        >
          ↗
        </button>
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
</style>
