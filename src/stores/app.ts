// 全局应用状态
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'

export const useAppStore = defineStore('app', () => {
  const sidebarCollapsed = ref(false)
  const onboardingComplete = ref(false)
  const { locale } = useI18n()

  const currentLocale = computed(() => locale.value)

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

  return {
    sidebarCollapsed,
    onboardingComplete,
    currentLocale,
    setLocale,
    loadSavedLocale,
  }
})
