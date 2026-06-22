// 桌面整理 — 一键整理
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'

export interface OrganizeDesktopResult {
  arranged: number
  skipped: number
  iconCount: number
  partitions: { name: string; iconCount: number }[]
}

export const useDesktopOrganizerStore = defineStore('desktopOrganizer', () => {
  const organizing = ref(false)
  const lastResult = ref<OrganizeDesktopResult | null>(null)
  const hasFences = ref(false)

  async function organizeDesktop(): Promise<OrganizeDesktopResult> {
    if (!isTauri()) throw new Error('Tauri API unavailable')
    organizing.value = true
    try {
      const result = await invoke<OrganizeDesktopResult>('organize_desktop_one_click')
      lastResult.value = result
      hasFences.value = (result.fences?.length ?? 0) > 0
      return result
    } finally {
      organizing.value = false
    }
  }

  async function clearFences() {
    if (!isTauri()) return
    await invoke('clear_fence_overlay')
    hasFences.value = false
  }

  return {
    organizing,
    lastResult,
    hasFences,
    organizeDesktop,
    clearFences,
  }
})
