// Standalone Binary Entrypoint: Aether Slides (PowerPoint)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    std::env::set_var("AETHER_APP_MODE", "slides");
    std::env::set_var("AETHER_APP_TITLE", "Aether Slides (PowerPoint)");
    unified_shell_tauri::run();
}
