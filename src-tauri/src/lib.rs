//! 灵境 LingScape — Tauri 应用入口
//! 负责窗口管理、系统托盘、开机自启、壁纸引擎、API Key 管理

mod api;
mod autostart;
mod crypto;
mod desktop_core;
mod tray;
mod video_player;
mod wallpaper_engine;

use tauri::{Manager, RunEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(autostart::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(wallpaper_engine::CurrentWallpaperState::default())
        .manage(video_player::VideoPlayerState::default())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;

                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }

            // 初始化系统托盘 (ST-001)
            let _tray = tray::create(app.handle())?;

            // 预创建 WebView 桌面播放器（保留供未来 HTML 壁纸；视频走 MPV）
            let handle = app.handle().clone();
            let init_handle = handle.clone();
            let _ = handle.run_on_main_thread(move || {
                wallpaper_engine::init_desktop_player(&init_handle);
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            wallpaper_engine::list_wallpapers,
            wallpaper_engine::import_wallpaper,
            wallpaper_engine::delete_wallpaper,
            wallpaper_engine::export_wallpaper,
            wallpaper_engine::set_wallpaper,
            wallpaper_engine::check_desktop_layer_conflicts,
            wallpaper_engine::get_current_wallpaper,
            wallpaper_engine::get_library_size,
            wallpaper_engine::clear_library,
            wallpaper_engine::is_fullscreen_app_running,
            wallpaper_engine::attach_desktop_player,
            crypto::crypto_encrypt,
            crypto::crypto_decrypt,
            crypto::crypto_list_platforms,
            api::test_connection,
            api::open_url,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if matches!(event, RunEvent::ExitRequested { .. } | RunEvent::Exit) {
                let vp = app_handle.state::<video_player::VideoPlayerState>();
                video_player::stop_video_wallpaper(&vp);
            }
        });
}
