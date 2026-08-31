use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use windows::core::{Interface, PCWSTR};
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
use windows::Win32::UI::Shell::Common::ITEMIDLIST;
use windows::Win32::UI::Shell::{
    BHID_SFUIObject, IContextMenu, IContextMenu2, IContextMenu3, ILFree, IShellFolder, IShellItem,
    SHBindToParent, SHCreateItemFromParsingName, SHGetDesktopFolder, SHParseDisplayName,
    CMIC_MASK_PTINVOKE, CMINVOKECOMMANDINFOEX, SEE_MASK_UNICODE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreatePopupMenu, DestroyMenu, GetCursorPos, GetMenuItemCount, GetMenuItemInfoW, GetMenuStringW,
    GetSubMenu, SetForegroundWindow, TrackPopupMenuEx, HMENU, MENUITEMINFOW, MFT_SEPARATOR,
    MF_BYPOSITION, MIIM_BITMAP, MIIM_FTYPE, MIIM_ID, MIIM_STATE, MIIM_STRING, MIIM_SUBMENU,
    SW_SHOWNORMAL, TPM_RETURNCMD, TPM_RIGHTBUTTON, WM_INITMENUPOPUP,
};

use crate::desktop_organize::ShellMenuEntry;

use super::builtin::{desktop_blank_builtin_menu, fallback_menu};
use super::host::{pump_for, pump_messages};
use super::icons::hbitmap_to_data_url;
use super::ids::is_builtin_command;
use super::ids::{BUILTIN_DELETE, CMD_FIRST, CMD_LAST};
use super::pin::{apply_win11_pin_row, is_start_or_quick_access_menu_item, strip_start_and_quick_access_pins};
use super::util::{clean_menu_label, invoke_working_directory, menu_flags_for_path, stage, wide};
use super::verbs::{command_verb, shell_execute_verb};

const QUERY_MENU_TIMEOUT: Duration = Duration::from_secs(5);

unsafe fn command_menu_label(hmenu: HMENU, command_id: u32) -> String {
    use windows::Win32::UI::WindowsAndMessaging::MF_BYCOMMAND;
    let mut buf = [0u16; 512];
    let n = GetMenuStringW(hmenu, command_id, Some(&mut buf), MF_BYCOMMAND);
    if n > 0 {
        return clean_menu_label(&String::from_utf16_lossy(&buf[..n as usize]));
    }
    String::new()
}

unsafe fn invoke_owner_hwnd(hwnd: HWND) -> HWND {
    let dv = find_shell_defview();
    if !dv.0.is_null() {
        return dv;
    }
    if hwnd.0.is_null() {
        windows::Win32::UI::WindowsAndMessaging::GetDesktopWindow()
    } else {
        hwnd
    }
}

unsafe fn acquire_item_menu(hwnd: HWND, path: &str) -> Result<IContextMenu, String> {
    let wpath = wide(path);

    // Modern Shell Item path — preferred over hand-rolled DEFCONTEXTMENU.
    if let Ok(item) = SHCreateItemFromParsingName::<_, _, IShellItem>(PCWSTR(wpath.as_ptr()), None)
    {
        if let Ok(menu) = item
            .BindToHandler::<Option<&windows::Win32::System::Com::IBindCtx>, IContextMenu>(
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
    SHParseDisplayName(
        PCWSTR(wpath.as_ptr()),
        None,
        &mut pidl_abs,
        0,
        Some(&mut sfgao),
    )
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
        if let Ok(dv) = FindWindowExW(Some(progman), None, w!("SHELLDLL_DefView"), PCWSTR::null()) {
            if !dv.0.is_null() {
                return dv;
            }
        }
    }

    // Wallpaper WorkerW hosts DefView on some Windows builds.
    struct Search {
        found: HWND,
    }
    unsafe extern "system" fn enum_cb(
        hwnd: HWND,
        lparam: windows::Win32::Foundation::LPARAM,
    ) -> windows::core::BOOL {
        let search = &mut *(lparam.0 as *mut Search);
        if let Ok(dv) = FindWindowExW(Some(hwnd), None, w!("SHELLDLL_DefView"), PCWSTR::null()) {
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
    let _ = EnumWindows(
        Some(enum_cb),
        windows::Win32::Foundation::LPARAM(&mut search as *mut _ as isize),
    );
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
    pcm: &IContextMenu,
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
                pin: false,
                destructive: false,
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

        if children.is_none() {
            let verb = command_verb(pcm, mii.wID).unwrap_or_default();
            if is_start_or_quick_access_menu_item(&verb, &label) {
                continue;
            }
        }

        out.push(ShellMenuEntry {
            id: if children.is_some() { 0 } else { mii.wID },
            label: label.clone(),
            disabled,
            separator: false,
            icon: unsafe { hbitmap_to_data_url(mii.hbmpItem) },
            children,
            menu_path,
            pin: false,
            destructive: mii.wID == BUILTIN_DELETE || label.contains("删除"),
        });
    }
    strip_start_and_quick_access_pins(out)
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
        let hr = pcm.QueryContextMenu(hmenu, 0, CMD_FIRST, CMD_LAST, menu_flags_for_path(path));
        if hr.is_err() {
            let _ = DestroyMenu(hmenu);
            return Err(format!("QueryContextMenu: {hr:?}"));
        }
        stage("读取菜单项");
        let items = enumerate_hmenu(&pcm, hmenu, &[], 0);
        let items = apply_win11_pin_row(Some(&pcm), items);
        let items = strip_start_and_quick_access_pins(items);
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
        Ok(Ok(items)) if !items.is_empty() => {
            if path.is_none() {
                Ok(super::builtin::ensure_paste_entry(
                    super::builtin::ensure_blank_refresh_pin(items),
                ))
            } else {
                Ok(items)
            }
        }
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
        let hr = pcm.QueryContextMenu(
            hmenu,
            0,
            CMD_FIRST,
            CMD_LAST,
            menu_flags_for_path(path),
        );
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
        let hr = pcm.QueryContextMenu(
            hmenu,
            0,
            CMD_FIRST,
            CMD_LAST,
            menu_flags_for_path(path),
        );
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
            let label = command_menu_label(hmenu, command_id);
            if is_start_or_quick_access_menu_item(&verb, &label) {
                let _ = DestroyMenu(hmenu);
                return Err("已移除该菜单项".into());
            }
            let handled = match verb.as_str() {
                "open" | "openas" | "runas" | "properties" | "edit" | "print" => {
                    Some(shell_execute_verb(p, &verb))
                }
                "delete" => Some(super::verbs::delete_to_recycle_bin(p)),
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
        let owner = invoke_owner_hwnd(hwnd);
        let workdir = invoke_working_directory(path);
        let workdir_w = workdir.as_deref().map(wide);
        let mut fmask = CMIC_MASK_PTINVOKE;
        if workdir_w.is_some() {
            fmask |= SEE_MASK_UNICODE;
        }
        let ici = CMINVOKECOMMANDINFOEX {
            cbSize: std::mem::size_of::<CMINVOKECOMMANDINFOEX>() as u32,
            fMask: fmask,
            hwnd: owner,
            lpVerb: windows::core::PCSTR(verb_offset as *const u8),
            nShow: SW_SHOWNORMAL.0 as i32,
            lpDirectoryW: workdir_w
                .as_ref()
                .map(|v| PCWSTR(v.as_ptr()))
                .unwrap_or_default(),
            ptInvoke: pt,
            ..Default::default()
        };
        // Pump while invoking — some handlers expect a live message queue.
        pump_messages();
        let invoke_hr = pcm.InvokeCommand(&ici as *const _ as *const _);
        // Keep host alive briefly so "Open … here" child processes can detach.
        pump_for(1200);
        let _ = DestroyMenu(hmenu);
        // Keep directory buffer alive through InvokeCommand + settle.
        drop(workdir_w);
        drop(workdir);
        invoke_hr.map_err(|e| format!("InvokeCommand: {e}"))
    }
}

pub fn show_native_shell_context_menu(hwnd: HWND, path: Option<&str>) -> Result<(), String> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let pcm = acquire_menu(hwnd, path)?;
        let hmenu = CreatePopupMenu().map_err(|e| format!("CreatePopupMenu: {e}"))?;
        let hr = pcm.QueryContextMenu(
            hmenu,
            0,
            CMD_FIRST,
            CMD_LAST,
            menu_flags_for_path(path),
        );
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
