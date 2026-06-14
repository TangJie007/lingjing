//! 壁纸引擎模块
//! 静态壁纸（Windows API）+ 视频壁纸（MPV + desktop_core，参考 Lively Wallpaper）

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};

/// 缩放模式（保留给后续设置壁纸风格时使用）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub enum ScalingMode {
    Fill,
    Fit,
    Stretch,
    Tile,
}

/// 壁纸条目
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WallpaperEntry {
    pub id: String,
    pub filename: String,
    pub path: String,
    pub media_type: String,
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub file_size: u64,
    pub source: String,
    pub created_at: String,
    pub prompt: Option<String>,
    pub plan: Option<String>,
    pub model: Option<String>,
    pub cost: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LibraryMeta {
    wallpapers: Vec<WallpaperEntry>,
}

fn library_dir(app: &AppHandle) -> PathBuf {
    let data_dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    data_dir.join("wallpapers")
}

fn meta_path(app: &AppHandle) -> PathBuf {
    library_dir(app).join("library.json")
}

fn load_meta(app: &AppHandle) -> LibraryMeta {
    let path = meta_path(app);
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(json) => serde_json::from_str(&json).unwrap_or(LibraryMeta {
                wallpapers: vec![],
            }),
            Err(_) => LibraryMeta {
                wallpapers: vec![],
            },
        }
    } else {
        LibraryMeta {
            wallpapers: vec![],
        }
    }
}

fn save_meta(app: &AppHandle, meta: &LibraryMeta) {
    let path = meta_path(app);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(meta) {
        let _ = fs::write(path, json);
    }
}

fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("wp_{:x}", ts)
}

fn ensure_library_dir(app: &AppHandle) -> PathBuf {
    let dir = library_dir(app);
    let _ = fs::create_dir_all(&dir);
    dir
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("{}", ts)
}

fn dir_size(path: &PathBuf) -> u64 {
    fn walk(path: &PathBuf, total: &mut u64) {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    *total += entry.metadata().map(|m| m.len()).unwrap_or(0);
                } else if p.is_dir() {
                    walk(&p, total);
                }
            }
        }
    }
    let mut total = 0;
    walk(path, &mut total);
    total
}

// ============================================================
// Tauri 命令
// ============================================================

#[tauri::command]
pub fn list_wallpapers(app: AppHandle) -> Vec<WallpaperEntry> {
    load_meta(&app).wallpapers
}

#[tauri::command]
pub fn import_wallpaper(app: AppHandle, source_path: String) -> Result<WallpaperEntry, String> {
    let src = PathBuf::from(&source_path);
    if !src.exists() {
        return Err("文件不存在".into());
    }

    let filename = src
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let ext = src
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();

    let media_type = match ext.as_str() {
        "mp4" | "webm" | "mov" | "mkv" | "avi" => "video",
        "gif" => "gif",
        _ => "image",
    };

    let dir = ensure_library_dir(&app);
    let id = generate_id();
    let dest_filename = format!("{}_{}", id, filename);
    let dest = dir.join(&dest_filename);

    fs::copy(&src, &dest).map_err(|e| format!("复制文件失败: {}", e))?;

    let file_size = fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);

    let entry = WallpaperEntry {
        id: id.clone(),
        filename: dest_filename,
        path: dest.to_string_lossy().to_string(),
        media_type: media_type.to_string(),
        format: ext,
        width: 0,
        height: 0,
        file_size,
        source: "local".to_string(),
        created_at: chrono_now(),
        prompt: None,
        plan: None,
        model: None,
        cost: None,
    };

    let mut meta = load_meta(&app);
    meta.wallpapers.push(entry.clone());
    save_meta(&app, &meta);

    Ok(entry)
}

#[tauri::command]
pub fn delete_wallpaper(app: AppHandle, id: String) -> Result<(), String> {
    let mut meta = load_meta(&app);
    if let Some(entry) = meta.wallpapers.iter().find(|w| w.id == id) {
        let path = PathBuf::from(&entry.path);
        if path.exists() {
            let _ = fs::remove_file(&path);
        }
    }
    meta.wallpapers.retain(|w| w.id != id);
    save_meta(&app, &meta);
    Ok(())
}

#[tauri::command]
pub fn export_wallpaper(app: AppHandle, id: String, dest_path: String) -> Result<(), String> {
    let meta = load_meta(&app);
    let entry = meta
        .wallpapers
        .iter()
        .find(|w| w.id == id)
        .ok_or("壁纸不存在")?;

    let src = PathBuf::from(&entry.path);
    let dest = PathBuf::from(&dest_path);
    fs::copy(&src, &dest).map_err(|e| format!("导出失败: {}", e))?;
    Ok(())
}

pub fn init_desktop_player(app: &AppHandle) {
    if let Err(e) = ensure_desktop_player_window(app) {
        log::warn!("预创建桌面播放器失败: {}", e);
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetWallpaperResult {
    pub conflicts: Vec<crate::desktop_core::DesktopLayerConflict>,
}

#[tauri::command]
pub fn check_desktop_layer_conflicts() -> Vec<crate::desktop_core::DesktopLayerConflict> {
    crate::desktop_core::detect_desktop_layer_conflicts()
}

#[tauri::command]
pub fn set_wallpaper(
    app: AppHandle,
    id: String,
    state: tauri::State<'_, CurrentWallpaperState>,
) -> Result<SetWallpaperResult, String> {
    let meta = load_meta(&app);
    let entry = meta
        .wallpapers
        .iter()
        .find(|w| w.id == id)
        .ok_or("壁纸不存在")?;

    let path = PathBuf::from(&entry.path);
    if !path.exists() {
        return Err(format!("壁纸文件不存在: {}", path.display()));
    }

    if entry.media_type == "video" {
        {
            let mut guard = state.0.lock().unwrap();
            *guard = Some(("video".into(), path.to_string_lossy().to_string(), id.clone()));
        }

        hide_desktop_player_window(&app);

        let path_for_thread = path.clone();
        let app_for_thread = app.clone();
        let (tx, rx) = std::sync::mpsc::sync_channel::<Result<(), String>>(1);
        app.clone()
            .run_on_main_thread(move || {
                let vp = app_for_thread.state::<crate::video_player::VideoPlayerState>();
                let result =
                    crate::video_player::switch_or_start_video_wallpaper(&vp, &path_for_thread);
                let _ = tx.send(result);
            })
            .map_err(|e| format!("调度主线程失败: {}", e))?;
        rx.recv()
            .map_err(|e| format!("主线程无响应: {}", e))??;

        let _ = app.emit(
            "wallpaper-changed",
            serde_json::json!({
                "type": "video",
                "path": path.to_string_lossy(),
                "id": id,
                "engine": "mpv"
            }),
        );
    } else {
        let was_video = {
            let guard = state.0.lock().unwrap();
            guard
                .as_ref()
                .map(|(t, _, _)| t == "video")
                .unwrap_or(false)
        };

        {
            let mut guard = state.0.lock().unwrap();
            *guard = Some(("image".into(), path.to_string_lossy().to_string(), id.clone()));
        }

        if was_video {
            let vp = app.state::<crate::video_player::VideoPlayerState>();
            crate::video_player::stop_video_wallpaper(&vp);
            hide_desktop_player_window(&app);
        } else {
            hide_desktop_player_window(&app);
        }
        set_windows_wallpaper(&path)?;
        let _ = app.emit_to(
            DESKTOP_PLAYER_LABEL,
            "wallpaper-changed",
            serde_json::json!({ "type": "image", "path": "", "id": id }),
        );
    }

    let app_meta = app.clone();
    let meta_id = id.clone();
    std::thread::spawn(move || {
        let mut meta = load_meta(&app_meta);
        for w in &mut meta.wallpapers {
            if w.id == meta_id {
                w.created_at = chrono_now();
            }
        }
        save_meta(&app_meta, &meta);
    });

    let app_conflicts = app.clone();
    std::thread::spawn(move || {
        let conflicts = crate::desktop_core::detect_desktop_layer_conflicts();
        if !conflicts.is_empty() {
            for c in &conflicts {
                log::warn!("检测到桌面层级冲突: {} ({})", c.name, c.id);
            }
            let _ = app_conflicts.emit("wallpaper-conflicts", &conflicts);
        }
    });

    Ok(SetWallpaperResult { conflicts: vec![] })
}

#[tauri::command]
pub fn get_library_size(app: AppHandle) -> u64 {
    dir_size(&library_dir(&app))
}

#[tauri::command]
pub fn clear_library(app: AppHandle) -> Result<(), String> {
    let dir = library_dir(&app);
    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|e| format!("清除失败: {}", e))?;
    }
    let _ = fs::create_dir_all(&dir);
    save_meta(
        &app,
        &LibraryMeta {
            wallpapers: vec![],
        },
    );
    Ok(())
}

#[tauri::command]
pub fn is_fullscreen_app_running() -> bool {
    #[cfg(target_os = "windows")]
    {
        detect_fullscreen_windows()
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

#[tauri::command]
pub fn get_current_wallpaper(
    state: tauri::State<'_, CurrentWallpaperState>,
) -> Option<serde_json::Value> {
    let guard = state.0.lock().unwrap();
    guard
        .as_ref()
        .map(|(t, p, id)| serde_json::json!({ "type": t, "path": p, "id": id }))
}

#[tauri::command]
pub fn attach_desktop_player(
    app: AppHandle,
    state: tauri::State<'_, CurrentWallpaperState>,
    vp: tauri::State<'_, crate::video_player::VideoPlayerState>,
) -> Result<(), String> {
    if !has_active_video_wallpaper(&state) {
        hide_desktop_player_window(&app);
        crate::video_player::stop_video_wallpaper(&vp);
        return Ok(());
    }
    run_reattach_on_main_thread(&app)
}

fn run_reattach_on_main_thread(app: &AppHandle) -> Result<(), String> {
    let handle = app.clone();
    let handle_in_closure = handle.clone();
    let (tx, rx) = std::sync::mpsc::sync_channel::<Result<(), String>>(1);
    handle
        .run_on_main_thread(move || {
            let vp = handle_in_closure.state::<crate::video_player::VideoPlayerState>();
            let result = crate::video_player::reattach_if_running(&vp);
            let _ = tx.send(result);
        })
        .map_err(|e| format!("调度主线程失败: {}", e))?;
    rx.recv().map_err(|e| format!("主线程无响应: {}", e))?
}

pub struct CurrentWallpaperState(pub std::sync::Mutex<Option<(String, String, String)>>);

impl Default for CurrentWallpaperState {
    fn default() -> Self {
        Self(std::sync::Mutex::new(None))
    }
}

#[cfg(target_os = "windows")]
fn set_windows_wallpaper(path: &PathBuf) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let path_str = path.to_string_lossy().to_string();
    let wide: Vec<u16> = OsStr::new(&path_str)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{
            SystemParametersInfoW, SPI_SETDESKWALLPAPER, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE,
        };

        // 先即时生效，再异步持久化，减少桌面卡顿感
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            Some(wide.as_ptr() as *mut _),
            SPIF_SENDCHANGE,
        )
        .map_err(|e| format!("设置桌面壁纸失败: {:?}", e))?;

        let wide_persist = wide.clone();
        std::thread::spawn(move || {
            let _ = SystemParametersInfoW(
                SPI_SETDESKWALLPAPER,
                0,
                Some(wide_persist.as_ptr() as *mut _),
                SPIF_UPDATEINIFILE,
            );
        });
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn set_windows_wallpaper(_path: &PathBuf) -> Result<(), String> {
    Err("当前平台不支持设置桌面壁纸".into())
}

#[cfg(target_os = "windows")]
fn detect_fullscreen_windows() -> bool {
    use windows::Win32::Foundation::{HWND, RECT};
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowRect};

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd == HWND::default() {
            return false;
        }

        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return false;
        }

        let window_width = rect.right - rect.left;
        let window_height = rect.bottom - rect.top;

        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut monitor_info: MONITORINFO = std::mem::zeroed();
        monitor_info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;

        if GetMonitorInfoW(monitor, &mut monitor_info).0 == 0 {
            return false;
        }

        let monitor_width = monitor_info.rcMonitor.right - monitor_info.rcMonitor.left;
        let monitor_height = monitor_info.rcMonitor.bottom - monitor_info.rcMonitor.top;

        window_width >= monitor_width - 50 && window_height >= monitor_height - 50
    }
}

const DESKTOP_PLAYER_LABEL: &str = "desktop-player";

fn ensure_desktop_player_window(app: &AppHandle) -> Result<(), String> {
    use tauri::{WebviewUrl, WebviewWindowBuilder};

    if app.get_webview_window(DESKTOP_PLAYER_LABEL).is_some() {
        return Ok(());
    }

    let (width, height) = get_primary_monitor_size();

    WebviewWindowBuilder::new(
        app,
        DESKTOP_PLAYER_LABEL,
        WebviewUrl::App("/#/desktop-player".into()),
    )
    .title("灵境 桌面播放器")
    .inner_size(width as f64, height as f64)
    .position(0.0, 0.0)
    .decorations(false)
    .resizable(false)
    .always_on_bottom(false)
    .skip_taskbar(true)
    .focusable(false)
    .visible(false)
    .transparent(false)
    .build()
    .map_err(|e| format!("创建桌面播放器失败: {}", e))?;

    log::info!("桌面播放器窗口已创建（隐藏，视频壁纸使用 MPV）");
    Ok(())
}

fn has_active_video_wallpaper(state: &tauri::State<'_, CurrentWallpaperState>) -> bool {
    let guard = state.0.lock().unwrap();
    guard
        .as_ref()
        .map(|(t, _, _)| t == "video")
        .unwrap_or(false)
}

fn hide_desktop_player_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(DESKTOP_PLAYER_LABEL) {
        let _ = window.hide();
    }
}

fn get_primary_monitor_size() -> (u32, u32) {
    #[cfg(target_os = "windows")]
    {
        unsafe {
            use windows::Win32::UI::WindowsAndMessaging::{
                GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN,
            };
            (GetSystemMetrics(SM_CXSCREEN) as u32, GetSystemMetrics(SM_CYSCREEN) as u32)
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        (1920, 1080)
    }
}
