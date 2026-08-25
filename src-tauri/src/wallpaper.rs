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
    use windows::core::{BOOL, PCWSTR};
    use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, RECT, WPARAM};
    use windows::Win32::Graphics::Gdi::{
        EnumDisplayMonitors, GetMonitorInfoW, RedrawWindow, HDC, HMONITOR, MONITORINFO,
        RDW_ALLCHILDREN, RDW_INVALIDATE, RDW_UPDATENOW,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, FindWindowExW, FindWindowW, GetClassNameW, GetParent, GetSystemMetrics,
        GetWindowLongPtrW, GetWindowRect, IsWindowVisible, SendMessageTimeoutW,
        SetLayeredWindowAttributes, SetParent, SetWindowLongPtrW, SetWindowPos, ShowWindow,
        GWL_EXSTYLE, HWND_BOTTOM, LWA_ALPHA, MONITORINFOF_PRIMARY, SMTO_NORMAL, SM_CXSCREEN,
        SM_CYSCREEN, SWP_FRAMECHANGED, SWP_HIDEWINDOW, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
        SWP_NOZORDER, SWP_SHOWWINDOW, SW_HIDE, SW_SHOW, WS_EX_APPWINDOW, WS_EX_LAYERED,
        WS_EX_NOACTIVATE, WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW,
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
            let n = GetClassNameW(hwnd, &mut buf);
            if n <= 0 {
                return String::new();
            }
            String::from_utf16_lossy(&buf[..n as usize])
        }
    }

    fn rect_of(hwnd: HWND) -> (i32, i32, i32, i32) {
        unsafe {
            let mut r = RECT::default();
            let _ = GetWindowRect(hwnd, &mut r);
            (r.left, r.top, r.right - r.left, r.bottom - r.top)
        }
    }

    unsafe extern "system" fn enum_monitors_proc(
        hmon: HMONITOR,
        _hdc: HDC,
        _lprc: *mut RECT,
        lparam: LPARAM,
    ) -> BOOL {
        let list = &mut *(lparam.0 as *mut Vec<(i32, i32, i32, i32, bool)>);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            rcMonitor: RECT::default(),
            rcWork: RECT::default(),
            dwFlags: 0,
        };
        if GetMonitorInfoW(hmon, &mut info).as_bool() {
            let r = info.rcMonitor;
            list.push((
                r.left,
                r.top,
                r.right - r.left,
                r.bottom - r.top,
                info.dwFlags & MONITORINFOF_PRIMARY != 0,
            ));
        }
        BOOL(1)
    }

    fn enum_monitors() -> Vec<(i32, i32, i32, i32, bool)> {
        let mut list: Vec<(i32, i32, i32, i32, bool)> = Vec::new();
        unsafe {
            let _ = EnumDisplayMonitors(
                None,
                None,
                Some(enum_monitors_proc),
                LPARAM(&mut list as *mut _ as isize),
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
            let child = HWND(hwnd_raw as *mut _);
            let parent = GetParent(child).unwrap_or(child);
            let (pl, pt, mut pw, mut ph) = rect_of(parent);
            if pw <= 0 || ph <= 0 {
                pw = GetSystemMetrics(SM_CXSCREEN);
                ph = GetSystemMetrics(SM_CYSCREEN);
            }
            let (_cx, _cy, cw, ch) = rect_of(child);
            if cw != pw || ch != ph {
                let _ = SetWindowPos(
                    child,
                    None,
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

    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let data = &mut *(lparam.0 as *mut EnumData);
        let class_def = wide("SHELLDLL_DefView");
        if let Ok(def) = FindWindowExW(Some(hwnd), None, PCWSTR(class_def.as_ptr()), PCWSTR::null())
        {
            data.defview_parent = hwnd;
            data.defview = def;
            let class_worker = wide("WorkerW");
            if let Ok(next) =
                FindWindowExW(None, Some(hwnd), PCWSTR(class_worker.as_ptr()), PCWSTR::null())
            {
                data.worker = next;
            }
        }
        BOOL(1)
    }

    fn spawn_workerw(progman: HWND) {
        unsafe {
            let mut result: usize = 0;
            let _ = SendMessageTimeoutW(
                progman,
                0x052C,
                WPARAM(0),
                LPARAM(0),
                SMTO_NORMAL,
                1000,
                Some(&mut result),
            );
            let _ = SendMessageTimeoutW(
                progman,
                0x052C,
                WPARAM(0xD),
                LPARAM(0),
                SMTO_NORMAL,
                1000,
                Some(&mut result),
            );
            let _ = SendMessageTimeoutW(
                progman,
                0x052C,
                WPARAM(0xD),
                LPARAM(1),
                SMTO_NORMAL,
                1000,
                Some(&mut result),
            );
        }
    }

    fn find_progman_child(progman: HWND, class: &str) -> HWND {
        unsafe {
            let cls = wide(class);
            FindWindowExW(Some(progman), None, PCWSTR(cls.as_ptr()), PCWSTR::null())
                .unwrap_or_default()
        }
    }

    fn is_raised_desktop(progman: HWND) -> bool {
        unsafe {
            let ex = GetWindowLongPtrW(progman, GWL_EXSTYLE) as u32;
            ex & WS_EX_NOREDIRECTIONBITMAP.0 != 0
        }
    }

    fn prepare_styles(child: HWND, layered: bool) {
        unsafe {
            let mut ex = GetWindowLongPtrW(child, GWL_EXSTYLE) as u32;
            ex &= !WS_EX_APPWINDOW.0;
            ex |= WS_EX_TOOLWINDOW.0 | WS_EX_NOACTIVATE.0;
            if layered {
                ex |= WS_EX_LAYERED.0;
            }
            SetWindowLongPtrW(child, GWL_EXSTYLE, ex as isize);
            if layered {
                let _ = SetLayeredWindowAttributes(child, COLORREF(0), 255, LWA_ALPHA);
            }
        }
    }

    pub fn attach_to_desktop(hwnd_raw: isize) -> Result<(i32, i32), String> {
        unsafe {
            let progman_class = wide("Progman");
            let progman = FindWindowW(PCWSTR(progman_class.as_ptr()), PCWSTR::null())
                .map_err(|_| "未找到 Progman 窗口".to_string())?;

            spawn_workerw(progman);

            let mut data = EnumData {
                defview_parent: HWND::default(),
                defview: HWND::default(),
                worker: HWND::default(),
            };
            let _ = EnumWindows(Some(enum_windows_proc), LPARAM(&mut data as *mut _ as isize));

            if data.defview.0.is_null() {
                data.defview = find_progman_child(progman, "SHELLDLL_DefView");
                if !data.defview.0.is_null() {
                    data.defview_parent = progman;
                }
            }
            if data.worker.0.is_null() {
                data.worker = find_progman_child(progman, "WorkerW");
            }

            let child = HWND(hwnd_raw as *mut _);
            let raised = is_raised_desktop(progman);
            prepare_styles(child, raised);

            let parent = if raised {
                progman
            } else if !data.worker.0.is_null() {
                data.worker
            } else {
                progman
            };

            let set_parent_err = SetParent(child, Some(parent)).is_err();
            if set_parent_err && GetParent(child).ok() != Some(parent) {
                return Err("SetParent 失败".into());
            }

            let (px, py, mut w, mut h) = rect_of(parent);
            if w <= 0 || h <= 0 {
                w = GetSystemMetrics(SM_CXSCREEN);
                h = GetSystemMetrics(SM_CYSCREEN);
            }
            let _ = (px, py);

            let insert_after = if !data.defview.0.is_null() && raised {
                Some(data.defview)
            } else {
                Some(HWND_BOTTOM)
            };

            let _ = SetWindowPos(
                child,
                insert_after,
                0,
                0,
                w,
                h,
                SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_FRAMECHANGED,
            );

            if raised && !data.worker.0.is_null() {
                let _ = SetWindowPos(
                    data.worker,
                    Some(child),
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                );
            }

            let _ = ShowWindow(child, SW_SHOW);
            let _ = RedrawWindow(
                Some(child),
                None,
                None,
                RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN,
            );

            let (px, py, pw, ph) = rect_of(parent);
            let (cx, cy, cw, ch) = rect_of(child);
            let visible = IsWindowVisible(child).as_bool();
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

    pub fn detach_from_desktop(hwnd_raw: isize) {
        unsafe {
            let child = HWND(hwnd_raw as *mut _);
            let _ = ShowWindow(child, SW_HIDE);
            let _ = SetParent(child, None);
            let _ = SetWindowPos(
                child,
                None,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_HIDEWINDOW
                    | SWP_FRAMECHANGED,
            );

            let progman_class = wide("Progman");
            let Ok(progman) = FindWindowW(PCWSTR(progman_class.as_ptr()), PCWSTR::null()) else {
                return;
            };

            let mut data = EnumData {
                defview_parent: HWND::default(),
                defview: HWND::default(),
                worker: HWND::default(),
            };
            let _ = EnumWindows(Some(enum_windows_proc), LPARAM(&mut data as *mut _ as isize));

            if data.defview.0.is_null() {
                data.defview = find_progman_child(progman, "SHELLDLL_DefView");
            }
            if data.worker.0.is_null() {
                data.worker = find_progman_child(progman, "WorkerW");
            }

            if is_raised_desktop(progman)
                && !data.worker.0.is_null()
                && !data.defview.0.is_null()
            {
                let _ = SetWindowPos(
                    data.worker,
                    Some(data.defview),
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                );
            }

            let refresh = if !data.defview.0.is_null() {
                data.defview
            } else {
                progman
            };
            let _ = RedrawWindow(
                Some(refresh),
                None,
                None,
                RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN,
            );
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
    eprintln!("[wallpaper] cleanup detached from desktop");
}

/// After sleep/hibernate, WorkerW may be recreated and video elements may stay paused.
/// Force re-attach and push play when the engine should still be playing.
pub fn on_system_resume(app: &AppHandle, should_play: bool, state: &EngineState) {
    eprintln!(
        "[wallpaper] system resume should_play={should_play} media={:?}",
        state.media_id
    );
    ATTACHED.store(false, Ordering::SeqCst);
    if let Err(e) = attach_existing(app) {
        eprintln!("[wallpaper] resume reattach failed: {e}");
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
