use crate::desktop_organize::ShellMenuEntry;

use super::clipboard::has_file_drop;
use super::entry::{item, sep};
use super::ids::*;
use super::pin::apply_win11_pin_row;

fn paste_item(enabled: bool) -> ShellMenuEntry {
    let mut e = item(BUILTIN_PASTE, "粘贴");
    e.disabled = !enabled;
    e
}

fn entry_is_paste(e: &ShellMenuEntry) -> bool {
    if e.separator {
        return false;
    }
    if e.id == BUILTIN_PASTE {
        return true;
    }
    let label = e.label.trim();
    label.contains("粘贴") || label.eq_ignore_ascii_case("Paste")
}

fn menu_has_paste(entries: &[ShellMenuEntry]) -> bool {
    for e in entries {
        if entry_is_paste(e) {
            return true;
        }
        if let Some(children) = e.children.as_ref() {
            if menu_has_paste(children) {
                return true;
            }
        }
    }
    false
}

/// Ensure blank-desktop menus expose Paste when CF_HDROP is available.
pub fn ensure_paste_entry(mut entries: Vec<ShellMenuEntry>) -> Vec<ShellMenuEntry> {
    let can_paste = has_file_drop();
    if menu_has_paste(&entries) {
        // Enable our builtin paste if clipboard now has files.
        for e in &mut entries {
            if e.id == BUILTIN_PASTE {
                e.disabled = !can_paste;
            }
        }
        return entries;
    }
    if !can_paste {
        return entries;
    }
    let paste = paste_item(true);
    if let Some(i) = entries
        .iter()
        .position(|e| e.id == BUILTIN_REFRESH || e.label.contains("刷新"))
    {
        let at = i + 1;
        if at < entries.len() && entries[at].separator {
            entries.insert(at + 1, paste);
        } else {
            entries.insert(at, sep());
            entries.insert(at + 1, paste);
        }
    } else {
        entries.insert(0, paste);
        if entries.len() > 1 && !entries[1].separator {
            entries.insert(1, sep());
        }
    }
    entries
}

fn entry_is_refresh(e: &ShellMenuEntry) -> bool {
    if e.separator || e.children.is_some() {
        return false;
    }
    e.id == BUILTIN_REFRESH || e.label.contains("刷新")
}

/// Blank desktop: always pin 刷新 in the header strip (Shell menus put it in the body).
pub fn ensure_blank_refresh_pin(entries: Vec<ShellMenuEntry>) -> Vec<ShellMenuEntry> {
    use super::pin::pin_icon_svg;

    let mut pins: Vec<ShellMenuEntry> = Vec::new();
    let mut body: Vec<ShellMenuEntry> = Vec::new();
    for e in entries {
        if entry_is_refresh(&e) {
            continue;
        }
        if e.pin {
            pins.push(e);
        } else {
            body.push(e);
        }
    }

    let mut refresh = item(BUILTIN_REFRESH, "刷新");
    refresh.pin = true;
    refresh.icon = Some(pin_icon_svg("refresh"));

    let mut out = Vec::with_capacity(pins.len() + body.len() + 2);
    out.push(refresh);
    out.append(&mut pins);
    if !body.is_empty() {
        while body.first().is_some_and(|e| e.separator) {
            body.remove(0);
        }
        out.push(sep());
        out.append(&mut body);
    }
    out
}

/// Minimal fallback when Shell QueryContextMenu hangs (files).
pub fn fallback_menu(path: &str) -> Vec<ShellMenuEntry> {
    if is_shell_namespace_path(path) {
        return namespace_builtin_menu(path);
    }
    if std::path::Path::new(path).is_dir() {
        return folder_builtin_menu();
    }
    apply_win11_pin_row(
        None,
        vec![
            item(BUILTIN_CUT, "剪切"),
            item(BUILTIN_COPY, "复制"),
            item(BUILTIN_RENAME, "重命名"),
            item(BUILTIN_DELETE, "删除"),
            sep(),
            item(BUILTIN_OPEN, "打开"),
            item(BUILTIN_OPEN_WITH, "打开方式"),
            item(BUILTIN_SHOW_IN_FOLDER, "在资源管理器中显示"),
            sep(),
            item(BUILTIN_PROPERTIES, "属性"),
        ],
    )
}

/// Shell namespace desktop icons (`::{CLSID}`) or managed builtin `.lnk` shortcuts.
pub fn is_shell_namespace_path(path: &str) -> bool {
    if path.trim_start().starts_with("::") {
        return true;
    }
    #[cfg(windows)]
    {
        return crate::desktop_organize::is_managed_builtin_link(path);
    }
    #[cfg(not(windows))]
    {
        false
    }
}

pub fn is_recycle_bin_path(path: &str) -> bool {
    #[cfg(windows)]
    {
        return crate::desktop_organize::builtin_kind_from_path(path) == Some("recycle");
    }
    #[cfg(not(windows))]
    {
        path.to_ascii_uppercase()
            .contains("645FF040-5081-101B-9F08-00AA002F954E")
    }
}

pub fn is_network_places_path(path: &str) -> bool {
    #[cfg(windows)]
    {
        return crate::desktop_organize::builtin_kind_from_path(path) == Some("network");
    }
    #[cfg(not(windows))]
    {
        path.to_ascii_uppercase()
            .contains("F02C1A0D-B21F-4110-8426-0A0C959C3602")
    }
}

/// Built-in menus for 回收站 / 网络 (always); 此电脑 uses this only as Shell fallback.
pub fn namespace_builtin_menu(path: &str) -> Vec<ShellMenuEntry> {
    if is_recycle_bin_path(path) {
        return recycle_builtin_menu();
    }
    if is_network_places_path(path) {
        return network_builtin_menu();
    }
    // 此电脑 / other namespace fallback
    vec![
        item(BUILTIN_OPEN, "打开"),
        sep(),
        item(BUILTIN_PIN_START, "固定到「开始」屏幕"),
        item(BUILTIN_PIN_QUICK_ACCESS, "固定到「快速访问」"),
        sep(),
        item(BUILTIN_PROPERTIES, "属性"),
    ]
}

fn recycle_builtin_menu() -> Vec<ShellMenuEntry> {
    vec![
        item(BUILTIN_OPEN, "打开"),
        item(BUILTIN_EMPTY_RECYCLE, "清空回收站"),
        sep(),
        item(BUILTIN_PIN_START, "固定到「开始」屏幕"),
        item(BUILTIN_PIN_QUICK_ACCESS, "固定到「快速访问」"),
        sep(),
        item(BUILTIN_PROPERTIES, "属性"),
    ]
}

fn network_builtin_menu() -> Vec<ShellMenuEntry> {
    vec![
        item(BUILTIN_OPEN, "打开"),
        sep(),
        item(BUILTIN_MAP_NETWORK_DRIVE, "映射网络驱动器"),
        item(BUILTIN_DISCONNECT_NETWORK_DRIVE, "断开网络驱动器连接"),
        sep(),
        item(BUILTIN_PIN_START, "固定到「开始」屏幕"),
        item(BUILTIN_PIN_QUICK_ACCESS, "固定到「快速访问」"),
        sep(),
        item(BUILTIN_PROPERTIES, "属性"),
    ]
}

/// Built-in folder menu replicating common Windows Explorer folder items.
/// Used instead of QueryContextMenu (folder Shell extensions often hang).
pub fn folder_builtin_menu() -> Vec<ShellMenuEntry> {
    apply_win11_pin_row(
        None,
        vec![
            item(BUILTIN_OPEN, "打开"),
            item(BUILTIN_OPEN_NEW_WINDOW, "在新窗口中打开"),
            sep(),
            item(BUILTIN_PIN_QUICK_ACCESS, "固定到「快速访问」"),
            sep(),
            item(BUILTIN_CUT, "剪切"),
            item(BUILTIN_COPY, "复制"),
            item(BUILTIN_CREATE_SHORTCUT, "创建快捷方式"),
            sep(),
            item(BUILTIN_DELETE, "删除"),
            item(BUILTIN_RENAME, "重命名"),
            sep(),
            item(BUILTIN_COMPRESS_ZIP, "压缩为 ZIP 文件"),
            sep(),
            item(BUILTIN_PROPERTIES, "属性"),
        ],
    )
}

/// Blank desktop / fence background menu (Explorer-like, no Shell hang).
pub fn desktop_blank_builtin_menu() -> Vec<ShellMenuEntry> {
    use super::pin::pin_icon_svg;

    let mut refresh = item(BUILTIN_REFRESH, "刷新");
    refresh.pin = true;
    refresh.icon = Some(pin_icon_svg("refresh"));

    ensure_paste_entry(vec![
        refresh,
        sep(),
        ShellMenuEntry {
            id: 0,
            label: "新建".into(),
            disabled: false,
            separator: false,
            icon: None,
            children: Some(vec![
                item(BUILTIN_NEW_FOLDER, "文件夹"),
                item(BUILTIN_NEW_TXT, "文本文档"),
            ]),
            menu_path: Vec::new(),
            pin: false,
            destructive: false,
        },
        sep(),
        item(BUILTIN_OPEN_DESKTOP, "打开桌面文件夹"),
        item(BUILTIN_OPEN_TERMINAL, "在终端中打开"),
        sep(),
        item(BUILTIN_DISPLAY_SETTINGS, "显示设置"),
        item(BUILTIN_PERSONALIZE, "个性化"),
    ])
}
