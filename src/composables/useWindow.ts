// 窗口控制 composable
// 关闭时最小化到托盘（而非退出）
import { onMounted, onUnmounted } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'

export function useWindow() {
  let unlistenClose: (() => void) | null = null

  async function setup() {
    try {
      const win = getCurrentWindow()
      // 拦截关闭事件，改为隐藏到托盘
      unlistenClose = await win.onCloseRequested(async (event) => {
        // 阻止默认关闭行为
        event.preventDefault()
        await win.hide()
      })
    } catch {
      // Tauri API 不可用时静默失败
    }
  }

  function cleanup() {
    if (unlistenClose) {
      unlistenClose()
      unlistenClose = null
    }
  }

  onMounted(setup)
  onUnmounted(cleanup)

  return { cleanup }
}
