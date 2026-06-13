<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAppStore } from '@/stores/app'
import { useTray } from '@/composables/useTray'
import { useWindow } from '@/composables/useWindow'
import HazyBackground from '@/components/layout/HazyBackground.vue'
import AppTitlebar from '@/components/layout/AppTitlebar.vue'
import FabCreate from '@/components/layout/FabCreate.vue'
import SearchOverlay from '@/components/layout/SearchOverlay.vue'
import AiCreatePanel from '@/components/ai/AiCreatePanel.vue'
import Toast from '@/components/common/Toast.vue'

const appStore = useAppStore()
const router = useRouter()
const route = useRoute()
const toastRef = ref<InstanceType<typeof Toast>>()

const searchOpen = ref(false)
const aiPanelOpen = ref(false)
const contentScrollRef = ref<HTMLElement | null>(null)
const contentScrolled = ref(false)

function onContentScroll() {
  contentScrolled.value = (contentScrollRef.value?.scrollTop ?? 0) > 2
}

const isOnboarding = computed(() => route.path === '/onboarding')
const showChrome = computed(() => !isOnboarding.value)

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

useWindow()

function toggleAiPanel() {
  aiPanelOpen.value = !aiPanelOpen.value
}

watch(
  () => route.path,
  () => nextTick(onContentScroll)
)

watch(
  () => route.query.ai,
  (value) => {
    if (value === 'open') {
      aiPanelOpen.value = true
      router.replace({ path: route.path, query: {} })
    }
  },
  { immediate: true }
)

onMounted(() => {
  appStore.loadSavedLocale()

  if (appStore.isFirstLaunch && route.path !== '/onboarding') {
    router.replace('/onboarding')
  }
})
</script>

<template>
  <HazyBackground v-if="showChrome" />

  <template v-if="isOnboarding">
    <router-view />
  </template>

  <div v-else class="app-shell">
    <AppTitlebar :scrolled="contentScrolled" @toggle-search="searchOpen = !searchOpen" />
    <SearchOverlay v-model:open="searchOpen" />
    <div class="main-layout">
      <div class="content-area">
        <div ref="contentScrollRef" class="content-scroll" @scroll="onContentScroll">
          <router-view />
        </div>
      </div>
    </div>

    <FabCreate :active="aiPanelOpen" @click="toggleAiPanel" />
    <AiCreatePanel v-model:open="aiPanelOpen" />
    <Toast ref="toastRef" />
  </div>
</template>
