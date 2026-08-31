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
        GWLP_WNDPROC, HICON, HTCLIENT, HWND_BOTTOM, LWA_ALPHA, MONITORINFOF_PRIMARY, SM_CXSCREEN,
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
        use base64::Engine;
        rgba_to_png_bytes(rgba, w, h).map(|buf| {
            format!(
                "data:image/png;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(&buf)
            )
        })
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

    /// True when the folder has at least one non-hidden entry (files or subdirs).
    fn dir_has_visible_entries(path: &std::path::Path) -> bool {
        let Ok(entries) = std::fs::read_dir(path) else {
            return false;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with('.') {
                continue;
            }
            // Skip common desktop.ini / Thumbs.db noise that still count as "content"
            // for Windows Explorer empty-folder glyph in some views — keep them as content
            // so we match Explorer: any entry → non-empty look.
            return true;
        }
        false
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
            SHGFI_SYSICONINDEX, SIIGBF_BIGGERSIZEOK, SIIGBF_ICONONLY, SIIGBF_SCALEUP,
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
                // Folders: prefer Shell thumbnail so non-empty dirs show the
                // "papers inside" glyph; ICONONLY always yields the empty look.
                let is_dir = path.is_dir();
                if is_dir && dir_has_visible_entries(path) {
                    icon = shell_item_image_png(
                        path,
                        48,
                        SIIGBF_BIGGERSIZEOK | SIIGBF_SCALEUP,
                    );
                }
                if icon.is_none() {
                    icon = shell_item_image_png(
                        path,
                        48,
                        SIIGBF_ICONONLY | SIIGBF_BIGGERSIZEOK,
                    );
                }
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
        (
            super::builtin_links::CLSID_COMPUTER,
            "此电脑",
            None,
        ),
        (
            super::builtin_links::CLSID_RECYCLE,
            "回收站",
            Some(windows::Win32::UI::Shell::SIID_RECYCLER),
        ),
        (
            super::builtin_links::CLSID_NETWORK,
            "网络",
            Some(windows::Win32::UI::Shell::SIID_MYNETWORK),
        ),
    ];

    pub fn scan_builtin_desktop_icons() -> Vec<super::types::DesktopItem> {
        let mut items = Vec::with_capacity(BUILTIN_DESKTOP_ICONS.len());
        for (clsid, fallback_name, stock) in BUILTIN_DESKTOP_ICONS {
            let path_obj = std::path::PathBuf::from(*clsid);
            let meta = shell_name_and_icon(&path_obj);
            let icon = extract_builtin_icon(clsid, *stock).or(meta.icon);
            let name = meta
                .display_name
                .map(|s| s.trim().to_string())
                .filter(|s| {
                    !s.is_empty()
                        && !s.eq_ignore_ascii_case("computer")
                        && !s.eq_ignore_ascii_case("recycle")
                        && !s.eq_ignore_ascii_case("network")
                })
                .unwrap_or_else(|| (*fallback_name).to_string());

            items.push(super::types::DesktopItem {
                name,
                path: (*clsid).to_string(),
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

    /// Desktop icon list under DefView (sibling of our fence after SetParent).
    fn find_defview_listview(defview: HWND) -> HWND {
        unsafe {
            if defview.0.is_null() {
                return HWND::default();
            }
            let listview = wide("SysListView32");
            FindWindowExW(
                Some(defview),
                None,
                PCWSTR(listview.as_ptr()),
                PCWSTR::null(),
            )
            .unwrap_or_default()
        }
    }

    /// Keep fence under SysListView32 in DefView Z-order (icons above organize UI).
    unsafe fn place_fence_below_icons(fence: HWND, defview: HWND) {
        let listview = find_defview_listview(defview);
        // hWndInsertAfter = sibling above us → fence sits just below SysListView32.
        let above = if !listview.0.is_null() {
            Some(listview)
        } else {
            Some(HWND_BOTTOM)
        };
        let _ = SetWindowPos(
            fence,
            above,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );
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

            let listview = find_defview_listview(parent);
            // Place under SysListView32 (or bottom of DefView if listview missing).
            let z_after = if !listview.0.is_null() {
                Some(listview)
            } else {
                Some(HWND_BOTTOM)
            };
            let _ = SetWindowPos(
                child,
                z_after,
                x,
                y,
                mw,
                mh,
                SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_FRAMECHANGED,
            );
            force_child_chrome(child, true);
            let _ = ShowWindow(child, SW_SHOW);
            // ShowWindow can reshuffle Z-order — pin under icons again.
            if parent == data.defview || !data.defview.0.is_null() {
                place_fence_below_icons(child, if data.defview.0.is_null() { parent } else { data.defview });
            }
            let _ = RedrawWindow(
                Some(child),
                None,
                None,
                RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN,
            );

            tracing::info!(
                "[desktop-organize] attached child={child:?} parent={parent:?} z-under={listview:?} parent={pl},{pt} {pw}x{ph} primary={x},{y} {mw}x{mh}"
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
