<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useWeather } from "../useWeather";

const props = defineProps<{
  fencesCollapsed: boolean;
  itemCount?: number;
  dragging?: boolean;
}>();

const emit = defineEmits<{
  toggleFences: [];
  openSettings: [];
  disableOrganize: [];
}>();

const { weather, weatherLoading, refreshWeather } = useWeather();

const expanded = ref(false);
const activity = ref(false);
const activityLabel = ref("");
const now = ref(new Date());
let collapseTimer: number | null = null;
let activityTimer: number | null = null;
let clockTimer: number | null = null;

const clockText = computed(() => {
  const d = now.value;
  const hh = String(d.getHours()).padStart(2, "0");
  const mm = String(d.getMinutes()).padStart(2, "0");
  return `${hh}:${mm}`;
});

const weekdayText = computed(() => {
  const labels = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];
  return labels[now.value.getDay()] ?? "周一";
});

const weatherCompact = computed(() => {
  const w = weather.value;
  if (!w) return weatherLoading.value ? "天气…" : "";
  return `${w.icon} ${w.tempC}°`;
});

const compactStatus = computed(() => {
  if (props.dragging) return "移动中";
  if (activity.value && activityLabel.value) return activityLabel.value;
  if (weatherCompact.value) return weatherCompact.value;
  if (props.fencesCollapsed) return "已收起";
  return "整理中";
});

function clearCollapseTimer() {
  if (collapseTimer != null) {
    window.clearTimeout(collapseTimer);
    collapseTimer = null;
  }
}

function clearActivityTimer() {
  if (activityTimer != null) {
    window.clearTimeout(activityTimer);
    activityTimer = null;
  }
}

function scheduleCollapse(ms = 5200) {
  clearCollapseTimer();
  collapseTimer = window.setTimeout(() => {
    expanded.value = false;
    collapseTimer = null;
  }, ms);
}

function flashActivity(label: string, ms = 2200) {
  if (expanded.value) return;
  activityLabel.value = label;
  activity.value = true;
  clearActivityTimer();
  activityTimer = window.setTimeout(() => {
    activity.value = false;
    activityLabel.value = "";
    activityTimer = null;
  }, ms);
}

function expand() {
  expanded.value = true;
  activity.value = false;
  scheduleCollapse();
  if (!weather.value) void refreshWeather();
}

function collapse() {
  expanded.value = false;
  clearCollapseTimer();
}

function onToggleFences() {
  emit("toggleFences");
  collapse();
}

function onOpenSettings() {
  emit("openSettings");
  collapse();
}

function onDisableOrganize() {
  emit("disableOrganize");
  collapse();
}

function onRefreshWeather() {
  void refreshWeather().then(() => {
    if (weather.value) {
      flashActivity(`${weather.value.icon} ${weather.value.tempC}° ${weather.value.label}`);
    }
  });
  scheduleCollapse();
}

function onShellPointerEnter() {
  if (expanded.value) clearCollapseTimer();
}

function onShellPointerLeave() {
  if (expanded.value) scheduleCollapse(2200);
}

function onDocPointerDown(e: PointerEvent) {
  const t = e.target as HTMLElement | null;
  if (!t?.closest?.(".dynamic-island")) collapse();
}

function onKey(e: KeyboardEvent) {
  if (e.key === "Escape") collapse();
}

watch(
  () => props.fencesCollapsed,
  (v) => flashActivity(v ? "围栏已收起" : "围栏已展开"),
);

watch(
  () => props.itemCount,
  (n, prev) => {
    if (prev == null || n == null || n === prev) return;
    flashActivity(n > prev ? `新增至 ${n} 项` : `现为 ${n} 项`);
  },
);

watch(
  () => props.dragging,
  (v) => {
    if (v) {
      activity.value = true;
      activityLabel.value = "移动中";
      clearActivityTimer();
    } else if (activityLabel.value === "移动中") {
      activity.value = false;
      activityLabel.value = "";
    }
  },
);

watch(weather, (w, prev) => {
  if (!w || !prev) return;
  if (w.tempC === prev.tempC && w.label === prev.label) return;
  flashActivity(`${w.icon} ${w.tempC}° ${w.label}`);
});

onMounted(() => {
  document.addEventListener("pointerdown", onDocPointerDown, true);
  window.addEventListener("keydown", onKey);
  clockTimer = window.setInterval(() => {
    now.value = new Date();
  }, 30_000);
});

onUnmounted(() => {
  document.removeEventListener("pointerdown", onDocPointerDown, true);
  window.removeEventListener("keydown", onKey);
  clearCollapseTimer();
  clearActivityTimer();
  if (clockTimer != null) window.clearInterval(clockTimer);
});
</script>

<template>
  <div
    class="dynamic-island no-drag"
    :class="{
      expanded,
      activity: activity || dragging,
      idle: !expanded && !activity && !dragging,
      'fences-hidden': fencesCollapsed,
      'has-weather': !!weather,
    }"
    role="toolbar"
    aria-label="灵镜桌面整理"
    @contextmenu.stop.prevent
  >
    <div
      class="island-shell"
      @pointerenter="onShellPointerEnter"
      @pointerleave="onShellPointerLeave"
    >
      <button
        v-show="!expanded"
        type="button"
        class="island-compact"
        :aria-expanded="false"
        @click="expand"
      >
        <span class="island-led" aria-hidden="true" />
        <span class="island-brand">{{ weekdayText }}</span>
        <span class="island-wave" aria-hidden="true">
          <i /><i /><i />
        </span>
        <span class="island-status">{{ compactStatus }}</span>
      </button>

      <div v-show="expanded" class="island-expanded" :aria-expanded="true">
        <div class="island-head">
          <span class="island-led" aria-hidden="true" />
          <div class="island-head-text">
            <div class="island-title">灵镜整理</div>
            <div class="island-sub">
              {{
                fencesCollapsed
                  ? "围栏已收起"
                  : itemCount != null
                    ? `桌面 ${itemCount} 项已分类`
                    : "桌面整理运行中"
              }}
            </div>
          </div>
          <div class="island-clock">{{ clockText }}</div>
          <button
            type="button"
            class="island-dismiss"
            aria-label="收起"
            @click="collapse"
          >
            ⌃
          </button>
        </div>

        <div class="island-weather" :class="{ empty: !weather }">
          <template v-if="weather">
            <div class="island-weather-main">
              <span class="island-weather-icon" aria-hidden="true">{{
                weather.icon
              }}</span>
              <div class="island-weather-temp">
                <span class="deg">{{ weather.tempC }}°</span>
                <span class="lbl">{{ weather.label }}</span>
              </div>
            </div>
            <div class="island-weather-meta">
              <span>{{ weather.city }}</span>
              <span>体感 {{ weather.feelsC }}°</span>
              <span>湿度 {{ weather.humidity }}%</span>
              <span>风 {{ weather.windKmh }} km/h</span>
            </div>
            <button
              type="button"
              class="island-weather-refresh"
              :disabled="weatherLoading"
              @click="onRefreshWeather"
            >
              {{ weatherLoading ? "更新中" : "刷新" }}
            </button>
          </template>
          <template v-else>
            <span class="island-weather-empty">{{
              weatherLoading ? "正在获取天气…" : "天气暂不可用"
            }}</span>
            <button
              type="button"
              class="island-weather-refresh"
              :disabled="weatherLoading"
              @click="onRefreshWeather"
            >
              重试
            </button>
          </template>
        </div>

        <div class="island-actions">
          <button type="button" class="island-action" @click="onToggleFences">
            <span class="island-action-ico" aria-hidden="true">{{
              fencesCollapsed ? "▢" : "▁"
            }}</span>
            <span>{{ fencesCollapsed ? "展开围栏" : "收起围栏" }}</span>
          </button>
          <button type="button" class="island-action" @click="onOpenSettings">
            <span class="island-action-ico" aria-hidden="true">⚙</span>
            <span>打开设置</span>
          </button>
          <button
            type="button"
            class="island-action danger"
            @click="onDisableOrganize"
          >
            <span class="island-action-ico" aria-hidden="true">✕</span>
            <span>关闭整理</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
