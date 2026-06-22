//! Desktop organizer — left: apps in tight columns; right: file fences with overlay.

#![cfg(target_os = "windows")]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
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
    SendMessageW, SetWindowLongPtrW, ShowWindow, SW_HIDE, SW_SHOW,
    GWL_EXSTYLE, GWL_STYLE,
    SM_CXICONSPACING, SM_CYICONSPACING, WS_EX_LAYERED, WS_EX_TRANSPARENT,
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
pub struct PartitionInfo {
    pub id: String,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub color: String,
    pub opacity: f32,
    pub collapsed: bool,
    pub icon_count: u32,
    #[serde(default)]
    pub icon_names: Vec<String>,
}

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
pub struct PartitionLayout {
    pub partitions: Vec<PartitionInfo>,
}

/// A visual fence rectangle rendered in the overlay window.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FenceSpec {
    pub label: String,
    pub color: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArrangeResult {
    pub arranged: u32,
    pub skipped: u32,
    pub partitions_processed: u32,
}

/// Shared state that the fence-overlay WebView reads on load.
pub struct FenceOverlayState(pub Mutex<Vec<FenceSpec>>);

impl Default for FenceOverlayState {
    fn default() -> Self {
        Self(Mutex::new(Vec::new()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizeDesktopResult {
    pub arranged: u32,
    pub skipped: u32,
    pub icon_count: u32,
    pub fences: Vec<FenceSpec>,
}

pub struct OrganizerState {
    pub layout: Mutex<PartitionLayout>,
}

impl Default for OrganizerState {
    fn default() -> Self {
        Self {
            layout: Mutex::new(PartitionLayout {
                partitions: Vec::new(),
            }),
        }
    }
}

fn layout_path(app_data: &PathBuf) -> PathBuf {
    app_data.join("partition_layout.json")
}

// ============================================================
// ListView cross-process helpers
// ============================================================

/// LVITEMW structure layout for 64-bit Windows.
/// Manually defined to guarantee correct alignment without needing Win32_UI_Controls feature.
#[repr(C)]
struct LvItemW {
    mask: u32,       // offset 0
    i_item: i32,     // offset 4
    i_sub_item: i32, // offset 8
    state: u32,      // offset 12
    state_mask: u32, // offset 16
    _pad1: u32,      // offset 20 — padding before 8-byte pointer
    psz_text: u64,   // offset 24 — LPWSTR (pointer in remote process)
    cch_text_max: i32, // offset 32
    i_image: i32,    // offset 36
    l_param: i64,    // offset 40
    i_indent: i32,   // offset 48
    i_group_id: i32, // offset 52
    c_columns: u32,  // offset 56
    _pad2: u32,      // offset 60
    pui_columns: u64, // offset 64
    pi_col_fmt: u64, // offset 72
    i_group: i32,    // offset 80
    _pad3: u32,      // offset 84
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

#[tauri::command]
pub fn create_partition(
    state: tauri::State<'_, OrganizerState>,
    name: String,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    color: Option<String>,
    opacity: Option<f32>,
) -> Result<PartitionInfo, String> {
    let id = format!(
        "part_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let partition = PartitionInfo {
        id: id.clone(),
        name,
        x,
        y,
        w,
        h,
        color: color.unwrap_or_else(|| "#008336".into()),
        opacity: opacity.unwrap_or(0.18),
        collapsed: false,
        icon_count: 0,
        icon_names: Vec::new(),
    };

    let mut guard = state.layout.lock().map_err(|e| e.to_string())?;
    guard.partitions.push(partition.clone());
    Ok(partition)
}

#[tauri::command]
pub fn delete_partition(
    state: tauri::State<'_, OrganizerState>,
    partition_id: String,
) -> Result<(), String> {
    let mut guard = state.layout.lock().map_err(|e| e.to_string())?;
    guard.partitions.retain(|p| p.id != partition_id);
    Ok(())
}

#[tauri::command]
pub fn update_partition(
    state: tauri::State<'_, OrganizerState>,
    partition_id: String,
    name: Option<String>,
    x: Option<i32>,
    y: Option<i32>,
    w: Option<i32>,
    h: Option<i32>,
    color: Option<String>,
    opacity: Option<f32>,
    collapsed: Option<bool>,
    icon_names: Option<Vec<String>>,
) -> Result<(), String> {
    let mut guard = state.layout.lock().map_err(|e| e.to_string())?;
    if let Some(p) = guard.partitions.iter_mut().find(|p| p.id == partition_id) {
        if let Some(v) = name {
            p.name = v;
        }
        if let Some(v) = x {
            p.x = v;
        }
        if let Some(v) = y {
            p.y = v;
        }
        if let Some(v) = w {
            p.w = v;
        }
        if let Some(v) = h {
            p.h = v;
        }
        if let Some(v) = color {
            p.color = v;
        }
        if let Some(v) = opacity {
            p.opacity = v;
        }
        if let Some(v) = collapsed {
            p.collapsed = v;
        }
        if let Some(v) = icon_names {
            p.icon_count = v.len() as u32;
            p.icon_names = v;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn move_icon_to_partition(
    state: tauri::State<'_, OrganizerState>,
    icon_name: String,
    partition_id: String,
) -> Result<(), String> {
    let mut guard = state.layout.lock().map_err(|e| e.to_string())?;

    // Remove from any existing partition
    for p in guard.partitions.iter_mut() {
        p.icon_names.retain(|n| n != &icon_name);
        p.icon_count = p.icon_names.len() as u32;
    }

    // Add to target partition
    if let Some(p) = guard.partitions.iter_mut().find(|p| p.id == partition_id) {
        p.icon_names.push(icon_name);
        p.icon_count = p.icon_names.len() as u32;
    }
    Ok(())
}

#[tauri::command]
pub fn get_partition_layout(
    state: tauri::State<'_, OrganizerState>,
) -> Result<PartitionLayout, String> {
    let guard = state.layout.lock().map_err(|e| e.to_string())?;
    Ok(guard.clone())
}

#[tauri::command]
pub fn save_partition_layout(
    app: tauri::AppHandle,
    state: tauri::State<'_, OrganizerState>,
) -> Result<(), String> {
    let guard = state.layout.lock().map_err(|e| e.to_string())?;
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取数据目录失败: {}", e))?;
    fs::create_dir_all(&app_data).map_err(|e| format!("创建目录失败: {}", e))?;

    let json = serde_json::to_string_pretty(&*guard).map_err(|e| format!("序列化失败: {}", e))?;
    fs::write(layout_path(&app_data), json).map_err(|e| format!("保存布局失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn load_partition_layout(
    app: tauri::AppHandle,
    state: tauri::State<'_, OrganizerState>,
) -> Result<PartitionLayout, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取数据目录失败: {}", e))?;
    let path = layout_path(&app_data);

    if path.exists() {
        let json = fs::read_to_string(&path).map_err(|e| format!("读取布局失败: {}", e))?;
        let layout: PartitionLayout =
            serde_json::from_str(&json).map_err(|e| format!("解析布局失败: {}", e))?;
        let mut guard = state.layout.lock().map_err(|e| e.to_string())?;
        *guard = layout.clone();
        Ok(layout)
    } else {
        let guard = state.layout.lock().map_err(|e| e.to_string())?;
        Ok(guard.clone())
    }
}

/// 一键整理桌面
/// 策略：以 LV 真实名单为基础分类（避免文件名 ≠ 显示名），先关闭自动排列再设坐标
/// 左侧：应用/快捷方式/文件夹/系统图标，上到下连续无空隙
/// 右侧：文件类按类型放入带标题栅格
#[tauri::command]
pub fn organize_desktop_one_click(
    app: tauri::AppHandle,
    fence_state: tauri::State<'_, FenceOverlayState>,
) -> Result<OrganizeDesktopResult, String> {
    let (cell_w, cell_h) = icon_spacing();
    let (screen_w, screen_h) = screen_size();
    let margin: i32 = 8;
    let rows_per_col = ((screen_h - margin) / cell_h).max(1);

    // ── Fence geometry ───────────────────────────────────────
    let fence_cols = 3i32;
    let fence_w    = fence_cols * cell_w;
    let fence_x    = screen_w - fence_w - margin;
    let title_h    = 28i32;
    let pad        = 6i32;
    let fence_gap  = 12i32;

    fn fence_h(n: usize, title_h: i32, pad: i32, cell_h: i32, cols: i32) -> i32 {
        if n == 0 { return 0; }
        title_h + pad + ((n as i32 - 1) / cols + 1) * cell_h + pad
    }

    // ── Find desktop ListView ─────────────────────────────────
    let lv = unsafe { find_desktop_listview_hwnd() }
        .ok_or("找不到桌面图标列表，请确保桌面可见")?;

    // Disable auto-arrange FIRST. When LVS_AUTOARRANGE is set, LVM_SETITEMPOSITION
    // is silently ignored and icons snap back to the Windows grid. We clear this flag
    // so our positions actually stick.
    disable_lv_auto_arrange(lv);

    // ── Read LV names (what Windows actually shows) ──────────
    let lv_name_map = lv_read_name_index_map(lv);
    if lv_name_map.is_empty() {
        return Err("无法读取桌面图标列表，可能需要管理员权限".into());
    }

    // ── Build stem/name → category map from filesystem ────────
    // "stem" = filename without extension, matches what LV shows when
    // "Hide extensions for known file types" is enabled.
    let mut stem_cat: HashMap<String, &str> = HashMap::new();
    if let Some(Ok(rd)) = dirs::desktop_dir().map(|d| fs::read_dir(d)) {
        for e in rd.flatten() {
            let path = e.path();
            let cat  = icon_category(&path);
            let full = e.file_name().to_string_lossy().to_lowercase();
            stem_cat.insert(full.clone(), cat);
            if let Some(st) = std::path::Path::new(&full).file_stem() {
                let s = st.to_string_lossy().to_lowercase();
                stem_cat.entry(s).or_insert(cat);
            }
        }
    }

    // ── Classify ALL LV items ─────────────────────────────────
    let mut sorted_names: Vec<String> = lv_name_map.keys().cloned().collect();

    // Log actual LV names to help debug locale/encoding issues.
    log::info!("桌面 LV 图标列表: {:?}", sorted_names);

    // Helper: is this LV item a virtual system icon (not present on filesystem)?
    // Covers: 此电脑, 回收站, 网络, 控制面板, etc.
    let is_virtual = |name: &str| -> bool {
        !stem_cat.contains_key(name)
            && !stem_cat.contains_key(&format!("{}.lnk", name) as &str)
    };

    // Helper: is this the "This PC / 此电脑 / 计算机" icon?
    // We use contains() rather than == to survive minor locale/encoding variations.
    let is_this_pc = |name: &str| -> bool {
        let lc = name.to_lowercase();
        lc.contains("电脑") || lc.contains("计算机")
            || lc == "this pc" || lc == "my computer"
    };

    sorted_names.sort_by(|a, b| {
        let a_pc   = is_this_pc(a);
        let b_pc   = is_this_pc(b);
        let a_virt = !a_pc && is_virtual(a);
        let b_virt = !b_pc && is_virtual(b);

        // Priority: 此电脑  >  other virtual icons  >  regular apps (alphabetical)
        if a_pc   && !b_pc   { return std::cmp::Ordering::Less; }
        if b_pc   && !a_pc   { return std::cmp::Ordering::Greater; }
        if a_virt && !b_virt { return std::cmp::Ordering::Less; }
        if b_virt && !a_virt { return std::cmp::Ordering::Greater; }
        a.cmp(b)
    });

    let mut app_idxs:    Vec<usize> = Vec::new(); // apps/shortcuts/virtual icons (left)
    let mut folder_idxs: Vec<usize> = Vec::new(); // user folders          (right fence)
    let mut doc_idxs:    Vec<usize> = Vec::new(); // documents             (right fence)
    let mut img_idxs:    Vec<usize> = Vec::new(); // images/video          (right fence)
    let mut other_idxs:  Vec<usize> = Vec::new(); // other files           (right fence)

    for name in &sorted_names {
        let &idx = lv_name_map.get(name).unwrap();
        // Look up category via filesystem stem map.
        // Virtual icons (此电脑, 回收站, 网络…) not in filesystem → "apps".
        let cat = stem_cat.get(name.as_str())
            .or_else(|| stem_cat.get(&format!("{}.lnk", name) as &str))
            .copied()
            .unwrap_or("apps");

        match cat {
            "folders" => folder_idxs.push(idx), // user folders → right side
            "apps"    => app_idxs.push(idx),
            "docs"    => doc_idxs.push(idx),
            "media"   => img_idxs.push(idx),
            _         => other_idxs.push(idx),
        }
    }

    let icon_count = lv_name_map.len() as u32;

    // ── Compute fence positions (right side, stacked top-to-bottom) ───
    let folder_h = fence_h(folder_idxs.len(), title_h, pad, cell_h, fence_cols);
    let doc_h    = fence_h(doc_idxs.len(),    title_h, pad, cell_h, fence_cols);
    let img_h    = fence_h(img_idxs.len(),    title_h, pad, cell_h, fence_cols);
    let other_h  = fence_h(other_idxs.len(),  title_h, pad, cell_h, fence_cols);

    let folder_y = margin;
    let doc_y    = folder_y + if !folder_idxs.is_empty() { folder_h + fence_gap } else { 0 };
    let img_y    = doc_y    + if !doc_idxs.is_empty()    { doc_h    + fence_gap } else { 0 };
    let other_y  = img_y    + if !img_idxs.is_empty()    { img_h    + fence_gap } else { 0 };

    // ── Place icons via LVM_SETITEMPOSITION ───────────────────
    // col_first = true  → fill column top-to-bottom, then advance right (apps)
    // col_first = false → fill row left-to-right, then advance down   (fence files)
    let place_all = |indices: &[usize], ox: i32, oy: i32, cols: i32, col_first: bool| {
        for (slot, &idx) in indices.iter().enumerate() {
            let s = slot as i32;
            let (cx, cy) = if col_first {
                (ox + (s / rows_per_col) * cell_w, oy + (s % rows_per_col) * cell_h)
            } else {
                (ox + (s % cols) * cell_w, oy + (s / cols) * cell_h)
            };
            lv_set_item_position(lv, idx, cx, cy);
        }
    };

    place_all(&app_idxs,    margin,        margin,               8,          true);
    place_all(&folder_idxs, fence_x + pad, folder_y + title_h + pad, fence_cols, false);
    place_all(&doc_idxs,    fence_x + pad, doc_y    + title_h + pad, fence_cols, false);
    place_all(&img_idxs,    fence_x + pad, img_y    + title_h + pad, fence_cols, false);
    place_all(&other_idxs,  fence_x + pad, other_y  + title_h + pad, fence_cols, false);

    lv_refresh(lv);

    let arranged = icon_count;
    let skipped  = 0u32;

    // ── Build fence specs ────────────────────────────────────
    let mut fences = Vec::new();
    if !folder_idxs.is_empty() {
        fences.push(FenceSpec {
            label: "文件夹".into(), color: "#008336".into(),
            x: fence_x, y: folder_y, w: fence_w, h: folder_h,
        });
    }
    if !doc_idxs.is_empty() {
        fences.push(FenceSpec {
            label: "文档".into(), color: "#068d9a".into(),
            x: fence_x, y: doc_y, w: fence_w, h: doc_h,
        });
    }
    if !img_idxs.is_empty() {
        fences.push(FenceSpec {
            label: "图片与视频".into(), color: "#aa6300".into(),
            x: fence_x, y: img_y, w: fence_w, h: img_h,
        });
    }
    if !other_idxs.is_empty() {
        fences.push(FenceSpec {
            label: "其他文件".into(), color: "#64748b".into(),
            x: fence_x, y: other_y, w: fence_w, h: other_h,
        });
    }

    log::info!(
        "桌面整理: total={} apps={} folders={} docs={} imgs={} other={} rows_per_col={}",
        icon_count, app_idxs.len(), folder_idxs.len(), doc_idxs.len(), img_idxs.len(),
        other_idxs.len(), rows_per_col
    );

    *fence_state.0.lock().map_err(|e| e.to_string())? = fences.clone();
    notify_fence_overlay(&app, &fences);

    Ok(OrganizeDesktopResult { arranged, skipped, icon_count, fences })
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

/// Return current fence layout to the overlay WebView when it loads.
#[tauri::command]
pub fn get_fence_data(
    state: tauri::State<'_, FenceOverlayState>,
) -> Vec<FenceSpec> {
    state.0.lock().map(|g| g.clone()).unwrap_or_default()
}

/// Called by the fence overlay Vue component once it has finished rendering fences.
/// The window was created at startup but hidden; we only show it after Vue has painted.
#[tauri::command]
pub fn show_fence_overlay(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("fence-overlay") {
        w.show().map_err(|e| format!("显示栅格叠加层失败: {}", e))?;
    }
    Ok(())
}

/// Called by the fence overlay Vue component to hide itself (no fences to show).
#[tauri::command]
pub fn hide_fence_overlay(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("fence-overlay") {
        let _ = w.hide();
    }
    Ok(())
}

/// Hide and clear the fence overlay.
#[tauri::command]
pub fn clear_fence_overlay(
    app: tauri::AppHandle,
    state: tauri::State<'_, FenceOverlayState>,
) -> Result<(), String> {
    *state.0.lock().map_err(|e| e.to_string())? = Vec::new();
    if let Some(w) = app.get_webview_window("fence-overlay") {
        let _ = w.hide();
    }
    Ok(())
}

/// 仅向已存在的叠加层窗口发送栅格数据，窗口本身在启动时预创建
fn notify_fence_overlay(app: &tauri::AppHandle, fences: &[FenceSpec]) {
    if let Some(w) = app.get_webview_window("fence-overlay") {
        let _ = w.emit("fence-update", fences);
    }
}

/// 在 app 启动时（主线程）预创建栅格叠加层窗口，与桌面播放器窗口同样的模式
pub fn init_fence_overlay(app: &tauri::AppHandle) {
    let (sw, sh) = screen_size();

    let result = WebviewWindowBuilder::new(
        app,
        "fence-overlay",
        WebviewUrl::App("/#/fence-overlay".into()),
    )
    .title("LingScape Fence Overlay")
    .inner_size(sw as f64, sh as f64)
    .position(0.0, 0.0)
    .decorations(false)
    .resizable(false)
    .always_on_bottom(false)
    .skip_taskbar(true)
    .focusable(false)
    .visible(false)
    .transparent(true)
    .build();

    match result {
        Ok(w) => {
            // Make it click-through so desktop icons remain interactive.
            if let Ok(hwnd) = w.hwnd() {
                unsafe {
                    let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
                    SetWindowLongPtrW(
                        hwnd,
                        GWL_EXSTYLE,
                        (ex | WS_EX_TRANSPARENT.0 | WS_EX_LAYERED.0) as isize,
                    );
                }
            }
            log::info!("栅格叠加层窗口已预创建（隐藏）");
        }
        Err(e) => log::warn!("预创建栅格叠加层失败: {}", e),
    }
}


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
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg" | "ico"
        | "mp4" | "avi" | "mkv" | "mov" | "webm" | "mp3" | "wav" => "media",
        "exe" | "lnk" | "url" | "msi" => "apps",
        _ => "other",
    }
}

/// Auto-assign icons to partitions based on file extension.
#[tauri::command]
pub fn auto_assign_icons(
    state: tauri::State<'_, OrganizerState>,
) -> Result<u32, String> {
    let desktop = dirs::desktop_dir().ok_or("无法获取桌面路径")?;
    let entries: Vec<_> = fs::read_dir(&desktop)
        .map_err(|e| format!("读取桌面目录失败: {}", e))?
        .flatten()
        .collect();

    let mut guard = state.layout.lock().map_err(|e| e.to_string())?;
    if guard.partitions.is_empty() {
        return Err("请先创建分区".into());
    }

    // Clear existing assignments
    for p in guard.partitions.iter_mut() {
        p.icon_names.clear();
        p.icon_count = 0;
    }

    let part_count = guard.partitions.len();
    let mut assigned = 0u32;

    for entry in &entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let ext = path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();

        // Simple type-based routing
        let target_idx: usize = if path.is_dir() {
            0 // first partition gets folders
        } else {
            match ext.as_str() {
                "doc" | "docx" | "pdf" | "txt" | "xlsx" | "pptx" | "xls" | "ppt" => {
                    1 % part_count
                }
                "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg" => 2 % part_count,
                "mp4" | "avi" | "mkv" | "mov" | "webm" => 3 % part_count,
                "zip" | "rar" | "7z" | "tar" | "gz" => (part_count - 1).min(4),
                "exe" | "lnk" | "url" => 0,
                _ => part_count.saturating_sub(1), // last partition for misc
            }
        };

        if let Some(p) = guard.partitions.get_mut(target_idx) {
            p.icon_names.push(name);
            p.icon_count += 1;
            assigned += 1;
        }
    }

    Ok(assigned)
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
