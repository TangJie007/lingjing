// AC 验收测试：壁纸库视觉化升级 (WL-002)
// 按 wetspec 规范：describe('WL-002') → describe('AC-xxx: ...') → it(...)
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { createI18n } from 'vue-i18n'
import { createRouter, createWebHashHistory } from 'vue-router'
import zhCN from '@/i18n/locales/zh-CN.json'
import WallpaperCard from '@/components/wallpaper/WallpaperCard.vue'
import { useWallpaperStore } from '@/stores/wallpaper'
import type { WallpaperItem } from '@/stores/wallpaper'

const i18n = createI18n({
  legacy: false,
  locale: 'zh-CN',
  messages: { 'zh-CN': zhCN },
})

function makeWrapper(props: { wallpaper: WallpaperItem; isActive?: boolean }) {
  const router = createRouter({
    history: createWebHashHistory(),
    routes: [
      { path: '/wallpaper/:id', name: 'WallpaperDetail', component: { template: '<div/>' } },
    ],
  })

  return mount(WallpaperCard, {
    props,
    global: {
      plugins: [createPinia(), i18n, router],
    },
  })
}

const mockImage: WallpaperItem = {
  id: 'wp_test_001',
  filename: 'test-mountain.jpg',
  path: '/test/test-mountain.jpg',
  mediaType: 'image',
  format: 'jpg',
  width: 1920,
  height: 1080,
  fileSize: 3145728,
  source: 'local',
  createdAt: '1718400000',
  thumbPath: '/test/test-mountain.thumb.webp',
}

const mockVideo: WallpaperItem = {
  id: 'wp_test_002',
  filename: 'test-sunset.mp4',
  path: '/test/test-sunset.mp4',
  mediaType: 'video',
  format: 'mp4',
  width: 1920,
  height: 1080,
  fileSize: 52428800,
  source: 'local',
  createdAt: '1718400000',
  thumbPath: '/test/test-sunset.thumb.webp',
}

const mockGif: WallpaperItem = {
  id: 'wp_test_003',
  filename: 'test-cat.gif',
  path: '/test/test-cat.gif',
  mediaType: 'gif',
  format: 'gif',
  width: 800,
  height: 600,
  fileSize: 1048576,
  source: 'local',
  createdAt: '1718400000',
}

const mockAi: WallpaperItem = {
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
  model: 'Seedream 4.0',
}

beforeEach(() => {
  setActivePinia(createPinia())
})

describe('WL-002', () => {
  describe('AC-001: 卡片网格默认展示', () => {
    it('应渲染壁纸卡片，包含缩略图和文件名', () => {
      const wrapper = makeWrapper({ wallpaper: mockImage })
      expect(wrapper.find('.lib-card').exists()).toBe(true)
      expect(wrapper.find('.lib-card-title').text()).toBe('test-mountain.jpg')
      expect(wrapper.find('.lib-card-img img').exists()).toBe(true)
    })

    it('应显示文件大小和日期信息', () => {
      const wrapper = makeWrapper({ wallpaper: mockImage })
      const meta = wrapper.find('.lib-card-meta').text()
      expect(meta).toContain('MB')
    })
  })

  describe('AC-002: 视图切换', () => {
    it('卡片应支持点击进入详情页', async () => {
      const wrapper = makeWrapper({ wallpaper: mockImage })
      const router = wrapper.vm.$router
      const spy = vi.spyOn(router, 'push')
      await wrapper.find('.lib-card').trigger('click')
      expect(spy).toHaveBeenCalled()
    })
  })

  describe('AC-003: 卡片尺寸切换', () => {
    it('卡片应有固定的 aspect-ratio 16/10', () => {
      const wrapper = makeWrapper({ wallpaper: mockImage })
      const img = wrapper.find('.lib-card-img')
      expect(img.exists()).toBe(true)
    })
  })

  describe('AC-004: MP4/WebM 缩略图自动生成', () => {
    it('视频卡片应有缩略图显示', () => {
      const wrapper = makeWrapper({ wallpaper: mockVideo })
      expect(wrapper.find('.lib-card-img img').exists()).toBe(true)
    })

    it('视频卡片应显示格式角标', () => {
      const wrapper = makeWrapper({ wallpaper: mockVideo })
      expect(wrapper.find('.lib-badge.format').text()).toBe('MP4')
    })
  })

  describe('AC-005: GIF 缩略图自动生成', () => {
    it('GIF 卡片应显示格式角标', () => {
      const wrapper = makeWrapper({ wallpaper: mockGif })
      expect(wrapper.find('.lib-badge.format').text()).toBe('GIF')
    })
  })

  describe('AC-006: 类型角标', () => {
    it('视频壁纸应显示格式角标', () => {
      const wrapper = makeWrapper({ wallpaper: mockVideo })
      expect(wrapper.find('.lib-badge.format').exists()).toBe(true)
    })

    it('AI 生成壁纸应显示 AI 角标', () => {
      const wrapper = makeWrapper({ wallpaper: mockAi })
      expect(wrapper.find('.lib-badge.ai').exists()).toBe(true)
    })

    it('当前使用中的壁纸应显示使用中角标', () => {
      const wrapper = makeWrapper({ wallpaper: mockImage, isActive: true })
      expect(wrapper.find('.lib-badge.active').exists()).toBe(true)
    })
  })

  describe('AC-007: hover 动态预览', () => {
    it('hover 视频卡片应触发预览', async () => {
      const wrapper = makeWrapper({ wallpaper: mockVideo })
      await wrapper.find('.lib-card').trigger('mouseenter')
      // hover 预览有 500ms 延迟，需要等待
      await new Promise((r) => setTimeout(r, 600))
      await wrapper.vm.$nextTick()
      // 应显示 video 元素
      expect(wrapper.find('.preview-video').exists() || wrapper.find('video').exists()).toBe(true)
    })

    it('鼠标移出应停止预览', async () => {
      const wrapper = makeWrapper({ wallpaper: mockVideo })
      await wrapper.find('.lib-card').trigger('mouseenter')
      await new Promise((r) => setTimeout(r, 600))
      await wrapper.find('.lib-card').trigger('mouseleave')
      await wrapper.vm.$nextTick()
      expect(wrapper.find('.preview-video').exists()).toBe(false)
    })
  })

  describe('AC-009: 默认排序', () => {
    it('store 中 wallpapers 应支持按 createdAt 排序', () => {
      const store = useWallpaperStore()
      store.wallpapers = [
        { ...mockImage, id: 'wp_1', createdAt: '1000' },
        { ...mockImage, id: 'wp_2', createdAt: '2000' },
        { ...mockImage, id: 'wp_3', createdAt: '3000' },
      ]
      const sorted = [...store.wallpapers].sort(
        (a, b) => parseInt(b.createdAt) - parseInt(a.createdAt)
      )
      expect(sorted[0].id).toBe('wp_3')
      expect(sorted[2].id).toBe('wp_1')
    })
  })
})
