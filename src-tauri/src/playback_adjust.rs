//! Playback adjustment (WP-004)
//! Sets MPV playback parameters via IPC: speed, brightness, saturation, contrast.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackParams {
    pub speed: f64,
    pub brightness: i32,
    pub saturation: i32,
    pub contrast: i32,
}

impl Default for PlaybackParams {
    fn default() -> Self {
        Self {
            speed: 1.0,
            brightness: 0,
            saturation: 0,
            contrast: 0,
        }
    }
}

#[tauri::command]
pub fn set_playback_params(
    state: tauri::State<'_, crate::video_player::VideoPlayerState>,
    params: PlaybackParams,
) -> Result<(), String> {
    let guard = state.0.lock().map_err(|e| e.to_string())?;

    for pipe in &guard.ipc_pipes {
        // Set speed
        let _ = mpv_set_property(pipe, "speed", &params.speed.to_string());
        // Set brightness
        let _ = mpv_set_property(pipe, "brightness", &params.brightness.to_string());
        // Set saturation
        let _ = mpv_set_property(pipe, "saturation", &params.saturation.to_string());
        // Set contrast
        let _ = mpv_set_property(pipe, "contrast", &params.contrast.to_string());
    }

    log::info!("播放参数已更新: speed={}, brightness={}, saturation={}, contrast={}",
        params.speed, params.brightness, params.saturation, params.contrast);
    Ok(())
}

#[cfg(target_os = "windows")]
fn mpv_set_property(pipe: &str, property: &str, value: &str) -> Result<(), String> {
    use std::io::Write;

    let cmd = serde_json::json!({
        "command": ["set_property", property, value]
    });
    let mut payload = serde_json::to_string(&cmd).map_err(|e| e.to_string())?;
    payload.push('\n');

    if let Ok(mut file) = std::fs::OpenOptions::new().write(true).open(pipe) {
        let _ = file.write_all(payload.as_bytes());
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn mpv_set_property(_pipe: &str, _property: &str, _value: &str) -> Result<(), String> {
    Ok(())
}
