use crate::error::MacroError;
use crate::ole_cfb::OleCompoundReader;
use crate::ovba_decompress::decompress_vba_stream;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VbaModuleType {
    Document,
    Procedural,
    Class,
    UserForm,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VbaModule {
    pub name: String,
    pub stream_name: String,
    pub module_type: VbaModuleType,
    pub text_offset: usize,
    pub source_code: String,
}

/// Extracted VBA Project representing all code modules from a `vbaProject.bin` part.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VbaProject {
    pub name: String,
    pub code_page: u16,
    pub modules: Vec<VbaModule>,
}

impl VbaProject {
    /// Extracts and decompresses all VBA modules from an OLE Compound `vbaProject.bin` payload.
    pub fn from_ole_bytes(data: &[u8]) -> Result<Self, MacroError> {
        let reader = OleCompoundReader::new(data)?;
        Self::from_ole_reader(&reader)
    }

    /// Parses project records and decompresses modules using an initialized `OleCompoundReader`.
    pub fn from_ole_reader(reader: &OleCompoundReader) -> Result<Self, MacroError> {
        let mut project_name = "VBAProject".to_string();
        let mut code_page = 1252u16;
        let mut modules = Vec::new();

        // 1. Locate and parse 'dir' stream (often inside 'VBA/dir' or root 'dir')
        let dir_stream_name = if reader.has_stream("VBA/dir") {
            "VBA/dir"
        } else if reader.has_stream("dir") {
            "dir"
        } else {
            ""
        };

        let mut parsed_modules_info = Vec::new();

        if !dir_stream_name.is_empty() {
            if let Ok(dir_raw) = reader.read_stream(dir_stream_name) {
                // The 'dir' stream in MS-OVBA is compressed with CompressedSourceCode
                let dir_bytes = match decompress_vba_stream(&dir_raw) {
                    Ok(decomp) => decomp,
                    Err(_) => dir_raw, // Fallback if already decompressed
                };

                // Parse dir stream records
                let mut pos = 0;
                let mut current_mod_name = String::new();
                let mut current_stream_name = String::new();
                let mut current_type = VbaModuleType::Procedural;

                while pos + 6 <= dir_bytes.len() {
                    let record_id = u16::from_le_bytes([dir_bytes[pos], dir_bytes[pos + 1]]);
                    let record_size = u32::from_le_bytes([
                        dir_bytes[pos + 2],
                        dir_bytes[pos + 3],
                        dir_bytes[pos + 4],
                        dir_bytes[pos + 5],
                    ]) as usize;
                    pos += 6;

                    if pos + record_size > dir_bytes.len() {
                        break;
                    }

                    match record_id {
                        0x0004 => {
                            // PROJECTNAME
                            project_name =
                                String::from_utf8_lossy(&dir_bytes[pos..pos + record_size])
                                    .to_string();
                        }
                        0x0003 => {
                            // PROJECTCODEPAGE
                            if record_size >= 2 {
                                code_page =
                                    u16::from_le_bytes([dir_bytes[pos], dir_bytes[pos + 1]]);
                            }
                        }
                        0x0019 => {
                            // MODULENAME
                            current_mod_name =
                                String::from_utf8_lossy(&dir_bytes[pos..pos + record_size])
                                    .to_string();
                        }
                        0x001A => {
                            // MODULESTREAMNAME
                            current_stream_name =
                                String::from_utf8_lossy(&dir_bytes[pos..pos + record_size])
                                    .to_string();
                        }
                        0x0031 => {
                            // MODULEOFFSET
                            if record_size >= 4 {
                                let offset_val = u32::from_le_bytes([
                                    dir_bytes[pos],
                                    dir_bytes[pos + 1],
                                    dir_bytes[pos + 2],
                                    dir_bytes[pos + 3],
                                ]) as usize;

                                if !current_mod_name.is_empty() {
                                    let s_name = if current_stream_name.is_empty() {
                                        current_mod_name.clone()
                                    } else {
                                        current_stream_name.clone()
                                    };
                                    parsed_modules_info.push((
                                        current_mod_name.clone(),
                                        s_name,
                                        current_type,
                                        offset_val,
                                    ));
                                    current_mod_name.clear();
                                    current_stream_name.clear();
                                    current_type = VbaModuleType::Procedural;
                                }
                            }
                        }
                        0x0021 => {
                            // MODULETYPE (Document/Class)
                            current_type = VbaModuleType::Document;
                        }
                        0x0022 => {
                            // MODULETYPE (Procedural)
                            current_type = VbaModuleType::Procedural;
                        }
                        0x0010 => {
                            // Terminating record
                            break;
                        }
                        _ => {}
                    }

                    pos += record_size;
                }
            }
        }

        // 2. If dir parsing succeeded, extract modules by offset
        if !parsed_modules_info.is_empty() {
            for (mod_name, stream_name, mod_type, offset) in parsed_modules_info {
                // Check both stream_name and VBA/stream_name
                let full_stream_path = if reader.has_stream(&format!("VBA/{stream_name}")) {
                    format!("VBA/{stream_name}")
                } else if reader.has_stream(&stream_name) {
                    stream_name.clone()
                } else {
                    continue;
                };

                if let Ok(raw_module_stream) = reader.read_stream(&full_stream_path) {
                    let source_bytes = if offset < raw_module_stream.len() {
                        &raw_module_stream[offset..]
                    } else {
                        &raw_module_stream[..]
                    };

                    let source_code = match decompress_vba_stream(source_bytes) {
                        Ok(decomp) => String::from_utf8_lossy(&decomp).to_string(),
                        Err(_) => String::from_utf8_lossy(source_bytes).to_string(),
                    };

                    modules.push(VbaModule {
                        name: mod_name,
                        stream_name,
                        module_type: mod_type,
                        text_offset: offset,
                        source_code,
                    });
                }
            }
        } else {
            // Fallback: Scan all streams in OLE container looking for VBA source code
            for s_name in reader.list_streams() {
                if s_name.eq_ignore_ascii_case("dir")
                    || s_name.eq_ignore_ascii_case("project")
                    || s_name.eq_ignore_ascii_case("projectwm")
                    || s_name.eq_ignore_ascii_case("_vba_project")
                {
                    continue;
                }

                if let Ok(stream_bytes) = reader.read_stream(&s_name) {
                    if stream_bytes.is_empty() {
                        continue;
                    }

                    // Look for 0x01 magic byte
                    let (decomp_result, found_offset) = if stream_bytes[0] == 0x01 {
                        (decompress_vba_stream(&stream_bytes), 0)
                    } else {
                        let mut match_decomp = None;
                        let mut matched_offset = 0;
                        for off in 1..usize::min(stream_bytes.len(), 2048) {
                            if stream_bytes[off] == 0x01 {
                                if let Ok(d) = decompress_vba_stream(&stream_bytes[off..]) {
                                    if !d.is_empty() {
                                        match_decomp = Some(d);
                                        matched_offset = off;
                                        break;
                                    }
                                }
                            }
                        }
                        (
                            match_decomp.ok_or_else(|| {
                                MacroError::DecompressionError("No magic".to_string())
                            }),
                            matched_offset,
                        )
                    };

                    if let Ok(decomp) = decomp_result {
                        let source_code = String::from_utf8_lossy(&decomp).to_string();
                        let base_name =
                            s_name.split('/').next_back().unwrap_or(&s_name).to_string();
                        modules.push(VbaModule {
                            name: base_name.clone(),
                            stream_name: s_name,
                            module_type: VbaModuleType::Procedural,
                            text_offset: found_offset,
                            source_code,
                        });
                    }
                }
            }
        }

        Ok(Self {
            name: project_name,
            code_page,
            modules,
        })
    }

    pub fn modules(&self) -> &[VbaModule] {
        &self.modules
    }

    pub fn get_module(&self, name: &str) -> Option<&VbaModule> {
        self.modules
            .iter()
            .find(|m| m.name.eq_ignore_ascii_case(name))
    }

    /// Concatenates all module source code into a single audit string.
    pub fn all_source_code(&self) -> String {
        let mut result = String::new();
        for m in &self.modules {
            result.push_str(&format!("' --- MODULE: {} ---\n", m.name));
            result.push_str(&m.source_code);
            result.push_str("\n\n");
        }
        result
    }
}
