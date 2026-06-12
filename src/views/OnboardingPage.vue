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

// 跳过向导
function skipOnboarding() {
  appStore.completeOnboarding()
  router.push('/ai-create')
}

// 测试 API Key
async function testConnection() {
  if (!apiKeyInput.value.trim()) return
  testing.value = true
  testResult.value = 'idle'
  // 模拟测试连接（实际由 Tauri IPC 调用）
  await new Promise((r) => setTimeout(r, 1500))
  const clean = apiKeysStore.sanitizeKey(apiKeyInput.value)
  if (clean.length > 10) {
    apiKeysStore.saveKey('volcano', clean)
    apiKeysStore.setStatus('volcano', 'connected')
    testResult.value = 'success'
  } else {
    testResult.value = 'failed'
  }
  testing.value = false
}

// 进入快速上手
function goToQuickStart() {
  currentStep.value = 2
}

// 完成向导
function finishOnboarding() {
  appStore.completeOnboarding()
  router.push('/ai-create')
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
              🔴 {{ t('wizard.apiSetup.disconnected') }}
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
          <div class="preset-cards">
            <button class="preset-card" @click="finishOnboarding">
              {{ t('wizard.quickStart.preset1') }}
            </button>
            <button class="preset-card" @click="finishOnboarding">
              {{ t('wizard.quickStart.preset2') }}
            </button>
            <button class="preset-card" @click="finishOnboarding">
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
    oklch(99.5% 0.002 163);
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
.wizard-progress-dot.active { background: var(--color-primary); }
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
.wizard-visual {
  width: 80px;
  height: 80px;
  margin: 0 auto var(--spacing-5);
  border-radius: var(--radius-xl);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 36px;
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
  background: linear-gradient(135deg, oklch(75% 0.16 90), oklch(70% 0.17 163));
  color: white;
  border: none;
  font-family: var(--font-display);
  font-size: var(--text-sm);
  font-weight: 600;
  cursor: pointer;
  transition: all 0.25s ease;
}
.btn-wizard-primary:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 16px oklch(70% 0.17 163 / 0.3);
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
