mod desktop;
mod desktop_organize;
mod favorites;
mod library;
mod paths;
mod power;
pub mod settings;
#[cfg(windows)]
mod shell_menu;
mod system;
mod util;
mod wallpaper;

pub use desktop::maybe_run_icons_restore_guard as maybe_run_desktop_icons_guard;
pub use desktop_organize::maybe_run_shell_menu_host;
pub use util::init_logging;

use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State,
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_dialog::DialogExt;
use wallpaper::{commands::RuntimeProfile, EngineHandle, EngineState};

#[tauri::command]
fn list_library(app: AppHandle) -> Result<Vec<library::LibraryItem>, String> {
    library::list_items(&app)
}

#[tauri::command]
fn import_media(app: AppHandle, paths: Vec<String>) -> Result<library::ImportResult, String> {
    library::import_paths(&app, paths)
}

#[tauri::command]
fn remove_library_item(
    app: AppHandle,
    engine: State<'_, EngineHandle>,
    id: String,
) -> Result<EngineState, String> {
    let was_current = engine
        .state
        .lock()
        .map(|s| s.media_id.as_deref() == Some(id.as_str()))
        .map_err(|_| "引擎状态锁失败".to_string())?;

    library::remove_item(&app, &id)?;

    if was_current {
        wallpaper::commands::clear_engine_wallpaper(&app, &engine)
    } else {
        engine
            .state
            .lock()
            .map(|s| s.clone())
            .map_err(|_| "引擎状态锁失败".into())
    }
}

#[tauri::command]
fn load_favorites(app: AppHandle) -> Result<serde_json::Value, String> {
    let (file, is_new) = favorites::load(&app)?;
    Ok(serde_json::json!({
        "ids": file.ids,
        "isNew": is_new,
    }))
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct FavPayload {
    id: String,
    favorite: bool,
}

#[tauri::command]
fn set_favorite(app: AppHandle, payload: FavPayload) -> Result<favorites::FavoritesFile, String> {
    favorites::set_favorite(&app, payload.id, payload.favorite)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppPaths {
    app_data_dir: String,
    library_dir: String,
}

#[tauri::command]
fn get_app_paths(app: AppHandle) -> Result<AppPaths, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法解析应用数据目录: {e}"))?;
    let library_dir = settings::library_root(&app)?;
    Ok(AppPaths {
        app_data_dir: app_data.to_string_lossy().to_string(),
        library_dir: library_dir.to_string_lossy().to_string(),
    })
}

#[tauri::command]
fn load_settings(app: AppHandle) -> Result<settings::AppSettings, String> {
    settings::load_settings(&app)
}

#[tauri::command]
fn save_settings(app: AppHandle, payload: settings::AppSettings) -> Result<(), String> {
    let prev = settings::load_settings(&app).unwrap_or_default();
    settings::persist_and_notify(&app, &payload)?;
    if prev.autostart != payload.autostart {
        let mgr = app.autolaunch();
        if payload.autostart {
            let _ = mgr.enable();
        } else {
            let _ = mgr.disable();
        }
    }
    if prev.hide_icons_on_double_click != payload.hide_icons_on_double_click {
        desktop::set_double_click_enabled(payload.hide_icons_on_double_click);
    }
    Ok(())
}

#[tauri::command]
fn has_version_record(app: AppHandle) -> Result<bool, String> {
    Ok(settings::has_version_record(&app))
}

#[tauri::command]
fn complete_first_run(app: AppHandle, autostart: bool) -> Result<(), String> {
    let version = app.package_info().version.to_string();
    settings::write_version_record(&app, &version)?;
    let mut next = settings::load_settings(&app)?;
    next.autostart = autostart;
    settings::persist_and_notify(&app, &next)?;
    let mgr = app.autolaunch();
    if autostart {
        let _ = mgr.enable();
    } else {
        let _ = mgr.disable();
    }
    Ok(())
}

#[tauri::command]
fn set_library_dir(app: AppHandle, new_dir: String) -> Result<paths::MigrationPlan, String> {
    paths::set_library_dir(&app, new_dir)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
struct MigratePayload {
    keep_originals: bool,
}

impl Default for MigratePayload {
    fn default() -> Self {
        Self {
            keep_originals: true,
        }
    }
}

#[tauri::command]
fn migrate_library(
    app: AppHandle,
    payload: MigratePayload,
) -> Result<paths::MigrationReport, String> {
    paths::migrate_library(&app, payload.keep_originals)
}

#[tauri::command]
fn get_last_wallpaper(app: AppHandle) -> Result<Option<settings::LastWallpaper>, String> {
    settings::load_last_wallpaper(&app)
}

fn focus_main_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let show_item = MenuItem::with_id(app, "show", "显示灵镜", true, None::<&str>)?;
    let pause_item = MenuItem::with_id(app, "pause", "暂停壁纸", true, None::<&str>)?;
    let play_item = MenuItem::with_id(app, "play", "恢复壁纸", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_item, &pause_item, &play_item, &quit_item])?;
    let icon: Image = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| tauri::Error::AssetNotFound("default window icon".into()))?;
    let mut builder = TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false);
    builder = builder.on_menu_event(|app, event| match event.id.as_ref() {
        "show" => focus_main_window(app),
        "pause" => {
            let app_clone = app.clone();
            tauri::async_runtime::spawn(async move {
                if let Some(state) = app_clone.try_state::<EngineHandle>() {
                    if let Ok(mut s) = state.state.lock() {
                        s.playing = false;
                        s.user_paused = true;
                        let snap = s.clone();
                        drop(s);
                        let _ = wallpaper::push_command(&app_clone, "pause", &snap);
                        let _ = app_clone.emit("engine-state", &snap);
                    }
                }
            });
        }
        "play" => {
            let app_clone = app.clone();
            tauri::async_runtime::spawn(async move {
                if let Some(state) = app_clone.try_state::<EngineHandle>() {
                    if let Ok(mut s) = state.state.lock() {
                        s.playing = true;
                        s.user_paused = false;
                        let snap = s.clone();
                        drop(s);
                        let _ = wallpaper::push_command(&app_clone, "play", &snap);
                        let _ = app_clone.emit("engine-state", &snap);
                    }
                }
            });
        }
        "quit" => {
            desktop_organize::cleanup(app);
            wallpaper::cleanup(app);
            app.exit(0);
        }
        _ => {}
    });
    builder = builder.on_tray_icon_event(|tray, event| {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = event
        {
            let app = tray.app_handle();
            if let Some(win) = app.get_webview_window("main") {
                // After sleep, is_visible() is often stale (true while window is
                // not actually interactable). Prefer "bring to front" over toggle.
                let visible = win.is_visible().unwrap_or(false);
                let minimized = win.is_minimized().unwrap_or(false);
                let focused = win.is_focused().unwrap_or(false);
                if visible && !minimized && focused {
                    let _ = win.hide();
                } else {
                    focus_main_window(app);
                }
            }
        }
    });
    builder.build(app)?;
    Ok(())
}

fn resolve_export_source(app: &AppHandle, uri: &str) -> Option<PathBuf> {
    let trimmed = uri.trim();
    if trimmed.is_empty() {
        return None;
    }
    let direct = Path::new(trimmed);
    if direct.is_file() {
        return Some(direct.to_path_buf());
    }
    if trimmed.starts_with('/') {
        if let Ok(res) = app.path().resource_dir() {
            let candidate = res.join(trimmed.trim_start_matches('/'));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

#[tauri::command]
fn export_wallpaper(
    app: AppHandle,
    uri: String,
    file_name: String,
) -> Result<Option<String>, String> {
    let src =
        resolve_export_source(&app, &uri).ok_or_else(|| "找不到可导出的源文件".to_string())?;
    let dest = app
        .dialog()
        .file()
        .set_file_name(&file_name)
        .blocking_save_file();
    match dest {
        Some(path) => {
            let dest_str = path.to_string();
            std::fs::copy(&src, &dest_str).map_err(|e| format!("复制失败: {e}"))?;
            Ok(Some(dest_str))
        }
        None => Ok(None),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Closing main only hides to tray; a second launch must not re-attach
        // wallpaper / desktop-organize (that deadlocks / Not Responding).
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            focus_main_window(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_http::init())
        .manage(EngineHandle::default())
        .manage(power::PowerWatcher::default())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .setup(|app| {
            desktop::recover_icons_after_crash();
            let low_power = system::detect_low_power_mode();
            if low_power {
                tracing::info!("[system] low power mode enabled (< 6GB RAM)");
            }
            app.manage(RuntimeProfile { low_power });

            if let Some(main) = app.get_webview_window("main") {
                if let Ok(hwnd) = main.hwnd() {
                    desktop::apply_frameless_dwm(hwnd.0 as isize);
                }
            }

            let handle = app.handle().clone();
            let handle_for_attach = handle.clone();
            let handle_for_settings = handle.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(800));
                match wallpaper::attach_existing(&handle_for_attach) {
                    Ok(()) => tracing::info!("[wallpaper] startup attach ok"),
                    Err(e) => tracing::info!("[wallpaper] startup attach skipped: {e}"),
                }
            });

            if let Ok(s) = settings::load_settings(&handle_for_settings) {
                let mgr = handle.autolaunch();
                if settings::has_version_record(&handle_for_settings) {
                    if s.autostart {
                        let _ = mgr.enable();
                    } else {
                        let _ = mgr.disable();
                    }
                } else {
                    let _ = mgr.disable();
                }
                desktop::set_double_click_enabled(s.hide_icons_on_double_click);
                if s.desktop_organize_enabled {
                    let app_clone = handle.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_millis(1200));
                        if let Err(e) = desktop_organize::set_enabled(&app_clone, true) {
                            tracing::info!("[desktop-organize] startup restore failed: {e}");
                        }
                    });
                }
            }

            if let Err(e) = build_tray(&handle) {
                tracing::info!("[tray] build failed: {e}");
            }

            let start_minimized = std::env::args().any(|a| a == "--minimized");
            if start_minimized {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.hide();
                }
            }

            let app_for_power = handle.clone();
            power::start_watcher(app_for_power);

            let app_for_restore = handle.clone();
            std::thread::spawn(move || {
                wallpaper::commands::restore_last_wallpaper(&app_for_restore)
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            start_drag,
            minimize_main,
            hide_main,
            show_main_settings,
            wallpaper::commands::set_wallpaper,
            wallpaper::commands::engine_play,
            wallpaper::commands::engine_pause,
            wallpaper::commands::engine_set_volume,
            wallpaper::commands::engine_get_state,
            wallpaper::commands::engine_report_progress,
            list_library,
            import_media,
            remove_library_item,
            load_favorites,
            set_favorite,
            get_app_paths,
            load_settings,
            save_settings,
            has_version_record,
            complete_first_run,
            set_library_dir,
            migrate_library,
            get_last_wallpaper,
            export_wallpaper,
            desktop_organize::item_commands::open_desktop_item,
            desktop_organize::menu_commands::list_desktop_shell_context_menu,
            desktop_organize::menu_commands::list_desktop_shell_context_submenu,
            desktop_organize::menu_commands::invoke_desktop_shell_context_command,
            desktop_organize::menu_commands::show_desktop_native_context_menu,
            desktop_organize::item_commands::show_desktop_item_in_folder,
            desktop_organize::item_commands::open_desktop_item_with,
            desktop_organize::item_commands::open_desktop_item_properties,
            desktop_organize::item_commands::rename_desktop_item,
            desktop_organize::item_commands::move_desktop_item_into_folder,
            desktop_organize::item_commands::delete_desktop_item,
            desktop_organize::layout::load_fence_layout,
            desktop_organize::layout::save_fence_layout,
            desktop_organize::drag::is_desktop_drag_over_foreign,
            desktop_organize::drag::start_desktop_file_drag,
            desktop_organize::drag::try_start_desktop_file_drag_if_foreign,
            desktop_organize::menu_commands::drop_files_to_desktop,
            desktop_organize::item_commands::set_desktop_organize,
            desktop_organize::item_commands::list_desktop_items,
            refresh_desktop_organize
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| match event {
            tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit => {
                desktop_organize::cleanup(app_handle);
                wallpaper::cleanup(app_handle);
            }
            _ => {}
        });
}

#[tauri::command]
fn refresh_desktop_organize(app: AppHandle) -> Result<(), String> {
    desktop_organize::refresh(&app)
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust!")
}

#[tauri::command]
fn start_drag(window: tauri::Window) {
    if window.label() != "main" {
        return;
    }
    #[cfg(windows)]
    {
        if let Ok(hwnd) = window.hwnd() {
            let raw = hwnd.0 as isize;
            desktop::apply_frameless_dwm(raw);
            std::thread::spawn(move || {
                desktop::drag_window_by_mouse(raw);
            });
            return;
        }
    }
    let _ = window.start_dragging();
}

#[tauri::command]
fn minimize_main(app: AppHandle) -> Result<(), String> {
    let win = app
        .get_webview_window("main")
        .ok_or_else(|| "主窗口不存在".to_string())?;
    #[cfg(windows)]
    {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_MINIMIZE};
        let hwnd = win.hwnd().map_err(|e| e.to_string())?;
        unsafe {
            let _ = ShowWindow(HWND(hwnd.0 as *mut _), SW_MINIMIZE);
        }
        return Ok(());
    }
    #[cfg(not(windows))]
    win.minimize().map_err(|e| e.to_string())
}

#[tauri::command]
fn hide_main(app: AppHandle) -> Result<(), String> {
    let win = app
        .get_webview_window("main")
        .ok_or_else(|| "主窗口不存在".to_string())?;
    win.hide().map_err(|e| e.to_string())
}

/// Show main window and ask it to open the Settings page (from Dynamic Island).
#[tauri::command]
fn show_main_settings(app: AppHandle) -> Result<(), String> {
    let win = app
        .get_webview_window("main")
        .ok_or_else(|| "主窗口不存在".to_string())?;
    #[cfg(windows)]
    {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_RESTORE};
        if let Ok(hwnd) = win.hwnd() {
            unsafe {
                let _ = ShowWindow(HWND(hwnd.0 as *mut _), SW_RESTORE);
            }
        }
    }
    let _ = win.unminimize();
    win.show().map_err(|e| e.to_string())?;
    let _ = win.set_focus();
    let _ = app.emit("navigate-settings", ());
    Ok(())
}
