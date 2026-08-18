// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

/// 示例命令：Rust 向后端问候
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// 无边框窗口：启动窗口拖拽（供前端 mousedown 时调用）
#[tauri::command]
fn start_drag(window: tauri::Window) {
    let _ = window.start_dragging();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![greet, start_drag])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
