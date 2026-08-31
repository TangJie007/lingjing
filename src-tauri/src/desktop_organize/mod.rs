//! Desktop organize: fence window lifecycle, desktop scan, Shell menus, drag-out.

pub mod drag;
#[cfg(windows)]
mod drop_target;
#[cfg(windows)]
mod builtin_links;
#[cfg(windows)]
pub(crate) use builtin_links::{
    builtin_kind_from_path, is_managed_builtin_link, CLSID_COMPUTER, CLSID_NETWORK, CLSID_RECYCLE,
};
mod icon_cache;
pub mod item_commands;
pub mod layout;
pub mod lifecycle;
pub mod menu_commands;
mod scan;
mod shell_host;
mod state;
mod types;
mod util;
#[cfg(windows)]
mod win;

pub use types::ShellMenuEntry;

pub use shell_host::maybe_run_shell_menu_host;
pub(crate) use shell_host::shell_host_stage;

pub use lifecycle::{cleanup, reassert, refresh, set_enabled};
