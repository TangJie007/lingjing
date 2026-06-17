// AC 验收测试：桌面分区整理 (DO-001)
// 按 wetspec 规范：describe('DO-001') → describe('AC-xxx: ...') → it(...)
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { createI18n } from 'vue-i18n'
import zhCN from '@/i18n/locales/zh-CN.json'
import { useDesktopOrganizerStore } from '@/stores/desktop-organizer'
import DesktopOrganizerPage from '@/views/DesktopOrganizerPage.vue'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(null),
  isTauri: vi.fn(() => false),
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
})

describe('DO-001', () => {
  describe('AC-003: 拖拽创建分区', () => {
    it('进入编辑模式后应显示模板按钮', async () => {
      const wrapper = mountPage()
      const store = useDesktopOrganizerStore()
      store.editing = true
      await wrapper.vm.$nextTick()
      expect(wrapper.find('.organizer-templates').exists()).toBe(true)
    })

    it('未进入编辑模式不应显示模板', () => {
      const wrapper = mountPage()
      expect(wrapper.find('.organizer-templates').exists()).toBe(false)
    })
  })

  describe('AC-004: 推荐模板创建', () => {
    it('应显示四个推荐模板按钮', async () => {
      const wrapper = mountPage()
      const store = useDesktopOrganizerStore()
      store.editing = true
      await wrapper.vm.$nextTick()
      const btns = wrapper.findAll('.template-btn')
      expect(btns.length).toBe(4)
    })
  })

  describe('AC-006: 卷起/展开', () => {
    it('toggleCollapse 应切换 collapsed 状态', () => {
      const store = useDesktopOrganizerStore()
      store.partitions = [{
        id: 'p1', name: '测试', x: 0, y: 0, w: 200, h: 200,
        color: '#000', opacity: 0.2, collapsed: false, iconCount: 0,
      }]
      store.toggleCollapse('p1')
      expect(store.partitions[0].collapsed).toBe(true)
      store.toggleCollapse('p1')
      expect(store.partitions[0].collapsed).toBe(false)
    })
  })

  describe('AC-008: 图标拖入分区', () => {
    it('moveIconToPartition 应增加图标计数', async () => {
      const store = useDesktopOrganizerStore()
      store.partitions = [{
        id: 'p1', name: '测试', x: 0, y: 0, w: 200, h: 200,
        color: '#000', opacity: 0.2, collapsed: false, iconCount: 0,
      }]
      await store.moveIconToPartition('/test/icon.png', 'p1')
      expect(store.partitions[0].iconCount).toBe(1)
    })
  })

  describe('AC-011: 删除分区不删文件', () => {
    it('deletePartition 应从列表中移除分区', async () => {
      const store = useDesktopOrganizerStore()
      store.partitions = [{
        id: 'p1', name: '测试', x: 0, y: 0, w: 200, h: 200,
        color: '#000', opacity: 0.2, collapsed: false, iconCount: 0,
      }]
      await store.deletePartition('p1')
      expect(store.partitions.length).toBe(0)
    })
  })

  describe('AC-001: 隐藏/显示图标', () => {
    it('toggleIcons 应切换 iconsHidden 状态', async () => {
      const store = useDesktopOrganizerStore()
      expect(store.iconsHidden).toBe(false)
      await store.toggleIcons()
      expect(store.iconsHidden).toBe(true)
      await store.toggleIcons()
      expect(store.iconsHidden).toBe(false)
    })
  })

  describe('AC-012: 启动恢复布局', () => {
    it('loadLayout 应能正常调用', async () => {
      const store = useDesktopOrganizerStore()
      await store.loadLayout()
      // 不应抛出异常
      expect(store.partitions).toBeDefined()
    })
  })
})
