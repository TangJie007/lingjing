//! Cache shell icons / image previews by path + mtime to speed up desktop rescans.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::time::SystemTime;

struct Entry {
    mtime: SystemTime,
    display_name: String,
    icon: Option<String>,
}

static CACHE: Mutex<Option<HashMap<String, Entry>>> = Mutex::new(None);

fn mtime_of(path: &Path) -> Option<SystemTime> {
    path.metadata().and_then(|m| m.modified()).ok()
}

fn cache() -> std::sync::MutexGuard<'static, Option<HashMap<String, Entry>>> {
    CACHE.lock().unwrap_or_else(|e| e.into_inner())
}

/// Return cached (display_name, icon) when mtime matches.
pub fn get(path: &Path) -> Option<(String, Option<String>)> {
    let mt = mtime_of(path)?;
    let key = path.to_string_lossy().to_string();
    let guard = cache();
    let map = guard.as_ref()?;
    let e = map.get(&key)?;
    if e.mtime != mt {
        return None;
    }
    Some((e.display_name.clone(), e.icon.clone()))
}

pub fn put(path: &Path, display_name: String, icon: Option<String>) {
    let Some(mt) = mtime_of(path) else {
        return;
    };
    let key = path.to_string_lossy().to_string();
    let mut guard = cache();
    let map = guard.get_or_insert_with(HashMap::new);
    // Bound memory: drop oldest-ish by clearing when huge.
    if map.len() > 800 {
        map.clear();
    }
    map.insert(
        key,
        Entry {
            mtime: mt,
            display_name,
            icon,
        },
    );
}

pub fn invalidate(path: &str) {
    let mut guard = cache();
    if let Some(map) = guard.as_mut() {
        map.remove(path);
    }
}

pub fn clear() {
    let mut guard = cache();
    *guard = None;
}
