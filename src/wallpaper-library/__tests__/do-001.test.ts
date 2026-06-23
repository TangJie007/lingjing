// AC 验收测试：桌面整理（一键归类） (DO-001)
// 按 wetspec 规范：describe('DO-001') → describe('AC-xxx: ...') → it(...)
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { createI18n } from 'vue-i18n'
import zhCN from '@/i18n/locales/zh-CN.json'
import { useDesktopOrganizerStore } from '@/stores/desktop-organizer'
import DesktopOrganizerPage from '@/views/DesktopOrganizerPage.vue'

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: invokeMock,
  isTauri: vi.fn(() => true),
}))

const i18n = createI18n({
  legacy: false,
  locale: 'zh-CN',
  messages: { 'zh-CN': zhCN },
})

function mountPage() {
  return mount(DesktopOrganizerPage, {
    global: { plugins: [createPinia(), i18n] },
  })
}

beforeEach(() => {
  setActivePinia(createPinia())
  invokeMock.mockReset()
})

describe('DO-001', () => {
  describe('AC-001: 一键整理', () => {
    it('应渲染一键整理按钮', () => {
      const wrapper = mountPage()
      expect(wrapper.find('.organizer-btn').exists()).toBe(true)
    })

    it('organizeDesktop 应调用后端命令并返回结果', async () => {
      invokeMock.mockResolvedValue({ arranged: 12, skipped: 0, iconCount: 12 })
      const store = useDesktopOrganizerStore()
      const result = await store.organizeDesktop()
      expect(invokeMock).toHaveBeenCalledWith('organize_desktop_one_click')
      expect(result.arranged).toBe(12)
      expect(store.lastResult?.iconCount).toBe(12)
    })
  })

  describe('AC-007: 整理结果反馈', () => {
    it('整理进行中 organizing 为 true，完成后恢复 false', async () => {
      invokeMock.mockResolvedValue({ arranged: 1, skipped: 0, iconCount: 1 })
      const store = useDesktopOrganizerStore()
      expect(store.organizing).toBe(false)
      const pending = store.organizeDesktop()
      expect(store.organizing).toBe(true)
      await pending
      expect(store.organizing).toBe(false)
    })

    it('非 Tauri 环境应抛出错误', async () => {
      const core = await import('@tauri-apps/api/core')
      vi.mocked(core.isTauri).mockReturnValueOnce(false)
      const store = useDesktopOrganizerStore()
      await expect(store.organizeDesktop()).rejects.toThrow()
    })
  })
})
