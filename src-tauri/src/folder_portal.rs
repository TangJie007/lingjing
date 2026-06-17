//! Folder portal (DO-002) — maps a real folder to a desktop partition.
//! Added to desktop_organizer module.

use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderPortalInfo {
    pub partition_id: String,
    pub folder_path: String,
    pub files: Vec<FileEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
}

#[tauri::command]
pub fn open_folder_portal(
    partition_id: String,
    folder_path: String,
) -> Result<FolderPortalInfo, String> {
    let path = PathBuf::from(&folder_path);
    if !path.exists() {
        return Err("文件夹不存在".into());
    }
    if !path.is_dir() {
        return Err("路径不是文件夹".into());
    }

    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(&path) {
        for entry in entries.flatten() {
            let p = entry.path();
            let meta = entry.metadata().ok();
            files.push(FileEntry {
                name: p.file_name().unwrap_or_default().to_string_lossy().to_string(),
                path: p.to_string_lossy().to_string(),
                is_dir: meta.as_ref().map(|m| m.is_dir()).unwrap_or(false),
                size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
            });
        }
    }

    Ok(FolderPortalInfo {
        partition_id,
        folder_path,
        files,
    })
}

#[tauri::command]
pub fn refresh_folder_portal(folder_path: String) -> Result<Vec<FileEntry>, String> {
    let path = PathBuf::from(&folder_path);
    if !path.exists() {
        return Err("文件夹不存在".into());
    }

    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(&path) {
        for entry in entries.flatten() {
            let p = entry.path();
            let meta = entry.metadata().ok();
            files.push(FileEntry {
                name: p.file_name().unwrap_or_default().to_string_lossy().to_string(),
                path: p.to_string_lossy().to_string(),
                is_dir: meta.as_ref().map(|m| m.is_dir()).unwrap_or(false),
                size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
            });
        }
    }

    Ok(files)
}
