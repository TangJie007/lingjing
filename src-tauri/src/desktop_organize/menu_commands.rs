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
        // 此电脑 / 回收站 / 网络: custom namespace menu (no Shell host — CLSID
        // QueryContextMenu hangs; shortcut menus were unreliable).
        // Blank desktop / normal files: persistent host.
        if let Some(ref p) = path_opt {
            if crate::shell_menu::is_shell_namespace_path(p) {
                return Ok(crate::shell_menu::namespace_builtin_menu(p));
            }
            if Path::new(p).is_dir() {
                return Ok(crate::shell_menu::folder_builtin_menu());
            }
        }
        return tauri::async_runtime::spawn_blocking(move || {
            match run_shell_menu_host("root", path_opt.as_deref(), &[]) {
                Ok(entries) if path_opt.is_none() => Ok(crate::shell_menu::ensure_paste_entry(
                    crate::shell_menu::ensure_blank_refresh_pin(entries),
                )),
                Ok(entries) => Ok(entries),
                Err(e) => Err(e),
            }
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
                Err(e) if e == "BUILTIN_PIN_START" => {
                    let p = path_for_builtin.ok_or_else(|| "路径为空".to_string())?;
                    pin_shell_item_to_start(&p)
                }
                Err(e) if e == "BUILTIN_PIN_HOME" => {
                    let p = path_for_builtin.ok_or_else(|| "路径为空".to_string())?;
                    pin_shell_item_to_quick_access(&p)
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
        BUILTIN_DISCONNECT_NETWORK_DRIVE, BUILTIN_DISPLAY_SETTINGS, BUILTIN_EMPTY_RECYCLE,
        BUILTIN_MAP_NETWORK_DRIVE, BUILTIN_NEW_FOLDER, BUILTIN_NEW_TXT, BUILTIN_OPEN,
        BUILTIN_OPEN_DESKTOP, BUILTIN_OPEN_NEW_WINDOW, BUILTIN_OPEN_TERMINAL, BUILTIN_OPEN_WITH,
        BUILTIN_PASTE, BUILTIN_PERSONALIZE, BUILTIN_PIN_QUICK_ACCESS, BUILTIN_PIN_START,
        BUILTIN_PROPERTIES, BUILTIN_REFRESH, BUILTIN_RENAME, BUILTIN_SHOW_IN_FOLDER,
    };
    match command_id {
        BUILTIN_OPEN => open_desktop_item(path.to_string()),
        BUILTIN_SHOW_IN_FOLDER => show_desktop_item_in_folder(path.to_string()),
        BUILTIN_OPEN_WITH => open_desktop_item_with(path.to_string()),
        BUILTIN_PROPERTIES => open_desktop_item_properties(path.to_string()),
        BUILTIN_OPEN_NEW_WINDOW => open_folder_in_new_window(path),
        BUILTIN_PIN_QUICK_ACCESS => pin_shell_item_to_quick_access(path),
        BUILTIN_PIN_START => pin_shell_item_to_start(path),
        BUILTIN_CUT => clipboard_set_files(&[path], true),
        BUILTIN_COPY => clipboard_set_files(&[path], false),
        BUILTIN_CREATE_SHORTCUT => create_desktop_shortcut(path),
        BUILTIN_DELETE => delete_desktop_item(app.clone(), path.to_string()),
        BUILTIN_RENAME => Err("重命名需由前端提供新名称".into()),
        BUILTIN_COMPRESS_ZIP => compress_path_to_zip(path),
        BUILTIN_REFRESH => refresh(app),
        BUILTIN_NEW_FOLDER => create_on_desktop("新建文件夹", true),
        BUILTIN_NEW_TXT => create_on_desktop("新建文本文档.txt", false),
        BUILTIN_OPEN_DESKTOP => open_user_desktop_folder(),
        BUILTIN_OPEN_TERMINAL => open_terminal_on_desktop(),
        BUILTIN_DISPLAY_SETTINGS => open_uri("ms-settings:display"),
        BUILTIN_PERSONALIZE => open_uri("ms-settings:personalization"),
        BUILTIN_PASTE => clipboard_paste_to_desktop(app),
        BUILTIN_EMPTY_RECYCLE => empty_recycle_bin(),
        BUILTIN_MAP_NETWORK_DRIVE => map_network_drive(),
        BUILTIN_DISCONNECT_NETWORK_DRIVE => disconnect_network_drive(),
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
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("新建");
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
fn empty_recycle_bin() -> Result<(), String> {
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::SHEmptyRecycleBinW;

    // SHERB_NOCONFIRMATION is intentionally omitted so Windows still prompts.
    const SHERB_NOPROGRESSUI: u32 = 0x0000_0002;
    const SHERB_NOSOUND: u32 = 0x0000_0004;
    unsafe {
        SHEmptyRecycleBinW(None, PCWSTR::null(), SHERB_NOPROGRESSUI | SHERB_NOSOUND)
            .map_err(|e| format!("清空回收站失败: {e}"))
    }
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
fn shell_namespace_target(path: &str) -> String {
    let trimmed = path.trim();
    if let Some(clsid) = crate::desktop_organize::namespace_clsid_for_path(trimmed) {
        return format!("shell:{clsid}");
    }
    if trimmed.starts_with("::") {
        format!("shell:{trimmed}")
    } else {
        trimmed.to_string()
    }
}

#[cfg(windows)]
fn invoke_shell_namespace_verb(path: &str, verb: &str, err_label: &str) -> Result<(), String> {
    let target = shell_namespace_target(path);
    let script = format!(
        "$s=(New-Object -ComObject Shell.Application).NameSpace([string]'{}'); if($null -eq $s){{exit 1}}; try {{ $s.Self.InvokeVerb([string]'{}') }} catch {{ exit 2 }}",
        target.replace('\'', "''"),
        verb.replace('\'', "''")
    );
    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .status()
        .map_err(|e| format!("{err_label}: {e}"))?;
    if !status.success() {
        return Err(err_label.into());
    }
    Ok(())
}

#[cfg(windows)]
fn invoke_shell_item_verb(path: &str, verb: &str, err_label: &str) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("路径为空".into());
    }

    // Namespace parsing names: NameSpace(shell::{CLSID}).Self
    if trimmed.starts_with("::") || trimmed.to_ascii_lowercase().starts_with("shell:") {
        return invoke_shell_namespace_verb(trimmed, verb, err_label);
    }
    if let Some(clsid) = crate::desktop_organize::namespace_clsid_for_path(trimmed) {
        return invoke_shell_namespace_verb(&format!("shell:{clsid}"), verb, err_label);
    }

    // Files/folders: parent folder + ParseName (NameSpace(file) is invalid).
    let pb = Path::new(trimmed);
    let (folder, name) = if pb.is_dir() {
        (trimmed.to_string(), String::new())
    } else {
        let parent = pb
            .parent()
            .ok_or_else(|| "无法解析父目录".to_string())?
            .to_string_lossy()
            .into_owned();
        let name = pb
            .file_name()
            .ok_or_else(|| "无法解析文件名".to_string())?
            .to_string_lossy()
            .into_owned();
        (parent, name)
    };

    let script = if name.is_empty() {
        format!(
            "$sh=New-Object -ComObject Shell.Application; $f=$sh.NameSpace([string]'{folder}'); if($null -eq $f){{exit 1}}; try {{ $f.Self.InvokeVerb([string]'{verb}') }} catch {{ exit 2 }}; exit 0",
            folder = folder.replace('\'', "''"),
            verb = verb.replace('\'', "''"),
        )
    } else {
        format!(
            "$sh=New-Object -ComObject Shell.Application; $f=$sh.NameSpace([string]'{folder}'); if($null -eq $f){{exit 1}}; $i=$f.ParseName([string]'{name}'); if($null -eq $i){{exit 1}}; try {{ $i.InvokeVerb([string]'{verb}') }} catch {{ exit 2 }}; exit 0",
            folder = folder.replace('\'', "''"),
            name = name.replace('\'', "''"),
            verb = verb.replace('\'', "''"),
        )
    };
    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .status()
        .map_err(|e| format!("{err_label}: {e}"))?;
    if !status.success() {
        return Err(err_label.into());
    }
    Ok(())
}

#[cfg(windows)]
fn pin_shell_item_to_quick_access(path: &str) -> Result<(), String> {
    invoke_shell_item_verb(path, "pintohome", "固定到快速访问失败")
}

#[cfg(windows)]
fn pin_shell_item_to_start(path: &str) -> Result<(), String> {
    // Prefer localized verb DoIt — InvokeVerb("startpin") often returns success
    // without pinning. Win11 usually denies DoIt with E_ACCESSDENIED.
    if try_pin_to_start_via_verbs(path).is_ok() {
        return Ok(());
    }
    match add_to_start_apps_list(path) {
        Ok(()) => Err(
            "Windows 禁止第三方程序直接固定到「开始」（桌面整理替代资源管理器菜单后会触发）。已放入开始菜单「所有应用」，请在开始菜单中右键该项目再选择固定。"
                .into(),
        ),
        Err(_) => Err(
            "Windows 禁止第三方程序直接固定到「开始」。请打开开始菜单搜索该应用后右键固定。"
                .into(),
        ),
    }
}

#[cfg(windows)]
fn try_pin_to_start_via_verbs(path: &str) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("路径为空".into());
    }
    if trimmed.starts_with("::")
        || trimmed.to_ascii_lowercase().starts_with("shell:")
        || crate::desktop_organize::namespace_clsid_for_path(trimmed).is_some()
    {
        return invoke_shell_item_verb(path, "startpin", "固定到开始失败")
            .or_else(|_| invoke_shell_item_verb(path, "pintostartscreen", "固定到开始失败"));
    }
    let pb = Path::new(trimmed);
    let folder = pb
        .parent()
        .ok_or_else(|| "无法解析父目录".to_string())?
        .to_string_lossy()
        .into_owned();
    let name = pb
        .file_name()
        .ok_or_else(|| "无法解析文件名".to_string())?
        .to_string_lossy()
        .into_owned();
    let script = format!(
        "$sh=New-Object -ComObject Shell.Application; $f=$sh.NameSpace([string]'{folder}'); if($null -eq $f){{exit 1}}; $i=$f.ParseName([string]'{name}'); if($null -eq $i){{exit 1}}; $hit=$false; foreach($v in @($i.Verbs())){{ $n=(($v.Name)+'') -replace '&',''; if($n -match '固定' -and $n -match '开始' -and $n -notmatch '取消'){{ $v.DoIt(); $hit=$true; break }} }}; if(-not $hit){{exit 2}}; exit 0",
        folder = folder.replace('\'', "''"),
        name = name.replace('\'', "''"),
    );
    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .status()
        .map_err(|e| format!("固定到开始失败: {e}"))?;
    if !status.success() {
        return Err("固定到开始失败".into());
    }
    Ok(())
}

/// Best-effort: put a shortcut into Start Menu\\Programs (All apps list, not a Start pin tile).
#[cfg(windows)]
fn add_to_start_apps_list(path: &str) -> Result<(), String> {
    let trimmed = path.trim();
    let programs =
        known_folders::get_known_folder_path(known_folders::KnownFolder::Programs)
            .ok_or_else(|| "无法定位开始菜单程序目录".to_string())?;
    fs::create_dir_all(&programs).map_err(|e| format!("创建开始菜单目录失败: {e}"))?;

    if let Some(clsid) = crate::desktop_organize::namespace_clsid_for_path(trimmed) {
        let name = match crate::desktop_organize::builtin_kind_from_path(trimmed) {
            Some("computer") => "此电脑",
            Some("recycle") => "回收站",
            Some("network") => "网络",
            _ => "系统项目",
        };
        let dest = programs.join(format!("{name}.lnk"));
        let guid = clsid.trim_start_matches(':');
        let script = format!(
            "$w=New-Object -ComObject WScript.Shell; $s=$w.CreateShortcut([string]'{dest}'); $s.TargetPath='explorer.exe'; $s.Arguments=[string]'shell::{guid}'; $s.Description=[string]'{name}'; $s.Save()",
            dest = dest.to_string_lossy().replace('\'', "''"),
            guid = guid.replace('\'', "''"),
            name = name.replace('\'', "''"),
        );
        let status = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .status()
            .map_err(|e| format!("创建开始菜单快捷方式失败: {e}"))?;
        if !status.success() {
            return Err("创建开始菜单快捷方式失败".into());
        }
        return Ok(());
    }

    let src = Path::new(trimmed);
    if !src.exists() {
        return Err("目标不存在".into());
    }
    let file_name = src
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "无法解析文件名".to_string())?;
    let dest = if src
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("lnk"))
    {
        programs.join(file_name)
    } else {
        let stem = src
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(file_name);
        programs.join(format!("{stem}.lnk"))
    };
    if src
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("lnk"))
    {
        fs::copy(src, &dest).map_err(|e| format!("复制快捷方式失败: {e}"))?;
        return Ok(());
    }
    let script = format!(
        "$w=New-Object -ComObject WScript.Shell; $s=$w.CreateShortcut([string]'{dest}'); $s.TargetPath=[string]'{src}'; $s.WorkingDirectory=[string]'{dir}'; $s.Save()",
        dest = dest.to_string_lossy().replace('\'', "''"),
        src = trimmed.replace('\'', "''"),
        dir = src
            .parent()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default()
            .replace('\'', "''"),
    );
    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .status()
        .map_err(|e| format!("创建开始菜单快捷方式失败: {e}"))?;
    if !status.success() {
        return Err("创建开始菜单快捷方式失败".into());
    }
    Ok(())
}

#[cfg(windows)]
fn map_network_drive() -> Result<(), String> {
    std::process::Command::new("rundll32")
        .args(["shell32.dll,SHHelpShortcuts_RunDLL", "Connect"])
        .spawn()
        .map_err(|e| format!("打开映射网络驱动器失败: {e}"))?;
    Ok(())
}

#[cfg(windows)]
fn disconnect_network_drive() -> Result<(), String> {
    std::process::Command::new("rundll32")
        .args(["shell32.dll,SHHelpShortcuts_RunDLL", "Disconnect"])
        .spawn()
        .map_err(|e| format!("打开断开网络驱动器失败: {e}"))?;
    Ok(())
}

#[cfg(windows)]
fn create_desktop_shortcut(path: &str) -> Result<(), String> {
    let src = Path::new(path);
    if !src.exists() {
        return Err("目标不存在".into());
    }
    let parent = src.parent().ok_or_else(|| "无法解析父目录".to_string())?;
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
    let parent = src.parent().ok_or_else(|| "无法解析父目录".to_string())?;
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
        let hmem = GlobalAlloc(GMEM_MOVEABLE, total).map_err(|e| format!("剪贴板分配失败: {e}"))?;
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

#[cfg(windows)]
fn clipboard_paste_to_desktop(app: &AppHandle) -> Result<(), String> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use windows::core::w;
    use windows::Win32::System::DataExchange::{
        CloseClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard,
        RegisterClipboardFormatW,
    };
    use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};
    use windows::Win32::System::Ole::CF_HDROP;
    use windows::Win32::UI::Shell::{DragQueryFileW, HDROP};

    unsafe {
        if OpenClipboard(None).is_err() {
            return Err("无法打开剪贴板".into());
        }
        if IsClipboardFormatAvailable(u32::from(CF_HDROP.0)).is_err() {
            let _ = CloseClipboard();
            return Err("剪贴板中没有可粘贴的文件".into());
        }

        let mut cut = false;
        let fmt = RegisterClipboardFormatW(w!("Preferred DropEffect"));
        if fmt != 0 {
            if let Ok(heffect) = GetClipboardData(fmt) {
                let ep =
                    GlobalLock(windows::Win32::Foundation::HGLOBAL(heffect.0 as _)) as *const u32;
                if !ep.is_null() {
                    cut = *ep == 2;
                    let _ = GlobalUnlock(windows::Win32::Foundation::HGLOBAL(heffect.0 as _));
                }
            }
        }

        let hdrop_handle = GetClipboardData(u32::from(CF_HDROP.0))
            .map_err(|_| "读取剪贴板文件失败".to_string())?;
        let hdrop = HDROP(hdrop_handle.0);
        let count = DragQueryFileW(hdrop, u32::MAX, None);
        if count == 0 {
            let _ = CloseClipboard();
            return Err("剪贴板中没有可粘贴的文件".into());
        }

        let mut sources: Vec<PathBuf> = Vec::with_capacity(count as usize);
        for i in 0..count {
            let mut buf = vec![0u16; 520];
            let n = DragQueryFileW(hdrop, i, Some(&mut buf));
            if n == 0 {
                continue;
            }
            let path = OsString::from_wide(&buf[..n as usize]);
            let pb = PathBuf::from(path);
            if pb.as_os_str().is_empty() {
                continue;
            }
            sources.push(pb);
        }
        let _ = CloseClipboard();

        if sources.is_empty() {
            return Err("剪贴板中没有可粘贴的文件".into());
        }

        place_paths_on_desktop(app, &sources, Some(cut))?;

        // Cut: clear clipboard so Paste does not repeat a move.
        if cut {
            if OpenClipboard(None).is_ok() {
                let _ = windows::Win32::System::DataExchange::EmptyClipboard();
                let _ = CloseClipboard();
            }
        }
    }

    Ok(())
}

#[cfg(windows)]
fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| format!("创建目录失败: {e}"))?;
    for entry in fs::read_dir(src).map_err(|e| format!("读取目录失败: {e}"))? {
        let entry = entry.map_err(|e| format!("读取目录项失败: {e}"))?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            fs::copy(&from, &to).map_err(|e| format!("复制失败: {e}"))?;
        }
    }
    Ok(())
}

#[cfg(windows)]
fn same_volume(a: &Path, b: &Path) -> bool {
    use std::path::Component;
    match (a.components().next(), b.components().next()) {
        (Some(Component::Prefix(pa)), Some(Component::Prefix(pb))) => pa == pb,
        _ => false,
    }
}

#[cfg(windows)]
fn is_direct_child_of(path: &Path, dir: &Path) -> bool {
    let Some(parent) = path.parent() else {
        return false;
    };
    match (parent.canonicalize(), dir.canonicalize()) {
        (Ok(p), Ok(d)) => p == d,
        _ => parent == dir,
    }
}

/// Place files/folders onto the user desktop (Explorer drop / Paste).
#[cfg(windows)]
pub(crate) fn place_paths_on_desktop(
    app: &AppHandle,
    paths: &[PathBuf],
    move_files: Option<bool>,
) -> Result<(), String> {
    let desktop = primary_desktop_dir()?;
    if paths.is_empty() {
        return Err("没有可放置的文件".into());
    }

    let mut did_anything = false;
    for src in paths {
        if src.as_os_str().is_empty() || !src.exists() {
            continue;
        }
        // Already sitting on the desktop — skip.
        if is_direct_child_of(src, &desktop) {
            continue;
        }

        let name = src
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("放置的文件");
        let dest = unique_path_in(&desktop, name);
        if src == &dest {
            continue;
        }

        let do_move = move_files.unwrap_or_else(|| same_volume(src, &desktop));
        if do_move {
            fs::rename(src, &dest).or_else(|_| {
                if src.is_dir() {
                    copy_dir_recursive(src, &dest)?;
                    fs::remove_dir_all(src).map_err(|e| format!("移动删除失败: {e}"))
                } else {
                    fs::copy(src, &dest).map_err(|e| format!("放置失败: {e}"))?;
                    fs::remove_file(src).map_err(|e| format!("移动删除失败: {e}"))
                }
            })?;
        } else if src.is_dir() {
            copy_dir_recursive(src, &dest)?;
        } else {
            fs::copy(src, &dest).map_err(|e| format!("放置失败: {e}"))?;
        }
        did_anything = true;
    }

    if did_anything || !paths.is_empty() {
        refresh(app)?;
    }
    Ok(())
}

/// Drop files from Explorer (or other apps) onto the desktop fence.
#[tauri::command]
pub async fn drop_files_to_desktop(
    app: AppHandle,
    paths: Vec<String>,
    move_files: Option<bool>,
) -> Result<(), String> {
    #[cfg(windows)]
    {
        let paths: Vec<PathBuf> = paths
            .into_iter()
            .map(|p| PathBuf::from(p.trim()))
            .filter(|p| !p.as_os_str().is_empty())
            .collect();
        return tauri::async_runtime::spawn_blocking(move || {
            place_paths_on_desktop(&app, &paths, move_files)
        })
        .await
        .map_err(|e| format!("放置文件任务失败: {e}"))?;
    }
    #[cfg(not(windows))]
    {
        let _ = (app, paths, move_files);
        Err("桌面整理仅支持 Windows".into())
    }
}

/// Show the real Windows Shell menu and execute the selected command before
/// releasing its COM objects, preserving dynamic/owner-drawn menu behavior.
#[tauri::command]
pub async fn show_desktop_native_context_menu(app: AppHandle, path: String) -> Result<(), String> {
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
