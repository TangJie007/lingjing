use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use tauri::{AppHandle, Emitter, Manager};

use super::scan::strip_extended_path;
use super::state::FENCE_LABEL;
use super::util::run_on_ui;
#[cfg(windows)]
use super::win;

static OUTGOING_WATCH_GEN: AtomicU64 = AtomicU64::new(0);

fn from_base64(input: &str) -> Option<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(input.trim()).ok()
}

fn decode_image_data_url(url: &str) -> Option<Vec<u8>> {
    let url = url.trim();
    let b64 = url
        .strip_prefix("data:image/png;base64,")
        .or_else(|| url.strip_prefix("data:image/PNG;base64,"))?;
    from_base64(b64)
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
fn is_desktop_root_class(class: &str) -> bool {
    // Only Progman/WorkerW identify the real desktop surface.
    // Do NOT treat SHELLDLL_DefView alone as desktop — Explorer folder windows
    // also host SHELLDLL_DefView under CabinetWClass.
    matches!(class, "Progman" | "WorkerW")
}

#[cfg(windows)]
fn is_explorer_frame_class(class: &str) -> bool {
    matches!(
        class,
        "CabinetWClass"
            | "ExploreWClass"
            | "Microsoft.UI.Content.DesktopChildSiteBridge"
            | "XamlExplorerHostIslandWindow"
            | "Windows.UI.Core.CoreWindow"
    ) || class.starts_with("Windows.UI.Core.CoreWindow")
        || class.contains("Explorer")
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

/// Classify cursor hit for outgoing drag. Returns (is_foreign, debug reason).
#[cfg(windows)]
fn classify_cursor_foreign(app: &AppHandle, fence_hwnd: isize) -> (bool, String) {
    use windows::Win32::Foundation::{HWND, POINT};
    use windows::Win32::UI::WindowsAndMessaging::{
        GetAncestor, GetCursorPos, GetParent, WindowFromPoint, GA_ROOT,
    };
    unsafe {
        let fence = HWND(fence_hwnd as *mut _);
        if fence.0.is_null() {
            return (false, "fence hwnd null".into());
        }
        let own = collect_own_hwnds(app, fence_hwnd);
        let mut pt = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut pt).is_err() {
            return (false, "GetCursorPos failed".into());
        }
        let hit = WindowFromPoint(pt);
        if hit.0.is_null() {
            return (true, format!("null hit at {},{}", pt.x, pt.y));
        }
        let hit_class = window_class_name(hit);
        let mut chain = Vec::new();
        // Fence is a WS_CHILD of the desktop DefView — walk parents.
        // SHELLDLL_DefView appears both on the desktop AND inside Explorer;
        // keep walking until Progman/WorkerW (desktop) or CabinetWClass (Explorer).
        let mut hwnd = hit;
        for _ in 0..24 {
            if hwnd.0.is_null() {
                break;
            }
            let id = hwnd.0 as isize;
            let class = window_class_name(hwnd);
            chain.push(format!("{class}({id:#x})"));
            if own.contains(&id) {
                return (
                    false,
                    format!(
                        "own hwnd at {},{} hit={hit_class} chain=[{}]",
                        pt.x,
                        pt.y,
                        chain.join(" > ")
                    ),
                );
            }
            if is_explorer_frame_class(&class) {
                return (
                    true,
                    format!(
                        "explorer frame at {},{} hit={hit_class} chain=[{}]",
                        pt.x,
                        pt.y,
                        chain.join(" > ")
                    ),
                );
            }
            if is_desktop_root_class(&class) {
                return (
                    false,
                    format!(
                        "desktop root at {},{} hit={hit_class} chain=[{}]",
                        pt.x,
                        pt.y,
                        chain.join(" > ")
                    ),
                );
            }
            hwnd = GetParent(hwnd).unwrap_or_default();
        }

        let root = GetAncestor(hit, GA_ROOT);
        if !root.0.is_null() {
            let rid = root.0 as isize;
            let rc = window_class_name(root);
            chain.push(format!("root:{rc}({rid:#x})"));
            if own.contains(&rid) {
                return (
                    false,
                    format!(
                        "own root at {},{} hit={hit_class} chain=[{}]",
                        pt.x,
                        pt.y,
                        chain.join(" > ")
                    ),
                );
            }
            if is_desktop_root_class(&rc) || rc == "Progman" {
                return (
                    false,
                    format!(
                        "desktop root-ancestor at {},{} hit={hit_class} chain=[{}]",
                        pt.x,
                        pt.y,
                        chain.join(" > ")
                    ),
                );
            }
            if is_explorer_frame_class(&rc) || !rc.is_empty() {
                return (
                    true,
                    format!(
                        "foreign root at {},{} hit={hit_class} chain=[{}]",
                        pt.x,
                        pt.y,
                        chain.join(" > ")
                    ),
                );
            }
        }
        (
            false,
            format!(
                "unclassified at {},{} hit={hit_class} chain=[{}]",
                pt.x,
                pt.y,
                chain.join(" > ")
            ),
        )
    }
}

#[cfg(windows)]
fn is_cursor_over_foreign_window(app: &AppHandle, fence_hwnd: isize) -> bool {
    classify_cursor_foreign(app, fence_hwnd).0
}

#[cfg(windows)]
mod ole_drag {
    use std::ffi::c_void;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;
    use std::sync::Once;

    use windows::core::{implement, BOOL, HRESULT, PCWSTR};
    use windows::Win32::Foundation::{
        COLORREF, DRAGDROP_S_CANCEL, DRAGDROP_S_DROP, DRAGDROP_S_USEDEFAULTCURSORS, POINT, SIZE,
        S_OK,
    };
    use windows::Win32::Graphics::Gdi::{CreateBitmap, GetObjectW, BITMAP, HBITMAP};
    use windows::Win32::Graphics::Imaging::{
        CLSID_WICImagingFactory, GUID_WICPixelFormat32bppPBGRA, IWICImagingFactory,
        WICConvertBitmapSource, WICDecodeMetadataCacheOnDemand,
    };
    use windows::Win32::System::Com::{
        CoCreateInstance, IDataObject, CLSCTX_INPROC_SERVER,
    };
    use windows::Win32::System::Ole::{
        DoDragDrop, IDropSource, IDropSource_Impl, OleInitialize, DROPEFFECT, DROPEFFECT_COPY,
        DROPEFFECT_MOVE,
    };
    use windows::Win32::System::SystemServices::{MK_LBUTTON, MODIFIERKEYS_FLAGS};
    use windows::Win32::UI::Shell::{
        BHID_DataObject, CLSID_DragDropHelper, Common, IDragSourceHelper, IShellItemArray,
        ILCreateFromPathW, ILFree, SHCreateShellItemArrayFromIDLists, SHDRAGIMAGE,
    };

    static OLE_INIT: Once = Once::new();
    static mut OLE_INIT_OK: bool = false;

    fn init_ole() -> Result<(), String> {
        OLE_INIT.call_once(|| {
            let ok = unsafe { OleInitialize(None).is_ok() };
            unsafe {
                OLE_INIT_OK = ok;
            }
        });
        if unsafe { OLE_INIT_OK } {
            Ok(())
        } else {
            Err("OleInitialize 失败".into())
        }
    }

    #[implement(IDropSource)]
    struct ShellFileDropSource;

    #[allow(non_snake_case)]
    impl IDropSource_Impl for ShellFileDropSource_Impl {
        fn QueryContinueDrag(
            &self,
            fescapepressed: BOOL,
            grfkeystate: MODIFIERKEYS_FLAGS,
        ) -> HRESULT {
            if fescapepressed.as_bool() {
                DRAGDROP_S_CANCEL
            } else if (grfkeystate.0 & MK_LBUTTON.0) == 0 {
                DRAGDROP_S_DROP
            } else {
                S_OK
            }
        }

        fn GiveFeedback(&self, _dweffect: DROPEFFECT) -> HRESULT {
            DRAGDROP_S_USEDEFAULTCURSORS
        }
    }

    fn file_data_object(path: &Path) -> Result<IDataObject, String> {
        unsafe {
            let wide: Vec<u16> = path.as_os_str().encode_wide().chain(once(0)).collect();
            let pidl = ILCreateFromPathW(PCWSTR::from_raw(wide.as_ptr()));
            if pidl.is_null() {
                return Err("无法创建 Shell 项".into());
            }
            let list: [*const Common::ITEMIDLIST; 1] = [pidl.cast_const()];
            let array = SHCreateShellItemArrayFromIDLists(&list);
            ILFree(Some(pidl));
            let array: IShellItemArray =
                array.map_err(|e| format!("SHCreateShellItemArrayFromIDLists: {e}"))?;
            array
                .BindToHandler(None, &BHID_DataObject)
                .map_err(|e| format!("BindToHandler(BHID_DataObject): {e}"))
        }
    }

    fn png_to_hbitmap(bytes: &[u8]) -> Option<HBITMAP> {
        unsafe {
            let factory: IWICImagingFactory =
                CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER).ok()?;
            let stream = factory.CreateStream().ok()?;
            stream.InitializeFromMemory(bytes).ok()?;
            let decoder = factory
                .CreateDecoderFromStream(&stream, std::ptr::null(), WICDecodeMetadataCacheOnDemand)
                .ok()?;
            let frame = decoder.GetFrame(0).ok()?;

            let mut width = 0u32;
            let mut height = 0u32;
            frame.GetSize(&mut width, &mut height).ok()?;

            let mut pixel_buf = vec![0u8; (width * height * 4) as usize];
            let pixel_format = frame.GetPixelFormat().ok()?;
            if pixel_format != GUID_WICPixelFormat32bppPBGRA {
                let converted =
                    WICConvertBitmapSource(&GUID_WICPixelFormat32bppPBGRA, &frame).ok()?;
                converted
                    .CopyPixels(std::ptr::null(), width * 4, &mut pixel_buf)
                    .ok()?;
            } else {
                frame
                    .CopyPixels(std::ptr::null(), width * 4, &mut pixel_buf)
                    .ok()?;
            }

            let hbmp = CreateBitmap(
                width as i32,
                height as i32,
                1,
                32,
                Some(pixel_buf.as_ptr() as *const c_void),
            );
            if hbmp.is_invalid() {
                None
            } else {
                Some(hbmp)
            }
        }
    }

    fn attach_drag_image(data_object: &IDataObject, preview_png: &[u8]) {
        let Some(hbitmap) = png_to_hbitmap(preview_png) else {
            return;
        };
        unsafe {
            let mut bitmap = BITMAP::default();
            let (width, height) = if GetObjectW(
                hbitmap.into(),
                std::mem::size_of::<BITMAP>() as i32,
                Some(&mut bitmap as *mut BITMAP as *mut c_void),
            ) == 0
            {
                (128, 128)
            } else {
                (bitmap.bmWidth, bitmap.bmHeight)
            };
            let drag_image = SHDRAGIMAGE {
                sizeDragImage: SIZE {
                    cx: width,
                    cy: height,
                },
                ptOffset: POINT { x: 0, y: 0 },
                hbmpDragImage: hbitmap,
                crColorKey: COLORREF(0),
            };
            if let Ok(helper) =
                CoCreateInstance::<_, IDragSourceHelper>(&CLSID_DragDropHelper, None, CLSCTX_INPROC_SERVER)
            {
                let _ = helper.InitializeFromBitmap(&drag_image, data_object);
            }
        }
    }

    /// Start OLE drag with both COPY and MOVE allowed; the drop target picks the effect.
    pub fn start(path: &Path, preview_png: &[u8]) -> Result<(), String> {
        init_ole()?;
        // Prefer non-extended paths — Explorer is picky about \\?\ prefixes in drag data.
        let clean = {
            let s = path.to_string_lossy().replace('/', "\\");
            let trimmed = s
                .strip_prefix(r"\\?\UNC\")
                .map(|rest| format!(r"\\{rest}"))
                .or_else(|| s.strip_prefix(r"\\?\").map(|rest| rest.to_string()))
                .unwrap_or(s);
            std::path::PathBuf::from(trimmed)
        };
        let data_object = file_data_object(&clean)?;
        attach_drag_image(&data_object, preview_png);
        let drop_source: IDropSource = ShellFileDropSource.into();
        let ok_effects = DROPEFFECT_COPY | DROPEFFECT_MOVE;
        let mut out_effect = DROPEFFECT::default();

        // Prevent our own desktop drop targets from swallowing the drag
        // (would look like "dropped successfully" but copy-onto-self is a no-op).
        super::super::drop_target::set_outgoing_drag(true);
        struct OutgoingGuard;
        impl Drop for OutgoingGuard {
            fn drop(&mut self) {
                super::super::drop_target::set_outgoing_drag(false);
            }
        }
        let _guard = OutgoingGuard;

        let hr = unsafe { DoDragDrop(&data_object, &drop_source, ok_effects, &mut out_effect) };
        tracing::info!(
            "[desktop-organize] DoDragDrop finished hr={hr:?} effect={out_effect:?} path={}",
            clean.display()
        );
        if hr.is_err() {
            return Err(format!("启动文件拖放失败: {hr:?}"));
        }
        Ok(())
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
        let (foreign, reason) = classify_cursor_foreign(&app, hwnd);
        tracing::info!("[desktop-organize] probe foreign={foreign} {reason}");
        Ok(foreign)
    }
    #[cfg(not(windows))]
    {
        let _ = app;
        Ok(false)
    }
}

/// Start a system shell file drag (CF_HDROP) so icons can be dropped into other apps.
/// Allowed effects are COPY|MOVE; the drop target (Explorer / upload UI / etc.) chooses.
/// `mode` is accepted for API compatibility and ignored.
#[tauri::command]
pub fn start_desktop_file_drag(
    app: AppHandle,
    path: String,
    mode: Option<String>,
    preview_data_url: Option<String>,
) -> Result<(), String> {
    let _ = mode;
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
        let _win = window;
        tracing::info!("[desktop-organize] starting shell file drag path={trimmed} effects=COPY|MOVE");
        return run_on_ui(&handle, move || {
            if !is_lbutton_down() {
                return Err("鼠标已松开，取消拖出".into());
            }
            ole_drag::start(&abs, &preview)?;
            tracing::info!("[desktop-organize] shell file drag finished path={trimmed}");
            // Refresh so moved-away items disappear from fences.
            let _ = super::lifecycle::refresh(&app);
            Ok(())
        })?;
    }
    #[cfg(not(windows))]
    {
        let _ = (window, abs, preview);
        Err("桌面整理拖出仅支持 Windows".into())
    }
}

/// Re-check cursor is over a foreign window and LBUTTON is down, then start OLE drag.
/// Returns `Ok(false)` when not over foreign (caller keeps probing). `Ok(true)` after a
/// completed drag session. Fails if the button was released before DoDragDrop.
#[tauri::command]
pub fn try_start_desktop_file_drag_if_foreign(
    app: AppHandle,
    path: String,
    mode: Option<String>,
    preview_data_url: Option<String>,
) -> Result<bool, String> {
    #[cfg(windows)]
    {
        let window = app
            .get_webview_window(FENCE_LABEL)
            .ok_or_else(|| "格子窗口未就绪".to_string())?;
        let hwnd = window.hwnd().map_err(|e| e.to_string())?.0 as isize;
        if !is_cursor_over_foreign_window(&app, hwnd) {
            return Ok(false);
        }
        if !is_lbutton_down() {
            return Err("鼠标已松开，取消拖出".into());
        }
        start_desktop_file_drag(app, path, mode, preview_data_url)?;
        Ok(true)
    }
    #[cfg(not(windows))]
    {
        let _ = (app, path, mode, preview_data_url);
        Ok(false)
    }
}

/// Native poller: while LBUTTON is held, watch for cursor over Explorer / other windows
/// and start OLE drag. Survives WebView timer throttling when focus is on Explorer.
/// Returns a generation id; call `cancel_desktop_outgoing_drag_watch` to abort.
#[tauri::command]
pub fn arm_desktop_outgoing_drag_watch(
    app: AppHandle,
    path: String,
    preview_data_url: Option<String>,
) -> Result<u64, String> {
    #[cfg(windows)]
    {
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
        let preview = drag_preview_png(&abs, preview_data_url.as_deref());
        let gen = OUTGOING_WATCH_GEN.fetch_add(1, Ordering::SeqCst) + 1;
        tracing::info!(
            "[desktop-organize] arm outgoing watch gen={gen} path={trimmed}"
        );

        std::thread::spawn(move || {
            let mut last_log = std::time::Instant::now()
                .checked_sub(std::time::Duration::from_secs(1))
                .unwrap_or_else(std::time::Instant::now);
            while OUTGOING_WATCH_GEN.load(Ordering::SeqCst) == gen {
                if !is_lbutton_down() {
                    tracing::info!(
                        "[desktop-organize] watch gen={gen} stop: LBUTTON up path={trimmed}"
                    );
                    break;
                }
                let Some(window) = app.get_webview_window(FENCE_LABEL) else {
                    tracing::info!("[desktop-organize] watch gen={gen} stop: no fence window");
                    break;
                };
                let Ok(hwnd) = window.hwnd() else {
                    tracing::info!("[desktop-organize] watch gen={gen} stop: no hwnd");
                    break;
                };
                let hwnd = hwnd.0 as isize;
                let (foreign, reason) = classify_cursor_foreign(&app, hwnd);
                if last_log.elapsed() >= std::time::Duration::from_millis(200) {
                    tracing::info!(
                        "[desktop-organize] watch gen={gen} foreign={foreign} {reason}"
                    );
                    last_log = std::time::Instant::now();
                }
                if !foreign {
                    std::thread::sleep(std::time::Duration::from_millis(16));
                    continue;
                }
                if OUTGOING_WATCH_GEN.load(Ordering::SeqCst) != gen {
                    break;
                }
                // Tell the webview to tear down Sortable *before* DoDragDrop,
                // while LBUTTON is still physically down.
                tracing::info!(
                    "[desktop-organize] watch gen={gen} handoff → OLE path={trimmed} {reason}"
                );
                let _ = app.emit("desktop-outgoing-drag-handoff", trimmed.clone());
                std::thread::sleep(std::time::Duration::from_millis(24));
                if OUTGOING_WATCH_GEN.load(Ordering::SeqCst) != gen || !is_lbutton_down() {
                    tracing::info!(
                        "[desktop-organize] watch gen={gen} aborted before DoDragDrop lbtn={} gen_ok={}",
                        is_lbutton_down(),
                        OUTGOING_WATCH_GEN.load(Ordering::SeqCst) == gen
                    );
                    break;
                }
                let app2 = app.clone();
                let abs2 = abs.clone();
                let preview2 = preview.clone();
                let path_log = trimmed.clone();
                let result = run_on_ui(&app, move || -> Result<(), String> {
                    if !is_lbutton_down() {
                        return Err("鼠标已松开，取消拖出".into());
                    }
                    tracing::info!(
                        "[desktop-organize] watch starting shell file drag path={path_log} effects=COPY|MOVE"
                    );
                    ole_drag::start(&abs2, &preview2)?;
                    let _ = super::lifecycle::refresh(&app2);
                    Ok(())
                });
                match result {
                    Ok(Ok(())) => {
                        tracing::info!(
                            "[desktop-organize] watch shell file drag finished path={trimmed}"
                        );
                        let _ = app.emit("desktop-outgoing-drag-done", true);
                    }
                    Ok(Err(e)) => {
                        tracing::info!(
                            "[desktop-organize] watch shell file drag failed path={trimmed}: {e}"
                        );
                        let _ = app.emit("desktop-outgoing-drag-done", false);
                    }
                    Err(e) => {
                        tracing::info!(
                            "[desktop-organize] watch shell file drag ui failed path={trimmed}: {e}"
                        );
                        let _ = app.emit("desktop-outgoing-drag-done", false);
                    }
                }
                // Invalidate this watch so we don't re-enter.
                let _ = OUTGOING_WATCH_GEN.compare_exchange(
                    gen,
                    gen.wrapping_add(1),
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                );
                break;
            }
        });

        Ok(gen)
    }
    #[cfg(not(windows))]
    {
        let _ = (app, path, preview_data_url);
        Ok(0)
    }
}

#[tauri::command]
pub fn cancel_desktop_outgoing_drag_watch() {
    let gen = OUTGOING_WATCH_GEN.fetch_add(1, Ordering::SeqCst);
    tracing::info!("[desktop-organize] cancel outgoing watch prev_gen={gen}");
}

/// Tiny valid PNG (1x1 transparent) used as drag preview when no icon file is handy.
const MINI_DRAG_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];
