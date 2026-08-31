//! Re-register OLE IDropTarget after SetParent to desktop DefView.
//! Must run on the UI thread that owns the HWND (RegisterDragDrop rule).

#![cfg(windows)]

extern crate windows_core;

use std::cell::UnsafeCell;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::ptr;
use std::sync::Mutex;

use tauri::{AppHandle, Emitter};
use windows::core::{implement, BOOL, Ref};
use windows::Win32::Foundation::{HWND, LPARAM, POINTL};
use windows::Win32::System::Com::{
    CoInitializeEx, IDataObject, DVASPECT_CONTENT, FORMATETC, TYMED_HGLOBAL,
    COINIT_APARTMENTTHREADED,
};
use windows::Win32::System::Ole::{
    IDropTarget, IDropTarget_Impl, RegisterDragDrop, RevokeDragDrop, CF_HDROP, DROPEFFECT,
    DROPEFFECT_COPY, DROPEFFECT_MOVE, DROPEFFECT_NONE,
};
use windows::Win32::System::SystemServices::{MK_CONTROL, MK_SHIFT, MODIFIERKEYS_FLAGS};
use windows::Win32::UI::Shell::{DragFinish, DragQueryFileW, HDROP};
use windows::Win32::UI::WindowsAndMessaging::EnumChildWindows;

use super::menu_commands::place_paths_on_desktop;

struct InstalledTarget {
    hwnd: isize,
    _target: IDropTarget,
}

unsafe impl Send for InstalledTarget {}
unsafe impl Sync for InstalledTarget {}

static INSTALLED: Mutex<Vec<InstalledTarget>> = Mutex::new(Vec::new());
static DROP_APP: Mutex<Option<AppHandle>> = Mutex::new(None);

fn hdrop_format() -> FORMATETC {
    FORMATETC {
        cfFormat: CF_HDROP.0,
        ptd: ptr::null_mut(),
        dwAspect: DVASPECT_CONTENT.0,
        lindex: -1,
        tymed: TYMED_HGLOBAL.0 as u32,
    }
}

pub fn set_app(app: AppHandle) {
    if let Ok(mut g) = DROP_APP.lock() {
        *g = Some(app);
    }
}

pub fn uninstall() {
    if let Ok(mut list) = INSTALLED.lock() {
        for entry in list.drain(..) {
            unsafe {
                let _ = RevokeDragDrop(HWND(entry.hwnd as *mut _));
            }
        }
    }
}

/// Install OLE drop targets. **Must be called on the UI/STA thread.**
pub fn install(hwnd_raw: isize) {
    uninstall();

    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    }

    let root = HWND(hwnd_raw as *mut _);
    let mut installed: Vec<InstalledTarget> = Vec::new();
    inject(root, &mut installed);

    unsafe {
        let mut callback = |child: HWND| {
            inject(child, &mut installed);
            true
        };
        let mut trait_obj: &mut dyn FnMut(HWND) -> bool = &mut callback;
        let closure_ptr: *mut std::ffi::c_void = std::mem::transmute(&mut trait_obj);
        let lparam = LPARAM(closure_ptr as isize);
        unsafe extern "system" fn enum_cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let closure =
                &mut *(lparam.0 as *mut std::ffi::c_void as *mut &mut dyn FnMut(HWND) -> bool);
            closure(hwnd).into()
        }
        let _ = EnumChildWindows(Some(root), Some(enum_cb), lparam);
    }

    tracing::info!(
        "[desktop-organize] OLE drop targets installed count={}",
        installed.len()
    );
    if let Ok(mut g) = INSTALLED.lock() {
        *g = installed;
    }
}

fn inject(hwnd: HWND, out: &mut Vec<InstalledTarget>) {
    if hwnd.0.is_null() {
        return;
    }
    let raw = hwnd.0 as isize;
    if out.iter().any(|t| t.hwnd == raw) {
        return;
    }
    let target: IDropTarget = FenceDropTarget::new().into();
    unsafe {
        let _ = RevokeDragDrop(hwnd);
        match RegisterDragDrop(hwnd, &target) {
            Ok(()) => {
                out.push(InstalledTarget {
                    hwnd: raw,
                    _target: target,
                });
            }
            Err(e) => {
                tracing::info!("[desktop-organize] RegisterDragDrop failed hwnd={hwnd:?}: {e}");
            }
        }
    }
}

/// Pick an effect that the drag source actually allows.
fn negotiate_effect(allowed: DROPEFFECT, keys: MODIFIERKEYS_FLAGS) -> DROPEFFECT {
    let prefer_copy = (keys.0 & MK_CONTROL.0) != 0;
    let prefer_move = (keys.0 & MK_SHIFT.0) != 0;
    let allow_copy = (allowed.0 & DROPEFFECT_COPY.0) != 0;
    let allow_move = (allowed.0 & DROPEFFECT_MOVE.0) != 0;

    if prefer_copy && allow_copy {
        return DROPEFFECT_COPY;
    }
    if prefer_move && allow_move {
        return DROPEFFECT_MOVE;
    }
    // Explorer-like default: move when possible, else copy.
    if allow_move {
        return DROPEFFECT_MOVE;
    }
    if allow_copy {
        return DROPEFFECT_COPY;
    }
    DROPEFFECT_NONE
}

fn keys_want_move(keys: MODIFIERKEYS_FLAGS, effect: DROPEFFECT) -> Option<bool> {
    if (keys.0 & MK_CONTROL.0) != 0 {
        Some(false)
    } else if (keys.0 & MK_SHIFT.0) != 0 {
        Some(true)
    } else if (effect.0 & DROPEFFECT_MOVE.0) != 0 {
        Some(true)
    } else if (effect.0 & DROPEFFECT_COPY.0) != 0 {
        Some(false)
    } else {
        None
    }
}

fn data_has_files(data_obj: Ref<'_, IDataObject>) -> bool {
    let Some(obj) = data_obj.as_ref() else {
        return false;
    };
    let fmt = hdrop_format();
    unsafe { obj.QueryGetData(&fmt).is_ok() }
}

unsafe fn iterate_filenames<F>(data_obj: Ref<'_, IDataObject>, mut callback: F) -> Option<HDROP>
where
    F: FnMut(PathBuf),
{
    let drop_format = hdrop_format();
    match data_obj
        .as_ref()
        .expect("null IDataObject")
        .GetData(&drop_format)
    {
        Ok(medium) => {
            let hdrop = HDROP(medium.u.hGlobal.0 as _);
            let item_count = DragQueryFileW(hdrop, 0xFFFFFFFF, None);
            for i in 0..item_count {
                let character_count = DragQueryFileW(hdrop, i, None) as usize;
                let mut path_buf = vec![0u16; character_count + 1];
                DragQueryFileW(hdrop, i, Some(&mut path_buf));
                callback(OsString::from_wide(&path_buf[0..character_count]).into());
            }
            Some(hdrop)
        }
        Err(_) => None,
    }
}

#[implement(IDropTarget)]
struct FenceDropTarget {
    cursor_effect: UnsafeCell<DROPEFFECT>,
    enter_valid: UnsafeCell<bool>,
}

impl FenceDropTarget {
    fn new() -> Self {
        Self {
            cursor_effect: UnsafeCell::new(DROPEFFECT_NONE),
            enter_valid: UnsafeCell::new(false),
        }
    }
}

unsafe impl Send for FenceDropTarget {}

#[allow(non_snake_case)]
impl IDropTarget_Impl for FenceDropTarget_Impl {
    fn DragEnter(
        &self,
        pDataObj: Ref<'_, IDataObject>,
        grfKeyState: MODIFIERKEYS_FLAGS,
        _pt: &POINTL,
        pdwEffect: *mut DROPEFFECT,
    ) -> windows::core::Result<()> {
        let allowed = unsafe { *pdwEffect };
        // QueryGetData only — some sources do not expose CF_HDROP until Drop.
        let valid = data_has_files(pDataObj);
        let effect = if valid {
            negotiate_effect(allowed, grfKeyState)
        } else {
            DROPEFFECT_NONE
        };
        tracing::info!(
            "[desktop-organize] DragEnter valid={valid} allowed={allowed:?} effect={effect:?}"
        );
        unsafe {
            *self.enter_valid.get() = valid && effect.0 != 0;
            *self.cursor_effect.get() = effect;
            *pdwEffect = effect;
        }
        Ok(())
    }

    fn DragOver(
        &self,
        grfKeyState: MODIFIERKEYS_FLAGS,
        _pt: &POINTL,
        pdwEffect: *mut DROPEFFECT,
    ) -> windows::core::Result<()> {
        unsafe {
            if *self.enter_valid.get() {
                let allowed = *pdwEffect;
                let effect = negotiate_effect(allowed, grfKeyState);
                *self.cursor_effect.get() = effect;
                *pdwEffect = effect;
            } else {
                *pdwEffect = DROPEFFECT_NONE;
            }
        }
        Ok(())
    }

    fn DragLeave(&self) -> windows::core::Result<()> {
        unsafe {
            *self.enter_valid.get() = false;
            *self.cursor_effect.get() = DROPEFFECT_NONE;
        }
        Ok(())
    }

    fn Drop(
        &self,
        pDataObj: Ref<'_, IDataObject>,
        grfKeyState: MODIFIERKEYS_FLAGS,
        _pt: &POINTL,
        pdwEffect: *mut DROPEFFECT,
    ) -> windows::core::Result<()> {
        if !unsafe { *self.enter_valid.get() } {
            unsafe {
                *pdwEffect = DROPEFFECT_NONE;
            }
            return Ok(());
        }

        let allowed = unsafe { *pdwEffect };
        let effect = negotiate_effect(allowed, grfKeyState);
        let move_files = keys_want_move(grfKeyState, effect);

        let mut paths = Vec::new();
        let hdrop = unsafe { iterate_filenames(pDataObj, |p| paths.push(p)) };
        if let Some(hdrop) = hdrop {
            unsafe { DragFinish(hdrop) };
        }

        unsafe {
            *pdwEffect = effect;
            *self.enter_valid.get() = false;
        }

        tracing::info!(
            "[desktop-organize] Drop paths={} effect={effect:?} move={move_files:?}",
            paths.len()
        );

        if paths.is_empty() {
            return Ok(());
        }

        let app = DROP_APP.lock().ok().and_then(|g| g.clone());
        if let Some(app) = app {
            let count = paths.len();
            std::thread::spawn(move || {
                if let Err(e) = place_paths_on_desktop(&app, &paths, move_files) {
                    tracing::info!("[desktop-organize] OLE drop place failed: {e}");
                    let _ = app.emit("fence-toast", e);
                } else {
                    tracing::info!("[desktop-organize] OLE drop placed count={count}");
                }
            });
        }

        Ok(())
    }
}
