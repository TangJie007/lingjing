//! Desktop namespace icons: 此电脑 / 回收站 / 网络 (`::{CLSID}` paths).
//!
//! These are virtual Shell items (not files). Context menus use the custom
//! `namespace_builtin_menu` — QueryContextMenu on raw CLSIDs tends to hang.

use std::path::Path;

pub const CLSID_COMPUTER: &str = "::{20D04FE0-3AEA-1069-A2D8-08002B30309D}";
pub const CLSID_RECYCLE: &str = "::{645FF040-5081-101B-9F08-00AA002F954E}";
/// Desktop “网络” namespace (Network).
pub const CLSID_NETWORK: &str = "::{F02C1A0D-BE21-4350-88B0-7367FC96EF3C}";
const CLSID_NETWORK_LEGACY: &str = "F02C1A0D-B21F-4110-8426-0A0C959C3602";

pub fn builtin_kind_from_path(path: &str) -> Option<&'static str> {
    let upper = path.to_ascii_uppercase();
    if upper.contains("20D04FE0-3AEA-1069-A2D8-08002B30309D") {
        return Some("computer");
    }
    if upper.contains("645FF040-5081-101B-9F08-00AA002F954E") {
        return Some("recycle");
    }
    if upper.contains("F02C1A0D-BE21-4350-88B0-7367FC96EF3C")
        || upper.contains(CLSID_NETWORK_LEGACY)
    {
        return Some("network");
    }
    let name = Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    // Legacy managed `.lnk` filenames (pre-CLSID-path rollback).
    match name.as_str() {
        "此电脑.lnk" | "computer.lnk" => Some("computer"),
        "回收站.lnk" | "recycle.lnk" => Some("recycle"),
        "网络.lnk" | "network.lnk" => Some("network"),
        _ => None,
    }
}

/// Parsing name (`::{CLSID}`) for a namespace icon or legacy managed link path.
pub fn namespace_clsid_for_path(path: &str) -> Option<&'static str> {
    match builtin_kind_from_path(path)? {
        "computer" => Some(CLSID_COMPUTER),
        "recycle" => Some(CLSID_RECYCLE),
        "network" => Some(CLSID_NETWORK),
        _ => None,
    }
}

/// True for raw `::{CLSID}` namespace paths (and legacy AppData builtin `.lnk`s).
pub fn is_shell_namespace_item(path: &str) -> bool {
    if path.trim_start().starts_with("::") {
        return true;
    }
    builtin_kind_from_path(path).is_some()
        && path
            .replace('/', "\\")
            .to_ascii_lowercase()
            .contains("builtin-links")
}
