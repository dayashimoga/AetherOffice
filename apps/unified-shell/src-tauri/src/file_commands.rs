//! Native File Save & Read IPC Commands (GEC-v6.0 Subphase 35.1.1)

use std::path::PathBuf;

pub fn save_file_bytes_impl(
    file_name: &str,
    bytes: &[u8],
    custom_path: Option<&str>,
) -> Result<String, String> {
    let target_path = if let Some(p) = custom_path {
        PathBuf::from(p)
    } else {
        let base_dir = if let Ok(profile) = std::env::var("USERPROFILE") {
            let dl = PathBuf::from(&profile).join("Downloads");
            if dl.exists() {
                dl
            } else {
                PathBuf::from(&profile).join("Documents")
            }
        } else if let Ok(home) = std::env::var("HOME") {
            let dl = PathBuf::from(&home).join("Downloads");
            if dl.exists() {
                dl
            } else {
                PathBuf::from(&home).join("Documents")
            }
        } else {
            std::env::current_dir().map_err(|e| e.to_string())?
        };

        if !base_dir.exists() {
            let _ = std::fs::create_dir_all(&base_dir);
        }

        base_dir.join(file_name)
    };

    if let Some(parent) = target_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {e}"))?;
        }
    }

    std::fs::write(&target_path, bytes)
        .map_err(|e| format!("Failed to write file to {}: {e}", target_path.display()))?;

    Ok(target_path.to_string_lossy().to_string())
}

pub fn read_file_bytes_impl(file_path: &str) -> Result<Vec<u8>, String> {
    std::fs::read(file_path).map_err(|e| format!("Failed to read file from {file_path}: {e}"))
}

#[cfg(not(test))]
#[tauri::command]
pub fn save_file_bytes(
    file_name: String,
    bytes: Vec<u8>,
    custom_path: Option<String>,
) -> Result<String, String> {
    save_file_bytes_impl(&file_name, &bytes, custom_path.as_deref())
}

#[cfg(not(test))]
#[tauri::command]
pub fn read_file_bytes(file_path: String) -> Result<Vec<u8>, String> {
    read_file_bytes_impl(&file_path)
}

#[cfg(test)]
pub fn save_file_bytes(
    file_name: String,
    bytes: Vec<u8>,
    custom_path: Option<String>,
) -> Result<String, String> {
    save_file_bytes_impl(&file_name, &bytes, custom_path.as_deref())
}

#[cfg(test)]
pub fn read_file_bytes(file_path: String) -> Result<Vec<u8>, String> {
    read_file_bytes_impl(&file_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_read_file_bytes() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_save_read_fc.docx");
        let sample_bytes = b"PK\x03\x04test_docx_data".to_vec();
        let saved_path = save_file_bytes(
            "test_save_read_fc.docx".to_string(),
            sample_bytes.clone(),
            Some(test_file.to_string_lossy().to_string()),
        );
        assert!(saved_path.is_ok());
        let read_back = read_file_bytes(test_file.to_string_lossy().to_string());
        assert!(read_back.is_ok());
        if let Ok(data) = read_back {
            assert_eq!(data, sample_bytes);
        }
        let _ = std::fs::remove_file(test_file);
    }
}
