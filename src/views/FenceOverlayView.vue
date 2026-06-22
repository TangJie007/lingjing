<script setup lang="ts">
// 栅格叠加层 — 渲染在桌面图标上方的透明分类框
// 窗口设置了 Win32 WS_EX_TRANSPARENT，鼠标事件完全穿透到桌面图标
// 窗口初始隐藏，本组件渲染完成后调用 show_fence_overlay 才显示，避免白色遮挡
import { ref, onMounted, nextTick } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'

interface FenceSpec {
  label: string
  color: string
  x: number
  y: number
  w: number
  h: number
}

const fences = ref<FenceSpec[]>([])

function hexToRgba(hex: string, alpha: number) {
  const m = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex)
  if (!m) return `rgba(0,0,0,${alpha})`
  return `rgba(${parseInt(m[1], 16)},${parseInt(m[2], 16)},${parseInt(m[3], 16)},${alpha})`
}

async function showSelf() {
  await nextTick()
  try { await invoke('show_fence_overlay') } catch { /* ignore */ }
}

async function hideSelf() {
  try { await invoke('hide_fence_overlay') } catch { /* ignore */ }
}

onMounted(async () => {
  // Listen for fence data pushed by the organize command
  await listen<FenceSpec[]>('fence-update', async (event) => {
    const data = event.payload ?? []
    fences.value = data
    if (data.length > 0) {
      await showSelf()
    } else {
      await hideSelf()
    }
  })

  // Also check if there's already fence data (e.g. app restarted after previous organize)
  try {
    const data = await invoke<FenceSpec[]>('get_fence_data')
    if (data?.length) {
      fences.value = data
      await showSelf()
    }
  } catch { /* ignore */ }
})
</script>

<template>
  <div class="fence-root">
    <div
      v-for="(f, i) in fences"
      :key="i"
      class="fence"
      :style="{
        left:        f.x + 'px',
        top:         f.y + 'px',
        width:       f.w + 'px',
        height:      f.h + 'px',
        borderColor: f.color,
        background:  hexToRgba(f.color, 0.08),
      }"
    >
      <div
        class="fence-title"
        :style="{ background: hexToRgba(f.color, 0.6) }"
      >
        {{ f.label }}
      </div>
    </div>
  </div>
</template>

<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
html, body {
  width: 100%;
  height: 100%;
  overflow: hidden;
  background: transparent !important;
}
#app { width: 100%; height: 100%; background: transparent; }
</style>

<style scoped>
.fence-root {
  position: fixed;
  inset: 0;
  pointer-events: none;
  background: transparent;
}
.fence {
  position: absolute;
  border: 1.5px solid;
  border-radius: 10px;
  overflow: hidden;
}
.fence-title {
  height: 28px;
  display: flex;
  align-items: center;
  padding: 0 10px;
  font-size: 12px;
  font-weight: 600;
  color: #fff;
  font-family: 'Inter', 'Noto Sans SC', system-ui, sans-serif;
  letter-spacing: 0.04em;
  text-shadow: 0 1px 2px rgba(0,0,0,0.5);
}
</style>
