<script setup lang="ts">
import { ref } from "vue";
import type { WallpaperItem } from "../data/catalog";

defineProps<{ current: WallpaperItem | null }>();
const emit = defineEmits<{ (e: "import"): void }>();

const playing = ref(false);
const volume = ref(70);
const muted = ref(false);
const loop = ref<"single" | "list" | "random">("list");

function togglePlay() {
  playing.value = !playing.value;
}
function cycleLoop() {
  loop.value =
    loop.value === "list" ? "single" : loop.value === "single" ? "random" : "list";
}
const loopIcon = () => (loop.value === "single" ? "🔂" : loop.value === "random" ? "🔀" : "🔁");
</script>

<template>
  <footer
    class="flex h-14 flex-shrink-0 items-center gap-3 border-t border-[var(--border)] bg-[var(--bg-elevated)] px-4"
  >
    <div class="flex w-44 items-center gap-2 overflow-hidden">
      <div
        v-if="current"
        class="h-9 w-14 flex-shrink-0 rounded-md"
        :style="{ background: current.thumb }"
      />
      <div class="min-w-0">
        <div class="truncate text-[12px] font-medium text-[var(--text)]">
          {{ current ? current.name : "未选择壁纸" }}
        </div>
        <div class="text-[10px] text-[var(--text-dim)]">
          {{ current ? current.type.toUpperCase() : "—" }}
        </div>
      </div>
    </div>

    <div class="flex items-center gap-1.5">
      <button class="ctrl" title="上一个">⏮</button>
      <button
        class="ctrl h-9 w-9 bg-[var(--primary)] text-white hover:opacity-90"
        :title="playing ? '暂停' : '播放'"
        @click="togglePlay"
      >
        {{ playing ? "❚❚" : "▶" }}
      </button>
      <button class="ctrl" title="下一个">⏭</button>
    </div>

    <div class="flex items-center gap-1.5">
      <button class="ctrl" :title="muted ? '取消静音' : '静音'" @click="muted = !muted">
        {{ muted ? "🔇" : "🔊" }}
      </button>
      <input
        type="range" min="0" max="100" :value="volume"
        class="h-1 w-20 cursor-pointer appearance-none rounded-full bg-[var(--border)]"
        @input="(e) => (volume = +(e.target as HTMLInputElement).value)"
      />
      <button class="ctrl" :title="'循环：' + loop" @click="cycleLoop">{{ loopIcon() }}</button>
    </div>

    <button
      class="ml-auto rounded-xl border border-[var(--border)] px-3 py-1.5 text-[12px] text-[var(--text)] hover:bg-black/5"
      @click="emit('import')"
    >
      ＋ 导入
    </button>
  </footer>
</template>

<style scoped>
.ctrl {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 32px;
  width: 32px;
  border-radius: 999px;
  font-size: 13px;
  color: var(--text-dim);
  transition: background 0.15s, color 0.15s;
}
.ctrl:hover {
  background: rgba(31, 35, 41, 0.08);
  color: var(--text);
}
input[type="range"]::-webkit-slider-thumb {
  appearance: none;
  height: 12px;
  width: 12px;
  border-radius: 999px;
  background: var(--primary);
}
</style>
