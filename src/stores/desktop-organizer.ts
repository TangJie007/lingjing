// 桌面整理 — 一键整理
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'

export interface OrganizeDesktopResult {
  arranged: number
  skipped: number
  iconCount: number
}

export const useDesktopOrganizerStore = defineStore('desktopOrganizer', () => {
  const organizing = ref(false)
  const lastResult = ref<OrganizeDesktopResult | null>(null)

  async function organizeDesktop(): Promise<OrganizeDesktopResult> {
    if (!isTauri()) throw new Error('Tauri API unavailable')
    organizing.value = true
    try {
      const result = await invoke<OrganizeDesktopResult>('organize_desktop_one_click')
      lastResult.value = result
      return result
    } finally {
      organizing.value = false
    }
  }

  return {
    organizing,
    lastResult,
    organizeDesktop,
  }
})
