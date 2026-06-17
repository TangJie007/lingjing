<script setup lang="ts">
// 文件夹门户组件 (DO-002)
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'

const { t } = useI18n()

interface FileEntry {
  name: string; path: string; isDir: boolean; size: number
}

const folderPath = ref('')
const files = ref<FileEntry[]>([])
const loading = ref(false)
const error = ref('')

async function openFolder() {
  if (!folderPath.value) return
  loading.value = true
  error.value = ''
  try {
    const result = await invoke<any>('open_folder_portal', {
      partitionId: 'portal',
      folderPath: folderPath.value,
    })
    files.value = result.files || []
  } catch (e: any) {
    error.value = e?.toString() || '打开失败'
  } finally {
    loading.value = false
  }
}

async function selectFolder() {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const result = await open({ directory: true, multiple: false })
    if (result) {
      folderPath.value = result as string
      await openFolder()
    }
  } catch { /* 降级 */ }
}

function formatSize(bytes: number) {
  if (bytes > 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  if (bytes > 1024) return `${(bytes / 1024).toFixed(0)} KB`
  return `${bytes} B`
}
</script>

<template>
  <div class="folder-portal">
    <div class="portal-header">
      <input
        v-model="folderPath"
        class="portal-input"
        placeholder="选择文件夹..."
        readonly
        @click="selectFolder"
      />
      <button class="org-btn" @click="selectFolder">{{ t('common.browse') }}</button>
      <button class="org-btn primary" :disabled="!folderPath" @click="openFolder">
        {{ t('common.confirm') }}
      </button>
    </div>
    <div v-if="loading" class="portal-loading">{{ t('common.loading') }}</div>
    <div v-else-if="error" class="portal-error">{{ error }}</div>
    <div v-else-if="files.length" class="portal-files">
      <div v-for="f in files" :key="f.path" class="portal-file">
        <span>{{ f.isDir ? '📁' : '📄' }}</span>
        <span class="file-name">{{ f.name }}</span>
        <span class="file-size">{{ formatSize(f.size) }}</span>
      </div>
    </div>
    <div v-else class="portal-empty">{{ t('desktop.noPartitions') }}</div>
  </div>
</template>

<style scoped>
.folder-portal { padding: 16px; }
.portal-header { display: flex; gap: 8px; margin-bottom: 16px; }
.portal-input {
  flex: 1; padding: 8px 12px; border-radius: 8px;
  border: 1px solid var(--color-border-subtle); background: var(--color-bg-surface);
  color: var(--color-text-primary); font-size: 13px; cursor: pointer;
}
.portal-files { display: flex; flex-direction: column; gap: 4px; }
.portal-file {
  display: flex; align-items: center; gap: 8px; padding: 8px 12px;
  border-radius: 8px; background: var(--color-bg-surface); font-size: 13px;
}
.file-name { flex: 1; color: var(--color-text-primary); }
.file-size { color: var(--color-text-tertiary); font-size: 12px; }
.portal-loading, .portal-error, .portal-empty {
  text-align: center; padding: 24px; color: var(--color-text-tertiary); font-size: 13px;
}
.portal-error { color: #e53e3e; }
.org-btn {
  padding: 8px 16px; border-radius: 10px; border: 1px solid var(--color-border-subtle);
  background: var(--color-bg-elevated); color: var(--color-text-secondary);
  font-family: var(--font-ui); font-size: 13px; cursor: pointer;
}
.org-btn.primary { background: var(--gradient-primary); color: white; border: none; }
.org-btn:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
