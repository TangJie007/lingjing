use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FavoritesFile {
    /// Stable ids: catalog numeric string ("1") or local ("l-uuid")
    pub ids: Vec<String>,
}

fn favorites_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法解析应用数据目录: {e}"))?;
    fs::create_dir_all(&dir).map_err(|e| format!("创建数据目录失败: {e}"))?;
    Ok(dir.join("favorites.json"))
}

pub fn load(app: &AppHandle) -> Result<(FavoritesFile, bool), String> {
    let path = favorites_path(app)?;
    if !path.exists() {
        return Ok((FavoritesFile::default(), true));
    }
    let raw = fs::read_to_string(&path).map_err(|e| format!("读取 favorites.json 失败: {e}"))?;
    let file = serde_json::from_str(&raw).map_err(|e| format!("解析 favorites.json 失败: {e}"))?;
    Ok((file, false))
}

pub fn save(app: &AppHandle, file: &FavoritesFile) -> Result<(), String> {
    let path = favorites_path(app)?;
    crate::util::write_json_atomic(&path, file)
}

pub fn set_favorite(app: &AppHandle, id: String, favorite: bool) -> Result<FavoritesFile, String> {
    let (mut file, _) = load(app)?;
    let mut set: HashSet<String> = file.ids.into_iter().collect();
    if favorite {
        set.insert(id.clone());
    } else {
        set.remove(&id);
    }
    file.ids = set.into_iter().collect();
    file.ids.sort();
    save(app, &file)?;
    let _ = crate::library::set_favorite_flag(app, &id, favorite);
    Ok(file)
}
