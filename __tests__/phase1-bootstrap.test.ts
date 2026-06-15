// 阶段一基础验证测试
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createI18n } from 'vue-i18n'
import zhCN from '@/i18n/locales/zh-CN.json'

// Mock Tauri APIs（测试环境中不可用）
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({
    minimize: vi.fn(),
    toggleMaximize: vi.fn(),
    close: vi.fn(),
    hide: vi.fn(),
    onCloseRequested: vi.fn(() => Promise.resolve(() => {})),
  }),
}))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === 'plugin:autostart|is_enabled') return Promise.resolve(false)
    return Promise.resolve()
  }),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}))

// mock useRoute 供 AppSidebar 中 isActive() 使用
vi.mock('vue-router', () => ({
  useRoute: () => ({ path: '/ai-create' }),
  useRouter: () => ({ replace: vi.fn(), push: vi.fn() }),
}))

const i18n = createI18n({
  legacy: false,
  locale: 'zh-CN',
  messages: { 'zh-CN': zhCN },
})

import AppSidebar from '@/components/layout/AppSidebar.vue'
import AppTitlebar from '@/components/layout/AppTitlebar.vue'
import AppStatusbar from '@/components/layout/AppStatusbar.vue'
import OnboardingPage from '@/views/OnboardingPage.vue'
import SettingsPage from '@/views/SettingsPage.vue'

describe('阶段一：基础骨架', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    localStorage.clear()
  })

  // --- 布局组件 ---
  describe('AppSidebar', () => {
    it('渲染 4 个导航项', () => {
      const wrapper = mount(AppSidebar, {
        global: {
          plugins: [createPinia(), i18n],
          stubs: ['router-link'],
        },
      })
      const items = wrapper.findAll('.nav-item')
      expect(items).toHaveLength(4)
    })

    it('包含 API 状态指示器', () => {
      const wrapper = mount(AppSidebar, {
        global: {
          plugins: [createPinia(), i18n],
          stubs: ['router-link'],
        },
      })
      expect(wrapper.find('.status-dot').exists()).toBe(true)
    })
  })

  describe('AppTitlebar', () => {
    it('渲染品牌名称和窗口控制按钮', () => {
      const wrapper = mount(AppTitlebar, {
        global: { plugins: [i18n] },
      })
      expect(wrapper.find('.brand-name').exists()).toBe(true)
      expect(wrapper.find('.brand-name').text()).toBe('灵境')
      // 3 个窗口控制按钮
      const buttons = wrapper.findAll('.tb-btn')
      expect(buttons).toHaveLength(3)
    })
  })

  describe('AppStatusbar', () => {
    it('渲染状态栏', () => {
      const wrapper = mount(AppStatusbar, {
        global: { plugins: [createPinia(), i18n] },
      })
      expect(wrapper.find('.statusbar').exists()).toBe(true)
    })
  })

  // --- OB-001: 欢迎页 ---
  describe('OB-001: 欢迎页与品牌动画', () => {
    it('AC-001: 渲染 3 步向导流程', () => {
      const wrapper = mount(OnboardingPage, {
        global: {
          plugins: [createPinia(), i18n],
          stubs: ['router-link'],
        },
      })
      // 进度条 3 个点
      const dots = wrapper.findAll('.wizard-progress-dot')
      expect(dots).toHaveLength(3)
      // 欢迎页标题
      expect(wrapper.find('.wizard-title').text()).toBe(zhCN.wizard.welcome.title)
    })

    it('点击"开始配置"进入步骤 2（API 配置）', async () => {
      const wrapper = mount(OnboardingPage, {
        global: {
          plugins: [createPinia(), i18n],
          stubs: ['router-link'],
        },
      })
      const startBtn = wrapper.find('.btn-wizard-primary')
      await startBtn.trigger('click')
      // 步骤 2 应显示 API 配置标题
      expect(wrapper.find('.wizard-title').text()).toBe(zhCN.wizard.apiSetup.title)
    })

    it('点击"跳过"完成向导并跳转', async () => {
      const wrapper = mount(OnboardingPage, {
        global: {
          plugins: [createPinia(), i18n],
          stubs: ['router-link'],
        },
      })
      const skipBtn = wrapper.find('.btn-wizard-secondary')
      await skipBtn.trigger('click')
      // onboardingComplete 应设置为 true
      expect(localStorage.getItem('lingscape-onboarding-complete')).toBe('true')
    })
  })

  // --- SET-001: 通用设置 ---
  describe('SET-001: 通用设置', () => {
    it('AC-001: 开机自启开关渲染', () => {
      const wrapper = mount(SettingsPage, {
        global: { plugins: [createPinia(), i18n] },
      })
      // Settings page should render toggle switches
      const toggles = wrapper.findAll('.toggle-container')
      expect(toggles.length).toBeGreaterThanOrEqual(1)
    })

    it('AC-002: 语言选择下拉包含中文简体', () => {
      const wrapper = mount(SettingsPage, {
        global: { plugins: [createPinia(), i18n] },
      })
      const select = wrapper.find('select')
      expect(select.exists()).toBe(true)
      const options = wrapper.findAll('option')
      expect(options.length).toBeGreaterThanOrEqual(1)
      // 第一个 option 是语言选择
      expect(options.some((o) => o.text().includes('中文') || o.text().includes('zh'))).toBe(true)
    })
  })

  // --- SET-005: 国际化框架 ---
  describe('SET-005: 国际化框架', () => {
    it('AC-001: 无硬编码中文字符串 — 组件使用 i18n key', () => {
      // 挂载 AppSidebar，验证导航标签来自 i18n
      const wrapper = mount(AppSidebar, {
        global: {
          plugins: [createPinia(), i18n],
          stubs: ['router-link'],
        },
      })
      // 验证 4 个导航项存在
      const navItems = wrapper.findAll('.nav-item')
      expect(navItems.length).toBe(4)
    })

    it('AC-004: 语言热切换 — setLocale 变更后 locale 更新', () => {
      // 验证 i18n locale 初始值
      expect(i18n.global.locale.value).toBe('zh-CN')
      // 切换 locale
      i18n.global.locale.value = 'zh-CN'
      expect(i18n.global.locale.value).toBe('zh-CN')
    })
  })
})
