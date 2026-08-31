//! Managed shortcuts for desktop namespace icons (此电脑 / 回收站 / 网络).
//!
//! Fence items use real `.lnk` files under app data. Shortcuts must be
//! IDList/PIDL links (same as Explorer “创建快捷方式”), not `explorer.exe` +
//! `shell:::{CLSID}` — only the former get working Shell context-menu verbs.

use std::fs;
use std::path::{Path, PathBuf};

pub const CLSID_COMPUTER: &str = "::{20D04FE0-3AEA-1069-A2D8-08002B30309D}";
pub const CLSID_RECYCLE: &str = "::{645FF040-5081-101B-9F08-00AA002F954E}";
/// Desktop “网络” namespace (Network). Legacy typo GUID kept in path detection only.
pub const CLSID_NETWORK: &str = "::{F02C1A0D-BE21-4350-88B0-7367FC96EF3C}";
const CLSID_NETWORK_LEGACY: &str = "F02C1A0D-B21F-4110-8426-0A0C959C3602";

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
    match name.as_str() {
        "此电脑.lnk" | "computer.lnk" => Some("computer"),
        "回收站.lnk" | "recycle.lnk" => Some("recycle"),
        "网络.lnk" | "network.lnk" => Some("network"),
        _ => None,
    }
}

/// Parsing name (`::{CLSID}`) for a managed link or raw namespace path.
pub fn namespace_clsid_for_path(path: &str) -> Option<&'static str> {
    match builtin_kind_from_path(path)? {
        "computer" => Some(CLSID_COMPUTER),
        "recycle" => Some(CLSID_RECYCLE),
        "network" => Some(CLSID_NETWORK),
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
        if !lnk.is_file() || !shortcut_matches_namespace(&lnk, spec.clsid) {
            let _ = fs::remove_file(&lnk);
            create_namespace_shortcut(&lnk, spec.clsid, spec.fallback_name)?;
        }
        out.push((lnk, *spec));
    }
    Ok(out)
}

fn shortcut_matches_namespace(lnk: &Path, clsid: &str) -> bool {
    link_target_parsing_name(lnk)
        .map(|name| {
            let a = name.to_ascii_uppercase().replace('/', "\\");
            let b = clsid.to_ascii_uppercase();
            a.contains(b.trim_start_matches(':')) || a == b
        })
        .unwrap_or(false)
}

fn create_namespace_shortcut(lnk: &Path, clsid: &str, name: &str) -> Result<(), String> {
    create_idlist_shortcut(lnk, clsid, name)?;
    if !lnk.is_file() || !shortcut_matches_namespace(lnk, clsid) {
        return Err(format!("创建{name}快捷方式失败"));
    }
    Ok(())
}

/// Public wrapper for creating an IDList namespace `.lnk` at an arbitrary path.
pub(crate) fn create_namespace_shortcut_for(
    lnk: &Path,
    clsid: &str,
    name: &str,
) -> Result<(), String> {
    create_namespace_shortcut(lnk, clsid, name)
}

/// Create an Explorer-style namespace `.lnk` via `IShellLink::SetIDList`.
fn create_idlist_shortcut(lnk: &Path, clsid: &str, name: &str) -> Result<(), String> {
    use windows::core::{Interface, PCWSTR};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoTaskMemFree, IPersistFile, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED,
    };
    use windows::Win32::UI::Shell::Common::ITEMIDLIST;
    use windows::Win32::UI::Shell::{IShellLinkW, SHParseDisplayName, ShellLink};

    let lnk_w = wide(lnk.to_string_lossy().as_ref());
    let clsid_w = wide(clsid);
    let name_w = wide(name);

    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        let mut pidl: *mut ITEMIDLIST = std::ptr::null_mut();
        SHParseDisplayName(PCWSTR(clsid_w.as_ptr()), None, &mut pidl, 0, None)
            .map_err(|e| format!("解析{name}命名空间失败: {e}"))?;
        if pidl.is_null() {
            return Err(format!("解析{name}命名空间失败"));
        }

        let result = (|| -> Result<(), String> {
            let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)
                .map_err(|e| format!("创建快捷方式对象失败: {e}"))?;
            link.SetIDList(pidl)
                .map_err(|e| format!("设置快捷方式目标失败: {e}"))?;
            let _ = link.SetDescription(PCWSTR(name_w.as_ptr()));
            let persist: IPersistFile = link
                .cast()
                .map_err(|e| format!("快捷方式持久化失败: {e}"))?;
            persist
                .Save(PCWSTR(lnk_w.as_ptr()), true)
                .map_err(|e| format!("保存{name}快捷方式失败: {e}"))?;
            Ok(())
        })();

        CoTaskMemFree(Some(pidl as *const _));
        result
    }
}

fn link_target_parsing_name(lnk: &Path) -> Option<String> {
    use windows::core::{Interface, PCWSTR};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoTaskMemFree, IPersistFile, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED, STGM,
    };
    use windows::Win32::UI::Shell::{
        IShellLinkW, SHGetNameFromIDList, ShellLink, SIGDN_DESKTOPABSOLUTEPARSING,
    };

    let lnk_w = wide(lnk.to_string_lossy().as_ref());
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;
        let persist: IPersistFile = link.cast().ok()?;
        persist.Load(PCWSTR(lnk_w.as_ptr()), STGM(0)).ok()?;
        let pidl = link.GetIDList().ok()?;
        if pidl.is_null() {
            return None;
        }
        let name = match SHGetNameFromIDList(pidl, SIGDN_DESKTOPABSOLUTEPARSING) {
            Ok(n) => n,
            Err(_) => {
                CoTaskMemFree(Some(pidl as *const _));
                return None;
            }
        };
        CoTaskMemFree(Some(pidl as *const _));
        if name.is_null() {
            return None;
        }
        let s = name.to_string().ok();
        CoTaskMemFree(Some(name.0 as *const _));
        s
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
