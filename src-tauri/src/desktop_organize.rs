use serde::Serialize;
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

#[derive(Debug, Clone, Serialize)]
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
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
    use windows_sys::Win32::Graphics::Gdi::{
        EnumDisplayMonitors, GetMonitorInfoW, RedrawWindow, HDC, HMONITOR, MONITORINFO,
        RDW_ALLCHILDREN, RDW_INVALIDATE, RDW_UPDATENOW,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallWindowProcW, EnumWindows, FindWindowExW, FindWindowW, GetParent, GetSystemMetrics,
        GetWindowLongPtrW, SetLayeredWindowAttributes, SetParent, SetWindowLongPtrW, SetWindowPos,
        SetWindowTextW, ShowWindow, GWL_EXSTYLE, GWL_STYLE, GWLP_WNDPROC, HICON, HTCLIENT, HWND_TOP,
        LWA_ALPHA, MONITORINFOF_PRIMARY, SM_CXSCREEN, SM_CYSCREEN, STYLESTRUCT, SW_HIDE,
        SWP_FRAMECHANGED, SWP_HIDEWINDOW, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
        SWP_SHOWWINDOW, SW_SHOW, WM_NCCALCSIZE, WM_NCHITTEST, WM_NCPAINT, WM_SETTEXT,
        WM_STYLECHANGED, WM_STYLECHANGING, WS_BORDER, WS_CAPTION, WS_CHILD, WS_CLIPCHILDREN,
        WS_CLIPSIBLINGS, WS_DLGFRAME, WS_EX_APPWINDOW, WS_EX_CLIENTEDGE, WS_EX_DLGMODALFRAME,
        WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_STATICEDGE, WS_EX_TOOLWINDOW, WS_EX_WINDOWEDGE,
        WS_POPUP, WS_SYSMENU, WS_THICKFRAME, WS_VISIBLE,
    };

    static ORIG_WNDPROC: AtomicIsize = AtomicIsize::new(0);
    static FENCE_SHOWN: AtomicBool = AtomicBool::new(false);
    static DESKTOP_DEFVIEW: AtomicIsize = AtomicIsize::new(0);

    thread_local! {
        static CTX_MENU_FWD: std::cell::RefCell<Option<CtxMenuFwd>> =
            std::cell::RefCell::new(None);
    }

    struct CtxMenuFwd {
        pcm2: *mut core::ffi::c_void,
        pcm2_handle_menu_msg: Option<
            unsafe extern "system" fn(*mut core::ffi::c_void, u32, WPARAM, LPARAM) -> i32,
        >,
        pcm3: *mut core::ffi::c_void,
        pcm3_handle_menu_msg2: Option<
            unsafe extern "system" fn(
                *mut core::ffi::c_void,
                u32,
                WPARAM,
                LPARAM,
                *mut LRESULT,
            ) -> i32,
        >,
    }

    impl Clone for CtxMenuFwd {
        fn clone(&self) -> Self {
            Self {
                pcm2: self.pcm2,
                pcm2_handle_menu_msg: self.pcm2_handle_menu_msg,
                pcm3: self.pcm3,
                pcm3_handle_menu_msg2: self.pcm3_handle_menu_msg2,
            }
        }
    }

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
        use std::os::windows::ffi::OsStrExt;
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
        hbmp: windows_sys::Win32::Graphics::Gdi::HBITMAP,
    ) -> Option<String> {
        use windows_sys::Win32::Graphics::Gdi::{
            CreateCompatibleDC, DeleteDC, GetDIBits, GetObjectW, BITMAP, BITMAPINFO,
            BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
        };

        let mut bm: BITMAP = std::mem::zeroed();
        if GetObjectW(
            hbmp,
            std::mem::size_of::<BITMAP>() as i32,
            &mut bm as *mut BITMAP as *mut core::ffi::c_void,
        ) == 0
            || bm.bmWidth <= 0
            || bm.bmHeight == 0
        {
            return None;
        }
        let w = bm.bmWidth;
        let h = bm.bmHeight.abs();
        let hdc = CreateCompatibleDC(std::ptr::null_mut());
        if hdc.is_null() {
            return None;
        }
        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmiHeader = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w,
            biHeight: -h,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        };
        let mut bits = vec![0u8; (w * h * 4) as usize];
        let got = GetDIBits(
            hdc,
            hbmp,
            0,
            h as u32,
            bits.as_mut_ptr() as *mut core::ffi::c_void,
            &mut bmi,
            DIB_RGB_COLORS,
        );
        DeleteDC(hdc);
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
        flags: i32,
    ) -> Option<String> {
        use windows_sys::Win32::Foundation::SIZE;
        use windows_sys::Win32::Graphics::Gdi::{DeleteObject, HBITMAP};
        use windows_sys::Win32::UI::Shell::SHCreateItemFromParsingName;

        #[repr(C)]
        struct FactoryVtbl {
            query_interface: unsafe extern "system" fn(
                *mut core::ffi::c_void,
                *const windows_sys::core::GUID,
                *mut *mut core::ffi::c_void,
            ) -> i32,
            add_ref: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
            release: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
            get_image: unsafe extern "system" fn(
                *mut core::ffi::c_void,
                SIZE,
                u32,
                *mut HBITMAP,
            ) -> i32,
        }

        const IID: windows_sys::core::GUID = windows_sys::core::GUID {
            data1: 0xbcc18b79,
            data2: 0xba16,
            data3: 0x442f,
            data4: [0x80, 0xc4, 0x8a, 0x59, 0xc3, 0x0c, 0x46, 0x3b],
        };

        let wpath = wide_path(path);
        let mut obj: *mut core::ffi::c_void = std::ptr::null_mut();
        let hr = SHCreateItemFromParsingName(
            wpath.as_ptr(),
            std::ptr::null_mut(),
            &IID,
            &mut obj,
        );
        if hr < 0 || obj.is_null() {
            return None;
        }
        let vtbl = *(obj as *mut *const FactoryVtbl);
        if vtbl.is_null() {
            return None;
        }
        let mut hbmp: HBITMAP = std::ptr::null_mut();
        let img_hr = ((*vtbl).get_image)(
            obj,
            SIZE { cx: px, cy: px },
            flags as u32,
            &mut hbmp,
        );
        ((*vtbl).release)(obj);
        if img_hr < 0 || hbmp.is_null() {
            return None;
        }
        let url = hbitmap_to_png_data_url(hbmp);
        DeleteObject(hbmp);
        url
    }

    pub fn image_file_preview(path: &std::path::Path, px: i32) -> Option<String> {
        use windows_sys::Win32::UI::Shell::{
            SIIGBF_BIGGERSIZEOK, SIIGBF_SCALEUP, SIIGBF_THUMBNAILONLY,
        };

        unsafe {
            shell_item_image_png(
                path,
                px,
                SIIGBF_THUMBNAILONLY | SIIGBF_BIGGERSIZEOK | SIIGBF_SCALEUP,
            )
            .or_else(|| shell_item_image_png(path, px, SIIGBF_BIGGERSIZEOK | SIIGBF_SCALEUP))
        }
    }

    unsafe fn hicon_native_size(hicon: HICON) -> i32 {
        use windows_sys::Win32::Graphics::Gdi::{DeleteObject, GetObjectW, BITMAP, HGDIOBJ};
        use windows_sys::Win32::UI::WindowsAndMessaging::{GetIconInfo, ICONINFO};

        let mut info = ICONINFO {
            fIcon: 0,
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: std::ptr::null_mut(),
            hbmColor: std::ptr::null_mut(),
        };
        if GetIconInfo(hicon, &mut info) == 0 {
            return 256;
        }
        let mut bm = BITMAP {
            bmType: 0,
            bmWidth: 0,
            bmHeight: 0,
            bmWidthBytes: 0,
            bmPlanes: 0,
            bmBitsPixel: 0,
            bmBits: std::ptr::null_mut(),
        };
        let hbmp: HGDIOBJ = if info.hbmColor.is_null() {
            info.hbmMask
        } else {
            info.hbmColor
        };
        GetObjectW(
            hbmp,
            std::mem::size_of::<BITMAP>() as i32,
            &mut bm as *mut BITMAP as *mut core::ffi::c_void,
        );
        if !info.hbmColor.is_null() {
            DeleteObject(info.hbmColor);
        }
        if !info.hbmMask.is_null() {
            DeleteObject(info.hbmMask);
        }
        bm.bmWidth.clamp(16, 256)
    }

    unsafe fn hicon_to_png_data_url(hicon: HICON) -> Option<String> {
        use windows_sys::Win32::Graphics::Gdi::{
            CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, SelectObject, BITMAPINFO,
            BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HGDIOBJ,
        };
        use windows_sys::Win32::UI::WindowsAndMessaging::{DrawIconEx, DI_NORMAL};

        if hicon.is_null() {
            return None;
        }
        let size = hicon_native_size(hicon);
        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmiHeader = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: size,
            biHeight: -size,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        };
        let hdc = CreateCompatibleDC(std::ptr::null_mut());
        if hdc.is_null() {
            return None;
        }
        let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
        let dib = CreateDIBSection(
            hdc,
            &bmi,
            DIB_RGB_COLORS,
            &mut bits,
            std::ptr::null_mut(),
            0,
        );
        if dib.is_null() || bits.is_null() {
            DeleteDC(hdc);
            return None;
        }
        let old = SelectObject(hdc, dib);
        let pixel_count = (size * size) as usize;
        std::ptr::write_bytes(bits, 0, pixel_count * 4);
        DrawIconEx(
            hdc,
            0,
            0,
            hicon,
            size,
            size,
            0,
            std::ptr::null_mut(),
            DI_NORMAL,
        );

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

        SelectObject(hdc, old as HGDIOBJ);
        DeleteObject(dib);
        DeleteDC(hdc);
        rgba_to_png_data_url(&rgba, size as u32, size as u32)
    }

    unsafe fn image_list_icon(list_id: u32, index: i32) -> Option<HICON> {
        use windows_sys::Win32::UI::Controls::{ImageList_GetIcon, HIMAGELIST, ILD_TRANSPARENT};
        use windows_sys::Win32::UI::Shell::SHGetImageList;

        const IID_IIMAGELIST: windows_sys::core::GUID = windows_sys::core::GUID {
            data1: 0x46eb5926,
            data2: 0x582e,
            data3: 0x4017,
            data4: [0x9f, 0xdf, 0xe8, 0x99, 0x8d, 0xaa, 0x09, 0x50],
        };

        let mut list: *mut core::ffi::c_void = std::ptr::null_mut();
        let hr = SHGetImageList(list_id as i32, &IID_IIMAGELIST, &mut list);
        if hr < 0 || list.is_null() {
            return None;
        }
        let icon = ImageList_GetIcon(list as HIMAGELIST, index, ILD_TRANSPARENT);
        if icon.is_null() {
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
        hmodule: windows_sys::Win32::Foundation::HMODULE,
        _lptype: windows_sys::core::PCWSTR,
        lpname: windows_sys::core::PCWSTR,
        lparam: isize,
    ) -> i32 {
        use windows_sys::Win32::System::LibraryLoader::{
            FindResourceW, LoadResource, LockResource, SizeofResource,
        };
        use windows_sys::Win32::UI::WindowsAndMessaging::RT_GROUP_ICON;

        let groups = &mut *(lparam as *mut Vec<(i32, Vec<i32>)>);
        let id = {
            let p = lpname as usize;
            if p < 0x10000 {
                p as i32
            } else {
                0
            }
        };
        let hrsrc = FindResourceW(hmodule, lpname, RT_GROUP_ICON);
        if hrsrc.is_null() {
            return 1;
        }
        let size = SizeofResource(hmodule, hrsrc) as usize;
        let hdata = LoadResource(hmodule, hrsrc);
        if hdata.is_null() {
            return 1;
        }
        let ptr = LockResource(hdata) as *const u8;
        if ptr.is_null() || size < 6 {
            return 1;
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
        1
    }

    unsafe fn pe_icon_sizes(path: &std::path::Path, index: i32) -> Vec<i32> {
        use windows_sys::Win32::Foundation::FreeLibrary;
        use windows_sys::Win32::System::LibraryLoader::{
            EnumResourceNamesW, LoadLibraryExW, LOAD_LIBRARY_AS_DATAFILE,
            LOAD_LIBRARY_AS_IMAGE_RESOURCE,
        };
        use windows_sys::Win32::UI::WindowsAndMessaging::RT_GROUP_ICON;

        let wpath = wide_path(path);
        let module = LoadLibraryExW(
            wpath.as_ptr(),
            std::ptr::null_mut(),
            LOAD_LIBRARY_AS_DATAFILE | LOAD_LIBRARY_AS_IMAGE_RESOURCE,
        );
        if module.is_null() {
            return Vec::new();
        }
        let mut groups: Vec<(i32, Vec<i32>)> = Vec::new();
        EnumResourceNamesW(
            module,
            RT_GROUP_ICON,
            Some(enum_group_icons),
            &mut groups as *mut _ as isize,
        );
        FreeLibrary(module);
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

    unsafe fn extract_icon_at(path: &std::path::Path, index: i32, size: i32) -> Option<HICON> {
        use windows_sys::Win32::UI::Shell::SHDefExtractIconW;
        use windows_sys::Win32::UI::WindowsAndMessaging::{DestroyIcon, PrivateExtractIconsW};

        let wpath = wide_path(path);
        let mut icon: HICON = std::ptr::null_mut();
        let mut icon_id: u32 = 0;
        let got = PrivateExtractIconsW(
            wpath.as_ptr(),
            index,
            size,
            size,
            &mut icon,
            &mut icon_id,
            1,
            0,
        );
        if got > 0 && !icon.is_null() {
            let native = hicon_native_size(icon);
            if (native - size).abs() <= 8 {
                return Some(icon);
            }
            DestroyIcon(icon);
            return None;
        }
        if size > 48 {
            return None;
        }
        let mut large = std::ptr::null_mut();
        let mut small = std::ptr::null_mut();
        let nsize = (size as u32) | ((size as u32) << 16);
        let hr = SHDefExtractIconW(wpath.as_ptr(), index, 0, &mut large, &mut small, nsize);
        if !small.is_null() {
            DestroyIcon(small);
        }
        if hr >= 0 && !large.is_null() {
            Some(large)
        } else {
            if !large.is_null() {
                DestroyIcon(large);
            }
            None
        }
    }

    unsafe fn assoc_default_icon(path: &std::path::Path) -> Option<(std::path::PathBuf, i32)> {
        use windows_sys::Win32::UI::Shell::{
            AssocQueryStringW, ASSOCF_INIT_DEFAULTTOSTAR, ASSOCF_NOTRUNCATE, ASSOCSTR_DEFAULTICON,
        };

        let ext = path.extension()?.to_str()?;
        let assoc = wide(&format!(".{}", ext.to_ascii_lowercase()));
        let mut len: u32 = 0;
        let _ = AssocQueryStringW(
            ASSOCF_INIT_DEFAULTTOSTAR | ASSOCF_NOTRUNCATE,
            ASSOCSTR_DEFAULTICON,
            assoc.as_ptr(),
            std::ptr::null(),
            std::ptr::null_mut(),
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
            assoc.as_ptr(),
            std::ptr::null(),
            buf.as_mut_ptr(),
            &mut written,
        );
        if hr < 0 {
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

        use windows_sys::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICONLOCATION};

        let wpath = wide_path(path);
        let mut info: SHFILEINFOW = std::mem::zeroed();
        let ok = SHGetFileInfoW(
            wpath.as_ptr(),
            0,
            &mut info,
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
        use windows_sys::Win32::UI::Shell::{SHIL_EXTRALARGE, SHIL_LARGE};

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
        use windows_sys::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
        use windows_sys::Win32::UI::Shell::{
            SHGetFileInfoW, SHFILEINFOW, SHGFI_DISPLAYNAME, SHGFI_ICON, SHGFI_LARGEICON,
            SHGFI_SYSICONINDEX,
        };
        use windows_sys::Win32::UI::WindowsAndMessaging::DestroyIcon;

        unsafe {
            let _ = CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED as u32);
            let wpath = wide_path(path);
            let mut info: SHFILEINFOW = std::mem::zeroed();
            let flags = SHGFI_SYSICONINDEX | SHGFI_DISPLAYNAME | SHGFI_ICON | SHGFI_LARGEICON;
            let ok = SHGetFileInfoW(
                wpath.as_ptr(),
                0,
                &mut info,
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
                use windows_sys::Win32::UI::Shell::{SIIGBF_BIGGERSIZEOK, SIIGBF_ICONONLY};
                icon = shell_item_image_png(path, 48, SIIGBF_ICONONLY | SIIGBF_BIGGERSIZEOK);
                if icon.is_none() {
                    if let Some(best) = extract_best_icon(path, info.iIcon) {
                        icon = hicon_to_png_data_url(best);
                        DestroyIcon(best);
                    }
                }
            }
            if icon.is_none() && ok != 0 && !info.hIcon.is_null() {
                icon = hicon_to_png_data_url(info.hIcon);
            }
            if !info.hIcon.is_null() {
                DestroyIcon(info.hIcon);
            }

            ShellMeta { display_name, icon }
        }
    }

    unsafe fn stock_icon_png(siid: i32) -> Option<String> {
        use windows_sys::Win32::UI::Shell::{
            SHGetStockIconInfo, SHGSI_ICON, SHGSI_LARGEICON, SHSTOCKICONINFO,
        };
        use windows_sys::Win32::UI::WindowsAndMessaging::DestroyIcon;

        let mut info = SHSTOCKICONINFO {
            cbSize: std::mem::size_of::<SHSTOCKICONINFO>() as u32,
            ..Default::default()
        };
        let hr = SHGetStockIconInfo(siid, SHGSI_ICON | SHGSI_LARGEICON, &mut info);
        if hr < 0 || info.hIcon.is_null() {
            return None;
        }
        let url = hicon_to_png_data_url(info.hIcon);
        DestroyIcon(info.hIcon);
        url
    }

    fn extract_builtin_icon(path: &str, stock_fallback: Option<i32>) -> Option<String> {
        use windows_sys::Win32::UI::Shell::{SIIGBF_BIGGERSIZEOK, SIIGBF_ICONONLY};

        let path_obj = std::path::PathBuf::from(path);
        let flags = SIIGBF_ICONONLY | SIIGBF_BIGGERSIZEOK;
        unsafe {
            if let Some(icon) = shell_item_image_png(&path_obj, 48, flags) {
                return Some(icon);
            }
            if path.contains("F02C1A0D-B21F-4110-8426-0A0C959C3602") {
                for alt in ["shell:NetworkPlacesFolder", "::{F02C1A0D-B21F-4110-8426-0A0C959C3602}\\"] {
                    if let Some(icon) = shell_item_image_png(std::path::Path::new(alt), 48, flags) {
                        return Some(icon);
                    }
                }
                if let Some(icon) = stock_icon_png(
                    windows_sys::Win32::UI::Shell::SIID_MYNETWORK,
                ) {
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

    const BUILTIN_DESKTOP_ICONS: &[(&str, &str, Option<i32>)] = &[
        ("::{20D04FE0-3AEA-1069-A2D8-08002B30309D}", "此电脑", None),
        (
            "::{645FF040-5081-101B-9F08-00AA002F954E}",
            "回收站",
            Some(windows_sys::Win32::UI::Shell::SIID_RECYCLER),
        ),
        (
            "::{F02C1A0D-B21F-4110-8426-0A0C959C3602}",
            "网络",
            Some(windows_sys::Win32::UI::Shell::SIID_MYNETWORK),
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
            let mut r = RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            windows_sys::Win32::UI::WindowsAndMessaging::GetWindowRect(hwnd, &mut r);
            (r.left, r.top, r.right - r.left, r.bottom - r.top)
        }
    }

    struct EnumData {
        defview: HWND,
        defview_parent: HWND,
    }

    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> i32 {
        let data = &mut *(lparam as *mut EnumData);
        let class_def = wide("SHELLDLL_DefView");
        let def = FindWindowExW(
            hwnd,
            std::ptr::null_mut(),
            class_def.as_ptr(),
            std::ptr::null(),
        );
        if !def.is_null() {
            data.defview = def;
            data.defview_parent = hwnd;
        }
        1
    }

    fn find_progman_child(progman: HWND, class: &str) -> HWND {
        unsafe {
            let cls = wide(class);
            FindWindowExW(
                progman,
                std::ptr::null_mut(),
                cls.as_ptr(),
                std::ptr::null(),
            )
        }
    }

    unsafe extern "system" fn enum_monitors_proc(
        hmon: HMONITOR,
        _hdc: HDC,
        _lprc: *mut RECT,
        lparam: LPARAM,
    ) -> i32 {
        let found = &mut *(lparam as *mut Option<(i32, i32, i32, i32)>);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if GetMonitorInfoW(hmon, &mut info) != 0 && info.dwFlags & MONITORINFOF_PRIMARY != 0 {
            let r = info.rcWork;
            *found = Some((r.left, r.top, r.right - r.left, r.bottom - r.top));
            return 0;
        }
        1
    }

    fn primary_work_rect() -> (i32, i32, i32, i32) {
        let mut found: Option<(i32, i32, i32, i32)> = None;
        unsafe {
            EnumDisplayMonitors(
                std::ptr::null_mut(),
                std::ptr::null(),
                Some(enum_monitors_proc),
                &mut found as *mut _ as LPARAM,
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
            style &= !(WS_POPUP | WS_CAPTION | WS_THICKFRAME | WS_BORDER | WS_DLGFRAME | WS_SYSMENU);
            style |= WS_CHILD | WS_CLIPSIBLINGS | WS_CLIPCHILDREN;
            if visible {
                style |= WS_VISIBLE;
            } else {
                style &= !WS_VISIBLE;
            }
            SetWindowLongPtrW(child, GWL_STYLE, style as isize);

            let mut ex = GetWindowLongPtrW(child, GWL_EXSTYLE) as u32;
            ex &= !(WS_EX_APPWINDOW
                | WS_EX_CLIENTEDGE
                | WS_EX_WINDOWEDGE
                | WS_EX_DLGMODALFRAME
                | WS_EX_STATICEDGE);
            ex |= WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_LAYERED;
            SetWindowLongPtrW(child, GWL_EXSTYLE, ex as isize);
            SetLayeredWindowAttributes(child, 0, 255, LWA_ALPHA);
            SetWindowTextW(child, [0u16].as_ptr());
        }
    }

    unsafe extern "system" fn fence_wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if msg == WM_SETTEXT {
            return 1;
        }
        if msg == WM_NCCALCSIZE || msg == WM_NCPAINT {
            return 0;
        }
        if msg == WM_NCHITTEST {
            return HTCLIENT as LRESULT;
        }
        if msg == WM_STYLECHANGING && lparam != 0 {
            let ss = &mut *(lparam as *mut STYLESTRUCT);
            if wparam as isize == GWL_STYLE as isize {
                ss.styleNew &=
                    !(WS_POPUP | WS_CAPTION | WS_THICKFRAME | WS_BORDER | WS_DLGFRAME | WS_SYSMENU);
                ss.styleNew |= WS_CHILD | WS_CLIPSIBLINGS | WS_CLIPCHILDREN;
                if FENCE_SHOWN.load(Ordering::SeqCst) {
                    ss.styleNew |= WS_VISIBLE;
                } else {
                    ss.styleNew &= !WS_VISIBLE;
                }
            }
            if wparam as isize == GWL_EXSTYLE as isize {
                ss.styleNew &= !(WS_EX_APPWINDOW
                    | WS_EX_CLIENTEDGE
                    | WS_EX_WINDOWEDGE
                    | WS_EX_DLGMODALFRAME
                    | WS_EX_STATICEDGE);
                ss.styleNew |= WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_LAYERED;
            }
        }
        if msg == WM_STYLECHANGED && FENCE_SHOWN.load(Ordering::SeqCst) {
            let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
            if style & (WS_POPUP | WS_CAPTION) != 0 || style & WS_CHILD == 0 {
                force_child_chrome(hwnd, true);
            }
        }
        if let Ok(fwd) = CTX_MENU_FWD.try_with(|c| c.borrow().clone()) {
            if let Some(fwd) = fwd {
                const WM_INITMENUPOPUP: u32 = 279;
                const WM_MEASUREITEM: u32 = 44;
                const WM_DRAWITEM: u32 = 43;
                const WM_MENUCHAR: u32 = 288;
                match msg {
                    WM_INITMENUPOPUP | WM_MEASUREITEM | WM_DRAWITEM => {
                        if let Some(handle) = fwd.pcm2_handle_menu_msg {
                            if handle(fwd.pcm2, msg, wparam, lparam) == 0 {
                                return 0;
                            }
                        }
                    }
                    WM_MENUCHAR => {
                        if let Some(handle) = fwd.pcm3_handle_menu_msg2 {
                            let mut lres: LRESULT = 0;
                            if handle(fwd.pcm3, msg, wparam, lparam, &mut lres) == 0 {
                                return lres;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        let orig = ORIG_WNDPROC.load(Ordering::SeqCst);
        if orig == 0 {
            return 0;
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
            use windows_sys::Win32::Graphics::Dwm::{
                DwmExtendFrameIntoClientArea, DwmSetWindowAttribute, DWMNCRP_DISABLED,
                DWMWA_BORDER_COLOR, DWMWA_CAPTION_COLOR, DWMWA_COLOR_NONE, DWMWA_NCRENDERING_POLICY,
            };
            use windows_sys::Win32::UI::Controls::MARGINS;

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
                DWMWA_BORDER_COLOR as u32,
                &none as *const _ as *const core::ffi::c_void,
                4,
            );
            let _ = DwmSetWindowAttribute(
                child,
                DWMWA_CAPTION_COLOR as u32,
                &none as *const _ as *const core::ffi::c_void,
                4,
            );
            let policy = DWMNCRP_DISABLED;
            let _ = DwmSetWindowAttribute(
                child,
                DWMWA_NCRENDERING_POLICY as u32,
                &policy as *const _ as *const core::ffi::c_void,
                4,
            );
        }
    }

    pub fn attach_fence_to_desktop(hwnd_raw: isize) -> Result<(i32, i32), String> {
        unsafe {
            let progman_class = wide("Progman");
            let progman = FindWindowW(progman_class.as_ptr(), std::ptr::null());
            if progman.is_null() {
                return Err("未找到 Progman 窗口".into());
            }

            let mut data = EnumData {
                defview: std::ptr::null_mut(),
                defview_parent: std::ptr::null_mut(),
            };
            EnumWindows(Some(enum_windows_proc), &mut data as *mut _ as LPARAM);

            if data.defview.is_null() {
                data.defview = find_progman_child(progman, "SHELLDLL_DefView");
                if !data.defview.is_null() {
                    data.defview_parent = progman;
                }
            }
            if !data.defview.is_null() {
                DESKTOP_DEFVIEW.store(data.defview as isize, Ordering::SeqCst);
            }

            let child = hwnd_raw as HWND;
            FENCE_SHOWN.store(true, Ordering::SeqCst);
            force_child_chrome(child, true);
            prepare_styles(child);

            let parent = if !data.defview.is_null() {
                data.defview
            } else {
                progman
            };

            let prev = SetParent(child, parent);
            if prev.is_null() && GetParent(child) != parent {
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

            SetWindowPos(
                child,
                HWND_TOP,
                x,
                y,
                mw,
                mh,
                SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_FRAMECHANGED,
            );
            force_child_chrome(child, true);
            ShowWindow(child, SW_SHOW);
            RedrawWindow(
                child,
                std::ptr::null(),
                std::ptr::null_mut(),
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
            let child = hwnd_raw as HWND;
            let style = GetWindowLongPtrW(child, GWL_STYLE) as u32;
            if style & (WS_POPUP | WS_CAPTION) != 0 || style & WS_CHILD == 0 {
                force_child_chrome(child, true);
            }
            SetWindowTextW(child, [0u16].as_ptr());
        }
    }

    pub fn hide_fence_from_desktop(hwnd_raw: isize) {
        unsafe {
            let child = hwnd_raw as HWND;
            FENCE_SHOWN.store(false, Ordering::SeqCst);
            DESKTOP_DEFVIEW.store(0, Ordering::SeqCst);
            force_child_chrome(child, false);
            ShowWindow(child, SW_HIDE);
            SetParent(child, std::ptr::null_mut());
            SetWindowPos(
                child,
                std::ptr::null_mut(),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_HIDEWINDOW | SWP_FRAMECHANGED,
            );

            let progman_class = wide("Progman");
            let progman = FindWindowW(progman_class.as_ptr(), std::ptr::null());
            if progman.is_null() {
                return;
            }
            let defview = find_progman_child(progman, "SHELLDLL_DefView");
            let refresh = if !defview.is_null() { defview } else { progman };
            RedrawWindow(
                refresh,
                std::ptr::null(),
                std::ptr::null_mut(),
                RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN,
            );
        }
    }

    pub fn shell_show_properties(path: &str) -> Result<(), String> {
        use windows_sys::Win32::UI::Shell::ShellExecuteW;
        use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOW;
        unsafe {
            let wpath = wide(path);
            let verb = wide("properties");
            let ret = ShellExecuteW(
                std::ptr::null_mut(),
                verb.as_ptr(),
                wpath.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                SW_SHOW,
            );
            if ret as isize <= 32 {
                return Err(format!("打开属性失败: code={}", ret as isize));
            }
        }
        Ok(())
    }

    fn desktop_defview_hwnd() -> Option<HWND> {
        let stored = DESKTOP_DEFVIEW.load(Ordering::SeqCst);
        if stored != 0 {
            return Some(stored as HWND);
        }
        unsafe {
            let progman_class = wide("Progman");
            let progman = FindWindowW(progman_class.as_ptr(), std::ptr::null());
            if progman.is_null() {
                return None;
            }
            let def = find_progman_child(progman, "SHELLDLL_DefView");
            if def.is_null() {
                return None;
            }
            DESKTOP_DEFVIEW.store(def as isize, Ordering::SeqCst);
            Some(def)
        }
    }

    fn menu_flags() -> u32 {
        use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_SHIFT};
        use windows_sys::Win32::UI::Shell::{CMF_EXPLORE, CMF_EXTENDEDVERBS, CMF_NORMAL};
        let mut flags = CMF_NORMAL | CMF_EXPLORE;
        unsafe {
            if (GetAsyncKeyState(VK_SHIFT as i32) as u16 & 0x8000) != 0 {
                flags |= CMF_EXTENDEDVERBS;
            }
        }
        flags
    }

    fn clean_menu_label(raw: &str) -> String {
        let s = raw.split('\t').next().unwrap_or(raw);
        let mut out = String::with_capacity(s.len());
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '&' {
                if chars.peek() == Some(&'&') {
                    out.push('&');
                    chars.next();
                }
                continue;
            }
            out.push(c);
        }
        out.trim().to_string()
    }

    unsafe fn hbitmap_to_data_url(
        hbmp: windows_sys::Win32::Graphics::Gdi::HBITMAP,
    ) -> Option<String> {
        use windows_sys::Win32::Graphics::Gdi::{
            GetDC, GetDIBits, ReleaseDC, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
            HBITMAP, RGBQUAD,
        };

        if hbmp.is_null() {
            return None;
        }
        let as_isize = hbmp as isize;
        if as_isize <= 16 && as_isize >= -16 {
            return None;
        }

        let hdc = GetDC(std::ptr::null_mut());
        if hdc.is_null() {
            return None;
        }

        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: 0,
                biHeight: 0,
                biPlanes: 1,
                biBitCount: 0,
                biCompression: BI_RGB as u32,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD {
                rgbBlue: 0,
                rgbGreen: 0,
                rgbRed: 0,
                rgbReserved: 0,
            }],
        };

        if GetDIBits(
            hdc,
            hbmp as HBITMAP,
            0,
            0,
            std::ptr::null_mut(),
            &mut bmi,
            DIB_RGB_COLORS,
        ) == 0
        {
            ReleaseDC(std::ptr::null_mut(), hdc);
            return None;
        }

        let w = bmi.bmiHeader.biWidth;
        let h_abs = bmi.bmiHeader.biHeight.abs();
        if w <= 0 || h_abs <= 0 || w > 256 || h_abs > 256 {
            ReleaseDC(std::ptr::null_mut(), hdc);
            return None;
        }

        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB as u32;
        bmi.bmiHeader.biHeight = -h_abs;
        bmi.bmiHeader.biSizeImage = (w * h_abs * 4) as u32;

        let mut bgra = vec![0u8; (w * h_abs * 4) as usize];
        let got = GetDIBits(
            hdc,
            hbmp as HBITMAP,
            0,
            h_abs as u32,
            bgra.as_mut_ptr() as *mut _,
            &mut bmi,
            DIB_RGB_COLORS,
        );
        ReleaseDC(std::ptr::null_mut(), hdc);
        if got == 0 {
            return None;
        }

        let mut rgba = vec![0u8; bgra.len()];
        for (i, chunk) in bgra.chunks_exact(4).enumerate() {
            let o = i * 4;
            rgba[o] = chunk[2];
            rgba[o + 1] = chunk[1];
            rgba[o + 2] = chunk[0];
            rgba[o + 3] = chunk[3];
        }
        rgba_to_png_data_url(&rgba, w as u32, h_abs as u32)
    }

    unsafe fn enumerate_hmenu(
        hmenu: windows_sys::Win32::UI::WindowsAndMessaging::HMENU,
        depth: u32,
    ) -> Vec<super::ShellMenuEntry> {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetMenuItemCount, GetMenuItemInfoW, GetSubMenu, MENUITEMINFOW, MIIM_BITMAP, MIIM_FTYPE,
            MIIM_ID, MIIM_STATE, MIIM_STRING, MIIM_SUBMENU, MFS_DISABLED, MFS_GRAYED, MFT_SEPARATOR,
        };

        if depth > 4 || hmenu.is_null() {
            return Vec::new();
        }

        let count = GetMenuItemCount(hmenu);
        if count <= 0 {
            return Vec::new();
        }

        let mut out = Vec::with_capacity(count as usize);
        for i in 0..count {
            let mut text_buf = [0u16; 512];
            let mut mii: MENUITEMINFOW = std::mem::zeroed();
            mii.cbSize = std::mem::size_of::<MENUITEMINFOW>() as u32;
            mii.fMask = MIIM_BITMAP | MIIM_FTYPE | MIIM_ID | MIIM_STATE | MIIM_STRING | MIIM_SUBMENU;
            mii.dwTypeData = text_buf.as_mut_ptr();
            mii.cch = text_buf.len() as u32 - 1;

            if GetMenuItemInfoW(hmenu, i as u32, 1, &mut mii) == 0 {
                continue;
            }

            if mii.fType & MFT_SEPARATOR != 0 {
                out.push(super::ShellMenuEntry {
                    id: 0,
                    label: String::new(),
                    disabled: true,
                    separator: true,
                    icon: None,
                    children: None,
                });
                continue;
            }

            let len = text_buf.iter().position(|&c| c == 0).unwrap_or(0);
            let label = clean_menu_label(&String::from_utf16_lossy(&text_buf[..len]));
            if label.is_empty() && mii.hSubMenu.is_null() {
                continue;
            }

            let disabled = mii.fState & (MFS_DISABLED | MFS_GRAYED) != 0;
            let icon = if !mii.hbmpItem.is_null() {
                hbitmap_to_data_url(mii.hbmpItem)
            } else {
                None
            };

            let mut children = None;
            let sub = if !mii.hSubMenu.is_null() {
                mii.hSubMenu
            } else {
                GetSubMenu(hmenu, i)
            };
            if !sub.is_null() {
                let kids = enumerate_hmenu(sub, depth + 1);
                if !kids.is_empty() {
                    children = Some(kids);
                }
            }

            out.push(super::ShellMenuEntry {
                id: if children.is_some() { 0 } else { mii.wID },
                label,
                disabled,
                separator: false,
                icon,
                children,
            });
        }
        out
    }

    unsafe fn acquire_context_menu(
        hwnd_invoke: HWND,
        path: Option<&str>,
    ) -> Result<
        (
            *mut core::ffi::c_void,
            *mut core::ffi::c_void,
            *mut windows_sys::Win32::UI::Shell::Common::ITEMIDLIST,
        ),
        String,
    > {
        use windows_sys::Win32::UI::Shell::{
            BHID_SFUIObject, CSIDL_DESKTOP, DEFCONTEXTMENU, ILFree, SHBindToObject, SHBindToParent,
            SHCreateDefaultContextMenu, SHCreateShellItemArrayFromIDLists,
            SHGetSpecialFolderLocation, SHParseDisplayName, Common::ITEMIDLIST,
        };

        const IID_ISHELLFOLDER: windows_sys::core::GUID = windows_sys::core::GUID {
            data1: 0x000214e6,
            data2: 0x0000,
            data3: 0x0000,
            data4: [0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
        };
        const IID_ICONTEXTMENU: windows_sys::core::GUID = windows_sys::core::GUID {
            data1: 0x000214e4,
            data2: 0x0000,
            data3: 0x0000,
            data4: [0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
        };

        #[repr(C)]
        struct IUnknownVtbl {
            query_interface: unsafe extern "system" fn(
                *mut core::ffi::c_void,
                *const windows_sys::core::GUID,
                *mut *mut core::ffi::c_void,
            ) -> i32,
            add_ref: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
            release: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
        }
        #[repr(C)]
        struct IShellFolderVtbl {
            base: IUnknownVtbl,
            parse_display_name: *const core::ffi::c_void,
            enum_objects: *const core::ffi::c_void,
            bind_to_object: *const core::ffi::c_void,
            bind_to_storage: *const core::ffi::c_void,
            compare_ids: *const core::ffi::c_void,
            create_view_object: *const core::ffi::c_void,
            get_attributes_of: *const core::ffi::c_void,
            get_ui_object_of: unsafe extern "system" fn(
                *mut core::ffi::c_void,
                HWND,
                u32,
                *const *const ITEMIDLIST,
                *const windows_sys::core::GUID,
                *mut u32,
                *mut *mut core::ffi::c_void,
            ) -> i32,
            get_display_name_of: *const core::ffi::c_void,
            set_name_of: *const core::ffi::c_void,
        }
        #[repr(C)]
        struct IShellItemArrayVtbl {
            base: IUnknownVtbl,
            get_count: *const core::ffi::c_void,
            get_item_at: *const core::ffi::c_void,
            enum_items: *const core::ffi::c_void,
            bind_to_handler: unsafe extern "system" fn(
                *mut core::ffi::c_void,
                *mut core::ffi::c_void,
                *const windows_sys::core::GUID,
                *const windows_sys::core::GUID,
                *mut *mut core::ffi::c_void,
            ) -> i32,
        }

        unsafe fn com_release(obj: *mut core::ffi::c_void) {
            if obj.is_null() {
                return;
            }
            let vtbl = *(obj as *mut *const IUnknownVtbl);
            ((*vtbl).release)(obj);
        }

        if path.is_none() {
            let mut desktop_pidl: *mut ITEMIDLIST = std::ptr::null_mut();
            let hr = SHGetSpecialFolderLocation(
                std::ptr::null_mut(),
                CSIDL_DESKTOP as i32,
                &mut desktop_pidl,
            );
            if hr < 0 {
                return Err(format!("SHGetSpecialFolderLocation: {hr}"));
            }
            let mut psf: *mut core::ffi::c_void = std::ptr::null_mut();
            let hr = SHBindToObject(
                std::ptr::null_mut(),
                desktop_pidl,
                std::ptr::null_mut(),
                &IID_ISHELLFOLDER,
                &mut psf,
            );
            if hr < 0 {
                ILFree(desktop_pidl);
                return Err(format!("SHBindToObject: {hr}"));
            }
            let dcm = DEFCONTEXTMENU {
                hwnd: hwnd_invoke,
                pcmcb: std::ptr::null_mut(),
                pidlFolder: desktop_pidl,
                psf,
                cidl: 0,
                apidl: std::ptr::null_mut(),
                punkAssociationInfo: std::ptr::null_mut(),
                cKeys: 0,
                aKeys: std::ptr::null(),
            };
            let mut pcm: *mut core::ffi::c_void = std::ptr::null_mut();
            let hr = SHCreateDefaultContextMenu(&dcm, &IID_ICONTEXTMENU, &mut pcm);
            if hr < 0 || pcm.is_null() {
                com_release(psf);
                ILFree(desktop_pidl);
                return Err(format!("SHCreateDefaultContextMenu desktop: {hr}"));
            }
            return Ok((pcm, psf, desktop_pidl));
        }

        let file_path = path.unwrap();
        let wpath = wide(file_path);
        let mut pidl_abs: *mut ITEMIDLIST = std::ptr::null_mut();
        let mut sfgao: u32 = 0;
        let hr = SHParseDisplayName(
            wpath.as_ptr(),
            std::ptr::null_mut(),
            &mut pidl_abs,
            0,
            &mut sfgao,
        );
        if hr < 0 {
            return Err(format!("解析路径失败: {hr}"));
        }
        let mut psf: *mut core::ffi::c_void = std::ptr::null_mut();
        let mut pidl_child: *mut ITEMIDLIST = std::ptr::null_mut();
        let hr = SHBindToParent(pidl_abs, &IID_ISHELLFOLDER, &mut psf, &mut pidl_child);
        if hr < 0 {
            ILFree(pidl_abs);
            return Err(format!("绑定 Shell 文件夹失败: {hr}"));
        }

        let mut pcm: *mut core::ffi::c_void = std::ptr::null_mut();
        let mut apidl: [*mut ITEMIDLIST; 1] = [pidl_child];
        let dcm = DEFCONTEXTMENU {
            hwnd: hwnd_invoke,
            pcmcb: std::ptr::null_mut(),
            pidlFolder: std::ptr::null_mut(),
            psf,
            cidl: 1,
            apidl: apidl.as_mut_ptr(),
            punkAssociationInfo: std::ptr::null_mut(),
            cKeys: 0,
            aKeys: std::ptr::null(),
        };
        let hr_def = SHCreateDefaultContextMenu(&dcm, &IID_ICONTEXTMENU, &mut pcm);
        if hr_def < 0 || pcm.is_null() {
            let pidl_ptr: *const ITEMIDLIST = pidl_abs;
            let mut psia: *mut core::ffi::c_void = std::ptr::null_mut();
            if SHCreateShellItemArrayFromIDLists(1, &pidl_ptr, &mut psia) >= 0 && !psia.is_null()
            {
                let psia_vtbl = *(psia as *mut *const IShellItemArrayVtbl);
                let _ = ((*psia_vtbl).bind_to_handler)(
                    psia,
                    std::ptr::null_mut(),
                    &BHID_SFUIObject,
                    &IID_ICONTEXTMENU,
                    &mut pcm,
                );
                com_release(psia);
            }
        }
        if pcm.is_null() {
            let child_array: [*const ITEMIDLIST; 1] = [pidl_child];
            let psf_vtbl = *(psf as *mut *const IShellFolderVtbl);
            let hr_ui = ((*psf_vtbl).get_ui_object_of)(
                psf,
                hwnd_invoke,
                1,
                child_array.as_ptr(),
                &IID_ICONTEXTMENU,
                std::ptr::null_mut(),
                &mut pcm,
            );
            if hr_ui < 0 {
                com_release(psf);
                ILFree(pidl_abs);
                return Err(format!("GetUIObjectOf: {hr_ui}"));
            }
        }
        Ok((pcm, psf, pidl_abs))
    }

    unsafe fn com_release_any(obj: *mut core::ffi::c_void) {
        if obj.is_null() {
            return;
        }
        #[repr(C)]
        struct IUnknownVtbl {
            query_interface: *const core::ffi::c_void,
            add_ref: *const core::ffi::c_void,
            release: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
        }
        let vtbl = *(obj as *mut *const IUnknownVtbl);
        ((*vtbl).release)(obj);
    }

    /// Enumerate Shell COM menu entries (labels + icons) for custom UI.
    pub fn list_shell_context_menu(
        hwnd_fence: HWND,
        path: Option<&str>,
    ) -> Result<Vec<super::ShellMenuEntry>, String> {
        use windows_sys::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
        use windows_sys::Win32::UI::Shell::ILFree;
        use windows_sys::Win32::UI::WindowsAndMessaging::{CreatePopupMenu, DestroyMenu};

        const CMD_FIRST: u32 = 1;
        const CMD_LAST: u32 = 0x7fff;

        #[repr(C)]
        struct IContextMenuVtbl {
            query_interface: *const core::ffi::c_void,
            add_ref: *const core::ffi::c_void,
            release: *const core::ffi::c_void,
            query_context_menu: unsafe extern "system" fn(
                *mut core::ffi::c_void,
                windows_sys::Win32::UI::WindowsAndMessaging::HMENU,
                u32,
                u32,
                u32,
                u32,
            ) -> i32,
            invoke_command: *const core::ffi::c_void,
            get_command_string: *const core::ffi::c_void,
        }

        unsafe {
            let _ = CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED as u32);
            let hwnd_invoke = desktop_defview_hwnd().unwrap_or(hwnd_fence);
            let flags = menu_flags();
            let (pcm, psf, pidl_abs) = acquire_context_menu(hwnd_invoke, path)?;

            let hmenu = CreatePopupMenu();
            if hmenu.is_null() {
                com_release_any(pcm);
                com_release_any(psf);
                if !pidl_abs.is_null() {
                    ILFree(pidl_abs);
                }
                return Err("创建菜单失败".into());
            }
            let pcm_vtbl = *(pcm as *mut *const IContextMenuVtbl);
            let hr = ((*pcm_vtbl).query_context_menu)(pcm, hmenu, 0, CMD_FIRST, CMD_LAST, flags);
            if hr < 0 {
                DestroyMenu(hmenu);
                com_release_any(pcm);
                com_release_any(psf);
                if !pidl_abs.is_null() {
                    ILFree(pidl_abs);
                }
                return Err(format!("QueryContextMenu: {hr}"));
            }

            let mut items = enumerate_hmenu(hmenu, 0);
            DestroyMenu(hmenu);
            com_release_any(pcm);
            com_release_any(psf);
            if !pidl_abs.is_null() {
                ILFree(pidl_abs);
            }

            if let Some(p) = path {
                if let Some(file_icon) = shell_name_and_icon(std::path::Path::new(p)).icon {
                    for entry in &mut items {
                        if !entry.separator && entry.children.is_none() && entry.icon.is_none() {
                            entry.icon = Some(file_icon);
                            break;
                        }
                    }
                }
            }

            Ok(items)
        }
    }

    /// Invoke a shell menu command id previously returned by list_shell_context_menu.
    pub fn invoke_shell_context_command(
        hwnd_fence: HWND,
        path: Option<&str>,
        command_id: u32,
    ) -> Result<(), String> {
        use windows_sys::Win32::Foundation::POINT;
        use windows_sys::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
        use windows_sys::Win32::UI::Shell::{
            CMINVOKECOMMANDINFOEX, CMIC_MASK_PTINVOKE, ILFree,
        };
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            CreatePopupMenu, DestroyMenu, GetCursorPos, SW_SHOWNORMAL,
        };

        const CMD_FIRST: u32 = 1;
        const CMD_LAST: u32 = 0x7fff;
        if command_id < CMD_FIRST {
            return Err("无效命令".into());
        }

        #[repr(C)]
        struct IContextMenuVtbl {
            query_interface: *const core::ffi::c_void,
            add_ref: *const core::ffi::c_void,
            release: *const core::ffi::c_void,
            query_context_menu: unsafe extern "system" fn(
                *mut core::ffi::c_void,
                windows_sys::Win32::UI::WindowsAndMessaging::HMENU,
                u32,
                u32,
                u32,
                u32,
            ) -> i32,
            invoke_command: unsafe extern "system" fn(
                *mut core::ffi::c_void,
                *const windows_sys::Win32::UI::Shell::CMINVOKECOMMANDINFO,
            ) -> i32,
            get_command_string: *const core::ffi::c_void,
        }

        unsafe {
            let _ = CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED as u32);
            let hwnd_invoke = desktop_defview_hwnd().unwrap_or(hwnd_fence);
            let flags = menu_flags();
            let mut pt = POINT { x: 0, y: 0 };
            let _ = GetCursorPos(&mut pt);

            let (pcm, psf, pidl_abs) = acquire_context_menu(hwnd_invoke, path)?;
            let hmenu = CreatePopupMenu();
            if hmenu.is_null() {
                com_release_any(pcm);
                com_release_any(psf);
                if !pidl_abs.is_null() {
                    ILFree(pidl_abs);
                }
                return Err("创建菜单失败".into());
            }
            let pcm_vtbl = *(pcm as *mut *const IContextMenuVtbl);
            let hr = ((*pcm_vtbl).query_context_menu)(pcm, hmenu, 0, CMD_FIRST, CMD_LAST, flags);
            if hr < 0 {
                DestroyMenu(hmenu);
                com_release_any(pcm);
                com_release_any(psf);
                if !pidl_abs.is_null() {
                    ILFree(pidl_abs);
                }
                return Err(format!("QueryContextMenu: {hr}"));
            }

            let verb_offset = (command_id - CMD_FIRST) as usize;
            let ici = CMINVOKECOMMANDINFOEX {
                cbSize: std::mem::size_of::<CMINVOKECOMMANDINFOEX>() as u32,
                fMask: CMIC_MASK_PTINVOKE,
                hwnd: hwnd_invoke,
                lpVerb: verb_offset as windows_sys::core::PCSTR,
                lpParameters: std::ptr::null(),
                lpDirectory: std::ptr::null(),
                nShow: SW_SHOWNORMAL,
                dwHotKey: 0,
                hIcon: std::ptr::null_mut(),
                lpTitle: std::ptr::null(),
                lpVerbW: std::ptr::null(),
                lpParametersW: std::ptr::null(),
                lpDirectoryW: std::ptr::null(),
                lpTitleW: std::ptr::null(),
                ptInvoke: pt,
            };
            let hr = ((*pcm_vtbl).invoke_command)(pcm, &ici as *const _ as *const _);
            DestroyMenu(hmenu);
            com_release_any(pcm);
            com_release_any(psf);
            if !pidl_abs.is_null() {
                ILFree(pidl_abs);
            }
            if hr < 0 {
                return Err(format!("InvokeCommand: {hr}"));
            }
            Ok(())
        }
    }

    /// Display and execute the native Shell context menu in one COM/menu lifetime.
    ///
    /// Keeping the same IContextMenu alive while TrackPopupMenuEx is running is
    /// required for dynamic and owner-drawn Shell extensions.
    pub fn show_native_shell_context_menu(
        hwnd_fence: HWND,
        path: Option<&str>,
    ) -> Result<(), String> {
        use windows_sys::Win32::Foundation::POINT;
        use windows_sys::Win32::System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED};
        use windows_sys::Win32::UI::Shell::{
            CMINVOKECOMMANDINFOEX, CMIC_MASK_PTINVOKE, ILFree,
        };
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            CreatePopupMenu, DestroyMenu, GetCursorPos, PostMessageW, SetForegroundWindow,
            TrackPopupMenuEx, SW_SHOWNORMAL, TPM_RETURNCMD, TPM_RIGHTBUTTON, WM_NULL,
        };

        const CMD_FIRST: u32 = 1;
        const CMD_LAST: u32 = 0x7fff;
        const IID_ICONTEXTMENU2: windows_sys::core::GUID = windows_sys::core::GUID {
            data1: 0x000214f4,
            data2: 0x0000,
            data3: 0x0000,
            data4: [0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
        };
        const IID_ICONTEXTMENU3: windows_sys::core::GUID = windows_sys::core::GUID {
            data1: 0xbcfce0a0,
            data2: 0xec17,
            data3: 0x11d0,
            data4: [0x8d, 0x10, 0x00, 0xa0, 0xc9, 0x0f, 0x27, 0x19],
        };

        #[repr(C)]
        struct IContextMenuVtbl {
            query_interface: unsafe extern "system" fn(
                *mut core::ffi::c_void,
                *const windows_sys::core::GUID,
                *mut *mut core::ffi::c_void,
            ) -> i32,
            add_ref: *const core::ffi::c_void,
            release: *const core::ffi::c_void,
            query_context_menu: unsafe extern "system" fn(
                *mut core::ffi::c_void,
                windows_sys::Win32::UI::WindowsAndMessaging::HMENU,
                u32,
                u32,
                u32,
                u32,
            ) -> i32,
            invoke_command: unsafe extern "system" fn(
                *mut core::ffi::c_void,
                *const windows_sys::Win32::UI::Shell::CMINVOKECOMMANDINFO,
            ) -> i32,
            get_command_string: *const core::ffi::c_void,
        }

        #[repr(C)]
        struct IContextMenu2Vtbl {
            base: IContextMenuVtbl,
            handle_menu_msg: unsafe extern "system" fn(
                *mut core::ffi::c_void,
                u32,
                WPARAM,
                LPARAM,
            ) -> i32,
        }

        #[repr(C)]
        struct IContextMenu3Vtbl {
            base: IContextMenu2Vtbl,
            handle_menu_msg2: unsafe extern "system" fn(
                *mut core::ffi::c_void,
                u32,
                WPARAM,
                LPARAM,
                *mut LRESULT,
            ) -> i32,
        }

        unsafe {
            let _ = CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED as u32);
            let (pcm, psf, pidl_abs) = acquire_context_menu(hwnd_fence, path)?;
            let hmenu = CreatePopupMenu();
            if hmenu.is_null() {
                com_release_any(pcm);
                com_release_any(psf);
                if !pidl_abs.is_null() {
                    ILFree(pidl_abs);
                }
                return Err("创建原生菜单失败".into());
            }

            let pcm_vtbl = *(pcm as *mut *const IContextMenuVtbl);
            let hr = ((*pcm_vtbl).query_context_menu)(
                pcm,
                hmenu,
                0,
                CMD_FIRST,
                CMD_LAST,
                menu_flags(),
            );
            if hr < 0 {
                DestroyMenu(hmenu);
                com_release_any(pcm);
                com_release_any(psf);
                if !pidl_abs.is_null() {
                    ILFree(pidl_abs);
                }
                return Err(format!("QueryContextMenu: {hr}"));
            }

            let mut pcm2: *mut core::ffi::c_void = std::ptr::null_mut();
            let mut pcm3: *mut core::ffi::c_void = std::ptr::null_mut();
            let _ = ((*pcm_vtbl).query_interface)(pcm, &IID_ICONTEXTMENU2, &mut pcm2);
            let _ = ((*pcm_vtbl).query_interface)(pcm, &IID_ICONTEXTMENU3, &mut pcm3);

            let pcm2_handle_menu_msg = if pcm2.is_null() {
                None
            } else {
                let vtbl = *(pcm2 as *mut *const IContextMenu2Vtbl);
                Some((*vtbl).handle_menu_msg)
            };
            let pcm3_handle_menu_msg2 = if pcm3.is_null() {
                None
            } else {
                let vtbl = *(pcm3 as *mut *const IContextMenu3Vtbl);
                Some((*vtbl).handle_menu_msg2)
            };
            CTX_MENU_FWD.with(|slot| {
                *slot.borrow_mut() = Some(CtxMenuFwd {
                    pcm2,
                    pcm2_handle_menu_msg,
                    pcm3,
                    pcm3_handle_menu_msg2,
                });
            });

            let mut pt = POINT { x: 0, y: 0 };
            let _ = GetCursorPos(&mut pt);
            // Prefer DefView as popup owner — fence HWND is a child of the desktop
            // and TrackPopupMenuEx often fails silently on it.
            let hwnd_popup = desktop_defview_hwnd().unwrap_or(hwnd_fence);
            let _ = SetForegroundWindow(hwnd_popup);
            let command_id = TrackPopupMenuEx(
                hmenu,
                TPM_RETURNCMD | TPM_RIGHTBUTTON,
                pt.x,
                pt.y,
                hwnd_popup,
                std::ptr::null(),
            ) as u32;

            CTX_MENU_FWD.with(|slot| *slot.borrow_mut() = None);
            let _ = PostMessageW(hwnd_popup, WM_NULL, 0, 0);

            let invoke_result = if command_id >= CMD_FIRST {
                let verb_offset = (command_id - CMD_FIRST) as usize;
                let ici = CMINVOKECOMMANDINFOEX {
                    cbSize: std::mem::size_of::<CMINVOKECOMMANDINFOEX>() as u32,
                    fMask: CMIC_MASK_PTINVOKE,
                    hwnd: hwnd_popup,
                    lpVerb: verb_offset as windows_sys::core::PCSTR,
                    lpParameters: std::ptr::null(),
                    lpDirectory: std::ptr::null(),
                    nShow: SW_SHOWNORMAL,
                    dwHotKey: 0,
                    hIcon: std::ptr::null_mut(),
                    lpTitle: std::ptr::null(),
                    lpVerbW: std::ptr::null(),
                    lpParametersW: std::ptr::null(),
                    lpDirectoryW: std::ptr::null(),
                    lpTitleW: std::ptr::null(),
                    ptInvoke: pt,
                };
                ((*pcm_vtbl).invoke_command)(pcm, &ici as *const _ as *const _)
            } else {
                0
            };

            DestroyMenu(hmenu);
            com_release_any(pcm3);
            com_release_any(pcm2);
            com_release_any(pcm);
            com_release_any(psf);
            if !pidl_abs.is_null() {
                ILFree(pidl_abs);
            }

            if invoke_result < 0 {
                return Err(format!("InvokeCommand: {invoke_result}"));
            }
            Ok(())
        }
    }
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
pub fn list_desktop_shell_context_menu(
    app: AppHandle,
    path: String,
) -> Result<Vec<ShellMenuEntry>, String> {
    let trimmed = path.trim();
    let path_opt = if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    };
    let window = app
        .get_webview_window(FENCE_LABEL)
        .ok_or_else(|| "格子窗口未就绪".to_string())?;

    #[cfg(windows)]
    {
        let hwnd_raw = window.hwnd().map_err(|e| e.to_string())?.0 as isize;
        let handle = app.clone();
        return run_on_ui(&handle, move || {
            win::list_shell_context_menu(
                hwnd_raw as windows_sys::Win32::Foundation::HWND,
                path_opt.as_deref(),
            )
        })?;
    }
    #[cfg(not(windows))]
    {
        let _ = (window, path_opt);
        Err("桌面整理仅支持 Windows".into())
    }
}

/// Invoke a Shell COM context menu command previously listed for path/blank desktop.
#[tauri::command]
pub fn invoke_desktop_shell_context_command(
    app: AppHandle,
    path: String,
    command_id: u32,
) -> Result<(), String> {
    let trimmed = path.trim();
    let path_opt = if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    };
    let window = app
        .get_webview_window(FENCE_LABEL)
        .ok_or_else(|| "格子窗口未就绪".to_string())?;

    #[cfg(windows)]
    {
        let hwnd_raw = window.hwnd().map_err(|e| e.to_string())?.0 as isize;
        let handle = app.clone();
        return run_on_ui(&handle, move || {
            win::invoke_shell_context_command(
                hwnd_raw as windows_sys::Win32::Foundation::HWND,
                path_opt.as_deref(),
                command_id,
            )
        })?;
    }
    #[cfg(not(windows))]
    {
        let _ = (window, path_opt, command_id);
        Err("桌面整理仅支持 Windows".into())
    }
}

/// Show the real Windows Shell menu and execute the selected command before
/// releasing its COM objects, preserving dynamic/owner-drawn menu behavior.
#[tauri::command]
pub fn show_desktop_native_context_menu(app: AppHandle, path: String) -> Result<(), String> {
    let trimmed = path.trim();
    let path_opt = if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    };
    let window = app
        .get_webview_window(FENCE_LABEL)
        .ok_or_else(|| "格子窗口未就绪".to_string())?;

    #[cfg(windows)]
    {
        let hwnd_raw = window.hwnd().map_err(|e| e.to_string())?.0 as isize;
        let handle = app.clone();
        return run_on_ui(&handle, move || {
            win::show_native_shell_context_menu(
                hwnd_raw as windows_sys::Win32::Foundation::HWND,
                path_opt.as_deref(),
            )
        })?;
    }
    #[cfg(not(windows))]
    {
        let _ = (window, path_opt);
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
        use windows_sys::Win32::UI::Shell::{
            FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FO_DELETE, SHFILEOPSTRUCTW, SHFileOperationW,
        };
        unsafe {
            let mut wpath: Vec<u16> = OsStr::new(trimmed)
                .encode_wide()
                .chain(std::iter::once(0))
                .chain(std::iter::once(0))
                .collect();
            let mut op = SHFILEOPSTRUCTW {
                hwnd: std::ptr::null_mut(),
                wFunc: FO_DELETE,
                pFrom: wpath.as_ptr(),
                pTo: std::ptr::null(),
                fFlags: (FOF_ALLOWUNDO | FOF_NOCONFIRMATION) as u16,
                fAnyOperationsAborted: 0,
                hNameMappings: std::ptr::null_mut(),
                lpszProgressTitle: std::ptr::null(),
            };
            let hr = SHFileOperationW(&mut op);
            if hr != 0 || op.fAnyOperationsAborted != 0 {
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
fn window_class_name(hwnd: windows_sys::Win32::Foundation::HWND) -> String {
    use windows_sys::Win32::UI::WindowsAndMessaging::GetClassNameW;
    unsafe {
        let mut buf = [0u16; 256];
        let n = GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
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
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
    unsafe { (GetAsyncKeyState(VK_LBUTTON as i32) as u16 & 0x8000) != 0 }
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
    use windows_sys::Win32::Foundation::{HWND, POINT};
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetParent, WindowFromPoint};
    unsafe {
        let fence = fence_hwnd as HWND;
        if fence.is_null() {
            return false;
        }
        let own = collect_own_hwnds(app, fence_hwnd);
        let mut pt = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut pt) == 0 {
            return false;
        }
        let mut hwnd = WindowFromPoint(pt);
        // Fence is a WS_CHILD of the desktop DefView — walk parents instead of GA_ROOT.
        for _ in 0..24 {
            if hwnd.is_null() {
                return true;
            }
            let id = hwnd as isize;
            if own.contains(&id) {
                return false;
            }
            let class = window_class_name(hwnd);
            if is_desktop_shell_class(&class) {
                return false;
            }
            hwnd = GetParent(hwnd);
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
