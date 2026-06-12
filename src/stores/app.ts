// 全局应用状态
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'

export const useAppStore = defineStore('app', () => {
  const sidebarCollapsed = ref(false)
  const onboardingComplete = ref(
    localStorage.getItem('lingscape-onboarding-complete') === 'true'
  )
  const { locale } = useI18n()

  const currentLocale = computed(() => locale.value)

  // 是否首次启动
  const isFirstLaunch = computed(() => !onboardingComplete.value)

  function setLocale(lang: string) {
    locale.value = lang
    localStorage.setItem('lingscape-locale', lang)
  }

  function loadSavedLocale() {
    const saved = localStorage.getItem('lingscape-locale')
    if (saved) {
      locale.value = saved
    }
  }

  // 标记向导已完成
  function completeOnboarding() {
    onboardingComplete.value = true
    localStorage.setItem('lingscape-onboarding-complete', 'true')
  }

  return {
    sidebarCollapsed,
    onboardingComplete,
    isFirstLaunch,
    currentLocale,
    setLocale,
    loadSavedLocale,
    completeOnboarding,
  }
})
