//! Managed shortcuts for desktop namespace icons (此电脑 / 回收站 / 网络).
//!
//! Fence items point at real `.lnk` files under app data instead of `::{CLSID}`
//! parsing names, so Shell menus / open behave like normal shortcuts.

use std::fs;
use std::path::{Path, PathBuf};

pub const CLSID_COMPUTER: &str = "::{20D04FE0-3AEA-1069-A2D8-08002B30309D}";
pub const CLSID_RECYCLE: &str = "::{645FF040-5081-101B-9F08-00AA002F954E}";
pub const CLSID_NETWORK: &str = "::{F02C1A0D-B21F-4110-8426-0A0C959C3602}";

const LINK_COMPUTER: &str = "此电脑.lnk";
const LINK_RECYCLE: &str = "回收站.lnk";
const LINK_NETWORK: &str = "网络.lnk";

#[derive(Clone, Copy)]
pub struct BuiltinNamespaceSpec {
    pub link_name: &'static str,
    pub clsid: &'static str,
    pub fallback_name: &'static str,
}

pub const BUILTIN_NAMESPACE_SPECS: &[BuiltinNamespaceSpec] = &[
    BuiltinNamespaceSpec {
        link_name: LINK_COMPUTER,
        clsid: CLSID_COMPUTER,
        fallback_name: "此电脑",
    },
    BuiltinNamespaceSpec {
        link_name: LINK_RECYCLE,
        clsid: CLSID_RECYCLE,
        fallback_name: "回收站",
    },
    BuiltinNamespaceSpec {
        link_name: LINK_NETWORK,
        clsid: CLSID_NETWORK,
        fallback_name: "网络",
    },
];

pub fn builtin_links_dir() -> Result<PathBuf, String> {
    let base = known_folders::get_known_folder_path(known_folders::KnownFolder::RoamingAppData)
        .ok_or_else(|| "无法定位 AppData\\Roaming".to_string())?;
    Ok(base.join("com.lingscape.app").join("builtin-links"))
}

pub fn is_managed_builtin_link(path: &str) -> bool {
    let Ok(dir) = builtin_links_dir() else {
        return false;
    };
    let p = Path::new(path);
    match (p.canonicalize(), dir.canonicalize()) {
        (Ok(a), Ok(b)) => a.starts_with(&b),
        _ => {
            let a = path.replace('/', "\\").to_ascii_lowercase();
            let b = dir.to_string_lossy().replace('/', "\\").to_ascii_lowercase();
            a.starts_with(&b)
        }
    }
}

pub fn builtin_kind_from_path(path: &str) -> Option<&'static str> {
    let upper = path.to_ascii_uppercase();
    if upper.contains("20D04FE0-3AEA-1069-A2D8-08002B30309D") {
        return Some("computer");
    }
    if upper.contains("645FF040-5081-101B-9F08-00AA002F954E") {
        return Some("recycle");
    }
    if upper.contains("F02C1A0D-B21F-4110-8426-0A0C959C3602") {
        return Some("network");
    }
    let name = Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match name.as_str() {
        "此电脑.lnk" | "computer.lnk" => Some("computer"),
        "回收站.lnk" | "recycle.lnk" => Some("recycle"),
        "网络.lnk" | "network.lnk" => Some("network"),
        _ => None,
    }
}

/// Ensure the three namespace shortcuts exist; return their absolute paths in order.
pub fn ensure_builtin_namespace_links() -> Result<Vec<(PathBuf, BuiltinNamespaceSpec)>, String> {
    let dir = builtin_links_dir()?;
    fs::create_dir_all(&dir).map_err(|e| format!("创建内置快捷方式目录失败: {e}"))?;

    let mut out = Vec::with_capacity(BUILTIN_NAMESPACE_SPECS.len());
    for spec in BUILTIN_NAMESPACE_SPECS {
        let lnk = dir.join(spec.link_name);
        if !lnk.is_file() {
            create_namespace_shortcut(&lnk, spec.clsid, spec.fallback_name)?;
        }
        out.push((lnk, *spec));
    }
    Ok(out)
}

fn create_namespace_shortcut(lnk: &Path, clsid: &str, name: &str) -> Result<(), String> {
    let lnk_s = lnk.to_string_lossy().replace('\'', "''");
    // Prefer shell:::{CLSID} target; fall back to explorer.exe + arguments.
    let target = if clsid.starts_with("::") {
        format!("shell:{clsid}")
    } else {
        clsid.to_string()
    };
    let target_esc = target.replace('\'', "''");
    let script = format!(
        "$w=New-Object -ComObject WScript.Shell; $s=$w.CreateShortcut([string]'{lnk}'); $s.TargetPath=[string]'{target}'; $s.Description=[string]'{name}'; $s.Save()",
        lnk = lnk_s,
        target = target_esc,
        name = name.replace('\'', "''"),
    );
    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .status()
        .map_err(|e| format!("创建{name}快捷方式失败: {e}"))?;
    if status.success() && lnk.is_file() {
        return Ok(());
    }

    // Fallback: explorer.exe shell:::{GUID}
    let guid = clsid.trim_start_matches(':');
    let args = format!("shell::{guid}");
    let script = format!(
        "$w=New-Object -ComObject WScript.Shell; $s=$w.CreateShortcut([string]'{lnk}'); $s.TargetPath='explorer.exe'; $s.Arguments=[string]'{args}'; $s.Description=[string]'{name}'; $s.Save()",
        lnk = lnk_s,
        args = args.replace('\'', "''"),
        name = name.replace('\'', "''"),
    );
    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .status()
        .map_err(|e| format!("创建{name}快捷方式失败: {e}"))?;
    if !status.success() || !lnk.is_file() {
        return Err(format!("创建{name}快捷方式失败"));
    }
    Ok(())
}
