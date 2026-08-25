// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if lingscape_lib::maybe_run_shell_menu_host() {
        return;
    }
    lingscape_lib::run()
}
