// ============================================================
// 灵境 Vue Router 配置
// Hash 模式（Tauri 无服务端）
// ============================================================
import { createRouter, createWebHashHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    redirect: '/library',
  },
  {
    path: '/library',
    name: 'Library',
    component: () => import('@/views/LibraryPage.vue'),
  },
  {
    path: '/settings',
    name: 'Settings',
    component: () => import('@/views/SettingsPage.vue'),
  },
  {
    path: '/ai-create',
    redirect: { path: '/library', query: { ai: 'open' } },
  },
  {
    path: '/api-keys',
    redirect: '/settings',
  },
  {
    path: '/onboarding',
    name: 'Onboarding',
    component: () => import('@/views/OnboardingPage.vue'),
  },
  {
    path: '/desktop-player',
    name: 'DesktopPlayer',
    component: () => import('@/views/DesktopPlayer.vue'),
  },
  {
    path: '/wallpaper/:id',
    name: 'WallpaperDetail',
    component: () => import('@/views/WallpaperDetailPage.vue'),
  },
  {
    path: '/desktop-organizer',
    name: 'DesktopOrganizer',
    component: () => import('@/views/DesktopOrganizerPage.vue'),
  },
  {
    path: '/fence-overlay',
    name: 'FenceOverlay',
    component: () => import('@/views/FenceOverlayView.vue'),
  },
]

const router = createRouter({
  history: createWebHashHistory(),
  routes,
})

export default router
