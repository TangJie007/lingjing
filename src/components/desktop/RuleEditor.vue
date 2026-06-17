<script setup lang="ts">
// 自动整理规则编辑器 (DO-004)
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { useDesktopOrganizerStore } from '@/stores/desktop-organizer'

const { t } = useI18n()
const orgStore = useDesktopOrganizerStore()

interface AutoRule {
  id: string; name: string; enabled: boolean; fileTypes: string[]
  keywords: string[]; targetPartition: string
}

const rules = ref<AutoRule[]>([])
const loading = ref(false)

onMounted(async () => {
  loading.value = true
  try {
    rules.value = await invoke<AutoRule[]>('get_auto_rules')
  } catch { /* 降级 */ }
  loading.value = false
})

async function toggleRule(rule: AutoRule) {
  rule.enabled = !rule.enabled
  await invoke('update_auto_rule', { ruleId: rule.id, enabled: rule.enabled, targetPartition: rule.targetPartition })
}

async function setTarget(rule: AutoRule, partitionId: string) {
  rule.targetPartition = partitionId
  await invoke('update_auto_rule', { ruleId: rule.id, targetPartition: partitionId })
}

async function applyAll() {
  await invoke('apply_auto_rules')
}
</script>

<template>
  <div class="rule-editor">
    <div v-if="loading" class="rule-loading">{{ t('common.loading') }}</div>
    <div v-else class="rule-list">
      <div v-for="rule in rules" :key="rule.id" class="rule-item">
        <label class="rule-label">
          <input type="checkbox" :checked="rule.enabled" @change="toggleRule(rule)" />
          <span>{{ rule.name }}</span>
        </label>
        <select
          v-if="rule.enabled"
          class="rule-select"
          :value="rule.targetPartition"
          @change="setTarget(rule, ($event.target as HTMLSelectElement).value)"
        >
          <option value="">{{ t('settings.selectWallpaper') }}</option>
          <option v-for="p in orgStore.partitions" :key="p.id" :value="p.id">{{ p.name }}</option>
        </select>
      </div>
    </div>
    <button class="org-btn primary" style="margin-top:12px" @click="applyAll">
      {{ t('desktop.startOrganizing') }}
    </button>
  </div>
</template>

<style scoped>
.rule-editor { padding: 16px; }
.rule-list { display: flex; flex-direction: column; gap: 12px; }
.rule-item { display: flex; align-items: center; gap: 12px; }
.rule-label { display: flex; align-items: center; gap: 8px; font-size: 13px; cursor: pointer; flex: 1; }
.rule-select {
  padding: 4px 8px; border-radius: 6px; border: 1px solid var(--color-border-subtle);
  background: var(--color-bg-elevated); font-size: 12px; min-width: 120px;
}
.org-btn {
  padding: 8px 16px; border-radius: 10px; border: 1px solid var(--color-border-subtle);
  background: var(--color-bg-elevated); color: var(--color-text-secondary);
  font-family: var(--font-ui); font-size: 13px; cursor: pointer;
}
.org-btn.primary { background: var(--gradient-primary); color: white; border: none; }
</style>
