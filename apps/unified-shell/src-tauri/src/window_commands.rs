//! Window Management Tauri IPC Commands (GEC-v4.0 Subphase 28.1.1)

#[cfg(not(test))]
use std::path::PathBuf;

pub fn create_app_window_metadata(
    app_type: &str,
    file_path: Option<&str>,
) -> (String, String, String) {
    let clean = app_type.to_lowercase();
    let title = match clean.as_str() {
        "word" | "writer" => "Aether Word",
        "excel" | "calc" => "Aether Excel",
        "ppt" | "powerpoint" | "slides" => "Aether PowerPoint",
        "pdf" => "Aether PDF Studio",
        "notes" | "notepad" => "Aether Notes",
        _ => "AetherOffice",
    };
    let file_param = match file_path {
        Some(f) => format!("&file={f}"),
        None => String::new(),
    };
    let url = format!("index.html?app={clean}{file_param}");
    let label = format!("win-{clean}");
    (label, title.to_string(), url)
}

#[cfg(not(test))]
#[tauri::command]
pub async fn spawn_app_window(
    app: tauri::AppHandle,
    app_type: String,
    file_path: Option<String>,
) -> Result<String, String> {
    let (label_prefix, title, url_str) =
        create_app_window_metadata(&app_type, file_path.as_deref());
    let now = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => d.as_millis(),
        Err(_) => 0,
    };
    let label = format!("{label_prefix}-{now}");

    let win = tauri::WebviewWindowBuilder::new(
        &app,
        &label,
        tauri::WebviewUrl::App(PathBuf::from(url_str)),
    )
    .title(title)
    .inner_size(1280.0, 800.0)
    .min_inner_size(1024.0, 600.0)
    .decorations(true)
    .resizable(true)
    .build()
    .map_err(|e| format!("Failed to spawn window: {e}"))?;

    let _ = win.set_focus();
    Ok(label)
}
