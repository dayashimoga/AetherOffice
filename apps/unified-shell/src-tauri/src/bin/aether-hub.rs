// Standalone Binary Entrypoint: AetherOffice Hub
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    std::env::set_var("AETHER_APP_MODE", "hub");
    std::env::set_var("AETHER_APP_TITLE", "AetherOffice Hub");
    unified_shell_tauri::run();
}
