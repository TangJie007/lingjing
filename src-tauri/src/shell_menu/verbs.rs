use std::mem::ManuallyDrop;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_APARTMENTTHREADED,
};
use windows::Win32::System::Variant::{VARIANT, VT_BSTR, VT_I4};
use windows::Win32::UI::Shell::{
    Folder2, FolderItem, FolderItemVerb, GCS_VERBA, IContextMenu, IShellDispatch, Shell,
    ShellExecuteExW, ShellExecuteW, SHFileOperationW, FO_DELETE, FOF_ALLOWUNDO,
    FOF_WANTNUKEWARNING, SEE_MASK_ASYNCOK, SEE_MASK_FLAG_NO_UI, SHELLEXECUTEINFOW, SHFILEOPSTRUCTW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AllowSetForegroundWindow, BringWindowToTop, DispatchMessageW, EnumWindows, GetClassNameW,
    GetWindowTextW, IsWindowVisible, PeekMessageW, SetForegroundWindow, ShowWindow, TranslateMessage,
    MSG, PM_REMOVE, SW_RESTORE, SW_SHOWNORMAL,
};
use windows::core::{BSTR, BOOL, Interface, PCWSTR};

use super::ids::CMD_FIRST;
use super::util::wide;

struct PropJob {
    path: String,
    done: Sender<Result<(), String>>,
}

static PROP_TX: OnceLock<Sender<PropJob>> = OnceLock::new();

pub(crate) fn command_verb(pcm: &IContextMenu, command_id: u32) -> Option<String> {
    if command_id < CMD_FIRST {
        return None;
    }
    let offset = (command_id - CMD_FIRST) as usize;
    let mut buf = [0u8; 128];
    unsafe {
        if pcm
            .GetCommandString(
                offset,
                GCS_VERBA,
                None,
                windows::core::PSTR(buf.as_mut_ptr()),
                buf.len() as u32,
            )
            .is_ok()
        {
            let len = buf.iter().position(|&c| c == 0).unwrap_or(0);
            if len > 0 {
                return Some(String::from_utf8_lossy(&buf[..len]).to_ascii_lowercase());
            }
        }
    }
    None
}

/// Open the native Explorer property sheet.
///
/// The sheet is owned by whichever thread calls `FolderItemVerb::DoIt` and
/// dies when that thread stops pumping messages. Explorer looks "process-free"
/// only because `explorer.exe` already pumps forever. We do the same on a
/// thread inside this app, so no PowerShell helper is spawned.
pub fn show_item_properties(path: &str) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("路径为空".into());
    }
    unsafe {
        let _ = AllowSetForegroundWindow(u32::MAX);
    }
    let tx = prop_sender();
    let (done_tx, done_rx) = mpsc::channel();
    tx.send(PropJob {
        path: trimmed.to_string(),
        done: done_tx,
    })
    .map_err(|_| "属性线程已退出".to_string())?;
    done_rx
        .recv_timeout(Duration::from_secs(20))
        .map_err(|_| "打开属性超时".to_string())?
}

fn prop_sender() -> &'static Sender<PropJob> {
    PROP_TX.get_or_init(|| {
        let (tx, rx) = mpsc::channel();
        std::thread::Builder::new()
            .name("lingscape-properties".into())
            .spawn(move || properties_sta_loop(rx))
            .expect("属性线程启动失败");
        tx
    })
}

fn properties_sta_loop(rx: Receiver<PropJob>) {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    }
    loop {
        while let Ok(job) = rx.try_recv() {
            let result = open_properties_com(&job.path);
            let _ = job.done.send(result);
        }
        pump_pending();
        std::thread::sleep(Duration::from_millis(16));
    }
}

fn pump_pending() {
    unsafe {
        let mut msg = MSG::default();
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

fn open_properties_com(path: &str) -> Result<(), String> {
    let item = shell_item(path)?;
    let verbs = unsafe { item.Verbs() }.map_err(|e| format!("读取菜单失败: {e}"))?;
    let count = unsafe { verbs.Count() }.map_err(|e| format!("读取菜单失败: {e}"))?;
    let mut opened = false;
    for i in 0..count {
        let Some(verb) = verb_at(&verbs, i) else {
            continue;
        };
        let name = unsafe { verb.Name() }
            .map(|b| b.to_string())
            .unwrap_or_default();
        let lower = name.to_ascii_lowercase();
        if name.contains('\u{5C5E}') || lower.contains("propert") {
            unsafe { verb.DoIt() }.map_err(|e| format!("打开属性失败: {e}"))?;
            opened = true;
            break;
        }
    }
    if !opened {
        return Err("该项目没有属性菜单".into());
    }
    raise_properties_window(path);
    tracing::info!("[shell-menu] properties shown path={path}");
    Ok(())
}

fn verb_at(
    verbs: &windows::Win32::UI::Shell::FolderItemVerbs,
    index: i32,
) -> Option<FolderItemVerb> {
    let key = variant_i4(index);
    unsafe { verbs.Item(&key) }.ok()
}

fn shell_item(path: &str) -> Result<FolderItem, String> {
    let shell: IShellDispatch =
        unsafe { CoCreateInstance(&Shell, None, CLSCTX_ALL) }.map_err(|e| format!("Shell: {e}"))?;
    if path.starts_with("::") {
        let folder = unsafe { shell.NameSpace(&variant_bstr(path)) }
            .map_err(|e| format!("无法打开该项所在位置: {e}"))?;
        let folder2: Folder2 = folder
            .cast()
            .map_err(|e| format!("无法打开该项: {e}"))?;
        return unsafe { folder2.Self_() }.map_err(|e| format!("无法打开该项: {e}"));
    }
    let pb = std::path::Path::new(path);
    let parent = pb
        .parent()
        .and_then(|p| p.to_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "无法打开该项所在位置".to_string())?;
    let name = pb
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "找不到该文件".to_string())?;
    let folder = unsafe { shell.NameSpace(&variant_bstr(parent)) }
        .map_err(|e| format!("无法打开该项所在位置: {e}"))?;
    unsafe { folder.ParseName(&BSTR::from(name)) }.map_err(|e| format!("找不到该文件: {e}"))
}

fn variant_i4(value: i32) -> VARIANT {
    let mut v = VARIANT::default();
    unsafe {
        let head = &mut *v.Anonymous.Anonymous;
        head.vt = VT_I4;
        head.Anonymous.lVal = value;
    }
    v
}

fn variant_bstr(value: &str) -> VARIANT {
    let mut v = VARIANT::default();
    unsafe {
        let head = &mut *v.Anonymous.Anonymous;
        head.vt = VT_BSTR;
        head.Anonymous.bstrVal = ManuallyDrop::new(BSTR::from(value));
    }
    v
}

struct RaiseState {
    needle: String,
    found: HWND,
}

unsafe extern "system" fn enum_prop_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let state = &mut *(lparam.0 as *mut RaiseState);
    if !IsWindowVisible(hwnd).as_bool() {
        return BOOL(1);
    }
    let mut class = [0u16; 64];
    let n = GetClassNameW(hwnd, &mut class);
    if n <= 0 || String::from_utf16_lossy(&class[..n as usize]) != "#32770" {
        return BOOL(1);
    }
    let mut title = [0u16; 512];
    let tn = GetWindowTextW(hwnd, &mut title);
    if tn <= 0 {
        return BOOL(1);
    }
    let title_s = String::from_utf16_lossy(&title[..tn as usize]);
    if !state.needle.is_empty()
        && title_s
            .to_ascii_lowercase()
            .contains(&state.needle.to_ascii_lowercase())
    {
        state.found = hwnd;
        return BOOL(0);
    }
    BOOL(1)
}

fn raise_properties_window(path: &str) {
    let needle = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();
    if needle.is_empty() {
        return;
    }
    let deadline = Instant::now() + Duration::from_millis(1500);
    while Instant::now() < deadline {
        let mut state = RaiseState {
            needle: needle.clone(),
            found: HWND::default(),
        };
        unsafe {
            let _ = EnumWindows(
                Some(enum_prop_window),
                LPARAM(&mut state as *mut RaiseState as isize),
            );
            if !state.found.is_invalid() {
                let _ = ShowWindow(state.found, SW_RESTORE);
                let _ = BringWindowToTop(state.found);
                let _ = SetForegroundWindow(state.found);
                return;
            }
        }
        pump_pending();
        std::thread::sleep(Duration::from_millis(40));
    }
}

pub(crate) fn shell_execute_verb(path: &str, verb: &str) -> Result<(), String> {
    if verb.eq_ignore_ascii_case("properties") {
        return show_item_properties(path);
    }

    let wpath = wide(path);
    let wverb = wide(verb);
    let is_runas = verb.eq_ignore_ascii_case("runas");

    if is_runas {
        unsafe {
            let mut info = SHELLEXECUTEINFOW {
                cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
                fMask: Default::default(),
                lpVerb: PCWSTR(wverb.as_ptr()),
                lpFile: PCWSTR(wpath.as_ptr()),
                nShow: SW_SHOWNORMAL.0 as i32,
                ..Default::default()
            };
            ShellExecuteExW(&mut info).map_err(|e| format!("以管理员身份运行失败: {e}"))?;
        }
        return Ok(());
    }

    unsafe {
        let mut info = SHELLEXECUTEINFOW {
            cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
            fMask: SEE_MASK_ASYNCOK | SEE_MASK_FLAG_NO_UI,
            lpVerb: PCWSTR(wverb.as_ptr()),
            lpFile: PCWSTR(wpath.as_ptr()),
            nShow: SW_SHOWNORMAL.0 as i32,
            ..Default::default()
        };
        if ShellExecuteExW(&mut info).is_ok() {
            return Ok(());
        }
        let ret = ShellExecuteW(
            None,
            PCWSTR(wverb.as_ptr()),
            PCWSTR(wpath.as_ptr()),
            None,
            None,
            SW_SHOWNORMAL,
        );
        if (ret.0 as isize) <= 32 {
            return Err(format!("执行“{verb}”失败: code={}", ret.0 as isize));
        }
    }
    Ok(())
}

/// Explorer-style delete: move to Recycle Bin with the system confirmation UI.
pub fn delete_to_recycle_bin(path: &str) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("路径为空".into());
    }
    if trimmed.starts_with("::") {
        return Err("系统图标不支持删除".into());
    }
    let mut from: Vec<u16> = trimmed.encode_utf16().chain([0u16, 0u16]).collect();
    let mut op = SHFILEOPSTRUCTW {
        hwnd: HWND::default(),
        wFunc: FO_DELETE,
        pFrom: PCWSTR(from.as_mut_ptr()),
        pTo: PCWSTR::null(),
        fFlags: FOF_ALLOWUNDO.0 as u16 | FOF_WANTNUKEWARNING.0 as u16,
        fAnyOperationsAborted: false.into(),
        hNameMappings: std::ptr::null_mut(),
        lpszProgressTitle: PCWSTR::null(),
    };
    let code = unsafe { SHFileOperationW(&mut op) };
    if op.fAnyOperationsAborted.as_bool() {
        return Err("已取消".into());
    }
    if code != 0 {
        return Err(format!("删除失败: code={code}"));
    }
    Ok(())
}
