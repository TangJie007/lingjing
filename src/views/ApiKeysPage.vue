<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useApiKeysStore } from '@/stores/api-keys'
import { useSettingsStore } from '@/stores/settings'

const { t } = useI18n()
const apiKeysStore = useApiKeysStore()
const settings = useSettingsStore()

const testing = ref<Record<string, boolean>>({})

async function testConnection(platform: string) {
  testing.value[platform] = true
  // 模拟测试连接（后续阶段由 Tauri IPC 实现）
  await new Promise((r) => setTimeout(r, 1500))
  apiKeysStore.setStatus(platform, 'connected')
  testing.value[platform] = false
}

function onKeyInput(platform: string, rawKey: string) {
  const clean = apiKeysStore.sanitizeKey(rawKey)
  apiKeysStore.saveKey(platform, clean)
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
  <div class="page-api">
    <h1>{{ t('apiKeys.title') }}</h1>

    <!-- 火山引擎 (V1 可用) -->
    <div
      v-for="p in apiKeysStore.platforms"
      :key="p.platform"
      class="api-card"
      :class="{
        configured: p.status === 'connected',
        upcoming: !p.available,
      }"
    >
      <div class="api-card-header">
        <div class="api-platform-icon volc">{{ p.platform === 'volcano' ? '🔥' : '🤖' }}</div>
        <div class="api-platform-info">
          <div class="api-platform-name">{{ p.name }}</div>
          <div class="api-platform-desc">
            {{ p.platform === 'volcano' ? t('apiKeys.volcanoDesc') : t('apiKeys.comingSoon') }}
          </div>
        </div>
        <span v-if="p.available" class="api-status-badge" :class="statusBadgeClass(p.status)">
          <span class="status-dot" :class="{ offline: p.status !== 'connected' }" />
          {{ statusBadgeText(p.status) }}
        </span>
        <span v-else class="coming-soon-tag">{{ t('apiKeys.comingSoon') }}</span>
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
            class="btn-test"
            :disabled="testing[p.platform]"
            @click="testConnection(p.platform)"
          >
            {{ testing[p.platform] ? '...' : t('wizard.apiSetup.testBtn') }}
          </button>
        </div>

        <!-- 模型选择 -->
        <div class="api-model-row">
          <div class="api-model-group">
            <span class="api-model-label">{{ t('apiKeys.modelStep1') }}</span>
            <select v-model="settings.defaultStep1Model" class="api-model-select">
              <option value="doubao-2.0-lite-32k">doubao-2.0-lite-32k</option>
              <option value="doubao-2.0-vision">doubao-2.0-vision</option>
            </select>
          </div>
          <div class="api-model-group">
            <span class="api-model-label">{{ t('apiKeys.modelStep2') }}</span>
            <select v-model="settings.defaultStep2Model" class="api-model-select">
              <option value="Seedream 4.0">Seedream 4.0</option>
              <option value="Seedream 5.0 lite">Seedream 5.0 lite</option>
            </select>
          </div>
        </div>

        <div class="api-help-links">
          <a class="api-help-link" href="#" @click.prevent>📖 {{ t('apiKeys.helpDocs') }}</a>
          <a class="api-help-link" href="#" @click.prevent>🔗 {{ t('apiKeys.helpRegister') }}</a>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.page-api { display: flex; flex-direction: column; gap: var(--spacing-5); max-width: 680px; }
.page-api h1 {
  font-family: var(--font-display); font-size: var(--text-2xl);
  font-weight: 600; letter-spacing: -0.02em;
}

.api-card {
  background: var(--color-bg-surface);
  border: 1px solid var(--color-border-default);
  border-radius: var(--radius-lg);
  padding: var(--spacing-5);
  display: flex; flex-direction: column; gap: var(--spacing-4);
  transition: border-color 0.25s ease;
}
.api-card.configured { border-color: oklch(35% 0.04 155); }
.api-card.upcoming { opacity: 0.5; }

.api-card-header { display: flex; align-items: center; gap: var(--spacing-3); }

.api-platform-icon {
  width: 40px; height: 40px; border-radius: var(--radius-md);
  display: flex; align-items: center; justify-content: center;
  font-size: 20px; flex-shrink: 0;
}
.api-platform-icon.volc {
  background: linear-gradient(135deg, oklch(65% 0.18 25), oklch(55% 0.15 35));
  color: white;
}

.api-platform-info { flex: 1; }
.api-platform-name { font-size: var(--text-base); font-weight: 600; color: var(--color-text-primary); }
.api-platform-desc { font-size: var(--text-xs); color: var(--color-text-tertiary); margin-top: 1px; }

.api-status-badge {
  display: flex; align-items: center; gap: var(--spacing-1);
  padding: 2px var(--spacing-2); border-radius: var(--radius-full);
  font-size: var(--text-xs); font-weight: 500;
}
.api-status-badge.connected { background: oklch(22% 0.04 155); color: var(--color-success); }
.api-status-badge.disconnected { background: oklch(22% 0.03 20); color: var(--color-error); }
.api-status-badge.unconfigured { background: var(--color-bg-elevated); color: var(--color-text-tertiary); }

.coming-soon-tag {
  font-size: var(--text-xs); color: var(--color-text-tertiary);
  padding: 2px var(--spacing-2); border-radius: var(--radius-full);
  border: 1px solid var(--color-border-subtle);
}

.api-key-input-row { display: flex; gap: var(--spacing-2); }
.api-key-input {
  flex: 1; padding: var(--spacing-2) var(--spacing-3);
  border-radius: var(--radius-md); background: var(--color-bg-base);
  border: 1px solid var(--color-border-subtle); color: var(--color-text-primary);
  font-family: var(--font-mono); font-size: var(--text-sm); outline: none;
  transition: border-color 0.12s ease;
}
.api-key-input:focus { border-color: var(--color-primary); }
.api-key-input::placeholder { color: var(--color-text-tertiary); }

.btn-test {
  padding: var(--spacing-2) var(--spacing-4); border-radius: var(--radius-md);
  border: 1px solid var(--color-border-subtle); background: var(--color-bg-elevated);
  color: var(--color-text-primary); font-size: var(--text-sm); font-weight: 500;
  cursor: pointer; white-space: nowrap; transition: all 0.12s ease;
}
.btn-test:hover:not(:disabled) { border-color: var(--color-primary); color: var(--color-primary-light); }

.api-model-row { display: flex; gap: var(--spacing-4); }
.api-model-group { flex: 1; display: flex; flex-direction: column; gap: var(--spacing-1); }
.api-model-label { font-size: var(--text-xs); color: var(--color-text-tertiary); font-weight: 500; }
.api-model-select {
  padding: var(--spacing-1) var(--spacing-2); border-radius: var(--radius-sm);
  background: var(--color-bg-elevated); border: 1px solid var(--color-border-subtle);
  color: var(--color-text-primary); font-size: var(--text-sm); outline: none; cursor: pointer;
}

.api-help-links {
  display: flex; gap: var(--spacing-3); padding-top: var(--spacing-2);
  border-top: 1px solid var(--color-border-subtle);
}
.api-help-link {
  font-size: var(--text-xs); color: var(--color-primary); cursor: pointer;
  text-decoration: none; display: flex; align-items: center; gap: var(--spacing-1);
}
.api-help-link:hover { text-decoration: underline; }

.status-dot {
  width: 6px; height: 6px; border-radius: 50%;
  background: var(--color-success); display: inline-block;
  animation: pulse-glow 2s ease-in-out infinite;
}
.status-dot.offline { background: var(--color-text-tertiary); animation: none; }
</style>
