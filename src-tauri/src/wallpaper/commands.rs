//! Tauri commands and runtime helpers for the wallpaper engine.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use tauri::{AppHandle, Emitter, Manager, State};

use super::{push_command, push_state, EngineHandle, EngineState, SetWallpaperPayload};
use crate::settings;
use crate::system;

/// Min gap between routine `engine-state` progress emits (ms).
const PROGRESS_EMIT_MIN_MS: u64 = 1000;
static LAST_PROGRESS_EMIT_MS: AtomicU64 = AtomicU64::new(0);

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub struct RuntimeProfile {
    pub low_power: bool,
}

fn runtime_low_power(app: &AppHandle) -> bool {
    app.try_state::<RuntimeProfile>()
        .map(|p| p.low_power)
        .unwrap_or(false)
}

pub fn apply_engine_runtime(state: &mut EngineState, app: &AppHandle) {
    state.low_power = runtime_low_power(app);
}

#[tauri::command]
pub fn set_wallpaper(
    app: AppHandle,
    engine: State<'_, EngineHandle>,
    payload: SetWallpaperPayload,
) -> Result<EngineState, String> {
    tracing::info!(
        "[engine] set_wallpaper id={} mediaType={} uri={}",
        payload.id,
        payload.media_type,
        payload.uri
    );

    // Online wallpapers: always play from `.onlinefile/list.json` local path.
    // Never stream remote COS/R2 URLs into the wallpaper engine (overseas = very slow).
    let (play_uri, persist_uri) = if payload.id.starts_with("online-") {
        match crate::online_cache::resolve_online_asset_uri(&app, &payload.id)? {
            Some(local_uri) => {
                tracing::info!("[engine] set_wallpaper using local online cache uri={local_uri}");
                let persist = system::filesystem_path_from_uri(&local_uri)
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_else(|| local_uri.clone());
                (local_uri, persist)
            }
            None => {
                return Err("请先下载到本地后再设置壁纸".into());
            }
        }
    } else {
        let u = payload.uri.trim().to_string();
        if u.is_empty() {
            return Err("该资源暂无可用媒体".into());
        }
        if let Some(path) = system::filesystem_path_from_uri(&u) {
            if !system::path_is_nonempty_file(&path) {
                return Err("媒体文件不存在或为空，无法设为壁纸".into());
            }
            let play = crate::online_cache::path_to_asset_uri(&path);
            let persist = path.to_string_lossy().into_owned();
            (play, persist)
        } else if system::is_bundled_app_asset_uri(&u) {
            // Packaged /samples/* and same-origin app assets.
            (u.clone(), u)
        } else if u.starts_with("http://") || u.starts_with("https://") {
            return Err("无法直接使用远程地址设壁纸，请先下载到本地".into());
        } else {
            return Err("媒体文件不存在，无法设为壁纸".into());
        }
    };

    let default_volume = settings::load_settings(&app)
        .map(|s| s.default_volume.clamp(0.0, 1.0))
        .unwrap_or(0.0);
    let muted = default_volume < 0.01;
    let volume = if muted { 0.8 } else { default_volume };
    let mut state = engine
        .state
        .lock()
        .map_err(|_| "引擎状态锁失败".to_string())?;
    state.media_id = Some(payload.id.clone());
    state.title = Some(payload.title.clone());
    state.media_type = Some(payload.media_type.clone());
    state.uri = Some(play_uri.clone());
    state.playing = true;
    state.volume = volume;
    state.muted = muted;
    state.user_paused = false;
    state.error = None;
    state.current_time = 0.0;
    state.duration = 0.0;
    apply_engine_runtime(&mut state, &app);
    let snapshot = state.clone();
    drop(state);
    push_command(&app, "set", &snapshot)?;
    push_state(&app, &snapshot);
    let _ = settings::save_last_wallpaper(
        &app,
        &settings::LastWallpaper {
            id: payload.id,
            title: payload.title,
            media_type: payload.media_type,
            uri: persist_uri,
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
pub fn engine_play(app: AppHandle, engine: State<'_, EngineHandle>) -> Result<EngineState, String> {
    let mut state = engine
        .state
        .lock()
        .map_err(|_| "引擎状态锁失败".to_string())?;
    state.playing = true;
    state.user_paused = false;
    let snapshot = state.clone();
    drop(state);
    push_command(&app, "play", &snapshot)?;
    push_state(&app, &snapshot);
    Ok(snapshot)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PausePayload {
    #[serde(default = "default_true")]
    manual: bool,
}

fn default_true() -> bool {
    true
}

#[tauri::command]
pub fn engine_pause(
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
    push_command(&app, "pause", &snapshot)?;
    push_state(&app, &snapshot);
    Ok(snapshot)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumePayload {
    volume: f64,
    muted: bool,
}

#[tauri::command]
pub fn engine_set_volume(
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
    push_command(&app, "volume", &snapshot)?;
    push_state(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
pub fn engine_get_state(engine: State<'_, EngineHandle>) -> Result<EngineState, String> {
    engine
        .state
        .lock()
        .map(|s| s.clone())
        .map_err(|_| "引擎状态锁失败".into())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct ProgressPayload {
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
pub fn engine_report_progress(
    app: AppHandle,
    engine: State<'_, EngineHandle>,
    payload: ProgressPayload,
) -> Result<(), String> {
    let mut state = engine
        .state
        .lock()
        .map_err(|_| "引擎状态锁失败".to_string())?;
    let prev_playing = state.playing;
    let prev_error = state.error.clone();
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
    let playing_changed = state.playing != prev_playing;
    let error_changed = state.error != prev_error;
    let now = now_unix_ms();
    let last = LAST_PROGRESS_EMIT_MS.load(Ordering::Relaxed);
    let due = now.saturating_sub(last) >= PROGRESS_EMIT_MIN_MS;
    // Always emit play/pause/error flips; throttle time-only ticks so the main
    // UI is not flooded (was every 250ms → Not Responding after long runs).
    let should_emit = playing_changed || error_changed || due || payload.error.is_some();
    let snapshot = if should_emit {
        Some(state.clone())
    } else {
        None
    };
    drop(state);
    if let Some(snapshot) = snapshot {
        LAST_PROGRESS_EMIT_MS.store(now, Ordering::Relaxed);
        let _ = app.emit("engine-state", &snapshot);
    }
    Ok(())
}

pub fn clear_engine_wallpaper(app: &AppHandle, engine: &EngineHandle) -> Result<EngineState, String> {
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
    // Detach wallpaper window so an empty black WebView does not cover the desktop.
    crate::wallpaper::cleanup(app);
    push_state(app, &snapshot);
    let _ = settings::clear_last_wallpaper(app);
    Ok(snapshot)
}

pub fn restore_last_wallpaper(app: &AppHandle) {
    let app_for_task = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
        let last = match settings::load_last_wallpaper(&app_for_task) {
            Ok(Some(l)) => l,
            _ => {
                tracing::info!("[engine] skip restore: no last wallpaper");
                return;
            }
        };

        // Prefer local `.onlinefile` path for online ids; never restore overseas COS URLs.
        let play_uri = if last.id.starts_with("online-") {
            match crate::online_cache::resolve_online_asset_uri(&app_for_task, &last.id) {
                Ok(Some(local_uri)) => local_uri,
                Ok(None) => {
                    tracing::info!(
                        "[engine] skip restore: online cache missing id={}",
                        last.id
                    );
                    let _ = settings::clear_last_wallpaper(&app_for_task);
                    return;
                }
                Err(e) => {
                    tracing::warn!("[engine] skip restore: resolve online cache failed: {e}");
                    return;
                }
            }
        } else if let Some(path) = system::filesystem_path_from_uri(&last.uri) {
            if !system::path_is_nonempty_file(&path) {
                tracing::info!(
                    "[engine] skip restore: media missing path={}",
                    path.display()
                );
                let _ = settings::clear_last_wallpaper(&app_for_task);
                return;
            }
            crate::online_cache::path_to_asset_uri(&path)
        } else {
            tracing::info!(
                "[engine] skip restore: no local file uri={}",
                last.uri
            );
            let _ = settings::clear_last_wallpaper(&app_for_task);
            return;
        };

        let mut state = EngineState {
            media_id: Some(last.id.clone()),
            title: Some(last.title.clone()),
            media_type: Some(last.media_type.clone()),
            uri: Some(play_uri),
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
        if let Err(e) = push_command(&app_for_task, "set", &state) {
            tracing::warn!("[engine] restore push_command failed: {e}");
            crate::wallpaper::cleanup(&app_for_task);
            return;
        }
        // Keep engine handle in sync if registered.
        if let Some(engine) = app_for_task.try_state::<EngineHandle>() {
            if let Ok(mut guard) = engine.state.lock() {
                *guard = state.clone();
            }
        }
        push_state(&app_for_task, &state);
    });
}
