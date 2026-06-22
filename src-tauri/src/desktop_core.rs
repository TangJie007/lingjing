//! Windows 桌面层嵌入（参考 Lively Wallpaper / 飞火动态壁纸）
//! 默认：ClassicWorkerW（MPV 在壁纸 WorkerW 内，不挡图标）
//! 共存：启动时临时隐藏竞争动态壁纸（飞火 / Lively / MPV 系），退出时恢复
//! 回退：ShellHostLayered / RaisedProgman

#![cfg(target_os = "windows")]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{EnumDisplayMonitors, HDC, HMONITOR};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumChildWindows, EnumWindows, FindWindowExW, FindWindowW, GetClassNameW, GetParent,
    GetWindowLongPtrW, GetWindowRect, GetWindowThreadProcessId, IsWindow, IsWindowVisible,
    MoveWindow,
    SetParent, SetWindowLongPtrW, SetWindowPos, ShowWindow, SendMessageW, GWL_EXSTYLE, GWL_STYLE,
    HWND_BOTTOM, HWND_TOP, SW_HIDE, SW_SHOW, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SWP_SHOWWINDOW, WS_CHILD, WS_EX_NOREDIRECTIONBITMAP, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_VISIBLE,
};

/// 已隐藏的竞争动态壁纸窗口（退出时恢复；存 isize 以满足 Sync）
static HIDDEN_COMPETING_PLAYERS: OnceLock<Mutex<Vec<isize>>> = OnceLock::new();

fn hidden_competing_players() -> &'static Mutex<Vec<isize>> {
    HIDDEN_COMPETING_PLAYERS.get_or_init(|| Mutex::new(Vec::new()))
}

fn hwnd_to_isize(hwnd: HWND) -> isize {
    hwnd.0 as isize
}

fn isize_to_hwnd(v: isize) -> HWND {
    HWND(v as *mut _)
}

/// 桌面 shell 层信息（SetupDesktopLayer 结果）
#[derive(Debug, Clone, Copy)]
pub struct DesktopLayer {
    pub progman: Option<HWND>,
    /// 含 SHELLDLL_DefView 的窗口（Progman 或 WorkerW）
    pub shell_host: HWND,
    pub shell_view: HWND,
    pub worker_w: Option<HWND>,
    pub is_raised: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachMode {
    /// Win11 24H2+：Progman + layered child
    RaisedProgman,
    /// Win10/Win11 回退：壁纸 WorkerW（层级低于 shell_host 子窗口方案）
    ClassicWorkerW,
    /// 飞火策略：MPV 挂 shell_host，Z-order 紧贴 DefView 下方（壁纸层最高位置，仍在图标之下）
    ShellHostLayered,
}

static WORKERW_SPAWNED: AtomicBool = AtomicBool::new(false);
static CACHED_LAYER: OnceLock<Mutex<Option<CachedDesktopLayer>>> = OnceLock::new();

#[derive(Debug, Clone, Copy)]
struct CachedDesktopLayer {
    progman: Option<isize>,
    shell_host: isize,
    shell_view: isize,
    worker_w: Option<isize>,
    is_raised: bool,
}

impl From<DesktopLayer> for CachedDesktopLayer {
    fn from(layer: DesktopLayer) -> Self {
        Self {
            progman: layer.progman.map(|h| h.0 as isize),
            shell_host: layer.shell_host.0 as isize,
            shell_view: layer.shell_view.0 as isize,
            worker_w: layer.worker_w.map(|h| h.0 as isize),
            is_raised: layer.is_raised,
        }
    }
}

impl CachedDesktopLayer {
    fn to_layer(self) -> DesktopLayer {
        DesktopLayer {
            progman: self.progman.map(|v| HWND(v as *mut _)),
            shell_host: HWND(self.shell_host as *mut _),
            shell_view: HWND(self.shell_view as *mut _),
            worker_w: self.worker_w.map(|v| HWND(v as *mut _)),
            is_raised: self.is_raised,
        }
    }
}

fn cached_layer_slot() -> &'static Mutex<Option<CachedDesktopLayer>> {
    CACHED_LAYER.get_or_init(|| Mutex::new(None))
}

fn layer_still_valid(layer: &CachedDesktopLayer) -> bool {
    unsafe {
        IsWindow(Some(HWND(layer.shell_view as *mut _))).as_bool()
            && layer
                .worker_w
                .map(|w| IsWindow(Some(HWND(w as *mut _))).as_bool())
                .unwrap_or(true)
    }
}

fn store_cached_layer(layer: DesktopLayer) {
    if let Ok(mut guard) = cached_layer_slot().lock() {
        *guard = Some(layer.into());
    }
}

/// 获取已缓存的桌面层；失效时重新探测（不重复发送 0x052C）
pub fn get_cached_desktop_layer() -> Result<DesktopLayer, String> {
    if let Ok(guard) = cached_layer_slot().lock() {
        if let Some(layer) = guard.as_ref() {
            if layer_still_valid(layer) {
                return Ok(layer.to_layer());
            }
        }
    }
    let layer = detect_desktop_layer()?;
    store_cached_layer(layer);
    Ok(layer)
}

/// 探测并准备桌面层（发送 0x052C 创建 WorkerW，仅首次）
pub fn setup_desktop_layer() -> Result<DesktopLayer, String> {
    if let Ok(guard) = cached_layer_slot().lock() {
        if let Some(layer) = guard.as_ref() {
            if layer_still_valid(layer) {
                return Ok(layer.to_layer());
            }
        }
    }

    let first_spawn = !WORKERW_SPAWNED.swap(true, Ordering::SeqCst);
    if first_spawn {
        spawn_workerw();
    }

    let max_attempts = if first_spawn { 8 } else { 3 };
    let sleep_ms = if first_spawn { 150 } else { 40 };

    for attempt in 0..max_attempts {
        let layer = detect_desktop_layer()?;
        if layer.worker_w.is_some() || attempt == max_attempts - 1 {
            if layer.worker_w.is_none() && first_spawn {
                log_workerw_probe(layer.progman, layer.shell_host);
            }
            store_cached_layer(layer);
            return Ok(layer);
        }
        std::thread::sleep(std::time::Duration::from_millis(sleep_ms));
    }
    let layer = detect_desktop_layer()?;
    store_cached_layer(layer);
    Ok(layer)
}

/// 仅探测桌面层（不重复发送 0x052C，供 Z-order 维持线程使用）
pub fn detect_desktop_layer() -> Result<DesktopLayer, String> {
    let progman = find_progman();
    let (shell_host, shell_view) =
        detect_shell_layer().ok_or("找不到 SHELLDLL_DefView 桌面图标层")?;

    let is_raised = progman
        .map(has_extended_style_noredirectionbitmap)
        .unwrap_or(false);

    let worker_w = if is_raised {
        progman.and_then(|p| find_child_workerw(p))
    } else {
        // Win11 常见：图标在顶层 WorkerW，壁纸在 Progman 子 WorkerW
        find_progman_wallpaper_workerw()
            .or_else(find_sibling_workerw)
            .or_else(find_classic_workerw)
    };

    log::info!(
        "DesktopLayer: progman={:?}, shell_host={:?}, shell_view={:?}, worker_w={:?}, raised={}, virtual={}x{}",
        progman,
        shell_host,
        shell_view,
        worker_w,
        is_raised,
        get_virtual_screen_rect().w,
        get_virtual_screen_rect().h,
    );

    Ok(DesktopLayer {
        progman,
        shell_host,
        shell_view,
        worker_w,
        is_raised,
    })
}

/// MPV --wid 启动后，将播放窗口压到桌面图标层（DefView）正下方
pub fn position_player_below_icons(hwnd: HWND, layer: &DesktopLayer) -> Result<(), String> {
    let screen = get_virtual_screen_rect();
    position_player_rect(hwnd, layer, screen.x, screen.y, screen.w, screen.h)
}

/// 将 MPV 窗口定位到指定显示器区域（多屏壁纸）
pub fn position_player_on_monitor(
    hwnd: HWND,
    layer: &DesktopLayer,
    monitor: MonitorRect,
) -> Result<(), String> {
    log::info!(
        "position_player_on_monitor: hwnd={:?}, monitor={}x{}@({},{})",
        hwnd,
        monitor.w,
        monitor.h,
        monitor.x,
        monitor.y
    );
    position_player_rect(hwnd, layer, monitor.x, monitor.y, monitor.w, monitor.h)
}

fn position_player_rect(
    hwnd: HWND,
    layer: &DesktopLayer,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) -> Result<(), String> {
    unsafe {
        apply_wallpaper_child_styles(hwnd)?;
        // 紧贴 DefView 下方（比 HWND_BOTTOM 更稳定，避免鼠标悬停时图标闪烁）
        SetWindowPos(
            hwnd,
            Some(layer.shell_view),
            x,
            y,
            w,
            h,
            SWP_NOACTIVATE | SWP_SHOWWINDOW,
        )
        .map_err(|e| format!("SetWindowPos(DefView 下方) 失败: {:?}", e))?;
        let _ = ShowWindow(hwnd, SW_SHOW);
    }
    Ok(())
}

/// 将 MPV 定位到壁纸 WorkerW 内的指定显示器区域（ClassicWorkerW 多屏）
pub fn position_player_in_wallpaper_worker(
    hwnd: HWND,
    monitor: MonitorRect,
) -> Result<(), String> {
    log::info!(
        "position_player_in_wallpaper_worker: hwnd={:?}, monitor={}x{}@({},{})",
        hwnd,
        monitor.w,
        monitor.h,
        monitor.x,
        monitor.y
    );
    unsafe {
        apply_wallpaper_child_styles(hwnd)?;
        MoveWindow(hwnd, monitor.x, monitor.y, monitor.w, monitor.h, true)
            .map_err(|e| format!("MoveWindow(壁纸 WorkerW) 失败: {:?}", e))?;
        SetWindowPos(
            hwnd,
            Some(HWND_BOTTOM),
            monitor.x,
            monitor.y,
            monitor.w,
            monitor.h,
            SWP_NOACTIVATE | SWP_SHOWWINDOW,
        )
        .map_err(|e| format!("SetWindowPos(壁纸 WorkerW 子窗口) 失败: {:?}", e))?;
        let _ = ShowWindow(hwnd, SW_SHOW);
    }
    Ok(())
}

/// 维持壁纸层在图标层下方
pub fn ensure_desktop_zorder(layer: &DesktopLayer) -> Result<(), String> {
    ensure_desktop_zorder_for_mode(layer, resolve_attach_mode(layer))
}

/// 看门狗空闲时刷新 Z-order（不触发完整 reattach）
pub fn refresh_desktop_zorder(layer: &DesktopLayer) -> Result<(), String> {
    match resolve_attach_mode(layer) {
        AttachMode::ClassicWorkerW => refresh_icon_zorder(layer),
        AttachMode::ShellHostLayered | AttachMode::RaisedProgman => {
            raise_icon_children(layer)?;
            lower_shell_host_video_below_icons(layer)?;
            Ok(())
        }
    }
}

fn ensure_desktop_zorder_for_mode(layer: &DesktopLayer, mode: AttachMode) -> Result<(), String> {
    match mode {
        AttachMode::ClassicWorkerW => ensure_desktop_zorder_inner(layer, true),
        AttachMode::ShellHostLayered | AttachMode::RaisedProgman => {
            raise_icon_children(layer)?;
            lower_shell_host_video_below_icons(layer)?;
            Ok(())
        }
    }
}

/// 仅刷新图标层 Z-order（ClassicWorkerW 专用，不移动壁纸 WorkerW）
pub fn refresh_icon_zorder(layer: &DesktopLayer) -> Result<(), String> {
    ensure_desktop_zorder_inner(layer, false)
}

fn ensure_desktop_zorder_inner(layer: &DesktopLayer, move_wallpaper: bool) -> Result<(), String> {
    let screen = get_virtual_screen_rect();
    unsafe {
        if move_wallpaper {
            if let Some(ww) = layer.worker_w {
                if is_valid_wallpaper_workerw(ww, layer.shell_host) {
                    let _ = ShowWindow(ww, SW_SHOW);
                    SetWindowPos(
                        ww,
                        Some(layer.shell_host),
                        screen.x,
                        screen.y,
                        screen.w,
                        screen.h,
                        SWP_NOACTIVATE | SWP_SHOWWINDOW,
                    )
                    .map_err(|e| format!("SetWindowPos(壁纸 WorkerW) 失败: {:?}", e))?;
                }
            }
        }
        raise_icon_children(layer)?;
        if layer.worker_w.is_none() {
            lower_shell_host_video_below_icons(layer)?;
        }
    }
    log::debug!(
        "桌面 Z-order 已刷新: shell_host={:?}, worker_w={:?}",
        layer.shell_host,
        layer.worker_w
    );
    Ok(())
}

fn lower_shell_host_video_below_icons(layer: &DesktopLayer) -> Result<(), String> {
    unsafe {
        struct VideoCtx {
            shell_view: HWND,
        }
        unsafe extern "system" fn enum_video(hwnd: HWND, lparam: LPARAM) -> windows_core::BOOL {
            let ctx = &*(lparam.0 as *const VideoCtx);
            let mut buf = [0u16; 64];
            if GetClassNameW(hwnd, &mut buf) == 0 {
                return windows_core::BOOL(1);
            }
            let class = String::from_utf16_lossy(
                &buf[..buf.iter().position(|&c| c == 0).unwrap_or(buf.len())],
            );
            if class != "mpv" && class != "glfw" {
                return windows_core::BOOL(1);
            }
            let _ = SetWindowPos(
                hwnd,
                Some(ctx.shell_view),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
            windows_core::BOOL(1)
        }
        let ctx = VideoCtx {
            shell_view: layer.shell_view,
        };
        let _ = EnumChildWindows(
            Some(layer.shell_host),
            Some(enum_video),
            LPARAM(&ctx as *const _ as isize),
        );
    }
    Ok(())
}

fn raise_icon_children(layer: &DesktopLayer) -> Result<(), String> {
    unsafe {
        let _ = ShowWindow(layer.shell_view, SW_SHOW);

        SetWindowPos(
            layer.shell_view,
            Some(HWND_TOP),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
        )
        .map_err(|e| format!("SetWindowPos(DefView TOP) 失败: {:?}", e))?;

        if let Some(list_view) = find_child_syslistview(layer.shell_host)
            .or_else(|| find_child_syslistview(layer.shell_view))
        {
            let _ = ShowWindow(list_view, SW_SHOW);
            SetWindowPos(
                list_view,
                Some(HWND_TOP),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
            )
            .map_err(|e| format!("SetWindowPos(SysListView32 TOP) 失败: {:?}", e))?;
        }
    }
    Ok(())
}


fn find_child_syslistview(parent: HWND) -> Option<HWND> {
    unsafe {
        match FindWindowExW(Some(parent), None, windows::core::w!("SysListView32"), None) {
            Ok(v) if v != HWND::default() => Some(v),
            _ => None,
        }
    }
}

pub fn is_window_visible(hwnd: HWND) -> bool {
    unsafe { IsWindowVisible(hwnd).as_bool() }
}

/// 其他动态壁纸播放窗口类名（飞火 / Lively / MPV 系）
const COMPETING_WALLPAPER_PLAYER_CLASSES: &[&str] =
    &["mpv", "glfw", "QWidget", "Chrome_WidgetWin_1", "CefBrowserWindow"];

/// 启动壁纸前隐藏占用桌面层的竞争动态壁纸窗口（飞火 / Lively / MPV 系）；退出时恢复
pub fn suppress_competing_wallpaper_players(our_pids: &[u32]) -> Result<(), String> {
    suppress_desktop_layer_interference(our_pids)
}

/// 停止壁纸时恢复此前隐藏的窗口
pub fn restore_competing_wallpaper_players() -> Result<(), String> {
    restore_desktop_layer_interference()
}

pub fn suppress_desktop_layer_interference(our_pids: &[u32]) -> Result<(), String> {
    let layer = detect_desktop_layer()?;
    let screen = get_virtual_screen_rect();
    let screen_area = screen.w as i64 * screen.h as i64;
    let min_player_area = screen_area / 4;
    let mut newly_hidden = Vec::new();
    let hosts = collect_wallpaper_search_roots(&layer);
    for host in hosts {
        collect_interference_windows_under(
            host,
            our_pids,
            min_player_area,
            &mut newly_hidden,
        );
    }
    if newly_hidden.is_empty() {
        return Ok(());
    }
    unsafe {
        for hwnd in &newly_hidden {
            let _ = ShowWindow(*hwnd, SW_HIDE);
            log::info!(
                "已临时隐藏桌面层干扰窗口: {:?} class={} pid={}",
                hwnd,
                window_class_name(*hwnd),
                window_process_id(*hwnd)
            );
        }
    }
    let mut guard = hidden_competing_players()
        .lock()
        .map_err(|e| format!("锁定竞争窗口列表失败: {}", e))?;
    for hwnd in newly_hidden {
        let id = hwnd_to_isize(hwnd);
        if !guard.contains(&id) {
            guard.push(id);
        }
    }
    Ok(())
}

pub fn restore_desktop_layer_interference() -> Result<(), String> {
    let mut guard = hidden_competing_players()
        .lock()
        .map_err(|e| format!("锁定竞争窗口列表失败: {}", e))?;
    unsafe {
        for id in guard.drain(..) {
            let hwnd = isize_to_hwnd(id);
            let _ = ShowWindow(hwnd, SW_SHOW);
            log::info!(
                "已恢复桌面层干扰窗口: {:?} class={}",
                hwnd,
                window_class_name(hwnd)
            );
        }
    }
    Ok(())
}

fn collect_interference_windows_under(
    root: HWND,
    our_pids: &[u32],
    min_player_area: i64,
    hidden: &mut Vec<HWND>,
) {
    unsafe {
        struct Ctx {
            our_pids: Vec<u32>,
            min_player_area: i64,
            hidden: *mut Vec<HWND>,
        }
        unsafe extern "system" fn cb(hwnd: HWND, lparam: LPARAM) -> windows_core::BOOL {
            let ctx = &mut *(lparam.0 as *mut Ctx);
            let hidden = unsafe { &mut *ctx.hidden };
            if is_shell_ui_window(hwnd) {
                return windows_core::BOOL(1);
            }
            if hidden.contains(&hwnd) {
                return windows_core::BOOL(1);
            }
            if should_suppress_desktop_window(hwnd, &ctx.our_pids, ctx.min_player_area) {
                hidden.push(hwnd);
            }
            let sub = Ctx {
                our_pids: ctx.our_pids.clone(),
                min_player_area: ctx.min_player_area,
                hidden: ctx.hidden,
            };
            let _ = EnumChildWindows(Some(hwnd), Some(cb), LPARAM(&sub as *const _ as isize));
            windows_core::BOOL(1)
        }
        let mut ctx = Ctx {
            our_pids: our_pids.to_vec(),
            min_player_area,
            hidden,
        };
        let _ = EnumChildWindows(Some(root), Some(cb), LPARAM(&mut ctx as *mut _ as isize));
    }
}

fn should_suppress_desktop_window(
    hwnd: HWND,
    our_pids: &[u32],
    min_player_area: i64,
) -> bool {
    if !is_window_visible(hwnd) {
        return false;
    }
    let pid = window_process_id(hwnd);
    if pid == 0 || our_pids.contains(&pid) {
        return false;
    }
    let class = window_class_name(hwnd);
    let area = get_window_rect(hwnd)
        .map(|r| (r.right - r.left) as i64 * (r.bottom - r.top) as i64)
        .unwrap_or(0);
    if area < min_player_area {
        return false;
    }
    COMPETING_WALLPAPER_PLAYER_CLASSES.contains(&class.as_str())
}

fn collect_wallpaper_search_roots(layer: &DesktopLayer) -> Vec<HWND> {
    let mut roots = vec![layer.shell_host];
    if let Some(ww) = layer.worker_w {
        roots.push(ww);
    }
    if let Some(progman) = layer.progman {
        roots.push(progman);
        unsafe {
            struct Ctx {
                workers: Vec<HWND>,
            }
            unsafe extern "system" fn cb(hwnd: HWND, lparam: LPARAM) -> windows_core::BOOL {
                let ctx = &mut *(lparam.0 as *mut Ctx);
                let mut buf = [0u16; 64];
                if GetClassNameW(hwnd, &mut buf) == 0 {
                    return windows_core::BOOL(1);
                }
                let class = String::from_utf16_lossy(
                    &buf[..buf.iter().position(|&c| c == 0).unwrap_or(buf.len())],
                );
                if class == "WorkerW" && find_child_defview(hwnd).is_none() {
                    ctx.workers.push(hwnd);
                }
                windows_core::BOOL(1)
            }
            let mut ctx = Ctx {
                workers: Vec::new(),
            };
            let _ = EnumChildWindows(Some(progman), Some(cb), LPARAM(&mut ctx as *mut _ as isize));
            roots.extend(ctx.workers);
        }
    }
    roots
}

fn is_shell_ui_window(hwnd: HWND) -> bool {
    let class = window_class_name(hwnd);
    matches!(
        class.as_str(),
        "SHELLDLL_DefView" | "SysListView32" | "WorkerW" | "Progman"
    )
}

fn window_class_name(hwnd: HWND) -> String {
    unsafe {
        let mut buf = [0u16; 256];
        if GetClassNameW(hwnd, &mut buf) == 0 {
            return String::new();
        }
        String::from_utf16_lossy(&buf[..buf.iter().position(|&c| c == 0).unwrap_or(buf.len())])
    }
}

fn window_process_id(hwnd: HWND) -> u32 {
    unsafe {
        let mut pid: u32 = 0;
        let _ = GetWindowThreadProcessId(hwnd, Some(&mut pid));
        pid
    }
}

/// 壁纸层异常需恢复
pub fn desktop_wallpaper_needs_recovery(layer: &DesktopLayer) -> bool {
    match resolve_attach_mode(layer) {
        AttachMode::ShellHostLayered | AttachMode::RaisedProgman => false,
        AttachMode::ClassicWorkerW => layer.worker_w.is_none()
            || layer
                .worker_w
                .is_some_and(|ww| !is_window_visible(ww)),
    }
}

unsafe fn apply_wallpaper_child_styles(hwnd: HWND) -> Result<(), String> {
    let mut ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
    ex |= WS_EX_NOACTIVATE.0;
    ex &= !WS_EX_TOOLWINDOW.0;
    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex as isize);
    Ok(())
}

/// 获取 MPV 嵌入目标窗口（--wid）
pub fn wid_target(layer: &DesktopLayer, mode: AttachMode) -> Option<HWND> {
    match mode {
        AttachMode::ClassicWorkerW => layer.worker_w,
        AttachMode::ShellHostLayered => Some(layer.shell_host),
        AttachMode::RaisedProgman => layer.progman,
    }
}

/// 是否用 mpv --wid 直接渲染（避免 SetParent 后 GPU 黑屏）
pub fn should_use_wid(layer: &DesktopLayer, mode: AttachMode) -> bool {
    match mode {
        AttachMode::ClassicWorkerW => layer
            .worker_w
            .map(|w| is_valid_wallpaper_workerw(w, layer.shell_host))
            .unwrap_or(false),
        AttachMode::ShellHostLayered | AttachMode::RaisedProgman => true,
    }
}

/// 将任意 HWND（WebView 等）挂到桌面图标层之下（MPV 优先走 --wid）
pub fn try_attach_to_desktop(hwnd: HWND, layer: &DesktopLayer) -> Result<(), String> {
    let screen = get_virtual_screen_rect();
    let mode = resolve_attach_mode(layer);

    log::info!(
        "TryAttachToDesktop: mode={:?}, hwnd={:?}, screen={}x{}@({},{})",
        mode,
        hwnd,
        screen.w,
        screen.h,
        screen.x,
        screen.y
    );

    unsafe {
        match mode {
            AttachMode::ClassicWorkerW => {
                apply_tool_window_styles(hwnd)?;
                let worker_w = layer
                    .worker_w
                    .ok_or("经典模式找不到 WorkerW")?;
                prepare_wallpaper_host(worker_w, layer.shell_host)?;
                set_child_visible(hwnd)?;
                SetParent(hwnd, Some(worker_w))
                    .map_err(|e| format!("SetParent(WorkerW) 失败: {:?}", e))?;
                resize_desktop_child(hwnd, screen.x, screen.y, screen.w, screen.h)?;
            }
            AttachMode::RaisedProgman | AttachMode::ShellHostLayered => {
                let parent = if mode == AttachMode::RaisedProgman {
                    layer.progman.ok_or("Raised 模式缺少 Progman")?
                } else {
                    layer.shell_host
                };

                set_child_visible(hwnd)?;
                SetParent(hwnd, Some(parent))
                    .map_err(|e| format!("SetParent 失败: {:?}", e))?;

                // Z-order 紧贴 DefView 下方，铺满虚拟桌面（多显示器）
                SetWindowPos(
                    hwnd,
                    Some(layer.shell_view),
                    screen.x,
                    screen.y,
                    screen.w,
                    screen.h,
                    SWP_NOACTIVATE | SWP_SHOWWINDOW,
                )
                .map_err(|e| format!("SetWindowPos(DefView) 失败: {:?}", e))?;

                ensure_workerw_below(hwnd, layer)?;
            }
        }

        let _ = ShowWindow(hwnd, SW_SHOW);
    }

    Ok(())
}

pub fn resolve_attach_mode(layer: &DesktopLayer) -> AttachMode {
    if layer.is_raised && layer.progman.is_some() {
        return AttachMode::RaisedProgman;
    }
    if let Some(worker_w) = layer.worker_w {
        if is_valid_wallpaper_workerw(worker_w, layer.shell_host) {
            return AttachMode::ClassicWorkerW;
        }
    }
    AttachMode::ShellHostLayered
}

/// 飞火策略启动前：隐藏静态壁纸 WorkerW，避免与 MPV 叠层干扰
pub fn prepare_shell_host_wallpaper(layer: &DesktopLayer) -> Result<(), String> {
    hide_static_wallpaper_workers(layer)
}

fn hide_static_wallpaper_workers(layer: &DesktopLayer) -> Result<(), String> {
    unsafe {
        if let Some(ww) = layer.worker_w {
            if is_valid_wallpaper_workerw(ww, layer.shell_host) {
                let _ = ShowWindow(ww, SW_HIDE);
            }
        }
        if let Some(ww) = find_sibling_workerw() {
            if Some(ww) != layer.worker_w {
                let _ = ShowWindow(ww, SW_HIDE);
            }
        }
    }
    Ok(())
}

/// 将 MPV 定位到指定显示器（按当前嵌入模式）
pub fn position_wallpaper_player(
    hwnd: HWND,
    layer: &DesktopLayer,
    mode: AttachMode,
    monitor: MonitorRect,
) -> Result<(), String> {
    match mode {
        AttachMode::ClassicWorkerW => position_player_in_wallpaper_worker(hwnd, monitor),
        AttachMode::ShellHostLayered | AttachMode::RaisedProgman => {
            position_player_on_monitor(hwnd, layer, monitor)
        }
    }
}

fn spawn_workerw() {
    let mut targets = Vec::new();
    if let Some(p) = find_progman() {
        targets.push(p);
    }
    if let Some((host, _)) = detect_shell_layer() {
        if !targets.contains(&host) {
            targets.push(host);
        }
    }
    unsafe {
        for t in targets {
            let _ = SendMessageW(t, 0x052C, Some(WPARAM(0xD)), Some(LPARAM(0)));
            let _ = SendMessageW(t, 0x052C, Some(WPARAM(0xD)), Some(LPARAM(1)));
        }
    }
    std::thread::sleep(std::time::Duration::from_millis(200));
}

fn find_progman_wallpaper_workerw() -> Option<HWND> {
    let progman = find_progman()?;
    if let Some(w) = find_progman_workerw_via_enum_child(progman) {
        return Some(w);
    }
    find_workerw_parented_by(progman)
}

struct ProgmanWorkerSearch {
    best: HWND,
    best_area: i32,
    min_area: i32,
}

unsafe extern "system" fn enum_progman_workerw_child(
    hwnd: HWND,
    lparam: LPARAM,
) -> windows_core::BOOL {
    let s = &mut *(lparam.0 as *mut ProgmanWorkerSearch);
    let mut buf = [0u16; 256];
    if GetClassNameW(hwnd, &mut buf) == 0 {
        return windows_core::BOOL(1);
    }
    let class = String::from_utf16_lossy(
        &buf[..buf.iter().position(|&c| c == 0).unwrap_or(buf.len())],
    );
    if class != "WorkerW" {
        return windows_core::BOOL(1);
    }
    if find_child_defview(hwnd).is_some() {
        return windows_core::BOOL(1);
    }
    let area = get_window_rect(hwnd)
        .map(|r| (r.right - r.left) * (r.bottom - r.top))
        .unwrap_or(0);
    if area >= s.min_area && area > s.best_area {
        s.best_area = area;
        s.best = hwnd;
    }
    windows_core::BOOL(1)
}

fn find_progman_workerw_via_enum_child(progman: HWND) -> Option<HWND> {
    let screen = get_virtual_screen_rect();
    let min_area = (screen.w * screen.h) / 4;
    unsafe {
        let mut s = ProgmanWorkerSearch {
            best: HWND::default(),
            best_area: 0,
            min_area,
        };
        let _ = EnumChildWindows(
            Some(progman),
            Some(enum_progman_workerw_child),
            LPARAM(&mut s as *mut _ as isize),
        );
        if s.best != HWND::default() {
            log::info!(
                "找到 Progman 子壁纸 WorkerW {:?} (area={})",
                s.best,
                s.best_area
            );
            Some(s.best)
        } else {
            None
        }
    }
}

struct ParentWorkerSearch {
    parent: HWND,
    best: HWND,
    best_area: i32,
    min_area: i32,
}

unsafe extern "system" fn enum_workerw_by_parent(
    hwnd: HWND,
    lparam: LPARAM,
) -> windows_core::BOOL {
    let s = &mut *(lparam.0 as *mut ParentWorkerSearch);
    let mut buf = [0u16; 256];
    if GetClassNameW(hwnd, &mut buf) == 0 {
        return windows_core::BOOL(1);
    }
    let class = String::from_utf16_lossy(
        &buf[..buf.iter().position(|&c| c == 0).unwrap_or(buf.len())],
    );
    if class != "WorkerW" {
        return windows_core::BOOL(1);
    }
    let parent = unsafe { GetParent(hwnd).ok() }.unwrap_or(HWND::default());
    if parent != s.parent {
        return windows_core::BOOL(1);
    }
    if find_child_defview(hwnd).is_some() {
        return windows_core::BOOL(1);
    }
    let area = get_window_rect(hwnd)
        .map(|r| (r.right - r.left) * (r.bottom - r.top))
        .unwrap_or(0);
    if area >= s.min_area && area > s.best_area {
        s.best_area = area;
        s.best = hwnd;
    }
    windows_core::BOOL(1)
}

fn find_workerw_parented_by(parent: HWND) -> Option<HWND> {
    let screen = get_virtual_screen_rect();
    let min_area = (screen.w * screen.h) / 4;
    unsafe {
        let mut s = ParentWorkerSearch {
            parent,
            best: HWND::default(),
            best_area: 0,
            min_area,
        };
        let _ = EnumWindows(
            Some(enum_workerw_by_parent),
            LPARAM(&mut s as *mut _ as isize),
        );
        if s.best != HWND::default() {
            log::info!(
                "找到父窗口 {:?} 下壁纸 WorkerW {:?} (area={})",
                parent,
                s.best,
                s.best_area
            );
            Some(s.best)
        } else {
            None
        }
    }
}

fn log_workerw_probe(progman: Option<HWND>, shell_host: HWND) {
    if let Some(p) = progman {
        unsafe {
            let mut count = 0u32;
            struct Ctx {
                count: *mut u32,
            }
            unsafe extern "system" fn log_child(hwnd: HWND, lparam: LPARAM) -> windows_core::BOOL {
                let ctx = &*(lparam.0 as *const Ctx);
                let mut buf = [0u16; 256];
                if GetClassNameW(hwnd, &mut buf) == 0 {
                    return windows_core::BOOL(1);
                }
                let class = String::from_utf16_lossy(
                    &buf[..buf.iter().position(|&c| c == 0).unwrap_or(buf.len())],
                );
                let has_dv = find_child_defview(hwnd).is_some();
                if let Some(rect) = get_window_rect(hwnd) {
                    log::warn!(
                        "Progman 子窗口[{}]: {:?} class={} DefView={} rect={}x{}",
                        unsafe { *ctx.count },
                        hwnd,
                        class,
                        has_dv,
                        rect.right - rect.left,
                        rect.bottom - rect.top
                    );
                }
                unsafe { *ctx.count += 1 };
                windows_core::BOOL(1)
            }
            let ctx = Ctx {
                count: &mut count as *mut u32,
            };
            let _ = EnumChildWindows(Some(p), Some(log_child), LPARAM(&ctx as *const _ as isize));
        }
    }
    log::warn!(
        "未找到壁纸 WorkerW，将回退 ShellHostLayered。progman={:?}, shell_host={:?}",
        progman,
        shell_host
    );
}

fn detect_shell_layer() -> Option<(HWND, HWND)> {
    if let Some(p) = find_progman() {
        if let Some(sv) = find_child_defview(p) {
            return Some((p, sv));
        }
    }

    struct Search {
        host: HWND,
        view: HWND,
    }
    unsafe extern "system" fn cb(hwnd: HWND, lparam: LPARAM) -> windows_core::BOOL {
        let s = &mut *(lparam.0 as *mut Search);
        if let Some(v) = find_child_defview(hwnd) {
            s.host = hwnd;
            s.view = v;
            return windows_core::BOOL(0);
        }
        windows_core::BOOL(1)
    }

    unsafe {
        let mut s = Search {
            host: HWND::default(),
            view: HWND::default(),
        };
        let _ = EnumWindows(Some(cb), LPARAM(&mut s as *mut _ as isize));
        if s.host != HWND::default() {
            Some((s.host, s.view))
        } else {
            None
        }
    }
}

fn find_progman() -> Option<HWND> {
    unsafe {
        let Ok(p) = FindWindowW(windows::core::w!("Progman"), None) else {
            return None;
        };
        if p == HWND::default() {
            None
        } else {
            Some(p)
        }
    }
}

fn find_child_defview(parent: HWND) -> Option<HWND> {
    unsafe {
        match FindWindowExW(Some(parent), None, windows::core::w!("SHELLDLL_DefView"), None) {
            Ok(v) if v != HWND::default() => Some(v),
            _ => None,
        }
    }
}

fn find_child_workerw(parent: HWND) -> Option<HWND> {
    unsafe {
        match FindWindowExW(Some(parent), None, windows::core::w!("WorkerW"), None) {
            Ok(w) if w != HWND::default() => Some(w),
            _ => None,
        }
    }
}

/// 图标宿主 WorkerW 之后的壁纸 WorkerW（Lively 经典路径）
fn find_wallpaper_workerw_after(icon_host: HWND) -> Option<HWND> {
    unsafe {
        let Ok(w) = FindWindowExW(None, Some(icon_host), windows::core::w!("WorkerW"), None) else {
            return None;
        };
        if w == HWND::default() || find_child_defview(w).is_some() {
            None
        } else {
            Some(w)
        }
    }
}

/// 准备壁纸 WorkerW 宿主（显示并铺满虚拟桌面，Z-order 在图标层下方）
pub fn prepare_wallpaper_host(worker_w: HWND, shell_host: HWND) -> Result<(), String> {
    let screen = get_virtual_screen_rect();
    prepare_wallpaper_host_sized(worker_w, shell_host, screen.x, screen.y, screen.w, screen.h)
}

fn prepare_wallpaper_host_sized(
    worker_w: HWND,
    shell_host: HWND,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) -> Result<(), String> {
    unsafe {
        let _ = ShowWindow(worker_w, SW_SHOW);
        SetWindowPos(
            worker_w,
            Some(shell_host),
            x,
            y,
            w,
            h,
            SWP_NOACTIVATE | SWP_SHOWWINDOW,
        )
        .map_err(|e| format!("SetWindowPos(WorkerW) 失败: {:?}", e))?;
    }
    log_window_info("prepare_wallpaper_host", worker_w);
    Ok(())
}

unsafe fn resize_desktop_child(hwnd: HWND, x: i32, y: i32, w: i32, h: i32) -> Result<(), String> {
    MoveWindow(hwnd, x, y, w, h, true)
        .map_err(|e| format!("MoveWindow 失败: {:?}", e))?;
    SetWindowPos(
        hwnd,
        None,
        x,
        y,
        w,
        h,
        SWP_NOACTIVATE | SWP_SHOWWINDOW,
    )
    .map_err(|e| format!("SetWindowPos(child) 失败: {:?}", e))?;
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub struct MonitorRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// 枚举所有显示器区域（虚拟桌面坐标）
pub fn enumerate_monitors() -> Vec<MonitorRect> {
    unsafe {
        struct State {
            monitors: Vec<MonitorRect>,
        }
        unsafe extern "system" fn callback(
            _hmon: HMONITOR,
            _hdc: HDC,
            lprc: *mut RECT,
            lparam: LPARAM,
        ) -> windows_core::BOOL {
            let state = &mut *(lparam.0 as *mut State);
            let rect = *lprc;
            state.monitors.push(MonitorRect {
                x: rect.left,
                y: rect.top,
                w: rect.right - rect.left,
                h: rect.bottom - rect.top,
            });
            windows_core::BOOL(1)
        }
        let mut state = State {
            monitors: Vec::new(),
        };
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(callback),
            LPARAM(&mut state as *mut _ as isize),
        );
        state.monitors.sort_by_key(|m| (m.x, m.y));
        for (i, m) in state.monitors.iter().enumerate() {
            log::info!(
                "Monitor[{}]: {}x{}@({},{})",
                i, m.w, m.h, m.x, m.y
            );
        }
        state.monitors
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VirtualScreen {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

pub fn get_virtual_screen_rect() -> VirtualScreen {
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{
            GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
            SM_YVIRTUALSCREEN,
        };
        VirtualScreen {
            x: GetSystemMetrics(SM_XVIRTUALSCREEN),
            y: GetSystemMetrics(SM_YVIRTUALSCREEN),
            w: GetSystemMetrics(SM_CXVIRTUALSCREEN),
            h: GetSystemMetrics(SM_CYVIRTUALSCREEN),
        }
    }
}

fn get_window_rect(hwnd: HWND) -> Option<RECT> {
    unsafe {
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_ok() {
            Some(rect)
        } else {
            None
        }
    }
}

fn is_valid_wallpaper_workerw(worker_w: HWND, shell_host: HWND) -> bool {
    worker_w != shell_host && find_child_defview(worker_w).is_none()
}

#[allow(dead_code)]
fn is_valid_classic_workerw(worker_w: HWND, shell_host: HWND) -> bool {
    is_valid_wallpaper_workerw(worker_w, shell_host) && is_top_level_workerw(worker_w)
}

fn is_top_level_workerw(hwnd: HWND) -> bool {
    unsafe { GetParent(hwnd).ok() == Some(HWND::default()) }
}

fn log_window_info(label: &str, hwnd: HWND) {
    if let Some(rect) = get_window_rect(hwnd) {
        let ww = rect.right - rect.left;
        let wh = rect.bottom - rect.top;
        let parent = unsafe { GetParent(hwnd).ok() }.unwrap_or(HWND::default());
        let visible = unsafe { IsWindowVisible(hwnd).as_bool() };
        log::info!(
            "{}: hwnd={:?}, parent={:?}, visible={}, rect={}x{}@({},{})",
            label,
            hwnd,
            parent,
            visible,
            ww,
            wh,
            rect.left,
            rect.top
        );
    }
}

struct HwndSearch {
    result: HWND,
}

unsafe extern "system" fn enum_classic_workerw(hwnd: HWND, lparam: LPARAM) -> windows_core::BOOL {
    let s = &mut *(lparam.0 as *mut HwndSearch);
    let mut buf = [0u16; 256];
    if GetClassNameW(hwnd, &mut buf) == 0 {
        return windows_core::BOOL(1);
    }
    let class = String::from_utf16_lossy(
        &buf[..buf.iter().position(|&c| c == 0).unwrap_or(buf.len())],
    );
    if class != "WorkerW" {
        return windows_core::BOOL(1);
    }
    if find_child_defview(hwnd).is_some() {
        return windows_core::BOOL(1);
    }
    s.result = hwnd;
    windows_core::BOOL(0)
}

fn find_classic_workerw() -> Option<HWND> {
    unsafe {
        let mut s = HwndSearch {
            result: HWND::default(),
        };
        let _ = EnumWindows(Some(enum_classic_workerw), LPARAM(&mut s as *mut _ as isize));
        if s.result != HWND::default() && is_top_level_workerw(s.result) {
            Some(s.result)
        } else {
            None
        }
    }
}

unsafe extern "system" fn enum_sibling_workerw(hwnd: HWND, lparam: LPARAM) -> windows_core::BOOL {
    let s = &mut *(lparam.0 as *mut HwndSearch);
    if find_child_defview(hwnd).is_none() {
        return windows_core::BOOL(1);
    }
    let mut after = Some(hwnd);
    loop {
        let Ok(w) = FindWindowExW(None, after, windows::core::w!("WorkerW"), None) else {
            break;
        };
        if w == HWND::default() {
            break;
        }
        after = Some(w);
        if find_child_defview(w).is_none() && is_top_level_workerw(w) {
            s.result = w;
            return windows_core::BOOL(0);
        }
    }
    windows_core::BOOL(1)
}

fn find_sibling_workerw() -> Option<HWND> {
    unsafe {
        let mut s = HwndSearch {
            result: HWND::default(),
        };
        let _ = EnumWindows(Some(enum_sibling_workerw), LPARAM(&mut s as *mut _ as isize));
        if s.result != HWND::default() && is_top_level_workerw(s.result) {
            Some(s.result)
        } else {
            None
        }
    }
}

fn has_extended_style_noredirectionbitmap(hwnd: HWND) -> bool {
    unsafe {
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        (ex & WS_EX_NOREDIRECTIONBITMAP.0) != 0
    }
}

unsafe fn apply_tool_window_styles(hwnd: HWND) -> Result<(), String> {
    let mut ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
    ex |= WS_EX_NOACTIVATE.0 | WS_EX_TOOLWINDOW.0;
    SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex as isize);
    Ok(())
}

unsafe fn ensure_workerw_below(player: HWND, layer: &DesktopLayer) -> Result<(), String> {
    if let Some(ww) = layer.worker_w {
        let _ = ShowWindow(ww, SW_HIDE);
        let _ = SetWindowPos(
            ww,
            Some(player),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );
    }
    if let Some(ww) = find_sibling_workerw() {
        if Some(ww) != layer.worker_w {
            let _ = ShowWindow(ww, SW_HIDE);
        }
    }
    Ok(())
}

fn get_screen_size() -> (i32, i32) {
    let s = get_virtual_screen_rect();
    (s.w, s.h)
}

unsafe fn set_child_visible(hwnd: HWND) -> Result<(), String> {
    let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
    SetWindowLongPtrW(
        hwnd,
        GWL_STYLE,
        (style | WS_CHILD.0 | WS_VISIBLE.0) as isize,
    );
    Ok(())
}
