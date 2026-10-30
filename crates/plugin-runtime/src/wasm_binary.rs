use crate::error::PluginError;
use crate::manifest::PluginManifest;

pub const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6d]; // \0asm
pub const WASM_VERSION_1: [u8; 4] = [0x01, 0x00, 0x00, 0x00];
pub const WASM_COMPONENT_LAYER: [u8; 4] = [0x0d, 0x00, 0x01, 0x00];

/// WebAssembly section identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionId {
    Custom,
    Type,
    Import,
    Function,
    Table,
    Memory,
    Global,
    Export,
    Start,
    Element,
    Code,
    Data,
    DataCount,
    Unknown(u8),
}

impl SectionId {
    pub fn as_u8(&self) -> u8 {
        match self {
            Self::Custom => 0,
            Self::Type => 1,
            Self::Import => 2,
            Self::Function => 3,
            Self::Table => 4,
            Self::Memory => 5,
            Self::Global => 6,
            Self::Export => 7,
            Self::Start => 8,
            Self::Element => 9,
            Self::Code => 10,
            Self::Data => 11,
            Self::DataCount => 12,
            Self::Unknown(val) => *val,
        }
    }
}

impl From<u8> for SectionId {
    fn from(val: u8) -> Self {
        match val {
            0 => Self::Custom,
            1 => Self::Type,
            2 => Self::Import,
            3 => Self::Function,
            4 => Self::Table,
            5 => Self::Memory,
            6 => Self::Global,
            7 => Self::Export,
            8 => Self::Start,
            9 => Self::Element,
            10 => Self::Code,
            11 => Self::Data,
            12 => Self::DataCount,
            other => Self::Unknown(other),
        }
    }
}

/// Representation of an exported symbol in a Wasm module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmExport {
    pub name: String,
    pub kind: u8,
    pub index: u32,
}

/// Representation of an imported symbol in a Wasm module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmImport {
    pub module: String,
    pub field: String,
    pub kind: u8,
}

/// Representation of memory limits in a Wasm module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryLimits {
    pub min_pages: u32,
    pub max_pages: Option<u32>,
}

/// Parsed section summary from a WebAssembly binary.
#[derive(Debug, Clone)]
pub struct ParsedSection {
    pub id: SectionId,
    pub raw_id: u8,
    pub payload: Vec<u8>,
}

/// A parsed WebAssembly component/module.
#[derive(Debug, Clone)]
pub struct ParsedWasmModule {
    pub is_component: bool,
    pub version: u32,
    pub sections: Vec<ParsedSection>,
    pub exports: Vec<WasmExport>,
    pub imports: Vec<WasmImport>,
    pub memory: Option<MemoryLimits>,
    pub embedded_manifest: Option<PluginManifest>,
}

/// Parse a LEB128 unsigned integer from a byte slice.
pub fn read_u32_leb128(bytes: &[u8], offset: &mut usize) -> Result<u32, PluginError> {
    let mut result: u32 = 0;
    let mut shift = 0;
    let start_offset = *offset;

    loop {
        if *offset >= bytes.len() {
            return Err(PluginError::WasmParseError(format!(
                "Unexpected EOF reading LEB128 integer at offset {start_offset}"
            )));
        }
        let byte = bytes[*offset];
        *offset += 1;

        let val = (byte & 0x7F) as u32;
        if shift >= 32 || (shift == 28 && val > 0x0F) {
            return Err(PluginError::WasmParseError(
                "LEB128 integer overflow".to_string(),
            ));
        }

        result |= val << shift;
        shift += 7;

        if (byte & 0x80) == 0 {
            break;
        }
    }

    Ok(result)
}

/// Write a u32 into LEB128 format.
pub fn write_u32_leb128(mut value: u32, buf: &mut Vec<u8>) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        buf.push(byte);
        if value == 0 {
            break;
        }
    }
}

/// Read a UTF-8 string with LEB128 length prefix.
fn read_wasm_string(bytes: &[u8], offset: &mut usize) -> Result<String, PluginError> {
    let len = read_u32_leb128(bytes, offset)? as usize;
    if *offset + len > bytes.len() {
        return Err(PluginError::WasmParseError(
            "String length exceeds section boundaries".to_string(),
        ));
    }
    let s_bytes = &bytes[*offset..*offset + len];
    *offset += len;
    std::str::from_utf8(s_bytes)
        .map(|s| s.to_string())
        .map_err(|e| PluginError::WasmParseError(format!("Invalid UTF-8 string: {e}")))
}

/// Parse a WebAssembly binary component or core module.
pub fn parse_wasm_module(bytes: &[u8]) -> Result<ParsedWasmModule, PluginError> {
    if bytes.len() < 8 {
        return Err(PluginError::WasmParseError(
            "Binary is too short to be a valid WebAssembly file (< 8 bytes)".to_string(),
        ));
    }

    if bytes[0..4] != WASM_MAGIC {
        return Err(PluginError::WasmParseError(
            "Invalid WebAssembly magic number".to_string(),
        ));
    }

    let version_bytes = [bytes[4], bytes[5], bytes[6], bytes[7]];
    let is_component = version_bytes == WASM_COMPONENT_LAYER;
    let version = u32::from_le_bytes(version_bytes);

    let mut offset = 8;
    let mut sections = Vec::new();
    let mut exports = Vec::new();
    let mut imports = Vec::new();
    let mut memory = None;
    let mut embedded_manifest = None;

    while offset < bytes.len() {
        let raw_id = bytes[offset];
        offset += 1;
        let id = SectionId::from(raw_id);

        let section_len = read_u32_leb128(bytes, &mut offset)? as usize;
        if offset + section_len > bytes.len() {
            return Err(PluginError::WasmParseError(format!(
                "Section {raw_id} length {section_len} extends beyond EOF at offset {offset}"
            )));
        }

        let section_payload = bytes[offset..offset + section_len].to_vec();
        offset += section_len;

        match id {
            SectionId::Custom => {
                let mut custom_offset = 0;
                if let Ok(name) = read_wasm_string(&section_payload, &mut custom_offset) {
                    if name == "aetheroffice:manifest" {
                        let manifest_bytes = &section_payload[custom_offset..];
                        if let Ok(json_str) = std::str::from_utf8(manifest_bytes) {
                            if let Ok(manifest) = PluginManifest::from_json_str(json_str) {
                                embedded_manifest = Some(manifest);
                            }
                        }
                    }
                }
            }
            SectionId::Export => {
                let mut exp_offset = 0;
                if let Ok(count) = read_u32_leb128(&section_payload, &mut exp_offset) {
                    for _ in 0..count {
                        if exp_offset >= section_payload.len() {
                            break;
                        }
                        if let Ok(name) = read_wasm_string(&section_payload, &mut exp_offset) {
                            if exp_offset < section_payload.len() {
                                let kind = section_payload[exp_offset];
                                exp_offset += 1;
                                if let Ok(idx) = read_u32_leb128(&section_payload, &mut exp_offset)
                                {
                                    exports.push(WasmExport {
                                        name,
                                        kind,
                                        index: idx,
                                    });
                                }
                            }
                        }
                    }
                }
            }
            SectionId::Import => {
                let mut imp_offset = 0;
                if let Ok(count) = read_u32_leb128(&section_payload, &mut imp_offset) {
                    for _ in 0..count {
                        if imp_offset >= section_payload.len() {
                            break;
                        }
                        if let (Ok(mod_name), Ok(field_name)) = (
                            read_wasm_string(&section_payload, &mut imp_offset),
                            read_wasm_string(&section_payload, &mut imp_offset),
                        ) {
                            if imp_offset < section_payload.len() {
                                let kind = section_payload[imp_offset];
                                imp_offset += 1;
                                // skip kind descriptor bytes safely
                                match kind {
                                    0 => {
                                        // function type index
                                        let _ = read_u32_leb128(&section_payload, &mut imp_offset);
                                    }
                                    1 => {
                                        // table limits
                                        imp_offset += 1;
                                        let _ = read_u32_leb128(&section_payload, &mut imp_offset);
                                    }
                                    2 => {
                                        // memory limits
                                        let flags = if imp_offset < section_payload.len() {
                                            section_payload[imp_offset]
                                        } else {
                                            0
                                        };
                                        imp_offset += 1;
                                        let _ = read_u32_leb128(&section_payload, &mut imp_offset);
                                        if flags == 1 {
                                            let _ =
                                                read_u32_leb128(&section_payload, &mut imp_offset);
                                        }
                                    }
                                    3 => {
                                        // global
                                        imp_offset += 2;
                                    }
                                    _ => {}
                                }
                                imports.push(WasmImport {
                                    module: mod_name,
                                    field: field_name,
                                    kind,
                                });
                            }
                        }
                    }
                }
            }
            SectionId::Memory => {
                let mut mem_offset = 0;
                if let Ok(count) = read_u32_leb128(&section_payload, &mut mem_offset) {
                    if count > 0 && mem_offset < section_payload.len() {
                        let flags = section_payload[mem_offset];
                        mem_offset += 1;
                        if let Ok(min_pages) = read_u32_leb128(&section_payload, &mut mem_offset) {
                            let max_pages = if flags == 1 {
                                read_u32_leb128(&section_payload, &mut mem_offset).ok()
                            } else {
                                None
                            };
                            memory = Some(MemoryLimits {
                                min_pages,
                                max_pages,
                            });
                        }
                    }
                }
            }
            _ => {}
        }

        sections.push(ParsedSection {
            id,
            raw_id,
            payload: section_payload,
        });
    }

    Ok(ParsedWasmModule {
        is_component,
        version,
        sections,
        exports,
        imports,
        memory,
        embedded_manifest,
    })
}

/// Helper builder for constructing valid Wasm binaries in tests and mocks.
#[derive(Default)]
pub struct WasmBinaryBuilder {
    is_component: bool,
    custom_sections: Vec<(String, Vec<u8>)>,
    exports: Vec<(String, u8, u32)>,
    imports: Vec<(String, String, u8)>,
    memory_limits: Option<(u32, Option<u32>)>,
}

impl WasmBinaryBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn component(mut self, is_component: bool) -> Self {
        self.is_component = is_component;
        self
    }

    pub fn with_custom_section(mut self, name: impl Into<String>, payload: Vec<u8>) -> Self {
        self.custom_sections.push((name.into(), payload));
        self
    }

    pub fn with_manifest(self, manifest: &PluginManifest) -> Result<Self, PluginError> {
        let json = manifest.to_json_string()?;
        Ok(self.with_custom_section("aetheroffice:manifest", json.into_bytes()))
    }

    pub fn with_export(mut self, name: impl Into<String>, kind: u8, index: u32) -> Self {
        self.exports.push((name.into(), kind, index));
        self
    }

    pub fn with_import(
        mut self,
        module: impl Into<String>,
        field: impl Into<String>,
        kind: u8,
    ) -> Self {
        self.imports.push((module.into(), field.into(), kind));
        self
    }

    pub fn with_memory(mut self, min_pages: u32, max_pages: Option<u32>) -> Self {
        self.memory_limits = Some((min_pages, max_pages));
        self
    }

    pub fn build(self) -> Vec<u8> {
        let mut binary = Vec::new();
        // Magic
        binary.extend_from_slice(&WASM_MAGIC);
        // Version
        if self.is_component {
            binary.extend_from_slice(&WASM_COMPONENT_LAYER);
        } else {
            binary.extend_from_slice(&WASM_VERSION_1);
        }

        // Custom sections
        for (name, payload) in self.custom_sections {
            let mut sec_data = Vec::new();
            write_u32_leb128(name.len() as u32, &mut sec_data);
            sec_data.extend_from_slice(name.as_bytes());
            sec_data.extend_from_slice(&payload);

            binary.push(SectionId::Custom.as_u8());
            write_u32_leb128(sec_data.len() as u32, &mut binary);
            binary.extend_from_slice(&sec_data);
        }

        // Import section
        if !self.imports.is_empty() {
            let mut sec_data = Vec::new();
            write_u32_leb128(self.imports.len() as u32, &mut sec_data);
            for (m, f, k) in &self.imports {
                write_u32_leb128(m.len() as u32, &mut sec_data);
                sec_data.extend_from_slice(m.as_bytes());
                write_u32_leb128(f.len() as u32, &mut sec_data);
                sec_data.extend_from_slice(f.as_bytes());
                sec_data.push(*k);
                if *k == 0 {
                    // func type 0
                    write_u32_leb128(0, &mut sec_data);
                }
            }
            binary.push(SectionId::Import.as_u8());
            write_u32_leb128(sec_data.len() as u32, &mut binary);
            binary.extend_from_slice(&sec_data);
        }

        // Memory section
        if let Some((min, max_opt)) = self.memory_limits {
            let mut sec_data = Vec::new();
            write_u32_leb128(1, &mut sec_data); // 1 memory
            if let Some(max) = max_opt {
                sec_data.push(1); // flag 1: min and max
                write_u32_leb128(min, &mut sec_data);
                write_u32_leb128(max, &mut sec_data);
            } else {
                sec_data.push(0); // flag 0: min only
                write_u32_leb128(min, &mut sec_data);
            }
            binary.push(SectionId::Memory.as_u8());
            write_u32_leb128(sec_data.len() as u32, &mut binary);
            binary.extend_from_slice(&sec_data);
        }

        // Export section
        if !self.exports.is_empty() {
            let mut sec_data = Vec::new();
            write_u32_leb128(self.exports.len() as u32, &mut sec_data);
            for (name, kind, idx) in &self.exports {
                write_u32_leb128(name.len() as u32, &mut sec_data);
                sec_data.extend_from_slice(name.as_bytes());
                sec_data.push(*kind);
                write_u32_leb128(*idx, &mut sec_data);
            }
            binary.push(SectionId::Export.as_u8());
            write_u32_leb128(sec_data.len() as u32, &mut binary);
            binary.extend_from_slice(&sec_data);
        }

        binary
    }
}
