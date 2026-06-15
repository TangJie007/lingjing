// AC 验收测试：壁纸详情页 (WL-003)
// 按 wetspec 规范：describe('WL-003') → describe('AC-xxx: ...') → it(...)
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { createI18n } from 'vue-i18n'
import { createRouter, createWebHashHistory } from 'vue-router'
import zhCN from '@/i18n/locales/zh-CN.json'
import { useWallpaperStore } from '@/stores/wallpaper'
import type { WallpaperItem } from '@/stores/wallpaper'

// Mock Tauri invoke to prevent "Tauri API unavailable" errors
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue([]),
  isTauri: vi.fn(() => false),
}))

const i18n = createI18n({
  legacy: false,
  locale: 'zh-CN',
  messages: { 'zh-CN': zhCN },
})

const mockAiWallpaper: WallpaperItem = {
  id: 'wp_test_004',
  filename: 'cyberpunk-alley.png',
  path: '/test/cyberpunk-alley.png',
  mediaType: 'image',
  format: 'png',
  width: 1920,
  height: 1080,
  fileSize: 2097152,
  source: 'ai',
  createdAt: '1718400000',
  thumbPath: '/test/cyberpunk-alley.thumb.webp',
  prompt: '赛博朋克夜晚雨巷',
  plan: 'A rainy cyberpunk alley at night, neon lights reflecting on wet pavement',
  model: 'Seedream 4.0',
}

const mockVideoWallpaper: WallpaperItem = {
  id: 'wp_test_005',
  filename: 'sunset-timelapse.mp4',
  path: '/test/sunset-timelapse.mp4',
  mediaType: 'video',
  format: 'mp4',
  width: 1920,
  height: 1080,
  fileSize: 52428800,
  source: 'local',
  createdAt: '1718400000',
}

async function makeWrapper(wallpaper: WallpaperItem | null, routeId: string = 'wp_test_004') {
  const pinia = createPinia()
  setActivePinia(pinia)
  const store = useWallpaperStore()
  if (wallpaper) {
    store.wallpapers = [wallpaper]
  }

  const router = createRouter({
    history: createWebHashHistory(),
    routes: [
      { path: '/wallpaper/:id', name: 'WallpaperDetail', component: { template: '<div/>' } },
    ],
  })
  await router.replace(`/wallpaper/${routeId}`)
  await router.isReady()

  const wrapper = mount(
    { template: '<div/>' },
    {
      global: {
        plugins: [pinia, i18n, router],
      },
    }
  )

  // Now mount the actual component with the same router
  const detailWrapper = mount(
    {
      template: '<WallpaperDetailPage />',
      components: {
        WallpaperDetailPage: (await import('@/views/WallpaperDetailPage.vue')).default,
      },
    },
    {
      global: {
        plugins: [pinia, i18n, router],
      },
    }
  )

  await detailWrapper.vm.$nextTick()
  return detailWrapper
}

beforeEach(() => {
  vi.clearAllMocks()
})

describe('WL-003', () => {
  describe('AC-001: 详情页布局', () => {
    it('应显示大图预览区域', async () => {
      const wrapper = await makeWrapper(mockAiWallpaper)
      expect(wrapper.find('.detail-preview').exists()).toBe(true)
      expect(wrapper.find('.preview-media').exists()).toBe(true)
    })

    it('应显示右侧元数据面板', async () => {
      const wrapper = await makeWrapper(mockAiWallpaper)
      expect(wrapper.find('.detail-info').exists()).toBe(true)
    })

    it('壁纸不存在时应显示提示', async () => {
      const wrapper = await makeWrapper(null)
      expect(wrapper.find('.detail-empty').exists()).toBe(true)
    })
  })

  describe('AC-002: 基础元数据展示', () => {
    it('应显示壁纸名称', async () => {
      const wrapper = await makeWrapper(mockAiWallpaper)
      expect(wrapper.find('.detail-name').text()).toBe('cyberpunk-alley.png')
    })

    it('应显示分辨率信息', async () => {
      const wrapper = await makeWrapper(mockAiWallpaper)
      const info = wrapper.find('.detail-info').text()
      expect(info).toContain('1920')
      expect(info).toContain('1080')
    })

    it('应显示文件大小', async () => {
      const wrapper = await makeWrapper(mockAiWallpaper)
      const info = wrapper.find('.detail-info').text()
      expect(info).toContain('MB')
    })

    it('应显示导入时间', async () => {
      const wrapper = await makeWrapper(mockAiWallpaper)
      const info = wrapper.find('.detail-info').text()
      expect(info).toContain('2024')
    })
  })

  describe('AC-003: AI 方案信息展示', () => {
    it('AI 生成壁纸应显示原始描述', async () => {
      const wrapper = await makeWrapper(mockAiWallpaper)
      const info = wrapper.find('.detail-info').text()
      expect(info).toContain('赛博朋克夜晚雨巷')
    })

    it('AI 生成壁纸应显示优化 Prompt', async () => {
      const wrapper = await makeWrapper(mockAiWallpaper)
      const info = wrapper.find('.detail-info').text()
      expect(info).toContain('cyberpunk')
    })

    it('AI 生成壁纸应显示模型名称', async () => {
      const wrapper = await makeWrapper(mockAiWallpaper)
      const info = wrapper.find('.detail-info').text()
      expect(info).toContain('Seedream 4.0')
    })
  })

  describe('AC-004: 复制 Prompt 和重新生成', () => {
    it('应显示复制 Prompt 按钮', async () => {
      const wrapper = await makeWrapper(mockAiWallpaper)
      const btns = wrapper.findAll('.action-btn')
      const copyBtn = btns.find((b) => b.text().includes('复制'))
      expect(copyBtn).toBeTruthy()
    })

    it('应显示重新生成按钮', async () => {
      const wrapper = await makeWrapper(mockAiWallpaper)
      const btns = wrapper.findAll('.action-btn')
      const regenBtn = btns.find((b) => b.text().includes('重新生成'))
      expect(regenBtn).toBeTruthy()
    })
  })

  describe('AC-005: 动态壁纸自动播放', () => {
    it('视频壁纸应显示 video 元素', async () => {
      const wrapper = await makeWrapper(mockVideoWallpaper, 'wp_test_005')
      const video = wrapper.find('video')
      expect(video.exists()).toBe(true)
    })

    it('video 应包含 controls 属性', async () => {
      const wrapper = await makeWrapper(mockVideoWallpaper, 'wp_test_005')
      const video = wrapper.find('video')
      expect(video.attributes('controls')).toBeDefined()
    })
  })

  describe('AC-007: 键盘快捷键', () => {
    it('按 Esc 应返回上一页', async () => {
      const wrapper = await makeWrapper(mockAiWallpaper)
      const spy = vi.spyOn(wrapper.vm.$router, 'back')
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
      expect(spy).toHaveBeenCalled()
    })
  })
})
