<script setup lang="ts">
// 首次启动向导 (OB-001~004)
// 3 步流程：欢迎 → API 配置 → 快速上手
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAppStore } from '@/stores/app'
import { useApiKeysStore } from '@/stores/api-keys'

const { t } = useI18n()
const router = useRouter()
const appStore = useAppStore()
const apiKeysStore = useApiKeysStore()

const currentStep = ref(0) // 0=欢迎, 1=API配置, 2=快速上手
const apiKeyInput = ref('')
const testing = ref(false)
const testResult = ref<'idle' | 'success' | 'failed'>('idle')
const testError = ref('')

// 跳过向导（OB-004：本地模式）
function skipOnboarding() {
  appStore.completeOnboarding()
  router.push('/library')
}

// 测试 API Key（API-002：真实 IPC 调用）
async function testConnection() {
  if (!apiKeyInput.value.trim()) return
  testing.value = true
  testResult.value = 'idle'
  testError.value = ''

  try {
    const clean = apiKeysStore.sanitizeKey(apiKeyInput.value)
    // 先加密并持久化 Key
    await apiKeysStore.persistKey('volcano', clean)

    // 发送真实测试请求
    const result = await apiKeysStore.testConnection('volcano')

    if (result.success) {
      testResult.value = 'success'
    } else {
      testResult.value = 'failed'
      testError.value = result.error || t('toast.connectionFailed')
    }
  } catch (e) {
    testResult.value = 'failed'
    testError.value = t('toast.networkError')
  }

  testing.value = false
}

// 打开火山引擎注册页面（OB-002）
async function openRegisterPage() {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('open_url', { url: 'https://console.volcengine.com/ark/region:ark+cn-beijing/overview' })
  } catch {
    // 降级：通过 window.open 打开
    window.open('https://console.volcengine.com/ark/region:ark+cn-beijing/overview', '_blank')
  }
}

// 进入快速上手
function goToQuickStart() {
  currentStep.value = 2
}

// 完成向导
function finishOnboarding() {
  appStore.completeOnboarding()
  router.push('/library')
}

// 一键生成：用预设 prompt 触发 AI 管线 (OB-003)
function startWithPreset(presetKey: string) {
  appStore.completeOnboarding()
  // 将预设 prompt 传递给 AI 面板，通过 query 参数携带
  router.push({ path: '/library', query: { ai: 'open', preset: presetKey } })
}
</script>

<template>
  <div class="wizard-overlay">
    <div class="wizard-container">
      <!-- 进度条 -->
      <div class="wizard-progress">
        <div
          v-for="i in 3"
          :key="i"
          class="wizard-progress-dot"
          :class="{ active: currentStep >= i - 1, done: currentStep > i - 1 }"
        />
      </div>

      <div class="wizard-card">
        <!-- Step 0: 欢迎 (OB-001) -->
        <template v-if="currentStep === 0">
          <div class="wizard-visual">✨</div>
          <h1 class="wizard-title">{{ t('wizard.welcome.title') }}</h1>
          <p class="wizard-subtitle">{{ t('wizard.welcome.subtitle') }}</p>
          <div class="wizard-actions">
            <button class="btn-wizard-primary" @click="currentStep = 1">
              {{ t('wizard.welcome.startBtn') }}
            </button>
            <button class="btn-wizard-secondary" @click="skipOnboarding">
              {{ t('wizard.welcome.skipBtn') }}
            </button>
          </div>
        </template>

        <!-- Step 1: API 配置 (OB-002) -->
        <template v-if="currentStep === 1">
          <div class="wizard-api-setup">
            <h1 class="wizard-title">{{ t('wizard.apiSetup.title') }}</h1>
            <p class="wizard-subtitle">{{ t('wizard.apiSetup.subtitle') }}</p>
            <div class="wizard-step-list">
              <div class="step-item">
                <span class="step-num">1</span>
                <span>{{ t('wizard.apiSetup.step1') }}</span>
                <button class="step-link" @click="openRegisterPage">
                  {{ t('wizard.apiSetup.registerLink') }}
                </button>
              </div>
              <div class="step-item">
                <span class="step-num">2</span>
                <span>{{ t('wizard.apiSetup.step2') }}</span>
              </div>
              <div class="step-item">
                <span class="step-num">3</span>
                <span>{{ t('wizard.apiSetup.step3') }}</span>
              </div>
            </div>
            <div class="api-key-row">
              <input
                v-model="apiKeyInput"
                type="password"
                class="api-key-input"
                :placeholder="t('apiKeys.keyPlaceholder')"
              />
              <button class="btn-test" :disabled="testing || !apiKeyInput.trim()" @click="testConnection">
                {{ testing ? '...' : t('wizard.apiSetup.testBtn') }}
              </button>
            </div>
            <p v-if="testResult === 'success'" class="test-msg success">
              🟢 {{ t('wizard.apiSetup.connected') }}
            </p>
            <p v-if="testResult === 'failed'" class="test-msg failed">
              🔴 {{ testError || t('wizard.apiSetup.disconnected') }}
            </p>
            <p class="register-hint">💡 {{ t('wizard.apiSetup.registerHint') }}</p>
          </div>
          <div class="wizard-actions">
            <button
              class="btn-wizard-primary"
              :disabled="testResult !== 'success'"
              @click="goToQuickStart"
            >
              {{ t('common.next') }}
            </button>
            <button class="btn-wizard-secondary" @click="skipOnboarding">
              {{ t('common.skip') }}
            </button>
          </div>
        </template>

        <!-- Step 2: 快速上手 (OB-003) -->
        <template v-if="currentStep === 2">
          <h1 class="wizard-title">{{ t('wizard.quickStart.title') }}</h1>
          <p class="wizard-subtitle">{{ t('wizard.quickStart.subtitle') }}</p>
          <div class="wizard-compat-tip">
            <p class="wizard-compat-title">{{ t('wizard.quickStart.compatTitle') }}</p>
            <ul class="wizard-compat-list">
              <li>{{ t('wizard.quickStart.compat1') }}</li>
              <li>{{ t('wizard.quickStart.compat2') }}</li>
              <li>{{ t('wizard.quickStart.compat3') }}</li>
            </ul>
          </div>
          <div class="preset-cards">
            <button class="preset-card" @click="startWithPreset('preset1')">
              {{ t('wizard.quickStart.preset1') }}
            </button>
            <button class="preset-card" @click="startWithPreset('preset2')">
              {{ t('wizard.quickStart.preset2') }}
            </button>
            <button class="preset-card" @click="startWithPreset('preset3')">
              {{ t('wizard.quickStart.preset3') }}
            </button>
          </div>
          <div class="wizard-actions">
            <button class="btn-wizard-secondary" @click="finishOnboarding">
              {{ t('common.skip') }}
            </button>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.wizard-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  background:
    radial-gradient(ellipse 130% 110% at 15% 85%, oklch(88% 0.07 170 / 0.35) 0%, transparent 55%),
    radial-gradient(ellipse 110% 130% at 85% 8%, oklch(90% 0.05 150 / 0.3) 0%, transparent 50%),
    radial-gradient(ellipse 150% 90% at 50% 45%, oklch(89% 0.04 163 / 0.25) 0%, transparent 60%),
    radial-gradient(ellipse 100% 110% at 78% 80%, oklch(92% 0.03 180 / 0.2) 0%, transparent 45%),
    radial-gradient(ellipse 80% 90% at 60% 70%, oklch(91% 0.035 90 / 0.18) 0%, transparent 50%),
    oklch(99.5% 0.002 163);
  background-size: 200% 200%, 180% 180%, 220% 220%, 160% 160%, 210% 210%, 100% 100%;
  animation: wizard-hazy-flow 28s ease-in-out infinite;
}
.wizard-container { width: 560px; max-height: 90vh; display: flex; flex-direction: column; }
.wizard-progress {
  display: flex;
  gap: var(--spacing-2);
  padding: 0 var(--spacing-4);
  margin-bottom: var(--spacing-6);
}
.wizard-progress-dot {
  height: 3px;
  flex: 1;
  border-radius: var(--radius-full);
  background: var(--color-border-subtle);
  transition: background 0.25s ease;
}
.wizard-progress-dot.active { background: var(--gradient-primary); }
.wizard-progress-dot.done { background: var(--color-primary-dark); }
.wizard-card {
  background: oklch(99% 0.004 163 / 0.72);
  backdrop-filter: blur(32px) saturate(1.3);
  border: 1px solid oklch(95% 0.005 163 / 0.5);
  border-radius: var(--radius-xl);
  padding: var(--spacing-10) var(--spacing-8);
  text-align: center;
  position: relative;
  overflow: hidden;
  box-shadow: 0 8px 40px oklch(0% 0 0 / 0.06), 0 1px 0 oklch(100% 0 0 / 0.4) inset;
}
.wizard-card::before {
  content: '';
  position: absolute;
  top: -80px;
  left: 50%;
  transform: translateX(-50%);
  width: 300px;
  height: 300px;
  border-radius: 50%;
  background: radial-gradient(circle, oklch(62% 0.12 178 / 0.15), transparent 70%);
  pointer-events: none;
}
.wizard-visual {
  width: 80px;
  height: 80px;
  margin: 0 auto var(--spacing-5);
  border-radius: var(--radius-xl);
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 36px;
}
.wizard-visual::before {
  content: '';
  position: absolute;
  inset: -2px;
  border-radius: inherit;
  padding: 2px;
  background: linear-gradient(135deg, oklch(75% 0.16 90), oklch(70% 0.17 163), oklch(75% 0.12 200));
  background-size: 300% 300%;
  animation: aurora-shift 4s ease infinite;
  -webkit-mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
  -webkit-mask-composite: xor;
  mask-composite: exclude;
}
.wizard-title {
  font-family: var(--font-display);
  font-size: var(--text-2xl);
  font-weight: 600;
  color: var(--color-text-primary);
  margin-bottom: var(--spacing-2);
}
.wizard-subtitle {
  font-size: var(--text-sm);
  color: var(--color-text-secondary);
  line-height: 1.8;
  margin-bottom: var(--spacing-6);
}
.wizard-actions {
  display: flex;
  gap: var(--spacing-3);
  justify-content: center;
}
.btn-wizard-primary {
  padding: var(--spacing-2) var(--spacing-6);
  border-radius: var(--radius-md);
  background: var(--gradient-primary);
  color: white;
  border: none;
  font-family: var(--font-display);
  font-size: var(--text-sm);
  font-weight: 600;
  cursor: pointer;
  transition: all 0.25s ease;
}
.btn-wizard-primary:hover:not(:disabled) {
  background: var(--gradient-primary-hover);
  transform: translateY(-1px);
  box-shadow: 0 4px 16px rgba(0, 131, 54, 0.3);
}
.btn-wizard-primary:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-wizard-secondary {
  padding: var(--spacing-2) var(--spacing-5);
  border-radius: var(--radius-md);
  background: transparent;
  border: 1px solid var(--color-border-default);
  color: var(--color-text-secondary);
  font-size: var(--text-sm);
  cursor: pointer;
  transition: all 0.12s ease;
}
.btn-wizard-secondary:hover { border-color: var(--color-border-strong); color: var(--color-text-primary); }
.wizard-api-setup { text-align: left; }
.wizard-step-list { display: flex; flex-direction: column; gap: var(--spacing-3); margin-bottom: var(--spacing-4); }
.step-item { display: flex; align-items: center; gap: var(--spacing-2); font-size: var(--text-sm); color: var(--color-text-secondary); }
.step-num {
  width: 24px; height: 24px; border-radius: 50%;
  background: var(--color-primary-surface); color: var(--color-primary-dark);
  display: flex; align-items: center; justify-content: center;
  font-size: var(--text-xs); font-weight: 600; flex-shrink: 0;
}
.step-link {
  font-size: var(--text-xs);
  color: var(--color-primary-light);
  background: none;
  border: none;
  cursor: pointer;
  text-decoration: underline;
  padding: 0;
  margin-left: var(--spacing-1);
}
.step-link:hover { color: var(--color-primary-dark); }
.api-key-row { display: flex; gap: var(--spacing-2); margin-top: var(--spacing-3); }
.api-key-input {
  flex: 1; padding: var(--spacing-2) var(--spacing-3);
  border-radius: var(--radius-md); background: var(--color-bg-base);
  border: 1px solid var(--color-border-subtle); color: var(--color-text-primary);
  font-family: var(--font-mono); font-size: var(--text-sm); outline: none;
}
.api-key-input:focus { border-color: var(--color-primary); }
.btn-test {
  padding: var(--spacing-2) var(--spacing-4); border-radius: var(--radius-md);
  border: 1px solid var(--color-border-subtle); background: var(--color-bg-elevated);
  color: var(--color-text-primary); font-size: var(--text-sm); font-weight: 500;
  cursor: pointer; white-space: nowrap;
}
.btn-test:hover:not(:disabled) { border-color: var(--color-primary); color: var(--color-primary-light); }
.test-msg { margin-top: var(--spacing-2); font-size: var(--text-sm); }
.test-msg.success { color: var(--color-success); }
.test-msg.failed { color: var(--color-error); }
.register-hint { margin-top: var(--spacing-3); font-size: var(--text-xs); color: var(--color-text-tertiary); }
.wizard-compat-tip {
  margin-bottom: var(--spacing-5);
  padding: var(--spacing-3) var(--spacing-4);
  border-radius: var(--radius-md);
  background: oklch(97% 0.01 163 / 0.8);
  border: 1px solid var(--color-border-subtle);
  text-align: left;
}
.wizard-compat-title {
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--color-text-primary);
  margin-bottom: var(--spacing-2);
}
.wizard-compat-list {
  margin: 0;
  padding-left: var(--spacing-5);
  font-size: var(--text-xs);
  color: var(--color-text-secondary);
  line-height: 1.7;
}
.preset-cards { display: flex; flex-direction: column; gap: var(--spacing-3); margin-bottom: var(--spacing-4); }
.preset-card {
  padding: var(--spacing-3) var(--spacing-4);
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border-subtle);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
  font-size: var(--text-sm);
  text-align: left;
  cursor: pointer;
  transition: all 0.12s ease;
  font-family: var(--font-body);
}
.preset-card:hover {
  border-color: var(--color-primary);
  color: var(--color-primary-dark);
  background: var(--color-primary-surface);
}
</style>
