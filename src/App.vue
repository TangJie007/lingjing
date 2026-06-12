<script setup lang="ts">
// 灵境 App 壳 — 标题栏 + 侧边栏 + 内容区 + 状态栏
import { onMounted } from 'vue'
import { useAppStore } from '@/stores/app'
import AppTitlebar from '@/components/layout/AppTitlebar.vue'
import AppSidebar from '@/components/layout/AppSidebar.vue'
import AppStatusbar from '@/components/layout/AppStatusbar.vue'

const appStore = useAppStore()

onMounted(() => {
  appStore.loadSavedLocale()
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
