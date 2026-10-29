// Standalone Binary Entrypoint: Aether Calc (Excel)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    std::env::set_var("AETHER_APP_MODE", "calc");
    std::env::set_var("AETHER_APP_TITLE", "Aether Calc (Excel)");
    unified_shell_tauri::run();
}
