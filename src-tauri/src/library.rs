use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItem {
    pub id: String,
    pub name: String,
    pub thumb: String,
    #[serde(rename = "type")]
    pub media_type: String,
    pub size: String,
    pub category: String,
    pub author: String,
    pub heat: String,
    pub favorite: bool,
    pub tags: Vec<String>,
    pub media_src: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct LibraryFile {
    items: Vec<LibraryItem>,
}

fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| format!("无法解析应用数据目录: {e}"))
}

fn library_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app_data_dir(app)?.join("library");
    fs::create_dir_all(&dir).map_err(|e| format!("创建库目录失败: {e}"))?;
    Ok(dir)
}

fn library_index_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join("library.json"))
}

fn load_library(app: &AppHandle) -> Result<LibraryFile, String> {
    let path = library_index_path(app)?;
    if !path.exists() {
        return Ok(LibraryFile::default());
    }
    let raw = fs::read_to_string(&path).map_err(|e| format!("读取 library.json 失败: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("解析 library.json 失败: {e}"))
}

fn save_library(app: &AppHandle, file: &LibraryFile) -> Result<(), String> {
    let path = library_index_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建数据目录失败: {e}"))?;
    }
    let raw = serde_json::to_string_pretty(file).map_err(|e| format!("序列化失败: {e}"))?;
    fs::write(&path, raw).map_err(|e| format!("写入 library.json 失败: {e}"))
}

fn ext_allowed(ext: &str) -> bool {
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "mp4" | "webm" | "gif" | "webp" | "jpg" | "jpeg" | "png"
    )
}

fn media_kind(ext: &str) -> &'static str {
    match ext.to_ascii_lowercase().as_str() {
        "mp4" | "webm" => "video",
        "gif" => "gif",
        _ => "image",
    }
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1}M", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{}K", bytes / 1024)
    } else {
        format!("{bytes}B")
    }
}

fn thumb_for(kind: &str) -> String {
    match kind {
        "video" => "linear-gradient(135deg,#c7d2fe,#4f46e5)".into(),
        "gif" => "linear-gradient(135deg,#bfdbfe,#2563eb)".into(),
        _ => "linear-gradient(135deg,#a5f3fc,#0891b2)".into(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub items: Vec<LibraryItem>,
    pub errors: Vec<String>,
}

pub fn list_items(app: &AppHandle) -> Result<Vec<LibraryItem>, String> {
    Ok(load_library(app)?.items)
}

pub fn import_paths(app: &AppHandle, paths: Vec<String>) -> Result<ImportResult, String> {
    let mut lib = load_library(app)?;
    let dir = library_dir(app)?;
    let mut imported = Vec::new();
    let mut errors = Vec::new();

    for src in paths {
        let src_path = PathBuf::from(&src);
        let ext = src_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();
        if !ext_allowed(&ext) {
            errors.push(format!("不支持的格式: {src}"));
            continue;
        }
        if !src_path.is_file() {
            errors.push(format!("文件不存在: {src}"));
            continue;
        }

        let id = format!("l-{}", Uuid::new_v4());
        let name = src_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("未命名")
            .to_string();
        let dest_name = format!("{id}.{ext}");
        let dest = dir.join(&dest_name);
        if let Err(e) = fs::copy(&src_path, &dest) {
            errors.push(format!("复制失败 {name}: {e}"));
            continue;
        }
        let meta = fs::metadata(&dest).ok();
        let size = meta.map(|m| format_size(m.len())).unwrap_or_else(|| "?".into());
        let kind = media_kind(&ext).to_string();
        let path_str = dest.to_string_lossy().to_string();
        let item = LibraryItem {
            id: id.clone(),
            name,
            thumb: thumb_for(&kind),
            media_type: kind,
            size,
            category: "本地".into(),
            author: "本地导入".into(),
            heat: "本地".into(),
            favorite: false,
            tags: vec!["#本地".into()],
            media_src: path_str.clone(),
            path: path_str,
        };
        lib.items.insert(0, item.clone());
        imported.push(item);
    }

    save_library(app, &lib)?;
    if imported.is_empty() && !errors.is_empty() {
        return Err(errors.join("; "));
    }
    Ok(ImportResult {
        items: imported,
        errors,
    })
}

pub fn set_favorite_flag(app: &AppHandle, id: &str, favorite: bool) -> Result<(), String> {
    let mut lib = load_library(app)?;
    if let Some(item) = lib.items.iter_mut().find(|i| i.id == id) {
        item.favorite = favorite;
        save_library(app, &lib)?;
    }
    Ok(())
}

pub fn remove_item(app: &AppHandle, id: &str) -> Result<(), String> {
    let mut lib = load_library(app)?;
    if let Some(pos) = lib.items.iter().position(|i| i.id == id) {
        let item = lib.items.remove(pos);
        let _ = fs::remove_file(Path::new(&item.path));
        save_library(app, &lib)?;
    }
    Ok(())
}
