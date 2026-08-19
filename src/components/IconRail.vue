<script setup lang="ts">
import { nextTick, onMounted, ref } from "vue";
import { NAV_FREQ, playClick } from "../composables/useAudio";

interface NavItem {
  key: string;
  label: string;
  badge?: boolean;
}

const props = defineProps<{ active: string }>();
const emit = defineEmits<{ (e: "nav", key: string): void }>();

const items: NavItem[] = [
  { key: "online", label: "在线" },
  { key: "local", label: "本地" },
  { key: "favorite", label: "我的", badge: true },
  { key: "settings", label: "设置" },
  { key: "about", label: "关于" },
];

const indicator = ref<HTMLElement | null>(null);

function moveIndicator(el: HTMLElement) {
  if (!indicator.value) return;
  const top = el.offsetTop;
  const h = el.offsetHeight;
  indicator.value.style.setProperty("--iy", `${top + h * 0.23}px`);
  indicator.value.style.setProperty("--ih", `${h * 0.54}px`);
}

function fire(n: NavItem, el: HTMLElement, ev?: MouseEvent) {
  const r = el.getBoundingClientRect();
  const cx = (ev?.clientX ?? r.left + r.width / 2) - r.left;
  const cy = (ev?.clientY ?? r.top + r.height / 2) - r.top;
  el.style.setProperty("--rx", `${cx}px`);
  el.style.setProperty("--ry", `${cy}px`);

  // 涟漪
  el.classList.remove("ripple");
  void el.offsetWidth;
  el.classList.add("ripple");
  setTimeout(() => el.classList.remove("ripple"), 560);

  // 弹簧回弹
  el.classList.remove("pop");
  void el.offsetWidth;
  el.classList.add("pop");
  setTimeout(() => el.classList.remove("pop"), 300);

  // 专属动效
  if (n.key === "settings") {
    el.classList.remove("spin");
    void el.offsetWidth;
    el.classList.add("spin");
    setTimeout(() => el.classList.remove("spin"), 640);
  }
  if (n.key === "favorite") {
    el.classList.remove("beat");
    void el.offsetWidth;
    el.classList.add("beat");
    setTimeout(() => el.classList.remove("beat"), 760);
  }

  emit("nav", n.key);
  nextTick(() => moveIndicator(el));
  playClick(NAV_FREQ[n.label] ?? 660);
}

function onKey(n: NavItem, el: HTMLElement, e: KeyboardEvent) {
  if (e.key === "Enter" || e.key === " ") {
    e.preventDefault();
    fire(n, el);
  }
}

onMounted(async () => {
  await nextTick();
  const first = document.querySelector<HTMLElement>(`.nav-item[data-nav="${props.active}"]`);
  if (first) moveIndicator(first);
});
</script>

<template>
  <aside
    class="sidebar flex w-20 flex-shrink-0 flex-col items-center gap-1 border-r border-[var(--border)] bg-[var(--surface)] py-3"
    role="navigation"
    aria-label="主导航"
  >
    <div
      ref="indicator"
      class="nav-indicator"
      aria-hidden="true"
    />
    <div class="mb-3 flex h-9 w-9 items-center justify-center rounded-xl bg-[var(--primary-soft)] text-lg">
      🌌
    </div>
    <button
      v-for="it in items"
      :key="it.key"
      :data-nav="it.label"
      :class="['nav-item', 'group', 'relative', 'flex', 'w-14', 'flex-col', 'items-center', 'gap-1', 'rounded-xl', 'py-2', props.active === it.key ? 'active' : '']"
      role="button"
      tabindex="0"
      :aria-label="it.label"
      @click="(e) => fire(it, e.currentTarget as HTMLElement, e)"
      @keydown="(e) => onKey(it, e.currentTarget as HTMLElement, e)"
    >
      <span class="nav-icon" :data-icon="it.key">
        <!-- 在线：4 圆角矩形 grid -->
        <svg v-if="it.key === 'online'" viewBox="0 0 24 24" aria-hidden="true">
          <rect x="3" y="3" width="7" height="7" rx="1.5" />
          <rect x="14" y="3" width="7" height="7" rx="1.5" />
          <rect x="3" y="14" width="7" height="7" rx="1.5" />
          <rect x="14" y="14" width="7" height="7" rx="1.5" />
        </svg>
        <!-- 本地：文件夹 -->
        <svg v-else-if="it.key === 'local'" viewBox="0 0 24 24" aria-hidden="true">
          <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
        </svg>
        <!-- 我的：心形 -->
        <svg v-else-if="it.key === 'favorite'" viewBox="0 0 24 24" aria-hidden="true">
          <path d="M12 20s-7-4.5-9.5-9C1 8 2.5 4.5 6 4.5c2 0 3.2 1.2 4 2.3.8-1.1 2-2.3 4-2.3 3.5 0 5 3.5 3.5 6.5C19 15.5 12 20 12 20z" />
        </svg>
        <!-- 设置：齿轮 -->
        <svg v-else-if="it.key === 'settings'" viewBox="0 0 24 24" aria-hidden="true">
          <circle cx="12" cy="12" r="3" />
          <path d="M12 2v3M12 19v3M2 12h3M19 12h3M5 5l2 2M17 17l2 2M19 5l-2 2M7 17l-2 2" />
        </svg>
        <!-- 关于：信息圆 -->
        <svg v-else viewBox="0 0 24 24" aria-hidden="true">
          <circle cx="12" cy="12" r="9" />
          <path d="M12 11v5M12 8h.01" />
        </svg>
      </span>
      <span class="text-[10px] font-medium">{{ it.label }}</span>
      <span v-if="it.badge" class="badge-dot" />
    </button>
  </aside>
</template>

<style scoped>
.sidebar {
  position: relative;
}

/* 跨项 3px 滑动指示条 */
.nav-indicator {
  position: absolute;
  left: 0;
  top: 0;
  width: 3px;
  height: var(--ih, 30px);
  border-radius: 0 3px 3px 0;
  background: var(--primary);
  transform: translateY(var(--iy, 16px));
  transition: transform var(--dur-base) var(--ease-spring), height var(--dur-base) var(--ease-spring);
  will-change: transform;
  pointer-events: none;
}

.nav-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 5px;
  padding: 11px 4px;
  border-radius: var(--r-md);
  font-size: 10.5px;
  color: var(--text-2);
  cursor: pointer;
  position: relative;
  overflow: hidden;
  transition: background var(--dur-fast) var(--ease), color var(--dur-fast) var(--ease),
    transform var(--dur-fast) var(--ease-spring);
  app-region: no-drag;
}
.nav-item:hover {
  background: var(--surface-2);
  color: var(--text);
}
.nav-item:active {
  transform: scale(0.95);
}
.nav-item.active {
  background: var(--primary-soft);
  color: var(--primary);
  font-weight: 600;
}

.nav-item .nav-icon {
  width: 23px;
  height: 23px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transform-origin: center;
  will-change: transform;
  transition: transform var(--dur-base) var(--ease-spring);
}
.nav-item .nav-icon svg {
  width: 100%;
  height: 100%;
  stroke: currentColor;
  fill: none;
  stroke-width: 1.8;
  stroke-linecap: round;
  stroke-linejoin: round;
}
.nav-item:hover .nav-icon {
  transform: translateY(-1px) scale(1.08);
}

/* 点击触发的动效类 */
.nav-item.ripple::after {
  content: "";
  position: absolute;
  left: var(--rx, 50%);
  top: var(--ry, 50%);
  width: 66px;
  height: 66px;
  margin: -33px 0 0 -33px;
  border-radius: 50%;
  background: radial-gradient(circle, currentColor 0%, transparent 68%);
  opacity: 0;
  pointer-events: none;
  animation: navRipple 380ms var(--ease-out);
}
.nav-item.pop .nav-icon {
  animation: navPop var(--dur-base) var(--ease-spring);
}
.nav-item.spin .nav-icon svg {
  animation: gearSpin 0.6s var(--ease);
}
.nav-item.beat .nav-icon svg {
  animation: heartBeat 0.72s var(--ease);
}

/* 我的红点 badge */
.badge-dot {
  position: absolute;
  top: 8px;
  right: 18px;
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--error);
  animation: pulse 2s var(--ease) infinite;
}
</style>
