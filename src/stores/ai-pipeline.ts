// AI 生成管线状态管理 (AI-001~003, WL-002)
//
// 管理两步 AI 管线的完整生命周期：
//   Step 1: 意图分析 → BuildPlan
//   Step 2: 壁纸生成 → GeneratedImage[]
//
// 状态机：idle → analyzing → plan_ready → generating → done

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// ---- 类型定义 ----

export interface BuildPlan {
  optimized_prompt: string
  style_tags: string[]
  color_scheme: string
  composition: string
}

export interface GeneratedImage {
  path: string
  filename: string
  width: number
  height: number
  file_size: number
}

export interface GenerateResult {
  images: GeneratedImage[]
  model: string
  resolution: string
}

export type PipelineStep = 'idle' | 'analyzing' | 'plan_ready' | 'generating' | 'done'

// ---- Store ----

export const useAiPipelineStore = defineStore('aiPipeline', () => {
  // 当前管线步骤
  const step = ref<PipelineStep>('idle')

  // Step 1 结果
  const plan = ref<BuildPlan | null>(null)

  // Step 2 结果
  const results = ref<GeneratedImage[]>([])

  // 生成使用的模型和分辨率
  const usedModel = ref('')
  const usedResolution = ref('')

  // 错误信息
  const error = ref('')

  // 用户原始输入
  const lastPrompt = ref('')
  const lastReferenceImage = ref<string | null>(null)

  // 计算属性
  const isIdle = computed(() => step.value === 'idle')
  const isAnalyzing = computed(() => step.value === 'analyzing')
  const isPlanReady = computed(() => step.value === 'plan_ready')
  const isGenerating = computed(() => step.value === 'generating')
  const isDone = computed(() => step.value === 'done')
  const hasError = computed(() => error.value !== '')

  // ---- Actions ----

  /// Step 1: 意图分析 (AI-001)
  async function analyzeIntent(
    apiKey: string,
    prompt: string,
    referenceImage?: string | null,
    imageMime?: string
  ): Promise<BuildPlan> {
    step.value = 'analyzing'
    error.value = ''
    lastPrompt.value = prompt
    lastReferenceImage.value = referenceImage ?? null

    try {
      const result = await invoke<BuildPlan>('ai_analyze', {
        apiKey,
        request: {
          prompt,
          reference_image: referenceImage ?? null,
          image_mime: imageMime ?? null,
        },
      })

      plan.value = result
      step.value = 'plan_ready'
      return result
    } catch (e) {
      const msg = typeof e === 'string' ? e : (e as Error).message || '分析失败'
      error.value = msg
      step.value = 'idle'
      throw e
    }
  }

  /// 编辑 plan（用户手动修改后）
  function updatePlan(updated: Partial<BuildPlan>) {
    if (!plan.value) return
    if (updated.optimized_prompt !== undefined) plan.value.optimized_prompt = updated.optimized_prompt
    if (updated.style_tags !== undefined) plan.value.style_tags = updated.style_tags
    if (updated.color_scheme !== undefined) plan.value.color_scheme = updated.color_scheme
    if (updated.composition !== undefined) plan.value.composition = updated.composition
  }

  /// Step 2: 壁纸生成 (AI-002)
  async function generateWallpaper(
    apiKey: string,
    model?: string,
    count?: number,
    resolution?: string
  ): Promise<GenerateResult> {
    if (!plan.value) throw new Error('构建方案为空，请先完成意图分析')

    step.value = 'generating'
    error.value = ''

    try {
      const result = await invoke<GenerateResult>('ai_generate', {
        app: null, // Tauri 会自动注入 AppHandle
        apiKey,
        request: {
          prompt: plan.value.optimized_prompt,
          model: model ?? null,
          count: count ?? 3,
          resolution: resolution ?? '1080P',
          style_tags: plan.value.style_tags,
          color_scheme: plan.value.color_scheme,
          composition: plan.value.composition,
        },
      })

      results.value = result.images
      usedModel.value = result.model
      usedResolution.value = result.resolution
      step.value = 'done'
      return result
    } catch (e) {
      const msg = typeof e === 'string' ? e : (e as Error).message || '生成失败'
      error.value = msg
      step.value = 'plan_ready' // 回退到 plan 状态，允许重试
      throw e
    }
  }

  /// 重置管线状态
  function reset() {
    step.value = 'idle'
    plan.value = null
    results.value = []
    error.value = ''
    lastPrompt.value = ''
    lastReferenceImage.value = null
    usedModel.value = ''
    usedResolution.value = ''
  }

  /// 从 plan_ready 状态重试生成
  function retryGenerate() {
    step.value = 'plan_ready'
    error.value = ''
  }

  return {
    // state
    step,
    plan,
    results,
    usedModel,
    usedResolution,
    error,
    lastPrompt,
    lastReferenceImage,
    // computed
    isIdle,
    isAnalyzing,
    isPlanReady,
    isGenerating,
    isDone,
    hasError,
    // actions
    analyzeIntent,
    updatePlan,
    generateWallpaper,
    reset,
    retryGenerate,
  }
})
