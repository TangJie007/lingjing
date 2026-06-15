<script setup lang="ts">
// 快捷键配置 (SET-010)
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

interface ShortcutEntry {
  id: string
  label: string
  defaultKeys: string
  keys: string
  recording: boolean
}

const shortcuts = ref<ShortcutEntry[]>([
  { id: 'switch', label: t('settings.shortcutSwitch'), defaultKeys: 'Ctrl+Shift+W', keys: 'Ctrl+Shift+W', recording: false },
  { id: 'pause', label: t('settings.shortcutPause'), defaultKeys: 'Ctrl+Shift+P', keys: 'Ctrl+Shift+P', recording: false },
  { id: 'hideIcons', label: t('settings.shortcutHideIcons'), defaultKeys: 'Ctrl+Shift+H', keys: 'Ctrl+Shift+H', recording: false },
])

function startRecording(entry: ShortcutEntry) {
  entry.recording = true
  entry.keys = '...'
}

function onKeydown(e: KeyboardEvent, entry: ShortcutEntry) {
  if (!entry.recording) return
  e.preventDefault()
  const parts: string[] = []
  if (e.ctrlKey) parts.push('Ctrl')
  if (e.shiftKey) parts.push('Shift')
  if (e.altKey) parts.push('Alt')
  if (e.metaKey) parts.push('Win')
  const key = e.key.length === 1 ? e.key.toUpperCase() : e.key
  if (!['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) {
    parts.push(key)
  }
  if (parts.length >= 2) {
    entry.keys = parts.join('+')
    entry.recording = false
  }
}

function resetShortcut(entry: ShortcutEntry) {
  entry.keys = entry.defaultKeys
  entry.recording = false
}
</script>

<template>
  <section class="settings-section">
    <h2 class="settings-section-title">{{ t('settings.shortcuts') }}</h2>
    <div v-for="entry in shortcuts" :key="entry.id" class="setting-row">
      <div class="setting-info">
        <span class="setting-label">{{ entry.label }}</span>
        <span class="setting-desc">{{ t('settings.shortcutDefault', { keys: entry.defaultKeys }) }}</span>
      </div>
      <div class="shortcut-input-wrap">
        <input
          class="shortcut-input"
          :value="entry.keys"
          readonly
          @focus="startRecording(entry)"
          @keydown="onKeydown($event, entry)"
          @blur="entry.recording = false"
        />
        <button
          v-if="entry.keys !== entry.defaultKeys"
          class="shortcut-reset"
          @click="resetShortcut(entry)"
        >
          ↺
        </button>
      </div>
    </div>
  </section>
</template>

<style scoped>
.shortcut-input-wrap {
  display: flex;
  align-items: center;
  gap: 8px;
}
.shortcut-input {
  width: 160px;
  padding: 6px 12px;
  border-radius: 8px;
  border: 1px solid var(--color-border-subtle);
  background: var(--color-bg-elevated);
  color: var(--color-text-primary);
  font-family: var(--font-mono);
  font-size: 13px;
  text-align: center;
  cursor: pointer;
  outline: none;
}
.shortcut-input:focus {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px rgba(0, 131, 54, 0.15);
}
.shortcut-reset {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  border: 1px solid var(--color-border-subtle);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
  font-size: 14px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}
.shortcut-reset:hover {
  background: var(--color-bg-surface);
}
</style>
