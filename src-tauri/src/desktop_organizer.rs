//! Desktop organizer — one-click tidy.
//! Arranges every desktop icon against the right edge of the primary screen,
//! grouped by category (folders → docs → media → other → apps). No overlay window.

#![cfg(target_os = "windows")]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::InvalidateRect;
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use windows::Win32::System::Memory::{
    VirtualAllocEx, VirtualFreeEx, MEM_RELEASE, PAGE_READWRITE, VIRTUAL_ALLOCATION_TYPE,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowExW, FindWindowW, GetSystemMetrics, GetWindowLongPtrW, GetWindowThreadProcessId,
    SendMessageW, SetWindowLongPtrW, ShowWindow, GWL_STYLE, SM_CXICONSPACING, SM_CYICONSPACING,
    SW_HIDE, SW_SHOW,
};

// ============================================================
// LVM constants (commctrl.h)
// ============================================================
const LVM_FIRST: u32 = 0x1000;
const LVM_GETITEMCOUNT: u32 = LVM_FIRST + 4;
const LVM_SETITEMPOSITION: u32 = LVM_FIRST + 15;
const LVM_GETITEMTEXTW: u32 = LVM_FIRST + 0x73;
const LVM_REDRAWITEMS: u32 = LVM_FIRST + 21;
const LVIF_TEXT: u32 = 0x0001;

// ============================================================
// Data types
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopIcon {
    pub name: String,
    pub path: String,
    pub ext: String,
    pub is_dir: bool,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizeDesktopResult {
    pub arranged: u32,
    pub skipped: u32,
    pub icon_count: u32,
}

// ============================================================
// ListView cross-process helpers
// ============================================================

/// LVITEMW structure layout for 64-bit Windows.
/// Manually defined to guarantee correct alignment without needing Win32_UI_Controls feature.
#[repr(C)]
struct LvItemW {
    mask: u32,         // offset 0
    i_item: i32,       // offset 4
    i_sub_item: i32,   // offset 8
    state: u32,        // offset 12
    state_mask: u32,   // offset 16
    _pad1: u32,        // offset 20 — padding before 8-byte pointer
    psz_text: u64,     // offset 24 — LPWSTR (pointer in remote process)
    cch_text_max: i32, // offset 32
    i_image: i32,      // offset 36
    l_param: i64,      // offset 40
    i_indent: i32,     // offset 48
    i_group_id: i32,   // offset 52
    c_columns: u32,    // offset 56
    _pad2: u32,        // offset 60
    pui_columns: u64,  // offset 64
    pi_col_fmt: u64,   // offset 72
    i_group: i32,      // offset 80
    _pad3: u32,        // offset 84
}
// static assert: sizeof == 88

unsafe fn find_desktop_listview_hwnd() -> Option<HWND> {
    let progman = FindWindowW(windows::core::w!("Progman"), None).ok()?;
    if progman == HWND::default() {
        return None;
    }

    // Try Progman → SHELLDLL_DefView → SysListView32
    if let Ok(def) = FindWindowExW(Some(progman), None, windows::core::w!("SHELLDLL_DefView"), None) {
        if def != HWND::default() {
            if let Ok(lv) = FindWindowExW(Some(def), None, windows::core::w!("SysListView32"), None) {
                if lv != HWND::default() {
                    return Some(lv);
                }
            }
        }
    }

    // Try WorkerW path (after 0x052C message)
    let mut prev_ww: Option<HWND> = None;
    loop {
        let ww = match FindWindowExW(None, prev_ww, windows::core::w!("WorkerW"), None) {
            Ok(h) if h != HWND::default() => h,
            _ => break,
        };
        if let Ok(def) =
            FindWindowExW(Some(ww), None, windows::core::w!("SHELLDLL_DefView"), None)
        {
            if def != HWND::default() {
                if let Ok(lv) = FindWindowExW(
                    Some(def),
                    None,
                    windows::core::w!("SysListView32"),
                    None,
                ) {
                    if lv != HWND::default() {
                        return Some(lv);
                    }
                }
            }
        }
        prev_ww = Some(ww);
    }

    None
}

fn lv_get_item_count(lv: HWND) -> i32 {
    unsafe {
        SendMessageW(lv, LVM_GETITEMCOUNT, Some(WPARAM(0)), Some(LPARAM(0))).0 as i32
    }
}

/// Read all item display names from a ListView using cross-process memory.
/// Returns a map: display_name (lowercase) → item_index.
fn lv_read_name_index_map(lv: HWND) -> HashMap<String, usize> {
    let mut map = HashMap::new();
    let count = lv_get_item_count(lv);
    if count <= 0 {
        return map;
    }

    unsafe {
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(lv, Some(&mut pid));
        if pid == 0 {
            return map;
        }

        let process = match OpenProcess(
            PROCESS_VM_OPERATION | PROCESS_VM_READ | PROCESS_VM_WRITE,
            false,
            pid,
        ) {
            Ok(h) if !h.is_invalid() => h,
            _ => return map,
        };

        const MAX_TEXT: usize = 260;
        let item_size = std::mem::size_of::<LvItemW>();
        let total = item_size + MAX_TEXT * 2; // UTF-16 chars

        let remote_buf = VirtualAllocEx(
            process,
            None,
            total,
            VIRTUAL_ALLOCATION_TYPE(0x1000 | 0x2000), // MEM_COMMIT | MEM_RESERVE
            PAGE_READWRITE,
        );

        if remote_buf.is_null() {
            let _ = CloseHandle(process);
            return map;
        }

        let text_remote_ptr = (remote_buf as usize + item_size) as u64;

        for i in 0..(count as usize) {
            let mut item: LvItemW = std::mem::zeroed();
            item.mask = LVIF_TEXT;
            item.i_item = i as i32;
            item.psz_text = text_remote_ptr;
            item.cch_text_max = MAX_TEXT as i32;

            if WriteProcessMemory(
                process,
                remote_buf,
                &item as *const _ as *const _,
                item_size,
                None,
            )
            .is_err()
            {
                continue;
            }

            SendMessageW(
                lv,
                LVM_GETITEMTEXTW,
                Some(WPARAM(i)),
                Some(LPARAM(remote_buf as isize)),
            );

            let mut text_buf = vec![0u16; MAX_TEXT];
            if ReadProcessMemory(
                process,
                text_remote_ptr as *const _,
                text_buf.as_mut_ptr() as *mut _,
                MAX_TEXT * 2,
                None,
            )
            .is_err()
            {
                continue;
            }

            let len = text_buf.iter().position(|&c| c == 0).unwrap_or(MAX_TEXT);
            let name = String::from_utf16_lossy(&text_buf[..len]);
            if !name.is_empty() {
                // Key by lowercase name for case-insensitive match
                map.insert(name.to_lowercase(), i);
            }
        }

        let _ = VirtualFreeEx(process, remote_buf, 0, MEM_RELEASE);
        let _ = CloseHandle(process);
    }

    map
}

/// Set the screen position of a ListView item (no cross-process needed).
/// Uses MAKELPARAM(x, y) which supports coordinates up to 32767 — sufficient for 8K screens.
fn lv_set_item_position(lv: HWND, idx: usize, x: i32, y: i32) {
    let lx = (x as u32) & 0xFFFF;
    let ly = (y as u32) & 0xFFFF;
    let lparam = LPARAM(((ly << 16) | lx) as isize);
    unsafe {
        SendMessageW(lv, LVM_SETITEMPOSITION, Some(WPARAM(idx)), Some(lparam));
    }
}

/// Primary monitor rectangle expressed in desktop-ListView client coordinates,
/// returned as (origin_x, origin_y, width, height).
///
/// On multi-monitor systems the desktop ListView spans the whole virtual desktop,
/// so positions are relative to the virtual top-left. The primary monitor's
/// top-left is virtual (0,0); the ListView window's screen origin tells us the
/// virtual top-left, so the primary offset in client coords = (-left, -top).
/// Confining the layout here guarantees a one-click tidy always lands on the
/// main screen, flush to its edges, regardless of secondary-monitor placement/DPI.
fn primary_area_in_lv_coords(lv: HWND) -> (i32, i32, i32, i32) {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;
    let (pw, ph) = screen_size(); // primary monitor size (SM_CXSCREEN / SM_CYSCREEN)
    unsafe {
        let mut r = RECT::default();
        if GetWindowRect(lv, &mut r).is_ok() {
            return (-r.left, -r.top, pw, ph);
        }
    }
    (0, 0, pw, ph)
}

fn lv_refresh(lv: HWND) {
    unsafe {
        SendMessageW(
            lv,
            LVM_REDRAWITEMS,
            Some(WPARAM(0)),
            Some(LPARAM(i32::MAX as isize)),
        );
        let _ = InvalidateRect(Some(lv), None, true);
    }
}

/// Clear the LVS_AUTOARRANGE bit so LVM_SETITEMPOSITION positions actually stick.
fn disable_lv_auto_arrange(lv: HWND) {
    const LVS_AUTOARRANGE: u32 = 0x0100;
    unsafe {
        let style = GetWindowLongPtrW(lv, GWL_STYLE) as u32;
        if style & LVS_AUTOARRANGE != 0 {
            SetWindowLongPtrW(lv, GWL_STYLE, (style & !LVS_AUTOARRANGE) as isize);
            log::info!("已禁用桌面 LVS_AUTOARRANGE，图标位置将持久化");
        }
    }
}

// ============================================================
// Tauri Commands
// ============================================================

/// Enumerate desktop icons from the filesystem. Fast and reliable.
/// Returns names, file types, and placeholder positions (0,0).
#[tauri::command]
pub fn enumerate_desktop_icons() -> Result<Vec<DesktopIcon>, String> {
    let desktop = dirs::desktop_dir().ok_or("无法获取桌面路径")?;
    let mut icons = Vec::new();

    if let Ok(entries) = fs::read_dir(&desktop) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let ext = path
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_lowercase();
            let is_dir = path.is_dir();
            icons.push(DesktopIcon {
                name,
                path: path.to_string_lossy().to_string(),
                ext,
                is_dir,
                x: 0,
                y: 0,
            });
        }
    }

    icons.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(icons)
}

/// 一键整理桌面 —— 经典左右分区（仿 Fences，但不绘制栅格框）。
///
/// 策略：以桌面 ListView 的真实显示名为基础分类（避免文件名 ≠ 显示名），
/// 先关闭自动排列再设坐标，否则 Windows 会把图标吸附回默认网格。
/// 左侧：应用/快捷方式/系统图标（此电脑排第一，其次其他虚拟图标，再按字母序），
///       自上而下填满一列后向右推进。
/// 右侧：文件夹 / 文档 / 图片 / 视频音频 / 其他（非应用图标）各自成区（横向带）——
///       每个类别占一条横向带（左→右填充、满则下行），各带自上而下堆叠、之间留
///       空行形成视觉分区，整体紧贴右边缘。
#[tauri::command]
pub fn organize_desktop_one_click() -> Result<OrganizeDesktopResult, String> {
    let (cell_w, cell_h) = icon_spacing();
    let margin: i32 = 8;

    // ── Find desktop ListView ─────────────────────────────────
    let lv = unsafe { find_desktop_listview_hwnd() }
        .ok_or("找不到桌面图标列表，请确保桌面可见")?;

    // Disable auto-arrange FIRST. When LVS_AUTOARRANGE is set, LVM_SETITEMPOSITION
    // is silently ignored and icons snap back to the Windows grid.
    disable_lv_auto_arrange(lv);

    // Confine the layout to the PRIMARY monitor (main screen), in ListView
    // client coordinates. (ox, oy) shifts the primary's top-left when the
    // desktop ListView spans multiple monitors.
    let (ox, oy, area_w, area_h) = primary_area_in_lv_coords(lv);
    let rows_per_col = ((area_h - margin * 2) / cell_h).max(1);
    log::info!(
        "桌面整理区域(主屏, LV坐标): origin=({},{}) size={}x{} rows_per_col={}",
        ox, oy, area_w, area_h, rows_per_col
    );

    // ── Read LV names (what Windows actually shows) ──────────
    let lv_name_map = lv_read_name_index_map(lv);
    if lv_name_map.is_empty() {
        return Err("无法读取桌面图标列表，可能需要管理员权限".into());
    }

    // ── Build stem/name → category map from filesystem ────────
    // "stem" = filename without extension, matches what LV shows when
    // "Hide extensions for known file types" is enabled.
    let mut stem_cat: HashMap<String, &str> = HashMap::new();
    if let Some(Ok(rd)) = dirs::desktop_dir().map(fs::read_dir) {
        for e in rd.flatten() {
            let path = e.path();
            let cat = icon_category(&path);
            let full = e.file_name().to_string_lossy().to_lowercase();
            stem_cat.insert(full.clone(), cat);
            if let Some(st) = std::path::Path::new(&full).file_stem() {
                let s = st.to_string_lossy().to_lowercase();
                stem_cat.entry(s).or_insert(cat);
            }
        }
    }

    // Is this LV item a virtual system icon (not present on filesystem)?
    // Covers 此电脑 / 回收站 / 网络 / 控制面板 etc.
    let is_virtual = |name: &str| -> bool {
        !stem_cat.contains_key(name) && !stem_cat.contains_key(&format!("{}.lnk", name) as &str)
    };
    // Is this the "This PC / 此电脑 / 计算机" icon? Use contains() to survive locale variations.
    let is_this_pc = |name: &str| -> bool {
        let lc = name.to_lowercase();
        lc.contains("电脑") || lc.contains("计算机") || lc == "this pc" || lc == "my computer"
    };

    // ── Sort: 此电脑 → 其他虚拟图标 → 普通项（字母序） ──────────
    let mut sorted_names: Vec<String> = lv_name_map.keys().cloned().collect();
    sorted_names.sort_by(|a, b| {
        let a_pc = is_this_pc(a);
        let b_pc = is_this_pc(b);
        let a_virt = !a_pc && is_virtual(a);
        let b_virt = !b_pc && is_virtual(b);
        if a_pc && !b_pc {
            return std::cmp::Ordering::Less;
        }
        if b_pc && !a_pc {
            return std::cmp::Ordering::Greater;
        }
        if a_virt && !b_virt {
            return std::cmp::Ordering::Less;
        }
        if b_virt && !a_virt {
            return std::cmp::Ordering::Greater;
        }
        a.cmp(b)
    });

    // ── Classify ALL LV items ─────────────────────────────────
    let mut app_idxs: Vec<usize> = Vec::new(); // apps/shortcuts/virtual icons → left
    let mut folder_idxs: Vec<usize> = Vec::new(); // user folders → right
    let mut doc_idxs: Vec<usize> = Vec::new(); // documents → right
    let mut img_idxs: Vec<usize> = Vec::new(); // images → right (own zone)
    let mut vid_idxs: Vec<usize> = Vec::new(); // video/audio → right
    let mut other_idxs: Vec<usize> = Vec::new(); // other files → right

    for name in &sorted_names {
        let &idx = lv_name_map.get(name).unwrap();
        // Virtual icons (此电脑, 回收站, 网络…) are not on the filesystem → "apps".
        let cat = stem_cat
            .get(name.as_str())
            .or_else(|| stem_cat.get(&format!("{}.lnk", name) as &str))
            .copied()
            .unwrap_or("apps");

        match cat {
            "folders" => folder_idxs.push(idx),
            "docs" => doc_idxs.push(idx),
            "images" => img_idxs.push(idx),
            "media" => vid_idxs.push(idx),
            "apps" => app_idxs.push(idx),
            _ => other_idxs.push(idx),
        }
    }

    let icon_count = lv_name_map.len() as u32;

    // ── LEFT: apps, fill a column top-to-bottom then advance right ──
    for (slot, &idx) in app_idxs.iter().enumerate() {
        let s = slot as i32;
        let x = ox + margin + (s / rows_per_col) * cell_w;
        let y = oy + margin + (s % rows_per_col) * cell_h;
        lv_set_item_position(lv, idx, x, y);
    }

    // ── RIGHT: non-app icons split into per-category zones (row bands) ──
    // Each category is a horizontal band `zone_cols` wide, filled left→right then
    // wrapping down; bands stack top→bottom separated by one empty row so they
    // read as distinct zones (分区) without drawing any overlay. The whole block
    // hugs the right edge.
    let zone_cols = 3i32;
    let zone_x0 = ox + (area_w - margin - zone_cols * cell_w).max(margin); // left edge of zone
    let gap_rows = 2i32; // empty rows between category zones (clear visual separation)

    // Place one category as a row band starting at `start_row`. Returns the
    // number of rows the band consumed.
    let place_zone = |indices: &[usize], start_row: i32| -> i32 {
        for (slot, &idx) in indices.iter().enumerate() {
            let s = slot as i32;
            let col = s % zone_cols;
            let row = start_row + s / zone_cols;
            let x = zone_x0 + col * cell_w;
            let y = oy + margin + row * cell_h;
            lv_set_item_position(lv, idx, x, y);
        }
        if indices.is_empty() {
            0
        } else {
            (indices.len() as i32 + zone_cols - 1) / zone_cols
        }
    };

    let mut row_cursor = 0i32;
    for zone in [&folder_idxs, &doc_idxs, &img_idxs, &vid_idxs, &other_idxs] {
        if zone.is_empty() {
            continue;
        }
        let used = place_zone(zone, row_cursor);
        row_cursor += used + gap_rows;
    }

    lv_refresh(lv);

    log::info!(
        "桌面整理: total={} apps(left)={} folders={} docs={} images={} video={} other={} rows_per_col={}",
        icon_count,
        app_idxs.len(),
        folder_idxs.len(),
        doc_idxs.len(),
        img_idxs.len(),
        vid_idxs.len(),
        other_idxs.len(),
        rows_per_col
    );

    Ok(OrganizeDesktopResult {
        arranged: icon_count,
        skipped: 0,
        icon_count,
    })
}

#[tauri::command]
pub fn hide_desktop_icons() -> Result<(), String> {
    unsafe {
        if let Some(shell_view) = find_shell_view() {
            let _ = ShowWindow(shell_view, SW_HIDE);
        }
    }
    Ok(())
}

#[tauri::command]
pub fn show_desktop_icons() -> Result<(), String> {
    unsafe {
        if let Some(shell_view) = find_shell_view() {
            let _ = ShowWindow(shell_view, SW_SHOW);
        }
    }
    Ok(())
}

// ============================================================
// Private helpers
// ============================================================

fn screen_size() -> (i32, i32) {
    unsafe {
        use windows::Win32::UI::WindowsAndMessaging::{SM_CXSCREEN, SM_CYSCREEN};
        (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN))
    }
}

fn icon_spacing() -> (i32, i32) {
    unsafe {
        let cw = GetSystemMetrics(SM_CXICONSPACING).max(72);
        let ch = GetSystemMetrics(SM_CYICONSPACING).max(72);
        (cw, ch)
    }
}

fn icon_category(path: &std::path::Path) -> &'static str {
    if path.is_dir() {
        return "folders";
    }
    let ext = path
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();
    match ext.as_str() {
        "doc" | "docx" | "pdf" | "txt" | "xlsx" | "pptx" | "xls" | "ppt" | "md" | "rtf" => "docs",
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg" | "ico" => "images",
        "mp4" | "avi" | "mkv" | "mov" | "webm" | "mp3" | "wav" => "media",
        "exe" | "lnk" | "url" | "msi" => "apps",
        _ => "other",
    }
}

unsafe fn find_shell_view() -> Option<HWND> {
    let progman = FindWindowW(windows::core::w!("Progman"), None).ok()?;
    if let Ok(def) =
        FindWindowExW(Some(progman), None, windows::core::w!("SHELLDLL_DefView"), None)
    {
        if def != HWND::default() {
            return Some(def);
        }
    }
    // Try WorkerW
    let mut prev_ww: Option<HWND> = None;
    loop {
        let ww = match FindWindowExW(None, prev_ww, windows::core::w!("WorkerW"), None) {
            Ok(h) if h != HWND::default() => h,
            _ => break,
        };
        if let Ok(def) =
            FindWindowExW(Some(ww), None, windows::core::w!("SHELLDLL_DefView"), None)
        {
            if def != HWND::default() {
                return Some(def);
            }
        }
        prev_ww = Some(ww);
    }
    None
}
