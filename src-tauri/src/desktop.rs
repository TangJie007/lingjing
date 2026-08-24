#[cfg(windows)]
mod win {
    use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, FindWindowExW, FindWindowW, GetClassNameW, GetParent, IsWindowVisible,
        SetWindowsHookExW, ShowWindow, UnhookWindowsHookEx, WindowFromPoint, HHOOK, SW_HIDE, SW_SHOW,
        WH_MOUSE_LL, WM_LBUTTONDBLCLK,
    };

    static ENABLED: AtomicBool = AtomicBool::new(false);
    static HOOK: AtomicIsize = AtomicIsize::new(0);

    #[repr(C)]
    struct MouseLowLevelHook {
        pt: POINT,
        mouse_data: u32,
        flags: u32,
        time: u32,
        extra_info: usize,
    }

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
            let mut worker = FindWindowW(workerw.as_ptr(), std::ptr::null());
            while !worker.is_null() {
                let shell =
                    FindWindowExW(worker, std::ptr::null_mut(), shelldll.as_ptr(), std::ptr::null());
                if !shell.is_null() {
                    let lv =
                        FindWindowExW(shell, std::ptr::null_mut(), listview.as_ptr(), std::ptr::null());
                    if !lv.is_null() {
                        return lv;
                    }
                }
                worker = FindWindowExW(std::ptr::null_mut(), worker, workerw.as_ptr(), std::ptr::null());
            }
            let progman = wide("Progman");
            let prog = FindWindowW(progman.as_ptr(), std::ptr::null());
            if !prog.is_null() {
                let shell =
                    FindWindowExW(prog, std::ptr::null_mut(), shelldll.as_ptr(), std::ptr::null());
                if !shell.is_null() {
                    let lv =
                        FindWindowExW(shell, std::ptr::null_mut(), listview.as_ptr(), std::ptr::null());
                    if !lv.is_null() {
                        return lv;
                    }
                }
            }
            std::ptr::null_mut()
        }
    }

    fn icons_visible() -> bool {
        unsafe {
            let lv = find_desktop_listview();
            if lv.is_null() {
                return true;
            }
            IsWindowVisible(lv) != 0
        }
    }

    pub fn set_icons_visible(visible: bool) {
        unsafe {
            let lv = find_desktop_listview();
            if !lv.is_null() {
                ShowWindow(lv, if visible { SW_SHOW } else { SW_HIDE });
            }
        }
    }

    fn toggle_icons_visible() {
        set_icons_visible(!icons_visible());
    }

    unsafe fn install_hook() {
        if HOOK.load(Ordering::SeqCst) != 0 {
            return;
        }
        let module = GetModuleHandleW(std::ptr::null());
        let hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), module, 0);
        if !hook.is_null() {
            HOOK.store(hook as isize, Ordering::SeqCst);
        }
    }

    unsafe fn uninstall_hook() {
        let handle = HOOK.swap(0, Ordering::SeqCst);
        if handle != 0 {
            UnhookWindowsHookEx(handle as HHOOK);
        }
    }

    fn class_name(hwnd: HWND) -> String {
        let mut buf = [0u16; 256];
        let len = unsafe { GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
        if len <= 0 {
            return String::new();
        }
        String::from_utf16_lossy(&buf[..len as usize])
    }

    fn is_desktop_hwnd(mut hwnd: HWND) -> bool {
        for _ in 0..10 {
            if hwnd.is_null() {
                break;
            }
            let name = class_name(hwnd);
            if name == "SysListView32" || name == "WorkerW" || name == "Progman" {
                return true;
            }
            hwnd = unsafe { GetParent(hwnd) };
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
        let hook = HOOK.load(Ordering::SeqCst) as HHOOK;
        if code >= 0 && ENABLED.load(Ordering::SeqCst) && wparam == WM_LBUTTONDBLCLK as WPARAM {
            let info = &*(lparam as *const MouseLowLevelHook);
            if is_desktop_at_point(info.pt.x, info.pt.y) {
                toggle_icons_visible();
            }
        }
        CallNextHookEx(hook, code, wparam, lparam)
    }
}

#[cfg(windows)]
pub use win::{set_double_click_enabled, set_icons_visible};

#[cfg(windows)]
pub fn apply_frameless_dwm(hwnd_raw: isize) {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_CAPTION_COLOR, DWMWA_COLOR_NONE,
    };
    unsafe {
        let hwnd = hwnd_raw as HWND;
        let none = DWMWA_COLOR_NONE;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_BORDER_COLOR as u32,
            &none as *const _ as *const core::ffi::c_void,
            4,
        );
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_CAPTION_COLOR as u32,
            &none as *const _ as *const core::ffi::c_void,
            4,
        );
    }
}

#[cfg(not(windows))]
pub fn apply_frameless_dwm(_hwnd_raw: isize) {}

#[cfg(windows)]
pub fn drag_window_by_mouse(hwnd_raw: isize) {
    use windows_sys::Win32::Foundation::{HWND, POINT, RECT};
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetCursorPos, GetWindowRect, SetWindowPos, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER,
    };
    unsafe {
        let hwnd = hwnd_raw as HWND;
        let mut origin = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut origin) == 0 {
            return;
        }
        let mut wr = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if GetWindowRect(hwnd, &mut wr) == 0 {
            return;
        }
        let dx = origin.x - wr.left;
        let dy = origin.y - wr.top;
        while GetAsyncKeyState(VK_LBUTTON as i32) as u16 & 0x8000 != 0 {
            let mut cur = POINT { x: 0, y: 0 };
            if GetCursorPos(&mut cur) != 0 {
                SetWindowPos(
                    hwnd,
                    std::ptr::null_mut(),
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
