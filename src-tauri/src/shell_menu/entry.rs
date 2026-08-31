use crate::desktop_organize::ShellMenuEntry;

use super::ids::BUILTIN_DELETE;

pub fn item(id: u32, label: &str) -> ShellMenuEntry {
    ShellMenuEntry {
        id,
        label: label.into(),
        disabled: false,
        separator: false,
        icon: None,
        children: None,
        menu_path: Vec::new(),
        pin: false,
        destructive: id == BUILTIN_DELETE,
    }
}

pub fn sep() -> ShellMenuEntry {
    ShellMenuEntry {
        id: 0,
        label: String::new(),
        disabled: true,
        separator: true,
        icon: None,
        children: None,
        menu_path: Vec::new(),
        pin: false,
        destructive: false,
    }
}
