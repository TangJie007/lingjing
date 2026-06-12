// API Key 管理状态 (API-001~003)
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export interface ApiKeyEntry {
  platform: string
  name: string
  key: string
  maskedKey: string
  status: 'connected' | 'disconnected' | 'unconfigured'
  available: boolean // V1 是否可用
}

export const useApiKeysStore = defineStore('apiKeys', () => {
  const platforms = ref<ApiKeyEntry[]>([
    {
      platform: 'volcano',
      name: '火山引擎',
      key: '',
      maskedKey: '',
      status: 'unconfigured',
      available: true,
    },
    {
      platform: 'deepseek',
      name: 'DeepSeek',
      key: '',
      maskedKey: '',
      status: 'unconfigured',
      available: false,
    },
    {
      platform: 'zhipu',
      name: '智谱AI',
      key: '',
      maskedKey: '',
      status: 'unconfigured',
      available: false,
    },
    {
      platform: 'kling',
      name: '快手可灵',
      key: '',
      maskedKey: '',
      status: 'unconfigured',
      available: false,
    },
  ])

  // 掩码显示（保留前3后4字符）
  function maskKey(key: string): string {
    if (key.length <= 8) return '****'
    return key.slice(0, 3) + '****' + key.slice(-4)
  }

  // 粘贴自动去空格
  function sanitizeKey(raw: string): string {
    return raw.trim().replace(/\s+/g, '')
  }

  function saveKey(platform: string, rawKey: string) {
    const entry = platforms.value.find((p) => p.platform === platform)
    if (!entry) return
    const clean = sanitizeKey(rawKey)
    entry.key = clean
    entry.maskedKey = maskKey(clean)
    entry.status = 'unconfigured' // 需测试连接后更新
  }

  function setStatus(platform: string, status: ApiKeyEntry['status']) {
    const entry = platforms.value.find((p) => p.platform === platform)
    if (entry) entry.status = status
  }

  const hasConfiguredKey = computed(() =>
    platforms.value.some((p) => p.available && p.status === 'connected')
  )

  return {
    platforms,
    maskKey,
    sanitizeKey,
    saveKey,
    setStatus,
    hasConfiguredKey,
  }
})
