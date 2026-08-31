#[cfg(windows)]
mod win {
    use std::os::windows::process::CommandExt;
    use std::path::PathBuf;
    use std::process::Command;
    use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{
        CloseHandle, HWND, LPARAM, LRESULT, POINT, WAIT_OBJECT_0, WAIT_TIMEOUT, WPARAM,
    };
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::System::Threading::{
        OpenProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, FindWindowExW, FindWindowW, GetClassNameW, GetParent, IsWindowVisible,
        SetWindowsHookExW, ShowWindow, UnhookWindowsHookEx, WindowFromPoint, HHOOK, MSLLHOOKSTRUCT,
        SW_HIDE, SW_SHOW, WH_MOUSE_LL, WM_LBUTTONDBLCLK,
    };

    static ENABLED: AtomicBool = AtomicBool::new(false);
    static HOOK: AtomicIsize = AtomicIsize::new(0);
    const GUARD_ARG: &str = "--desktop-icons-guard";
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    const GUARD_FILE: &str = "lingscape-desktop-icons.guard";

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub fn set_double_click_enabled(enabled: bool) {
        ENABLED.store(enabled, Ordering::SeqCst);
        unsafe {
            if enabled {
                install_hook();
            } else {
                uninstall_hook();
            }
        }
    }

    fn find_desktop_listview() -> HWND {
        unsafe {
            let workerw = wide("WorkerW");
            let shelldll = wide("SHELLDLL_DefView");
            let listview = wide("SysListView32");
            let mut worker =
                FindWindowW(PCWSTR(workerw.as_ptr()), PCWSTR::null()).unwrap_or_default();
            while !worker.0.is_null() {
                if let Ok(shell) = FindWindowExW(
                    Some(worker),
                    None,
                    PCWSTR(shelldll.as_ptr()),
                    PCWSTR::null(),
                ) {
                    if let Ok(lv) = FindWindowExW(
                        Some(shell),
                        None,
                        PCWSTR(listview.as_ptr()),
                        PCWSTR::null(),
                    ) {
                        return lv;
                    }
                }
                worker = FindWindowExW(
                    None,
                    Some(worker),
                    PCWSTR(workerw.as_ptr()),
                    PCWSTR::null(),
                )
                .unwrap_or_default();
            }
            let progman = wide("Progman");
            if let Ok(prog) = FindWindowW(PCWSTR(progman.as_ptr()), PCWSTR::null()) {
                if let Ok(shell) = FindWindowExW(
                    Some(prog),
                    None,
                    PCWSTR(shelldll.as_ptr()),
                    PCWSTR::null(),
                ) {
                    if let Ok(lv) = FindWindowExW(
                        Some(shell),
                        None,
                        PCWSTR(listview.as_ptr()),
                        PCWSTR::null(),
                    ) {
                        return lv;
                    }
                }
            }
            HWND::default()
        }
    }

    fn icons_visible() -> bool {
        unsafe {
            let lv = find_desktop_listview();
            if lv.0.is_null() {
                return true;
            }
            IsWindowVisible(lv).as_bool()
        }
    }

    pub fn set_icons_visible(visible: bool) {
        unsafe {
            let lv = find_desktop_listview();
            if !lv.0.is_null() {
                let _ = ShowWindow(lv, if visible { SW_SHOW } else { SW_HIDE });
            }
        }
    }

    fn guard_path() -> PathBuf {
        std::env::temp_dir().join(GUARD_FILE)
    }

    fn guard_owner() -> Option<u32> {
        std::fs::read_to_string(guard_path())
            .ok()
            .and_then(|s| s.trim().parse().ok())
    }

    fn process_running(pid: u32) -> bool {
        let Ok(process) = (unsafe { OpenProcess(PROCESS_SYNCHRONIZE, false, pid) }) else {
            return false;
        };
        let result = unsafe { WaitForSingleObject(process, 0) } == WAIT_TIMEOUT;
        unsafe {
            let _ = CloseHandle(process);
        }
        result
    }

    pub fn recover_icons_after_crash() {
        let path = guard_path();
        let stale = match guard_owner() {
            Some(pid) => !process_running(pid),
            None => path.exists(),
        };
        if stale {
            set_icons_visible(true);
            let _ = std::fs::remove_file(path);
            tracing::info!("[desktop-organize] recovered icons after previous crash");
        }
    }

    pub fn start_icons_restore_guard() -> Result<(), String> {
        let pid = std::process::id();
        if let Some(owner) = guard_owner() {
            if owner == pid {
                return Ok(());
            }
            if process_running(owner) {
                return Err("另一个灵镜实例正在管理桌面图标".into());
            }
        }

        let path = guard_path();
        std::fs::write(&path, pid.to_string())
            .map_err(|e| format!("创建桌面图标恢复标记失败: {e}"))?;
        let exe = std::env::current_exe().map_err(|e| format!("获取程序路径失败: {e}"))?;
        match Command::new(exe)
            .arg(GUARD_ARG)
            .arg(pid.to_string())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
        {
            Ok(_) => Ok(()),
            Err(e) => {
                let _ = std::fs::remove_file(path);
                Err(format!("启动桌面图标恢复守护失败: {e}"))
            }
        }
    }

    pub fn stop_icons_restore_guard() {
        if guard_owner() == Some(std::process::id()) {
            let _ = std::fs::remove_file(guard_path());
        }
    }

    pub fn maybe_run_icons_restore_guard() -> bool {
        let mut args = std::env::args().skip(1);
        if args.next().as_deref() != Some(GUARD_ARG) {
            return false;
        }
        let Some(pid) = args.next().and_then(|s| s.parse::<u32>().ok()) else {
            return true;
        };

        let process = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, false, pid) };
        if let Ok(process) = process {
            loop {
                if guard_owner() != Some(pid) {
                    unsafe {
                        let _ = CloseHandle(process);
                    }
                    return true;
                }
                match unsafe { WaitForSingleObject(process, 250) } {
                    WAIT_TIMEOUT => continue,
                    WAIT_OBJECT_0 => break,
                    _ => break,
                }
            }
            unsafe {
                let _ = CloseHandle(process);
            }
        }

        // Only the guard that still owns the marker may restore/remove it.
        if guard_owner() == Some(pid) {
            set_icons_visible(true);
            let _ = std::fs::remove_file(guard_path());
        }
        true
    }

    fn toggle_icons_visible() {
        set_icons_visible(!icons_visible());
    }

    unsafe fn install_hook() {
        if HOOK.load(Ordering::SeqCst) != 0 {
            return;
        }
        let module = GetModuleHandleW(None).ok().map(Into::into);
        if let Ok(hook) = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), module, 0) {
            HOOK.store(hook.0 as isize, Ordering::SeqCst);
        }
    }

    unsafe fn uninstall_hook() {
        let handle = HOOK.swap(0, Ordering::SeqCst);
        if handle != 0 {
            let _ = UnhookWindowsHookEx(HHOOK(handle as *mut _));
        }
    }

    fn class_name(hwnd: HWND) -> String {
        let mut buf = [0u16; 256];
        let len = unsafe { GetClassNameW(hwnd, &mut buf) };
        if len <= 0 {
            return String::new();
        }
        String::from_utf16_lossy(&buf[..len as usize])
    }

    fn is_desktop_hwnd(mut hwnd: HWND) -> bool {
        for _ in 0..10 {
            if hwnd.0.is_null() {
                break;
            }
            let name = class_name(hwnd);
            if name == "SysListView32" || name == "WorkerW" || name == "Progman" {
                return true;
            }
            hwnd = unsafe { GetParent(hwnd).unwrap_or_default() };
        }
        false
    }

    fn is_desktop_at_point(x: i32, y: i32) -> bool {
        unsafe {
            let pt = POINT { x, y };
            let hwnd = WindowFromPoint(pt);
            is_desktop_hwnd(hwnd)
        }
    }

    unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        let hook = HHOOK(HOOK.load(Ordering::SeqCst) as *mut _);
        if code >= 0
            && ENABLED.load(Ordering::SeqCst)
            && wparam.0 == WM_LBUTTONDBLCLK as usize
        {
            let info = &*(lparam.0 as *const MSLLHOOKSTRUCT);
            if is_desktop_at_point(info.pt.x, info.pt.y) {
                toggle_icons_visible();
            }
        }
        CallNextHookEx(Some(hook), code, wparam, lparam)
    }
}

#[cfg(windows)]
pub use win::{
    maybe_run_icons_restore_guard, recover_icons_after_crash, set_double_click_enabled,
    set_icons_visible, start_icons_restore_guard, stop_icons_restore_guard,
};

#[cfg(windows)]
pub fn apply_frameless_dwm(hwnd_raw: isize) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_CAPTION_COLOR, DWMWA_COLOR_NONE,
    };
    unsafe {
        let hwnd = HWND(hwnd_raw as *mut _);
        let none = DWMWA_COLOR_NONE;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_BORDER_COLOR,
            &none as *const _ as *const core::ffi::c_void,
            4,
        );
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_CAPTION_COLOR,
            &none as *const _ as *const core::ffi::c_void,
            4,
        );
    }
}

#[cfg(not(windows))]
pub fn apply_frameless_dwm(_hwnd_raw: isize) {}

#[cfg(windows)]
pub fn drag_window_by_mouse(hwnd_raw: isize) {
    use windows::Win32::Foundation::{HWND, POINT, RECT};
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
    use windows::Win32::UI::WindowsAndMessaging::{
        GetCursorPos, GetWindowRect, SetWindowPos, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER,
    };
    unsafe {
        let hwnd = HWND(hwnd_raw as *mut _);
        let mut origin = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut origin).is_err() {
            return;
        }
        let mut wr = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if GetWindowRect(hwnd, &mut wr).is_err() {
            return;
        }
        let dx = origin.x - wr.left;
        let dy = origin.y - wr.top;
        while GetAsyncKeyState(VK_LBUTTON.0 as i32) as u16 & 0x8000 != 0 {
            let mut cur = POINT { x: 0, y: 0 };
            if GetCursorPos(&mut cur).is_ok() {
                let _ = SetWindowPos(
                    hwnd,
                    None,
                    cur.x - dx,
                    cur.y - dy,
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
            std::thread::sleep(std::time::Duration::from_millis(8));
        }
    }
}

#[cfg(not(windows))]
pub fn drag_window_by_mouse(_hwnd_raw: isize) {}

#[cfg(not(windows))]
pub fn set_double_click_enabled(_enabled: bool) {}

#[cfg(not(windows))]
pub fn set_icons_visible(_visible: bool) {}

#[cfg(not(windows))]
pub fn start_icons_restore_guard() -> Result<(), String> {
    Ok(())
}

#[cfg(not(windows))]
pub fn stop_icons_restore_guard() {}

#[cfg(not(windows))]
pub fn recover_icons_after_crash() {}

#[cfg(not(windows))]
pub fn maybe_run_icons_restore_guard() -> bool {
    false
}
