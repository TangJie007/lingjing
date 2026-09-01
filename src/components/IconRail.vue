<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { NAV_ROUTE_NAMES, type NavRouteName } from "../router";
import { NAV_FREQ, playClick } from "../composables/useAudio";

interface NavItem {
  key: NavRouteName;
  label: string;
  aria: string;
}

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
  { key: "online", label: "在线", aria: "在线资源与我的收藏" },
  { key: "local", label: "本地", aria: "本地资源" },
  { key: "pet", label: "桌宠", aria: "桌宠管理" },
  { key: "settings", label: "设置", aria: "设置" },
  { key: "about", label: "关于", aria: "关于" },
];

const indicator = ref<HTMLElement | null>(null);
const rail = ref<HTMLElement | null>(null);

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
</script>

<template>
  <div ref="rail" class="sidebar" role="navigation" aria-label="主导航">
    <div ref="indicator" class="nav-indicator" aria-hidden="true" />
    <template v-for="it in items" :key="it.key">
      <div v-if="it.key === 'settings'" class="nav-sep" />
      <div
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
        <svg v-else-if="it.key === 'pet'" viewBox="0 0 24 24" aria-hidden="true">
          <circle cx="7.5" cy="8" r="2.2" />
          <circle cx="16.5" cy="8" r="2.2" />
          <circle cx="5" cy="13.5" r="2" />
          <circle cx="19" cy="13.5" r="2" />
          <ellipse cx="12" cy="16.5" rx="4.2" ry="3.4" />
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
      </div>
    </template>
  </div>
</template>
