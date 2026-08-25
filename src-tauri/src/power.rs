use crate::desktop_organize;
use crate::settings;
use crate::wallpaper::{self, EngineHandle};
use serde::Serialize;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use tauri::{AppHandle, Emitter, Listener, Manager};

#[cfg(windows)]
mod win {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetSystemMetrics, GetWindowLongW, GetWindowRect, GWL_STYLE,
        SM_REMOTESESSION, WS_MAXIMIZE, WS_POPUP,
    };

    pub fn is_remote_session() -> bool {
        unsafe { GetSystemMetrics(SM_REMOTESESSION) != 0 }
    }

    pub fn is_foreground_fullscreen() -> bool {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_null() {
                return false;
            }
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            if GetWindowRect(hwnd, &mut rect) == 0 {
                return false;
            }
            let mon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
            if mon.is_null() {
                return false;
            }
            let mut mi = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            if GetMonitorInfoW(mon, &mut mi) == 0 {
                return false;
            }
            let mr = mi.rcMonitor;
            let covers_monitor = rect.left >= mr.left
                && rect.top >= mr.top
                && rect.right <= mr.right
                && rect.bottom <= mr.bottom;
            if !covers_monitor {
                return false;
            }
            let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
            let maximized = style & WS_MAXIMIZE != 0;
            let popup = style & WS_POPUP != 0;
            // 普通最大化窗口（浏览器、资源管理器等）不触发暂停
            !(maximized && !popup)
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
    tauri::async_runtime::spawn(async move {
        let _ = app.listen("settings-updated", move |event| {
            if let Ok(next) = serde_json::from_str::<settings::AppSettings>(event.payload()) {
                if let Ok(mut guard) = flags_for_listen.write() {
                    *guard = Flags::from(next);
                }
            }
        });
        std::future::pending::<()>().await;
    });
}

fn handle_system_resume(app: &AppHandle) {
    eprintln!("[power] system resume detected");
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
                    resume_grace_until = now + Duration::from_secs(8);
                    handle_system_resume(&app);
                    // Reset edge detectors so wake-time fullscreen/lock UI
                    // does not leave the engine stuck paused.
                    last_fullscreen = false;
                    last_remote = win::is_remote_session();
                    last_battery_state = Some(read_battery_state());
                }
            }
            last_wall = now;

            let in_grace = now < resume_grace_until;
            let flags = flags.read().map(|g| *g).unwrap_or_default();
            if flags.pause_on_fullscreen && !in_grace {
                let fs = win::is_foreground_fullscreen();
                if fs != last_fullscreen {
                    last_fullscreen = fs;
                    let payload = PauseRecommendPayload {
                        action: if fs { "pause" } else { "play" },
                        reason: "fullscreen",
                    };
                    let _ = app.emit("engine-pause-recommend", &payload);
                }
            }
            if flags.pause_on_rdp {
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
            if flags.pause_on_battery {
                let on_battery = read_battery_state();
                if Some(on_battery) != last_battery_state {
                    last_battery_state = Some(on_battery);
                    let payload = PauseRecommendPayload {
                        action: if on_battery { "pause" } else { "play" },
                        reason: if on_battery { "battery" } else { "power" },
                    };
                    let _ = app.emit("engine-pause-recommend", &payload);
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });
}

fn read_battery_state() -> bool {
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
        unsafe {
            let mut status: SYSTEM_POWER_STATUS = std::mem::zeroed();
            if GetSystemPowerStatus(&mut status) != 0 {
                return status.ACLineStatus == 0;
            }
            false
        }
    }
    #[cfg(not(windows))]
    {
        false
    }
}
