//! 视频动态壁纸播放器（参考 Lively / weebp：mpv --wid，多屏每显示器一个实例）

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{HWND, LPARAM};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    EnumChildWindows, EnumWindows, GetClassNameW, GetWindowThreadProcessId,
    ShowWindow, SW_SHOW,
};

#[cfg(target_os = "windows")]
use crate::desktop_core::{self, AttachMode, MonitorRect};

#[cfg(target_os = "windows")]
static DESKTOP_WATCHDOG: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "windows")]
fn start_desktop_watchdog(state: &VideoPlayerState) {
    if DESKTOP_WATCHDOG.swap(true, Ordering::SeqCst) {
        return;
    }
    let state_addr = state as *const VideoPlayerState as usize;
    std::thread::spawn(move || {
        while DESKTOP_WATCHDOG.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_secs(3));
            if !DESKTOP_WATCHDOG.load(Ordering::SeqCst) {
                break;
            }
            let state = unsafe { &*(state_addr as *const VideoPlayerState) };
            let _ = maintain_desktop_wallpaper(state);
        }
    });
}

#[cfg(target_os = "windows")]
fn maintain_desktop_wallpaper(state: &VideoPlayerState) -> Result<(), String> {
    let pids: Vec<u32> = {
        let guard = state.0.lock().unwrap();
        if guard.pids.is_empty() {
            return Ok(());
        }
        guard.pids.clone()
    };

    let layer = desktop_core::detect_desktop_layer()?;

    let _ = desktop_core::suppress_competing_wallpaper_players(&pids);

    let mpv_missing = pids.iter().any(|&pid| !mpv_has_visible_window(pid));
    if desktop_core::desktop_wallpaper_needs_recovery(&layer) || mpv_missing {
        log::info!("检测到桌面壁纸层异常，正在恢复动态壁纸...");
        reattach_if_running(state)?;
    } else {
        desktop_core::refresh_desktop_zorder(&layer)?;
    }
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn mpv_has_visible_window(pid: u32) -> bool {
    find_mpv_anywhere(pid)
        .map(|hwnd| desktop_core::is_window_visible(hwnd))
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
fn stop_desktop_watchdog() {
    DESKTOP_WATCHDOG.store(false, Ordering::SeqCst);
}

#[cfg(target_os = "windows")]
fn stop_zorder_maintainer() {
    stop_desktop_watchdog();
}

pub struct VideoPlayerState(pub Mutex<VideoPlayerInner>);

pub struct VideoPlayerInner {
    pub children: Vec<Child>,
    pub pids: Vec<u32>,
}

impl Default for VideoPlayerInner {
    fn default() -> Self {
        Self {
            children: Vec::new(),
            pids: Vec::new(),
        }
    }
}

impl Default for VideoPlayerState {
    fn default() -> Self {
        Self(Mutex::new(VideoPlayerInner::default()))
    }
}

/// 启动 MPV 视频壁纸（须在 Windows UI 主线程调用）
pub fn start_video_wallpaper(state: &VideoPlayerState, path: &Path) -> Result<(), String> {
    stop_video_wallpaper(state);

    let mpv = resolve_mpv_executable().ok_or(
        "未找到 mpv.exe。请安装 MPV 或将 mpv.exe 放到应用目录 bin/mpv/ 下（https://mpv.io/installation/）",
    )?;

    let path_str = path.to_string_lossy().replace('\\', "/");
    log::info!("启动 MPV 视频壁纸: {}", path_str);

    #[cfg(target_os = "windows")]
    {
        let layer = desktop_core::setup_desktop_layer()?;
        let mode = desktop_core::resolve_attach_mode(&layer);
        let use_wid = desktop_core::should_use_wid(&layer, mode);
        let monitors = desktop_core::enumerate_monitors();
        if monitors.is_empty() {
            return Err("未检测到显示器".into());
        }

        let _ = desktop_core::suppress_competing_wallpaper_players(&[]);

        let wid_target = if use_wid {
            let target = desktop_core::wid_target(&layer, mode)
                .ok_or("找不到 MPV --wid 目标窗口")?;
            if mode == AttachMode::ClassicWorkerW {
                desktop_core::prepare_wallpaper_host(target, layer.shell_host)?;
            } else if mode == AttachMode::ShellHostLayered {
                desktop_core::prepare_shell_host_wallpaper(&layer)?;
            }
            log::info!("MPV --wid 模式: {:?} → {:?}", mode, target);
            Some(target)
        } else {
            log::info!("MPV SetParent 模式: {:?}", mode);
            None
        };

        let mpv_dir = mpv.parent().map(PathBuf::from);
        let mut spawned_children = Vec::new();
        let mut spawned_pids = Vec::new();

        for (index, monitor) in monitors.iter().enumerate() {
            let mut args = mpv_base_args(*monitor);
            if let Some(target) = wid_target {
                args.push(format!("--wid={}", target.0 as usize));
            } else {
                args.push("--force-window=yes".into());
            }
            args.push(path_str.clone());

            let mut cmd = Command::new(&mpv);
            if let Some(ref dir) = mpv_dir {
                cmd.current_dir(dir);
            }

            let child = cmd
                .args(&args)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| format!("启动 MPV 失败 (显示器 {}): {}", index, e))?;

            let pid = child.id();
            spawned_pids.push(pid);
            spawned_children.push(child);

            std::thread::sleep(Duration::from_millis(350));

            if use_wid {
                if let Some(parent) = wid_target {
                    let hwnd = wait_for_mpv_window(pid, Some(parent), &spawned_pids[..index])?;
                    desktop_core::position_wallpaper_player(hwnd, &layer, mode, *monitor)?;
                    log::info!(
                        "MPV[{}] 窗口 {:?} 已铺满显示器 {}x{}@({},{})",
                        index,
                        hwnd,
                        monitor.w,
                        monitor.h,
                        monitor.x,
                        monitor.y
                    );
                }
            } else if index == 0 {
                let hwnd = wait_for_mpv_window(pid, None, &[])?;
                desktop_core::try_attach_to_desktop(hwnd, &layer)?;
                unsafe {
                    let _ = ShowWindow(hwnd, SW_SHOW);
                }
            }
        }

        desktop_core::ensure_desktop_zorder(&layer)?;
        let _ = desktop_core::suppress_competing_wallpaper_players(&spawned_pids);
        start_desktop_watchdog(state);

        let mut guard = state.0.lock().unwrap();
        guard.children = spawned_children;
        guard.pids = spawned_pids;
        return Ok(());
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (mpv, path_str);
        Err("视频动态壁纸仅支持 Windows".into())
    }
}

fn mpv_base_args(monitor: MonitorRect) -> Vec<String> {
    vec![
        "--no-border".into(),
        "--loop-file=inf".into(),
        "--hwdec=auto-safe".into(),
        "--vo=gpu".into(),
        "--gpu-context=win".into(),
        "--mute=yes".into(),
        "--osc=no".into(),
        "--input-default-bindings=no".into(),
        "--input-vo-keyboard=no".into(),
        "--no-keepaspect-window".into(),
        "--panscan=1.0".into(),
        "--cursor-autohide=always".into(),
        "--input-cursor=no".into(),
        format!(
            "--geometry={}x{}+{}+{}",
            monitor.w, monitor.h, monitor.x, monitor.y
        ),
    ]
}

pub fn stop_video_wallpaper(state: &VideoPlayerState) {
    #[cfg(target_os = "windows")]
    {
        stop_desktop_watchdog();
        let _ = desktop_core::restore_competing_wallpaper_players();
    }

    let mut guard = state.0.lock().unwrap();
    kill_all_processes(&mut guard);
}

fn kill_all_processes(inner: &mut VideoPlayerInner) {
    let pids = inner.pids.clone();
    for mut child in inner.children.drain(..) {
        let _ = child.kill();
        let _ = child.wait();
    }
    inner.pids.clear();

    #[cfg(target_os = "windows")]
    force_kill_pids(&pids);

    if !pids.is_empty() {
        log::info!("已停止 MPV 进程: {:?}", pids);
    }
}

#[cfg(target_os = "windows")]
fn force_kill_pids(pids: &[u32]) {
    for &pid in pids {
        if pid == 0 {
            continue;
        }
        let status = Command::new("taskkill")
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        if let Ok(s) = status {
            if !s.success() {
                log::warn!("taskkill 未能终止 MPV pid={}", pid);
            }
        }
    }
}

impl Drop for VideoPlayerState {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        {
            stop_desktop_watchdog();
        }
        if let Ok(mut guard) = self.0.lock() {
            kill_all_processes(&mut guard);
        }
    }
}

pub fn reattach_if_running(state: &VideoPlayerState) -> Result<(), String> {
    let (pids, monitors) = {
        let guard = state.0.lock().unwrap();
        if guard.pids.is_empty() {
            return Ok(());
        }
        (guard.pids.clone(), desktop_core::enumerate_monitors())
    };

    #[cfg(target_os = "windows")]
    {
        let layer = desktop_core::detect_desktop_layer()?;
        let mode = desktop_core::resolve_attach_mode(&layer);
        let use_wid = desktop_core::should_use_wid(&layer, mode);
        let wid_parent = desktop_core::wid_target(&layer, mode);

        for (index, &pid) in pids.iter().enumerate() {
            let monitor = monitors.get(index).copied().unwrap_or_else(|| {
                desktop_core::get_virtual_screen_rect().into()
            });

            if use_wid {
                if let Some(parent) = wid_parent {
                    let skip: Vec<u32> = pids.iter().copied().filter(|&p| p != pid).collect();
                    let hwnd = wait_for_mpv_window(pid, Some(parent), &skip)?;
                    desktop_core::position_wallpaper_player(hwnd, &layer, mode, monitor)?;
                }
            } else if index == 0 {
                let hwnd = wait_for_mpv_window(pid, None, &[])?;
                desktop_core::try_attach_to_desktop(hwnd, &layer)?;
                unsafe {
                    let _ = ShowWindow(hwnd, SW_SHOW);
                }
            }
        }

        desktop_core::ensure_desktop_zorder(&layer)?;
    }
    Ok(())
}

fn resolve_mpv_executable() -> Option<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for candidate in [
                dir.join("bin").join("mpv").join("mpv.exe"),
                dir.join("mpv.exe"),
            ] {
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }
    }
    std::env::var_os("PATH").and_then(|path_var| {
        std::env::split_paths(&path_var)
            .map(|d| d.join("mpv.exe"))
            .find(|p| p.exists())
    })
}

#[cfg(target_os = "windows")]
impl From<desktop_core::VirtualScreen> for MonitorRect {
    fn from(screen: desktop_core::VirtualScreen) -> Self {
        Self {
            x: screen.x,
            y: screen.y,
            w: screen.w,
            h: screen.h,
        }
    }
}

#[cfg(target_os = "windows")]
fn wait_for_mpv_window(
    pid: u32,
    parent_hint: Option<HWND>,
    skip_pids: &[u32],
) -> Result<HWND, String> {
    for _ in 0..100 {
        if let Some(parent) = parent_hint {
            if let Some(hwnd) = find_mpv_under_parent(parent, pid) {
                return Ok(hwnd);
            }
        }
        if let Some(hwnd) = find_mpv_top_level(pid) {
            return Ok(hwnd);
        }
        if let Some(hwnd) = find_mpv_anywhere(pid) {
            return Ok(hwnd);
        }
        let _ = skip_pids;
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(format!("等待 MPV 窗口超时 (pid={})", pid))
}

#[cfg(windows)]
fn is_mpv_class(hwnd: HWND) -> bool {
    unsafe {
        let mut buf = [0u16; 64];
        if GetClassNameW(hwnd, &mut buf) == 0 {
            return false;
        }
        let class = String::from_utf16_lossy(
            &buf[..buf.iter().position(|&c| c == 0).unwrap_or(buf.len())],
        );
        class == "mpv" || class == "glfw"
    }
}

#[cfg(windows)]
fn hwnd_owned_by_pid(hwnd: HWND, pid: u32) -> bool {
    unsafe {
        let mut win_pid: u32 = 0;
        let _ = GetWindowThreadProcessId(hwnd, Some(&mut win_pid));
        win_pid == pid
    }
}

#[cfg(windows)]
fn find_mpv_under_parent(parent: HWND, pid: u32) -> Option<HWND> {
    unsafe {
        struct Search {
            pid: u32,
            hwnd: HWND,
        }
        unsafe extern "system" fn enum_child(hwnd: HWND, lparam: LPARAM) -> windows_core::BOOL {
            let s = &mut *(lparam.0 as *mut Search);
            if hwnd_owned_by_pid(hwnd, s.pid) && is_mpv_class(hwnd) {
                s.hwnd = hwnd;
                return windows_core::BOOL(0);
            }
            windows_core::BOOL(1)
        }
        let mut s = Search {
            pid,
            hwnd: HWND::default(),
        };
        let _ = EnumChildWindows(Some(parent), Some(enum_child), LPARAM(&mut s as *mut _ as isize));
        if s.hwnd != HWND::default() {
            Some(s.hwnd)
        } else {
            None
        }
    }
}

#[cfg(windows)]
struct PidSearch {
    pid: u32,
    hwnd: HWND,
}

#[cfg(windows)]
unsafe extern "system" fn enum_pid_window(hwnd: HWND, lparam: LPARAM) -> windows_core::BOOL {
    let s = &mut *(lparam.0 as *mut PidSearch);
    if !hwnd_owned_by_pid(hwnd, s.pid) || !is_mpv_class(hwnd) {
        return windows_core::BOOL(1);
    }
    s.hwnd = hwnd;
    windows_core::BOOL(0)
}

#[cfg(windows)]
fn find_mpv_anywhere(pid: u32) -> Option<HWND> {
    unsafe {
        struct Search {
            pid: u32,
            hwnd: HWND,
        }
        unsafe extern "system" fn enum_top(hwnd: HWND, lparam: LPARAM) -> windows_core::BOOL {
            let s = &mut *(lparam.0 as *mut Search);
            if let Some(h) = find_mpv_under_parent(hwnd, s.pid) {
                s.hwnd = h;
                return windows_core::BOOL(0);
            }
            windows_core::BOOL(1)
        }
        let mut s = Search {
            pid,
            hwnd: HWND::default(),
        };
        let _ = EnumWindows(Some(enum_top), LPARAM(&mut s as *mut _ as isize));
        if s.hwnd != HWND::default() {
            Some(s.hwnd)
        } else {
            None
        }
    }
}

#[cfg(windows)]
fn find_mpv_top_level(pid: u32) -> Option<HWND> {
    unsafe {
        let mut s = PidSearch {
            pid,
            hwnd: HWND::default(),
        };
        let _ = EnumWindows(Some(enum_pid_window), LPARAM(&mut s as *mut _ as isize));
        if s.hwnd != HWND::default() {
            Some(s.hwnd)
        } else {
            None
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn reattach_if_running(_state: &VideoPlayerState) -> Result<(), String> {
    Ok(())
}
