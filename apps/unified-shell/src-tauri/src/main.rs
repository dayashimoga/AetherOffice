#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(e) = tauri::Builder::default().run(tauri::generate_context!()) {
        eprintln!("Error while running OpenOfficeRS application: {e}");
        std::process::exit(1);
    }
}
