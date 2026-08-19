import { createRouter, createWebHashHistory, type RouteRecordRaw } from "vue-router";
import WallpaperGrid from "../components/WallpaperGrid.vue";
import FavoritesView from "../components/FavoritesView.vue";
import LocalLibraryView from "../components/LocalLibraryView.vue";
import SettingsView from "../components/SettingsView.vue";
import AboutView from "../views/AboutView.vue";
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
  "favorite",
  "settings",
  "about",
] as const;

export type NavRouteName = (typeof NAV_ROUTE_NAMES)[number];

const routes: RouteRecordRaw[] = [
  { path: "/", redirect: { name: "local" } },
  {
    path: "/local",
    name: "local",
    component: LocalLibraryView,
    meta: { showDrawer: true },
  },
  {
    path: "/online",
    name: "online",
    component: WallpaperGrid,
    meta: { showDrawer: true, requiresOnline: true },
  },
  {
    path: "/favorite",
    name: "favorite",
    component: FavoritesView,
    meta: { showDrawer: true },
  },
  {
    path: "/settings",
    name: "settings",
    component: SettingsView,
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
