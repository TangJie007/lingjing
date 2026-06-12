<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useApiKeysStore } from '@/stores/api-keys'
import { useSettingsStore } from '@/stores/settings'

const open = defineModel<boolean>('open', { default: false })
const { t } = useI18n()
const apiKeysStore = useApiKeysStore()
const settings = useSettingsStore()

const prompt = ref('')
const style = ref('auto')
const model = ref(settings.defaultStep2Model)
const generating = ref(false)

const styles = [
  { key: 'auto', label: t('ai.styleAuto') },
  { key: 'cyberpunk', label: t('ai.styleCyberpunk') },
  { key: 'ink', label: t('ai.styleInk') },
  { key: 'anime', label: t('ai.styleAnime') },
  { key: 'minimal', label: t('ai.styleMinimal') },
]

function close() {
  open.value = false
}

function generate() {
  if (!prompt.value.trim()) return
  generating.value = true
}

function selectStyle(key: string) {
  style.value = key
}
</script>

<template>
  <div class="ai-panel-overlay" :class="{ visible: open }" @click="close" />
  <div class="ai-panel" :class="{ visible: open }">
    <div class="ai-panel-header">
      <div class="ai-panel-brand">
        <svg width="20" height="20" viewBox="0 0 48 48" fill="none" style="flex-shrink:0">
          <defs>
            <linearGradient id="aip-fill" x1="4" y1="36" x2="44" y2="8" gradientUnits="userSpaceOnUse">
              <stop offset="0%" stop-color="#fbbf24" />
              <stop offset="25%" stop-color="#a3e635" />
              <stop offset="55%" stop-color="#34d399" />
              <stop offset="85%" stop-color="#22d3ee" />
              <stop offset="100%" stop-color="#67e8f9" />
            </linearGradient>
          </defs>
          <path d="M 24 3 C 14 3, 5 12, 5 24 C 5 35, 12 43, 20 45 C 28 47, 38 42, 41 33 C 44 24, 42 14, 34 8 C 30 5, 27 3, 24 3 Z" fill="url(#aip-fill)" />
        </svg>
        <span>{{ t('ai.title') }}</span>
      </div>
      <button class="ai-panel-close" @click="close">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <path d="M18 6L6 18" />
          <path d="M6 6l12 12" />
        </svg>
      </button>
    </div>

    <div class="ai-panel-body">
      <div class="page-ai-create">
        <div class="prompt-area">
          <div class="prompt-input-wrap">
            <textarea
              v-model="prompt"
              class="prompt-textarea"
              :placeholder="t('ai.placeholder')"
              rows="3"
            />
            <div class="prompt-footer">
              <div class="prompt-options">
                <button type="button" class="prompt-chip">
                  <span class="chip-icon">⊞</span>
                  {{ t('ai.referenceHint') }}
                </button>
                <button type="button" class="prompt-chip">
                  <span class="chip-icon">◉</span>
                  {{ t('ai.randomInspiration') }}
                </button>
                <button type="button" class="prompt-chip">
                  <span class="chip-icon">✎</span>
                  {{ t('ai.history') }}
                </button>
              </div>
              <button
                class="btn-generate"
                :disabled="!prompt.trim() || generating"
                @click="generate"
              >
                <span class="gen-icon">✨</span>
                {{ generating ? t('ai.analyzing') : t('ai.generateBtn') }}
              </button>
            </div>
          </div>
        </div>

        <div class="config-row">
          <div class="config-group">
            <span class="config-label">{{ t('ai.styleLabel') }}</span>
            <div class="style-pills">
              <button
                v-for="s in styles"
                :key="s.key"
                type="button"
                class="style-pill"
                :class="{ active: style === s.key }"
                @click="selectStyle(s.key)"
              >
                {{ s.label }}
              </button>
            </div>
          </div>
          <div class="config-group">
            <span class="config-label">{{ t('ai.modelLabel') }}</span>
            <select v-model="model" class="config-select">
              <option value="Seedream 4.0">Seedream 4.0</option>
              <option value="Seedream 5.0 lite">Seedream 5.0 lite</option>
            </select>
          </div>
          <div class="config-group">
            <span class="config-label">{{ t('ai.countLabel') }}</span>
            <select v-model.number="settings.defaultGenerationCount" class="config-select" style="min-width:80px">
              <option :value="1">1</option>
              <option :value="3">3</option>
              <option :value="5">5</option>
            </select>
          </div>
          <div class="config-group">
            <span class="config-label">{{ t('ai.resolutionLabel') }}</span>
            <select v-model="settings.defaultResolution" class="config-select" style="min-width:100px">
              <option value="1080P">1920×1080</option>
              <option value="2K">2560×1440</option>
              <option value="4K">3840×2160</option>
            </select>
          </div>
        </div>

        <div v-if="!apiKeysStore.hasConfiguredKey" class="empty-guide">
          <div class="guide-visual">✨</div>
          <h3>{{ t('ai.emptyGuide') }}</h3>
          <p>{{ t('ai.emptyGuideCta') }}</p>
        </div>

        <div v-if="generating" class="generating-state">
          <div class="gen-spinner" />
          <span class="gen-text">{{ t('ai.analyzing') }}</span>
          <span class="gen-step">{{ t('ai.step1of2') }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.empty-guide {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-8) var(--spacing-4);
  text-align: center;
}
.guide-visual {
  width: 80px;
  height: 80px;
  border-radius: var(--radius-xl);
  background: oklch(99% 0.004 163);
  border: 1px solid var(--color-border-subtle);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 32px;
  margin-bottom: var(--spacing-4);
}
.empty-guide h3 {
  font-family: var(--font-display);
  font-size: var(--text-base);
  font-weight: 600;
  color: var(--color-text-primary);
  margin-bottom: var(--spacing-2);
}
.empty-guide p { font-size: var(--text-sm); color: var(--color-text-secondary); }
</style>
