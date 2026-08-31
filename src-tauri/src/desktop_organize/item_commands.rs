use std::fs;
use std::path::Path;

use tauri::AppHandle;

use crate::settings;
use super::lifecycle::set_enabled;
use super::scan::scan_desktop_items;
use super::types::DesktopItem;
use super::util::run_on_ui;
#[cfg(windows)]
use super::win;

#[tauri::command]
pub fn set_desktop_organize(app: AppHandle, enabled: bool) -> Result<(), String> {
    tracing::info!("[desktop-organize] set_desktop_organize enabled={enabled}");
    set_enabled(&app, enabled)?;
    let mut s = settings::load_settings(&app).unwrap_or_default();
    s.desktop_organize_enabled = enabled;
    settings::persist_and_notify(&app, &s)?;
    Ok(())
}

#[tauri::command]
pub fn list_desktop_items(app: AppHandle) -> Result<Vec<DesktopItem>, String> {
    run_on_ui(&app, scan_desktop_items)?
}

#[tauri::command]
pub fn open_desktop_item(path: String) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("路径为空".into());
    }
    let p = Path::new(trimmed);
    if !trimmed.starts_with("::") && !p.exists() {
        return Err("文件不存在".into());
    }
    #[cfg(windows)]
    {
        if trimmed.starts_with("::") {
            let target = if trimmed
                .to_ascii_uppercase()
                .contains("F02C1A0D-B21F-4110-8426-0A0C959C3602")
            {
                "shell:NetworkPlacesFolder"
            } else {
                trimmed
            };
            std::process::Command::new("explorer.exe")
                .arg(target)
                .spawn()
                .map_err(|e| format!("打开系统图标失败: {e}"))?;
            return Ok(());
        }
        std::process::Command::new("cmd")
            .args(["/C", "start", "", trimmed])
            .spawn()
            .map_err(|e| format!("打开失败: {e}"))?;
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        let _ = trimmed;
        Err("桌面整理仅支持 Windows".into())
    }
}

#[tauri::command]
pub fn show_desktop_item_in_folder(path: String) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("路径为空".into());
    }
    if trimmed.starts_with("::") {
        return Err("系统图标不支持此操作".into());
    }
    let p = Path::new(trimmed);
    if !p.exists() {
        return Err("文件不存在".into());
    }
    #[cfg(windows)]
    {
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", trimmed))
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {e}"))?;
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        let _ = trimmed;
        Err("桌面整理仅支持 Windows".into())
    }
}

#[tauri::command]
pub fn open_desktop_item_with(path: String) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("路径为空".into());
    }
    if trimmed.starts_with("::") {
        return Err("系统图标不支持此操作".into());
    }
    let p = Path::new(trimmed);
    if !p.exists() {
        return Err("文件不存在".into());
    }
    #[cfg(windows)]
    {
        std::process::Command::new("rundll32")
            .args(["shell32.dll,OpenAs_RunDLL", trimmed])
            .spawn()
            .map_err(|e| format!("打开方式失败: {e}"))?;
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        let _ = trimmed;
        Err("桌面整理仅支持 Windows".into())
    }
}

#[tauri::command]
pub fn open_desktop_item_properties(path: String) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("路径为空".into());
    }
    if trimmed.starts_with("::") {
        return Err("系统图标不支持此操作".into());
    }
    let p = Path::new(trimmed);
    if !p.exists() {
        return Err("文件不存在".into());
    }
    #[cfg(windows)]
    {
        return win::shell_show_properties(trimmed);
    }
    #[cfg(not(windows))]
    {
        let _ = trimmed;
        Err("桌面整理仅支持 Windows".into())
    }
}

#[tauri::command]
pub fn rename_desktop_item(app: AppHandle, path: String, new_name: String) -> Result<String, String> {
    let trimmed = path.trim();
    let name = new_name.trim();
    if trimmed.is_empty() || name.is_empty() {
        return Err("路径或名称为空".into());
    }
    if trimmed.starts_with("::") {
        return Err("系统图标不支持重命名".into());
    }
    if name.contains(['\\', '/', ':', '*', '?', '"', '<', '>', '|']) {
        return Err("名称包含非法字符".into());
    }
    let old = Path::new(trimmed);
    if !old.exists() {
        return Err("文件不存在".into());
    }
    let parent = old.parent().ok_or_else(|| "无法解析父目录".to_string())?;
    let new_path = parent.join(name);
    if new_path.exists() {
        return Err("目标名称已存在".into());
    }
    fs::rename(old, &new_path).map_err(|e| format!("重命名失败: {e}"))?;
    let new = new_path.to_string_lossy().into_owned();
    if let Ok(mut layout) = super::layout::load_layout(&app) {
        super::layout::migrate_path(&mut layout, trimmed, &new);
        let _ = super::layout::save_layout(&app, &layout);
    }
    Ok(new)
}

#[tauri::command]
pub fn delete_desktop_item(app: AppHandle, path: String) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("路径为空".into());
    }
    if trimmed.starts_with("::") {
        return Err("系统图标不支持删除".into());
    }
    let p = Path::new(trimmed);
    if !p.exists() {
        return Err("文件不存在".into());
    }
    #[cfg(windows)]
    {
        trash::delete(trimmed).map_err(|e| format!("删除失败: {e}"))?;
        if let Ok(mut layout) = super::layout::load_layout(&app) {
            super::layout::remove_path(&mut layout, trimmed);
            let _ = super::layout::save_layout(&app, &layout);
        }
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        let _ = trimmed;
        Err("桌面整理仅支持 Windows".into())
    }
}
