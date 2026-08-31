//! Wallpaper webview engine: WorkerW attach, state push, JS inject.

pub mod commands;
mod types;
#[cfg(windows)]
mod win;

pub use types::{EngineHandle, EngineState, MonitorTile, SetWallpaperPayload};

use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager, PhysicalSize, WebviewWindow};

const WALLPAPER_LABEL: &str = "wallpaper";
static ATTACHED: AtomicBool = AtomicBool::new(false);

fn run_on_ui<T, F>(app: &AppHandle, f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    app.run_on_main_thread(move || {
        let _ = tx.send(f());
    })
    .map_err(|e| format!("无法切到 UI 线程: {e}"))?;
    rx.recv()
        .map_err(|e| format!("等待 UI 线程失败: {e}"))
}

fn attach_existing_inner(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(WALLPAPER_LABEL)
        .ok_or_else(|| "wallpaper window not registered".to_string())?;
    #[cfg(windows)]
    {
        let _ = window.show();
        let _ = window.set_ignore_cursor_events(true);
        let hwnd = window.hwnd().map_err(|e| format!("获取 HWND 失败: {e}"))?;
        let (w, h) = win::attach_to_desktop(hwnd.0 as isize)?;
        let _ = window.set_size(PhysicalSize::new(w as u32, h as u32));
        let _ = window.show();
        ATTACHED.store(true, Ordering::SeqCst);
    }
    #[cfg(not(windows))]
    {
        let _ = window;
        return Err("壁纸引擎仅支持 Windows".into());
    }
    Ok(())
}

pub fn attach_existing(app: &AppHandle) -> Result<(), String> {
    let handle = app.clone();
    run_on_ui(app, move || attach_existing_inner(&handle))?
}

pub fn cleanup(app: &AppHandle) {
    if !ATTACHED.load(Ordering::SeqCst) {
        return;
    }
    let Some(window) = app.get_webview_window(WALLPAPER_LABEL) else {
        ATTACHED.store(false, Ordering::SeqCst);
        return;
    };
    #[cfg(windows)]
    if let Ok(hwnd) = window.hwnd() {
        win::detach_from_desktop(hwnd.0 as isize);
    }
    let _ = window.hide();
    ATTACHED.store(false, Ordering::SeqCst);
    tracing::info!("[wallpaper] cleanup detached from desktop");
}

/// After sleep/hibernate, WorkerW may be recreated and video elements may stay paused.
/// Force re-attach and push play when the engine should still be playing.
pub fn on_system_resume(app: &AppHandle, should_play: bool, state: &EngineState) {
    tracing::info!(
        "[wallpaper] system resume should_play={should_play} media={:?}",
        state.media_id
    );
    ATTACHED.store(false, Ordering::SeqCst);
    if let Err(e) = attach_existing(app) {
        tracing::info!("[wallpaper] resume reattach failed: {e}");
        return;
    }
    if should_play && state.media_id.is_some() {
        let mut snap = state.clone();
        snap.playing = true;
        let _ = push_command(app, "play", &snap);
    } else if state.media_id.is_some() {
        let _ = push_command(app, "set", state);
    }
}

fn ensure_wallpaper_inner(app: &AppHandle) -> Result<WebviewWindow, String> {
    let window = app
        .get_webview_window(WALLPAPER_LABEL)
        .ok_or_else(|| "wallpaper window not registered".to_string())?;
    if !ATTACHED.load(Ordering::SeqCst) {
        attach_existing_inner(app)?;
    }
    Ok(window)
}

pub fn push_state(app: &AppHandle, state: &EngineState) {
    let _ = app.emit("engine-state", state);
}

fn inject_command(window: &WebviewWindow, cmd: &str, state: &EngineState) -> Result<(), String> {
    let layout = {
        #[cfg(windows)]
        {
            match window.hwnd() {
                Ok(hwnd) => {
                    let (w, h, tiles) = win::layout_for_child(hwnd.0 as isize);
                    let _ = window.set_size(PhysicalSize::new(w as u32, h as u32));
                    tracing::info!("[wallpaper] layout tiles={tiles:?}");
                    tiles
                }
                Err(_) => Vec::new(),
            }
        }
        #[cfg(not(windows))]
        Vec::new()
    };
    #[derive(Serialize, Clone)]
    #[serde(rename_all = "camelCase")]
    struct Msg<'a> {
        cmd: &'a str,
        state: &'a EngineState,
        layout: &'a [MonitorTile],
    }
    let _ = window.emit(
        "wallpaper-cmd",
        &Msg {
            cmd,
            state,
            layout: &layout,
        },
    );
    let json = serde_json::to_string(state).unwrap_or_else(|_| "{}".into());
    let layout_json = serde_json::to_string(&layout).unwrap_or_else(|_| "[]".into());
    let script = format!(
        r#"(function(s,c,layout,n){{
  function go(){{
    if (window.__wallpaperApply) {{ window.__wallpaperApply(s,c,layout); return; }}
    if (n < 25) {{ n += 1; setTimeout(go, 120); }}
  }}
  go();
}})({json}, {cmd:?}, {layout_json}, 0);"#
    );
    window
        .eval(&script)
        .map_err(|e| format!("向壁纸窗口注入命令失败: {e}"))?;
    Ok(())
}

pub fn push_command(app: &AppHandle, cmd: &str, state: &EngineState) -> Result<(), String> {
    let handle = app.clone();
    let cmd = cmd.to_string();
    let state = state.clone();
    run_on_ui(app, move || {
        let window = ensure_wallpaper_inner(&handle)?;
        let _ = window.show();
        inject_command(&window, &cmd, &state)
    })?
}
