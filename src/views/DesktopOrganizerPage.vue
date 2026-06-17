<script setup lang="ts">
// 桌面整理页面 (DO-001)
// 分区创建、命名、调色、卷起/展开、图标拖入
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useDesktopOrganizerStore } from '@/stores/desktop-organizer'
import type { PartitionInfo } from '@/stores/desktop-organizer'

const { t } = useI18n()
const store = useDesktopOrganizerStore()

const creating = ref(false)
const createStart = ref({ x: 0, y: 0 })
const createRect = ref({ x: 0, y: 0, w: 0, h: 0 })
const newName = ref('')
const showNameDialog = ref(false)
const pendingPartition = ref<{ x: number; y: number; w: number; h: number } | null>(null)

const templatePartitions = [
  { name: '工作区', color: '#008336' },
  { name: '娱乐', color: '#aa6300' },
  { name: '工具', color: '#068d9a' },
  { name: '临时', color: '#666666' },
]

onMounted(() => {
  store.loadLayout()
  store.enumerateIcons()
})

function startCreate(e: MouseEvent) {
  if (!store.editing) return
  creating.value = true
  createStart.value = { x: e.clientX, y: e.clientY }
  createRect.value = { x: e.clientX, y: e.clientY, w: 0, h: 0 }
}

function onDrag(e: MouseEvent) {
  if (!creating.value) return
  createRect.value = {
    x: Math.min(createStart.value.x, e.clientX),
    y: Math.min(createStart.value.y, e.clientY),
    w: Math.abs(e.clientX - createStart.value.x),
    h: Math.abs(e.clientY - createStart.value.y),
  }
}

function endCreate() {
  if (!creating.value) return
  creating.value = false
  if (createRect.value.w < 50 || createRect.value.h < 50) return
  pendingPartition.value = { ...createRect.value }
  newName.value = ''
  showNameDialog.value = true
}

async function confirmCreate() {
  if (!pendingPartition.value) return
  const name = newName.value || '新分区'
  await store.createPartition(
    name,
    pendingPartition.value.x,
    pendingPartition.value.y,
    pendingPartition.value.w,
    pendingPartition.value.h,
  )
  showNameDialog.value = false
  pendingPartition.value = null
}

async function createFromTemplate(tpl: { name: string; color: string }) {
  await store.createPartition(tpl.name, 100, 100 + store.partitions.length * 220, 300, 200, tpl.color)
}

function getPartitionStyle(p: PartitionInfo) {
  return {
    left: `${p.x}px`,
    top: `${p.y}px`,
    width: `${p.w}px`,
    height: p.collapsed ? '36px' : `${p.h}px`,
    background: `${p.color}${Math.round(p.opacity * 255).toString(16).padStart(2, '0')}`,
    borderColor: p.color,
  }
}
</script>

<template>
  <div class="organizer-page">
    <!-- 工具栏 -->
    <div class="organizer-toolbar">
      <button
        class="org-btn"
        :class="{ active: store.editing }"
        @click="store.editing = !store.editing"
      >
        {{ store.editing ? t('desktop.doneEditing') : t('desktop.editMode') }}
      </button>
      <button class="org-btn" @click="store.toggleIcons">
        {{ store.iconsHidden ? t('desktop.showIcons') : t('desktop.hideIcons') }}
      </button>
      <span class="org-stat">{{ t('desktop.iconCount', { count: store.totalIcons }) }}</span>
    </div>

    <!-- 模板分区 -->
    <div v-if="store.editing" class="organizer-templates">
      <span class="template-label">{{ t('desktop.quickCreate') }}:</span>
      <button
        v-for="tpl in templatePartitions"
        :key="tpl.name"
        class="template-btn"
        @click="createFromTemplate(tpl)"
      >
        {{ tpl.name }}
      </button>
    </div>

    <!-- 分区画布 -->
    <div
      class="organizer-canvas"
      :class="{ editing: store.editing }"
      @mousedown="startCreate"
      @mousemove="onDrag"
      @mouseup="endCreate"
    >
      <!-- 创建中的拖拽矩形 -->
      <div
        v-if="creating"
        class="create-rect"
        :style="{
          left: `${createRect.x}px`,
          top: `${createRect.y}px`,
          width: `${createRect.w}px`,
          height: `${createRect.h}px`,
        }"
      />

      <!-- 分区 -->
      <div
        v-for="p in store.partitions"
        :key="p.id"
        class="partition"
        :style="getPartitionStyle(p)"
      >
        <div class="partition-header" @dblclick="store.toggleCollapse(p.id)">
          <span class="partition-name">{{ p.name }}</span>
          <span v-if="p.collapsed" class="partition-count">({{ p.iconCount }})</span>
          <button
            v-if="store.editing"
            class="partition-delete"
            @click.stop="store.deletePartition(p.id)"
          >
            ×
          </button>
        </div>
        <div v-if="!p.collapsed" class="partition-body">
          <div class="partition-placeholder">
            {{ t('desktop.dragIconsHere') }}
          </div>
        </div>
      </div>

      <!-- 空状态 -->
      <div v-if="store.partitions.length === 0 && !store.editing" class="organizer-empty">
        <p>{{ t('desktop.noPartitions') }}</p>
        <button class="org-btn primary" @click="store.editing = true">
          {{ t('desktop.startOrganizing') }}
        </button>
      </div>
    </div>

    <!-- 命名对话框 -->
    <div v-if="showNameDialog" class="dialog-overlay" @click.self="showNameDialog = false">
      <div class="dialog-box">
        <h3>{{ t('desktop.namePartition') }}</h3>
        <input
          v-model="newName"
          class="dialog-input"
          :placeholder="t('desktop.namePlaceholder')"
          @keyup.enter="confirmCreate"
          autofocus
        />
        <div class="dialog-actions">
          <button class="org-btn" @click="showNameDialog = false">{{ t('common.cancel') }}</button>
          <button class="org-btn primary" @click="confirmCreate">{{ t('common.confirm') }}</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.organizer-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.organizer-toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 24px;
  border-bottom: 1px solid var(--color-border-subtle);
  flex-shrink: 0;
}

.org-btn {
  padding: 8px 16px;
  border-radius: 10px;
  border: 1px solid var(--color-border-subtle);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
  font-family: var(--font-ui);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s;
}
.org-btn:hover { background: var(--color-bg-surface); }
.org-btn.active {
  background: var(--color-primary-surface);
  border-color: var(--color-primary);
  color: var(--color-primary);
}
.org-btn.primary {
  background: var(--gradient-primary);
  color: white;
  border: none;
}

.org-stat {
  font-size: 13px;
  color: var(--color-text-tertiary);
  margin-left: auto;
}

.organizer-templates {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 24px;
  border-bottom: 1px solid var(--color-border-subtle);
}
.template-label {
  font-size: 13px;
  color: var(--color-text-tertiary);
}
.template-btn {
  padding: 4px 12px;
  border-radius: 6px;
  border: 1px solid var(--color-border-subtle);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
  font-size: 12px;
  cursor: pointer;
}
.template-btn:hover { background: var(--color-bg-surface); }

.organizer-canvas {
  flex: 1;
  position: relative;
  overflow: auto;
  background: rgba(0, 0, 0, 0.02);
}
.organizer-canvas.editing {
  cursor: crosshair;
}

.create-rect {
  position: absolute;
  border: 2px dashed var(--color-primary);
  background: rgba(0, 131, 54, 0.08);
  border-radius: 8px;
  pointer-events: none;
  z-index: 10;
}

.partition {
  position: absolute;
  border-radius: 12px;
  border: 1px solid;
  overflow: hidden;
  transition: height 0.3s ease;
  backdrop-filter: blur(4px);
}
.partition-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  background: rgba(0, 0, 0, 0.15);
  cursor: pointer;
  user-select: none;
}
.partition-name {
  font-size: 13px;
  font-weight: 600;
  color: white;
  font-family: var(--font-ui);
}
.partition-count {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.7);
}
.partition-delete {
  margin-left: auto;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: none;
  background: rgba(255, 255, 255, 0.2);
  color: white;
  font-size: 16px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}
.partition-delete:hover { background: rgba(255, 0, 0, 0.4); }

.partition-body {
  padding: 12px;
  min-height: 60px;
}
.partition-placeholder {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
  text-align: center;
  padding: 20px;
}

.organizer-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 16px;
  color: var(--color-text-tertiary);
}

.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}
.dialog-box {
  background: var(--color-bg-elevated);
  border-radius: 16px;
  padding: 24px;
  min-width: 320px;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.2);
}
.dialog-box h3 {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: 16px;
}
.dialog-input {
  width: 100%;
  padding: 10px 14px;
  border-radius: 10px;
  border: 1px solid var(--color-border-subtle);
  background: var(--color-bg-surface);
  color: var(--color-text-primary);
  font-size: 14px;
  outline: none;
  margin-bottom: 16px;
}
.dialog-input:focus { border-color: var(--color-primary); }
.dialog-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}
</style>
