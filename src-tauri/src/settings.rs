use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};

const SETTINGS_FILE: &str = "settings.json";
const LAST_WALLPAPER_FILE: &str = "last_wallpaper.json";
const LIBRARY_SUBDIR: &str = "library";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default = "default_true")]
    pub autostart: bool,
    #[serde(default)]
    pub hide_icons_on_double_click: bool,
    #[serde(default = "default_true")]
    pub pause_on_fullscreen: bool,
    #[serde(default = "default_true")]
    pub pause_on_battery: bool,
    #[serde(default = "default_true")]
    pub pause_on_rdp: bool,
    #[serde(default = "default_true")]
    pub sound_on: bool,
    #[serde(default = "default_volume")]
    pub default_volume: f64,
    #[serde(default = "default_true")]
    pub import_copy_to_data: bool,
    #[serde(default)]
    pub library_dir_override: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            autostart: true,
            hide_icons_on_double_click: false,
            pause_on_fullscreen: true,
            pause_on_battery: true,
            pause_on_rdp: true,
            sound_on: true,
            default_volume: 0.8,
            import_copy_to_data: true,
            library_dir_override: None,
        }
    }
}

fn default_true() -> bool {
    true
}
fn default_volume() -> f64 {
    0.8
}

fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| format!("无法解析应用数据目录: {e}"))
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join(SETTINGS_FILE))
}

fn last_wallpaper_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join(LAST_WALLPAPER_FILE))
}

pub fn load_settings(app: &AppHandle) -> Result<AppSettings, String> {
    let path = settings_path(app)?;
    if !path.exists() {
        return Ok(AppSettings::default());
    }
    let raw = fs::read_to_string(&path).map_err(|e| format!("读取 settings.json 失败: {e}"))?;
    let value: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| format!("解析 settings.json 失败: {e}"))?;
    let settings: AppSettings = serde_json::from_value(value.clone()).unwrap_or_default();
    Ok(settings)
}

pub fn save_settings(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建数据目录失败: {e}"))?;
    }
    let raw = serde_json::to_string_pretty(settings).map_err(|e| format!("序列化失败: {e}"))?;
    fs::write(&path, raw).map_err(|e| format!("写入 settings.json 失败: {e}"))
}

pub fn persist_and_notify(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    save_settings(app, settings)?;
    let _ = app.emit("settings-updated", settings);
    Ok(())
}

pub fn library_root(app: &AppHandle) -> Result<PathBuf, String> {
    let settings = load_settings(app)?;
    if let Some(custom) = settings.library_dir_override {
        if !custom.trim().is_empty() {
            return Ok(PathBuf::from(custom));
        }
    }
    Ok(app_data_dir(app)?.join(LIBRARY_SUBDIR))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastWallpaper {
    pub id: String,
    pub title: String,
    pub media_type: String,
    pub uri: String,
    pub source: String,
}

pub fn save_last_wallpaper(app: &AppHandle, last: &LastWallpaper) -> Result<(), String> {
    let path = last_wallpaper_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建数据目录失败: {e}"))?;
    }
    let raw = serde_json::to_string_pretty(last).map_err(|e| format!("序列化失败: {e}"))?;
    fs::write(&path, raw).map_err(|e| format!("写入 last_wallpaper.json 失败: {e}"))
}

pub fn load_last_wallpaper(app: &AppHandle) -> Result<Option<LastWallpaper>, String> {
    let path = last_wallpaper_path(app)?;
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).map_err(|e| format!("读取 last_wallpaper.json 失败: {e}"))?;
    let value: LastWallpaper =
        serde_json::from_str(&raw).map_err(|e| format!("解析 last_wallpaper.json 失败: {e}"))?;
    Ok(Some(value))
}

pub fn clear_last_wallpaper(app: &AppHandle) -> Result<(), String> {
    let path = last_wallpaper_path(app)?;
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("删除 last_wallpaper.json 失败: {e}"))?;
    }
    Ok(())
}
