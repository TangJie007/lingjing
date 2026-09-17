<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { NAV_ROUTE_NAMES, type NavRouteName } from "../router";
import { NAV_FREQ, playClick } from "../composables/useAudio";
import iconOnline from "../assets/svg/online.svg";
import iconLocal from "../assets/svg/local.svg";
import iconPat from "../assets/svg/pat.svg";
import iconSetting from "../assets/svg/setting.svg";
import iconAbout from "../assets/svg/about.svg";

interface NavItem {
  key: NavRouteName;
  label: string;
  aria: string;
  icon: string;
}

const route = useRoute();
const router = useRouter();

const activeKey = computed((): NavRouteName => {
  const name = route.name;
  if (name === "wallpaper-detail") {
    const nav = route.query.nav;
    if (
      typeof nav === "string" &&
      (NAV_ROUTE_NAMES as readonly string[]).includes(nav)
    ) {
      return nav as NavRouteName;
    }
    return "local";
  }
  if (typeof name === "string" && (NAV_ROUTE_NAMES as readonly string[]).includes(name)) {
    return name as NavRouteName;
  }
  return "local";
});

const items: NavItem[] = [
  { key: "online", label: "在线", aria: "在线资源与我的收藏", icon: iconOnline },
  { key: "local", label: "本地", aria: "本地资源", icon: iconLocal },
  { key: "pet", label: "桌宠", aria: "桌宠管理", icon: iconPat },
  { key: "settings", label: "设置", aria: "设置", icon: iconSetting },
  { key: "about", label: "关于", aria: "关于", icon: iconAbout },
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
        v-if="it.key === 'about'"
        class="nav-spacer"
        aria-hidden="true"
      />
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
        <img
          class="nav-icon"
          :src="it.icon"
          alt=""
          draggable="false"
          aria-hidden="true"
        />
        <span>{{ it.label }}</span>
      </div>
    </template>
  </div>
</template>
