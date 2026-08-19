use crate::settings;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationPlan {
    pub from_dir: String,
    pub to_dir: String,
    pub files: Vec<MigrationFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationFile {
    pub rel_path: String,
    pub from_path: String,
    pub to_path: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MigrationReport {
    pub copied: u32,
    pub skipped: u32,
    pub failed: u32,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ProgressPayload {
    pub done: u32,
    pub total: u32,
    pub rel_path: String,
}

fn list_relative(from: &Path, root: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if !from.exists() {
        return Ok(out);
    }
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Ok(rel) = path.strip_prefix(root) {
                out.push(rel.to_path_buf());
            }
        }
    }
    Ok(out)
}

fn plan_migration(app: &AppHandle, to_dir: &str) -> Result<MigrationPlan, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法解析应用数据目录: {e}"))?;
    let from_dir = data_dir.clone();
    let to_path = PathBuf::from(to_dir);
    if !to_path.exists() {
        fs::create_dir_all(&to_path).map_err(|e| format!("无法创建目标目录: {e}"))?;
    }
    let mut files = Vec::new();
    let candidates = [
        from_dir.join("library.json"),
        from_dir.join("favorites.json"),
        from_dir.join("settings.json"),
        from_dir.join("last_wallpaper.json"),
    ];
    for c in candidates.iter() {
        if !c.is_file() {
            continue;
        }
        let rel = c
            .strip_prefix(&from_dir)
            .map_err(|e| format!("strip_prefix 失败: {e}"))?;
        let to = to_path.join(rel);
        let size = fs::metadata(c).map(|m| m.len()).unwrap_or(0);
        files.push(MigrationFile {
            rel_path: rel.to_string_lossy().to_string(),
            from_path: c.to_string_lossy().to_string(),
            to_path: to.to_string_lossy().to_string(),
            size,
        });
    }
    let library_from = from_dir.join("library");
    if library_from.exists() {
        let rels = list_relative(&library_from, &library_from).unwrap_or_default();
        for rel in rels {
            let from = library_from.join(&rel);
            let to = to_path.join("library").join(&rel);
            let size = fs::metadata(&from).map(|m| m.len()).unwrap_or(0);
            files.push(MigrationFile {
                rel_path: format!("library/{}", rel.to_string_lossy()),
                from_path: from.to_string_lossy().to_string(),
                to_path: to.to_string_lossy().to_string(),
                size,
            });
        }
    }
    Ok(MigrationPlan {
        from_dir: from_dir.to_string_lossy().to_string(),
        to_dir: to_path.to_string_lossy().to_string(),
        files,
    })
}

pub fn set_library_dir(app: &AppHandle, new_dir: String) -> Result<MigrationPlan, String> {
    if new_dir.trim().is_empty() {
        return Err("路径不能为空".into());
    }
    let mut settings = settings::load_settings(app)?;
    let trimmed = new_dir.trim().to_string();
    if !PathBuf::from(&trimmed).exists() {
        fs::create_dir_all(&trimmed).map_err(|e| format!("无法创建目录: {e}"))?;
    }
    let old_override = settings.library_dir_override.clone();
    settings.library_dir_override = Some(trimmed.clone());
    settings::save_settings(app, &settings)?;
    let plan = match plan_migration(app, &trimmed) {
        Ok(plan) => plan,
        Err(e) => {
            settings.library_dir_override = old_override;
            let _ = settings::save_settings(app, &settings);
            return Err(e);
        }
    };
    Ok(plan)
}

pub fn migrate_library(
    app: &AppHandle,
    keep_originals: bool,
) -> Result<MigrationReport, String> {
    let mut s = settings::load_settings(app)?;
    let to_dir = s
        .library_dir_override
        .clone()
        .ok_or_else(|| "尚未选择新路径".to_string())?;
    let plan = plan_migration(app, &to_dir)?;
    let mut report = MigrationReport::default();
    let total = plan.files.len() as u32;
    for (idx, file) in plan.files.iter().enumerate() {
        let done = idx as u32;
        let _ = app.emit(
            "library-migration-progress",
            &ProgressPayload {
                done,
                total,
                rel_path: file.rel_path.clone(),
            },
        );
        let from = PathBuf::from(&file.from_path);
        let to = PathBuf::from(&file.to_path);
        if let Some(parent) = to.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if to.exists() && !keep_originals {
            if fs::remove_file(&to).is_err() {
                report.failed += 1;
                report.errors.push(format!("无法覆盖 {}", to.display()));
                continue;
            }
        }
        if to.exists() {
            report.skipped += 1;
            continue;
        }
        if fs::copy(&from, &to).is_err() {
            report.failed += 1;
            report
                .errors
                .push(format!("复制失败 {} -> {}", from.display(), to.display()));
            continue;
        }
        if !keep_originals {
            let _ = fs::remove_file(&from);
        }
        report.copied += 1;
    }
    let _ = app.emit(
        "library-migration-progress",
        &ProgressPayload {
            done: total,
            total,
            rel_path: "".into(),
        },
    );
    if report.copied > 0 {
        s.library_dir_override = Some(to_dir);
        s.import_copy_to_data = true;
        let _ = settings::save_settings(app, &s);
    }
    Ok(report)
}
