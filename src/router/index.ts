// ============================================================
// 灵境 Vue Router 配置
// Hash 模式（Tauri 无服务端）
// ============================================================
import { createRouter, createWebHashHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    redirect: '/ai-create',
  },
  {
    path: '/ai-create',
    name: 'AiCreate',
    component: () => import('@/views/AiCreatePage.vue'),
  },
  {
    path: '/library',
    name: 'Library',
    component: () => import('@/views/LibraryPage.vue'),
  },
  {
    path: '/api-keys',
    name: 'ApiKeys',
    component: () => import('@/views/ApiKeysPage.vue'),
  },
  {
    path: '/settings',
    name: 'Settings',
    component: () => import('@/views/SettingsPage.vue'),
  },
  {
    path: '/onboarding',
    name: 'Onboarding',
    component: () => import('@/views/OnboardingPage.vue'),
  },
]

const router = createRouter({
  history: createWebHashHistory(),
  routes,
})

export default router
