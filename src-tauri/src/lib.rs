mod desktop;
mod desktop_organize;
#[cfg(windows)]
mod shell_menu;
mod favorites;
mod library;
mod paths;
mod power;
pub mod settings;
mod system;
mod util;
mod wallpaper;

pub use desktop_organize::maybe_run_shell_menu_host;
pub use util::init_logging;

use std::path::{Path, PathBuf};
use serde::Serialize;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State,
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_dialog::DialogExt;
use wallpaper::{EngineHandle, EngineState, SetWallpaperPayload};

struct RuntimeProfile {
    low_power: bool,
}

fn runtime_low_power(app: &AppHandle) -> bool {
    app.try_state::<RuntimeProfile>()
        .map(|p| p.low_power)
        .unwrap_or(false)
}

fn apply_engine_runtime(state: &mut EngineState, app: &AppHandle) {
    state.low_power = runtime_low_power(app);
}

#[tauri::command]
fn set_wallpaper(
    app: AppHandle,
    engine: State<'_, EngineHandle>,
    payload: SetWallpaperPayload,
) -> Result<EngineState, String> {
    tracing::info!(
        "[engine] set_wallpaper id={} mediaType={} uri={}",
        payload.id, payload.media_type, payload.uri
    );
    if payload.uri.trim().is_empty() {
        return Err("该资源暂无可用媒体".into());
    }
    let default_volume = settings::load_settings(&app)
        .map(|s| s.default_volume.clamp(0.0, 1.0))
        .unwrap_or(0.8);
    let mut state = engine
        .state
        .lock()
        .map_err(|_| "引擎状态锁失败".to_string())?;
    state.media_id = Some(payload.id.clone());
    state.title = Some(payload.title.clone());
    state.media_type = Some(payload.media_type.clone());
    state.uri = Some(payload.uri.clone());
    state.playing = true;
    state.volume = default_volume;
    state.muted = false;
    state.user_paused = false;
    state.error = None;
    state.current_time = 0.0;
    state.duration = 0.0;
    apply_engine_runtime(&mut state, &app);
    let snapshot = state.clone();
    drop(state);
    wallpaper::push_command(&app, "set", &snapshot)?;
    wallpaper::push_state(&app, &snapshot);
    let _ = settings::save_last_wallpaper(
        &app,
        &settings::LastWallpaper {
            id: payload.id,
            title: payload.title,
            media_type: payload.media_type,
            uri: payload.uri,
            source: "engine".into(),
        },
    );
    tracing::info!(
        "[engine] set push_command ok media_id={:?}",
        snapshot.media_id
    );
    Ok(snapshot)
}

#[tauri::command]
fn engine_play(app: AppHandle, engine: State<'_, EngineHandle>) -> Result<EngineState, String> {
    let mut state = engine
        .state
        .lock()
        .map_err(|_| "引擎状态锁失败".to_string())?;
    state.playing = true;
    state.user_paused = false;
    let snapshot = state.clone();
    drop(state);
    wallpaper::push_command(&app, "play", &snapshot)?;
    wallpaper::push_state(&app, &snapshot);
    Ok(snapshot)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PausePayload {
    #[serde(default = "default_true")]
    manual: bool,
}

fn default_true() -> bool {
    true
}

#[tauri::command]
fn engine_pause(
    app: AppHandle,
    engine: State<'_, EngineHandle>,
    payload: Option<PausePayload>,
) -> Result<EngineState, String> {
    let manual = payload.map(|p| p.manual).unwrap_or(true);
    let mut state = engine
        .state
        .lock()
        .map_err(|_| "引擎状态锁失败".to_string())?;
    if manual {
        state.user_paused = true;
    }
    state.playing = false;
    let snapshot = state.clone();
    drop(state);
    wallpaper::push_command(&app, "pause", &snapshot)?;
    wallpaper::push_state(&app, &snapshot);
    Ok(snapshot)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct VolumePayload {
    volume: f64,
    muted: bool,
}

#[tauri::command]
fn engine_set_volume(
    app: AppHandle,
    engine: State<'_, EngineHandle>,
    payload: VolumePayload,
) -> Result<EngineState, String> {
    let mut state = engine
        .state
        .lock()
        .map_err(|_| "引擎状态锁失败".to_string())?;
    state.volume = payload.volume.clamp(0.0, 1.0);
    state.muted = payload.muted;
    let snapshot = state.clone();
    drop(state);
    wallpaper::push_command(&app, "volume", &snapshot)?;
    wallpaper::push_state(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
fn engine_get_state(engine: State<'_, EngineHandle>) -> Result<EngineState, String> {
    engine
        .state
        .lock()
        .map(|s| s.clone())
        .map_err(|_| "引擎状态锁失败".into())
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
struct ProgressPayload {
    current_time: f64,
    duration: f64,
    playing: Option<bool>,
    error: Option<String>,
}

impl Default for ProgressPayload {
    fn default() -> Self {
        Self {
            current_time: 0.0,
            duration: 0.0,
            playing: None,
            error: None,
        }
    }
}

fn is_benign_play_error(msg: &str) -> bool {
    let m = msg.to_ascii_lowercase();
    m.contains("aborterror")
        || m.contains("interrupted by a new load")
        || m.contains("interrupted by a call to pause")
}

#[tauri::command]
fn engine_report_progress(
    app: AppHandle,
    engine: State<'_, EngineHandle>,
    payload: ProgressPayload,
) -> Result<(), String> {
    let mut state = engine
        .state
        .lock()
        .map_err(|_| "引擎状态锁失败".to_string())?;
    state.current_time = payload.current_time;
    state.duration = payload.duration;
    if let Some(p) = payload.playing {
        // Don't let a single stalled primary decoder flip global "playing"
        // to false after sleep; only honor explicit pause/play commands.
        if p || state.user_paused {
            state.playing = p;
        }
    }
    if let Some(err) = payload.error.as_ref() {
        if !is_benign_play_error(err) {
            state.error = Some(err.clone());
        }
    } else if payload.duration > 0.0 || payload.playing.unwrap_or(false) {
        state.error = None;
    }
    let snapshot = state.clone();
    drop(state);
    let _ = app.emit("engine-state", &snapshot);
    Ok(())
}

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
        clear_engine_wallpaper(&app, &engine)
    } else {
        engine
            .state
            .lock()
            .map(|s| s.clone())
            .map_err(|_| "引擎状态锁失败".into())
    }
}

fn clear_engine_wallpaper(app: &AppHandle, engine: &EngineHandle) -> Result<EngineState, String> {
    let mut state = engine
        .state
        .lock()
        .map_err(|_| "引擎状态锁失败".to_string())?;
    state.media_id = None;
    state.title = None;
    state.media_type = None;
    state.uri = None;
    state.playing = false;
    state.current_time = 0.0;
    state.duration = 0.0;
    state.error = None;
    state.user_paused = false;
    let snapshot = state.clone();
    drop(state);
    wallpaper::push_command(app, "clear", &snapshot)?;
    wallpaper::push_state(app, &snapshot);
    let _ = settings::clear_last_wallpaper(app);
    Ok(snapshot)
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

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let show_item = MenuItem::with_id(app, "show", "显示灵镜", true, None::<&str>)?;
    let pause_item = MenuItem::with_id(app, "pause", "暂停壁纸", true, None::<&str>)?;
    let play_item = MenuItem::with_id(app, "play", "恢复壁纸", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[&show_item, &pause_item, &play_item, &quit_item],
    )?;
    let icon: Image = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| tauri::Error::AssetNotFound("default window icon".into()))?;
    let mut builder = TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false);
    builder = builder.on_menu_event(|app, event| match event.id.as_ref() {
        "show" => {
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.show();
                let _ = win.unminimize();
                let _ = win.set_focus();
            }
        }
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
                    let _ = win.show();
                    let _ = win.unminimize();
                    let _ = win.set_focus();
                }
            }
        }
    });
    builder.build(app)?;
    Ok(())
}

fn restore_last_wallpaper(app: &AppHandle) {
    let app_for_task = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
        let last = match settings::load_last_wallpaper(&app_for_task) {
            Ok(Some(l)) => l,
            _ => return,
        };
        if !system::media_path_exists(&last.uri) {
            tracing::info!(
                "[engine] skip restore: media missing uri={}",
                last.uri
            );
            let _ = settings::clear_last_wallpaper(&app_for_task);
            return;
        }
        let mut state = EngineState {
            media_id: Some(last.id.clone()),
            title: Some(last.title.clone()),
            media_type: Some(last.media_type.clone()),
            uri: Some(last.uri.clone()),
            playing: true,
            volume: settings::load_settings(&app_for_task)
                .map(|s| s.default_volume)
                .unwrap_or(0.8),
            muted: false,
            user_paused: false,
            current_time: 0.0,
            duration: 0.0,
            error: None,
            low_power: false,
        };
        apply_engine_runtime(&mut state, &app_for_task);
        let _ = wallpaper::push_command(&app_for_task, "set", &state);
    });
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
fn export_wallpaper(app: AppHandle, uri: String, file_name: String) -> Result<Option<String>, String> {
    let src = resolve_export_source(&app, &uri)
        .ok_or_else(|| "找不到可导出的源文件".to_string())?;
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
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_os::init())
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
            std::thread::spawn(move || restore_last_wallpaper(&app_for_restore));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            start_drag,
            minimize_main,
            hide_main,
            set_wallpaper,
            engine_play,
            engine_pause,
            engine_set_volume,
            engine_get_state,
            engine_report_progress,
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
            desktop_organize::open_desktop_item,
            desktop_organize::list_desktop_shell_context_menu,
            desktop_organize::list_desktop_shell_context_submenu,
            desktop_organize::invoke_desktop_shell_context_command,
            desktop_organize::show_desktop_native_context_menu,
            desktop_organize::show_desktop_item_in_folder,
            desktop_organize::open_desktop_item_with,
            desktop_organize::open_desktop_item_properties,
            desktop_organize::rename_desktop_item,
            desktop_organize::delete_desktop_item,
            desktop_organize::is_desktop_drag_over_foreign,
            desktop_organize::start_desktop_file_drag,
            desktop_organize::set_desktop_organize,
            desktop_organize::list_desktop_items,
            refresh_desktop_organize
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| {
            match event {
                tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit => {
                    desktop_organize::cleanup(app_handle);
                    wallpaper::cleanup(app_handle);
                }
                _ => {}
            }
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
