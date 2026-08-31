use crate::desktop_organize::ShellMenuEntry;

use super::entry::{item, sep};
use super::ids::*;
use super::pin::apply_win11_pin_row;

/// Minimal fallback when Shell QueryContextMenu hangs (files).
pub fn fallback_menu(path: &str) -> Vec<ShellMenuEntry> {
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
    vec![
        item(BUILTIN_REFRESH, "刷新"),
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
    ]
}

