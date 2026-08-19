use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EngineState {
    pub media_id: Option<String>,
    pub title: Option<String>,
    pub media_type: Option<String>,
    pub uri: Option<String>,
    pub playing: bool,
    pub volume: f64,
    pub muted: bool,
    pub current_time: f64,
    pub duration: f64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetWallpaperPayload {
    pub id: String,
    pub title: String,
    pub media_type: String,
    pub uri: String,
}

pub struct EngineHandle {
    pub state: Mutex<EngineState>,
}

impl Default for EngineHandle {
    fn default() -> Self {
        Self {
            state: Mutex::new(EngineState {
                playing: true,
                volume: 0.8,
                muted: false,
                ..Default::default()
            }),
        }
    }
}

const WALLPAPER_LABEL: &str = "wallpaper";

#[cfg(windows)]
mod win {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{HWND, LPARAM, WPARAM};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, FindWindowExW, FindWindowW, GetSystemMetrics, SendMessageTimeoutW, SetParent,
        SetWindowPos, SM_CXSCREEN, SM_CYSCREEN, SMTO_NORMAL, SWP_NOACTIVATE, SWP_SHOWWINDOW,
    };

    fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    struct EnumData {
        worker: HWND,
    }

    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> i32 {
        let data = &mut *(lparam as *mut EnumData);
        let class_def = wide("SHELLDLL_DefView");
        let def = FindWindowExW(hwnd, std::ptr::null_mut(), class_def.as_ptr(), std::ptr::null());
        if !def.is_null() {
            let class_worker = wide("WorkerW");
            let next = FindWindowExW(
                std::ptr::null_mut(),
                hwnd,
                class_worker.as_ptr(),
                std::ptr::null(),
            );
            if !next.is_null() {
                data.worker = next;
            }
        }
        1
    }

    pub fn attach_to_desktop(hwnd_raw: isize) -> Result<(), String> {
        unsafe {
            let progman_class = wide("Progman");
            let progman = FindWindowW(progman_class.as_ptr(), std::ptr::null());
            if progman.is_null() {
                return Err("未找到 Progman 窗口".into());
            }

            let mut result: usize = 0;
            SendMessageTimeoutW(
                progman,
                0x052C,
                0 as WPARAM,
                0 as LPARAM,
                SMTO_NORMAL,
                1000,
                &mut result,
            );

            let mut data = EnumData {
                worker: std::ptr::null_mut(),
            };
            EnumWindows(Some(enum_windows_proc), &mut data as *mut _ as LPARAM);

            let worker = if !data.worker.is_null() {
                data.worker
            } else {
                let class_worker = wide("WorkerW");
                FindWindowExW(
                    progman,
                    std::ptr::null_mut(),
                    class_worker.as_ptr(),
                    std::ptr::null(),
                )
            };

            if worker.is_null() {
                return Err("未找到 WorkerW，无法附着桌面壁纸层".into());
            }

            let child = hwnd_raw as HWND;
            if SetParent(child, worker).is_null() {
                return Err("SetParent 失败".into());
            }

            let w = GetSystemMetrics(SM_CXSCREEN);
            let h = GetSystemMetrics(SM_CYSCREEN);
            SetWindowPos(
                child,
                std::ptr::null_mut(),
                0,
                0,
                w,
                h,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
            Ok(())
        }
    }
}

pub fn ensure_wallpaper(app: &AppHandle) -> Result<WebviewWindow, String> {
    if let Some(w) = app.get_webview_window(WALLPAPER_LABEL) {
        return Ok(w);
    }

    let window = WebviewWindowBuilder::new(
        app,
        WALLPAPER_LABEL,
        WebviewUrl::App("wallpaper.html".into()),
    )
    .title("LINGJING Wallpaper")
    .decorations(false)
    .transparent(false)
    .skip_taskbar(true)
    .visible(false)
    .focused(false)
    .resizable(false)
    .maximizable(false)
    .minimizable(false)
    .closable(false)
    .build()
    .map_err(|e| format!("创建壁纸窗口失败: {e}"))?;

    #[cfg(windows)]
    {
        let hwnd = window.hwnd().map_err(|e| format!("获取 HWND 失败: {e}"))?;
        win::attach_to_desktop(hwnd.0 as isize)?;
        let _ = window.show();
        Ok(window)
    }
    #[cfg(not(windows))]
    {
        let _ = window;
        Err("壁纸引擎仅支持 Windows".into())
    }
}

pub fn push_command(app: &AppHandle, cmd: &str, state: &EngineState) -> Result<(), String> {
    let window = ensure_wallpaper(app)?;
    #[derive(Serialize, Clone)]
    #[serde(rename_all = "camelCase")]
    struct Msg<'a> {
        cmd: &'a str,
        state: &'a EngineState,
    }
    window
        .emit("wallpaper-cmd", Msg { cmd, state })
        .map_err(|e| format!("向壁纸窗口发送命令失败: {e}"))?;
    let json = serde_json::to_string(state).unwrap_or_else(|_| "{}".into());
    let script = format!("window.__wallpaperApply && window.__wallpaperApply({json}, {cmd:?});");
    let _ = window.eval(&script);
    Ok(())
}

pub fn destroy_wallpaper(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(WALLPAPER_LABEL) {
        let _ = w.close();
    }
}
