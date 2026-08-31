use windows::core::PCWSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, DispatchMessageW, PeekMessageW, TranslateMessage, MSG,
    PM_REMOVE, WM_QUIT, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP,
};

use super::util::wide;
use std::time::Duration;

pub(crate) unsafe fn pump_for(ms: u64) {
    let deadline = std::time::Instant::now() + Duration::from_millis(ms);
    while std::time::Instant::now() < deadline {
        pump_messages();
        std::thread::sleep(Duration::from_millis(20));
    }
}

pub fn create_host_window() -> Result<HWND, String> {
    unsafe {
        let class = wide("STATIC");
        let title = wide("LingScape Shell Menu Host");
        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            PCWSTR(class.as_ptr()),
            PCWSTR(title.as_ptr()),
            WS_POPUP,
            0,
            0,
            1,
            1,
            None,
            None,
            None,
            None,
        )
        .map_err(|e| format!("创建 Shell 菜单宿主窗口失败: {e}"))?;
        Ok(hwnd)
    }
}

pub fn pump_messages() {
    unsafe {
        let mut msg = MSG::default();
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            if msg.message == WM_QUIT {
                break;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

pub fn destroy_host_window(hwnd: HWND) {
    if !hwnd.0.is_null() {
        unsafe {
            let _ = DestroyWindow(hwnd);
        }
    }
}

