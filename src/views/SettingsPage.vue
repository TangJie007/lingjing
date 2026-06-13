<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '@/stores/settings'
import { useAppStore } from '@/stores/app'
import { useApiKeysStore } from '@/stores/api-keys'
import { invoke } from '@tauri-apps/api/core'
import ToggleSwitch from '@/components/common/ToggleSwitch.vue'

const { t } = useI18n()
const router = useRouter()
const settings = useSettingsStore()
const appStore = useAppStore()
const apiKeysStore = useApiKeysStore()

const autoStartLoading = ref(true)
const testing = ref<Record<string, boolean>>({})
const testError = ref<Record<string, string>>({})

onMounted(async () => {
  try {
    settings.autoStart = await invoke<boolean>('plugin:autostart|is_enabled')
  } catch {
    // Tauri API 不可用时使用 localStorage 中持久化的值
  } finally {
    autoStartLoading.value = false
  }

  // 加载已持久化的 API Key
  await apiKeysStore.loadSavedKeys()
})

watch(
  () => settings.autoStart,
  async (enabled) => {
    if (autoStartLoading.value) return
    try {
      if (enabled) {
        await invoke('plugin:autostart|enable')
      } else {
        await invoke('plugin:autostart|disable')
      }
    } catch {
      settings.autoStart = !enabled
    }
  }
)

watch(
  () => settings.locale,
  (lang) => {
    appStore.setLocale(lang)
  }
)

function goBack() {
  router.push('/library')
}

// 真实测试连接（API-002）
async function testConnection(platform: string) {
  testing.value[platform] = true
  testError.value[platform] = ''

  try {
    const result = await apiKeysStore.testConnection(platform)
    if (!result.success) {
      testError.value[platform] = result.error || t('toast.connectionFailed')
    }
  } catch {
    testError.value[platform] = t('toast.networkError')
  }

  testing.value[platform] = false
}

// 输入 Key 时自动加密持久化
async function onKeyInput(platform: string, rawKey: string) {
  const clean = apiKeysStore.sanitizeKey(rawKey)
  if (clean) {
    await apiKeysStore.persistKey(platform, clean)
  } else {
    apiKeysStore.saveKey(platform, '')
  }
}

// 打开外部链接
async function openUrl(url: string) {
  try {
    await invoke('open_url', { url })
  } catch {
    window.open(url, '_blank')
  }
}

function statusBadgeClass(status: string) {
  return {
    connected: 'connected',
    disconnected: 'disconnected',
    unconfigured: 'unconfigured',
  }[status] ?? 'unconfigured'
}

function statusBadgeText(status: string) {
  return {
    connected: t('apiKeys.connected'),
    disconnected: t('apiKeys.disconnected'),
    unconfigured: t('apiKeys.unconfigured'),
  }[status] ?? ''
}
</script>

<template>
  <div class="page-settings">
    <div class="settings-header">
      <button type="button" class="settings-back-btn" @click="goBack">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M19 12H5" />
          <path d="M12 19l-7-7 7-7" />
        </svg>
        {{ t('common.back') }}
      </button>
      <h1>{{ t('settings.title') }}</h1>
    </div>

    <!-- API 管理 -->
    <section class="settings-section">
      <h2 class="settings-section-title">{{ t('apiKeys.title') }}</h2>
      <div
        v-for="p in apiKeysStore.platforms.filter((item) => ['volcano', 'deepseek', 'zhipu'].includes(item.platform))"
        :key="p.platform"
        class="api-card"
        :class="{
          configured: p.status === 'connected',
          upcoming: !p.available,
        }"
      >
        <div class="api-card-header">
          <div class="api-platform-icon volc">{{ p.platform === 'volcano' ? 'V' : 'D' }}</div>
          <div class="api-platform-info">
            <div class="api-platform-name">{{ p.name }}</div>
            <div class="api-platform-desc">
              <template v-if="p.platform === 'volcano'">{{ t('apiKeys.volcanoDesc') }}</template>
              <template v-else-if="p.platform === 'deepseek'">DeepSeek V4 — 第一步意图分析备选</template>
              <template v-else-if="p.platform === 'zhipu'">GLM-4V-Plus — 第一步视觉理解备选</template>
              <template v-else>{{ t('apiKeys.comingSoon') }}</template>
            </div>
          </div>
          <span v-if="p.available" class="api-status-badge" :class="statusBadgeClass(p.status)">
            <span class="status-dot-inline" :class="{ offline: p.status !== 'connected' }" />
            {{ statusBadgeText(p.status) }}
          </span>
          <span v-else class="coming-soon-tag">
            {{ p.platform === 'deepseek' ? 'V1.1' : p.platform === 'zhipu' ? 'V1.2' : t('apiKeys.comingSoon') }}
          </span>
        </div>

        <template v-if="p.available">
          <div class="api-key-input-row">
            <input
              type="password"
              class="api-key-input"
              :placeholder="t('apiKeys.keyPlaceholder')"
              :value="p.maskedKey"
              @input="(e: Event) => onKeyInput(p.platform, (e.target as HTMLInputElement).value)"
            />
            <button
              type="button"
              class="btn-test"
              :disabled="testing[p.platform]"
              @click="testConnection(p.platform)"
            >
              {{ testing[p.platform] ? '...' : t('wizard.apiSetup.testBtn') }}
            </button>
          </div>

          <div class="api-model-row">
            <div class="api-model-group">
              <span class="api-model-label">{{ t('apiKeys.modelStep1') }}</span>
              <select v-model="settings.defaultStep1Model" class="api-model-select">
                <option value="auto">{{ t('apiKeys.modelAuto') }}</option>
                <option value="doubao-2.0-vision">doubao-2.0-vision</option>
                <option value="doubao-2.0-lite-32k">doubao-2.0-lite-32k</option>
              </select>
            </div>
            <div class="api-model-group">
              <span class="api-model-label">{{ t('apiKeys.modelStep2') }}</span>
              <select v-model="settings.defaultStep2Model" class="api-model-select">
                <option value="Seedream 4.0">Seedream 4.0</option>
                <option value="Seedream 5.0 lite">Seedream 5.0 lite</option>
                <option value="即梦视频 3.0 Pro">即梦视频 3.0 Pro</option>
              </select>
            </div>
          </div>

          <div class="api-help-links">
            <a class="api-help-link" href="#" @click.prevent="openUrl('https://console.volcengine.com/ark/region:ark+cn-beijing/overview')">⟶ {{ t('apiKeys.helpRegister') }}</a>
            <a class="api-help-link" href="#" @click.prevent="openUrl('https://www.volcengine.com/docs/82379')">⟶ {{ t('apiKeys.helpDocs') }}</a>
            <a class="api-help-link" href="#" @click.prevent="openUrl('https://www.volcengine.com/docs/82379/1099463')">⟶ {{ t('apiKeys.helpPricing') }}</a>
          </div>
        </template>
      </div>
    </section>

    <!-- 通用 -->
    <section class="settings-section">
      <h2 class="settings-section-title">{{ t('settings.general') }}</h2>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ t('settings.autoStart') }}</span>
          <span class="setting-desc">{{ t('settings.autoStartDesc') }}</span>
        </div>
        <ToggleSwitch v-model="settings.autoStart" :disabled="autoStartLoading" />
      </div>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ t('settings.language') }}</span>
          <span class="setting-desc">{{ t('settings.languageDesc') }}</span>
        </div>
        <select v-model="settings.locale" class="setting-select">
          <option value="zh-CN">{{ t('settings.langZhCN') }}</option>
          <option value="en" disabled>{{ t('settings.langEn') }}</option>
        </select>
      </div>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ t('settings.minimizeToTray') }}</span>
          <span class="setting-desc">{{ t('settings.minimizeToTrayDesc') }}</span>
        </div>
        <ToggleSwitch :model-value="true" disabled />
      </div>
    </section>

    <!-- 壁纸 -->
    <section class="settings-section">
      <h2 class="settings-section-title">{{ t('settings.wallpaper') }}</h2>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ t('settings.fps') }}</span>
          <span class="setting-desc">{{ t('settings.fpsDesc') }}</span>
        </div>
        <select v-model.number="settings.videoFps" class="setting-select">
          <option :value="30">30 FPS</option>
          <option :value="60">60 FPS</option>
        </select>
      </div>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ t('settings.scaling') }}</span>
          <span class="setting-desc">{{ t('settings.scalingDesc') }}</span>
        </div>
        <select v-model="settings.scalingMode" class="setting-select">
          <option value="fill">{{ t('settings.scalingFill') }}</option>
          <option value="fit">{{ t('settings.scalingFit') }}</option>
          <option value="stretch">{{ t('settings.scalingStretch') }}</option>
          <option value="tile">{{ t('settings.scalingTile') }}</option>
        </select>
      </div>
    </section>

    <!-- 性能 -->
    <section class="settings-section">
      <h2 class="settings-section-title">{{ t('settings.performance') }}</h2>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ t('settings.pauseOnFullscreen') }}</span>
          <span class="setting-desc">{{ t('settings.pauseOnFullscreenDesc') }}</span>
        </div>
        <ToggleSwitch v-model="settings.pauseOnFullscreen" />
      </div>
    </section>

    <!-- 存储 -->
    <section class="settings-section">
      <h2 class="settings-section-title">{{ t('settings.storage') }}</h2>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ t('settings.cacheLimit') }}</span>
          <span class="setting-desc">{{ t('settings.cacheLimitDesc') }}</span>
        </div>
        <input
          v-model.number="settings.cacheLimitGB"
          type="number"
          min="1"
          max="50"
          class="setting-input"
        />
      </div>
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{{ t('settings.clearCache') }}</span>
          <span class="setting-desc">{{ t('settings.clearCacheDesc') }}</span>
        </div>
        <button type="button" class="btn-danger">{{ t('settings.clearCache') }}</button>
      </div>
    </section>

    <!-- 关于 -->
    <section class="settings-section">
      <h2 class="settings-section-title">{{ t('settings.about') }}</h2>
      <div class="setting-row">
        <span class="setting-label">{{ t('settings.version') }}</span>
        <span class="setting-desc">1.0.0</span>
      </div>
      <div class="setting-row">
        <span class="setting-desc">{{ t('settings.copyright') }}</span>
      </div>
    </section>
  </div>
</template>

<style scoped>
.status-dot-inline {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--color-success);
  display: inline-block;
}
.status-dot-inline.offline { background: var(--color-text-tertiary); }
</style>
