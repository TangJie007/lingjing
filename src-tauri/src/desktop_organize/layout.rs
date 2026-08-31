use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

const LAYOUT_FILE: &str = "fence_layout.json";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FenceLayout {
    #[serde(default)]
    pub app_order: Vec<String>,
    #[serde(default)]
    pub image_order: Vec<String>,
    #[serde(default)]
    pub document_order: Vec<String>,
    #[serde(default)]
    pub folder_order: Vec<String>,
    #[serde(default)]
    pub media_order: Vec<String>,
    #[serde(default)]
    pub archive_order: Vec<String>,
    #[serde(default)]
    pub categories: HashMap<String, String>,
}

fn layout_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|p| p.join(LAYOUT_FILE))
        .map_err(|e| format!("无法解析应用数据目录: {e}"))
}

pub fn load_layout(app: &AppHandle) -> Result<FenceLayout, String> {
    let path = layout_path(app)?;
    if !path.exists() {
        return Ok(FenceLayout::default());
    }
    let bytes = std::fs::read(&path).map_err(|e| format!("读取布局失败: {e}"))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("解析布局失败: {e}"))
}

pub fn save_layout(app: &AppHandle, layout: &FenceLayout) -> Result<(), String> {
    let path = layout_path(app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建数据目录失败: {e}"))?;
    }
    crate::util::write_json_atomic(&path, layout)
}

fn path_key(p: &str) -> String {
    p.replace('/', "\\").to_ascii_lowercase()
}

fn same_path(a: &str, b: &str) -> bool {
    path_key(a) == path_key(b)
}

fn replace_in_order(order: &mut Vec<String>, old: &str, new: &str) {
    if let Some(idx) = order.iter().position(|p| same_path(p, old)) {
        order[idx] = new.to_string();
    }
}

pub fn migrate_path(layout: &mut FenceLayout, old: &str, new: &str) {
    replace_in_order(&mut layout.app_order, old, new);
    replace_in_order(&mut layout.image_order, old, new);
    replace_in_order(&mut layout.document_order, old, new);
    replace_in_order(&mut layout.folder_order, old, new);
    replace_in_order(&mut layout.media_order, old, new);
    replace_in_order(&mut layout.archive_order, old, new);
    let cat_key = layout
        .categories
        .keys()
        .find(|p| same_path(p, old))
        .cloned();
    if let Some(key) = cat_key {
        if let Some(cat) = layout.categories.remove(&key) {
            layout.categories.insert(new.to_string(), cat);
        }
    }
}

pub fn remove_path(layout: &mut FenceLayout, path: &str) {
    layout.app_order.retain(|p| !same_path(p, path));
    layout.image_order.retain(|p| !same_path(p, path));
    layout.document_order.retain(|p| !same_path(p, path));
    layout.folder_order.retain(|p| !same_path(p, path));
    layout.media_order.retain(|p| !same_path(p, path));
    layout.archive_order.retain(|p| !same_path(p, path));
    let cat_keys: Vec<String> = layout
        .categories
        .keys()
        .filter(|p| same_path(p, path))
        .cloned()
        .collect();
    for key in cat_keys {
        layout.categories.remove(&key);
    }
}

#[tauri::command]
pub fn load_fence_layout(app: AppHandle) -> Result<FenceLayout, String> {
    load_layout(&app)
}

#[tauri::command]
pub fn save_fence_layout(app: AppHandle, layout: FenceLayout) -> Result<(), String> {
    save_layout(&app, &layout)
}
