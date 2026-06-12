<script setup lang="ts">
// 灵境 App 壳 — 标题栏 + 侧边栏 + 内容区 + 状态栏
import { onMounted, ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAppStore } from '@/stores/app'
import { useTray } from '@/composables/useTray'
import { useWindow } from '@/composables/useWindow'
import AppTitlebar from '@/components/layout/AppTitlebar.vue'
import AppSidebar from '@/components/layout/AppSidebar.vue'
import AppStatusbar from '@/components/layout/AppStatusbar.vue'
import Toast from '@/components/common/Toast.vue'

const appStore = useAppStore()
const router = useRouter()
const route = useRoute()
const toastRef = ref<InstanceType<typeof Toast>>()

// 系统托盘事件监听 (ST-001)
useTray((action) => {
  switch (action) {
    case 'pause':
      toastRef.value?.show('info', '壁纸播放控制将在下一阶段实现')
      break
    case 'next':
      toastRef.value?.show('info', '壁纸切换将在下一阶段实现')
      break
  }
})

// 窗口关闭时最小化到托盘
useWindow()

onMounted(() => {
  appStore.loadSavedLocale()

  // 首次启动 → 跳转向导页 (OB-001)
  if (appStore.isFirstLaunch && route.path !== '/onboarding') {
    router.replace('/onboarding')
  }
})
</script>

<template>
  <div class="app-shell">
    <AppTitlebar />
    <div class="main-layout">
      <AppSidebar />
      <div class="content-area">
        <div class="content-scroll">
          <router-view />
        </div>
        <AppStatusbar />
      </div>
    </div>
    <Toast ref="toastRef" />
  </div>
</template>

<style scoped>
.app-shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  position: relative;
}
.main-layout {
  display: flex;
  flex: 1;
  overflow: hidden;
}
.content-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: transparent;
}
.content-scroll {
  flex: 1;
  overflow-y: auto;
  padding: calc(var(--titlebar-height) + var(--spacing-4)) var(--spacing-8) var(--spacing-6);
}
</style>
