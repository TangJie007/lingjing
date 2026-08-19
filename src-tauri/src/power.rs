use crate::settings;
use serde::Serialize;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Listener};

#[cfg(windows)]
mod win {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetSystemMetrics, GetWindowRect, SM_REMOTESESSION,
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
            rect.left >= mr.left
                && rect.top >= mr.top
                && rect.right <= mr.right
                && rect.bottom <= mr.bottom
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
        loop {
            let flags = flags.read().map(|g| *g).unwrap_or_default();
            if flags.pause_on_fullscreen {
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
