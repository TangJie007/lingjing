<script setup lang="ts">
// AI 创作底部面板 — 两步 AI 管线 (AI-001~003, AI-002)
// Step 1: 意图分析 → BuildPlan
// Step 1.5: 方案预览与确认
// Step 2: 壁纸生成 → 结果展示

import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useApiKeysStore } from '@/stores/api-keys'
import { useSettingsStore } from '@/stores/settings'
import { useWallpaperStore } from '@/stores/wallpaper'
import { useAiPipelineStore } from '@/stores/ai-pipeline'

const open = defineModel<boolean>('open', { default: false })
const props = defineProps<{
  presetPrompt?: string
}>()
const { t } = useI18n()
const apiKeysStore = useApiKeysStore()
const settings = useSettingsStore()
const wallpaperStore = useWallpaperStore()
const pipeline = useAiPipelineStore()

const prompt = ref('')
const style = ref('auto')
const model = ref(settings.defaultStep2Model)
const editedPlan = ref('')

const styles = [
  { key: 'auto', label: t('ai.styleAuto') },
  { key: 'cyberpunk', label: t('ai.styleCyberpunk') },
  { key: 'ink', label: t('ai.styleInk') },
  { key: 'anime', label: t('ai.styleAnime') },
  { key: 'minimal', label: t('ai.styleMinimal') },
]

// 当 panel 打开且有预设 prompt 时，自动填入并触发
watch(open, (val) => {
  if (val) {
    if (props.presetPrompt) {
      prompt.value = props.presetPrompt
      // 自动触发分析
      setTimeout(() => startAnalysis(), 300)
    }
  } else {
    pipeline.reset()
    prompt.value = ''
  }
})

function close() {
  open.value = false
}

// Step 1: 意图分析
async function startAnalysis() {
  if (!prompt.value.trim() || pipeline.isAnalyzing || pipeline.isGenerating) return

  try {
    const apiKey = await apiKeysStore.getDecryptedKey('volcano')
    if (!apiKey) {
      pipeline.error = t('toast.apiKeyInvalid')
      return
    }

    // 拼接风格到 prompt
    let fullPrompt = prompt.value.trim()
    if (style.value !== 'auto') {
      const styleLabel = styles.find((s) => s.key === style.value)?.label || style.value
      fullPrompt = `${fullPrompt}，风格：${styleLabel}`
    }

    await pipeline.analyzeIntent(apiKey, fullPrompt)

    // 填充编辑区
    if (pipeline.plan) {
      editedPlan.value = pipeline.plan.optimized_prompt
    }
  } catch {
    // 错误已存入 pipeline.error
  }
}

// Step 2: 壁纸生成
async function startGeneration() {
  if (!pipeline.plan || pipeline.isGenerating) return

  // 如果有编辑，先更新 plan
  if (editedPlan.value !== pipeline.plan.optimized_prompt) {
    pipeline.updatePlan({ optimized_prompt: editedPlan.value })
  }

  try {
    const apiKey = await apiKeysStore.getDecryptedKey('volcano')
    if (!apiKey) {
      pipeline.error = t('toast.apiKeyInvalid')
      return
    }

    const result = await pipeline.generateWallpaper(
      apiKey,
      model.value,
      settings.defaultGenerationCount,
      settings.defaultResolution
    )

    // 将生成结果加入壁纸库 (WL-002)
    for (const img of result.images) {
      wallpaperStore.addAiWallpaper({
        filename: img.filename,
        path: img.path,
        mediaType: 'image',
        format: 'png',
        width: img.width,
        height: img.height,
        fileSize: img.file_size,
        source: 'ai',
        prompt: pipeline.plan?.optimized_prompt,
        plan: JSON.stringify(pipeline.plan),
        model: result.model,
        cost: undefined, // V1 暂不追踪费用
      })
    }
  } catch {
    // 错误已存入 pipeline.error
  }
}

// 重试
function retry() {
  if (pipeline.isPlanReady) {
    startGeneration()
  } else {
    pipeline.reset()
  }
}

function selectStyle(key: string) {
  style.value = key
}

// 从面板直接触发（兼容旧入口）
function generate() {
  if (pipeline.isIdle) {
    startAnalysis()
  }
}

// 关闭面板时触发（如果有新壁纸）
function applyWallpaper(imagePath: string) {
  // 触发壁纸应用：将图片路径传递给 wallpaper store
  const wallpaper = wallpaperStore.wallpapers.find((w) => w.path === imagePath)
  if (wallpaper) {
    wallpaperStore.setWallpaper(wallpaper.id)
    close()
  }
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
          <path d="M18 6L6 18" /><path d="M6 6l12 12" />
        </svg>
      </button>
    </div>

    <div class="ai-panel-body">
      <div class="page-ai-create">
        <!-- 空状态：未配置 API Key -->
        <div v-if="!apiKeysStore.hasConfiguredKey" class="empty-guide">
          <div class="guide-visual">✨</div>
          <h3>{{ t('ai.emptyGuide') }}</h3>
          <p>{{ t('ai.emptyGuideCta') }}</p>
        </div>

        <!-- Step 0: 输入阶段 -->
        <template v-if="apiKeysStore.hasConfiguredKey && pipeline.isIdle">
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
                  :disabled="!prompt.trim()"
                  @click="generate"
                >
                  <span class="gen-icon">✨</span>
                  {{ t('ai.generateBtn') }}
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
        </template>

        <!-- Step 1: 分析中 -->
        <div v-if="pipeline.isAnalyzing" class="generating-state">
          <div class="gen-spinner" />
          <span class="gen-text">{{ t('ai.analyzing') }}</span>
          <span class="gen-step">{{ t('ai.step1of2') }}</span>
        </div>

        <!-- Step 1.5: 方案预览与确认 (AI-003) -->
        <div v-if="pipeline.isPlanReady && pipeline.plan" class="plan-preview">
          <h3 class="plan-title">{{ t('ai.planTitle') }}</h3>

          <div class="plan-card">
            <div class="plan-field">
              <label class="plan-label">Prompt</label>
              <textarea
                v-model="editedPlan"
                class="plan-textarea"
                rows="4"
              />
            </div>
            <div class="plan-tags">
              <span class="plan-label">{{ t('ai.styleLabel') }}</span>
              <div class="style-tags">
                <span v-for="tag in pipeline.plan.style_tags" :key="tag" class="tag-badge">
                  {{ tag }}
                </span>
              </div>
            </div>
            <div class="plan-meta">
              <div class="plan-meta-item">
                <span class="plan-label">{{ t('ai.colorScheme') }}</span>
                <span class="plan-value">{{ pipeline.plan.color_scheme }}</span>
              </div>
              <div class="plan-meta-item">
                <span class="plan-label">{{ t('ai.composition') }}</span>
                <span class="plan-value">{{ pipeline.plan.composition }}</span>
              </div>
            </div>
          </div>

          <div class="plan-actions">
            <button class="btn-generate" @click="startGeneration">
              <span class="gen-icon">🎨</span>
              {{ t('ai.planConfirm') }}
            </button>
            <button class="btn-back" @click="pipeline.reset()">
              {{ t('ai.planEdit') }}
            </button>
          </div>
        </div>

        <!-- Step 2: 生成中 -->
        <div v-if="pipeline.isGenerating" class="generating-state">
          <div class="gen-spinner" />
          <span class="gen-text">{{ t('ai.generating') }}</span>
          <span class="gen-step">{{ t('ai.step2of2') }}</span>
        </div>

        <!-- Step 2: 生成完成 — 结果展示 -->
        <div v-if="pipeline.isDone && pipeline.results.length > 0" class="result-section">
          <h3 class="result-title">{{ t('ai.resultTitle') }}</h3>
          <div class="result-grid">
            <div
              v-for="(img, idx) in pipeline.results"
              :key="idx"
              class="result-card"
            >
              <div class="result-img-wrap">
                <img
                  :src="img.path"
                  :alt="img.filename"
                  class="result-img"
                  loading="lazy"
                />
              </div>
              <div class="result-info">
                <span class="result-size">{{ img.width }}×{{ img.height }}</span>
                <div class="result-actions">
                  <button class="btn-apply" @click="applyWallpaper(img.path)">
                    {{ t('ai.applyBtn') }}
                  </button>
                </div>
              </div>
            </div>
          </div>
          <div class="result-footer">
            <button class="btn-generate" @click="pipeline.reset()">
              {{ t('ai.newGenerate') }}
            </button>
          </div>
        </div>

        <!-- 错误状态 -->
        <div v-if="pipeline.hasError && !pipeline.isAnalyzing && !pipeline.isGenerating" class="error-state">
          <div class="error-icon">⚠️</div>
          <p class="error-text">{{ pipeline.error }}</p>
          <button class="btn-generate" @click="retry">
            {{ t('common.retry') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 方案预览 (AI-003) */
.plan-preview {
  padding: var(--spacing-4) 0;
}
.plan-title {
  font-family: var(--font-display);
  font-size: var(--text-lg);
  font-weight: 600;
  color: var(--color-text-primary);
  margin-bottom: var(--spacing-4);
}
.plan-card {
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-lg);
  padding: var(--spacing-4);
  margin-bottom: var(--spacing-4);
}
.plan-field {
  margin-bottom: var(--spacing-3);
}
.plan-label {
  display: block;
  font-size: var(--text-xs);
  font-weight: 600;
  color: var(--color-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: var(--spacing-1);
}
.plan-textarea {
  width: 100%;
  padding: var(--spacing-2) var(--spacing-3);
  border-radius: var(--radius-md);
  background: var(--color-bg-base);
  border: 1px solid var(--color-border-subtle);
  color: var(--color-text-primary);
  font-family: var(--font-body);
  font-size: var(--text-sm);
  line-height: 1.6;
  resize: vertical;
  outline: none;
}
.plan-textarea:focus {
  border-color: var(--color-primary);
}
.plan-tags {
  margin-bottom: var(--spacing-3);
}
.style-tags {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-1);
  margin-top: var(--spacing-1);
}
.tag-badge {
  padding: 2px var(--spacing-2);
  border-radius: var(--radius-full);
  background: var(--color-primary-surface);
  color: var(--color-primary-dark);
  font-size: var(--text-xs);
  font-weight: 500;
}
.plan-meta {
  display: flex;
  gap: var(--spacing-4);
}
.plan-meta-item {
  flex: 1;
}
.plan-value {
  font-size: var(--text-sm);
  color: var(--color-text-secondary);
  margin-top: var(--spacing-1);
}
.plan-actions {
  display: flex;
  gap: var(--spacing-3);
}
.btn-back {
  padding: var(--spacing-2) var(--spacing-4);
  border-radius: var(--radius-md);
  background: transparent;
  border: 1px solid var(--color-border-default);
  color: var(--color-text-secondary);
  font-size: var(--text-sm);
  cursor: pointer;
  transition: all 0.12s ease;
}
.btn-back:hover {
  border-color: var(--color-border-strong);
  color: var(--color-text-primary);
}

/* 结果展示 (AI-002) */
.result-section {
  padding: var(--spacing-4) 0;
}
.result-title {
  font-family: var(--font-display);
  font-size: var(--text-lg);
  font-weight: 600;
  color: var(--color-text-primary);
  margin-bottom: var(--spacing-4);
}
.result-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: var(--spacing-3);
  margin-bottom: var(--spacing-4);
}
.result-card {
  border-radius: var(--radius-md);
  overflow: hidden;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border-subtle);
  transition: all 0.2s ease;
}
.result-card:hover {
  border-color: var(--color-primary);
  box-shadow: 0 4px 12px rgba(0, 131, 54, 0.15);
}
.result-img-wrap {
  aspect-ratio: 16 / 9;
  overflow: hidden;
  background: var(--color-bg-deep);
}
.result-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.result-info {
  padding: var(--spacing-2);
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.result-size {
  font-size: var(--text-xs);
  color: var(--color-text-tertiary);
}
.btn-apply {
  padding: 2px var(--spacing-2);
  border-radius: var(--radius-sm);
  background: var(--gradient-primary);
  color: white;
  border: none;
  font-size: var(--text-xs);
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}
.btn-apply:hover {
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(0, 131, 54, 0.3);
}
.result-footer {
  display: flex;
  justify-content: center;
}

/* 错误状态 */
.error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: var(--spacing-8) var(--spacing-4);
  text-align: center;
  gap: var(--spacing-3);
}
.error-icon {
  font-size: 32px;
}
.error-text {
  font-size: var(--text-sm);
  color: var(--color-error);
  line-height: 1.6;
}

/* 原有样式 */
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
