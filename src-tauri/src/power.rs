use crate::desktop_organize;
use crate::settings;
use crate::wallpaper::{self, EngineHandle};
use serde::Serialize;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use tauri::{AppHandle, Emitter, Listener, Manager};

#[cfg(windows)]
mod win {
    use windows::Win32::Foundation::{HWND, RECT};
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetClassNameW, GetForegroundWindow, GetSystemMetrics, GetWindowLongW, GetWindowRect,
        GWL_EXSTYLE, GWL_STYLE, SM_REMOTESESSION, WS_CAPTION, WS_EX_TOPMOST, WS_MAXIMIZE, WS_POPUP,
        WS_THICKFRAME,
    };

    pub fn is_remote_session() -> bool {
        unsafe { GetSystemMetrics(SM_REMOTESESSION) != 0 }
    }

    fn class_name(hwnd: HWND) -> String {
        unsafe {
            let mut buf = [0u16; 256];
            let n = GetClassNameW(hwnd, &mut buf);
            if n <= 0 {
                return String::new();
            }
            String::from_utf16_lossy(&buf[..n as usize])
        }
    }

    fn is_shell_or_system_window(hwnd: HWND) -> bool {
        let cls = class_name(hwnd);
        matches!(
            cls.as_str(),
            "Progman"
                | "WorkerW"
                | "Shell_TrayWnd"
                | "Shell_SecondaryTrayWnd"
                | "XamlExplorerHostIslandWindow"
                | "MultitaskingViewFrame"
                | "ForegroundStaging"
                | "Windows.UI.Core.CoreWindow"
                | "ImmersiveLauncher"
                | "Windows.Internal.Shell.TabProxyWindow"
        ) || cls.starts_with("LockScreen")
    }

    /// True only for exclusive / borderless fullscreen-like windows that cover the monitor.
    pub fn is_foreground_fullscreen() -> bool {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0.is_null() || is_shell_or_system_window(hwnd) {
                return false;
            }
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            if GetWindowRect(hwnd, &mut rect).is_err() {
                return false;
            }
            let mon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
            if mon.is_invalid() {
                return false;
            }
            let mut mi = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                rcMonitor: RECT::default(),
                rcWork: RECT::default(),
                dwFlags: 0,
            };
            if !GetMonitorInfoW(mon, &mut mi).as_bool() {
                return false;
            }
            let mr = mi.rcMonitor;
            let mon_w = (mr.right - mr.left).max(1) as i64;
            let mon_h = (mr.bottom - mr.top).max(1) as i64;
            let win_w = (rect.right - rect.left).max(0) as i64;
            let win_h = (rect.bottom - rect.top).max(0) as i64;
            if win_w <= 0 || win_h <= 0 {
                return false;
            }

            // Must actually cover the monitor (previous bug: "contained in" was inverted).
            let covers = rect.left <= mr.left + 2
                && rect.top <= mr.top + 2
                && rect.right >= mr.right - 2
                && rect.bottom >= mr.bottom - 2;
            if !covers {
                return false;
            }
            let area_ratio = (win_w * win_h) as f64 / (mon_w * mon_h) as f64;
            if area_ratio < 0.95 {
                return false;
            }

            let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
            let ex = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
            let maximized = style & WS_MAXIMIZE.0 != 0;
            let popup = style & WS_POPUP.0 != 0;
            let has_caption = style & WS_CAPTION.0 != 0;
            let thickframe = style & WS_THICKFRAME.0 != 0;
            let topmost = ex & WS_EX_TOPMOST.0 != 0;

            // Ordinary maximized desktop apps (browser, explorer, IDE) → not "fullscreen mode".
            if maximized && has_caption && !popup {
                return false;
            }
            // Borderless / exclusive / game-style fullscreen.
            popup || !has_caption || !thickframe || topmost || (!maximized && area_ratio >= 0.98)
        }
    }
}

#[cfg(not(windows))]
mod win {
    pub fn is_remote_session() -> bool {
        false
    }
    pub fn is_foreground_fullscreen() -> bool {
        false
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PauseRecommendPayload {
    pub action: &'static str,
    pub reason: &'static str,
}

pub struct PowerWatcher;

impl Default for PowerWatcher {
    fn default() -> Self {
        Self
    }
}

#[derive(Default, Debug, Clone, Copy)]
struct Flags {
    pause_on_fullscreen: bool,
    pause_on_battery: bool,
    pause_on_rdp: bool,
}

impl From<settings::AppSettings> for Flags {
    fn from(s: settings::AppSettings) -> Self {
        Self {
            pause_on_fullscreen: s.pause_on_fullscreen,
            pause_on_battery: s.pause_on_battery,
            pause_on_rdp: s.pause_on_rdp,
        }
    }
}

fn spawn_settings_listener(app: AppHandle, flags: Arc<RwLock<Flags>>) {
    let flags_for_listen = flags.clone();
    let app_for_resume = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = app.listen("settings-updated", move |event| {
            let prev = flags_for_listen.read().map(|g| *g).unwrap_or_default();
            if let Ok(next) = serde_json::from_str::<settings::AppSettings>(event.payload()) {
                let next_flags = Flags::from(next);
                if let Ok(mut guard) = flags_for_listen.write() {
                    *guard = next_flags;
                }
                // Turning an auto-pause switch off should not leave playback stuck.
                let disabled_pause = (prev.pause_on_fullscreen && !next_flags.pause_on_fullscreen)
                    || (prev.pause_on_battery && !next_flags.pause_on_battery)
                    || (prev.pause_on_rdp && !next_flags.pause_on_rdp);
                if disabled_pause {
                    let _ = app_for_resume.emit(
                        "engine-pause-recommend",
                        &PauseRecommendPayload {
                            action: "play",
                            reason: "settings",
                        },
                    );
                }
            }
        });
        std::future::pending::<()>().await;
    });
}

fn handle_system_resume(app: &AppHandle) {
    tracing::info!("[power] system resume detected");
    let _ = app.emit(
        "engine-pause-recommend",
        &PauseRecommendPayload {
            action: "resume-system",
            reason: "resume",
        },
    );

    let (should_play, snap) = app
        .try_state::<EngineHandle>()
        .and_then(|engine| {
            engine.state.lock().ok().map(|s| {
                let play = s.media_id.is_some() && !s.user_paused;
                (play, s.clone())
            })
        })
        .unwrap_or_else(|| (false, Default::default()));

    let app_clone = app.clone();
    let snap_clone = snap.clone();
    // Give Explorer a moment to rebuild WorkerW after wake.
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(800)).await;
        wallpaper::on_system_resume(&app_clone, should_play, &snap_clone);
        desktop_organize::reassert(&app_clone);
        // Second pass: some machines rebuild desktop layers slowly.
        tokio::time::sleep(Duration::from_millis(1600)).await;
        wallpaper::on_system_resume(&app_clone, should_play, &snap_clone);
        desktop_organize::reassert(&app_clone);
    });
}

pub fn start_watcher(app: AppHandle) {
    let flags = Arc::new(RwLock::new(
        settings::load_settings(&app)
            .map(Flags::from)
            .unwrap_or_default(),
    ));
    spawn_settings_listener(app.clone(), flags.clone());

    tauri::async_runtime::spawn(async move {
        let mut last_fullscreen = false;
        let mut last_remote = false;
        let mut last_battery_state: Option<bool> = None;
        let mut last_wall = SystemTime::now();
        let mut resume_grace_until = SystemTime::UNIX_EPOCH;
        loop {
            let now = SystemTime::now();
            if let Ok(gap) = now.duration_since(last_wall) {
                // Instant does not advance during sleep; wall clock does.
                if gap > Duration::from_secs(4) {
                    resume_grace_until = now + Duration::from_secs(12);
                    handle_system_resume(&app);
                    // Reset edge detectors so wake-time lock UI / AC flicker
                    // does not leave the engine stuck paused.
                    last_fullscreen = false;
                    last_remote = win::is_remote_session();
                    last_battery_state = read_battery_state();
                }
            }
            last_wall = now;

            let in_grace = now < resume_grace_until;
            let flags = flags.read().map(|g| *g).unwrap_or_default();

            if flags.pause_on_fullscreen {
                if in_grace {
                    last_fullscreen = false;
                } else {
                    let fs = win::is_foreground_fullscreen();
                    if fs != last_fullscreen {
                        last_fullscreen = fs;
                        tracing::info!("[power] fullscreen detect -> {fs}");
                        let payload = PauseRecommendPayload {
                            action: if fs { "pause" } else { "play" },
                            reason: "fullscreen",
                        };
                        let _ = app.emit("engine-pause-recommend", &payload);
                    }
                }
            } else if last_fullscreen {
                // Setting turned off while we thought we were fullscreen.
                last_fullscreen = false;
            }

            if flags.pause_on_rdp && !in_grace {
                let remote = win::is_remote_session();
                if remote != last_remote {
                    last_remote = remote;
                    let payload = PauseRecommendPayload {
                        action: if remote { "pause" } else { "play" },
                        reason: "rdp",
                    };
                    let _ = app.emit("engine-pause-recommend", &payload);
                }
            }

            if flags.pause_on_battery && !in_grace {
                if let Some(on_battery) = read_battery_state() {
                    if Some(on_battery) != last_battery_state {
                        last_battery_state = Some(on_battery);
                        let payload = PauseRecommendPayload {
                            action: if on_battery { "pause" } else { "play" },
                            reason: if on_battery { "battery" } else { "power" },
                        };
                        let _ = app.emit("engine-pause-recommend", &payload);
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });
}

/// `None` = unknown / unreliable reading (do not edge-trigger pause).
fn read_battery_state() -> Option<bool> {
    #[cfg(windows)]
    {
        use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
        unsafe {
            let mut status: SYSTEM_POWER_STATUS = std::mem::zeroed();
            if GetSystemPowerStatus(&mut status).is_err() {
                return None;
            }
            // 0 = offline (battery), 1 = AC, 255 = unknown
            match status.ACLineStatus {
                0 => Some(true),
                1 => Some(false),
                _ => None,
            }
        }
    }
    #[cfg(not(windows))]
    {
        None
    }
}
