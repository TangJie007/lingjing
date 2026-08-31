use std::fs;
use std::path::{Path, PathBuf};

use tauri::AppHandle;

use crate::settings;
use super::lifecycle::set_enabled;
use super::scan::scan_desktop_items;
use super::types::DesktopItem;
use super::util::run_on_ui;
#[cfg(windows)]
use super::win;

fn paths_same_file(a: &Path, b: &Path) -> bool {
    if a == b {
        return true;
    }
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(x), Ok(y)) => x == y,
        _ => {
            let norm = |p: &Path| {
                p.to_string_lossy()
                    .replace('/', "\\")
                    .trim_end_matches('\\')
                    .to_ascii_lowercase()
            };
            norm(a) == norm(b)
        }
    }
}

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
    let items = run_on_ui(&app, scan_desktop_items)??;
    #[cfg(windows)]
    {
        migrate_builtin_clsid_layout(&app, &items);
    }
    Ok(items)
}

/// Rewrite fence layout entries that still point at legacy managed `.lnk` paths.
#[cfg(windows)]
fn migrate_builtin_clsid_layout(app: &AppHandle, items: &[DesktopItem]) {
    let Ok(mut layout) = super::layout::load_layout(app) else {
        return;
    };
    let mut changed = false;
    for item in items.iter().filter(|i| i.builtin) {
        let Some(kind) = super::builtin_kind_from_path(&item.path) else {
            continue;
        };
        let clsid = match kind {
            "computer" => super::CLSID_COMPUTER,
            "recycle" => super::CLSID_RECYCLE,
            "network" => super::CLSID_NETWORK,
            _ => continue,
        };
        // Collect legacy keys that should map to this CLSID.
        let legacy_keys: Vec<String> = layout
            .app_order
            .iter()
            .cloned()
            .chain(layout.categories.keys().cloned())
            .filter(|p| {
                if p == &item.path || p == clsid {
                    return false;
                }
                match super::builtin_kind_from_path(p) {
                    Some(k) if k == kind => true,
                    _ => path_has_clsid(p, clsid),
                }
            })
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        for old in legacy_keys {
            super::layout::migrate_path(&mut layout, &old, &item.path);
            changed = true;
        }
    }
    if changed {
        let _ = super::layout::save_layout(app, &layout);
    }
}

#[cfg(windows)]
fn path_has_clsid(path: &str, clsid: &str) -> bool {
    let a = path.replace('/', "\\").to_ascii_uppercase();
    let b = clsid.replace('/', "\\").to_ascii_uppercase();
    a == b || a.contains(b.trim_start_matches(':'))
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
        if let Some(clsid) = super::namespace_clsid_for_path(trimmed) {
            let target = if matches!(super::builtin_kind_from_path(trimmed), Some("network")) {
                "shell:NetworkPlacesFolder".to_string()
            } else {
                format!("shell:{clsid}")
            };
            std::process::Command::new("explorer.exe")
                .arg(&target)
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
    let is_namespace = trimmed.starts_with("::")
        || {
            #[cfg(windows)]
            {
                super::namespace_clsid_for_path(trimmed).is_some()
            }
            #[cfg(not(windows))]
            {
                false
            }
        };
    if !is_namespace {
        let p = Path::new(trimmed);
        if !p.exists() {
            return Err("文件不存在".into());
        }
    }
    #[cfg(windows)]
    {
        let props_path = super::namespace_clsid_for_path(trimmed).unwrap_or(trimmed);
        return win::shell_show_properties(props_path);
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
    #[cfg(windows)]
    if super::is_shell_namespace_item(trimmed) {
        return Err("系统图标不支持重命名".into());
    }
    if name.contains(['\\', '/', ':', '*', '?', '"', '<', '>', '|']) {
        return Err("名称包含非法字符".into());
    }
    let old = Path::new(trimmed);
    if !old.exists() {
        return Err("文件不存在".into());
    }
    let final_name = {
        let candidate = Path::new(name);
        match (
            old.extension().and_then(|e| e.to_str()),
            candidate.extension().and_then(|e| e.to_str()),
        ) {
            (Some(ext), None) => format!("{name}.{ext}"),
            _ => name.to_string(),
        }
    };
    let parent = old.parent().ok_or_else(|| "无法解析父目录".to_string())?;
    let new_path = parent.join(&final_name);
    if paths_same_file(old, &new_path) {
        return Ok(trimmed.to_string());
    }
    if new_path.exists() {
        return Err("目标名称已存在".into());
    }
    fs::rename(old, &new_path).map_err(|e| format!("重命名失败: {e}"))?;
    super::icon_cache::invalidate(trimmed);
    let new = new_path.to_string_lossy().into_owned();
    if let Ok(mut layout) = super::layout::load_layout(&app) {
        super::layout::migrate_path(&mut layout, trimmed, &new);
        let _ = super::layout::save_layout(&app, &layout);
    }
    Ok(new)
}

/// Move a desktop item into a folder on the desktop (Explorer-like drop-on-folder).
#[tauri::command]
pub fn move_desktop_item_into_folder(
    app: AppHandle,
    path: String,
    folder_path: String,
) -> Result<String, String> {
    let src = path.trim();
    let dest_folder = folder_path.trim();
    if src.is_empty() || dest_folder.is_empty() {
        return Err("路径为空".into());
    }
    if src.starts_with("::") || dest_folder.starts_with("::") {
        return Err("系统图标不支持此操作".into());
    }
    let src_path = Path::new(src);
    let folder = Path::new(dest_folder);
    tracing::info!(
        "[desktop-organize] move into folder src={src} folder={dest_folder}"
    );
    if !src_path.exists() {
        return Err(format!("源文件不存在: {src}"));
    }
    if !folder.is_dir() {
        return Err("目标不是文件夹".into());
    }
    // Refuse moving a folder into itself or a descendant.
    if src_path.is_dir() {
        let src_canon = src_path
            .canonicalize()
            .unwrap_or_else(|_| src_path.to_path_buf());
        let folder_canon = folder
            .canonicalize()
            .unwrap_or_else(|_| folder.to_path_buf());
        if folder_canon == src_canon || folder_canon.starts_with(&src_canon) {
            return Err("不能将文件夹移动到自身或其子目录中".into());
        }
    }
    let name = src_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "无法解析文件名".to_string())?;
    let dest = unique_path_in_dir(folder, name);
    if dest.as_path() == src_path {
        return Ok(src.to_string());
    }

    fs::rename(src_path, &dest).or_else(|_| {
        if src_path.is_dir() {
            copy_dir_recursive(src_path, &dest)?;
            fs::remove_dir_all(src_path).map_err(|e| format!("移动删除失败: {e}"))
        } else {
            fs::copy(src_path, &dest).map_err(|e| format!("移动失败: {e}"))?;
            fs::remove_file(src_path).map_err(|e| format!("移动删除失败: {e}"))
        }
    })?;

    let new_path = dest.to_string_lossy().into_owned();
    super::icon_cache::invalidate(src);
    if let Ok(mut layout) = super::layout::load_layout(&app) {
        // Item leaves the desktop root listing — drop from fence orders.
        super::layout::remove_path(&mut layout, src);
        let _ = super::layout::save_layout(&app, &layout);
    }
    let _ = super::lifecycle::refresh(&app);
    tracing::info!("[desktop-organize] moved into folder -> {new_path}");
    Ok(new_path)
}

fn unique_path_in_dir(dir: &Path, name: &str) -> PathBuf {
    let candidate = dir.join(name);
    if !candidate.exists() {
        return candidate;
    }
    let path = Path::new(name);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("文件");
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
        crate::shell_menu::delete_to_recycle_bin(trimmed)?;
        super::icon_cache::invalidate(trimmed);
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
