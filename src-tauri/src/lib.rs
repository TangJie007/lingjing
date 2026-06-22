//! 灵境 LingScape — Tauri 应用入口
//! 负责窗口管理、系统托盘、开机自启、壁纸引擎、API Key 管理

mod api;
mod auto_rotate;
mod auto_rules;
mod autostart;
mod crypto;
mod desktop_core;
#[cfg(target_os = "windows")]
mod desktop_organizer;
mod folder_portal;
mod playback_adjust;
mod thumbnail;
mod tray;
mod update_checker;
mod video_player;
mod wallpaper_backup;
mod wallpaper_engine;

use tauri::{Manager, RunEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(autostart::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(wallpaper_engine::CurrentWallpaperState::default())
        .manage(video_player::VideoPlayerState::default())
        .manage(auto_rotate::RotateState::default())
        .manage(auto_rules::RuleState::default());

    #[cfg(target_os = "windows")]
    let builder = builder
        .manage(desktop_organizer::OrganizerState::default())
        .manage(desktop_organizer::FenceOverlayState::default());

    builder
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

            // 备份原始壁纸 (SET-006)
            if let Ok(data_dir) = app.handle().path().app_data_dir() {
                let _ = crate::wallpaper_backup::backup_original_wallpaper(
                    data_dir.to_string_lossy().to_string(),
                );
                // 检查崩溃恢复 (SET-006)
                if let Ok(recovery) = crate::wallpaper_backup::check_crash_recovery(
                    data_dir.to_string_lossy().to_string(),
                ) {
                    if recovery["needsRecovery"].as_bool().unwrap_or(false) {
                        log::warn!("检测到上次异常退出，存在未恢复的壁纸快照");
                    }
                }
            }

            // 启动定时轮换 (SET-007)
            auto_rotate::start_rotation(app.handle().clone());

            // 预创建 WebView 桌面播放器（保留供未来 HTML 壁纸；视频走 MPV）
            let handle = app.handle().clone();
            let init_handle = handle.clone();
            let _ = handle.run_on_main_thread(move || {
                wallpaper_engine::init_desktop_player(&init_handle);
                #[cfg(target_os = "windows")]
                {
                    if let Err(e) = crate::desktop_core::setup_desktop_layer() {
                        log::warn!("预初始化桌面层失败: {}", e);
                    }
                    crate::desktop_organizer::init_fence_overlay(&init_handle);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            wallpaper_engine::list_wallpapers,
            wallpaper_engine::import_wallpaper,
            wallpaper_engine::delete_wallpaper,
            wallpaper_engine::export_wallpaper,
            wallpaper_engine::set_wallpaper,
            wallpaper_engine::get_current_wallpaper,
            wallpaper_engine::get_library_size,
            wallpaper_engine::clear_library,
            wallpaper_engine::is_fullscreen_app_running,
            wallpaper_engine::attach_desktop_player,
            thumbnail::generate_thumbnail,
            wallpaper_backup::backup_original_wallpaper,
            wallpaper_backup::restore_original_wallpaper,
            wallpaper_backup::check_crash_recovery,
            auto_rotate::set_rotate_config,
            auto_rotate::get_rotate_config,
            update_checker::check_update,
            folder_portal::open_folder_portal,
            folder_portal::refresh_folder_portal,
            auto_rules::get_auto_rules,
            auto_rules::update_auto_rule,
            auto_rules::apply_auto_rules,
            playback_adjust::set_playback_params,
            #[cfg(target_os = "windows")]
            desktop_organizer::enumerate_desktop_icons,
            #[cfg(target_os = "windows")]
            desktop_organizer::create_partition,
            #[cfg(target_os = "windows")]
            desktop_organizer::delete_partition,
            #[cfg(target_os = "windows")]
            desktop_organizer::update_partition,
            #[cfg(target_os = "windows")]
            desktop_organizer::move_icon_to_partition,
            #[cfg(target_os = "windows")]
            desktop_organizer::get_partition_layout,
            #[cfg(target_os = "windows")]
            desktop_organizer::save_partition_layout,
            #[cfg(target_os = "windows")]
            desktop_organizer::load_partition_layout,
            #[cfg(target_os = "windows")]
            desktop_organizer::organize_desktop_one_click,
            #[cfg(target_os = "windows")]
            desktop_organizer::get_fence_data,
            #[cfg(target_os = "windows")]
            desktop_organizer::show_fence_overlay,
            #[cfg(target_os = "windows")]
            desktop_organizer::hide_fence_overlay,
            #[cfg(target_os = "windows")]
            desktop_organizer::clear_fence_overlay,
            #[cfg(target_os = "windows")]
            desktop_organizer::hide_desktop_icons,
            #[cfg(target_os = "windows")]
            desktop_organizer::show_desktop_icons,
            crypto::crypto_encrypt,
            crypto::crypto_decrypt,
            crypto::crypto_list_platforms,
            api::test_connection,
            api::open_url,
            api::doubao::ai_analyze,
            api::seedream::ai_generate,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if matches!(event, RunEvent::ExitRequested { .. } | RunEvent::Exit) {
                let vp = app_handle.state::<video_player::VideoPlayerState>();
                video_player::stop_video_wallpaper(&vp);
                auto_rotate::stop_rotation(&app_handle.state::<auto_rotate::RotateState>());
                // Restore original wallpaper on exit
                if let Ok(data_dir) = app_handle.path().app_data_dir() {
                    let _ = wallpaper_backup::restore_original_wallpaper(
                        data_dir.to_string_lossy().to_string(),
                    );
                }
            }
        });
}
