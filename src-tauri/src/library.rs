use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
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
    /// Unix epoch milliseconds when imported; used as video poster cache key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub imported_at: Option<i64>,
    #[serde(default, skip_serializing)]
    pub missing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct LibraryFile {
    items: Vec<LibraryItem>,
}

fn library_root(app: &AppHandle) -> Result<PathBuf, String> {
    crate::settings::library_root(app)
}

fn library_index_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(library_root(app)?.join("library.json"))
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
    crate::util::write_json_atomic(&path, file)
}

fn ext_allowed(ext: &str) -> bool {
    matches!(ext.to_ascii_lowercase().as_str(), "mp4" | "webm")
}

fn media_kind(ext: &str) -> &'static str {
    match ext.to_ascii_lowercase().as_str() {
        "mp4" | "webm" => "video",
        "gif" => "gif",
        _ => "image",
    }
}

fn format_size(bytes: u64) -> String {
    bytesize::ByteSize::b(bytes).to_string()
}

fn thumb_for(kind: &str) -> String {
    match kind {
        "video" => "linear-gradient(135deg,#c7d2fe,#4f46e5)".into(),
        "gif" => "linear-gradient(135deg,#bfdbfe,#2563eb)".into(),
        _ => "linear-gradient(135deg,#a5f3fc,#0891b2)".into(),
    }
}

fn now_imported_at_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub items: Vec<LibraryItem>,
    pub errors: Vec<String>,
}

pub fn list_items(app: &AppHandle) -> Result<Vec<LibraryItem>, String> {
    let mut items = load_library(app)?.items;
    for item in &mut items {
        item.missing = !std::path::Path::new(&item.path).is_file();
    }
    Ok(items)
}

pub fn import_paths(app: &AppHandle, paths: Vec<String>) -> Result<ImportResult, String> {
    let mut lib = load_library(app)?;
    let copy_to_data = crate::settings::load_settings(app)
        .map(|s| s.import_copy_to_data)
        .unwrap_or(true);
    let dir = library_root(app)?;
    if copy_to_data {
        fs::create_dir_all(&dir).map_err(|e| format!("创建库目录失败: {e}"))?;
    }
    let mut imported = Vec::new();
    let mut errors = Vec::new();
    let mut imported_at_base = now_imported_at_ms();

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
        let meta = fs::metadata(&src_path).ok();
        let size = meta
            .as_ref()
            .map(|m| format_size(m.len()))
            .unwrap_or_else(|| "?".into());
        let kind = media_kind(&ext).to_string();
        let (path_str, stored) = if copy_to_data {
            let dest_name = format!("{id}.{ext}");
            let dest = dir.join(&dest_name);
            if let Err(e) = fs::copy(&src_path, &dest) {
                errors.push(format!("复制失败 {name}: {e}"));
                continue;
            }
            let p = dest.to_string_lossy().to_string();
            (p.clone(), p)
        } else {
            (src_path.to_string_lossy().to_string(), src_path.to_string_lossy().to_string())
        };
        let imported_at = {
            let t = imported_at_base;
            imported_at_base += 1;
            t
        };
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
            media_src: path_str,
            path: stored,
            imported_at: Some(imported_at),
            missing: false,
        };
        lib.items.insert(0, item.clone());
        imported.push(item);
    }

    if !imported.is_empty() {
        save_library(app, &lib)?;
    }
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
        let path = std::path::Path::new(&item.path);
        if path.exists() && path.is_file() {
            let _ = fs::remove_file(path);
        }
        save_library(app, &lib)?;
        let _ = crate::favorites::set_favorite(app, id.to_string(), false);
    }
    Ok(())
}
