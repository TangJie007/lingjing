use std::fs;
use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};

use super::item_commands::{
    delete_desktop_item, open_desktop_item, open_desktop_item_properties, open_desktop_item_with,
    show_desktop_item_in_folder,
};
use super::lifecycle::refresh;
use super::scan::desktop_scan_dirs;
use super::shell_host::run_shell_menu_host;
use super::state::FENCE_LABEL;
use super::types::ShellMenuEntry;

/// List Shell COM context menu entries (custom UI; includes icons when available).
/// Folders always use the built-in Explorer-like menu (QueryContextMenu hangs on many folders).
#[tauri::command]
pub async fn list_desktop_shell_context_menu(
    app: AppHandle,
    path: String,
) -> Result<Vec<ShellMenuEntry>, String> {
    let trimmed = path.trim().to_string();
    let path_opt = if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.clone())
    };
    let _window = app
        .get_webview_window(FENCE_LABEL)
        .ok_or_else(|| "格子窗口未就绪".to_string())?;

    #[cfg(windows)]
    {
        // Folders: built-in (QueryContextMenu often hangs on folder extensions).
        // Blank desktop: real Shell desktop-background menu via host (path = None).
        if let Some(ref p) = path_opt {
            if Path::new(p).is_dir() {
                return Ok(crate::shell_menu::folder_builtin_menu());
            }
        }
        return tauri::async_runtime::spawn_blocking(move || {
            run_shell_menu_host("root", path_opt.as_deref(), &[])
        })
        .await
        .map_err(|e| format!("加载右键菜单任务失败: {e}"))?;
    }
    #[cfg(not(windows))]
    {
        let _ = (_window, path_opt);
        Err("桌面整理仅支持 Windows".into())
    }
}

#[tauri::command]
pub async fn list_desktop_shell_context_submenu(
    app: AppHandle,
    path: String,
    menu_path: Vec<u32>,
) -> Result<Vec<ShellMenuEntry>, String> {
    let trimmed = path.trim();
    let path_opt = if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    };
    let _window = app
        .get_webview_window(FENCE_LABEL)
        .ok_or_else(|| "格子窗口未就绪".to_string())?;

    #[cfg(windows)]
    {
        return tauri::async_runtime::spawn_blocking(move || {
            run_shell_menu_host("submenu", path_opt.as_deref(), &menu_path)
        })
        .await
        .map_err(|e| format!("加载二级菜单任务失败: {e}"))?;
    }
    #[cfg(not(windows))]
    {
        let _ = (_window, path_opt, menu_path);
        Err("桌面整理仅支持 Windows".into())
    }
}

/// Invoke a Shell COM context menu command previously listed for path/blank desktop.
#[tauri::command]
pub async fn invoke_desktop_shell_context_command(
    app: AppHandle,
    path: String,
    command_id: u32,
    menu_path: Vec<u32>,
) -> Result<(), String> {
    let trimmed = path.trim();
    let path_opt = if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    };
    let _window = app
        .get_webview_window(FENCE_LABEL)
        .ok_or_else(|| "格子窗口未就绪".to_string())?;

    #[cfg(windows)]
    {
        if crate::shell_menu::is_builtin_command(command_id) {
            return dispatch_builtin_shell_command(
                &app,
                path_opt.as_deref().unwrap_or(""),
                command_id,
            );
        }
        let path_for_builtin = path_opt.clone();
        let mut payload = vec![command_id];
        payload.extend(menu_path.iter().copied());
        return tauri::async_runtime::spawn_blocking(move || {
            match run_shell_menu_host("invoke", path_opt.as_deref(), &payload) {
                Ok(_) => Ok(()),
                Err(e) if e == "BUILTIN_CUT" => {
                    let p = path_for_builtin.ok_or_else(|| "路径为空".to_string())?;
                    clipboard_set_files(&[&p], true)
                }
                Err(e) if e == "BUILTIN_COPY" => {
                    let p = path_for_builtin.ok_or_else(|| "路径为空".to_string())?;
                    clipboard_set_files(&[&p], false)
                }
                Err(e) if e == "BUILTIN_LINK" => {
                    let p = path_for_builtin.ok_or_else(|| "路径为空".to_string())?;
                    create_desktop_shortcut(&p)
                }
                Err(e) if e == "BUILTIN_RENAME" => Err("重命名需由前端提供新名称".into()),
                Err(e) => Err(e),
            }
        })
        .await
        .map_err(|e| format!("执行菜单命令任务失败: {e}"))?;
    }
    #[cfg(not(windows))]
    {
        let _ = (_window, path_opt, command_id, menu_path);
        Err("桌面整理仅支持 Windows".into())
    }
}

#[cfg(windows)]
fn dispatch_builtin_shell_command(
    app: &AppHandle,
    path: &str,
    command_id: u32,
) -> Result<(), String> {
    use crate::shell_menu::{
        BUILTIN_COMPRESS_ZIP, BUILTIN_COPY, BUILTIN_CREATE_SHORTCUT, BUILTIN_CUT, BUILTIN_DELETE,
        BUILTIN_DISPLAY_SETTINGS, BUILTIN_NEW_FOLDER, BUILTIN_NEW_TXT, BUILTIN_OPEN,
        BUILTIN_OPEN_DESKTOP, BUILTIN_OPEN_NEW_WINDOW, BUILTIN_OPEN_TERMINAL, BUILTIN_OPEN_WITH,
        BUILTIN_PERSONALIZE, BUILTIN_PIN_QUICK_ACCESS, BUILTIN_PROPERTIES, BUILTIN_REFRESH,
        BUILTIN_RENAME, BUILTIN_SHOW_IN_FOLDER,
    };
    match command_id {
        BUILTIN_OPEN => open_desktop_item(path.to_string()),
        BUILTIN_SHOW_IN_FOLDER => show_desktop_item_in_folder(path.to_string()),
        BUILTIN_OPEN_WITH => open_desktop_item_with(path.to_string()),
        BUILTIN_PROPERTIES => open_desktop_item_properties(path.to_string()),
        BUILTIN_OPEN_NEW_WINDOW => open_folder_in_new_window(path),
        BUILTIN_PIN_QUICK_ACCESS => pin_folder_to_quick_access(path),
        BUILTIN_CUT => clipboard_set_files(&[path], true),
        BUILTIN_COPY => clipboard_set_files(&[path], false),
        BUILTIN_CREATE_SHORTCUT => create_desktop_shortcut(path),
        BUILTIN_DELETE => delete_desktop_item(path.to_string()),
        BUILTIN_RENAME => Err("重命名需由前端提供新名称".into()),
        BUILTIN_COMPRESS_ZIP => compress_path_to_zip(path),
        BUILTIN_REFRESH => refresh(app),
        BUILTIN_NEW_FOLDER => create_on_desktop("新建文件夹", true),
        BUILTIN_NEW_TXT => create_on_desktop("新建文本文档.txt", false),
        BUILTIN_OPEN_DESKTOP => open_user_desktop_folder(),
        BUILTIN_OPEN_TERMINAL => open_terminal_on_desktop(),
        BUILTIN_DISPLAY_SETTINGS => open_uri("ms-settings:display"),
        BUILTIN_PERSONALIZE => open_uri("ms-settings:personalization"),
        _ => Err("未知内置命令".into()),
    }
}

#[cfg(windows)]
fn primary_desktop_dir() -> Result<PathBuf, String> {
    desktop_scan_dirs()
        .into_iter()
        .next()
        .ok_or_else(|| "无法定位桌面目录".to_string())
}

#[cfg(windows)]
fn unique_path_in(dir: &Path, name: &str) -> PathBuf {
    let candidate = dir.join(name);
    if !candidate.exists() {
        return candidate;
    }
    let path = Path::new(name);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("新建");
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|e| format!(".{e}"))
        .unwrap_or_default();
    for n in 2..200 {
        let p = dir.join(format!("{stem} ({n}){ext}"));
        if !p.exists() {
            return p;
        }
    }
    dir.join(format!("{stem}-{}{ext}", uuid::Uuid::new_v4()))
}

#[cfg(windows)]
fn create_on_desktop(name: &str, is_dir: bool) -> Result<(), String> {
    let desktop = primary_desktop_dir()?;
    let dest = unique_path_in(&desktop, name);
    if is_dir {
        fs::create_dir(&dest).map_err(|e| format!("新建文件夹失败: {e}"))?;
    } else {
        fs::write(&dest, "").map_err(|e| format!("新建文本文档失败: {e}"))?;
    }
    Ok(())
}

#[cfg(windows)]
fn open_user_desktop_folder() -> Result<(), String> {
    let desktop = primary_desktop_dir()?;
    std::process::Command::new("explorer")
        .arg(&desktop)
        .spawn()
        .map_err(|e| format!("打开桌面失败: {e}"))?;
    Ok(())
}

#[cfg(windows)]
fn open_terminal_on_desktop() -> Result<(), String> {
    let desktop = primary_desktop_dir()?;
    // Prefer Windows Terminal; fall back to cmd.
    if std::process::Command::new("wt")
        .args(["-d"])
        .arg(&desktop)
        .spawn()
        .is_ok()
    {
        return Ok(());
    }
    std::process::Command::new("cmd")
        .args(["/K", "cd", "/D"])
        .arg(&desktop)
        .spawn()
        .map_err(|e| format!("打开终端失败: {e}"))?;
    Ok(())
}

#[cfg(windows)]
fn open_uri(uri: &str) -> Result<(), String> {
    std::process::Command::new("cmd")
        .args(["/C", "start", "", uri])
        .spawn()
        .map_err(|e| format!("打开失败: {e}"))?;
    Ok(())
}

#[cfg(windows)]
fn open_folder_in_new_window(path: &str) -> Result<(), String> {
    std::process::Command::new("explorer")
        .arg(format!("/n,/e,{path}"))
        .spawn()
        .map_err(|e| format!("在新窗口中打开失败: {e}"))?;
    Ok(())
}

#[cfg(windows)]
fn pin_folder_to_quick_access(path: &str) -> Result<(), String> {
    // Shell verb "pintohome" — same as Explorer “固定到快速访问”.
    let script = format!(
        "$s=(New-Object -ComObject Shell.Application).NameSpace([string]'{}'); if($null -eq $s){{exit 1}}; $s.Self.InvokeVerb('pintohome')",
        path.replace('\'', "''")
    );
    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .status()
        .map_err(|e| format!("固定到快速访问失败: {e}"))?;
    if !status.success() {
        return Err("固定到快速访问失败".into());
    }
    Ok(())
}

#[cfg(windows)]
fn create_desktop_shortcut(path: &str) -> Result<(), String> {
    let src = Path::new(path);
    if !src.exists() {
        return Err("目标不存在".into());
    }
    let parent = src
        .parent()
        .ok_or_else(|| "无法解析父目录".to_string())?;
    let stem = src
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("快捷方式");
    let mut dest = parent.join(format!("{stem} - 快捷方式.lnk"));
    let mut n = 2u32;
    while dest.exists() {
        dest = parent.join(format!("{stem} - 快捷方式 ({n}).lnk"));
        n += 1;
        if n > 99 {
            return Err("快捷方式文件过多".into());
        }
    }
    let script = format!(
        "$w=New-Object -ComObject WScript.Shell; $s=$w.CreateShortcut([string]'{}'); $s.TargetPath=[string]'{}'; $s.Save()",
        dest.to_string_lossy().replace('\'', "''"),
        path.replace('\'', "''")
    );
    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .status()
        .map_err(|e| format!("创建快捷方式失败: {e}"))?;
    if !status.success() {
        return Err("创建快捷方式失败".into());
    }
    Ok(())
}

#[cfg(windows)]
fn compress_path_to_zip(path: &str) -> Result<(), String> {
    let src = Path::new(path);
    if !src.exists() {
        return Err("目标不存在".into());
    }
    let parent = src
        .parent()
        .ok_or_else(|| "无法解析父目录".to_string())?;
    let stem = src
        .file_stem()
        .and_then(|s| s.to_str())
        .or_else(|| src.file_name().and_then(|s| s.to_str()))
        .unwrap_or("archive");
    let mut dest = parent.join(format!("{stem}.zip"));
    let mut n = 2u32;
    while dest.exists() {
        dest = parent.join(format!("{stem} ({n}).zip"));
        n += 1;
        if n > 99 {
            return Err("压缩文件过多".into());
        }
    }
    let script = format!(
        "Compress-Archive -LiteralPath '{}' -DestinationPath '{}' -Force",
        path.replace('\'', "''"),
        dest.to_string_lossy().replace('\'', "''")
    );
    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .status()
        .map_err(|e| format!("压缩失败: {e}"))?;
    if !status.success() {
        return Err("压缩为 ZIP 失败".into());
    }
    Ok(())
}

#[cfg(windows)]
fn clipboard_set_files(paths: &[&str], cut: bool) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::mem::size_of;
    use std::os::windows::ffi::OsStrExt;
    use windows::core::w;
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, OpenClipboard, RegisterClipboardFormatW, SetClipboardData,
    };
    use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
    use windows::Win32::System::Ole::CF_HDROP;

    #[repr(C)]
    struct DropFiles {
        p_files: u32,
        pt: windows::Win32::Foundation::POINT,
        f_nc: i32,
        f_wide: i32,
    }

    let mut encoded: Vec<u16> = Vec::new();
    for p in paths {
        encoded.extend(OsStr::new(p).encode_wide());
        encoded.push(0);
    }
    encoded.push(0);
    let path_bytes = encoded.len() * 2;
    let header_size = size_of::<DropFiles>();
    let total = header_size + path_bytes;

    unsafe {
        let hmem = GlobalAlloc(GMEM_MOVEABLE, total)
            .map_err(|e| format!("剪贴板分配失败: {e}"))?;
        let ptr = GlobalLock(hmem) as *mut u8;
        if ptr.is_null() {
            return Err("剪贴板锁定失败".into());
        }
        let header = DropFiles {
            p_files: header_size as u32,
            pt: windows::Win32::Foundation::POINT { x: 0, y: 0 },
            f_nc: 0,
            f_wide: 1,
        };
        std::ptr::write_unaligned(ptr as *mut DropFiles, header);
        std::ptr::copy_nonoverlapping(
            encoded.as_ptr() as *const u8,
            ptr.add(header_size),
            path_bytes,
        );
        let _ = GlobalUnlock(hmem);

        if !OpenClipboard(None).is_ok() {
            return Err("无法打开剪贴板".into());
        }
        let _ = EmptyClipboard();
        if SetClipboardData(u32::from(CF_HDROP.0), Some(HANDLE(hmem.0 as _))).is_err() {
            let _ = CloseClipboard();
            return Err("写入剪贴板失败".into());
        }

        let fmt = RegisterClipboardFormatW(w!("Preferred DropEffect"));
        if fmt != 0 {
            if let Ok(heffect) = GlobalAlloc(GMEM_MOVEABLE, 4) {
                let ep = GlobalLock(heffect) as *mut u32;
                if !ep.is_null() {
                    // DROPEFFECT_MOVE = 2, DROPEFFECT_COPY = 1
                    *ep = if cut { 2 } else { 1 };
                    let _ = GlobalUnlock(heffect);
                    let _ = SetClipboardData(fmt, Some(HANDLE(heffect.0 as _)));
                }
            }
        }
        let _ = CloseClipboard();
    }
    Ok(())
}

/// Show the real Windows Shell menu and execute the selected command before
/// releasing its COM objects, preserving dynamic/owner-drawn menu behavior.
#[tauri::command]
pub async fn show_desktop_native_context_menu(
    app: AppHandle,
    path: String,
) -> Result<(), String> {
    let trimmed = path.trim();
    let path_opt = if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    };
    let _window = app
        .get_webview_window(FENCE_LABEL)
        .ok_or_else(|| "格子窗口未就绪".to_string())?;

    #[cfg(windows)]
    {
        return tauri::async_runtime::spawn_blocking(move || {
            run_shell_menu_host("native", path_opt.as_deref(), &[]).map(|_| ())
        })
        .await
        .map_err(|e| format!("显示原生右键菜单任务失败: {e}"))?;
    }
    #[cfg(not(windows))]
    {
        let _ = (_window, path_opt);
        Err("桌面整理仅支持 Windows".into())
    }
}
