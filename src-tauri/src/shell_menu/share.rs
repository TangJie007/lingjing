//! Native Share in OUR process with a real top-level owner HWND.
//! PowerShell `FolderItem.DoIt()` fails with ERROR_INVALID_WINDOW_HANDLE because
//! powershell.exe has no suitable share owner — do not use that path.

use std::ffi::OsString;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use windows::core::{factory, Interface, PCSTR, PCWSTR, HSTRING};
use windows::ApplicationModel::DataTransfer::{
    DataRequest, DataRequestedEventArgs, DataTransferManager,
};
use windows::Foundation::TypedEventHandler;
use windows::Storage::{IStorageItem, StorageFile, StorageFolder};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::HBRUSH;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::Common::ITEMIDLIST;
use windows::Win32::UI::Shell::{
    BHID_SFUIObject, IContextMenu, IShellFolder, IShellItem, ILFree, SHBindToParent,
    SHCreateItemFromParsingName, SHParseDisplayName, CMINVOKECOMMANDINFOEX, CMIC_MASK_PTINVOKE,
    IDataTransferManagerInterop,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DestroyWindow, GetCursorPos,
    GetMenuItemCount, GetMenuItemInfoW, GetMenuStringW, RegisterClassExW, SetForegroundWindow,
    ShowWindow, CS_HREDRAW, CS_VREDRAW, MENUITEMINFOW, MF_BYPOSITION, MFT_SEPARATOR, MIIM_FTYPE,
    MIIM_ID, SW_SHOW, WINDOW_EX_STYLE, WNDCLASSEXW, WS_EX_TOOLWINDOW, WS_OVERLAPPEDWINDOW,
    WM_DESTROY, HMENU,
};
use windows_collections::IIterable;

use super::host::{pump_for, pump_messages};
use super::ids::{CMD_FIRST, CMD_LAST};
use super::util::{clean_menu_label, menu_flags, wide};
use super::verbs::command_verb;

static SHARE_CLASS: OnceLock<Vec<u16>> = OnceLock::new();
static OWNED_HWND: Mutex<Option<isize>> = Mutex::new(None);

/// Must run on the thread that owns the temporary top-level window.
pub fn share_path_native(_owner_hwnd_raw: isize, path: &str) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("路径为空".into());
    }
    if !Path::new(trimmed).exists() {
        return Err("目标不存在".into());
    }

    // Use a real, activatable top-level owner. The Fence HWND is a child of
    // SHELLDLL_DefView, hidden Tauri windows are not reliable share owners, and
    // borderless WS_POPUP owners are accepted initially by some Windows 11
    // builds but immediately canceled.
    let hwnd = create_owner_window()?;
    let hwnd_raw = hwnd.0 as isize;
    if let Ok(mut g) = OWNED_HWND.lock() {
        if let Some(old) = g.replace(hwnd_raw) {
            destroy_hwnd(HWND(old as _));
        }
    }
    // Keep fallback owner alive for the share sheet lifetime.
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(180));
        finish_owner(hwnd_raw);
    });

    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
    }

    // 1) WinRT Share UI (works for normal files StorageFile can open).
    match share_via_dtm(hwnd, trimmed) {
        Ok(()) => {
            tracing::info!("[share] DataTransferManager UI shown path={trimmed}");
            return Ok(());
        }
        Err(e) if e == "已取消共享" => {
            finish_owner(hwnd_raw);
            return Err(e);
        }
        Err(e) => tracing::warn!("[share] DataTransferManager failed: {e}"),
    }

    // 2) In-process Shell InvokeCommand with OUR owner HWND (not PowerShell).
    let result = share_via_shell_invoke(hwnd, trimmed).map(|_| {
        tracing::info!("[share] Shell InvokeCommand ok path={trimmed}");
        unsafe { pump_for(1500) };
    });
    if let Err(e) = &result {
        tracing::error!("[share] Shell InvokeCommand failed: {e}");
        finish_owner(hwnd_raw);
    }
    result
}

fn share_via_dtm(hwnd: HWND, path: &str) -> Result<(), String> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    }
    let supported = DataTransferManager::IsSupported().map_err(|e| e.to_string())?;
    if !supported {
        return Err("系统不支持 DataTransferManager".into());
    }

    let title = Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("共享")
        .to_string();

    let interop: IDataTransferManagerInterop =
        factory::<DataTransferManager, IDataTransferManagerInterop>().map_err(|e| e.to_string())?;
    let manager: DataTransferManager =
        unsafe { interop.GetForWindow(hwnd) }.map_err(|e| format!("GetForWindow: {e}"))?;

    let (tx_data, rx_data) = mpsc::sync_channel::<Result<(), String>>(1);
    let (tx_done, rx_done) = mpsc::sync_channel::<bool>(1);
    let title_h = HSTRING::from(title.as_str());
    let path_h = path.to_string();

    let handler = TypedEventHandler::new(
        move |_s: windows::core::Ref<'_, DataTransferManager>,
              args: windows::core::Ref<'_, DataRequestedEventArgs>| {
            let result = (|| -> Result<(), String> {
                tracing::info!("[share] DataRequested");
                let args = args.as_ref().ok_or_else(|| "DataRequested args null".to_string())?;
                let request = args.Request().map_err(|e| e.to_string())?;
                fill_request(&title_h, &path_h, &request).map_err(|e| e.to_string())?;
                let data = request.Data().map_err(|e| e.to_string())?;
                let txc = tx_done.clone();
                let _ = data.ShareCompleted(&TypedEventHandler::new(move |_, _| {
                    let _ = txc.send(true);
                    Ok(())
                }));
                let txc2 = tx_done.clone();
                let _ = data.ShareCanceled(&TypedEventHandler::new(move |_, _| {
                    tracing::info!("[share] ShareCanceled");
                    let _ = txc2.send(false);
                    Ok(())
                }));
                Ok(())
            })();
            let _ = tx_data.send(result);
            Ok(())
        },
    );

    let token = manager
        .DataRequested(&handler)
        .map_err(|e| format!("DataRequested register: {e}"))?;

    if let Err(e) = unsafe { interop.ShowShareUIForWindow(hwnd) } {
        let _ = manager.RemoveDataRequested(token);
        return Err(format!("ShowShareUIForWindow: {e}"));
    }

    let setup = rx_data
        .recv_timeout(Duration::from_secs(10))
        .map_err(|e| format!("等待 DataRequested 失败: {e}"))?;
    if let Err(e) = setup {
        let _ = manager.RemoveDataRequested(token);
        return Err(e);
    }

    // Keep the manager, token and handler alive for the complete share session.
    let completed = rx_done
        .recv()
        .map_err(|e| format!("等待共享结果失败: {e}"))?;
    let _ = manager.RemoveDataRequested(token);
    drop(handler);
    if completed {
        Ok(())
    } else {
        Err("已取消共享".into())
    }
}

fn share_via_shell_invoke(hwnd: HWND, path: &str) -> Result<(), String> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let pcm = acquire_item_menu(hwnd, path)?;
        let hmenu = CreatePopupMenu().map_err(|e| format!("CreatePopupMenu: {e}"))?;
        let hr = pcm.QueryContextMenu(hmenu, 0, CMD_FIRST, CMD_LAST, menu_flags());
        if hr.is_err() {
            let _ = DestroyMenu(hmenu);
            return Err(format!("QueryContextMenu: {hr:?}"));
        }
        let Some(command_id) = find_share_id(&pcm, hmenu) else {
            let _ = DestroyMenu(hmenu);
            return Err("菜单中未找到共享命令".into());
        };
        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);
        let verb_offset = (command_id - CMD_FIRST) as usize;
        let ici = CMINVOKECOMMANDINFOEX {
            cbSize: std::mem::size_of::<CMINVOKECOMMANDINFOEX>() as u32,
            fMask: CMIC_MASK_PTINVOKE,
            hwnd,
            lpVerb: PCSTR(verb_offset as *const u8),
            nShow: SW_SHOW.0 as i32,
            ptInvoke: pt,
            ..Default::default()
        };
        pump_messages();
        let invoke_hr = pcm.InvokeCommand(&ici as *const _ as *const _);
        let _ = DestroyMenu(hmenu);
        invoke_hr.map_err(|e| format!("InvokeCommand: {e}"))
    }
}

unsafe fn find_share_id(pcm: &IContextMenu, hmenu: HMENU) -> Option<u32> {
    let count = GetMenuItemCount(Some(hmenu));
    let mut exact = None;
    let mut fuzzy = None;
    for i in 0..count {
        let mut info = MENUITEMINFOW {
            cbSize: std::mem::size_of::<MENUITEMINFOW>() as u32,
            fMask: MIIM_FTYPE | MIIM_ID,
            ..Default::default()
        };
        if GetMenuItemInfoW(hmenu, i as u32, true, &mut info).is_err() {
            continue;
        }
        if info.fType == MFT_SEPARATOR || info.wID < CMD_FIRST {
            continue;
        }
        let verb = command_verb(pcm, info.wID).unwrap_or_default();
        let mut buf = vec![0u16; 256];
        let len = GetMenuStringW(hmenu, i as u32, Some(&mut buf), MF_BYPOSITION);
        let label = if len > 0 {
            clean_menu_label(&String::from_utf16_lossy(&buf[..len as usize]))
        } else {
            String::new()
        };
        let label_n = label
            .trim()
            .trim_end_matches(|c: char| c == ')' || c.is_ascii_alphanumeric() || c == '(')
            .trim()
            .to_string();
        // "共享(H)" / "共享"
        let is_exact = label_n == "共享" || label_n == "分享" || label.eq_ignore_ascii_case("Share");
        let is_fuzzy = (label.contains("共享") || label.contains("Share") || verb.contains("share"))
            && !label.contains("授予访问")
            && !label.contains("高级共享");
        if is_exact {
            exact = Some(info.wID);
        } else if is_fuzzy && fuzzy.is_none() {
            fuzzy = Some(info.wID);
        }
    }
    exact.or(fuzzy)
}

unsafe fn acquire_item_menu(hwnd: HWND, path: &str) -> Result<IContextMenu, String> {
    let wpath = wide(path);
    if let Ok(item) =
        SHCreateItemFromParsingName::<_, _, IShellItem>(PCWSTR(wpath.as_ptr()), None)
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
    let mut pidl_abs: *mut ITEMIDLIST = std::ptr::null_mut();
    let mut sfgao = 0u32;
    SHParseDisplayName(PCWSTR(wpath.as_ptr()), None, &mut pidl_abs, 0, Some(&mut sfgao))
        .map_err(|e| format!("SHParseDisplayName: {e}"))?;
    if pidl_abs.is_null() {
        return Err("解析路径失败".into());
    }
    let mut pidl_child: *mut ITEMIDLIST = std::ptr::null_mut();
    let psf = match SHBindToParent::<IShellFolder>(pidl_abs, Some(&mut pidl_child)) {
        Ok(f) => f,
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

fn fill_request(
    title: &HSTRING,
    path: &str,
    request: &DataRequest,
) -> windows::core::Result<()> {
    let data = request.Data()?;
    let properties = data.Properties()?;
    properties.SetTitle(title)?;
    properties.SetDescription(title)?;
    let items = resolve_storage_items(path)?
        .into_iter()
        .map(Some)
        .collect::<Vec<Option<IStorageItem>>>();
    let items: IIterable<IStorageItem> = items.into();
    data.SetStorageItemsReadOnly(&items)?;
    Ok(())
}

fn resolve_storage_items(path: &str) -> windows::core::Result<Vec<IStorageItem>> {
    let path = std::fs::canonicalize(path)?;
    let path = strip_verbatim_prefix(&path);
    let hpath = HSTRING::from(path.as_path());
    let item: IStorageItem = if path.is_dir() {
        StorageFolder::GetFolderFromPathAsync(&hpath)?.get()?.cast()?
    } else {
        StorageFile::GetFileFromPathAsync(&hpath)?.get()?.cast()?
    };
    Ok(vec![item])
}

fn strip_verbatim_prefix(path: &Path) -> PathBuf {
    const VERBATIM: &str = r"\\?\";
    const VERBATIM_UNC: &str = r"\\?\UNC\";
    let wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    let verbatim: Vec<u16> = VERBATIM.encode_utf16().collect();
    let verbatim_unc: Vec<u16> = VERBATIM_UNC.encode_utf16().collect();
    if wide.starts_with(&verbatim_unc) {
        let mut rebuilt: Vec<u16> = r"\\".encode_utf16().collect();
        rebuilt.extend_from_slice(&wide[verbatim_unc.len()..]);
        PathBuf::from(OsString::from_wide(&rebuilt))
    } else if wide.starts_with(&verbatim) {
        PathBuf::from(OsString::from_wide(&wide[verbatim.len()..]))
    } else {
        path.to_path_buf()
    }
}

fn ensure_class() -> Result<&'static [u16], String> {
    if let Some(v) = SHARE_CLASS.get() {
        return Ok(v.as_slice());
    }
    let class_name = wide("LingScapeShareOwner");
    unsafe {
        let hinstance = GetModuleHandleW(None).map_err(|e| e.to_string())?;
        let wnd = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(share_wnd_proc),
            hInstance: HINSTANCE(hinstance.0),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            hbrBackground: HBRUSH(std::ptr::null_mut()),
            ..Default::default()
        };
        let atom = RegisterClassExW(&wnd);
        if atom == 0 {
            let err = windows::core::Error::from_win32();
            if err.code().0 as u32 & 0xFFFF != 1410 {
                return Err(format!("RegisterClassExW: {err}"));
            }
        }
    }
    let _ = SHARE_CLASS.set(class_name);
    SHARE_CLASS
        .get()
        .map(|v| v.as_slice())
        .ok_or_else(|| "share class init failed".into())
}

unsafe extern "system" fn share_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_DESTROY {
        return LRESULT(0);
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

fn create_owner_window() -> Result<HWND, String> {
    let class = ensure_class()?;
    unsafe {
        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);
        let title = wide("LingScape Share");
        CreateWindowExW(
            WINDOW_EX_STYLE(WS_EX_TOOLWINDOW.0),
            PCWSTR(class.as_ptr()),
            PCWSTR(title.as_ptr()),
            WS_OVERLAPPEDWINDOW,
            pt.x.saturating_sub(16),
            pt.y.saturating_sub(16),
            160,
            80,
            None,
            None,
            None,
            None,
        )
        .map_err(|e| format!("创建共享窗口失败: {e}"))
    }
}

fn destroy_hwnd(hwnd: HWND) {
    if !hwnd.0.is_null() {
        unsafe {
            let _ = DestroyWindow(hwnd);
        }
    }
}

fn finish_owner(hwnd_raw: isize) {
    let should = OWNED_HWND
        .lock()
        .ok()
        .and_then(|mut g| {
            if g.as_ref() == Some(&hwnd_raw) {
                *g = None;
                Some(())
            } else {
                None
            }
        })
        .is_some();
    if should {
        destroy_hwnd(HWND(hwnd_raw as _));
    }
}
