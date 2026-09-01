import { createRouter, createWebHashHistory, type RouteRecordRaw } from "vue-router";
import OnlineView from "../views/OnlineView.vue";
import LocalView from "../views/LocalView.vue";
import PetView from "../views/PetView.vue";
import SettingsView from "../views/SettingsView.vue";
import AboutView from "../views/AboutView.vue";
import WallpaperDetailView from "../views/WallpaperDetailView.vue";
import { useSettings } from "../composables/useSettings";

declare module "vue-router" {
  interface RouteMeta {
    showDrawer?: boolean;
    requiresOnline?: boolean;
  }
}

export const NAV_ROUTE_NAMES = [
  "online",
  "local",
  "pet",
  "settings",
  "about",
] as const;

export type NavRouteName = (typeof NAV_ROUTE_NAMES)[number];

const routes: RouteRecordRaw[] = [
  { path: "/", redirect: { name: "local" } },
  {
    path: "/local",
    name: "local",
    component: LocalView,
    meta: { showDrawer: true },
  },
  {
    path: "/online",
    name: "online",
    component: OnlineView,
    meta: { showDrawer: true },
  },
  {
    path: "/favorite",
    redirect: { name: "online", query: { tab: "mine" } },
  },
  {
    path: "/pet",
    name: "pet",
    component: PetView,
  },
  {
    path: "/settings",
    name: "settings",
    component: SettingsView,
  },
  {
    path: "/wallpaper/:id",
    name: "wallpaper-detail",
    component: WallpaperDetailView,
    meta: { showDrawer: false },
  },
  {
    path: "/about",
    name: "about",
    component: AboutView,
  },
  { path: "/:pathMatch(.*)*", redirect: { name: "local" } },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

router.beforeEach((to) => {
  if (!to.meta.requiresOnline) return true;
  const settings = useSettings();
  if (settings.value.onlineEnabled) return true;
  return { name: "local" };
});

export default router;
