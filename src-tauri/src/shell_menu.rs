//! Typed Windows Shell context-menu helpers via the official `windows` crate.
//!
//! Keeps COM lifetimes, QueryInterface and PIDL handling out of hand-rolled
//! vtables. Still expected to run inside the isolated Shell-menu host process.

#![cfg(windows)]

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use windows::core::{Interface, PCWSTR};
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_SHIFT};
use windows::Win32::UI::Shell::Common::ITEMIDLIST;
use windows::Win32::UI::Shell::{
    BHID_SFUIObject, IContextMenu, IContextMenu2, IContextMenu3, IShellFolder, IShellItem, ILFree,
    SHBindToParent, SHCreateItemFromParsingName, SHGetDesktopFolder, SHParseDisplayName,
    CMF_EXTENDEDVERBS, CMF_NORMAL, CMINVOKECOMMANDINFOEX, CMIC_MASK_PTINVOKE, GCS_VERBA,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreatePopupMenu, CreateWindowExW, DestroyMenu, DestroyWindow, DispatchMessageW,
    GetCursorPos, GetMenuItemCount, GetMenuItemInfoW, GetMenuStringW, GetSubMenu,
    PeekMessageW, SetForegroundWindow, TrackPopupMenuEx, TranslateMessage, HMENU,
    MENUITEMINFOW, MF_BYPOSITION, MFT_SEPARATOR, MIIM_BITMAP,
    MIIM_FTYPE, MIIM_ID, MIIM_STATE, MIIM_STRING, MIIM_SUBMENU, MSG, PM_REMOVE, SW_SHOWNORMAL,
    TPM_RETURNCMD, TPM_RIGHTBUTTON, WM_INITMENUPOPUP, WM_QUIT, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_POPUP,
};

use crate::desktop_organize::ShellMenuEntry;

const CMD_FIRST: u32 = 1;
const CMD_LAST: u32 = 0x7fff;

/// Reserved IDs for built-in fallback when Shell extensions block QueryContextMenu.
pub const BUILTIN_CMD_BASE: u32 = 0xF000_0000;
pub const BUILTIN_OPEN: u32 = BUILTIN_CMD_BASE + 1;
pub const BUILTIN_SHOW_IN_FOLDER: u32 = BUILTIN_CMD_BASE + 2;
pub const BUILTIN_OPEN_WITH: u32 = BUILTIN_CMD_BASE + 3;
pub const BUILTIN_PROPERTIES: u32 = BUILTIN_CMD_BASE + 4;
pub const BUILTIN_OPEN_NEW_WINDOW: u32 = BUILTIN_CMD_BASE + 5;
pub const BUILTIN_PIN_QUICK_ACCESS: u32 = BUILTIN_CMD_BASE + 6;
pub const BUILTIN_CUT: u32 = BUILTIN_CMD_BASE + 7;
pub const BUILTIN_COPY: u32 = BUILTIN_CMD_BASE + 8;
pub const BUILTIN_CREATE_SHORTCUT: u32 = BUILTIN_CMD_BASE + 9;
pub const BUILTIN_DELETE: u32 = BUILTIN_CMD_BASE + 10;
pub const BUILTIN_RENAME: u32 = BUILTIN_CMD_BASE + 11;
pub const BUILTIN_COMPRESS_ZIP: u32 = BUILTIN_CMD_BASE + 12;
pub const BUILTIN_REFRESH: u32 = BUILTIN_CMD_BASE + 13;
pub const BUILTIN_NEW_FOLDER: u32 = BUILTIN_CMD_BASE + 14;
pub const BUILTIN_NEW_TXT: u32 = BUILTIN_CMD_BASE + 15;
pub const BUILTIN_OPEN_DESKTOP: u32 = BUILTIN_CMD_BASE + 16;
pub const BUILTIN_OPEN_TERMINAL: u32 = BUILTIN_CMD_BASE + 17;
pub const BUILTIN_DISPLAY_SETTINGS: u32 = BUILTIN_CMD_BASE + 18;
pub const BUILTIN_PERSONALIZE: u32 = BUILTIN_CMD_BASE + 19;

const QUERY_MENU_TIMEOUT: Duration = Duration::from_secs(5);

fn stage(s: &str) {
    crate::desktop_organize::shell_host_stage(s);
}

fn wide(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn menu_flags() -> u32 {
    let mut flags = CMF_NORMAL;
    unsafe {
        if (GetAsyncKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000) != 0 {
            flags |= CMF_EXTENDEDVERBS;
        }
    }
    flags
}

fn clean_menu_label(raw: &str) -> String {
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

unsafe fn acquire_item_menu(hwnd: HWND, path: &str) -> Result<IContextMenu, String> {
    let wpath = wide(path);

    // Modern Shell Item path — preferred over hand-rolled DEFCONTEXTMENU.
    if let Ok(item) =
        SHCreateItemFromParsingName::<_, _, IShellItem>(PCWSTR(wpath.as_ptr()), None)
    {
        if let Ok(menu) =
            item.BindToHandler::<Option<&windows::Win32::System::Com::IBindCtx>, IContextMenu>(
                None,
                &BHID_SFUIObject,
            )
        {
            return Ok(menu);
        }
    }

    // Fallback: parent IShellFolder::GetUIObjectOf.
    let mut pidl_abs: *mut ITEMIDLIST = std::ptr::null_mut();
    let mut sfgao = 0u32;
    SHParseDisplayName(PCWSTR(wpath.as_ptr()), None, &mut pidl_abs, 0, Some(&mut sfgao))
        .map_err(|e| format!("SHParseDisplayName: {e}"))?;
    if pidl_abs.is_null() {
        return Err("解析路径失败".into());
    }

    let mut pidl_child: *mut ITEMIDLIST = std::ptr::null_mut();
    let psf = match SHBindToParent::<IShellFolder>(pidl_abs, Some(&mut pidl_child)) {
        Ok(folder) => folder,
        Err(e) => {
            ILFree(Some(pidl_abs));
            return Err(format!("SHBindToParent: {e}"));
        }
    };
    let child = [pidl_child as *const ITEMIDLIST];
    let result = psf.GetUIObjectOf::<IContextMenu>(hwnd, &child, None);
    ILFree(Some(pidl_abs));
    result.map_err(|e| format!("GetUIObjectOf: {e}"))
}

unsafe fn find_shell_defview() -> HWND {
    use windows::core::w;
    use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, FindWindowExW, FindWindowW};

    let progman = FindWindowW(w!("Progman"), PCWSTR::null()).unwrap_or_default();
    if !progman.0.is_null() {
        if let Ok(dv) =
            FindWindowExW(Some(progman), None, w!("SHELLDLL_DefView"), PCWSTR::null())
        {
            if !dv.0.is_null() {
                return dv;
            }
        }
    }

    // Wallpaper WorkerW hosts DefView on some Windows builds.
    struct Search {
        found: HWND,
    }
    unsafe extern "system" fn enum_cb(hwnd: HWND, lparam: windows::Win32::Foundation::LPARAM) -> windows::core::BOOL {
        let search = &mut *(lparam.0 as *mut Search);
        if let Ok(dv) =
            FindWindowExW(Some(hwnd), None, w!("SHELLDLL_DefView"), PCWSTR::null())
        {
            if !dv.0.is_null() {
                search.found = dv;
                return false.into();
            }
        }
        true.into()
    }
    let mut search = Search {
        found: HWND::default(),
    };
    let _ = EnumWindows(Some(enum_cb), windows::Win32::Foundation::LPARAM(&mut search as *mut _ as isize));
    search.found
}

unsafe fn acquire_desktop_bg_menu(hwnd: HWND) -> Result<IContextMenu, String> {
    // Prefer real DefView as owner so the desktop background menu matches Explorer.
    let owner = {
        let dv = find_shell_defview();
        if dv.0.is_null() {
            hwnd
        } else {
            dv
        }
    };
    let desktop = SHGetDesktopFolder().map_err(|e| format!("SHGetDesktopFolder: {e}"))?;
    desktop
        .CreateViewObject::<IContextMenu>(owner)
        .map_err(|e| format!("CreateViewObject: {e}"))
}

unsafe fn acquire_menu(hwnd: HWND, path: Option<&str>) -> Result<IContextMenu, String> {
    match path {
        Some(p) => acquire_item_menu(hwnd, p),
        None => acquire_desktop_bg_menu(hwnd),
    }
}

unsafe fn init_submenu(pcm: &IContextMenu, submenu: HMENU, position: u32) {
    let lparam = windows::Win32::Foundation::LPARAM(position as isize);
    if let Ok(pcm3) = pcm.cast::<IContextMenu3>() {
        let mut result = windows::Win32::Foundation::LRESULT(0);
        let _ = pcm3.HandleMenuMsg2(
            WM_INITMENUPOPUP,
            windows::Win32::Foundation::WPARAM(submenu.0 as usize),
            lparam,
            Some(&mut result),
        );
        return;
    }
    if let Ok(pcm2) = pcm.cast::<IContextMenu2>() {
        let _ = pcm2.HandleMenuMsg(
            WM_INITMENUPOPUP,
            windows::Win32::Foundation::WPARAM(submenu.0 as usize),
            lparam,
        );
    }
}

unsafe fn enumerate_hmenu(
    _pcm: &IContextMenu,
    hmenu: HMENU,
    parent_path: &[u32],
    _depth: u32,
) -> Vec<ShellMenuEntry> {
    if hmenu.is_invalid() {
        return Vec::new();
    }
    let count = GetMenuItemCount(Some(hmenu));
    if count <= 0 {
        return Vec::new();
    }

    let mut out = Vec::with_capacity(count as usize);
    for i in 0..count {
        let mut text_buf = [0u16; 512];
        let mut mii = MENUITEMINFOW {
            cbSize: std::mem::size_of::<MENUITEMINFOW>() as u32,
            fMask: MIIM_BITMAP | MIIM_FTYPE | MIIM_ID | MIIM_STATE | MIIM_STRING | MIIM_SUBMENU,
            dwTypeData: windows::core::PWSTR(text_buf.as_mut_ptr()),
            cch: text_buf.len() as u32 - 1,
            ..Default::default()
        };
        if GetMenuItemInfoW(hmenu, i as u32, true, &mut mii).is_err() {
            continue;
        }
        if mii.fType.contains(MFT_SEPARATOR) {
            out.push(ShellMenuEntry {
                id: 0,
                label: String::new(),
                disabled: true,
                separator: true,
                icon: None,
                children: None,
                menu_path: parent_path.to_vec(),
            });
            continue;
        }

        let len = text_buf.iter().position(|&c| c == 0).unwrap_or(0);
        let mut label = clean_menu_label(&String::from_utf16_lossy(&text_buf[..len]));
        if label.is_empty() {
            let mut alt = [0u16; 512];
            let n = GetMenuStringW(hmenu, i as u32, Some(&mut alt), MF_BYPOSITION);
            if n > 0 {
                label = clean_menu_label(&String::from_utf16_lossy(&alt[..n as usize]));
            }
        }
        if label.is_empty() && mii.hSubMenu.is_invalid() {
            continue;
        }
        if label.is_empty() {
            label = "…".into();
        }

        let disabled = (mii.fState.0 & 0x3) != 0;
        let mut menu_path = parent_path.to_vec();
        // Cascade menus (打开方式 / 发送到 / …) must be filled via WM_INITMENUPOPUP.
        // Never eagerly recurse: a non-zero GetMenuItemCount often means a placeholder.
        let children = if !mii.hSubMenu.is_invalid() {
            menu_path.push(i as u32);
            Some(Vec::new())
        } else {
            None
        };

        out.push(ShellMenuEntry {
            id: if children.is_some() { 0 } else { mii.wID },
            label,
            disabled,
            separator: false,
            icon: None,
            children,
            menu_path,
        });
    }
    out
}

unsafe fn initialize_submenu_path(
    pcm: &IContextMenu,
    root: HMENU,
    menu_path: &[u32],
) -> Result<HMENU, String> {
    if menu_path.len() > 4 {
        return Err("菜单层级过深".into());
    }
    let mut current = root;
    for &position in menu_path {
        let submenu = GetSubMenu(current, position as i32);
        if submenu.is_invalid() {
            return Err("二级菜单已失效，请重新打开".into());
        }
        // Always send WM_INITMENUPOPUP — "打开方式" etc. keep a placeholder item
        // so GetMenuItemCount > 0 before init and would otherwise stay empty of real apps.
        init_submenu(pcm, submenu, position);
        pump_messages();
        current = submenu;
    }
    Ok(current)
}

fn item(id: u32, label: &str) -> ShellMenuEntry {
    ShellMenuEntry {
        id,
        label: label.into(),
        disabled: false,
        separator: false,
        icon: None,
        children: None,
        menu_path: Vec::new(),
    }
}

fn sep() -> ShellMenuEntry {
    ShellMenuEntry {
        id: 0,
        label: String::new(),
        disabled: true,
        separator: true,
        icon: None,
        children: None,
        menu_path: Vec::new(),
    }
}

/// Minimal fallback when Shell QueryContextMenu hangs (files).
pub fn fallback_menu(path: &str) -> Vec<ShellMenuEntry> {
    if std::path::Path::new(path).is_dir() {
        return folder_builtin_menu();
    }
    vec![
        item(BUILTIN_OPEN, "打开"),
        item(BUILTIN_OPEN_WITH, "打开方式"),
        item(BUILTIN_SHOW_IN_FOLDER, "在资源管理器中显示"),
        sep(),
        item(BUILTIN_PROPERTIES, "属性"),
    ]
}

/// Built-in folder menu replicating common Windows Explorer folder items.
/// Used instead of QueryContextMenu (folder Shell extensions often hang).
pub fn folder_builtin_menu() -> Vec<ShellMenuEntry> {
    vec![
        item(BUILTIN_OPEN, "打开"),
        item(BUILTIN_OPEN_NEW_WINDOW, "在新窗口中打开"),
        sep(),
        item(BUILTIN_PIN_QUICK_ACCESS, "固定到「快速访问」"),
        sep(),
        item(BUILTIN_CUT, "剪切"),
        item(BUILTIN_COPY, "复制"),
        item(BUILTIN_CREATE_SHORTCUT, "创建快捷方式"),
        sep(),
        item(BUILTIN_DELETE, "删除"),
        item(BUILTIN_RENAME, "重命名"),
        sep(),
        item(BUILTIN_COMPRESS_ZIP, "压缩为 ZIP 文件"),
        sep(),
        item(BUILTIN_PROPERTIES, "属性"),
    ]
}

/// Blank desktop / fence background menu (Explorer-like, no Shell hang).
pub fn desktop_blank_builtin_menu() -> Vec<ShellMenuEntry> {
    vec![
        item(BUILTIN_REFRESH, "刷新"),
        sep(),
        ShellMenuEntry {
            id: 0,
            label: "新建".into(),
            disabled: false,
            separator: false,
            icon: None,
            children: Some(vec![
                item(BUILTIN_NEW_FOLDER, "文件夹"),
                item(BUILTIN_NEW_TXT, "文本文档"),
            ]),
            menu_path: Vec::new(),
        },
        sep(),
        item(BUILTIN_OPEN_DESKTOP, "打开桌面文件夹"),
        item(BUILTIN_OPEN_TERMINAL, "在终端中打开"),
        sep(),
        item(BUILTIN_DISPLAY_SETTINGS, "显示设置"),
        item(BUILTIN_PERSONALIZE, "个性化"),
    ]
}

fn list_shell_context_menu_inner(
    hwnd: HWND,
    path: Option<&str>,
) -> Result<Vec<ShellMenuEntry>, String> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        stage("获取 IContextMenu");
        let pcm = acquire_menu(hwnd, path)?;
        stage("创建菜单句柄");
        let hmenu = CreatePopupMenu().map_err(|e| format!("CreatePopupMenu: {e}"))?;
        stage("QueryContextMenu");
        let hr = pcm.QueryContextMenu(hmenu, 0, CMD_FIRST, CMD_LAST, menu_flags());
        if hr.is_err() {
            let _ = DestroyMenu(hmenu);
            return Err(format!("QueryContextMenu: {hr:?}"));
        }
        stage("读取菜单项");
        let items = enumerate_hmenu(&pcm, hmenu, &[], 0);
        stage("完成");
        let _ = DestroyMenu(hmenu);
        Ok(items)
    }
}

pub fn list_shell_context_menu(
    hwnd: HWND,
    path: Option<&str>,
) -> Result<Vec<ShellMenuEntry>, String> {
    let path_owned = path.map(|s| s.to_string());
    let hwnd_raw = hwnd.0 as isize;
    let (tx, rx) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let hwnd = HWND(hwnd_raw as *mut _);
        let result = list_shell_context_menu_inner(hwnd, path_owned.as_deref());
        let _ = tx.send(result);
    });
    match rx.recv_timeout(QUERY_MENU_TIMEOUT) {
        Ok(Ok(items)) if !items.is_empty() => Ok(items),
        Ok(Ok(_)) | Ok(Err(_)) | Err(mpsc::RecvTimeoutError::Timeout) => {
            stage("QueryContextMenu 超时，使用内置菜单");
            if let Some(p) = path {
                Ok(fallback_menu(p))
            } else {
                Ok(desktop_blank_builtin_menu())
            }
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => Err("菜单加载线程异常退出".into()),
    }
}

pub fn list_shell_context_submenu(
    hwnd: HWND,
    path: Option<&str>,
    menu_path: &[u32],
) -> Result<Vec<ShellMenuEntry>, String> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        stage("二级菜单：获取 IContextMenu");
        let pcm = acquire_menu(hwnd, path)?;
        let hmenu = CreatePopupMenu().map_err(|e| format!("CreatePopupMenu: {e}"))?;
        stage("二级菜单：QueryContextMenu");
        let hr = pcm.QueryContextMenu(hmenu, 0, CMD_FIRST, CMD_LAST, menu_flags());
        if hr.is_err() {
            let _ = DestroyMenu(hmenu);
            return Err(format!("QueryContextMenu submenu: {hr:?}"));
        }
        stage("二级菜单：初始化");
        let result = (|| {
            let submenu = initialize_submenu_path(&pcm, hmenu, menu_path)?;
            stage("二级菜单：读取菜单项");
            Ok(enumerate_hmenu(&pcm, submenu, menu_path, 0))
        })();
        let _ = DestroyMenu(hmenu);
        result
    }
}

pub fn is_builtin_command(command_id: u32) -> bool {
    command_id >= BUILTIN_CMD_BASE
}

fn command_verb(pcm: &IContextMenu, command_id: u32) -> Option<String> {
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

fn shell_execute_verb(path: &str, verb: &str) -> Result<(), String> {
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
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

pub fn invoke_shell_context_command(
    hwnd: HWND,
    path: Option<&str>,
    command_id: u32,
    menu_path: &[u32],
) -> Result<(), String> {
    if is_builtin_command(command_id) {
        return Err("内置命令应在上层处理".into());
    }
    if command_id < CMD_FIRST {
        return Err("无效命令".into());
    }
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let pcm = acquire_menu(hwnd, path)?;
        let hmenu = CreatePopupMenu().map_err(|e| format!("CreatePopupMenu: {e}"))?;
        let hr = pcm.QueryContextMenu(hmenu, 0, CMD_FIRST, CMD_LAST, menu_flags());
        if hr.is_err() {
            let _ = DestroyMenu(hmenu);
            return Err(format!("QueryContextMenu: {hr:?}"));
        }
        if let Err(e) = initialize_submenu_path(&pcm, hmenu, menu_path) {
            let _ = DestroyMenu(hmenu);
            return Err(e);
        }

        // Prefer reliable ShellExecute for common verbs — InvokeCommand often fails
        // outside Explorer (no IContextMenuSite / wrong HWND / no message pump).
        if let (Some(p), Some(verb)) = (path, command_verb(&pcm, command_id)) {
            let handled = match verb.as_str() {
                "open" | "openas" | "runas" | "properties" | "edit" | "print" => {
                    Some(shell_execute_verb(p, &verb))
                }
                "delete" => Some(
                    trash::delete(p).map_err(|e| format!("删除失败: {e}")),
                ),
                "cut" => Some(Err("BUILTIN_CUT".into())),
                "copy" => Some(Err("BUILTIN_COPY".into())),
                "link" => Some(Err("BUILTIN_LINK".into())),
                "rename" => Some(Err("BUILTIN_RENAME".into())),
                _ => None,
            };
            if let Some(result) = handled {
                let _ = DestroyMenu(hmenu);
                return result;
            }
        }

        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);
        let verb_offset = (command_id - CMD_FIRST) as usize;
        let owner = if hwnd.0.is_null() {
            windows::Win32::UI::WindowsAndMessaging::GetDesktopWindow()
        } else {
            hwnd
        };
        let ici = CMINVOKECOMMANDINFOEX {
            cbSize: std::mem::size_of::<CMINVOKECOMMANDINFOEX>() as u32,
            fMask: CMIC_MASK_PTINVOKE,
            hwnd: owner,
            lpVerb: windows::core::PCSTR(verb_offset as *const u8),
            nShow: SW_SHOWNORMAL.0 as i32,
            ptInvoke: pt,
            ..Default::default()
        };
        // Pump while invoking — some handlers expect a live message queue.
        pump_messages();
        let invoke_hr = pcm.InvokeCommand(&ici as *const _ as *const _);
        pump_messages();
        let _ = DestroyMenu(hmenu);
        invoke_hr.map_err(|e| format!("InvokeCommand: {e}"))
    }
}

pub fn show_native_shell_context_menu(
    hwnd: HWND,
    path: Option<&str>,
) -> Result<(), String> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let pcm = acquire_menu(hwnd, path)?;
        let hmenu = CreatePopupMenu().map_err(|e| format!("CreatePopupMenu: {e}"))?;
        let hr = pcm.QueryContextMenu(hmenu, 0, CMD_FIRST, CMD_LAST, menu_flags());
        if hr.is_err() {
            let _ = DestroyMenu(hmenu);
            return Err(format!("QueryContextMenu: {hr:?}"));
        }

        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);
        let _ = SetForegroundWindow(hwnd);
        let flags = TPM_RETURNCMD.0 | TPM_RIGHTBUTTON.0;
        let command_id = TrackPopupMenuEx(hmenu, flags, pt.x, pt.y, hwnd, None).0 as u32;

        let invoke_result = if command_id >= CMD_FIRST {
            let verb_offset = (command_id - CMD_FIRST) as usize;
            let ici = CMINVOKECOMMANDINFOEX {
                cbSize: std::mem::size_of::<CMINVOKECOMMANDINFOEX>() as u32,
                fMask: CMIC_MASK_PTINVOKE,
                hwnd,
                lpVerb: windows::core::PCSTR(verb_offset as *const u8),
                nShow: SW_SHOWNORMAL.0 as i32,
                ptInvoke: pt,
                ..Default::default()
            };
            pcm.InvokeCommand(&ici as *const _ as *const _)
        } else {
            Ok(())
        };
        let _ = DestroyMenu(hmenu);
        invoke_result.map_err(|e| format!("InvokeCommand: {e}"))
    }
}
