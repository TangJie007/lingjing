<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { NAV_ROUTE_NAMES, type NavRouteName } from "../router";
import { NAV_FREQ, playClick } from "../composables/useAudio";

interface NavItem {
  key: NavRouteName;
  label: string;
  aria: string;
  badge?: boolean;
}

const props = withDefaults(
  defineProps<{ onlineEnabled?: boolean }>(),
  { onlineEnabled: false },
);

const route = useRoute();
const router = useRouter();

const activeKey = computed(() => {
  const name = route.name;
  if (typeof name === "string" && (NAV_ROUTE_NAMES as readonly string[]).includes(name)) {
    return name as NavRouteName;
  }
  return "local" as NavRouteName;
});

const items: NavItem[] = [
  { key: "online", label: "在线", aria: "在线资源" },
  { key: "local", label: "本地", aria: "本地资源" },
  { key: "favorite", label: "我的", aria: "我的收藏", badge: true },
  { key: "settings", label: "设置", aria: "设置" },
  { key: "about", label: "关于", aria: "关于" },
];

const indicator = ref<HTMLElement | null>(null);
const rail = ref<HTMLElement | null>(null);

function showItem(it: NavItem) {
  return it.key !== "online" || props.onlineEnabled;
}

function moveIndicator(el: HTMLElement) {
  if (!indicator.value) return;
  indicator.value.style.setProperty("--iy", `${el.offsetTop + el.offsetHeight * 0.23}px`);
  indicator.value.style.setProperty("--ih", `${el.offsetHeight * 0.54}px`);
}

function fire(n: NavItem, el: HTMLElement, ev?: MouseEvent) {
  const r = el.getBoundingClientRect();
  el.style.setProperty("--rx", `${(ev?.clientX ?? r.left + r.width / 2) - r.left}px`);
  el.style.setProperty("--ry", `${(ev?.clientY ?? r.top + r.height / 2) - r.top}px`);

  el.classList.remove("ripple");
  void el.offsetWidth;
  el.classList.add("ripple");
  setTimeout(() => el.classList.remove("ripple"), 560);

  el.classList.remove("pop");
  void el.offsetWidth;
  el.classList.add("pop");
  setTimeout(() => el.classList.remove("pop"), 300);

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

  if (n.key === "online" && !props.onlineEnabled) return;
  if (route.name !== n.key) {
    void router.push({ name: n.key });
  }
  nextTick(() => moveIndicator(el));
  playClick(NAV_FREQ[n.label] ?? 660);
}

function syncIndicator() {
  const el = rail.value?.querySelector<HTMLElement>(`.nav-item[data-nav-key="${activeKey.value}"]`);
  if (el) moveIndicator(el);
}

onMounted(() => nextTick(syncIndicator));
watch(activeKey, () => nextTick(syncIndicator));
watch(() => props.onlineEnabled, () => nextTick(syncIndicator));
</script>

<template>
  <div ref="rail" class="sidebar" role="navigation" aria-label="主导航">
    <div ref="indicator" class="nav-indicator" aria-hidden="true" />
    <template v-for="it in items" :key="it.key">
      <div v-if="it.key === 'settings'" class="nav-sep" />
      <div
        v-if="showItem(it)"
        class="nav-item"
        :class="{ active: activeKey === it.key }"
        :data-nav="it.label"
        :data-nav-key="it.key"
        role="button"
        tabindex="0"
        :aria-label="it.aria"
        :aria-current="activeKey === it.key ? 'page' : undefined"
        @click="(e) => fire(it, e.currentTarget as HTMLElement, e)"
        @keydown="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); fire(it, e.currentTarget as HTMLElement); } }"
      >
        <svg v-if="it.key === 'online'" viewBox="0 0 24 24" aria-hidden="true">
          <rect x="3" y="3" width="7" height="7" rx="1.5" />
          <rect x="14" y="3" width="7" height="7" rx="1.5" />
          <rect x="3" y="14" width="7" height="7" rx="1.5" />
          <rect x="14" y="14" width="7" height="7" rx="1.5" />
        </svg>
        <svg v-else-if="it.key === 'local'" viewBox="0 0 24 24" aria-hidden="true">
          <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
        </svg>
        <svg v-else-if="it.key === 'favorite'" viewBox="0 0 24 24" aria-hidden="true">
          <path d="M12 20s-7-4.5-9.5-9C1 8 2.5 4.5 6 4.5c2 0 3.2 1.2 4 2.3.8-1.1 2-2.3 4-2.3 3.5 0 5 3.5 3.5 6.5C19 15.5 12 20 12 20z" />
        </svg>
        <svg v-else-if="it.key === 'settings'" viewBox="0 0 24 24" aria-hidden="true">
          <circle cx="12" cy="12" r="3" />
          <path d="M12 2v3M12 19v3M2 12h3M19 12h3M5 5l2 2M17 17l2 2M19 5l-2 2M7 17l-2 2" />
        </svg>
        <svg v-else viewBox="0 0 24 24" aria-hidden="true">
          <circle cx="12" cy="12" r="9" />
          <path d="M12 11v5M12 8h.01" />
        </svg>
        <span>{{ it.label }}</span>
        <span v-if="it.badge" class="badge-dot" />
      </div>
    </template>
  </div>
</template>
