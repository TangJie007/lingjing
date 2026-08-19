use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, PhysicalSize, WebviewWindow};

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
    #[serde(default)]
    pub user_paused: bool,
    #[serde(default)]
    pub low_power: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MonitorTile {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub primary: bool,
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

#[cfg(windows)]
mod win {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{HWND, LPARAM, RECT};
    use windows_sys::Win32::Graphics::Gdi::{
        EnumDisplayMonitors, GetMonitorInfoW, RedrawWindow, HDC, HMONITOR, MONITORINFO,
        RDW_ALLCHILDREN, RDW_INVALIDATE, RDW_UPDATENOW,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, FindWindowExW, FindWindowW, GetClassNameW, GetParent, GetSystemMetrics,
        GetWindowLongPtrW, GetWindowRect, IsWindowVisible, SendMessageTimeoutW,
        SetLayeredWindowAttributes, SetParent, SetWindowLongPtrW, SetWindowPos, ShowWindow,
        GWL_EXSTYLE, HWND_BOTTOM, LWA_ALPHA, MONITORINFOF_PRIMARY, SMTO_NORMAL, SM_CXSCREEN,
        SM_CYSCREEN, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
        SWP_SHOWWINDOW, SW_SHOW, WS_EX_APPWINDOW, WS_EX_LAYERED, WS_EX_NOACTIVATE,
        WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW,
    };

    fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    fn class_name(hwnd: HWND) -> String {
        unsafe {
            let mut buf = [0u16; 256];
            let n = GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
            if n <= 0 {
                return String::new();
            }
            String::from_utf16_lossy(&buf[..n as usize])
        }
    }

    fn rect_of(hwnd: HWND) -> (i32, i32, i32, i32) {
        unsafe {
            let mut r = RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            GetWindowRect(hwnd, &mut r);
            (r.left, r.top, r.right - r.left, r.bottom - r.top)
        }
    }

    unsafe extern "system" fn enum_monitors_proc(
        hmon: HMONITOR,
        _hdc: HDC,
        _lprc: *mut RECT,
        lparam: LPARAM,
    ) -> i32 {
        let list = &mut *(lparam as *mut Vec<(i32, i32, i32, i32, bool)>);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if GetMonitorInfoW(hmon, &mut info) != 0 {
            let r = info.rcMonitor;
            list.push((
                r.left,
                r.top,
                r.right - r.left,
                r.bottom - r.top,
                info.dwFlags & MONITORINFOF_PRIMARY != 0,
            ));
        }
        1
    }

    fn enum_monitors() -> Vec<(i32, i32, i32, i32, bool)> {
        let mut list: Vec<(i32, i32, i32, i32, bool)> = Vec::new();
        unsafe {
            EnumDisplayMonitors(
                std::ptr::null_mut(),
                std::ptr::null(),
                Some(enum_monitors_proc),
                &mut list as *mut _ as LPARAM,
            );
        }
        list.sort_by_key(|m| (m.0, m.1));
        list
    }

    fn tiles_in_parent(pl: i32, pt: i32, pw: i32, ph: i32) -> Vec<super::MonitorTile> {
        let pr = pl + pw;
        let pb = pt + ph;
        let mut tiles: Vec<super::MonitorTile> = enum_monitors()
            .into_iter()
            .filter_map(|(x, y, w, h, primary)| {
                let left = x.max(pl);
                let top = y.max(pt);
                let right = (x + w).min(pr);
                let bottom = (y + h).min(pb);
                if right - left <= 0 || bottom - top <= 0 {
                    return None;
                }
                Some(super::MonitorTile {
                    x: left - pl,
                    y: top - pt,
                    w: right - left,
                    h: bottom - top,
                    primary,
                })
            })
            .collect();
        if tiles.is_empty() {
            tiles.push(super::MonitorTile {
                x: 0,
                y: 0,
                w: pw.max(1),
                h: ph.max(1),
                primary: true,
            });
        } else if !tiles.iter().any(|t| t.primary) {
            tiles[0].primary = true;
        }
        tiles
    }

    pub fn layout_for_child(hwnd_raw: isize) -> (i32, i32, Vec<super::MonitorTile>) {
        unsafe {
            let child = hwnd_raw as HWND;
            let parent = {
                let p = GetParent(child);
                if p.is_null() {
                    child
                } else {
                    p
                }
            };
            let (pl, pt, mut pw, mut ph) = rect_of(parent);
            if pw <= 0 || ph <= 0 {
                pw = GetSystemMetrics(SM_CXSCREEN);
                ph = GetSystemMetrics(SM_CYSCREEN);
            }
            let (_cx, _cy, cw, ch) = rect_of(child);
            if cw != pw || ch != ph {
                SetWindowPos(
                    child,
                    std::ptr::null_mut(),
                    0,
                    0,
                    pw,
                    ph,
                    SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_FRAMECHANGED | SWP_NOZORDER,
                );
            }
            let _ = (pl, pt);
            (pw, ph, tiles_in_parent(pl, pt, pw, ph))
        }
    }

    struct EnumData {
        defview_parent: HWND,
        defview: HWND,
        worker: HWND,
    }

    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> i32 {
        let data = &mut *(lparam as *mut EnumData);
        let class_def = wide("SHELLDLL_DefView");
        let def = FindWindowExW(
            hwnd,
            std::ptr::null_mut(),
            class_def.as_ptr(),
            std::ptr::null(),
        );
        if !def.is_null() {
            data.defview_parent = hwnd;
            data.defview = def;
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

    fn spawn_workerw(progman: HWND) {
        unsafe {
            let mut result: usize = 0;
            SendMessageTimeoutW(progman, 0x052C, 0, 0, SMTO_NORMAL, 1000, &mut result);
            SendMessageTimeoutW(progman, 0x052C, 0xD, 0, SMTO_NORMAL, 1000, &mut result);
            SendMessageTimeoutW(progman, 0x052C, 0xD, 1, SMTO_NORMAL, 1000, &mut result);
        }
    }

    fn find_progman_child(progman: HWND, class: &str) -> HWND {
        unsafe {
            let cls = wide(class);
            FindWindowExW(
                progman,
                std::ptr::null_mut(),
                cls.as_ptr(),
                std::ptr::null(),
            )
        }
    }

    fn is_raised_desktop(progman: HWND) -> bool {
        unsafe {
            let ex = GetWindowLongPtrW(progman, GWL_EXSTYLE) as u32;
            ex & WS_EX_NOREDIRECTIONBITMAP != 0
        }
    }

    fn prepare_styles(child: HWND, layered: bool) {
        unsafe {
            let mut ex = GetWindowLongPtrW(child, GWL_EXSTYLE) as u32;
            ex &= !WS_EX_APPWINDOW;
            ex |= WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE;
            if layered {
                ex |= WS_EX_LAYERED;
            }
            SetWindowLongPtrW(child, GWL_EXSTYLE, ex as isize);
            if layered {
                SetLayeredWindowAttributes(child, 0, 255, LWA_ALPHA);
            }
        }
    }

    pub fn attach_to_desktop(hwnd_raw: isize) -> Result<(i32, i32), String> {
        unsafe {
            let progman_class = wide("Progman");
            let progman = FindWindowW(progman_class.as_ptr(), std::ptr::null());
            if progman.is_null() {
                return Err("未找到 Progman 窗口".into());
            }

            spawn_workerw(progman);

            let mut data = EnumData {
                defview_parent: std::ptr::null_mut(),
                defview: std::ptr::null_mut(),
                worker: std::ptr::null_mut(),
            };
            EnumWindows(Some(enum_windows_proc), &mut data as *mut _ as LPARAM);

            if data.defview.is_null() {
                data.defview = find_progman_child(progman, "SHELLDLL_DefView");
                if !data.defview.is_null() {
                    data.defview_parent = progman;
                }
            }
            if data.worker.is_null() {
                data.worker = find_progman_child(progman, "WorkerW");
            }

            let child = hwnd_raw as HWND;
            let raised = is_raised_desktop(progman);
            prepare_styles(child, raised);

            let parent = if raised {
                progman
            } else if !data.worker.is_null() {
                data.worker
            } else {
                progman
            };

            if SetParent(child, parent).is_null() && GetParent(child) != parent {
                return Err("SetParent 失败".into());
            }

            let (px, py, mut w, mut h) = rect_of(parent);
            if w <= 0 || h <= 0 {
                w = GetSystemMetrics(SM_CXSCREEN);
                h = GetSystemMetrics(SM_CYSCREEN);
            }
            let _ = (px, py);

            let insert_after = if !data.defview.is_null() && raised {
                data.defview
            } else {
                HWND_BOTTOM
            };

            SetWindowPos(
                child,
                insert_after,
                0,
                0,
                w,
                h,
                SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_FRAMECHANGED,
            );

            if raised && !data.worker.is_null() {
                SetWindowPos(
                    data.worker,
                    child,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                );
            }

            ShowWindow(child, SW_SHOW);
            RedrawWindow(
                child,
                std::ptr::null(),
                std::ptr::null_mut(),
                RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN,
            );

            let (px, py, pw, ph) = rect_of(parent);
            let (cx, cy, cw, ch) = rect_of(child);
            let visible = IsWindowVisible(child) != 0;
            let tiles = tiles_in_parent(px, py, pw, ph);
            eprintln!(
                "[wallpaper] attached hwnd={child:?} class={} parent={parent:?} parentClass={} parentRect={px},{py} {pw}x{ph} childRect={cx},{cy} {cw}x{ch} visible={visible} raised={raised} defview={:?} worker={:?} tiles={tiles:?}",
                class_name(child),
                class_name(parent),
                data.defview,
                data.worker
            );
            Ok((w, h))
        }
    }
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
                    eprintln!("[wallpaper] layout tiles={tiles:?}");
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
