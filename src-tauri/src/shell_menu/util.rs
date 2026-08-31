use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_SHIFT};
use windows::Win32::UI::Shell::{CMF_EXTENDEDVERBS, CMF_ITEMMENU, CMF_NORMAL};

pub fn stage(s: &str) {
    crate::desktop_organize::shell_host_stage(s);
}

pub fn wide(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Working directory for InvokeCommand — required by "Open … here" Shell extensions.
pub fn invoke_working_directory(path: Option<&str>) -> Option<String> {
    match path {
        None => known_folders::get_known_folder_path(known_folders::KnownFolder::Desktop)
            .map(|p| p.to_string_lossy().into_owned()),
        Some(p) => {
            let pb = std::path::Path::new(p);
            if pb.is_dir() {
                Some(p.to_string())
            } else {
                pb.parent()
                    .filter(|parent| !parent.as_os_str().is_empty())
                    .map(|parent| parent.to_string_lossy().into_owned())
            }
        }
    }
}

/// CMF flags for QueryContextMenu.
/// - 回收站 / 网络: CMF_ITEMMENU only (narrowest for these)
/// - other `::{CLSID}` (e.g. 此电脑): CMF_NORMAL | CMF_ITEMMENU
/// - files: CMF_NORMAL (+ CMF_EXTENDEDVERBS when Shift held)
pub fn menu_flags_for_path(path: Option<&str>) -> u32 {
    if let Some(p) = path {
        let trimmed = p.trim_start();
        if trimmed.starts_with("::") {
            let upper = trimmed.to_ascii_uppercase();
            // 回收站 / 网络
            if upper.contains("645FF040-5081-101B-9F08-00AA002F954E")
                || upper.contains("F02C1A0D-B21F-4110-8426-0A0C959C3602")
            {
                return CMF_ITEMMENU;
            }
            return CMF_NORMAL | CMF_ITEMMENU;
        }
    }
    let mut flags = CMF_NORMAL;
    unsafe {
        if (GetAsyncKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000) != 0 {
            flags |= CMF_EXTENDEDVERBS;
        }
    }
    flags
}

pub fn clean_menu_label(raw: &str) -> String {
    let s = raw.split('\t').next().unwrap_or(raw);
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '&' {
            if chars.peek() == Some(&'&') {
                out.push('&');
                chars.next();
            }
            continue;
        }
        out.push(c);
    }
    out.trim().to_string()
}
