use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager};

use crate::desktop;
use crate::settings;

const FENCE_LABEL: &str = "desktop-fence";
static ACTIVE: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
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

fn scan_dir(dir: &Path, items: &mut Vec<DesktopItem>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        let is_dir = meta.is_dir();
        let (display_name, icon) = shell_name_and_icon(&path, &name, is_dir);
        items.push(DesktopItem {
            name: display_name,
            path: path.to_string_lossy().to_string(),
            is_dir,
            kind: classify_kind(&name, is_dir),
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
        return "other".into();
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
    if APPS.contains(&ext.as_str()) {
        "app".into()
    } else if IMAGES.contains(&ext.as_str()) {
        "image".into()
    } else if DOCS.contains(&ext.as_str()) {
        "document".into()
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
    if let Ok(home) = std::env::var("USERPROFILE") {
        scan_dir(&PathBuf::from(home).join("Desktop"), &mut items);
    }
    let public = std::env::var("PUBLIC").unwrap_or_else(|_| r"C:\Users\Public".into());
    scan_dir(&PathBuf::from(public).join("Desktop"), &mut items);
    items.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(items)
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

    fn rgba_to_png_data_url(rgba: &[u8], w: u32, h: u32) -> Option<String> {
        let mut buf = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut buf, w, h);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().ok()?;
            writer.write_image_data(rgba).ok()?;
        }
        Some(format!("data:image/png;base64,{}", to_base64(&buf)))
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

    unsafe fn shell_item_icon_png(path: &std::path::Path, px: i32) -> Option<String> {
        use windows_sys::Win32::Foundation::SIZE;
        use windows_sys::Win32::Graphics::Gdi::{DeleteObject, HBITMAP};
        use windows_sys::Win32::UI::Shell::{
            SHCreateItemFromParsingName, SIIGBF_BIGGERSIZEOK, SIIGBF_ICONONLY,
        };

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
        let flags = SIIGBF_ICONONLY | SIIGBF_BIGGERSIZEOK;
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

    unsafe fn icon_source(path: &std::path::Path) -> (std::path::PathBuf, i32) {
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
                if let Some(best) = extract_best_icon(path, info.iIcon) {
                    icon = hicon_to_png_data_url(best);
                    DestroyIcon(best);
                }
                if icon.is_none() {
                    icon = shell_item_icon_png(path, 48);
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

    pub fn hide_fence_from_desktop(hwnd_raw: isize) {
        unsafe {
            let child = hwnd_raw as HWND;
            FENCE_SHOWN.store(false, Ordering::SeqCst);
            force_child_chrome(child, false);
            ShowWindow(child, SW_HIDE);
            SetWindowPos(
                child,
                std::ptr::null_mut(),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_HIDEWINDOW | SWP_FRAMECHANGED,
            );
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
        let _ = win::attach_fence_to_desktop(hwnd.0 as isize);
    }
    #[cfg(not(windows))]
    {
        return Err("桌面整理仅支持 Windows".into());
    }

    let items = scan_desktop_items()?;
    let _ = window.eval("location.reload()");
    push_items_to_fence(app, &items)?;
    ACTIVE.store(true, Ordering::SeqCst);
    eprintln!("[desktop-organize] enabled items={}", items.len());
    Ok(())
}

fn disable_inner(app: &AppHandle) -> Result<(), String> {
    ACTIVE.store(false, Ordering::SeqCst);

    if let Some(window) = app.get_webview_window(FENCE_LABEL) {
        #[cfg(windows)]
        if let Ok(hwnd) = window.hwnd() {
            win::hide_fence_from_desktop(hwnd.0 as isize);
        }
        let _ = window.eval(
            "document.getElementById('apps').innerHTML='';document.getElementById('images').innerHTML='';document.getElementById('documents').innerHTML='';",
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
    if let Ok(hwnd) = window.hwnd() {
        let _ = win::attach_fence_to_desktop(hwnd.0 as isize);
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
    if !p.exists() {
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
