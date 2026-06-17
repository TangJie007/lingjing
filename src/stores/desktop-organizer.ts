// 桌面整理状态管理 (DO-001~004)
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface PartitionInfo {
  id: string
  name: string
  x: number
  y: number
  w: number
  h: number
  color: string
  opacity: number
  collapsed: boolean
  iconCount: number
}

export interface DesktopIcon {
  name: string
  path: string
  x: number
  y: number
}

export const useDesktopOrganizerStore = defineStore('desktopOrganizer', () => {
  const partitions = ref<PartitionInfo[]>([])
  const icons = ref<DesktopIcon[]>([])
  const editing = ref(false)
  const iconsHidden = ref(false)

  const totalIcons = computed(() => icons.value?.length ?? 0)

  async function loadLayout() {
    try {
      const layout = await invoke<any>('get_partition_layout')
      if (layout?.partitions) {
        partitions.value = layout.partitions
      }
    } catch { /* 首次使用，无布局 */ }
  }

  async function saveLayout() {
    try {
      await invoke('save_partition_layout', { appData: '' })
    } catch (e) {
      console.error('保存分区布局失败:', e)
    }
  }

  async function enumerateIcons() {
    try {
      icons.value = await invoke<DesktopIcon[]>('enumerate_desktop_icons')
    } catch {
      icons.value = []
    }
  }

  async function createPartition(
    name: string, x: number, y: number, w: number, h: number,
    color = '#000000', opacity = 0.2
  ): Promise<PartitionInfo | null> {
    try {
      const p = await invoke<PartitionInfo>('create_partition', {
        name, x, y, w, h, color, opacity,
      })
      partitions.value.push(p)
      return p
    } catch (e) {
      console.error('创建分区失败:', e)
      return null
    }
  }

  async function deletePartition(id: string) {
    try {
      await invoke('delete_partition', { partitionId: id })
      partitions.value = partitions.value.filter((p) => p.id !== id)
    } catch (e) {
      console.error('删除分区失败:', e)
    }
  }

  async function updatePartition(id: string, updates: Partial<PartitionInfo>) {
    try {
      await invoke('update_partition', { partitionId: id, ...updates })
      const p = partitions.value.find((p) => p.id === id)
      if (p) Object.assign(p, updates)
    } catch (e) {
      console.error('更新分区失败:', e)
    }
  }

  async function moveIconToPartition(iconPath: string, partitionId: string) {
    try {
      await invoke('move_icon_to_partition', { iconPath, partitionId })
      const p = partitions.value.find((p) => p.id === partitionId)
      if (p) p.iconCount++
    } catch (e) {
      console.error('移动图标失败:', e)
    }
  }

  async function toggleIcons() {
    try {
      if (iconsHidden.value) {
        await invoke('show_desktop_icons')
      } else {
        await invoke('hide_desktop_icons')
      }
      iconsHidden.value = !iconsHidden.value
    } catch (e) {
      console.error('切换图标显示失败:', e)
    }
  }

  function toggleCollapse(id: string) {
    const p = partitions.value.find((p) => p.id === id)
    if (p) {
      p.collapsed = !p.collapsed
      updatePartition(id, { collapsed: p.collapsed })
    }
  }

  return {
    partitions,
    icons,
    editing,
    iconsHidden,
    totalIcons,
    loadLayout,
    saveLayout,
    enumerateIcons,
    createPartition,
    deletePartition,
    updatePartition,
    moveIconToPartition,
    toggleIcons,
    toggleCollapse,
  }
})
