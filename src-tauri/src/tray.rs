//! 系统托盘模块 (ST-001)
//! 托盘图标 + 右键菜单：暂停/切换/打开/退出

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Emitter, Manager, Runtime,
};

/// 创建系统托盘图标和右键菜单
pub fn create<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    let pause_item = MenuItemBuilder::with_id("pause", "暂停壁纸").build(app)?;
    let next_item = MenuItemBuilder::with_id("next", "下一张").build(app)?;
    let open_item = MenuItemBuilder::with_id("open", "打开灵境").build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", "退出").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&pause_item)
        .item(&next_item)
        .separator()
        .item(&open_item)
        .separator()
        .item(&quit_item)
        .build()?;

    let _tray = TrayIconBuilder::with_id("lingscape-tray")
        .tooltip("灵境 LingScape")
        .icon(app.default_window_icon().cloned().unwrap())
        .menu(&menu)
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            match id {
                "pause" => {
                    let _ = app.emit("tray-action", "pause");
                }
                "next" => {
                    let _ = app.emit("tray-action", "next");
                }
                "open" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    let vp = app.state::<crate::video_player::VideoPlayerState>();
                    crate::video_player::stop_video_wallpaper(&vp);
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}
