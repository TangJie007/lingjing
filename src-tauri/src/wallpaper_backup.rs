//! Original wallpaper backup & restore (SET-006)
//! Backs up the user's original Windows wallpaper on startup,
//! restores it on exit. Also detects crash recovery on next launch.

use std::fs;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    SystemParametersInfoW, SPI_GETDESKWALLPAPER, SPI_SETDESKWALLPAPER,
    SPIF_SENDCHANGE, SPIF_UPDATEINIFILE,
};

fn backup_dir(app_data: &PathBuf) -> PathBuf {
    app_data.join("backup")
}

fn backup_meta_path(app_data: &PathBuf) -> PathBuf {
    backup_dir(app_data).join("backup_meta.json")
}

fn snapshot_flag_path(app_data: &PathBuf) -> PathBuf {
    backup_dir(app_data).join(".snapshot_exists")
}

/// Backup the current Windows wallpaper on startup.
/// Saves the wallpaper path and copies the file to the backup directory.
#[tauri::command]
pub fn backup_original_wallpaper(app_data: String) -> Result<bool, String> {
    let app_data = PathBuf::from(&app_data);
    let dir = backup_dir(&app_data);
    fs::create_dir_all(&dir).map_err(|e| format!("创建备份目录失败: {}", e))?;

    #[cfg(target_os = "windows")]
    {
        let current_path = get_current_wallpaper_path()?;
        if current_path.is_empty() {
            log::info!("当前无壁纸路径，跳过备份");
            return Ok(false);
        }

        let src = PathBuf::from(&current_path);
        let ext = src.extension().unwrap_or_default().to_string_lossy();
        let dest = dir.join(format!("original_wallpaper.{}", ext));

        if src.exists() {
            fs::copy(&src, &dest).map_err(|e| format!("备份壁纸文件失败: {}", e))?;
        }

        // Save metadata
        let meta = serde_json::json!({
            "original_path": current_path,
            "backup_path": dest.to_string_lossy().to_string(),
        });
        fs::write(
            backup_meta_path(&app_data),
            serde_json::to_string_pretty(&meta).unwrap_or_default(),
        )
        .map_err(|e| format!("保存备份元数据失败: {}", e))?;

        // Mark snapshot exists
        fs::write(snapshot_flag_path(&app_data), "1")
            .map_err(|e| format!("标记快照失败: {}", e))?;

        log::info!("原始壁纸已备份: {}", current_path);
        Ok(true)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = app_data;
        Ok(false)
    }
}

/// Restore the original wallpaper on exit.
#[tauri::command]
pub fn restore_original_wallpaper(app_data: String) -> Result<bool, String> {
    let app_data = PathBuf::from(&app_data);

    if !snapshot_flag_path(&app_data).exists() {
        return Ok(false);
    }

    #[cfg(target_os = "windows")]
    {
        let meta_path = backup_meta_path(&app_data);
        if !meta_path.exists() {
            return Ok(false);
        }

        let meta: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&meta_path).unwrap_or_default(),
        )
        .unwrap_or_default();

        let backup_path = meta["backup_path"].as_str().unwrap_or("");
        if !backup_path.is_empty() {
            set_windows_wallpaper(backup_path)?;
        }

        // Clean up
        let _ = fs::remove_file(snapshot_flag_path(&app_data));
        let _ = fs::remove_file(meta_path);
        let _ = fs::remove_dir_all(backup_dir(&app_data));

        log::info!("原始壁纸已恢复");
        Ok(true)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = app_data;
        Ok(false)
    }
}

/// Check if a crash recovery is needed (snapshot exists from previous run).
#[tauri::command]
pub fn check_crash_recovery(app_data: String) -> Result<serde_json::Value, String> {
    let app_data = PathBuf::from(&app_data);

    let needs_recovery = snapshot_flag_path(&app_data).exists();
    let backup_path = if needs_recovery {
        let meta_path = backup_meta_path(&app_data);
        if meta_path.exists() {
            let meta: serde_json::Value = serde_json::from_str(
                &fs::read_to_string(&meta_path).unwrap_or_default(),
            )
            .unwrap_or_default();
            meta["original_path"].as_str().unwrap_or("").to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    Ok(serde_json::json!({
        "needsRecovery": needs_recovery,
        "backupPath": backup_path,
    }))
}

#[cfg(target_os = "windows")]
fn get_current_wallpaper_path() -> Result<String, String> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    unsafe {
        let mut buf = vec![0u16; 520]; // MAX_PATH
        SystemParametersInfoW(
            SPI_GETDESKWALLPAPER,
            buf.len() as u32,
            Some(buf.as_mut_ptr() as *mut _),
            SPIF_UPDATEINIFILE,
        )
        .map_err(|e| format!("读取当前壁纸路径失败: {:?}", e))?;

        let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        let path = OsString::from_wide(&buf[..len])
            .to_string_lossy()
            .to_string();
        Ok(path)
    }
}

#[cfg(target_os = "windows")]
fn set_windows_wallpaper(path: &str) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let wide: Vec<u16> = OsStr::new(path).encode_wide().chain(std::iter::once(0)).collect();

    unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            Some(wide.as_ptr() as *mut _),
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        )
        .map_err(|e| format!("恢复壁纸失败: {:?}", e))?;
    }
    Ok(())
}
