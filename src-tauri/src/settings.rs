use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};

const SETTINGS_FILE: &str = "settings.json";
const VERSION_RECORD_FILE: &str = "version_record.json";
const LIBRARY_SUBDIR: &str = "library";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionRecord {
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default)]
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
    #[serde(default)]
    pub library_dir_override: Option<String>,
    #[serde(default = "default_loop_mode")]
    pub loop_mode: String,
    #[serde(default)]
    pub online_enabled: bool,
    #[serde(default)]
    pub desktop_organize_enabled: bool,
    #[serde(default = "default_api_base_url")]
    pub api_base_url: String,
    /// Set only after the user successfully applies a wallpaper. Absent means never applied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_wallpaper: Option<LastWallpaper>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            autostart: false,
            hide_icons_on_double_click: false,
            pause_on_fullscreen: true,
            pause_on_battery: true,
            pause_on_rdp: true,
            sound_on: true,
            default_volume: 0.0,
            library_dir_override: None,
            loop_mode: default_loop_mode(),
            online_enabled: false,
            desktop_organize_enabled: false,
            api_base_url: default_api_base_url(),
            last_wallpaper: None,
        }
    }
}

fn default_true() -> bool {
    true
}
fn default_volume() -> f64 {
    0.0
}
fn default_loop_mode() -> String {
    "single".into()
}
const DEV_API_BASE_URL: &str = "http://localhost:3080";
const PROD_API_BASE_URL: &str = "https://36fa666671.eicp.vip";

/// debug 构建 → 本地 Gateway 3080；release 打包 → 线上域名
fn default_api_base_url() -> String {
    if cfg!(debug_assertions) {
        DEV_API_BASE_URL.into()
    } else {
        PROD_API_BASE_URL.into()
    }
}

fn normalize_api_base_url(url: &str) -> String {
    let trimmed = url.trim().trim_end_matches('/');
    if trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("http://localhost:3002")
        || trimmed.eq_ignore_ascii_case("https://localhost:3002")
        || trimmed.eq_ignore_ascii_case("http://localhost:8000")
        || trimmed.eq_ignore_ascii_case("http://127.0.0.1:8000")
    {
        return default_api_base_url();
    }
    if cfg!(debug_assertions) {
        if trimmed.eq_ignore_ascii_case(PROD_API_BASE_URL)
            || trimmed.eq_ignore_ascii_case("http://36fa666671.eicp.vip")
        {
            return DEV_API_BASE_URL.into();
        }
    } else if trimmed.eq_ignore_ascii_case(DEV_API_BASE_URL)
        || trimmed.eq_ignore_ascii_case("http://127.0.0.1:3080")
        || trimmed.eq_ignore_ascii_case("http://localhost:8000")
        || trimmed.eq_ignore_ascii_case("http://127.0.0.1:8000")
    {
        return PROD_API_BASE_URL.into();
    }
    trimmed.to_string()
}

fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| format!("无法解析应用数据目录: {e}"))
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join(SETTINGS_FILE))
}

fn version_record_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join(VERSION_RECORD_FILE))
}

pub fn has_version_record(app: &AppHandle) -> bool {
    version_record_path(app)
        .map(|p| p.exists())
        .unwrap_or(false)
}

pub fn write_version_record(app: &AppHandle, version: &str) -> Result<(), String> {
    let path = version_record_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建数据目录失败: {e}"))?;
    }
    let record = VersionRecord {
        version: version.to_string(),
    };
    crate::util::write_json_atomic(&path, &record)
}

pub fn load_settings(app: &AppHandle) -> Result<AppSettings, String> {
    let path = settings_path(app)?;
    let mut settings = if !path.exists() {
        AppSettings::default()
    } else {
        let raw = fs::read_to_string(&path).map_err(|e| format!("读取 settings.json 失败: {e}"))?;
        let value: serde_json::Value =
            serde_json::from_str(&raw).map_err(|e| format!("解析 settings.json 失败: {e}"))?;
        let mut settings: AppSettings = serde_json::from_value(value).unwrap_or_default();
        settings.api_base_url = normalize_api_base_url(&settings.api_base_url);
        settings
    };
    if settings.last_wallpaper.is_none() {
        if let Some(legacy) = read_legacy_last_wallpaper(app) {
            settings.last_wallpaper = Some(legacy);
            let _ = write_settings_file(app, &settings);
            if let Ok(dir) = app_data_dir(app) {
                let _ = fs::remove_file(dir.join("last_wallpaper.json"));
            }
        }
    }
    Ok(settings)
}

/// Frontend saves the whole settings object and does not know about last wallpaper.
/// Keep the on-disk value so a volume/toggle save cannot wipe it.
pub fn save_settings(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let mut next = settings.clone();
    next.last_wallpaper = load_settings(app).ok().and_then(|s| s.last_wallpaper);
    write_settings_file(app, &next)
}

fn write_settings_file(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    crate::util::write_json_atomic(&path, settings)
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
    let mut settings = load_settings(app).unwrap_or_default();
    settings.last_wallpaper = Some(last.clone());
    write_settings_file(app, &settings)
}

pub fn load_last_wallpaper(app: &AppHandle) -> Result<Option<LastWallpaper>, String> {
    Ok(load_settings(app)?.last_wallpaper)
}

pub fn clear_last_wallpaper(app: &AppHandle) -> Result<(), String> {
    let mut settings = load_settings(app).unwrap_or_default();
    settings.last_wallpaper = None;
    write_settings_file(app, &settings)?;
    if let Ok(dir) = app_data_dir(app) {
        let legacy = dir.join("last_wallpaper.json");
        if legacy.exists() {
            let _ = fs::remove_file(legacy);
        }
    }
    Ok(())
}

fn read_legacy_last_wallpaper(app: &AppHandle) -> Option<LastWallpaper> {
    let path = app_data_dir(app).ok()?.join("last_wallpaper.json");
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

/// True only after the user has successfully applied a wallpaper.
pub fn has_applied_wallpaper(app: &AppHandle) -> bool {
    load_last_wallpaper(app).ok().flatten().is_some()
}
