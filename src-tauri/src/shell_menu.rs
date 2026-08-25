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
    BHID_SFUIObject, IContextMenu, IContextMenu2, IContextMenu3, IShellFolder, IShellItem,
    SHBindToParent, SHCreateItemFromParsingName, SHGetDesktopFolder, SHParseDisplayName, ILFree,
    CMF_EXTENDEDVERBS, CMF_NORMAL, CMINVOKECOMMANDINFOEX, CMIC_MASK_PTINVOKE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreatePopupMenu, CreateWindowExW, DestroyMenu, DestroyWindow, DispatchMessageW,
    GetCursorPos, GetMenuItemCount, GetMenuItemInfoW, GetMenuStringW, GetSubMenu,
    PeekMessageW, SetForegroundWindow, TrackPopupMenuEx, TranslateMessage, HMENU,
    MENUITEMINFOW, MF_BYPOSITION, MFS_DISABLED, MFS_GRAYED, MFT_SEPARATOR, MIIM_BITMAP,
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

unsafe fn acquire_desktop_bg_menu(hwnd: HWND) -> Result<IContextMenu, String> {
    let desktop = SHGetDesktopFolder().map_err(|e| format!("SHGetDesktopFolder: {e}"))?;
    desktop
        .CreateViewObject::<IContextMenu>(hwnd)
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
    pcm: &IContextMenu,
    hmenu: HMENU,
    parent_path: &[u32],
    depth: u32,
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

        let disabled = mii.fState.contains(MFS_DISABLED) || mii.fState.contains(MFS_GRAYED);
        let mut menu_path = parent_path.to_vec();
        let children = if !mii.hSubMenu.is_invalid() {
            menu_path.push(i as u32);
            if depth < 4 && GetMenuItemCount(Some(mii.hSubMenu)) > 0 {
                Some(enumerate_hmenu(pcm, mii.hSubMenu, &menu_path, depth + 1))
            } else {
                Some(Vec::new())
            }
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
        if GetMenuItemCount(Some(submenu)) <= 0 {
            init_submenu(pcm, submenu, position);
        }
        current = submenu;
    }
    Ok(current)
}

fn fallback_menu(path: &str) -> Vec<ShellMenuEntry> {
    let is_dir = std::path::Path::new(path).is_dir();
    let mut items = vec![ShellMenuEntry {
        id: BUILTIN_OPEN,
        label: "打开".into(),
        disabled: false,
        separator: false,
        icon: None,
        children: None,
        menu_path: Vec::new(),
    }];
    if !is_dir {
        items.push(ShellMenuEntry {
            id: BUILTIN_OPEN_WITH,
            label: "打开方式".into(),
            disabled: false,
            separator: false,
            icon: None,
            children: None,
            menu_path: Vec::new(),
        });
    }
    items.push(ShellMenuEntry {
        id: BUILTIN_SHOW_IN_FOLDER,
        label: "在资源管理器中显示".into(),
        disabled: false,
        separator: false,
        icon: None,
        children: None,
        menu_path: Vec::new(),
    });
    items.push(ShellMenuEntry {
        id: 0,
        label: String::new(),
        disabled: true,
        separator: true,
        icon: None,
        children: None,
        menu_path: Vec::new(),
    });
    items.push(ShellMenuEntry {
        id: BUILTIN_PROPERTIES,
        label: "属性".into(),
        disabled: false,
        separator: false,
        icon: None,
        children: None,
        menu_path: Vec::new(),
    });
    items
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
            if let Some(p) = path {
                stage("QueryContextMenu 超时，使用内置菜单");
                Ok(fallback_menu(p))
            } else {
                Err("桌面背景菜单加载超时".into())
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

        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);
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
        let invoke_hr = pcm.InvokeCommand(&ici as *const _ as *const _);
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
