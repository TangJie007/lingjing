//! Cache online wallpapers under `{library_root}/.onlinefile/` (not part of local library index).

use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::AppHandle;

const ONLINE_CACHE_DIR: &str = ".onlinefile";

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

fn resolve_ext(payload: &CacheOnlinePayload, fetch_url: &str, content_type: Option<&str>) -> String {
    if let Some(e) = payload
        .file_ext
        .as_ref()
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
    if let Some(e) = ext_from_url(&payload.url) {
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

    let response = req.send().await.map_err(|e| format!("下载失败: {e}"))?;
    let status = response.status();
    if status.as_u16() == 429 {
        return Err("下载请求过于频繁，请稍后再试".into());
    }
    if status.as_u16() == 404 {
        return Err("壁纸不存在或未上架".into());
    }
    if !(status.is_redirection() || status.is_success()) {
        return Err(format!("下载失败: HTTP {status}"));
    }

    let location = response
        .headers()
        .get(reqwest::header::LOCATION)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    Ok(location)
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
        // Already a local path / asset — return as-is
        return Ok(url);
    }

    let stem = sanitize_file_stem(payload.id.trim());
    let dir = online_cache_root(&app)?;
    fs::create_dir_all(&dir).map_err(|e| format!("创建 .onlinefile 目录失败: {e}"))?;

    let auth = payload
        .authorization
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    // Resolve gateway → signed object URL (and record member download when JWT present).
    let mut fetch_url = url.clone();
    if payload.record_download {
        let location = probe_download_gateway(&url, auth.as_deref()).await?;
        if let Some(loc) = location.filter(|s| !s.trim().is_empty()) {
            fetch_url = loc;
        }
    }

    if let Some(existing) = find_existing_cache(&dir, &stem) {
        return Ok(existing.to_string_lossy().to_string());
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .build()
        .map_err(|e| format!("创建下载客户端失败: {e}"))?;

    // Presigned R2 URL must not carry Authorization — it invalidates the signature.
    // Only attach auth when still hitting our own gateway (record_download already resolved).
    let mut req = client.get(&fetch_url);
    if !payload.record_download {
        if let Some(ref a) = auth {
            req = req.header(reqwest::header::AUTHORIZATION, authorization_value(a));
        }
    }

    let response = req
        .send()
        .await
        .map_err(|e| format!("下载失败: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("下载失败: HTTP {}", response.status()));
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let ext = resolve_ext(&payload, &fetch_url, content_type.as_deref());
    let dest = dir.join(format!("{stem}.{ext}"));
    let tmp = dir.join(format!("{stem}.{ext}.part"));

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("读取下载内容失败: {e}"))?;
    if bytes.is_empty() {
        return Err("下载内容为空".into());
    }

    {
        let mut file =
            fs::File::create(&tmp).map_err(|e| format!("写入临时文件失败: {e}"))?;
        file.write_all(&bytes)
            .map_err(|e| format!("写入临时文件失败: {e}"))?;
        file.sync_all().ok();
    }
    fs::rename(&tmp, &dest).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("保存缓存失败: {e}")
    })?;

    Ok(dest.to_string_lossy().to_string())
}

/// True if path is inside the `.onlinefile` cache (must not be indexed as local library).
#[allow(dead_code)]
pub fn is_online_cache_path(app: &AppHandle, path: &Path) -> bool {
    let Ok(root) = online_cache_root(app) else {
        return false;
    };
    path.starts_with(&root)
}
