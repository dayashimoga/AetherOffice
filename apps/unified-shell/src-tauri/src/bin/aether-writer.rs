// Standalone Binary Entrypoint: Aether Writer (Word)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    std::env::set_var("AETHER_APP_MODE", "writer");
    std::env::set_var("AETHER_APP_TITLE", "Aether Writer (Word)");
    unified_shell_tauri::run();
}
