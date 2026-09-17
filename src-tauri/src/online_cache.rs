//! Cache online wallpapers under `{library_root}/.onlinefile/` (not part of local library index).

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

const ONLINE_CACHE_DIR: &str = ".onlinefile";
const PROGRESS_EVENT: &str = "online-download-progress";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheOnlinePayload {
    /// Wallpaper id, e.g. `online-12`
    pub id: String,
    /// Remote media URL (http/https). May be the site download gateway
    /// (`…/download`) which 302s to a signed object URL.
    pub url: String,
    /// Optional `Bearer …` / full Authorization header value
    #[serde(default)]
    pub authorization: Option<String>,
    /// Preferred extension without dot, e.g. `mp4`
    #[serde(default)]
    pub file_ext: Option<String>,
    /// When true, always hit `url` first (no redirect follow) so the server can
    /// record a member download even if a local cache already exists.
    #[serde(default)]
    pub record_download: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveOnlinePayload {
    /// Wallpaper id, e.g. `online-12`
    pub id: String,
    /// Download gateway URL (`…/download`) or direct media URL
    pub url: String,
    #[serde(default)]
    pub authorization: Option<String>,
    /// Suggested file name for the save dialog, e.g. `aurora.mp4`
    pub file_name: String,
    /// Hit gateway first to record member download history
    #[serde(default)]
    pub record_download: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadProgressPayload {
    id: String,
    /// Bytes received so far
    downloaded: u64,
    /// Total size when Content-Length is known
    total: Option<u64>,
    /// `resolving` | `downloading` | `done` | `cached`
    phase: &'static str,
}

fn emit_progress(app: &AppHandle, payload: DownloadProgressPayload) {
    let _ = app.emit(PROGRESS_EVENT, payload);
}

fn online_cache_root(app: &AppHandle) -> Result<PathBuf, String> {
    let root = crate::settings::library_root(app)?;
    Ok(root.join(ONLINE_CACHE_DIR))
}

fn sanitize_file_stem(id: &str) -> String {
    let mut out = String::with_capacity(id.len());
    for ch in id.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "online".into()
    } else {
        out
    }
}

fn ext_from_url(url: &str) -> Option<String> {
    let path = url.split('?').next().unwrap_or(url);
    let name = path.rsplit('/').next().unwrap_or("");
    let ext = name.rsplit('.').next().unwrap_or("");
    if ext.is_empty() || ext.len() > 8 || ext == name {
        return None;
    }
    if ext.chars().all(|c| c.is_ascii_alphanumeric()) {
        Some(ext.to_ascii_lowercase())
    } else {
        None
    }
}

fn ext_from_content_type(ct: &str) -> Option<&'static str> {
    let main = ct.split(';').next().unwrap_or(ct).trim().to_ascii_lowercase();
    match main.as_str() {
        "video/mp4" => Some("mp4"),
        "video/webm" => Some("webm"),
        "image/gif" => Some("gif"),
        "image/png" => Some("png"),
        "image/jpeg" | "image/jpg" => Some("jpg"),
        "image/webp" => Some("webp"),
        _ => None,
    }
}

fn resolve_ext(
    preferred: Option<&str>,
    fetch_url: &str,
    original_url: &str,
    content_type: Option<&str>,
) -> String {
    if let Some(e) = preferred
        .map(|s| s.trim().trim_start_matches('.').to_ascii_lowercase())
        .filter(|s| !s.is_empty() && s.len() <= 8)
    {
        return e;
    }
    if let Some(e) = ext_from_url(fetch_url) {
        return e;
    }
    if let Some(e) = content_type.and_then(ext_from_content_type) {
        return e.to_string();
    }
    if let Some(e) = ext_from_url(original_url) {
        return e;
    }
    "mp4".into()
}

fn find_existing_cache(dir: &Path, stem: &str) -> Option<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return None;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if name == stem {
            if let Ok(meta) = entry.metadata() {
                if meta.len() > 0 {
                    return Some(path);
                }
            }
        }
    }
    None
}

fn authorization_value(raw: &str) -> String {
    let auth = raw.trim();
    if auth.to_ascii_lowercase().starts_with("bearer ") {
        auth.to_string()
    } else {
        format!("Bearer {auth}")
    }
}

/// GET `url` without following redirects. Used to hit `/download` so the API can
/// write download history; returns the `Location` when present.
async fn probe_download_gateway(url: &str, authorization: Option<&str>) -> Result<Option<String>, String> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建下载客户端失败: {e}"))?;

    let mut req = client.get(url);
    if let Some(auth) = authorization.map(str::trim).filter(|s| !s.is_empty()) {
        req = req.header(reqwest::header::AUTHORIZATION, authorization_value(auth));
    }

    let response = req.send().await.map_err(|e| format!("获取下载地址失败: {e}"))?;
    let status = response.status();
    if status.as_u16() == 429 {
        return Err("下载请求过于频繁，请稍后再试".into());
    }
    if status.as_u16() == 404 {
        return Err("壁纸不存在或未上架".into());
    }
    if !(status.is_redirection() || status.is_success()) {
        return Err(format!("获取下载地址失败: HTTP {status}"));
    }

    let location = response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    Ok(location)
}

fn resolve_fetch_url(gateway_url: &str, location: Option<String>) -> String {
    let Some(loc) = location.filter(|s| !s.trim().is_empty()) else {
        return gateway_url.to_string();
    };
    let loc = loc.trim();
    if loc.starts_with("http://") || loc.starts_with("https://") {
        return loc.to_string();
    }
    // Relative Location — join against gateway origin
    if let Ok(base) = reqwest::Url::parse(gateway_url) {
        if let Ok(joined) = base.join(loc) {
            return joined.to_string();
        }
    }
    loc.to_string()
}

struct StreamDownloadResult {
    #[allow(dead_code)]
    downloaded: u64,
    #[allow(dead_code)]
    total: Option<u64>,
    content_type: Option<String>,
}

/// Stream `fetch_url` into `dest`, writing via a `.part` temp file. Emits progress.
async fn stream_url_to_file(
    app: &AppHandle,
    progress_id: &str,
    fetch_url: &str,
    authorization: Option<&str>,
    dest: &Path,
) -> Result<StreamDownloadResult, String> {
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|e| format!("创建下载客户端失败: {e}"))?;

    let mut req = client.get(fetch_url);
    if let Some(auth) = authorization.map(str::trim).filter(|s| !s.is_empty()) {
        req = req.header(reqwest::header::AUTHORIZATION, authorization_value(auth));
    }

    let mut response = req.send().await.map_err(|e| format!("下载失败: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("下载失败: HTTP {}", response.status()));
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let total = response.content_length();

    let tmp = dest
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!(
            "{}.part",
            dest.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("download.bin")
        ));

    emit_progress(
        app,
        DownloadProgressPayload {
            id: progress_id.to_string(),
            downloaded: 0,
            total,
            phase: "downloading",
        },
    );

    let mut file = fs::File::create(&tmp).map_err(|e| format!("写入临时文件失败: {e}"))?;
    let mut downloaded: u64 = 0;
    let mut last_emit = Instant::now() - Duration::from_secs(1);

    loop {
        let chunk = response.chunk().await.map_err(|e| {
            let _ = fs::remove_file(&tmp);
            format!("读取下载内容失败: {e}")
        })?;
        let Some(chunk) = chunk else {
            break;
        };
        if chunk.is_empty() {
            continue;
        }
        file.write_all(&chunk).map_err(|e| {
            let _ = fs::remove_file(&tmp);
            format!("写入临时文件失败: {e}")
        })?;
        downloaded = downloaded.saturating_add(chunk.len() as u64);

        let due = last_emit.elapsed() >= Duration::from_millis(80);
        let finished = total.is_some_and(|t| t > 0 && downloaded >= t);
        if due || finished {
            emit_progress(
                app,
                DownloadProgressPayload {
                    id: progress_id.to_string(),
                    downloaded,
                    total,
                    phase: "downloading",
                },
            );
            last_emit = Instant::now();
        }
    }

    if downloaded == 0 {
        let _ = fs::remove_file(&tmp);
        return Err("下载内容为空".into());
    }

    file.sync_all().ok();
    drop(file);

    if dest.exists() {
        let _ = fs::remove_file(dest);
    }
    fs::rename(&tmp, dest).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("保存文件失败: {e}")
    })?;

    emit_progress(
        app,
        DownloadProgressPayload {
            id: progress_id.to_string(),
            downloaded,
            total: total.or(Some(downloaded)),
            phase: "done",
        },
    );

    Ok(StreamDownloadResult {
        downloaded,
        total,
        content_type,
    })
}

/// Download remote media into `{library_root}/.onlinefile/{id}.{ext}`.
/// Reuses an existing non-empty file for the same id. Never writes into library.json.
#[tauri::command]
pub async fn cache_online_wallpaper(
    app: AppHandle,
    payload: CacheOnlinePayload,
) -> Result<String, String> {
    let url = payload.url.trim().to_string();
    if url.is_empty() {
        return Err("缺少下载地址".into());
    }
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Ok(url);
    }

    let stem = sanitize_file_stem(payload.id.trim());
    let progress_id = payload.id.trim().to_string();
    let dir = online_cache_root(&app)?;
    fs::create_dir_all(&dir).map_err(|e| format!("创建 .onlinefile 目录失败: {e}"))?;

    let auth = payload
        .authorization
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    emit_progress(
        &app,
        DownloadProgressPayload {
            id: progress_id.clone(),
            downloaded: 0,
            total: None,
            phase: "resolving",
        },
    );

    let mut fetch_url = url.clone();
    let mut fetch_auth = if payload.record_download {
        None
    } else {
        auth.clone()
    };
    if payload.record_download {
        let location = probe_download_gateway(&url, auth.as_deref()).await?;
        fetch_url = resolve_fetch_url(&url, location);
        fetch_auth = None; // never send Authorization to signed R2 URL
    }

    if let Some(existing) = find_existing_cache(&dir, &stem) {
        emit_progress(
            &app,
            DownloadProgressPayload {
                id: progress_id,
                downloaded: 1,
                total: Some(1),
                phase: "cached",
            },
        );
        return Ok(existing.to_string_lossy().to_string());
    }

    // Probe content-type/ext via a lightweight HEAD isn't reliable on R2; stream once.
    // Use preferred ext first so dest path is known before streaming.
    let preferred = payload.file_ext.clone();
    let guess_ext = resolve_ext(preferred.as_deref(), &fetch_url, &url, None);
    let dest = dir.join(format!("{stem}.{guess_ext}"));

    let result = stream_url_to_file(
        &app,
        &progress_id,
        &fetch_url,
        fetch_auth.as_deref(),
        &dest,
    )
    .await?;

    // If server content-type implies a better ext and file was saved with guess, rename.
    let better = resolve_ext(
        preferred.as_deref(),
        &fetch_url,
        &url,
        result.content_type.as_deref(),
    );
    if better != guess_ext {
        let renamed = dir.join(format!("{stem}.{better}"));
        if renamed != dest {
            let _ = fs::rename(&dest, &renamed);
            return Ok(renamed.to_string_lossy().to_string());
        }
    }

    Ok(dest.to_string_lossy().to_string())
}

/// Pick a save path, then stream the online wallpaper there (with progress).
/// Returns `None` when the user cancels the dialog.
#[tauri::command]
pub async fn save_online_wallpaper(
    app: AppHandle,
    payload: SaveOnlinePayload,
) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let url = payload.url.trim().to_string();
    if url.is_empty() {
        return Err("缺少下载地址".into());
    }
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("下载地址无效".into());
    }

    let file_name = payload.file_name.trim();
    let file_name = if file_name.is_empty() {
        "wallpaper.bin".to_string()
    } else {
        file_name.to_string()
    };

    let progress_id = payload.id.trim().to_string();
    let app_for_dialog = app.clone();
    let suggested = file_name.clone();
    let dest = tauri::async_runtime::spawn_blocking(move || {
        app_for_dialog
            .dialog()
            .file()
            .set_file_name(&suggested)
            .blocking_save_file()
            .and_then(|p| p.into_path().ok())
    })
    .await
    .map_err(|e| format!("打开保存对话框失败: {e}"))?;

    let Some(dest) = dest else {
        return Ok(None);
    };

    let auth = payload
        .authorization
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    emit_progress(
        &app,
        DownloadProgressPayload {
            id: progress_id.clone(),
            downloaded: 0,
            total: None,
            phase: "resolving",
        },
    );

    let mut fetch_url = url.clone();
    let mut fetch_auth = auth.clone();
    if payload.record_download {
        let location = probe_download_gateway(&url, auth.as_deref()).await?;
        fetch_url = resolve_fetch_url(&url, location);
        fetch_auth = None;
    }

    stream_url_to_file(
        &app,
        &progress_id,
        &fetch_url,
        fetch_auth.as_deref(),
        &dest,
    )
    .await?;

    Ok(Some(dest.to_string_lossy().to_string()))
}

/// True if path is inside the `.onlinefile` cache (must not be indexed as local library).
#[allow(dead_code)]
pub fn is_online_cache_path(app: &AppHandle, path: &Path) -> bool {
    let Ok(root) = online_cache_root(app) else {
        return false;
    };
    path.starts_with(&root)
}
