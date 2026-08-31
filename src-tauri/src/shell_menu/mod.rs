//! Typed Windows Shell context-menu helpers via the official `windows` crate.
//!
//! Keeps COM lifetimes, QueryInterface and PIDL handling out of hand-rolled
//! vtables. Still expected to run inside the isolated Shell-menu host process.

mod builtin;
mod clipboard;
mod context;
mod entry;
mod host;
mod icons;
mod ids;
mod pin;
#[cfg(windows)]
mod share;
mod util;
mod verbs;

pub use builtin::{ensure_blank_refresh_pin, ensure_paste_entry, folder_builtin_menu};
pub use context::{
    invoke_shell_context_command, list_shell_context_menu, list_shell_context_submenu,
    show_explorer_desktop_context_menu, show_native_shell_context_menu,
};
pub use host::{create_host_window, destroy_host_window, pump_messages};
pub use ids::{
    is_builtin_command, BUILTIN_COMPRESS_ZIP, BUILTIN_COPY, BUILTIN_CREATE_SHORTCUT, BUILTIN_CUT,
    BUILTIN_DELETE, BUILTIN_DISPLAY_SETTINGS, BUILTIN_NEW_FOLDER, BUILTIN_NEW_TXT, BUILTIN_OPEN,
    BUILTIN_OPEN_DESKTOP, BUILTIN_OPEN_NEW_WINDOW, BUILTIN_OPEN_TERMINAL, BUILTIN_OPEN_WITH,
    BUILTIN_PASTE, BUILTIN_PERSONALIZE, BUILTIN_PIN_QUICK_ACCESS, BUILTIN_PROPERTIES,
    BUILTIN_REFRESH, BUILTIN_RENAME, BUILTIN_SHARE, BUILTIN_SHOW_IN_FOLDER,
};
#[cfg(windows)]
pub use share::share_path_native;
pub use verbs::delete_to_recycle_bin;
