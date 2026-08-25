use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

use crate::desktop;
use crate::settings;

const FENCE_LABEL: &str = "desktop-fence";
static ACTIVE: AtomicBool = AtomicBool::new(false);
static WATCH_GEN: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub kind: String,
    #[serde(default)]
    pub builtin: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellMenuEntry {
    pub id: u32,
    pub label: String,
    pub disabled: bool,
    pub separator: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<ShellMenuEntry>>,
    pub menu_path: Vec<u32>,
}


fn run_on_ui<T, F>(app: &AppHandle, f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    app.run_on_main_thread(move || {
        let _ = tx.send(f());
    })
    .map_err(|e| format!("无法切到 UI 线程: {e}"))?;
    rx.recv()
        .map_err(|e| format!("等待 UI 线程失败: {e}"))
}

fn scan_dir(dir: &Path, items: &mut Vec<DesktopItem>, seen: &mut std::collections::HashSet<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        let path_key = path.to_string_lossy().to_string();
        if seen.contains(&path_key) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        let is_dir = meta.is_dir();
        let kind = classify_kind(&name, is_dir);
        let (display_name, mut icon) = shell_name_and_icon(&path, &name, is_dir);
        #[cfg(windows)]
        if kind == "image" {
            if let Some(preview) = win::image_file_preview(&path, 96) {
                icon = Some(preview);
            }
        }
        seen.insert(path_key);
        items.push(DesktopItem {
            name: display_name,
            path: path.to_string_lossy().to_string(),
            is_dir,
            kind,
            builtin: false,
            icon,
        });
    }
}

fn file_ext(file_name: &str) -> String {
    Path::new(file_name)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
}

fn classify_kind(file_name: &str, is_dir: bool) -> String {
    if is_dir {
        return "folder".into();
    }
    let ext = file_ext(file_name);
    const APPS: &[&str] = &["lnk", "url", "exe", "bat", "cmd", "msi", "com", "appref-ms"];
    const IMAGES: &[&str] = &[
        "png", "jpg", "jpeg", "gif", "webp", "bmp", "ico", "svg", "tif", "tiff", "heic", "heif",
        "raw", "dng", "jfif",
    ];
    const DOCS: &[&str] = &[
        "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", "md", "csv", "rtf", "odt",
        "ods", "odp", "epub", "wps", "et", "dps",
    ];
    const ARCHIVES: &[&str] = &[
        "zip", "rar", "7z", "tar", "gz", "bz2", "xz", "iso", "cab", "arj", "lzh",
    ];
    const MEDIA: &[&str] = &[
        "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "mpg", "mpeg", "3gp",
        "mp3", "wav", "flac", "aac", "m4a", "wma", "ogg", "opus", "aiff", "mid",
    ];
    if APPS.contains(&ext.as_str()) {
        "app".into()
    } else if IMAGES.contains(&ext.as_str()) {
        "image".into()
    } else if DOCS.contains(&ext.as_str()) {
        "document".into()
    } else if ARCHIVES.contains(&ext.as_str()) {
        "archive".into()
    } else if MEDIA.contains(&ext.as_str()) {
        "media".into()
    } else {
        "other".into()
    }
}

fn fallback_display_name(file_name: &str) -> String {
    let lower = file_name.to_ascii_lowercase();
    if let Some(stripped) = lower
        .strip_suffix(".lnk")
        .or_else(|| lower.strip_suffix(".url"))
        .or_else(|| lower.strip_suffix(".exe"))
    {
        return file_name[..stripped.len()].to_string();
    }
    file_name.to_string()
}

fn shell_name_and_icon(path: &Path, file_name: &str, _is_dir: bool) -> (String, Option<String>) {
    #[cfg(windows)]
    {
        let meta = win::shell_name_and_icon(path);
        let name = meta
            .display_name
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| fallback_display_name(file_name));
        return (name, meta.icon);
    }
    #[cfg(not(windows))]
    {
        (fallback_display_name(file_name), None)
    }
}

pub fn scan_desktop_items() -> Result<Vec<DesktopItem>, String> {
    let mut items = Vec::new();
    let mut seen = std::collections::HashSet::new();
    #[cfg(windows)]
    for item in win::scan_builtin_desktop_icons() {
        seen.insert(item.path.clone());
        items.push(item);
    }
    for dir in desktop_scan_dirs() {
        scan_dir(&dir, &mut items, &mut seen);
    }
    items.sort_by(|a, b| {
        builtin_rank(&a.path).cmp(&builtin_rank(&b.path))
            .then_with(|| b.is_dir.cmp(&a.is_dir))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(items)
}

fn builtin_rank(path: &str) -> u8 {
    match path.to_ascii_uppercase() {
        p if p.contains("20D04FE0-3AEA-1069-A2D8-08002B30309D") => 0,
        p if p.contains("645FF040-5081-101B-9F08-00AA002F954E") => 1,
        p if p.contains("F02C1A0D-B21F-4110-8426-0A0C959C3602") => 2,
        _ => 3,
    }
}

#[cfg(windows)]
mod win {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

    use windows::core::{BOOL, PCWSTR, PWSTR};
    use windows::Win32::Foundation::{
        COLORREF, HWND, LPARAM, LRESULT, RECT, SIZE, WPARAM,
    };
    use windows::Win32::Graphics::Gdi::{
        EnumDisplayMonitors, GetMonitorInfoW, RedrawWindow, HDC, HMONITOR, MONITORINFO,
        RDW_ALLCHILDREN, RDW_INVALIDATE, RDW_UPDATENOW,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CallWindowProcW, EnumWindows, FindWindowExW, FindWindowW, GetParent, GetSystemMetrics,
        GetWindowLongPtrW, GetWindowRect, SetLayeredWindowAttributes, SetParent,
        SetWindowLongPtrW, SetWindowPos, SetWindowTextW, ShowWindow, GWL_EXSTYLE, GWL_STYLE,
        GWLP_WNDPROC, HICON, HTCLIENT, HWND_TOP, LWA_ALPHA, MONITORINFOF_PRIMARY, SM_CXSCREEN,
        SM_CYSCREEN, STYLESTRUCT, SW_HIDE, SW_SHOW, SWP_FRAMECHANGED, SWP_HIDEWINDOW,
        SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, WM_NCCALCSIZE,
        WM_NCHITTEST, WM_NCPAINT, WM_SETTEXT, WM_STYLECHANGED, WM_STYLECHANGING, WS_BORDER,
        WS_CAPTION, WS_CHILD, WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_DLGFRAME, WS_EX_APPWINDOW,
        WS_EX_CLIENTEDGE, WS_EX_DLGMODALFRAME, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_STATICEDGE,
        WS_EX_TOOLWINDOW, WS_EX_WINDOWEDGE, WS_POPUP, WS_SYSMENU, WS_THICKFRAME, WS_VISIBLE,
    };

    static ORIG_WNDPROC: AtomicIsize = AtomicIsize::new(0);
    static FENCE_SHOWN: AtomicBool = AtomicBool::new(false);
    static DESKTOP_DEFVIEW: AtomicIsize = AtomicIsize::new(0);

    pub struct ShellMeta {
        pub display_name: Option<String>,
        pub icon: Option<String>,
    }

    fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    fn wide_path(path: &std::path::Path) -> Vec<u16> {
        path.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    fn to_base64(data: &[u8]) -> String {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
        let mut i = 0;
        while i + 3 <= data.len() {
            let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8) | data[i + 2] as u32;
            out.push(CHARS[((n >> 18) & 63) as usize] as char);
            out.push(CHARS[((n >> 12) & 63) as usize] as char);
            out.push(CHARS[((n >> 6) & 63) as usize] as char);
            out.push(CHARS[(n & 63) as usize] as char);
            i += 3;
        }
        match data.len() - i {
            1 => {
                let n = (data[i] as u32) << 16;
                out.push(CHARS[((n >> 18) & 63) as usize] as char);
                out.push(CHARS[((n >> 12) & 63) as usize] as char);
                out.push('=');
                out.push('=');
            }
            2 => {
                let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8);
                out.push(CHARS[((n >> 18) & 63) as usize] as char);
                out.push(CHARS[((n >> 12) & 63) as usize] as char);
                out.push(CHARS[((n >> 6) & 63) as usize] as char);
                out.push('=');
            }
            _ => {}
        }
        out
    }

    fn rgba_to_png_bytes(rgba: &[u8], w: u32, h: u32) -> Option<Vec<u8>> {
        let mut buf = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut buf, w, h);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().ok()?;
            writer.write_image_data(rgba).ok()?;
        }
        Some(buf)
    }

    fn rgba_to_png_data_url(rgba: &[u8], w: u32, h: u32) -> Option<String> {
        rgba_to_png_bytes(rgba, w, h).map(|buf| format!("data:image/png;base64,{}", to_base64(&buf)))
    }

    fn expand_env_path(s: &str) -> String {
        let mut out = String::new();
        let mut rest = s;
        while let Some(start) = rest.find('%') {
            out.push_str(&rest[..start]);
            rest = &rest[start + 1..];
            if let Some(end) = rest.find('%') {
                let key = &rest[..end];
                if key.is_empty() {
                    out.push('%');
                } else if let Ok(val) = std::env::var(key) {
                    out.push_str(&val);
                } else {
                    out.push('%');
                    out.push_str(key);
                    out.push('%');
                }
                rest = &rest[end + 1..];
            } else {
                out.push('%');
                out.push_str(rest);
                rest = "";
            }
        }
        out.push_str(rest);
        out
    }

    fn parse_icon_location(raw: &str, fallback_index: i32) -> (std::path::PathBuf, i32) {
        let trimmed = raw.trim().trim_matches('"');
        let (path_part, index) = if let Some((left, right)) = trimmed.rsplit_once(',') {
            if let Ok(n) = right.trim().parse::<i32>() {
                (left.trim().trim_matches('"'), n)
            } else {
                (trimmed, fallback_index)
            }
        } else {
            (trimmed, fallback_index)
        };
        (std::path::PathBuf::from(expand_env_path(path_part)), index)
    }

    fn resolve_icon_file(path: std::path::PathBuf) -> std::path::PathBuf {
        if path.exists() {
            return path;
        }
        if let Ok(root) = std::env::var("SystemRoot") {
            let sys32 = std::path::Path::new(&root).join("System32").join(&path);
            if sys32.exists() {
                return sys32;
            }
            let syswow = std::path::Path::new(&root).join("SysWOW64").join(&path);
            if syswow.exists() {
                return syswow;
            }
        }
        path
    }

    unsafe fn hbitmap_to_png_data_url(
        hbmp: windows::Win32::Graphics::Gdi::HBITMAP,
    ) -> Option<String> {
        use windows::Win32::Graphics::Gdi::{
            CreateCompatibleDC, DeleteDC, GetDIBits, GetObjectW, BITMAP, BITMAPINFO,
            BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
        };

        let mut bm = BITMAP::default();
        if GetObjectW(
            hbmp.into(),
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bm as *mut BITMAP as *mut core::ffi::c_void),
        ) == 0
            || bm.bmWidth <= 0
            || bm.bmHeight == 0
        {
            return None;
        }
        let w = bm.bmWidth;
        let h = bm.bmHeight.abs();
        let hdc = CreateCompatibleDC(None);
        if hdc.is_invalid() {
            return None;
        }
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [Default::default()],
        };
        let mut bits = vec![0u8; (w * h * 4) as usize];
        let got = GetDIBits(
            hdc,
            hbmp,
            0,
            h as u32,
            Some(bits.as_mut_ptr() as *mut core::ffi::c_void),
            &mut bmi,
            DIB_RGB_COLORS,
        );
        let _ = DeleteDC(hdc);
        if got == 0 {
            return None;
        }
        let mut rgba = Vec::with_capacity(bits.len());
        for px in bits.chunks_exact(4) {
            let b = px[0] as u32;
            let g = px[1] as u32;
            let r = px[2] as u32;
            let a = px[3] as u32;
            let (r, g, b) = if a > 0 && a < 255 {
                (
                    (r * 255 / a).min(255) as u8,
                    (g * 255 / a).min(255) as u8,
                    (b * 255 / a).min(255) as u8,
                )
            } else {
                (r as u8, g as u8, b as u8)
            };
            rgba.extend_from_slice(&[r, g, b, a as u8]);
        }
        rgba_to_png_data_url(&rgba, w as u32, h as u32)
    }

    unsafe fn shell_item_image_png(
        path: &std::path::Path,
        px: i32,
        flags: windows::Win32::UI::Shell::SIIGBF,
    ) -> Option<String> {
        use windows::Win32::Graphics::Gdi::DeleteObject;
        use windows::Win32::UI::Shell::{IShellItemImageFactory, SHCreateItemFromParsingName};

        let wpath = wide_path(path);
        let factory: IShellItemImageFactory =
            SHCreateItemFromParsingName(PCWSTR(wpath.as_ptr()), None).ok()?;
        let hbmp = factory
            .GetImage(SIZE { cx: px, cy: px }, flags)
            .ok()?;
        if hbmp.is_invalid() {
            return None;
        }
        let url = hbitmap_to_png_data_url(hbmp);
        let _ = DeleteObject(hbmp.into());
        url
    }

    pub fn image_file_preview(path: &std::path::Path, px: i32) -> Option<String> {
        use windows::Win32::UI::Shell::{
            SIIGBF_BIGGERSIZEOK, SIIGBF_SCALEUP, SIIGBF_THUMBNAILONLY,
        };

        unsafe {
            shell_item_image_png(
                path,
                px,
                SIIGBF_THUMBNAILONLY | SIIGBF_BIGGERSIZEOK | SIIGBF_SCALEUP,
            )
            .or_else(|| {
                shell_item_image_png(path, px, SIIGBF_BIGGERSIZEOK | SIIGBF_SCALEUP)
            })
        }
    }

    unsafe fn hicon_native_size(hicon: HICON) -> i32 {
        use windows::Win32::Graphics::Gdi::{DeleteObject, GetObjectW, BITMAP};
        use windows::Win32::UI::WindowsAndMessaging::{GetIconInfo, ICONINFO};

        let mut info = ICONINFO::default();
        if GetIconInfo(hicon, &mut info).is_err() {
            return 256;
        }
        let mut bm = BITMAP::default();
        let hbmp = if info.hbmColor.is_invalid() {
            info.hbmMask
        } else {
            info.hbmColor
        };
        GetObjectW(
            hbmp.into(),
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bm as *mut BITMAP as *mut core::ffi::c_void),
        );
        if !info.hbmColor.is_invalid() {
            let _ = DeleteObject(info.hbmColor.into());
        }
        if !info.hbmMask.is_invalid() {
            let _ = DeleteObject(info.hbmMask.into());
        }
        bm.bmWidth.clamp(16, 256)
    }

    unsafe fn hicon_to_png_data_url(hicon: HICON) -> Option<String> {
        use windows::Win32::Graphics::Gdi::{
            CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, SelectObject, BITMAPINFO,
            BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
        };
        use windows::Win32::UI::WindowsAndMessaging::{DrawIconEx, DI_NORMAL};

        if hicon.0.is_null() {
            return None;
        }
        let size = hicon_native_size(hicon);
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: size,
                biHeight: -size,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [Default::default()],
        };
        let hdc = CreateCompatibleDC(None);
        if hdc.is_invalid() {
            return None;
        }
        let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
        let dib = match CreateDIBSection(Some(hdc), &bmi, DIB_RGB_COLORS, &mut bits, None, 0) {
            Ok(d) => d,
            Err(_) => {
                let _ = DeleteDC(hdc);
                return None;
            }
        };
        if dib.is_invalid() || bits.is_null() {
            let _ = DeleteDC(hdc);
            return None;
        }
        let old = SelectObject(hdc, dib.into());
        let pixel_count = (size * size) as usize;
        std::ptr::write_bytes(bits, 0, pixel_count * 4);
        let _ = DrawIconEx(hdc, 0, 0, hicon, size, size, 0, None, DI_NORMAL);

        let raw = std::slice::from_raw_parts(bits as *const u8, pixel_count * 4);
        let mut rgba = Vec::with_capacity(pixel_count * 4);
        let mut has_alpha = false;
        for px in raw.chunks_exact(4) {
            let b = px[0];
            let g = px[1];
            let r = px[2];
            let a = px[3];
            if a != 0 {
                has_alpha = true;
            }
            rgba.extend_from_slice(&[r, g, b, a]);
        }
        if !has_alpha {
            for px in rgba.chunks_exact_mut(4) {
                if px[0] != 0 || px[1] != 0 || px[2] != 0 {
                    px[3] = 255;
                }
            }
        }

        SelectObject(hdc, old);
        let _ = DeleteObject(dib.into());
        let _ = DeleteDC(hdc);
        rgba_to_png_data_url(&rgba, size as u32, size as u32)
    }

    unsafe fn image_list_icon(list_id: u32, index: i32) -> Option<HICON> {
        use windows::Win32::UI::Controls::{IImageList, ILD_TRANSPARENT};
        use windows::Win32::UI::Shell::SHGetImageList;

        let list: IImageList = SHGetImageList(list_id as i32).ok()?;
        let icon = list.GetIcon(index, ILD_TRANSPARENT.0).ok()?;
        if icon.0.is_null() {
            None
        } else {
            Some(icon)
        }
    }

    fn parse_ico_sizes(bytes: &[u8]) -> Vec<i32> {
        if bytes.len() < 6 {
            return Vec::new();
        }
        let count = u16::from_le_bytes([bytes[4], bytes[5]]) as usize;
        let mut sizes = Vec::new();
        let mut off = 6usize;
        for _ in 0..count {
            if off + 16 > bytes.len() {
                break;
            }
            let w = bytes[off];
            sizes.push(if w == 0 { 256 } else { w as i32 });
            off += 16;
        }
        sizes
    }

    unsafe extern "system" fn enum_group_icons(
        hmodule: windows::Win32::Foundation::HMODULE,
        _lptype: PCWSTR,
        lpname: PCWSTR,
        lparam: isize,
    ) -> BOOL {
        use windows::Win32::System::LibraryLoader::{
            FindResourceW, LoadResource, LockResource, SizeofResource,
        };
        use windows::Win32::UI::WindowsAndMessaging::RT_GROUP_ICON;

        let groups = &mut *(lparam as *mut Vec<(i32, Vec<i32>)>);
        let id = {
            let p = lpname.0 as usize;
            if p < 0x10000 {
                p as i32
            } else {
                0
            }
        };
        let hrsrc = FindResourceW(Some(hmodule), lpname, RT_GROUP_ICON);
        if hrsrc.0.is_null() {
            return BOOL(1);
        }
        let size = SizeofResource(Some(hmodule), hrsrc) as usize;
        let Ok(hdata) = LoadResource(Some(hmodule), hrsrc) else {
            return BOOL(1);
        };
        let ptr = LockResource(hdata) as *const u8;
        if ptr.is_null() || size < 6 {
            return BOOL(1);
        }
        let bytes = std::slice::from_raw_parts(ptr, size);
        let count = u16::from_le_bytes([bytes[4], bytes[5]]) as usize;
        let mut sizes = Vec::new();
        let mut off = 6usize;
        for _ in 0..count {
            if off + 14 > bytes.len() {
                break;
            }
            let w = bytes[off];
            sizes.push(if w == 0 { 256 } else { w as i32 });
            off += 14;
        }
        groups.push((id, sizes));
        BOOL(1)
    }

    unsafe fn pe_icon_sizes(path: &std::path::Path, index: i32) -> Vec<i32> {
        use windows::Win32::Foundation::FreeLibrary;
        use windows::Win32::System::LibraryLoader::{
            EnumResourceNamesW, LoadLibraryExW, LOAD_LIBRARY_AS_DATAFILE,
            LOAD_LIBRARY_AS_IMAGE_RESOURCE,
        };
        use windows::Win32::UI::WindowsAndMessaging::RT_GROUP_ICON;

        let wpath = wide_path(path);
        let Ok(module) = LoadLibraryExW(
            PCWSTR(wpath.as_ptr()),
            None,
            LOAD_LIBRARY_AS_DATAFILE | LOAD_LIBRARY_AS_IMAGE_RESOURCE,
        ) else {
            return Vec::new();
        };
        let mut groups: Vec<(i32, Vec<i32>)> = Vec::new();
        let _ = EnumResourceNamesW(
            Some(module),
            RT_GROUP_ICON,
            Some(enum_group_icons),
            &mut groups as *mut _ as isize,
        );
        let _ = FreeLibrary(module);
        if index < 0 {
            let id = -index;
            return groups
                .into_iter()
                .find(|(gid, _)| *gid == id)
                .map(|(_, s)| s)
                .unwrap_or_default();
        }
        groups
            .iter()
            .find(|(gid, _)| *gid == index)
            .map(|(_, s)| s.clone())
            .or_else(|| groups.get(index as usize).map(|(_, s)| s.clone()))
            .unwrap_or_default()
    }

    fn native_icon_sizes(path: &std::path::Path, index: i32) -> Vec<i32> {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if ext == "ico" || ext == "cur" {
            if let Ok(bytes) = std::fs::read(path) {
                return parse_ico_sizes(&bytes);
            }
            return Vec::new();
        }
        unsafe { pe_icon_sizes(path, index) }
    }

    fn path_to_fixed_wide(path: &std::path::Path) -> [u16; 260] {
        let mut buf = [0u16; 260];
        let wide: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let n = wide.len().min(259);
        buf[..n].copy_from_slice(&wide[..n]);
        buf
    }

    unsafe fn extract_icon_at(path: &std::path::Path, index: i32, size: i32) -> Option<HICON> {
        use windows::Win32::UI::Shell::SHDefExtractIconW;
        use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, PrivateExtractIconsW};

        let fixed = path_to_fixed_wide(path);
        let wpath = wide_path(path);
        let mut icons = [HICON::default()];
        let mut icon_id: u32 = 0;
        let got = PrivateExtractIconsW(
            &fixed,
            index,
            size,
            size,
            Some(&mut icons),
            Some(&mut icon_id),
            0,
        );
        let icon = icons[0];
        if got > 0 && !icon.0.is_null() {
            let native = hicon_native_size(icon);
            if (native - size).abs() <= 8 {
                return Some(icon);
            }
            let _ = DestroyIcon(icon);
            return None;
        }
        if size > 48 {
            return None;
        }
        let mut large = HICON::default();
        let mut small = HICON::default();
        let nsize = (size as u32) | ((size as u32) << 16);
        let hr = SHDefExtractIconW(
            PCWSTR(wpath.as_ptr()),
            index,
            0,
            Some(&mut large),
            Some(&mut small),
            nsize,
        );
        if !small.0.is_null() {
            let _ = DestroyIcon(small);
        }
        if hr.is_ok() && !large.0.is_null() {
            Some(large)
        } else {
            if !large.0.is_null() {
                let _ = DestroyIcon(large);
            }
            None
        }
    }

    unsafe fn assoc_default_icon(path: &std::path::Path) -> Option<(std::path::PathBuf, i32)> {
        use windows::Win32::UI::Shell::{
            AssocQueryStringW, ASSOCF_INIT_DEFAULTTOSTAR, ASSOCF_NOTRUNCATE, ASSOCSTR_DEFAULTICON,
        };

        let ext = path.extension()?.to_str()?;
        let assoc = wide(&format!(".{}", ext.to_ascii_lowercase()));
        let mut len: u32 = 0;
        let _ = AssocQueryStringW(
            ASSOCF_INIT_DEFAULTTOSTAR | ASSOCF_NOTRUNCATE,
            ASSOCSTR_DEFAULTICON,
            PCWSTR(assoc.as_ptr()),
            PCWSTR::null(),
            None,
            &mut len,
        );
        if len == 0 || len > 4096 {
            return None;
        }
        let mut buf = vec![0u16; len as usize];
        let mut written = len;
        let hr = AssocQueryStringW(
            ASSOCF_INIT_DEFAULTTOSTAR | ASSOCF_NOTRUNCATE,
            ASSOCSTR_DEFAULTICON,
            PCWSTR(assoc.as_ptr()),
            PCWSTR::null(),
            Some(PWSTR(buf.as_mut_ptr())),
            &mut written,
        );
        if hr.is_err() {
            return None;
        }
        let n = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        if n == 0 {
            return None;
        }
        let loc = String::from_utf16_lossy(&buf[..n]);
        let (p, idx) = parse_icon_location(&loc, 0);
        let p = resolve_icon_file(p);
        if p.exists() {
            Some((p, idx))
        } else {
            None
        }
    }

    fn parse_internet_shortcut_icon(path: &std::path::Path) -> Option<(std::path::PathBuf, i32)> {
        let content = std::fs::read_to_string(path).ok()?;
        let mut icon_file: Option<String> = None;
        let mut icon_index = 0i32;
        for line in content.lines() {
            let line = line.trim();
            if let Some(value) = line.strip_prefix("IconFile=") {
                let trimmed = value.trim().trim_matches('"');
                if !trimmed.is_empty() {
                    icon_file = Some(expand_env_path(trimmed));
                }
            } else if let Some(value) = line.strip_prefix("IconIndex=") {
                icon_index = value.trim().parse().unwrap_or(0);
            }
        }
        let icon_path = resolve_icon_file(std::path::PathBuf::from(icon_file?));
        if icon_path.exists() {
            Some((icon_path, icon_index))
        } else {
            None
        }
    }

    unsafe fn icon_source(path: &std::path::Path) -> (std::path::PathBuf, i32) {
        if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("url"))
        {
            if let Some(found) = parse_internet_shortcut_icon(path) {
                return found;
            }
        }

        use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
        use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICONLOCATION};

        let wpath = wide_path(path);
        let mut info = SHFILEINFOW::default();
        let ok = SHGetFileInfoW(
            PCWSTR(wpath.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICONLOCATION,
        );
        if ok != 0 {
            let raw = info.szDisplayName;
            let len = raw.iter().position(|&c| c == 0).unwrap_or(raw.len());
            if len > 0 {
                let loc = String::from_utf16_lossy(&raw[..len]);
                let (p, idx) = parse_icon_location(&loc, info.iIcon);
                let p = resolve_icon_file(p);
                if p.exists() {
                    return (p, idx);
                }
            }
        }
        if let Some(assoc) = assoc_default_icon(path) {
            return assoc;
        }
        (path.to_path_buf(), 0)
    }

    unsafe fn extract_best_icon(path: &std::path::Path, sys_index: i32) -> Option<HICON> {
        use windows::Win32::UI::Shell::{SHIL_EXTRALARGE, SHIL_LARGE};

        let (src, index) = icon_source(path);
        let sizes = native_icon_sizes(&src, index);
        let want = sizes.iter().copied().max().unwrap_or(0);
        if want >= 16 {
            if let Some(icon) = extract_icon_at(&src, index, want) {
                return Some(icon);
            }
            if let Some(icon) = extract_icon_at(path, 0, want) {
                return Some(icon);
            }
        }
        if want >= 48 || want == 0 {
            if let Some(icon) = image_list_icon(SHIL_EXTRALARGE, sys_index) {
                return Some(icon);
            }
        }
        image_list_icon(SHIL_LARGE, sys_index)
    }

    pub fn shell_name_and_icon(path: &std::path::Path) -> ShellMeta {
        use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
        use windows::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
        use windows::Win32::UI::Shell::{
            SHGetFileInfoW, SHFILEINFOW, SHGFI_DISPLAYNAME, SHGFI_ICON, SHGFI_LARGEICON,
            SHGFI_SYSICONINDEX, SIIGBF_BIGGERSIZEOK, SIIGBF_ICONONLY,
        };
        use windows::Win32::UI::WindowsAndMessaging::DestroyIcon;

        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            let wpath = wide_path(path);
            let mut info = SHFILEINFOW::default();
            let flags = SHGFI_SYSICONINDEX | SHGFI_DISPLAYNAME | SHGFI_ICON | SHGFI_LARGEICON;
            let ok = SHGetFileInfoW(
                PCWSTR(wpath.as_ptr()),
                FILE_FLAGS_AND_ATTRIBUTES(0),
                Some(&mut info),
                std::mem::size_of::<SHFILEINFOW>() as u32,
                flags,
            );

            let display_name = {
                let raw = info.szDisplayName;
                let len = raw.iter().position(|&c| c == 0).unwrap_or(raw.len());
                if len == 0 {
                    None
                } else {
                    Some(String::from_utf16_lossy(&raw[..len]))
                }
            };

            let mut icon = None;
            if ok != 0 {
                icon = shell_item_image_png(path, 48, SIIGBF_ICONONLY | SIIGBF_BIGGERSIZEOK);
                if icon.is_none() {
                    if let Some(best) = extract_best_icon(path, info.iIcon) {
                        icon = hicon_to_png_data_url(best);
                        let _ = DestroyIcon(best);
                    }
                }
            }
            if icon.is_none() && ok != 0 && !info.hIcon.0.is_null() {
                icon = hicon_to_png_data_url(info.hIcon);
            }
            if !info.hIcon.0.is_null() {
                let _ = DestroyIcon(info.hIcon);
            }

            ShellMeta { display_name, icon }
        }
    }

    unsafe fn stock_icon_png(siid: windows::Win32::UI::Shell::SHSTOCKICONID) -> Option<String> {
        use windows::Win32::UI::Shell::{
            SHGetStockIconInfo, SHGSI_ICON, SHGSI_LARGEICON, SHSTOCKICONINFO,
        };
        use windows::Win32::UI::WindowsAndMessaging::DestroyIcon;

        let mut info = SHSTOCKICONINFO {
            cbSize: std::mem::size_of::<SHSTOCKICONINFO>() as u32,
            ..Default::default()
        };
        if SHGetStockIconInfo(siid, SHGSI_ICON | SHGSI_LARGEICON, &mut info).is_err()
            || info.hIcon.0.is_null()
        {
            return None;
        }
        let url = hicon_to_png_data_url(info.hIcon);
        let _ = DestroyIcon(info.hIcon);
        url
    }

    fn extract_builtin_icon(
        path: &str,
        stock_fallback: Option<windows::Win32::UI::Shell::SHSTOCKICONID>,
    ) -> Option<String> {
        use windows::Win32::UI::Shell::{SIID_MYNETWORK, SIIGBF_BIGGERSIZEOK, SIIGBF_ICONONLY};

        let path_obj = std::path::PathBuf::from(path);
        let flags = SIIGBF_ICONONLY | SIIGBF_BIGGERSIZEOK;
        unsafe {
            if let Some(icon) = shell_item_image_png(&path_obj, 48, flags) {
                return Some(icon);
            }
            if path.contains("F02C1A0D-B21F-4110-8426-0A0C959C3602") {
                for alt in [
                    "shell:NetworkPlacesFolder",
                    "::{F02C1A0D-B21F-4110-8426-0A0C959C3602}\\",
                ] {
                    if let Some(icon) = shell_item_image_png(std::path::Path::new(alt), 48, flags)
                    {
                        return Some(icon);
                    }
                }
                if let Some(icon) = stock_icon_png(SIID_MYNETWORK) {
                    return Some(icon);
                }
            }
            if let Some(siid) = stock_fallback {
                if let Some(icon) = stock_icon_png(siid) {
                    return Some(icon);
                }
            }
        }
        shell_name_and_icon(&path_obj).icon
    }

    const BUILTIN_DESKTOP_ICONS: &[(
        &str,
        &str,
        Option<windows::Win32::UI::Shell::SHSTOCKICONID>,
    )] = &[
        ("::{20D04FE0-3AEA-1069-A2D8-08002B30309D}", "此电脑", None),
        (
            "::{645FF040-5081-101B-9F08-00AA002F954E}",
            "回收站",
            Some(windows::Win32::UI::Shell::SIID_RECYCLER),
        ),
        (
            "::{F02C1A0D-B21F-4110-8426-0A0C959C3602}",
            "网络",
            Some(windows::Win32::UI::Shell::SIID_MYNETWORK),
        ),
    ];

    pub fn scan_builtin_desktop_icons() -> Vec<super::DesktopItem> {
        let mut items = Vec::with_capacity(BUILTIN_DESKTOP_ICONS.len());
        for (path, fallback_name, stock) in BUILTIN_DESKTOP_ICONS {
            let path_obj = std::path::PathBuf::from(*path);
            let meta = shell_name_and_icon(&path_obj);
            let icon = extract_builtin_icon(path, *stock);
            let name = meta
                .display_name
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| (*fallback_name).to_string());
            items.push(super::DesktopItem {
                name,
                path: (*path).to_string(),
                is_dir: false,
                kind: "app".into(),
                builtin: true,
                icon,
            });
        }
        items
    }

    fn rect_of(hwnd: HWND) -> (i32, i32, i32, i32) {
        unsafe {
            let mut r = RECT::default();
            let _ = GetWindowRect(hwnd, &mut r);
            (r.left, r.top, r.right - r.left, r.bottom - r.top)
        }
    }

    struct EnumData {
        defview: HWND,
        defview_parent: HWND,
    }

    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let data = &mut *(lparam.0 as *mut EnumData);
        let class_def = wide("SHELLDLL_DefView");
        if let Ok(def) =
            FindWindowExW(Some(hwnd), None, PCWSTR(class_def.as_ptr()), PCWSTR::null())
        {
            data.defview = def;
            data.defview_parent = hwnd;
        }
        BOOL(1)
    }

    fn find_progman_child(progman: HWND, class: &str) -> HWND {
        unsafe {
            let cls = wide(class);
            FindWindowExW(Some(progman), None, PCWSTR(cls.as_ptr()), PCWSTR::null())
                .unwrap_or_default()
        }
    }

    unsafe extern "system" fn enum_monitors_proc(
        hmon: HMONITOR,
        _hdc: HDC,
        _lprc: *mut RECT,
        lparam: LPARAM,
    ) -> BOOL {
        let found = &mut *(lparam.0 as *mut Option<(i32, i32, i32, i32)>);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if GetMonitorInfoW(hmon, &mut info).as_bool()
            && info.dwFlags & MONITORINFOF_PRIMARY != 0
        {
            let r = info.rcWork;
            *found = Some((r.left, r.top, r.right - r.left, r.bottom - r.top));
            return BOOL(0);
        }
        BOOL(1)
    }

    fn primary_work_rect() -> (i32, i32, i32, i32) {
        let mut found: Option<(i32, i32, i32, i32)> = None;
        unsafe {
            let _ = EnumDisplayMonitors(
                None,
                None,
                Some(enum_monitors_proc),
                LPARAM(&mut found as *mut _ as isize),
            );
        }
        found.unwrap_or_else(|| unsafe {
            (
                0,
                0,
                GetSystemMetrics(SM_CXSCREEN),
                GetSystemMetrics(SM_CYSCREEN),
            )
        })
    }

    fn force_child_chrome(child: HWND, visible: bool) {
        unsafe {
            let mut style = GetWindowLongPtrW(child, GWL_STYLE) as u32;
            style &= !(WS_POPUP.0
                | WS_CAPTION.0
                | WS_THICKFRAME.0
                | WS_BORDER.0
                | WS_DLGFRAME.0
                | WS_SYSMENU.0);
            style |= WS_CHILD.0 | WS_CLIPSIBLINGS.0 | WS_CLIPCHILDREN.0;
            if visible {
                style |= WS_VISIBLE.0;
            } else {
                style &= !WS_VISIBLE.0;
            }
            SetWindowLongPtrW(child, GWL_STYLE, style as isize);

            let mut ex = GetWindowLongPtrW(child, GWL_EXSTYLE) as u32;
            ex &= !(WS_EX_APPWINDOW.0
                | WS_EX_CLIENTEDGE.0
                | WS_EX_WINDOWEDGE.0
                | WS_EX_DLGMODALFRAME.0
                | WS_EX_STATICEDGE.0);
            ex |= WS_EX_TOOLWINDOW.0 | WS_EX_NOACTIVATE.0 | WS_EX_LAYERED.0;
            SetWindowLongPtrW(child, GWL_EXSTYLE, ex as isize);
            let _ = SetLayeredWindowAttributes(child, COLORREF(0), 255, LWA_ALPHA);
            let empty = [0u16];
            let _ = SetWindowTextW(child, PCWSTR(empty.as_ptr()));
        }
    }

    unsafe extern "system" fn fence_wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if msg == WM_SETTEXT {
            return LRESULT(1);
        }
        if msg == WM_NCCALCSIZE || msg == WM_NCPAINT {
            return LRESULT(0);
        }
        if msg == WM_NCHITTEST {
            return LRESULT(HTCLIENT as isize);
        }
        if msg == WM_STYLECHANGING && lparam.0 != 0 {
            let ss = &mut *(lparam.0 as *mut STYLESTRUCT);
            if wparam.0 as isize == GWL_STYLE.0 as isize {
                ss.styleNew &= !(WS_POPUP.0
                    | WS_CAPTION.0
                    | WS_THICKFRAME.0
                    | WS_BORDER.0
                    | WS_DLGFRAME.0
                    | WS_SYSMENU.0);
                ss.styleNew |= WS_CHILD.0 | WS_CLIPSIBLINGS.0 | WS_CLIPCHILDREN.0;
                if FENCE_SHOWN.load(Ordering::SeqCst) {
                    ss.styleNew |= WS_VISIBLE.0;
                } else {
                    ss.styleNew &= !WS_VISIBLE.0;
                }
            }
            if wparam.0 as isize == GWL_EXSTYLE.0 as isize {
                ss.styleNew &= !(WS_EX_APPWINDOW.0
                    | WS_EX_CLIENTEDGE.0
                    | WS_EX_WINDOWEDGE.0
                    | WS_EX_DLGMODALFRAME.0
                    | WS_EX_STATICEDGE.0);
                ss.styleNew |= WS_EX_TOOLWINDOW.0 | WS_EX_NOACTIVATE.0 | WS_EX_LAYERED.0;
            }
        }
        if msg == WM_STYLECHANGED && FENCE_SHOWN.load(Ordering::SeqCst) {
            let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
            if style & (WS_POPUP.0 | WS_CAPTION.0) != 0 || style & WS_CHILD.0 == 0 {
                force_child_chrome(hwnd, true);
            }
        }
        let orig = ORIG_WNDPROC.load(Ordering::SeqCst);
        if orig == 0 {
            return LRESULT(0);
        }
        let proc: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT =
            std::mem::transmute(orig);
        CallWindowProcW(Some(proc), hwnd, msg, wparam, lparam)
    }

    fn install_subclass(hwnd: HWND) {
        unsafe {
            if ORIG_WNDPROC.load(Ordering::SeqCst) != 0 {
                return;
            }
            let prev = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, fence_wndproc as *const () as isize);
            if prev != 0 {
                ORIG_WNDPROC.store(prev, Ordering::SeqCst);
            }
        }
    }

    fn prepare_styles(child: HWND) {
        unsafe {
            use windows::Win32::Graphics::Dwm::{
                DwmExtendFrameIntoClientArea, DwmSetWindowAttribute, DWMNCRP_DISABLED,
                DWMWA_BORDER_COLOR, DWMWA_CAPTION_COLOR, DWMWA_COLOR_NONE, DWMWA_NCRENDERING_POLICY,
            };
            use windows::Win32::UI::Controls::MARGINS;

            let margins = MARGINS {
                cxLeftWidth: -1,
                cxRightWidth: -1,
                cyTopHeight: -1,
                cyBottomHeight: -1,
            };
            let _ = DwmExtendFrameIntoClientArea(child, &margins);

            let none = DWMWA_COLOR_NONE;
            let _ = DwmSetWindowAttribute(
                child,
                DWMWA_BORDER_COLOR,
                &none as *const _ as *const core::ffi::c_void,
                4,
            );
            let _ = DwmSetWindowAttribute(
                child,
                DWMWA_CAPTION_COLOR,
                &none as *const _ as *const core::ffi::c_void,
                4,
            );
            let policy = DWMNCRP_DISABLED;
            let _ = DwmSetWindowAttribute(
                child,
                DWMWA_NCRENDERING_POLICY,
                &policy as *const _ as *const core::ffi::c_void,
                4,
            );
        }
    }

    pub fn attach_fence_to_desktop(hwnd_raw: isize) -> Result<(i32, i32), String> {
        unsafe {
            let progman_class = wide("Progman");
            let progman = FindWindowW(PCWSTR(progman_class.as_ptr()), PCWSTR::null())
                .map_err(|_| "未找到 Progman 窗口".to_string())?;

            let mut data = EnumData {
                defview: HWND::default(),
                defview_parent: HWND::default(),
            };
            let _ = EnumWindows(Some(enum_windows_proc), LPARAM(&mut data as *mut _ as isize));

            if data.defview.0.is_null() {
                data.defview = find_progman_child(progman, "SHELLDLL_DefView");
                if !data.defview.0.is_null() {
                    data.defview_parent = progman;
                }
            }
            if !data.defview.0.is_null() {
                DESKTOP_DEFVIEW.store(data.defview.0 as isize, Ordering::SeqCst);
            }

            let child = HWND(hwnd_raw as *mut _);
            FENCE_SHOWN.store(true, Ordering::SeqCst);
            force_child_chrome(child, true);
            prepare_styles(child);

            let parent = if !data.defview.0.is_null() {
                data.defview
            } else {
                progman
            };

            let set_parent_err = SetParent(child, Some(parent)).is_err();
            if set_parent_err && GetParent(child).ok() != Some(parent) {
                return Err("SetParent 失败".into());
            }

            force_child_chrome(child, true);
            install_subclass(child);
            prepare_styles(child);

            let (pl, pt, pw, ph) = rect_of(parent);
            let (mx, my, mut mw, mut mh) = primary_work_rect();
            let mut x = mx - pl;
            let mut y = my - pt;
            if mw <= 0 || mh <= 0 {
                mw = if pw > 0 { pw } else { 1920 };
                mh = if ph > 0 { ph } else { 1080 };
                x = 0;
                y = 0;
            }

            let _ = SetWindowPos(
                child,
                Some(HWND_TOP),
                x,
                y,
                mw,
                mh,
                SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_FRAMECHANGED,
            );
            force_child_chrome(child, true);
            let _ = ShowWindow(child, SW_SHOW);
            let _ = RedrawWindow(
                Some(child),
                None,
                None,
                RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN,
            );

            eprintln!(
                "[desktop-organize] attached child={child:?} parent={parent:?} parent={pl},{pt} {pw}x{ph} primary={x},{y} {mw}x{mh}"
            );
            Ok((mw, mh))
        }
    }

    pub fn touch_fence_chrome(hwnd_raw: isize) {
        unsafe {
            if !FENCE_SHOWN.load(Ordering::SeqCst) {
                return;
            }
            let child = HWND(hwnd_raw as *mut _);
            let style = GetWindowLongPtrW(child, GWL_STYLE) as u32;
            if style & (WS_POPUP.0 | WS_CAPTION.0) != 0 || style & WS_CHILD.0 == 0 {
                force_child_chrome(child, true);
            }
            let empty = [0u16];
            let _ = SetWindowTextW(child, PCWSTR(empty.as_ptr()));
        }
    }

    pub fn hide_fence_from_desktop(hwnd_raw: isize) {
        unsafe {
            let child = HWND(hwnd_raw as *mut _);
            FENCE_SHOWN.store(false, Ordering::SeqCst);
            DESKTOP_DEFVIEW.store(0, Ordering::SeqCst);
            force_child_chrome(child, false);
            let _ = ShowWindow(child, SW_HIDE);
            let _ = SetParent(child, None);
            let _ = SetWindowPos(
                child,
                None,
                0,
                0,
                0,
                0,
                SWP_NOMOVE
                    | SWP_NOSIZE
                    | SWP_NOZORDER
                    | SWP_NOACTIVATE
                    | SWP_HIDEWINDOW
                    | SWP_FRAMECHANGED,
            );

            let progman_class = wide("Progman");
            let Ok(progman) = FindWindowW(PCWSTR(progman_class.as_ptr()), PCWSTR::null()) else {
                return;
            };
            let defview = find_progman_child(progman, "SHELLDLL_DefView");
            let refresh = if !defview.0.is_null() {
                defview
            } else {
                progman
            };
            let _ = RedrawWindow(
                Some(refresh),
                None,
                None,
                RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN,
            );
        }
    }

    pub fn shell_show_properties(path: &str) -> Result<(), String> {
        use windows::Win32::UI::Shell::ShellExecuteW;
        unsafe {
            let wpath = wide(path);
            let verb = wide("properties");
            let ret = ShellExecuteW(
                None,
                PCWSTR(verb.as_ptr()),
                PCWSTR(wpath.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOW,
            );
            if (ret.0 as isize) <= 32 {
                return Err(format!("打开属性失败: code={}", ret.0 as isize));
            }
        }
        Ok(())
    }
}

const SHELL_MENU_HOST_ARG: &str = "--lingscape-shell-menu-host";
const SHELL_MENU_TIMEOUT: Duration = Duration::from_secs(10);

pub(crate) fn shell_host_stage(stage: &str) {
    if let Some(path) = std::env::var_os("LINGSCAPE_SHELL_MENU_STATUS") {
        let _ = fs::write(path, stage);
    }
}

#[cfg(windows)]
fn run_shell_menu_host(
    mode: &str,
    path: Option<&str>,
    menu_path: &[u32],
) -> Result<Vec<ShellMenuEntry>, String> {
    use std::process::{Command, Stdio};

    let output_path = std::env::temp_dir().join(format!(
        "lingscape-shell-menu-{}-{}.json",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    let status_path = output_path.with_extension("status");
    let menu_path_json =
        serde_json::to_string(menu_path).map_err(|e| format!("序列化菜单路径失败: {e}"))?;
    let mut child = Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
        .arg(SHELL_MENU_HOST_ARG)
        .arg(mode)
        .arg(path.unwrap_or(""))
        .arg(menu_path_json)
        .arg(&output_path)
        .env("LINGSCAPE_SHELL_MENU_STATUS", &status_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("启动 Shell 菜单进程失败: {e}"))?;

    let timeout = if mode == "native" {
        Duration::from_secs(300)
    } else {
        SHELL_MENU_TIMEOUT
    };
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let bytes = fs::read(&output_path)
                    .map_err(|e| format!("读取 Shell 菜单结果失败: {e}"));
                let _ = fs::remove_file(&output_path);
                let _ = fs::remove_file(&status_path);
                if !status.success() {
                    return Err(format!("Shell 菜单进程异常退出: {status}"));
                }
                let result: Result<Vec<ShellMenuEntry>, String> = serde_json::from_slice(
                    &bytes?,
                )
                .map_err(|e| format!("解析 Shell 菜单结果失败: {e}"))?;
                return result;
            }
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(20));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = fs::remove_file(&output_path);
                let stage = fs::read_to_string(&status_path)
                    .unwrap_or_else(|_| "未知阶段".into());
                let _ = fs::remove_file(&status_path);
                eprintln!("[desktop-organize] shell menu timeout stage={stage}");
                return Err(format!("菜单加载超时（{stage}）"));
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = fs::remove_file(&output_path);
                let _ = fs::remove_file(&status_path);
                return Err(format!("等待 Shell 菜单进程失败: {e}"));
            }
        }
    }
}

/// Handle the isolated Shell-menu subprocess before Tauri starts.
pub fn maybe_run_shell_menu_host() -> bool {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) != Some(SHELL_MENU_HOST_ARG) {
        return false;
    }

    let result = (|| -> Result<Vec<ShellMenuEntry>, String> {
        let mode = args.get(2).ok_or_else(|| "缺少菜单模式".to_string())?;
        let path = args.get(3).ok_or_else(|| "缺少菜单路径".to_string())?;
        let menu_path: Vec<u32> = serde_json::from_str(
            args.get(4).ok_or_else(|| "缺少二级菜单路径".to_string())?,
        )
        .map_err(|e| format!("解析二级菜单路径失败: {e}"))?;
        let path = (!path.is_empty()).then_some(path.as_str());

        #[cfg(windows)]
        {
            let hwnd = crate::shell_menu::create_host_window()?;
            crate::shell_menu::pump_messages();
            let result = match mode.as_str() {
                "root" => crate::shell_menu::list_shell_context_menu(hwnd, path),
                "submenu" => {
                    crate::shell_menu::list_shell_context_submenu(hwnd, path, &menu_path)
                }
                "native" => crate::shell_menu::show_native_shell_context_menu(hwnd, path)
                    .map(|_| Vec::new()),
                _ => Err("未知菜单模式".into()),
            };
            crate::shell_menu::pump_messages();
            crate::shell_menu::destroy_host_window(hwnd);
            result
        }
        #[cfg(not(windows))]
        {
            let _ = (mode, path, menu_path);
            Err("桌面整理仅支持 Windows".into())
        }
    })();

    if let Some(output) = args.get(5) {
        if let Ok(json) = serde_json::to_vec(&result) {
            let _ = fs::write(output, json);
        }
    }
    true
}

fn push_items_to_fence(app: &AppHandle, items: &[DesktopItem]) -> Result<(), String> {
    let _ = app.emit("fence-items", items);
    let window = match app.get_webview_window(FENCE_LABEL) {
        Some(w) => w,
        None => return Ok(()),
    };
    let json = serde_json::to_string(items).map_err(|e| format!("序列化桌面项失败: {e}"))?;
    let script = format!(
        r#"(function(items,n){{function go(){{var a=document.getElementById("apps");if(a){{a.style.paddingTop="28px";a.style.paddingLeft="16px";}}if(window.__fenceApply){{window.__fenceApply(items);return;}}if(n<40){{n+=1;setTimeout(go,100);}}}}go();}})({json},0);"#
    );
    let _ = window.eval(&script);
    Ok(())
}

fn enable_inner(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window(FENCE_LABEL)
        .ok_or_else(|| "格子窗口未注册，请重启应用".to_string())?;

    #[cfg(windows)]
    {
        desktop::set_icons_visible(false);
        let hwnd = match window.hwnd() {
            Ok(h) => h,
            Err(e) => {
                desktop::set_icons_visible(true);
                return Err(format!("获取 HWND 失败: {e}"));
            }
        };
        let _ = window.set_title("");
        let _ = window.set_decorations(false);
        let _ = window.set_shadow(false);
        if let Err(e) = win::attach_fence_to_desktop(hwnd.0 as isize) {
            desktop::set_icons_visible(true);
            return Err(e);
        }
        let _ = window.set_ignore_cursor_events(false);
    }
    #[cfg(not(windows))]
    {
        return Err("桌面整理仅支持 Windows".into());
    }

    let items = scan_desktop_items()?;
    let _ = window.eval("location.reload()");
    push_items_to_fence(app, &items)?;
    ACTIVE.store(true, Ordering::SeqCst);
    start_desktop_watch(app);
    eprintln!("[desktop-organize] enabled items={}", items.len());
    Ok(())
}

fn disable_inner(app: &AppHandle) -> Result<(), String> {
    ACTIVE.store(false, Ordering::SeqCst);
    stop_desktop_watch();

    if let Some(window) = app.get_webview_window(FENCE_LABEL) {
        #[cfg(windows)]
        if let Ok(hwnd) = window.hwnd() {
            win::hide_fence_from_desktop(hwnd.0 as isize);
        }
        let _ = window.eval(
            "document.getElementById('apps').innerHTML='';document.getElementById('images').innerHTML='';document.getElementById('documents').innerHTML='';document.getElementById('folders').innerHTML='';document.getElementById('media').innerHTML='';document.getElementById('archives').innerHTML='';",
        );
        let _ = window.hide();
    }

    #[cfg(windows)]
    desktop::set_icons_visible(true);

    eprintln!("[desktop-organize] disabled");
    Ok(())
}

pub fn set_enabled(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let handle = app.clone();
    run_on_ui(app, move || {
        if enabled {
            enable_inner(&handle)
        } else {
            disable_inner(&handle)
        }
    })?
}

pub fn refresh(app: &AppHandle) -> Result<(), String> {
    if !ACTIVE.load(Ordering::SeqCst) {
        return Ok(());
    }
    let items = run_on_ui(app, scan_desktop_items)??;
    push_items_to_fence(app, &items)
}

pub fn reassert(app: &AppHandle) {
    if !ACTIVE.load(Ordering::SeqCst) {
        return;
    }
    let Some(window) = app.get_webview_window(FENCE_LABEL) else {
        return;
    };
    #[cfg(windows)]
    {
        if let Ok(hwnd) = window.hwnd() {
            let _ = win::attach_fence_to_desktop(hwnd.0 as isize);
            win::touch_fence_chrome(hwnd.0 as isize);
        }
        let _ = window.set_ignore_cursor_events(false);
    }
}

pub fn cleanup(app: &AppHandle) {
    if ACTIVE.load(Ordering::SeqCst) {
        let _ = disable_inner(app);
    }
}

#[tauri::command]
pub fn set_desktop_organize(app: AppHandle, enabled: bool) -> Result<(), String> {
    eprintln!("[desktop-organize] set_desktop_organize enabled={enabled}");
    set_enabled(&app, enabled)?;
    let mut s = settings::load_settings(&app).unwrap_or_default();
    s.desktop_organize_enabled = enabled;
    settings::persist_and_notify(&app, &s)?;
    Ok(())
}

#[tauri::command]
pub fn list_desktop_items(app: AppHandle) -> Result<Vec<DesktopItem>, String> {
    run_on_ui(&app, scan_desktop_items)?
}

#[tauri::command]
pub fn open_desktop_item(path: String) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("路径为空".into());
    }
    let p = Path::new(trimmed);
    if !trimmed.starts_with("::") && !p.exists() {
        return Err("文件不存在".into());
    }
    #[cfg(windows)]
    {
        if trimmed.starts_with("::") {
            let target = if trimmed
                .to_ascii_uppercase()
                .contains("F02C1A0D-B21F-4110-8426-0A0C959C3602")
            {
                "shell:NetworkPlacesFolder"
            } else {
                trimmed
            };
            std::process::Command::new("explorer.exe")
                .arg(target)
                .spawn()
                .map_err(|e| format!("打开系统图标失败: {e}"))?;
            return Ok(());
        }
        std::process::Command::new("cmd")
            .args(["/C", "start", "", trimmed])
            .spawn()
            .map_err(|e| format!("打开失败: {e}"))?;
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        let _ = trimmed;
        Err("桌面整理仅支持 Windows".into())
    }
}

/// List Shell COM context menu entries (custom UI; includes icons when available).
#[tauri::command]
pub async fn list_desktop_shell_context_menu(
    app: AppHandle,
    path: String,
) -> Result<Vec<ShellMenuEntry>, String> {
    let trimmed = path.trim();
    let path_opt = if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    };
    let _window = app
        .get_webview_window(FENCE_LABEL)
        .ok_or_else(|| "格子窗口未就绪".to_string())?;

    #[cfg(windows)]
    {
        return tauri::async_runtime::spawn_blocking(move || {
            run_shell_menu_host("root", path_opt.as_deref(), &[])
        })
        .await
        .map_err(|e| format!("加载右键菜单任务失败: {e}"))?;
    }
    #[cfg(not(windows))]
    {
        let _ = (_window, path_opt);
        Err("桌面整理仅支持 Windows".into())
    }
}

#[tauri::command]
pub async fn list_desktop_shell_context_submenu(
    app: AppHandle,
    path: String,
    menu_path: Vec<u32>,
) -> Result<Vec<ShellMenuEntry>, String> {
    let trimmed = path.trim();
    let path_opt = if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    };
    let _window = app
        .get_webview_window(FENCE_LABEL)
        .ok_or_else(|| "格子窗口未就绪".to_string())?;

    #[cfg(windows)]
    {
        return tauri::async_runtime::spawn_blocking(move || {
            run_shell_menu_host("submenu", path_opt.as_deref(), &menu_path)
        })
        .await
        .map_err(|e| format!("加载二级菜单任务失败: {e}"))?;
    }
    #[cfg(not(windows))]
    {
        let _ = (_window, path_opt, menu_path);
        Err("桌面整理仅支持 Windows".into())
    }
}

/// Invoke a Shell COM context menu command previously listed for path/blank desktop.
#[tauri::command]
pub async fn invoke_desktop_shell_context_command(
    app: AppHandle,
    path: String,
    command_id: u32,
    menu_path: Vec<u32>,
) -> Result<(), String> {
    let trimmed = path.trim();
    let path_opt = if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    };
    let _window = app
        .get_webview_window(FENCE_LABEL)
        .ok_or_else(|| "格子窗口未就绪".to_string())?;

    #[cfg(windows)]
    {
        return tauri::async_runtime::spawn_blocking(move || {
            let hwnd = crate::shell_menu::create_host_window()?;
            crate::shell_menu::pump_messages();
            let result = crate::shell_menu::invoke_shell_context_command(
                hwnd,
                path_opt.as_deref(),
                command_id,
                &menu_path,
            );
            crate::shell_menu::pump_messages();
            crate::shell_menu::destroy_host_window(hwnd);
            result
        })
        .await
        .map_err(|e| format!("执行菜单命令任务失败: {e}"))?;
    }
    #[cfg(not(windows))]
    {
        let _ = (_window, path_opt, command_id, menu_path);
        Err("桌面整理仅支持 Windows".into())
    }
}

/// Show the real Windows Shell menu and execute the selected command before
/// releasing its COM objects, preserving dynamic/owner-drawn menu behavior.
#[tauri::command]
pub async fn show_desktop_native_context_menu(
    app: AppHandle,
    path: String,
) -> Result<(), String> {
    let trimmed = path.trim();
    let path_opt = if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    };
    let _window = app
        .get_webview_window(FENCE_LABEL)
        .ok_or_else(|| "格子窗口未就绪".to_string())?;

    #[cfg(windows)]
    {
        return tauri::async_runtime::spawn_blocking(move || {
            run_shell_menu_host("native", path_opt.as_deref(), &[]).map(|_| ())
        })
        .await
        .map_err(|e| format!("显示原生右键菜单任务失败: {e}"))?;
    }
    #[cfg(not(windows))]
    {
        let _ = (_window, path_opt);
        Err("桌面整理仅支持 Windows".into())
    }
}

#[tauri::command]
pub fn show_desktop_item_in_folder(path: String) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("路径为空".into());
    }
    if trimmed.starts_with("::") {
        return Err("系统图标不支持此操作".into());
    }
    let p = Path::new(trimmed);
    if !p.exists() {
        return Err("文件不存在".into());
    }
    #[cfg(windows)]
    {
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", trimmed))
            .spawn()
            .map_err(|e| format!("打开文件夹失败: {e}"))?;
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        let _ = trimmed;
        Err("桌面整理仅支持 Windows".into())
    }
}

#[tauri::command]
pub fn open_desktop_item_with(path: String) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("路径为空".into());
    }
    if trimmed.starts_with("::") {
        return Err("系统图标不支持此操作".into());
    }
    let p = Path::new(trimmed);
    if !p.exists() {
        return Err("文件不存在".into());
    }
    #[cfg(windows)]
    {
        std::process::Command::new("rundll32")
            .args(["shell32.dll,OpenAs_RunDLL", trimmed])
            .spawn()
            .map_err(|e| format!("打开方式失败: {e}"))?;
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        let _ = trimmed;
        Err("桌面整理仅支持 Windows".into())
    }
}

#[tauri::command]
pub fn open_desktop_item_properties(path: String) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("路径为空".into());
    }
    if trimmed.starts_with("::") {
        return Err("系统图标不支持此操作".into());
    }
    let p = Path::new(trimmed);
    if !p.exists() {
        return Err("文件不存在".into());
    }
    #[cfg(windows)]
    {
        return win::shell_show_properties(trimmed);
    }
    #[cfg(not(windows))]
    {
        let _ = trimmed;
        Err("桌面整理仅支持 Windows".into())
    }
}

#[tauri::command]
pub fn rename_desktop_item(path: String, new_name: String) -> Result<(), String> {
    let trimmed = path.trim();
    let name = new_name.trim();
    if trimmed.is_empty() || name.is_empty() {
        return Err("路径或名称为空".into());
    }
    if trimmed.starts_with("::") {
        return Err("系统图标不支持重命名".into());
    }
    if name.contains(['\\', '/', ':', '*', '?', '"', '<', '>', '|']) {
        return Err("名称包含非法字符".into());
    }
    let old = Path::new(trimmed);
    if !old.exists() {
        return Err("文件不存在".into());
    }
    let parent = old.parent().ok_or_else(|| "无法解析父目录".to_string())?;
    let new_path = parent.join(name);
    if new_path.exists() {
        return Err("目标名称已存在".into());
    }
    fs::rename(old, &new_path).map_err(|e| format!("重命名失败: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn delete_desktop_item(path: String) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("路径为空".into());
    }
    if trimmed.starts_with("::") {
        return Err("系统图标不支持删除".into());
    }
    let p = Path::new(trimmed);
    if !p.exists() {
        return Err("文件不存在".into());
    }
    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use windows::core::{BOOL, PCWSTR};
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::Shell::{
            FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FO_DELETE, SHFILEOPSTRUCTW, SHFileOperationW,
        };
        unsafe {
            let wpath: Vec<u16> = OsStr::new(trimmed)
                .encode_wide()
                .chain(std::iter::once(0))
                .chain(std::iter::once(0))
                .collect();
            let mut op = SHFILEOPSTRUCTW {
                hwnd: HWND::default(),
                wFunc: FO_DELETE,
                pFrom: PCWSTR(wpath.as_ptr()),
                pTo: PCWSTR::null(),
                fFlags: (FOF_ALLOWUNDO.0 | FOF_NOCONFIRMATION.0) as u16,
                fAnyOperationsAborted: BOOL(0),
                hNameMappings: std::ptr::null_mut(),
                lpszProgressTitle: PCWSTR::null(),
            };
            let hr = SHFileOperationW(&mut op);
            if hr != 0 || op.fAnyOperationsAborted.as_bool() {
                return Err(format!("删除失败: code={hr}"));
            }
        }
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        let _ = trimmed;
        Err("桌面整理仅支持 Windows".into())
    }
}

fn strip_extended_path(path: PathBuf) -> PathBuf {
    let s = path.to_string_lossy();
    if let Some(rest) = s.strip_prefix(r"\\?\") {
        PathBuf::from(rest)
    } else {
        path
    }
}

fn desktop_scan_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(home) = std::env::var("USERPROFILE") {
        dirs.push(PathBuf::from(home).join("Desktop"));
    }
    let public = std::env::var("PUBLIC").unwrap_or_else(|_| r"C:\Users\Public".into());
    dirs.push(PathBuf::from(public).join("Desktop"));
    dirs
}

fn stop_desktop_watch() {
    WATCH_GEN.fetch_add(1, Ordering::SeqCst);
}

fn start_desktop_watch(app: &AppHandle) {
    stop_desktop_watch();
    let gen = WATCH_GEN.load(Ordering::SeqCst);
    let app = app.clone();
    std::thread::Builder::new()
        .name("desktop-fence-watch".into())
        .spawn(move || {
            use notify::{RecommendedWatcher, RecursiveMode, Watcher};
            use std::sync::mpsc::{RecvTimeoutError, channel};

            let (tx, rx) = channel();
            let mut watcher = match RecommendedWatcher::new(
                tx,
                notify::Config::default().with_poll_interval(Duration::from_secs(2)),
            ) {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("[desktop-organize] watcher create failed: {e}");
                    return;
                }
            };

            let mut watching = false;
            for dir in desktop_scan_dirs() {
                if !dir.is_dir() {
                    continue;
                }
                match watcher.watch(&dir, RecursiveMode::NonRecursive) {
                    Ok(()) => {
                        watching = true;
                        eprintln!("[desktop-organize] watching {}", dir.display());
                    }
                    Err(e) => eprintln!("[desktop-organize] watch {} failed: {e}", dir.display()),
                }
            }
            if !watching {
                return;
            }

            let mut pending_at: Option<Instant> = None;
            loop {
                if WATCH_GEN.load(Ordering::SeqCst) != gen {
                    break;
                }
                match rx.recv_timeout(Duration::from_millis(200)) {
                    Ok(Ok(_event)) => {
                        pending_at = Some(Instant::now());
                    }
                    Ok(Err(e)) => {
                        eprintln!("[desktop-organize] watcher error: {e}");
                        break;
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        if let Some(at) = pending_at {
                            if at.elapsed() >= Duration::from_millis(450) {
                                pending_at = None;
                                if ACTIVE.load(Ordering::SeqCst) {
                                    if let Err(e) = refresh(&app) {
                                        eprintln!("[desktop-organize] watch refresh failed: {e}");
                                    }
                                }
                            }
                        }
                    }
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }
            eprintln!("[desktop-organize] watcher stopped");
        })
        .ok();
}

fn from_base64(input: &str) -> Option<Vec<u8>> {
    const TABLE: &[u8; 256] = &{
        let mut t = [0xffu8; 256];
        let mut i = 0u8;
        while i < 26 {
            t[(b'A' + i) as usize] = i;
            t[(b'a' + i) as usize] = 26 + i;
            i += 1;
        }
        i = 0;
        while i < 10 {
            t[(b'0' + i) as usize] = 52 + i;
            i += 1;
        }
        t[b'+' as usize] = 62;
        t[b'/' as usize] = 63;
        t
    };

    let bytes: Vec<u8> = input
        .bytes()
        .filter(|b| !b.is_ascii_whitespace())
        .collect();
    if bytes.is_empty() || bytes.len() % 4 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    for chunk in bytes.chunks_exact(4) {
        let a = TABLE[chunk[0] as usize];
        let b = TABLE[chunk[1] as usize];
        let (c, pad_c) = if chunk[2] == b'=' {
            (0, true)
        } else {
            (TABLE[chunk[2] as usize], false)
        };
        let (d, pad_d) = if chunk[3] == b'=' {
            (0, true)
        } else {
            (TABLE[chunk[3] as usize], false)
        };
        if a == 0xff || b == 0xff || (!pad_c && c == 0xff) || (!pad_d && d == 0xff) {
            return None;
        }
        out.push((a << 2) | (b >> 4));
        if !pad_c {
            out.push((b << 4) | (c >> 2));
        }
        if !pad_d {
            out.push((c << 6) | d);
        }
    }
    Some(out)
}

fn decode_image_data_url(url: &str) -> Option<Vec<u8>> {
    let url = url.trim();
    let b64 = url
        .strip_prefix("data:image/png;base64,")
        .or_else(|| url.strip_prefix("data:image/PNG;base64,"))?;
    from_base64(b64)
}

fn parse_drag_mode(mode: Option<&str>) -> drag::DragMode {
    match mode.map(|s| s.trim().to_ascii_lowercase()).as_deref() {
        Some("move") => drag::DragMode::Move,
        _ => drag::DragMode::Copy,
    }
}

fn drag_preview_png(path: &Path, preview_data_url: Option<&str>) -> Vec<u8> {
    if let Some(url) = preview_data_url {
        if let Some(bytes) = decode_image_data_url(url) {
            return bytes;
        }
    }
    #[cfg(windows)]
    {
        let is_image = {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            matches!(
                ext.as_str(),
                "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" | "tif" | "tiff" | "jfif"
            )
        };
        let url = if is_image {
            win::image_file_preview(path, 96)
        } else {
            win::shell_name_and_icon(path).icon
        };
        if let Some(url) = url {
            if let Some(bytes) = decode_image_data_url(&url) {
                return bytes;
            }
        }
    }
    MINI_DRAG_PNG.to_vec()
}

#[cfg(windows)]
fn window_class_name(hwnd: windows::Win32::Foundation::HWND) -> String {
    use windows::Win32::UI::WindowsAndMessaging::GetClassNameW;
    unsafe {
        let mut buf = [0u16; 256];
        let n = GetClassNameW(hwnd, &mut buf);
        if n <= 0 {
            return String::new();
        }
        String::from_utf16_lossy(&buf[..n as usize])
    }
}

#[cfg(windows)]
fn is_desktop_shell_class(class: &str) -> bool {
    matches!(
        class,
        "Progman" | "WorkerW" | "SHELLDLL_DefView" | "SysListView32"
    )
}

#[cfg(windows)]
fn is_lbutton_down() -> bool {
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
    unsafe { (GetAsyncKeyState(VK_LBUTTON.0 as i32) as u16 & 0x8000) != 0 }
}

#[cfg(windows)]
fn collect_own_hwnds(app: &AppHandle, fence_hwnd: isize) -> Vec<isize> {
    let mut own = vec![fence_hwnd];
    for label in [FENCE_LABEL, "wallpaper"] {
        if let Some(w) = app.get_webview_window(label) {
            if let Ok(h) = w.hwnd() {
                let v = h.0 as isize;
                if !own.contains(&v) {
                    own.push(v);
                }
            }
        }
    }
    own
}

#[cfg(windows)]
fn is_cursor_over_foreign_window(app: &AppHandle, fence_hwnd: isize) -> bool {
    use windows::Win32::Foundation::{HWND, POINT};
    use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetParent, WindowFromPoint};
    unsafe {
        let fence = HWND(fence_hwnd as *mut _);
        if fence.0.is_null() {
            return false;
        }
        let own = collect_own_hwnds(app, fence_hwnd);
        let mut pt = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut pt).is_err() {
            return false;
        }
        let mut hwnd = WindowFromPoint(pt);
        // Fence is a WS_CHILD of the desktop DefView — walk parents instead of GA_ROOT.
        for _ in 0..24 {
            if hwnd.0.is_null() {
                return true;
            }
            let id = hwnd.0 as isize;
            if own.contains(&id) {
                return false;
            }
            let class = window_class_name(hwnd);
            if is_desktop_shell_class(&class) {
                return false;
            }
            hwnd = GetParent(hwnd).unwrap_or_default();
        }
        true
    }
}

#[tauri::command]
pub fn is_desktop_drag_over_foreign(app: AppHandle) -> Result<bool, String> {
    #[cfg(windows)]
    {
        let window = app
            .get_webview_window(FENCE_LABEL)
            .ok_or_else(|| "格子窗口未就绪".to_string())?;
        let hwnd = window.hwnd().map_err(|e| e.to_string())?.0 as isize;
        Ok(is_cursor_over_foreign_window(&app, hwnd))
    }
    #[cfg(not(windows))]
    {
        let _ = app;
        Ok(false)
    }
}

/// Start a system shell file drag (CF_HDROP) so icons can be dropped into other apps.
/// `mode`: "copy" (default) or "move". Hold Shift in the UI to request move.
#[tauri::command]
pub fn start_desktop_file_drag(
    app: AppHandle,
    path: String,
    mode: Option<String>,
    preview_data_url: Option<String>,
) -> Result<(), String> {
    let trimmed = path.trim().to_string();
    if trimmed.is_empty() {
        return Err("路径为空".into());
    }
    if trimmed.starts_with("::") {
        return Err("系统图标不支持拖出到其他程序".into());
    }
    let path_buf = PathBuf::from(&trimmed);
    if !path_buf.exists() {
        return Err("文件不存在".into());
    }
    let abs = strip_extended_path(std::fs::canonicalize(&path_buf).unwrap_or(path_buf));
    let drag_mode = parse_drag_mode(mode.as_deref());
    let preview = drag_preview_png(&abs, preview_data_url.as_deref());

    let window = app
        .get_webview_window(FENCE_LABEL)
        .ok_or_else(|| "格子窗口未就绪".to_string())?;

    #[cfg(windows)]
    {
        if !is_lbutton_down() {
            return Err("鼠标已松开，取消拖出".into());
        }
        let handle = app.clone();
        let win = window.clone();
        return run_on_ui(&handle, move || {
            if !is_lbutton_down() {
                return Err("鼠标已松开，取消拖出".into());
            }
            let item = drag::DragItem::Files(vec![abs]);
            let preview = drag::Image::Raw(preview);
            let opts = drag::Options {
                mode: drag_mode,
                skip_animatation_on_cancel_or_failure: true,
            };
            drag::start_drag(&win, item, preview, |_result, _pos| {}, opts)
                .map_err(|e| format!("启动文件拖放失败: {e}"))?;
            eprintln!("[desktop-organize] shell file drag finished path={trimmed}");
            Ok(())
        })?;
    }
    #[cfg(not(windows))]
    {
        let _ = (window, abs, drag_mode, preview);
        Err("桌面整理拖出仅支持 Windows".into())
    }
}

/// Tiny valid PNG (1x1 transparent) used as drag preview when no icon file is handy.
const MINI_DRAG_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];
