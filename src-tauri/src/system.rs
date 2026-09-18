use std::path::{Path, PathBuf};

#[cfg(windows)]
pub fn detect_low_power_mode() -> bool {
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    unsafe {
        let mut status: MEMORYSTATUSEX = std::mem::zeroed();
        status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        if GlobalMemoryStatusEx(&mut status).is_err() {
            return false;
        }
        // < 6 GB physical RAM → low power tier
        status.ullTotalPhys < 6 * 1024 * 1024 * 1024
    }
}

#[cfg(not(windows))]
pub fn detect_low_power_mode() -> bool {
    false
}

fn strip_extended_path(path: &str) -> &str {
    path.strip_prefix(r"\\?\").unwrap_or(path)
}

/// Absolute local filesystem path, if `uri` points at a real file.
pub fn filesystem_path_from_uri(uri: &str) -> Option<PathBuf> {
    let trimmed = uri.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(rest) = trimmed.strip_prefix("http://asset.localhost/") {
        let decoded = urlencoding_lite_decode(rest);
        let path = PathBuf::from(strip_extended_path(&decoded));
        return path.is_file().then_some(path);
    }
    if let Some(rest) = trimmed.strip_prefix("https://asset.localhost/") {
        let decoded = urlencoding_lite_decode(rest);
        let path = PathBuf::from(strip_extended_path(&decoded));
        return path.is_file().then_some(path);
    }
    if let Some(rest) = trimmed.strip_prefix("asset://localhost/") {
        let decoded = urlencoding_lite_decode(rest);
        let path = PathBuf::from(strip_extended_path(&decoded));
        return path.is_file().then_some(path);
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return None;
    }

    let path_str = trimmed
        .strip_prefix("file:///")
        .or_else(|| trimmed.strip_prefix("file://"))
        .unwrap_or(trimmed);
    let path_str = strip_extended_path(path_str);
    let path = if path_str.starts_with('/')
        && path_str.len() > 2
        && path_str.as_bytes().get(2) == Some(&b':')
    {
        // /C:/Users/... → C:/Users/...
        PathBuf::from(&path_str[1..])
    } else {
        PathBuf::from(path_str.replace('/', std::path::MAIN_SEPARATOR_STR))
    };
    path.is_file().then_some(path)
}

/// Decode %XX in asset URLs without pulling a full URL crate.
fn urlencoding_lite_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let h = |c: u8| -> Option<u8> {
                match c {
                    b'0'..=b'9' => Some(c - b'0'),
                    b'a'..=b'f' => Some(c - b'a' + 10),
                    b'A'..=b'F' => Some(c - b'A' + 10),
                    _ => None,
                }
            };
            if let (Some(a), Some(b)) = (h(bytes[i + 1]), h(bytes[i + 2])) {
                out.push(a << 4 | b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub fn media_path_exists(uri: &str) -> bool {
    filesystem_path_from_uri(uri).is_some()
}

/// Bundled sample / same-origin app asset (not a remote COS URL).
pub fn is_bundled_app_asset_uri(uri: &str) -> bool {
    let t = uri.trim().to_ascii_lowercase();
    if t.is_empty() {
        return false;
    }
    // Relative public samples served by the app webview origin.
    if t.starts_with("/samples/") {
        return true;
    }
    let is_http = t.starts_with("http://") || t.starts_with("https://");
    if !is_http {
        return false;
    }
    // Dev Vite / Tauri localhost app origins only — never remote CDN.
    let host_ok = t.contains("://localhost")
        || t.contains("://127.0.0.1")
        || t.contains("://tauri.localhost")
        || t.contains("://asset.localhost");
    host_ok && t.contains("/samples/")
}

pub fn path_is_nonempty_file(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|m| m.is_file() && m.len() > 0)
        .unwrap_or(false)
}
