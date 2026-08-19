<script setup lang="ts">
import { computed, inject, nextTick, onMounted, ref, type Ref } from "vue";
import { CATALOG, CATEGORIES, SORTS, type WallpaperItem } from "../data/catalog";

const activeCat = ref<string>("全部");
const sort = ref<string>("最热");
// Top-bar search input drives this filter (A4). Falls back to local ref when
// App does not provide (e.g. unit tests).
const search = inject<Ref<string>>("topbarSearch", ref(""));

// Skeleton placeholders (A19) — render a brief grid of placeholder cards while
// the real list mounts, then swap them out. Combined with the staggered
// `animate-card-in` on real cards this produces the "staggered upward float
// with skeleton placeholders" entrance the spec calls for.
const ready = ref(false);
onMounted(() => nextTick(() => (ready.value = true)));

const emit = defineEmits<{
  (e: "select", item: WallpaperItem): void;
  (e: "set", item: WallpaperItem): void;
  (e: "context", item: WallpaperItem, x: number, y: number): void;
}>();

const list = computed(() => {
  let r = CATALOG;
  if (activeCat.value !== "全部") r = r.filter((i) => i.category === activeCat.value);
  if (search.value.trim()) {
    const q = search.value.trim().toLowerCase();
    r = r.filter((i) => i.name.toLowerCase().includes(q));
  }
  return r;
});

function pick(item: WallpaperItem) {
  emit("select", item);
}
function setWp(item: WallpaperItem) {
  emit("set", item);
}
function onContext(item: WallpaperItem, e: MouseEvent) {
  e.preventDefault();
  emit("context", item, e.clientX, e.clientY);
}
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col">
    <!-- 分类标签云 + 排序 -->
    <div class="flex flex-wrap items-center gap-1.5 px-4 pb-2 pt-3">
      <button
        v-for="c in CATEGORIES"
        :key="c"
        class="rounded-full px-3 py-1 text-[12px] font-medium transition-colors duration-[var(--dur-fast)]"
        :class="
          activeCat === c
            ? 'bg-[var(--primary)] text-white'
            : 'bg-[var(--bg-elevated)] text-[var(--text-dim)] hover:text-[var(--text)]'
        "
        @click="activeCat = c"
      >
        {{ c }}
      </button>
      <div class="ml-auto flex items-center gap-1 text-[12px] text-[var(--text-dim)]">
        <button
          v-for="s in SORTS"
          :key="s"
          class="rounded-lg px-2 py-0.5 transition-colors"
          :class="sort === s ? 'bg-[var(--primary-soft)] text-[var(--primary)]' : 'hover:text-[var(--text)]'"
          @click="sort = s"
        >
          {{ s }}
        </button>
      </div>
    </div>

    <!-- 卡片网格 -->
    <div class="grid min-h-0 flex-1 content-start gap-3.5 overflow-y-auto px-4 pb-4"
      style="grid-template-columns: repeat(auto-fill, minmax(150px, 1fr))">
      <!-- A19: skeleton placeholders while the catalog mounts -->
      <template v-if="!ready">
        <div
          v-for="i in 12"
          :key="`sk-${i}`"
          class="skeleton-card overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--surface)]"
        >
          <div class="aspect-[16/10] w-full bg-[var(--border)]" />
          <div class="flex items-center justify-between gap-2 px-2.5 py-2">
            <span class="h-3 w-2/3 rounded-full bg-[var(--border)]" />
            <span class="h-2.5 w-8 rounded-full bg-[var(--border)]" />
          </div>
        </div>
      </template>

      <div
        v-for="(item, idx) in list"
        v-else
        :key="item.id"
        class="card group relative cursor-pointer overflow-hidden rounded-xl border border-[var(--border)] bg-[var(--surface)] shadow-sm transition-transform duration-[var(--dur-base)] hover:-translate-y-1 hover:shadow-md animate-card-in"
        :style="{ animationDelay: idx * 28 + 'ms' }"
        @click="pick(item)"
        @contextmenu="onContext(item, $event)"
      >
        <div class="aspect-[16/10] w-full" :style="{ background: item.thumb }" />
        <div class="flex items-center justify-between gap-2 px-2.5 py-2">
          <span class="truncate text-[12px] font-medium text-[var(--text)]">{{ item.name }}</span>
          <span class="shrink-0 text-[10px] uppercase text-[var(--text-dim)]">{{ item.type }}</span>
        </div>
        <span class="absolute left-2 top-2 rounded-full bg-black/35 px-2 py-0.5 text-[10px] text-white backdrop-blur">
          {{ item.size }}
        </span>
        <span v-if="item.favorite" class="absolute right-2 top-2 text-[12px]">❤️</span>

        <!-- hover 快动作 -->
        <div
          class="quick-actions absolute inset-x-0 bottom-0 flex translate-y-full items-center justify-center gap-2 bg-gradient-to-t from-black/55 to-transparent p-2.5 transition-transform duration-[var(--dur-base)] group-hover:translate-y-0"
        >
          <button
            class="rounded-lg bg-white/90 px-3 py-1 text-[12px] font-medium text-[var(--text)] hover:bg-white"
            @click.stop="pick(item)"
          >
            ▶ 预览
          </button>
          <button
            class="rounded-lg bg-[var(--primary)] px-3 py-1 text-[12px] font-medium text-white hover:opacity-90"
            @click.stop="setWp(item)"
          >
            设为壁纸
          </button>
        </div>
      </div>

      <p v-if="ready && list.length === 0" class="col-span-full py-10 text-center text-sm text-[var(--text-dim)]">
        没有匹配的壁纸
      </p>
    </div>
  </div>
</template>

<style scoped>
.card {
  app-region: no-drag;
}
.quick-actions {
  app-region: no-drag;
}
</style>
