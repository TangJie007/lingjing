//! Clipboard helpers for Shell-menu paste affordances (CF_HDROP).

#[cfg(windows)]
pub fn has_file_drop() -> bool {
    use windows::Win32::System::DataExchange::{
        CloseClipboard, IsClipboardFormatAvailable, OpenClipboard,
    };
    use windows::Win32::System::Ole::CF_HDROP;

    unsafe {
        if OpenClipboard(None).is_err() {
            return false;
        }
        let ok = IsClipboardFormatAvailable(u32::from(CF_HDROP.0)).is_ok();
        let _ = CloseClipboard();
        ok
    }
}

#[cfg(not(windows))]
pub fn has_file_drop() -> bool {
    false
}
