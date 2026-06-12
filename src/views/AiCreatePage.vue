<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useApiKeysStore } from '@/stores/api-keys'
import { useSettingsStore } from '@/stores/settings'

const { t } = useI18n()
const apiKeysStore = useApiKeysStore()
const settings = useSettingsStore()

const prompt = ref('')
const style = ref('auto')
const model = ref(settings.defaultStep1Model)
const generating = ref(false)

const styles = [
  { key: 'auto', label: t('ai.styleAuto') },
  { key: 'cyberpunk', label: t('ai.styleCyberpunk') },
  { key: 'ink', label: t('ai.styleInk') },
  { key: 'anime', label: t('ai.styleAnime') },
  { key: 'minimal', label: t('ai.styleMinimal') },
]

function generate() {
  if (!prompt.value.trim()) return
  generating.value = true
  // 后续阶段接入 AI 管线
}
</script>

<template>
  <div class="page-ai-create">
    <!-- 标题 -->
    <div class="create-header">
      <h1>{{ t('ai.title') }}</h1>
      <span class="subtitle">{{ t('ai.subtitle') }}</span>
    </div>

    <!-- Prompt 输入区 -->
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
            <button class="prompt-chip">
              <span class="chip-icon">🖼️</span>
              {{ t('ai.referenceHint') }}
            </button>
          </div>
          <button class="btn-generate" :disabled="!prompt.trim() || generating" @click="generate">
            <span class="gen-icon">✨</span>
            {{ generating ? t('ai.analyzing') : t('ai.generateBtn') }}
          </button>
        </div>
      </div>
    </div>

    <!-- 风格 & 模型配置 -->
    <div class="config-row">
      <div class="config-group">
        <span class="config-label">{{ t('ai.styleLabel') }}</span>
        <div class="style-pills">
          <button
            v-for="s in styles"
            :key="s.key"
            class="style-pill"
            :class="{ active: style === s.key }"
            @click="style = s.key"
          >
            {{ s.label }}
          </button>
        </div>
      </div>
      <div class="config-group">
        <span class="config-label">{{ t('ai.modelLabel') }}</span>
        <select v-model="model" class="config-select">
          <option value="doubao-2.0-lite-32k">doubao-2.0-lite-32k</option>
          <option value="doubao-2.0-vision">doubao-2.0-vision</option>
        </select>
      </div>
      <div class="config-group">
        <span class="config-label">{{ t('ai.countLabel') }}</span>
        <select v-model.number="settings.defaultGenerationCount" class="config-select">
          <option :value="1">1</option>
          <option :value="3">3</option>
          <option :value="5">5</option>
        </select>
      </div>
      <div class="config-group">
        <span class="config-label">{{ t('ai.resolutionLabel') }}</span>
        <select v-model="settings.defaultResolution" class="config-select">
          <option value="1080P">1080P</option>
          <option value="2K">2K</option>
          <option value="4K">4K</option>
        </select>
      </div>
    </div>

    <!-- 未配置 API Key 时的引导 -->
    <div v-if="!apiKeysStore.hasConfiguredKey" class="empty-guide">
      <div class="guide-visual">✨</div>
      <h3>{{ t('ai.emptyGuide') }}</h3>
      <p>{{ t('ai.emptyGuideCta') }}</p>
    </div>

    <!-- 生成中状态 -->
    <div v-if="generating" class="generating-state">
      <div class="gen-spinner" />
      <span class="gen-text">{{ t('ai.analyzing') }}</span>
      <span class="gen-step">{{ t('ai.step1of2') }}</span>
    </div>
  </div>
</template>

<style scoped>
.page-ai-create { display: flex; flex-direction: column; gap: var(--spacing-6); max-width: 860px; }

.create-header { display: flex; align-items: baseline; gap: var(--spacing-3); }
.create-header h1 {
  font-family: var(--font-display);
  font-size: var(--text-2xl);
  font-weight: 600;
  letter-spacing: -0.02em;
  color: var(--color-text-primary);
}
.subtitle { font-size: var(--text-sm); color: var(--color-text-secondary); }

/* Prompt 输入 */
.prompt-area { position: relative; }
.prompt-input-wrap {
  position: relative;
  border-radius: var(--radius-lg);
  background: oklch(100% 0 0 / 0.75);
  backdrop-filter: blur(16px) saturate(1.3);
  border: 1px solid oklch(92% 0.005 163 / 0.6);
  overflow: hidden;
  transition: border-color 0.25s ease;
}
.prompt-input-wrap:focus-within { border-color: var(--color-primary); }
.prompt-input-wrap:focus-within::before {
  content: '';
  position: absolute; inset: -1px; border-radius: inherit; padding: 1px;
  background: linear-gradient(135deg, oklch(75% 0.16 90), oklch(70% 0.17 163), oklch(75% 0.12 200), oklch(75% 0.16 90));
  background-size: 300% 300%;
  animation: aurora-shift 4s ease infinite;
  -webkit-mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
  -webkit-mask-composite: xor;
  mask-composite: exclude;
  z-index: 1; pointer-events: none;
}

.prompt-textarea {
  width: 100%; min-height: 100px;
  background: transparent; border: none; outline: none; resize: vertical;
  padding: var(--spacing-4) var(--spacing-5);
  font-family: var(--font-body); font-size: var(--text-base);
  color: var(--color-text-primary); line-height: 1.8;
}
.prompt-textarea::placeholder { color: var(--color-text-tertiary); }

.prompt-footer {
  display: flex; align-items: center; justify-content: space-between;
  padding: var(--spacing-2) var(--spacing-3) var(--spacing-3);
}
.prompt-options { display: flex; gap: var(--spacing-2); }

/* 配置行 */
.config-row { display: flex; gap: var(--spacing-4); flex-wrap: wrap; }
.config-group { display: flex; flex-direction: column; gap: var(--spacing-2); }
.config-label {
  font-size: var(--text-xs); font-weight: 500;
  color: var(--color-text-tertiary); letter-spacing: 0.03em;
}
.style-pills { display: flex; gap: var(--spacing-1); }

/* 生成中 */
.generating-state {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  padding: var(--spacing-12) var(--spacing-4); gap: var(--spacing-4);
}
.gen-spinner {
  width: 48px; height: 48px; border-radius: 50%;
  border: 2px solid var(--color-border-subtle);
  border-top-color: var(--color-primary);
  animation: spin 1s linear infinite; position: relative;
}
.gen-spinner::after {
  content: ''; position: absolute; inset: 4px; border-radius: 50%;
  border: 2px solid transparent; border-top-color: var(--color-accent);
  animation: spin 1.5s linear infinite reverse;
}
.gen-text { font-size: var(--text-sm); color: var(--color-text-secondary); }
.gen-step { font-size: var(--text-xs); color: var(--color-text-tertiary); }

/* 空引导 */
.empty-guide {
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  padding: var(--spacing-16) var(--spacing-4); text-align: center;
}
.guide-visual {
  width: 120px; height: 120px; border-radius: var(--radius-xl);
  background: var(--color-bg-elevated); border: 1px solid var(--color-border-subtle);
  display: flex; align-items: center; justify-content: center;
  font-size: 40px; margin-bottom: var(--spacing-5); position: relative; overflow: hidden;
}
.guide-visual::before {
  content: ''; position: absolute; inset: -1px; border-radius: inherit; padding: 1px;
  background: linear-gradient(135deg, oklch(75% 0.16 90), oklch(70% 0.17 163), oklch(75% 0.12 200));
  background-size: 300% 300%;
  animation: aurora-shift 4s ease infinite;
  -webkit-mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
  -webkit-mask-composite: xor; mask-composite: exclude;
}
.empty-guide h3 {
  font-family: var(--font-display); font-size: var(--text-lg); font-weight: 600;
  color: var(--color-text-primary); margin-bottom: var(--spacing-2);
}
.empty-guide p { font-size: var(--text-sm); color: var(--color-text-secondary); max-width: 400px; }
</style>
