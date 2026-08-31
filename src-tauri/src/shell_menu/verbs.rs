use windows::Win32::UI::Shell::{IContextMenu, GCS_VERBA, ShellExecuteW};
use windows::core::PCWSTR;
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
    let wpath = wide(path);
    let wverb = wide(verb);
    unsafe {
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
