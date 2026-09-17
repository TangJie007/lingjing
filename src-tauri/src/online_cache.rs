//! Cache online wallpapers under `{library_root}/.onlinefile/` (not part of local library index).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

const ONLINE_CACHE_DIR: &str = ".onlinefile";
const LIST_FILE: &str = "list.json";
const PROGRESS_EVENT: &str = "online-download-progress";
/// Parallel Range parts when the object is large enough.
const MULTIPART_PARTS: u64 = 6;
/// Below this size, single-stream is usually fine / cheaper to probe.
const MULTIPART_MIN_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnlineFileListItem {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub file_name: String,
    /// Absolute path of the cached file (primary lookup for set-wallpaper).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_path: Option<String>,
    #[serde(default)]
    pub file_size: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    pub downloaded_at: String,
    /// User-chosen export path from the save dialog (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub export_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OnlineFileListFile {
    version: u32,
    items: Vec<OnlineFileListItem>,
}

impl Default for OnlineFileListFile {
    fn default() -> Self {
        Self {
            version: 1,
            items: Vec::new(),
        }
    }
}

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
    /// Download gateway URL (`…/download`). With `record_download`, probed
    /// (no redirect follow) for history; bytes come from the 302 `Location`
    /// unless `fetch_url` overrides.
    pub url: String,
    /// Optional override for the byte stream URL. Prefer leaving unset so
    /// downloads follow `/download` → Location.
    #[serde(default)]
    pub fetch_url: Option<String>,
    #[serde(default)]
    pub authorization: Option<String>,
    /// Preferred extension without dot (also used when guessing cache file name)
    #[serde(default)]
    pub file_ext: Option<String>,
    /// Optional display name hint (kept for API compat; cache path uses id stem)
    #[serde(default)]
    #[allow(dead_code)]
    pub file_name: String,
    /// Display title stored in list.json
    #[serde(default)]
    pub title: Option<String>,
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

fn cancel_flags() -> &'static Mutex<HashMap<String, Arc<AtomicBool>>> {
    static FLAGS: OnceLock<Mutex<HashMap<String, Arc<AtomicBool>>>> = OnceLock::new();
    FLAGS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn register_cancel_flag(id: &str) -> Arc<AtomicBool> {
    let flag = Arc::new(AtomicBool::new(false));
    if let Ok(mut map) = cancel_flags().lock() {
        map.insert(id.to_string(), Arc::clone(&flag));
    }
    flag
}

fn unregister_cancel_flag(id: &str) {
    if let Ok(mut map) = cancel_flags().lock() {
        map.remove(id);
    }
}

fn is_cancelled(flag: &AtomicBool) -> bool {
    flag.load(Ordering::SeqCst)
}

/// Request cancellation of an in-flight online download by wallpaper id.
#[tauri::command]
pub fn cancel_online_download(id: String) -> Result<bool, String> {
    let id = id.trim();
    if id.is_empty() {
        return Ok(false);
    }
    let mut hit = false;
    {
        let map = cancel_flags()
            .lock()
            .map_err(|_| "取消下载锁失败".to_string())?;
        if let Some(flag) = map.get(id) {
            flag.store(true, Ordering::SeqCst);
            hit = true;
        }
    }
    if abort_online_file_write_inner(id).is_ok() {
        hit = true;
    }
    Ok(hit)
}

struct ActiveFileWrite {
    part_path: PathBuf,
    dest_path: PathBuf,
    writer: BufWriter<fs::File>,
    downloaded: u64,
    title: Option<String>,
    preferred_ext: String,
}

fn active_file_writes() -> &'static Mutex<HashMap<String, ActiveFileWrite>> {
    static WRITES: OnceLock<Mutex<HashMap<String, ActiveFileWrite>>> = OnceLock::new();
    WRITES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn abort_online_file_write_inner(id: &str) -> Result<(), String> {
    let mut map = active_file_writes()
        .lock()
        .map_err(|_| "下载写盘锁失败".to_string())?;
    if let Some(active) = map.remove(id) {
        drop(active.writer);
        let _ = fs::remove_file(&active.part_path);
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeginOnlineWritePayload {
    pub id: String,
    #[serde(default)]
    pub file_ext: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BeginOnlineWriteResult {
    pub cached: bool,
    pub path: String,
}

/// Start a chunked write into `.onlinefile` (used by plugin-http downloads).
#[tauri::command]
pub fn begin_online_file_write(
    app: AppHandle,
    payload: BeginOnlineWritePayload,
) -> Result<BeginOnlineWriteResult, String> {
    let progress_id = payload.id.trim().to_string();
    if progress_id.is_empty() {
        return Err("缺少壁纸 ID".into());
    }
    let dir = online_cache_root(&app)?;
    fs::create_dir_all(&dir).map_err(|e| format!("创建 .onlinefile 目录失败: {e}"))?;

    let title = payload
        .title
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());

    if let Some(existing) = resolve_cached_path(&dir, &progress_id) {
        upsert_list_item(&dir, &progress_id, title, &existing, None, None)?;
        return Ok(BeginOnlineWriteResult {
            cached: true,
            path: existing.to_string_lossy().to_string(),
        });
    }

    let ext = payload
        .file_ext
        .as_deref()
        .map(|s| s.trim().trim_start_matches('.').to_ascii_lowercase())
        .filter(|s| !s.is_empty() && s.len() <= 8)
        .unwrap_or_else(|| "bin".into());
    let stem = sanitize_file_stem(&progress_id);
    let dest = dir.join(format!("{stem}.{ext}"));
    let part = dir.join(format!(
        "{}.part",
        dest.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("download.bin")
    ));
    let _ = fs::remove_file(&part);
    let file = fs::File::create(&part).map_err(|e| format!("写入临时文件失败: {e}"))?;
    let writer = BufWriter::with_capacity(256 * 1024, file);

    let mut map = active_file_writes()
        .lock()
        .map_err(|_| "下载写盘锁失败".to_string())?;
    if let Some(prev) = map.remove(&progress_id) {
        drop(prev.writer);
        let _ = fs::remove_file(&prev.part_path);
    }
    map.insert(
        progress_id,
        ActiveFileWrite {
            part_path: part,
            dest_path: dest.clone(),
            writer,
            downloaded: 0,
            title: title.map(|s| s.to_string()),
            preferred_ext: ext,
        },
    );

    Ok(BeginOnlineWriteResult {
        cached: false,
        path: dest.to_string_lossy().to_string(),
    })
}

/// Append a chunk from plugin-http streaming download (base64 to avoid huge JSON number arrays).
#[tauri::command]
pub fn append_online_file_write(id: String, chunk_base64: String) -> Result<u64, String> {
    let id = id.trim();
    if id.is_empty() {
        return Err("缺少壁纸 ID".into());
    }
    let raw = chunk_base64.trim();
    if raw.is_empty() {
        let map = active_file_writes()
            .lock()
            .map_err(|_| "下载写盘锁失败".to_string())?;
        return Ok(map.get(id).map(|w| w.downloaded).unwrap_or(0));
    }
    let chunk = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, raw)
        .map_err(|e| format!("解码下载数据失败: {e}"))?;
    if chunk.is_empty() {
        let map = active_file_writes()
            .lock()
            .map_err(|_| "下载写盘锁失败".to_string())?;
        return Ok(map.get(id).map(|w| w.downloaded).unwrap_or(0));
    }
    let mut map = active_file_writes()
        .lock()
        .map_err(|_| "下载写盘锁失败".to_string())?;
    let active = map
        .get_mut(id)
        .ok_or_else(|| "下载会话不存在或已结束".to_string())?;
    active
        .writer
        .write_all(&chunk)
        .map_err(|e| format!("写入临时文件失败: {e}"))?;
    active.downloaded = active.downloaded.saturating_add(chunk.len() as u64);
    Ok(active.downloaded)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinishOnlineWritePayload {
    pub id: String,
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub fetch_url: Option<String>,
}

/// Finalize chunked write: flush, rename, update list.json.
#[tauri::command]
pub fn finish_online_file_write(
    app: AppHandle,
    payload: FinishOnlineWritePayload,
) -> Result<String, String> {
    let progress_id = payload.id.trim().to_string();
    if progress_id.is_empty() {
        return Err("缺少壁纸 ID".into());
    }
    let mut map = active_file_writes()
        .lock()
        .map_err(|_| "下载写盘锁失败".to_string())?;
    let mut active = map
        .remove(&progress_id)
        .ok_or_else(|| "下载会话不存在或已结束".to_string())?;

    if active.downloaded == 0 {
        drop(active.writer);
        let _ = fs::remove_file(&active.part_path);
        return Err("下载内容为空".into());
    }

    active.writer.flush().map_err(|e| {
        let _ = fs::remove_file(&active.part_path);
        format!("写入临时文件失败: {e}")
    })?;
    let file = active.writer.into_inner().map_err(|e| {
        let _ = fs::remove_file(&active.part_path);
        format!("写入临时文件失败: {e}")
    })?;
    file.sync_all().ok();
    drop(file);

    let dir = online_cache_root(&app)?;
    let fetch_url = payload.fetch_url.as_deref().unwrap_or("");
    let better = resolve_ext(
        Some(&active.preferred_ext),
        fetch_url,
        fetch_url,
        payload.content_type.as_deref(),
    );
    let mut dest = active.dest_path;
    if better != active.preferred_ext {
        let stem = sanitize_file_stem(&progress_id);
        dest = dir.join(format!("{stem}.{better}"));
    }

    if dest.exists() {
        let _ = fs::remove_file(&dest);
    }
    fs::rename(&active.part_path, &dest).map_err(|e| {
        let _ = fs::remove_file(&active.part_path);
        format!("保存文件失败: {e}")
    })?;

    upsert_list_item(
        &dir,
        &progress_id,
        active.title.as_deref(),
        &dest,
        payload.content_type.as_deref(),
        None,
    )?;

    Ok(dest.to_string_lossy().to_string())
}

/// Abort an in-progress chunked write (plugin-http path).
#[tauri::command]
pub fn abort_online_file_write(id: String) -> Result<(), String> {
    abort_online_file_write_inner(id.trim())
}

fn online_cache_root(app: &AppHandle) -> Result<PathBuf, String> {
    let root = crate::settings::library_root(app)?;
    Ok(root.join(ONLINE_CACHE_DIR))
}

fn list_json_path(dir: &Path) -> PathBuf {
    dir.join(LIST_FILE)
}

fn read_list_file(dir: &Path) -> OnlineFileListFile {
    let path = list_json_path(dir);
    let Ok(raw) = fs::read_to_string(&path) else {
        return OnlineFileListFile::default();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

fn write_list_file(dir: &Path, list: &OnlineFileListFile) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| format!("创建 .onlinefile 目录失败: {e}"))?;
    let path = list_json_path(dir);
    let tmp = dir.join(format!("{LIST_FILE}.tmp"));
    let raw = serde_json::to_string_pretty(list).map_err(|e| format!("序列化 list.json 失败: {e}"))?;
    fs::write(&tmp, raw).map_err(|e| format!("写入 list.json 失败: {e}"))?;
    fs::rename(&tmp, &path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("保存 list.json 失败: {e}")
    })
}

fn now_rfc3339() -> String {
    // Local wall-clock ISO-like stamp without extra deps.
    use std::time::SystemTime;
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

fn path_is_nonempty_file(p: &Path) -> bool {
    p.is_file() && fs::metadata(p).map(|m| m.len() > 0).unwrap_or(false)
}

fn resolve_cached_path(dir: &Path, id: &str) -> Option<PathBuf> {
    let stem = sanitize_file_stem(id);
    let list = read_list_file(dir);
    if let Some(item) = list.items.iter().find(|i| i.id == id || sanitize_file_stem(&i.id) == stem)
    {
        if let Some(local) = item.local_path.as_ref() {
            let p = PathBuf::from(local.trim());
            if path_is_nonempty_file(&p) {
                return Some(p);
            }
        }
        let p = dir.join(&item.file_name);
        if path_is_nonempty_file(&p) {
            return Some(p);
        }
    }
    find_existing_cache(dir, &stem)
}

/// Match `@tauri-apps/api` `convertFileSrc` on Windows (asset protocol).
pub fn path_to_asset_uri(path: &Path) -> String {
    let normalized = path.to_string_lossy().replace('\\', "/");
    let mut encoded = String::with_capacity(normalized.len() + 8);
    for ch in normalized.chars() {
        match ch {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' | '/' | ':' => {
                encoded.push(ch);
            }
            _ => {
                let mut buf = [0u8; 4];
                for b in ch.encode_utf8(&mut buf).as_bytes() {
                    encoded.push_str(&format!("%{b:02X}"));
                }
            }
        }
    }
    format!("http://asset.localhost/{encoded}")
}

/// Resolve online wallpaper to a local asset URI via `.onlinefile/list.json`.
/// Returns `None` when not cached locally (caller must not fall back to remote COS).
pub fn resolve_online_asset_uri(app: &AppHandle, id: &str) -> Result<Option<String>, String> {
    let id = id.trim();
    if id.is_empty() {
        return Ok(None);
    }
    let dir = online_cache_root(app)?;
    Ok(resolve_cached_path(&dir, id).map(|p| path_to_asset_uri(&p)))
}

fn upsert_list_item(
    dir: &Path,
    id: &str,
    title: Option<&str>,
    file_path: &Path,
    mime_type: Option<&str>,
    export_path: Option<&str>,
) -> Result<OnlineFileListItem, String> {
    let file_name = file_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "无效的缓存文件名".to_string())?
        .to_string();
    let file_size = fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);
    let export = export_path
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let mut list = read_list_file(dir);
    let prev_export = list
        .items
        .iter()
        .find(|i| i.id == id)
        .and_then(|i| i.export_path.clone());
    let abs = file_path
        .canonicalize()
        .unwrap_or_else(|_| file_path.to_path_buf());
    let abs_str = abs.to_string_lossy();
    // Windows canonicalize may prefix `\\?\` which breaks asset URLs.
    let abs_clean = abs_str
        .strip_prefix(r"\\?\")
        .unwrap_or(abs_str.as_ref())
        .to_string();
    let item = OnlineFileListItem {
        id: id.to_string(),
        title: title
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        file_name,
        local_path: Some(abs_clean),
        file_size,
        mime_type: mime_type
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        downloaded_at: now_rfc3339(),
        export_path: export.or(prev_export),
    };

    if let Some(existing) = list.items.iter_mut().find(|i| i.id == id) {
        *existing = item.clone();
    } else {
        list.items.push(item.clone());
    }
    list.version = 1;
    write_list_file(dir, &list)?;
    Ok(item)
}

/// Absolute path of a locally cached online wallpaper, if present.
#[tauri::command]
pub fn get_online_file_path(app: AppHandle, id: String) -> Result<Option<String>, String> {
    let dir = online_cache_root(&app)?;
    let id = id.trim();
    if id.is_empty() {
        return Ok(None);
    }
    if let Some(p) = resolve_cached_path(&dir, id) {
        return Ok(Some(p.to_string_lossy().to_string()));
    }
    // Fall back to recorded export path if cache file is gone but export still exists.
    let list = read_list_file(&dir);
    if let Some(item) = list.items.iter().find(|i| i.id == id) {
        if let Some(export) = item.export_path.as_ref() {
            let p = PathBuf::from(export);
            if path_is_nonempty_file(&p) {
                return Ok(Some(p.to_string_lossy().to_string()));
            }
        }
    }
    Ok(None)
}

/// Return recorded `.onlinefile` downloads whose files still exist.
#[tauri::command]
pub fn list_online_file_downloads(app: AppHandle) -> Result<Vec<OnlineFileListItem>, String> {
    let dir = online_cache_root(&app)?;
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut list = read_list_file(&dir);
    let before = list.items.len();
    list.items.retain(|item| {
        if let Some(local) = item.local_path.as_ref() {
            let p = PathBuf::from(local.trim());
            if path_is_nonempty_file(&p) {
                return true;
            }
        }
        let p = dir.join(&item.file_name);
        path_is_nonempty_file(&p)
    });
    if list.items.len() != before {
        let _ = write_list_file(&dir, &list);
    }
    // Ensure each row exposes an absolute localPath for the UI / set-wallpaper.
    let items = list
        .items
        .into_iter()
        .map(|mut item| {
            if item
                .local_path
                .as_ref()
                .map(|s| s.trim().is_empty())
                .unwrap_or(true)
            {
                item.local_path = Some(dir.join(&item.file_name).to_string_lossy().to_string());
            }
            item
        })
        .collect();
    Ok(items)
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
        .http1_only()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建下载客户端失败: {e}"))?;

    let mut req = client
        .get(url)
        // Avoid any Content-Encoding negotiation on the gateway probe.
        .header(reqwest::header::ACCEPT_ENCODING, "identity");
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

fn build_download_client(max_idle: usize) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        // R2/CDN + HTTP/2 on Windows often fails mid-body with
        // "error decoding response body". Stick to HTTP/1.1.
        .http1_only()
        .connect_timeout(std::time::Duration::from_secs(60))
        .timeout(std::time::Duration::from_secs(30 * 60))
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .pool_max_idle_per_host(max_idle)
        .build()
        .map_err(|e| format!("创建下载客户端失败: {e}"))
}

fn parse_content_range_total(headers: &reqwest::header::HeaderMap) -> Option<u64> {
    let cr = headers
        .get(reqwest::header::CONTENT_RANGE)?
        .to_str()
        .ok()?;
    // e.g. "bytes 0-0/123456"
    let total = cr.rsplit('/').next()?.trim();
    if total == "*" {
        return None;
    }
    total.parse().ok()
}

fn part_ranges(total: u64, parts: u64) -> Vec<(u64, u64)> {
    let parts = parts.clamp(1, 16);
    let mut out = Vec::with_capacity(parts as usize);
    let chunk = total / parts;
    let mut start = 0u64;
    for i in 0..parts {
        let end = if i + 1 == parts {
            total.saturating_sub(1)
        } else {
            (start + chunk).saturating_sub(1).min(total.saturating_sub(1))
        };
        if start > end {
            break;
        }
        out.push((start, end));
        start = end.saturating_add(1);
        if start >= total {
            break;
        }
    }
    out
}

/// Probe whether `fetch_url` supports byte ranges. Returns `(total, content_type)` when usable.
async fn probe_multipart_size(
    client: &reqwest::Client,
    fetch_url: &str,
    authorization: Option<&str>,
) -> Result<Option<(u64, Option<String>)>, String> {
    let mut req = client
        .get(fetch_url)
        .header(reqwest::header::RANGE, "bytes=0-0")
        .header(reqwest::header::ACCEPT_ENCODING, "identity")
        .header(reqwest::header::ACCEPT, "*/*");
    if let Some(auth) = authorization.map(str::trim).filter(|s| !s.is_empty()) {
        req = req.header(reqwest::header::AUTHORIZATION, authorization_value(auth));
    }

    let response = req
        .send()
        .await
        .map_err(|e| format!("探测分片下载失败: {e}"))?;
    let status = response.status();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if status.as_u16() == 206 {
        let total = parse_content_range_total(response.headers());
        // Tiny probe body — drop it.
        let _ = response.bytes().await;
        return Ok(total.filter(|t| *t > 0).map(|t| (t, content_type)));
    }

    // Server ignored Range (200) or error — do not consume a possible full body.
    drop(response);
    Ok(None)
}

async fn download_one_range(
    client: reqwest::Client,
    fetch_url: String,
    authorization: Option<String>,
    start: u64,
    end: u64,
    part_path: PathBuf,
    downloaded: Arc<AtomicU64>,
    cancel: Option<Arc<AtomicBool>>,
) -> Result<(), String> {
    if cancel.as_ref().is_some_and(|f| is_cancelled(f)) {
        return Err("已取消下载".into());
    }

    let mut req = client
        .get(&fetch_url)
        .header(
            reqwest::header::RANGE,
            format!("bytes={start}-{end}"),
        )
        .header(reqwest::header::ACCEPT_ENCODING, "identity")
        .header(reqwest::header::ACCEPT, "*/*");
    if let Some(auth) = authorization
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        req = req.header(reqwest::header::AUTHORIZATION, authorization_value(auth));
    }

    let mut response = req
        .send()
        .await
        .map_err(|e| format!("分片下载失败 ({start}-{end}): {e}"))?;
    let status = response.status().as_u16();
    if status != 206 {
        return Err(format!("分片下载失败 ({start}-{end}): HTTP {status}"));
    }

    let expected = end.saturating_sub(start).saturating_add(1);
    let mut file = BufWriter::with_capacity(
        256 * 1024,
        fs::File::create(&part_path).map_err(|e| format!("写入分片失败: {e}"))?,
    );
    let mut got = 0u64;

    loop {
        if cancel.as_ref().is_some_and(|f| is_cancelled(f)) {
            let _ = fs::remove_file(&part_path);
            return Err("已取消下载".into());
        }
        let chunk = response.chunk().await.map_err(|e| {
            let _ = fs::remove_file(&part_path);
            format!("读取分片失败 ({start}-{end}): {e}")
        })?;
        let Some(chunk) = chunk else {
            break;
        };
        if chunk.is_empty() {
            continue;
        }
        file.write_all(&chunk).map_err(|e| {
            let _ = fs::remove_file(&part_path);
            format!("写入分片失败: {e}")
        })?;
        let n = chunk.len() as u64;
        got = got.saturating_add(n);
        downloaded.fetch_add(n, Ordering::Relaxed);
    }

    file.flush().map_err(|e| {
        let _ = fs::remove_file(&part_path);
        format!("写入分片失败: {e}")
    })?;

    if got != expected {
        let _ = fs::remove_file(&part_path);
        return Err(format!(
            "分片长度不匹配 ({start}-{end}): got {got}, expected {expected}"
        ));
    }
    Ok(())
}

async fn multipart_stream_url_to_file(
    app: &AppHandle,
    progress_id: &str,
    fetch_url: &str,
    authorization: Option<&str>,
    dest: &Path,
    cancel: Option<&Arc<AtomicBool>>,
    total: u64,
    content_type: Option<String>,
) -> Result<StreamDownloadResult, String> {
    let client = build_download_client(MULTIPART_PARTS as usize)?;
    let ranges = part_ranges(total, MULTIPART_PARTS);
    if ranges.len() < 2 {
        return Err("multipart_skip".into());
    }

    let tmp = dest
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!(
            "{}.part",
            dest.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("download.bin")
        ));
    let parent = tmp.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();
    let stem = tmp
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("download.bin");

    emit_progress(
        app,
        DownloadProgressPayload {
            id: progress_id.to_string(),
            downloaded: 0,
            total: Some(total),
            phase: "downloading",
        },
    );

    let downloaded = Arc::new(AtomicU64::new(0));
    let cancel_flag = cancel.cloned();

    let mut handles = Vec::with_capacity(ranges.len());
    let mut part_paths = Vec::with_capacity(ranges.len());

    for (i, (start, end)) in ranges.iter().copied().enumerate() {
        let part_path = parent.join(format!("{stem}.{i}"));
        let _ = fs::remove_file(&part_path);
        part_paths.push(part_path.clone());
        let client = client.clone();
        let fetch_url = fetch_url.to_string();
        let auth = authorization.map(|s| s.to_string());
        let downloaded = Arc::clone(&downloaded);
        let cancel_flag = cancel_flag.clone();
        handles.push(tokio::spawn(async move {
            download_one_range(
                client,
                fetch_url,
                auth,
                start,
                end,
                part_path,
                downloaded,
                cancel_flag,
            )
            .await
        }));
    }

    // Progress ticker while parts run.
    let ticker_app = app.clone();
    let ticker_id = progress_id.to_string();
    let ticker_downloaded = Arc::clone(&downloaded);
    let ticker_cancel = cancel_flag.clone();
    let ticker = tokio::spawn(async move {
        let mut last = Instant::now() - Duration::from_secs(1);
        loop {
            if ticker_cancel
                .as_ref()
                .is_some_and(|f| f.load(Ordering::SeqCst))
            {
                break;
            }
            let got = ticker_downloaded.load(Ordering::Relaxed);
            if last.elapsed() >= Duration::from_millis(100) {
                emit_progress(
                    &ticker_app,
                    DownloadProgressPayload {
                        id: ticker_id.clone(),
                        downloaded: got.min(total),
                        total: Some(total),
                        phase: "downloading",
                    },
                );
                last = Instant::now();
            }
            if got >= total {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    let mut first_err: Option<String> = None;
    for h in handles {
        match h.await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                if first_err.is_none() {
                    first_err = Some(e);
                }
            }
            Err(e) => {
                if first_err.is_none() {
                    first_err = Some(format!("分片任务失败: {e}"));
                }
            }
        }
    }
    ticker.abort();

    if let Some(e) = first_err {
        for p in &part_paths {
            let _ = fs::remove_file(p);
        }
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }

    if cancel.is_some_and(|c| is_cancelled(c)) {
        for p in &part_paths {
            let _ = fs::remove_file(p);
        }
        let _ = fs::remove_file(&tmp);
        return Err("已取消下载".into());
    }

    // Concatenate parts in order into the final .part file.
    let mut out = BufWriter::with_capacity(
        256 * 1024,
        fs::File::create(&tmp).map_err(|e| format!("写入临时文件失败: {e}"))?,
    );
    for p in &part_paths {
        let mut f = fs::File::open(p).map_err(|e| format!("读取分片失败: {e}"))?;
        let mut buf = vec![0u8; 256 * 1024];
        loop {
            let n = f.read(&mut buf).map_err(|e| format!("读取分片失败: {e}"))?;
            if n == 0 {
                break;
            }
            out.write_all(&buf[..n])
                .map_err(|e| format!("写入临时文件失败: {e}"))?;
        }
        let _ = fs::remove_file(p);
    }
    out.flush()
        .map_err(|e| format!("写入临时文件失败: {e}"))?;
    let file = out.into_inner().map_err(|e| format!("写入临时文件失败: {e}"))?;
    file.sync_all().ok();
    drop(file);

    let got = downloaded.load(Ordering::Relaxed);
    if got == 0 {
        let _ = fs::remove_file(&tmp);
        return Err("下载内容为空".into());
    }

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
            downloaded: total,
            total: Some(total),
            phase: "done",
        },
    );

    Ok(StreamDownloadResult {
        downloaded: total,
        total: Some(total),
        content_type,
    })
}

/// Stream `fetch_url` into `dest`. Prefers parallel Range parts; falls back to single stream.
async fn stream_url_to_file(
    app: &AppHandle,
    progress_id: &str,
    fetch_url: &str,
    authorization: Option<&str>,
    dest: &Path,
    cancel: Option<Arc<AtomicBool>>,
) -> Result<StreamDownloadResult, String> {
    let probe_client = build_download_client(2)?;
    if let Ok(Some((total, content_type))) =
        probe_multipart_size(&probe_client, fetch_url, authorization).await
    {
        if total >= MULTIPART_MIN_BYTES {
            match multipart_stream_url_to_file(
                app,
                progress_id,
                fetch_url,
                authorization,
                dest,
                cancel.as_ref(),
                total,
                content_type.clone(),
            )
            .await
            {
                Ok(r) => return Ok(r),
                Err(e) if e == "multipart_skip" || e == "已取消下载" => {
                    if e == "已取消下载" {
                        return Err(e);
                    }
                }
                Err(e) => {
                    // Range path failed mid-way — clean and fall back to single stream once.
                    tracing::warn!(error = %e, "multipart download failed; falling back to single stream");
                }
            }
        }
    }

    stream_url_to_file_single(app, progress_id, fetch_url, authorization, dest, cancel.as_deref())
        .await
}

/// Single-connection stream into `dest` via a `.part` temp file. Emits progress.
async fn stream_url_to_file_single(
    app: &AppHandle,
    progress_id: &str,
    fetch_url: &str,
    authorization: Option<&str>,
    dest: &Path,
    cancel: Option<&AtomicBool>,
) -> Result<StreamDownloadResult, String> {
    let client = build_download_client(0)?;

    let mut req = client
        .get(fetch_url)
        // Binary media must not go through gzip/br decoding middleware.
        .header(reqwest::header::ACCEPT_ENCODING, "identity")
        .header(reqwest::header::ACCEPT, "*/*");
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

    let mut file = BufWriter::with_capacity(
        256 * 1024,
        fs::File::create(&tmp).map_err(|e| format!("写入临时文件失败: {e}"))?,
    );
    let mut downloaded: u64 = 0;
    let mut last_emit = Instant::now() - Duration::from_secs(1);

    loop {
        if cancel.is_some_and(is_cancelled) {
            let _ = fs::remove_file(&tmp);
            return Err("已取消下载".into());
        }
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

    if cancel.is_some_and(is_cancelled) {
        let _ = fs::remove_file(&tmp);
        return Err("已取消下载".into());
    }

    if downloaded == 0 {
        let _ = fs::remove_file(&tmp);
        return Err("下载内容为空".into());
    }

    file.flush().map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("写入临时文件失败: {e}")
    })?;
    let file = file.into_inner().map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("写入临时文件失败: {e}")
    })?;
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

    if let Some(existing) = resolve_cached_path(&dir, &progress_id) {
        let _ = upsert_list_item(&dir, &progress_id, None, &existing, None, None);
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

    let mut fetch_url = url.clone();
    let mut fetch_auth = if payload.record_download {
        None
    } else {
        auth.clone()
    };
    if payload.record_download {
        let location = probe_download_gateway(&url, auth.as_deref()).await?;
        fetch_url = resolve_fetch_url(&url, location);
        fetch_auth = None;
    }

    let preferred = payload.file_ext.clone();
    let guess_ext = resolve_ext(preferred.as_deref(), &fetch_url, &url, None);
    let stem = sanitize_file_stem(&progress_id);
    let dest = dir.join(format!("{stem}.{guess_ext}"));

    let cancel = register_cancel_flag(&progress_id);
    let result = stream_url_to_file(
        &app,
        &progress_id,
        &fetch_url,
        fetch_auth.as_deref(),
        &dest,
        Some(Arc::clone(&cancel)),
    )
    .await;
    unregister_cancel_flag(&progress_id);
    let result = result?;

    let better = resolve_ext(
        preferred.as_deref(),
        &fetch_url,
        &url,
        result.content_type.as_deref(),
    );
    let final_path = if better != guess_ext {
        let renamed = dir.join(format!("{stem}.{better}"));
        if renamed != dest {
            let _ = fs::rename(&dest, &renamed);
            renamed
        } else {
            dest
        }
    } else {
        dest
    };

    upsert_list_item(
        &dir,
        &progress_id,
        None,
        &final_path,
        result.content_type.as_deref(),
        None,
    )?;

    Ok(final_path.to_string_lossy().to_string())
}

/// User download: stream into `{library_root}/.onlinefile/{id}.{ext}` (+ list.json).
/// Skips network when already cached. No system save dialog / second copy.
#[tauri::command]
pub async fn save_online_wallpaper(
    app: AppHandle,
    payload: SaveOnlinePayload,
) -> Result<String, String> {
    let url = payload.url.trim().to_string();
    if url.is_empty() {
        return Err("缺少下载地址".into());
    }
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("下载地址无效".into());
    }

    let progress_id = payload.id.trim().to_string();
    let dir = online_cache_root(&app)?;
    fs::create_dir_all(&dir).map_err(|e| format!("创建 .onlinefile 目录失败: {e}"))?;

    let title = payload
        .title
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());

    emit_progress(
        &app,
        DownloadProgressPayload {
            id: progress_id.clone(),
            downloaded: 0,
            total: None,
            phase: "resolving",
        },
    );

    if let Some(existing) = resolve_cached_path(&dir, &progress_id) {
        upsert_list_item(&dir, &progress_id, title, &existing, None, None)?;
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

    let auth = payload
        .authorization
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let preferred_fetch = payload
        .fetch_url
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| s.starts_with("http://") || s.starts_with("https://"));

    let mut fetch_url = preferred_fetch.clone().unwrap_or_else(|| url.clone());
    // Presigned / CDN object URLs must not carry Authorization.
    let mut fetch_auth = if preferred_fetch.is_some() {
        None
    } else {
        auth.clone()
    };
    if payload.record_download {
        if preferred_fetch.is_none() {
            let location = probe_download_gateway(&url, auth.as_deref()).await?;
            fetch_url = resolve_fetch_url(&url, location);
            fetch_auth = None;
        } else {
            // History only — bytes come from the explicit fetch_url.
            let _ = probe_download_gateway(&url, auth.as_deref()).await?;
        }
    }

    let preferred = payload.file_ext.clone();
    let guess_ext = resolve_ext(preferred.as_deref(), &fetch_url, &url, None);
    let stem = sanitize_file_stem(&progress_id);
    let dest = dir.join(format!("{stem}.{guess_ext}"));

    let cancel = register_cancel_flag(&progress_id);
    let result = stream_url_to_file(
        &app,
        &progress_id,
        &fetch_url,
        fetch_auth.as_deref(),
        &dest,
        Some(Arc::clone(&cancel)),
    )
    .await;
    unregister_cancel_flag(&progress_id);
    let result = result?;

    let better = resolve_ext(
        preferred.as_deref(),
        &fetch_url,
        &url,
        result.content_type.as_deref(),
    );
    let final_path = if better != guess_ext {
        let renamed = dir.join(format!("{stem}.{better}"));
        if renamed != dest {
            let _ = fs::rename(&dest, &renamed);
            renamed
        } else {
            dest
        }
    } else {
        dest
    };

    upsert_list_item(
        &dir,
        &progress_id,
        title,
        &final_path,
        result.content_type.as_deref(),
        None,
    )?;

    emit_progress(
        &app,
        DownloadProgressPayload {
            id: progress_id,
            downloaded: 1,
            total: Some(1),
            phase: "done",
        },
    );

    Ok(final_path.to_string_lossy().to_string())
}

/// True if path is inside the `.onlinefile` cache (must not be indexed as local library).
#[allow(dead_code)]
pub fn is_online_cache_path(app: &AppHandle, path: &Path) -> bool {
    let Ok(root) = online_cache_root(app) else {
        return false;
    };
    path.starts_with(&root)
}
