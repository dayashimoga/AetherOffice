// Standalone Binary Entrypoint: Aether PDF Studio
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    std::env::set_var("AETHER_APP_MODE", "pdf");
    std::env::set_var("AETHER_APP_TITLE", "Aether PDF Studio");
    unified_shell_tauri::run();
}
