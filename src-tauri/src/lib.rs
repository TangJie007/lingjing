mod favorites;
mod library;
mod wallpaper;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use wallpaper::{EngineHandle, EngineState, SetWallpaperPayload};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust!")
}

#[tauri::command]
fn start_drag(window: tauri::Window) {
    let _ = window.start_dragging();
}

#[tauri::command]
fn set_wallpaper(
    app: AppHandle,
    engine: State<'_, EngineHandle>,
    payload: SetWallpaperPayload,
) -> Result<EngineState, String> {
    if payload.uri.trim().is_empty() {
        return Err("该资源暂无可用媒体".into());
    }
    let mut state = engine.state.lock().map_err(|_| "引擎状态锁失败".to_string())?;
    state.media_id = Some(payload.id);
    state.title = Some(payload.title);
    state.media_type = Some(payload.media_type);
    state.uri = Some(payload.uri);
    state.playing = true;
    state.error = None;
    state.current_time = 0.0;
    state.duration = 0.0;
    let snapshot = state.clone();
    drop(state);
    wallpaper::push_command(&app, "set", &snapshot)?;
    let _ = app.emit("engine-state", &snapshot);
    Ok(snapshot)
}

#[tauri::command]
fn engine_play(app: AppHandle, engine: State<'_, EngineHandle>) -> Result<EngineState, String> {
    let mut state = engine.state.lock().map_err(|_| "引擎状态锁失败".to_string())?;
    state.playing = true;
    let snapshot = state.clone();
    drop(state);
    wallpaper::push_command(&app, "play", &snapshot)?;
    let _ = app.emit("engine-state", &snapshot);
    Ok(snapshot)
}

#[tauri::command]
fn engine_pause(app: AppHandle, engine: State<'_, EngineHandle>) -> Result<EngineState, String> {
    let mut state = engine.state.lock().map_err(|_| "引擎状态锁失败".to_string())?;
    state.playing = false;
    let snapshot = state.clone();
    drop(state);
    wallpaper::push_command(&app, "pause", &snapshot)?;
    let _ = app.emit("engine-state", &snapshot);
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
    let mut state = engine.state.lock().map_err(|_| "引擎状态锁失败".to_string())?;
    state.volume = payload.volume.clamp(0.0, 1.0);
    state.muted = payload.muted;
    let snapshot = state.clone();
    drop(state);
    wallpaper::push_command(&app, "volume", &snapshot)?;
    let _ = app.emit("engine-state", &snapshot);
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
struct ProgressPayload {
    current_time: f64,
    duration: f64,
    playing: Option<bool>,
    error: Option<String>,
}

#[tauri::command]
fn engine_report_progress(
    app: AppHandle,
    engine: State<'_, EngineHandle>,
    payload: ProgressPayload,
) -> Result<(), String> {
    let mut state = engine.state.lock().map_err(|_| "引擎状态锁失败".to_string())?;
    state.current_time = payload.current_time;
    state.duration = payload.duration;
    if let Some(p) = payload.playing {
        state.playing = p;
    }
    if payload.error.is_some() {
        state.error = payload.error.clone();
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
    Ok(AppPaths {
        app_data_dir: app_data.to_string_lossy().to_string(),
        library_dir: app_data.join("library").to_string_lossy().to_string(),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(EngineHandle::default())
        .invoke_handler(tauri::generate_handler![
            greet,
            start_drag,
            set_wallpaper,
            engine_play,
            engine_pause,
            engine_set_volume,
            engine_get_state,
            engine_report_progress,
            list_library,
            import_media,
            load_favorites,
            set_favorite,
            get_app_paths
        ])
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::Destroyed = event {
                    wallpaper::destroy_wallpaper(window.app_handle());
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
