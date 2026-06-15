//! Auto wallpaper rotation (SET-007)
//! Schedules wallpaper changes at configurable intervals.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{Emitter, Manager};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RotateConfig {
    pub enabled: bool,
    pub source: String,     // "all" | "favorites" | "tag:xxx" | "manual"
    pub interval: String,   // "15min" | "30min" | "1hour" | "daily" | "unlock"
    pub order: String,      // "sequential" | "random"
    pub time_range: String, // "all" | "work" | "custom"
    pub custom_start: Option<String>,
    pub custom_end: Option<String>,
}

impl Default for RotateConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            source: "all".into(),
            interval: "30min".into(),
            order: "sequential".into(),
            time_range: "all".into(),
            custom_start: None,
            custom_end: None,
        }
    }
}

pub struct RotateState {
    pub config: Mutex<RotateConfig>,
    pub running: AtomicBool,
}

impl Default for RotateState {
    fn default() -> Self {
        Self {
            config: Mutex::new(RotateConfig::default()),
            running: AtomicBool::new(false),
        }
    }
}

fn interval_to_ms(interval: &str) -> u64 {
    match interval {
        "15min" => 15 * 60 * 1000,
        "30min" => 30 * 60 * 1000,
        "1hour" => 60 * 60 * 1000,
        "daily" => 24 * 60 * 60 * 1000,
        _ => 30 * 60 * 1000,
    }
}

#[tauri::command]
pub fn set_rotate_config(
    state: tauri::State<'_, RotateState>,
    config: RotateConfig,
) -> Result<(), String> {
    let mut guard = state.config.lock().map_err(|e| e.to_string())?;
    *guard = config;
    Ok(())
}

#[tauri::command]
pub fn get_rotate_config(
    state: tauri::State<'_, RotateState>,
) -> Result<RotateConfig, String> {
    let guard = state.config.lock().map_err(|e| e.to_string())?;
    Ok(guard.clone())
}

/// Start the rotation timer. Called from the main thread.
pub fn start_rotation(app: tauri::AppHandle) {
    let state = app.state::<RotateState>();
    if state.running.swap(true, Ordering::SeqCst) {
        return;
    }

    let app_clone = app.clone();
    std::thread::spawn(move || {
        loop {
            let interval_ms = {
                let rotate_state = app_clone.state::<RotateState>();
                let guard = rotate_state.config.lock().unwrap();
                if !guard.enabled {
                    drop(guard);
                    rotate_state.running.store(false, Ordering::SeqCst);
                    break;
                }
                interval_to_ms(&guard.interval)
            };

            std::thread::sleep(Duration::from_millis(interval_ms));

            let rotate_state = app_clone.state::<RotateState>();
            if !rotate_state.running.load(Ordering::SeqCst) {
                break;
            }

            let _ = app_clone.emit("wallpaper-rotate", ());
        }
    });
}

/// Stop the rotation timer.
pub fn stop_rotation(state: &RotateState) {
    state.running.store(false, Ordering::SeqCst);
}
