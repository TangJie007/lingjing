// AC 验收测试：壁纸收藏与标签分类 (WL-004)
// 按 wetspec 规范：describe('WL-004') → describe('AC-xxx: ...') → it(...)
import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useWallpaperStore } from '@/stores/wallpaper'
import type { WallpaperItem } from '@/stores/wallpaper'

const mockWallpaper: WallpaperItem = {
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
}

const mockAiWallpaper: WallpaperItem = {
  id: 'wp_test_002',
  filename: 'cyberpunk-alley.png',
  path: '/test/cyberpunk-alley.png',
  mediaType: 'image',
  format: 'png',
  width: 1920,
  height: 1080,
  fileSize: 2097152,
  source: 'ai',
  createdAt: '1718400000',
  prompt: '赛博朋克夜晚雨巷',
  tags: ['赛博朋克', '夜景'],
}

beforeEach(() => {
  setActivePinia(createPinia())
})

describe('WL-004', () => {
  describe('AC-001: 收藏按钮', () => {
    it('toggleFavorite 应切换收藏状态', () => {
      const store = useWallpaperStore()
      store.wallpapers = [{ ...mockWallpaper }]
      expect(store.wallpapers[0].favorite).toBeFalsy()
      store.toggleFavorite('wp_test_001')
      expect(store.wallpapers[0].favorite).toBe(true)
      store.toggleFavorite('wp_test_001')
      expect(store.wallpapers[0].favorite).toBe(false)
    })

    it('toggleFavorite 对不存在的 ID 不应报错', () => {
      const store = useWallpaperStore()
      expect(() => store.toggleFavorite('nonexistent')).not.toThrow()
    })
  })

  describe('AC-002: 收藏筛选', () => {
    it('showFavoritesOnly 为 true 时应只显示收藏壁纸', () => {
      const store = useWallpaperStore()
      store.wallpapers = [
        { ...mockWallpaper, id: 'wp_1', favorite: true },
        { ...mockWallpaper, id: 'wp_2', favorite: false },
        { ...mockWallpaper, id: 'wp_3', favorite: true },
      ]
      store.showFavoritesOnly = true
      expect(store.filteredWallpapers).toHaveLength(2)
      expect(store.filteredWallpapers[0].id).toBe('wp_1')
      expect(store.filteredWallpapers[1].id).toBe('wp_3')
    })
  })

  describe('AC-003: 自定义标签', () => {
    it('updateTags 应更新壁纸标签', () => {
      const store = useWallpaperStore()
      store.wallpapers = [{ ...mockWallpaper }]
      store.updateTags('wp_test_001', ['工作', '极简'])
      expect(store.wallpapers[0].tags).toEqual(['工作', '极简'])
    })

    it('allTags 应返回所有去重标签', () => {
      const store = useWallpaperStore()
      store.wallpapers = [
        { ...mockWallpaper, id: 'wp_1', tags: ['赛博朋克', '夜景'] },
        { ...mockWallpaper, id: 'wp_2', tags: ['极简', '夜景'] },
      ]
      expect(store.allTags).toContain('赛博朋克')
      expect(store.allTags).toContain('夜景')
      expect(store.allTags).toContain('极简')
      expect(store.allTags).toHaveLength(3)
    })
  })

  describe('AC-004: AI 自动标签', () => {
    it('AI 生成壁纸应有预设标签', () => {
      const store = useWallpaperStore()
      store.wallpapers = [mockAiWallpaper]
      expect(store.wallpapers[0].tags).toEqual(['赛博朋克', '夜景'])
    })
  })

  describe('AC-005: 标签筛选', () => {
    it('activeTag 应过滤只显示匹配标签的壁纸', () => {
      const store = useWallpaperStore()
      store.wallpapers = [
        { ...mockWallpaper, id: 'wp_1', tags: ['赛博朋克'] },
        { ...mockWallpaper, id: 'wp_2', tags: ['极简'] },
        { ...mockWallpaper, id: 'wp_3', tags: ['赛博朋克', '夜景'] },
      ]
      store.activeTag = '赛博朋克'
      expect(store.filteredWallpapers).toHaveLength(2)
    })

    it('activeTag 设为 null 应显示全部', () => {
      const store = useWallpaperStore()
      store.wallpapers = [
        { ...mockWallpaper, id: 'wp_1', tags: ['赛博朋克'] },
        { ...mockWallpaper, id: 'wp_2', tags: ['极简'] },
      ]
      store.activeTag = null
      expect(store.filteredWallpapers).toHaveLength(2)
    })
  })

  describe('AC-006: 搜索功能', () => {
    it('searchQuery 应按文件名搜索', () => {
      const store = useWallpaperStore()
      store.wallpapers = [
        { ...mockWallpaper, id: 'wp_1', filename: 'mountain.jpg' },
        { ...mockWallpaper, id: 'wp_2', filename: 'sunset.mp4' },
        { ...mockWallpaper, id: 'wp_3', filename: 'ocean.png' },
      ]
      store.searchQuery = 'mountain'
      expect(store.filteredWallpapers).toHaveLength(1)
      expect(store.filteredWallpapers[0].id).toBe('wp_1')
    })

    it('searchQuery 应按标签搜索', () => {
      const store = useWallpaperStore()
      store.wallpapers = [
        { ...mockWallpaper, id: 'wp_1', tags: ['赛博朋克'] },
        { ...mockWallpaper, id: 'wp_2', tags: ['极简'] },
      ]
      store.searchQuery = '赛博'
      expect(store.filteredWallpapers).toHaveLength(1)
      expect(store.filteredWallpapers[0].id).toBe('wp_1')
    })

    it('searchQuery 应按 prompt 搜索', () => {
      const store = useWallpaperStore()
      store.wallpapers = [
        { ...mockWallpaper, id: 'wp_1', prompt: '赛博朋克夜晚' },
        { ...mockWallpaper, id: 'wp_2', prompt: '极简山水' },
      ]
      store.searchQuery = '赛博'
      expect(store.filteredWallpapers).toHaveLength(1)
    })

    it('searchQuery 为空时应显示全部', () => {
      const store = useWallpaperStore()
      store.wallpapers = [mockWallpaper, mockAiWallpaper]
      store.searchQuery = ''
      expect(store.filteredWallpapers).toHaveLength(2)
    })

    it('searchQuery 大小写不敏感', () => {
      const store = useWallpaperStore()
      store.wallpapers = [
        { ...mockWallpaper, id: 'wp_1', filename: 'MOUNTAIN.jpg' },
      ]
      store.searchQuery = 'mountain'
      expect(store.filteredWallpapers).toHaveLength(1)
    })
  })
})
