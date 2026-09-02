use windows::Win32::UI::Shell::{
    IContextMenu, GCS_VERBA, ShellExecuteExW, ShellExecuteW, SHFileOperationW, FO_DELETE,
    FOF_ALLOWUNDO, FOF_WANTNUKEWARNING, SEE_MASK_ASYNCOK, SEE_MASK_FLAG_NO_UI, SHELLEXECUTEINFOW,
    SHFILEOPSTRUCTW,
};
use windows::core::PCWSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

use super::ids::CMD_FIRST;
use super::util::wide;

pub(crate) fn command_verb(pcm: &IContextMenu, command_id: u32) -> Option<String> {
    if command_id < CMD_FIRST {
        return None;
    }
    let offset = (command_id - CMD_FIRST) as usize;
    let mut buf = [0u8; 128];
    unsafe {
        if pcm
            .GetCommandString(
                offset,
                GCS_VERBA,
                None,
                windows::core::PSTR(buf.as_mut_ptr()),
                buf.len() as u32,
            )
            .is_ok()
        {
            let len = buf.iter().position(|&c| c == 0).unwrap_or(0);
            if len > 0 {
                return Some(String::from_utf8_lossy(&buf[..len]).to_ascii_lowercase());
            }
        }
    }
    None
}

pub(crate) fn shell_execute_verb(path: &str, verb: &str) -> Result<(), String> {
    // Properties: helper process owns the dialog (host exits right after invoke).
    if verb.eq_ignore_ascii_case("properties") {
        std::process::Command::new("rundll32")
            .arg("shell32.dll,ShellExec_RunDLL")
            .arg("properties")
            .arg(path)
            .spawn()
            .map_err(|e| format!("打开属性失败: {e}"))?;
        return Ok(());
    }

    // Prefer async ShellExecuteEx so the menu host can exit without waiting.
    let wpath = wide(path);
    let wverb = wide(verb);
    unsafe {
        let mut info = SHELLEXECUTEINFOW {
            cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
            fMask: SEE_MASK_ASYNCOK | SEE_MASK_FLAG_NO_UI,
            lpVerb: PCWSTR(wverb.as_ptr()),
            lpFile: PCWSTR(wpath.as_ptr()),
            nShow: SW_SHOWNORMAL.0 as i32,
            ..Default::default()
        };
        if ShellExecuteExW(&mut info).is_ok() {
            return Ok(());
        }
        let ret = ShellExecuteW(
            None,
            PCWSTR(wverb.as_ptr()),
            PCWSTR(wpath.as_ptr()),
            None,
            None,
            SW_SHOWNORMAL,
        );
        if (ret.0 as isize) <= 32 {
            return Err(format!("执行“{verb}”失败: code={}", ret.0 as isize));
        }
    }
    Ok(())
}

/// Explorer-style delete: move to Recycle Bin with the system confirmation UI.
pub fn delete_to_recycle_bin(path: &str) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("路径为空".into());
    }
    if trimmed.starts_with("::") {
        return Err("系统图标不支持删除".into());
    }
    // SHFileOperation requires a double-null-terminated list.
    let mut from: Vec<u16> = trimmed.encode_utf16().chain([0u16, 0u16]).collect();
    let mut op = SHFILEOPSTRUCTW {
        hwnd: HWND::default(),
        wFunc: FO_DELETE,
        pFrom: PCWSTR(from.as_mut_ptr()),
        pTo: PCWSTR::null(),
        // ALLOWUNDO → Recycle Bin; no FOF_NOCONFIRMATION → system confirm dialog.
        fFlags: FOF_ALLOWUNDO.0 as u16 | FOF_WANTNUKEWARNING.0 as u16,
        fAnyOperationsAborted: false.into(),
        hNameMappings: std::ptr::null_mut(),
        lpszProgressTitle: PCWSTR::null(),
    };
    let code = unsafe { SHFileOperationW(&mut op) };
    if op.fAnyOperationsAborted.as_bool() {
        return Err("已取消".into());
    }
    if code != 0 {
        return Err(format!("删除失败: code={code}"));
    }
    Ok(())
}
