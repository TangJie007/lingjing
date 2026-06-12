// 系统托盘事件监听 (ST-001)
// 监听 Rust 侧发出的 tray-action 事件并分发处理
import { onMounted, onUnmounted } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export type TrayAction = 'pause' | 'next'

type TrayActionCallback = (action: TrayAction) => void

export function useTray(onAction?: TrayActionCallback) {
  let unlisten: UnlistenFn | null = null

  async function setup() {
    try {
      unlisten = await listen<string>('tray-action', (event) => {
        const action = event.payload as TrayAction
        if (onAction) {
          onAction(action)
        }
      })
    } catch {
      // Tauri API 不可用时静默失败（如浏览器开发环境）
    }
  }

  function cleanup() {
    if (unlisten) {
      unlisten()
      unlisten = null
    }
  }

  onMounted(setup)
  onUnmounted(cleanup)

  return { cleanup }
}
