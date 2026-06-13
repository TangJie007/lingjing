// API Key 管理状态 (API-001~003)
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

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

  /// 从 Rust 后端加载已持久化的 Key 信息
  async function loadSavedKeys() {
    try {
      const saved: Array<{ platform: string; masked_key: string }> = await invoke(
        'crypto_list_platforms'
      )
      for (const item of saved) {
        const entry = platforms.value.find((p) => p.platform === item.platform)
        if (entry) {
          entry.maskedKey = item.masked_key
          // 标记为 unconfigured，需要用户测试连接
          entry.status = 'unconfigured'
        }
      }
    } catch {
      // 首次启动或无已保存的 Key
    }
  }

  /// 持久化 Key 到 Rust 后端（加密存储）
  async function persistKey(platform: string, rawKey: string): Promise<string> {
    const clean = sanitizeKey(rawKey)
    if (!clean) {
      // 空 Key：仅更新本地状态
      const entry = platforms.value.find((p) => p.platform === platform)
      if (entry) {
        entry.key = ''
        entry.maskedKey = ''
        entry.status = 'unconfigured'
      }
      return ''
    }
    // 调用 Rust 加密并持久化
    const masked = await invoke<string>('crypto_encrypt', {
      platform,
      plaintext: clean,
    })
    // 更新本地状态
    const entry = platforms.value.find((p) => p.platform === platform)
    if (entry) {
      entry.key = clean
      entry.maskedKey = masked
      entry.status = 'unconfigured' // 需测试连接后更新
    }
    return masked
  }

  /// 解密读取 Key（用于 API 调用时获取明文）
  async function getDecryptedKey(platform: string): Promise<string> {
    try {
      return await invoke<string>('crypto_decrypt', { platform })
    } catch {
      return ''
    }
  }

  /// 测试 API 连接
  async function testConnection(platform: string): Promise<{
    success: boolean
    error?: string
  }> {
    const entry = platforms.value.find((p) => p.platform === platform)
    if (!entry || !entry.key) {
      return { success: false, error: '未配置 API Key' }
    }

    const result = await invoke<{
      success: boolean
      error: string | null
      status_code: number | null
    }>('test_connection', { apiKey: entry.key })

    if (result.success) {
      setStatus(platform, 'connected')
    } else {
      setStatus(platform, 'disconnected')
    }

    return {
      success: result.success,
      error: result.error ?? undefined,
    }
  }

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
    loadSavedKeys,
    persistKey,
    getDecryptedKey,
    testConnection,
  }
})
