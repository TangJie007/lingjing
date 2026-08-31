use tauri::{AppHandle, Emitter, Manager};

use crate::desktop;
use super::scan::{desktop_scan_dirs, scan_desktop_items};
use super::state::{bump_watch_generation, is_active, set_active, watch_generation, FENCE_LABEL};
use super::types::DesktopItem;
use super::util::run_on_ui;
#[cfg(windows)]
use super::win;

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
        desktop::start_icons_restore_guard()?;
        desktop::set_icons_visible(false);
        let hwnd = match window.hwnd() {
            Ok(h) => h,
            Err(e) => {
                desktop::set_icons_visible(true);
                desktop::stop_icons_restore_guard();
                return Err(format!("获取 HWND 失败: {e}"));
            }
        };
        let _ = window.set_title("");
        let _ = window.set_decorations(false);
        let _ = window.set_shadow(false);
        if let Err(e) = win::attach_fence_to_desktop(hwnd.0 as isize) {
            desktop::set_icons_visible(true);
            desktop::stop_icons_restore_guard();
            return Err(e);
        }
        let _ = window.set_ignore_cursor_events(false);
        // SetParent invalidates tao/wry RegisterDragDrop — reinstall OLE targets.
        super::drop_target::set_app(app.clone());
        super::drop_target::install(hwnd.0 as isize);
    }
    #[cfg(not(windows))]
    {
        return Err("桌面整理仅支持 Windows".into());
    }

    let items = match scan_desktop_items() {
        Ok(items) => items,
        Err(e) => {
            let _ = disable_inner(app);
            return Err(e);
        }
    };
    let _ = window.eval("location.reload()");
    // reload recreates WebView2 child HWNDs — debounced reinstall (coalesced).
    #[cfg(windows)]
    {
        let hwnd_raw = window.hwnd().map(|h| h.0 as isize).unwrap_or(0);
        super::drop_target::schedule_install(app, hwnd_raw, &[600, 1800]);
    }
    if let Err(e) = push_items_to_fence(app, &items) {
        let _ = disable_inner(app);
        return Err(e);
    }
    set_active(true);
    start_desktop_watch(app);
    tracing::info!("[desktop-organize] enabled items={}", items.len());
    Ok(())
}

fn disable_inner(app: &AppHandle) -> Result<(), String> {
    set_active(false);
    stop_desktop_watch();
    super::icon_cache::clear();
    #[cfg(windows)]
    super::drop_target::uninstall();

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
    {
        desktop::set_icons_visible(true);
        desktop::stop_icons_restore_guard();
    }

    tracing::info!("[desktop-organize] disabled");
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
    if !is_active() {
        return Ok(());
    }
    let items = run_on_ui(app, scan_desktop_items)??;
    push_items_to_fence(app, &items)
}

pub fn reassert(app: &AppHandle) {
    if !is_active() {
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
            // Debounce: power/display events can burst reassert.
            super::drop_target::schedule_install(app, hwnd.0 as isize, &[200]);
        }
        let _ = window.set_ignore_cursor_events(false);
    }
}

pub fn cleanup(app: &AppHandle) {
    if is_active() {
        let _ = disable_inner(app);
    }
}

fn stop_desktop_watch() {
    bump_watch_generation();
}

fn start_desktop_watch(app: &AppHandle) {
    use std::time::Duration;

    stop_desktop_watch();
    let gen = watch_generation();
    let app = app.clone();
    std::thread::Builder::new()
        .name("desktop-fence-watch".into())
        .spawn(move || {
            use notify::RecursiveMode;
            use notify_debouncer_mini::new_debouncer;
            use std::sync::mpsc::channel;

            let (tx, rx) = channel();
            let mut debouncer = match new_debouncer(Duration::from_millis(650), tx) {
                Ok(d) => d,
                Err(e) => {
                    tracing::info!("[desktop-organize] watcher create failed: {e}");
                    return;
                }
            };

            let mut watching = false;
            for dir in desktop_scan_dirs() {
                if !dir.is_dir() {
                    continue;
                }
                match debouncer.watcher().watch(&dir, RecursiveMode::NonRecursive) {
                    Ok(()) => {
                        watching = true;
                        tracing::info!("[desktop-organize] watching {}", dir.display());
                    }
                    Err(e) => {
                        tracing::info!("[desktop-organize] watch {} failed: {e}", dir.display())
                    }
                }
            }
            if !watching {
                return;
            }

            loop {
                if watch_generation() != gen {
                    break;
                }
                match rx.recv_timeout(Duration::from_millis(200)) {
                    Ok(Ok(_events)) => {
                        if is_active() {
                            if let Err(e) = refresh(&app) {
                                tracing::info!("[desktop-organize] watch refresh failed: {e}");
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        tracing::info!("[desktop-organize] watcher error: {e}");
                        break;
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
            tracing::info!("[desktop-organize] watcher stopped");
        })
        .ok();
}
