//! 开机自启模块 (SET-001)
//! 使用 tauri-plugin-autostart，底层封装 auto_launch 库
//! 支持 Windows 注册表 / macOS LaunchAgent / Linux autostart

use tauri::Runtime;
use tauri_plugin_autostart::Builder;

/// 创建 autostart 插件实例
pub fn init<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    Builder::new().build()
}
