// 阶段一基础验证测试
import { describe, it, expect, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia } from 'pinia'
import { createI18n } from 'vue-i18n'
import zhCN from '@/i18n/locales/zh-CN.json'

const i18n = createI18n({
  legacy: false,
  locale: 'zh-CN',
  messages: { 'zh-CN': zhCN },
})

// mock useRoute 供 AppSidebar 中 isActive() 使用
vi.mock('vue-router', () => ({
  useRoute: () => ({ path: '/ai-create' }),
}))

import AppSidebar from '@/components/layout/AppSidebar.vue'

describe('阶段一：基础骨架', () => {
  it('AppSidebar 渲染 4 个导航项', () => {
    const wrapper = mount(AppSidebar, {
      global: {
        plugins: [createPinia(), i18n],
        stubs: ['router-link'],
      },
    })
    const items = wrapper.findAll('.nav-item')
    expect(items).toHaveLength(4)
  })

  it('AppSidebar 包含 API 状态指示器', () => {
    const wrapper = mount(AppSidebar, {
      global: {
        plugins: [createPinia(), i18n],
        stubs: ['router-link'],
      },
    })
    expect(wrapper.find('.status-dot').exists()).toBe(true)
  })
})
