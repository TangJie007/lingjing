//! Small shared helpers (atomic JSON writes, logging bootstrap).

use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::Path;

pub fn init_logging() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("lingscape=info,lingscape_lib=info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}

/// Serialize `value` as pretty JSON and replace `path` via a same-directory temp file.
pub fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
    let raw = serde_json::to_string_pretty(value).map_err(|e| format!("序列化失败: {e}"))?;
    let mut tmp =
        tempfile::NamedTempFile::new_in(parent).map_err(|e| format!("创建临时文件失败: {e}"))?;
    tmp.write_all(raw.as_bytes())
        .map_err(|e| format!("写入临时文件失败: {e}"))?;
    tmp.flush()
        .map_err(|e| format!("刷新临时文件失败: {e}"))?;
    // Windows rename cannot replace an existing file.
    if path.exists() {
        let _ = fs::remove_file(path);
    }
    tmp.persist(path)
        .map_err(|e| format!("写入失败: {}", e.error))?;
    Ok(())
}
