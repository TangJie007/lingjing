use std::fs;
use std::path::{Path, PathBuf};

use super::types::DesktopItem;
#[cfg(windows)]
use super::win;

fn scan_dir(dir: &Path, items: &mut Vec<DesktopItem>, seen: &mut std::collections::HashSet<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        let path_key = path.to_string_lossy().to_string();
        if seen.contains(&path_key) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        let is_dir = meta.is_dir();
        let kind = classify_kind(&name, is_dir);
        let (display_name, mut icon) = shell_name_and_icon(&path, &name, is_dir);
        #[cfg(windows)]
        if kind == "image" {
            if let Some(preview) = win::image_file_preview(&path, 96) {
                icon = Some(preview);
            }
        }
        seen.insert(path_key);
        items.push(DesktopItem {
            name: display_name,
            path: path.to_string_lossy().to_string(),
            is_dir,
            kind,
            builtin: false,
            icon,
        });
    }
}

fn file_ext(file_name: &str) -> String {
    Path::new(file_name)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
}

fn classify_kind(file_name: &str, is_dir: bool) -> String {
    if is_dir {
        return "folder".into();
    }
    let ext = file_ext(file_name);
    const APPS: &[&str] = &["lnk", "url", "exe", "bat", "cmd", "msi", "com", "appref-ms"];
    const IMAGES: &[&str] = &[
        "png", "jpg", "jpeg", "gif", "webp", "bmp", "ico", "svg", "tif", "tiff", "heic", "heif",
        "raw", "dng", "jfif",
    ];
    const DOCS: &[&str] = &[
        "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", "md", "csv", "rtf", "odt",
        "ods", "odp", "epub", "wps", "et", "dps",
    ];
    const ARCHIVES: &[&str] = &[
        "zip", "rar", "7z", "tar", "gz", "bz2", "xz", "iso", "cab", "arj", "lzh",
    ];
    const MEDIA: &[&str] = &[
        "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "mpg", "mpeg", "3gp",
        "mp3", "wav", "flac", "aac", "m4a", "wma", "ogg", "opus", "aiff", "mid",
    ];
    if APPS.contains(&ext.as_str()) {
        "app".into()
    } else if IMAGES.contains(&ext.as_str()) {
        "image".into()
    } else if DOCS.contains(&ext.as_str()) {
        "document".into()
    } else if ARCHIVES.contains(&ext.as_str()) {
        "archive".into()
    } else if MEDIA.contains(&ext.as_str()) {
        "media".into()
    } else {
        "other".into()
    }
}

fn fallback_display_name(file_name: &str) -> String {
    let lower = file_name.to_ascii_lowercase();
    if let Some(stripped) = lower
        .strip_suffix(".lnk")
        .or_else(|| lower.strip_suffix(".url"))
        .or_else(|| lower.strip_suffix(".exe"))
    {
        return file_name[..stripped.len()].to_string();
    }
    file_name.to_string()
}

fn shell_name_and_icon(path: &Path, file_name: &str, _is_dir: bool) -> (String, Option<String>) {
    #[cfg(windows)]
    {
        let meta = win::shell_name_and_icon(path);
        let name = meta
            .display_name
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| fallback_display_name(file_name));
        return (name, meta.icon);
    }
    #[cfg(not(windows))]
    {
        (fallback_display_name(file_name), None)
    }
}

pub fn scan_desktop_items() -> Result<Vec<DesktopItem>, String> {
    let mut items = Vec::new();
    let mut seen = std::collections::HashSet::new();
    #[cfg(windows)]
    for item in win::scan_builtin_desktop_icons() {
        seen.insert(item.path.clone());
        items.push(item);
    }
    for dir in desktop_scan_dirs() {
        scan_dir(&dir, &mut items, &mut seen);
    }
    items.sort_by(|a, b| {
        builtin_rank(&a.path).cmp(&builtin_rank(&b.path))
            .then_with(|| b.is_dir.cmp(&a.is_dir))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(items)
}

fn builtin_rank(path: &str) -> u8 {
    match path.to_ascii_uppercase() {
        p if p.contains("20D04FE0-3AEA-1069-A2D8-08002B30309D") => 0,
        p if p.contains("645FF040-5081-101B-9F08-00AA002F954E") => 1,
        p if p.contains("F02C1A0D-B21F-4110-8426-0A0C959C3602") => 2,
        _ => 3,
    }
}

pub(crate) fn strip_extended_path(path: PathBuf) -> PathBuf {
    let s = path.to_string_lossy();
    if let Some(rest) = s.strip_prefix(r"\\?\") {
        PathBuf::from(rest)
    } else {
        path
    }
}

pub(crate) fn desktop_scan_dirs() -> Vec<PathBuf> {
    use known_folders::{get_known_folder_path, KnownFolder};
    let mut dirs = Vec::new();
    if let Some(desktop) = get_known_folder_path(KnownFolder::Desktop) {
        dirs.push(desktop);
    }
    if let Some(public_desktop) = get_known_folder_path(KnownFolder::PublicDesktop) {
        dirs.push(public_desktop);
    }
    // Fallback when Known Folders API is unavailable.
    if dirs.is_empty() {
        if let Ok(home) = std::env::var("USERPROFILE") {
            dirs.push(PathBuf::from(home).join("Desktop"));
        }
        let public = std::env::var("PUBLIC").unwrap_or_else(|_| r"C:\Users\Public".into());
        dirs.push(PathBuf::from(public).join("Desktop"));
    }
    dirs
}
