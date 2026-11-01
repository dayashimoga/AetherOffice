//! WebAssembly Core Bridge & Browser Fallback Engine
//! Conforms strictly to GEC-v7.0 Article I, III, IV, and VI:
//! - Pure safe Rust with zero panics or unhandled errors.
//! - High-performance in-memory session orchestration for browser, webview, and headless environments.
//! - Lossless round-trip byte preservation and typed JSON interop for presentation shells.

use crate::commands::transactions::GlobalOfficeState;
use crate::document_io::{DocumentIoEngine, DocumentMetadata};
use crate::shell::DesktopSessionManager;
use crate::ui_dispatcher::{UiActionEvent, UiCommandDispatcher, UiDispatchResult};
use crate::{DocumentType, SessionSummary};
use excel_engine::package::ExcelWorkbookPackage;
use ppt_engine::package::PresentationPackage;
use std::io::Cursor;
use std::sync::{Mutex, OnceLock};
use word_engine::package::WordPackage;

/// In-memory engine managing document sessions inside a WebAssembly or headless environment.
pub struct WasmOfficeEngine {
    pub session_manager: DesktopSessionManager,
}

impl Default for WasmOfficeEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl WasmOfficeEngine {
    /// Creates a new isolated WASM office engine instance.
    pub fn new() -> Self {
        Self {
            session_manager: DesktopSessionManager::new(),
        }
    }

    /// Creates a new blank document session.
    pub fn create_session(
        &mut self,
        doc_type_str: &str,
        title: &str,
    ) -> Result<SessionSummary, String> {
        let doc_type = parse_document_type(doc_type_str);
        let summary = self.session_manager.create_session(doc_type, title);
        let mut state = GlobalOfficeState::new();

        match doc_type {
            DocumentType::Word => {
                state.word_content = String::new();
            }
            DocumentType::Excel => {
                state.excel_sheets = vec!["Sheet1".to_string()];
            }
            DocumentType::PowerPoint => {
                state.ppt_slides = vec![1];
            }
            DocumentType::Pdf => {
                state.word_content = "OpenOfficeRS Native PDF Document".to_string();
            }
        }

        self.session_manager
            .office_states
            .insert(summary.session_id.clone(), state);

        Ok(summary)
    }

    /// Ingests a raw document binary package and re-hydrates its DOM and unparsed store.
    pub fn open_document_bytes(
        &mut self,
        bytes: &[u8],
        file_name: &str,
    ) -> Result<SessionSummary, String> {
        let doc_type = DocumentIoEngine::detect_doc_type(file_name);
        let mut office_state = GlobalOfficeState::new();

        match doc_type {
            DocumentType::Word => {
                let pkg = WordPackage::from_bytes(bytes)
                    .map_err(|e| format!("Failed to parse Word package: {e}"))?;
                office_state.word_content = pkg.document.get_full_text();
                office_state.unparsed_store = Some(pkg.unparsed_store);
                office_state.raw_package_bytes = Some(bytes.to_vec());
            }
            DocumentType::Excel => {
                let pkg = ExcelWorkbookPackage::open(Cursor::new(bytes))
                    .map_err(|e| format!("Failed to parse Excel package: {e}"))?;

                office_state.excel_sheets =
                    pkg.workbook.sheets.iter().map(|s| s.name.clone()).collect();

                if let Some((_, ws)) = pkg.worksheets.first() {
                    for (row, col, cell_val, _) in ws.grid.iter_cells() {
                        if let Ok(coord) = excel_engine::grid::coords_to_cell(row, col) {
                            let text_val = match &cell_val {
                                excel_engine::grid::CellValue::Number(num) => num.to_string(),
                                excel_engine::grid::CellValue::SharedString(idx) => pkg
                                    .shared_strings
                                    .get(*idx as usize)
                                    .unwrap_or("")
                                    .to_string(),
                                excel_engine::grid::CellValue::InlineString(s) => s.clone(),
                                excel_engine::grid::CellValue::Boolean(b) => {
                                    if *b {
                                        "TRUE".into()
                                    } else {
                                        "FALSE".into()
                                    }
                                }
                                excel_engine::grid::CellValue::Formula { formula, .. } => {
                                    format!("={formula}")
                                }
                                _ => String::new(),
                            };
                            if !text_val.is_empty() {
                                office_state.excel_cells.insert(coord, text_val);
                            }
                        }
                    }
                }

                office_state.unparsed_store = Some(pkg.unparsed_store);
                office_state.raw_package_bytes = Some(bytes.to_vec());
            }
            DocumentType::PowerPoint => {
                let pkg = PresentationPackage::open(Cursor::new(bytes))
                    .map_err(|e| format!("Failed to parse PowerPoint package: {e}"))?;
                let count = pkg.slide_count();
                office_state.ppt_slides = (1..=(count.max(1) as u32)).collect();
                office_state.unparsed_store = Some(pkg.unparsed_store);
                office_state.raw_package_bytes = Some(bytes.to_vec());
            }
            DocumentType::Pdf => {
                let text = match pdf_engine::PdfDocument::parse(bytes.to_vec()) {
                    Ok(pdf) => {
                        let count = pdf.page_count();
                        format!("PDF Document ({count} pages)")
                    }
                    Err(_) => "PDF Document (Raw Bytes)".to_string(),
                };
                office_state.word_content = text;
                office_state.raw_package_bytes = Some(bytes.to_vec());
            }
        }

        let mut summary = self.session_manager.create_session(doc_type, file_name);
        summary.file_path = Some(file_name.to_string());
        summary.is_dirty = false;

        self.session_manager
            .office_states
            .insert(summary.session_id.clone(), office_state);

        if let Some(tab) = self
            .session_manager
            .tab_bar
            .tabs
            .iter_mut()
            .find(|t| t.id == summary.session_id)
        {
            tab.file_path = Some(file_name.to_string());
            tab.is_dirty = false;
        }

        Ok(summary)
    }

    /// Dispatches a typed user action event into the document engine.
    pub fn dispatch_action(
        &mut self,
        session_id: &str,
        action: UiActionEvent,
    ) -> Result<UiDispatchResult, String> {
        UiCommandDispatcher::dispatch(&mut self.session_manager, session_id, action)
    }

    /// Serializes the active document session into standard binary package bytes.
    pub fn save_document_bytes(&mut self, session_id: &str) -> Result<Vec<u8>, String> {
        let doc_type = self
            .session_manager
            .get_session(session_id)
            .map(|s| s.doc_type)
            .ok_or_else(|| format!("Session '{session_id}' not found"))?;

        let office_state = self
            .session_manager
            .office_states
            .get(session_id)
            .cloned()
            .unwrap_or_else(GlobalOfficeState::new);

        let bytes = match doc_type {
            DocumentType::Word => DocumentIoEngine::serialize_word_package_to_bytes(&office_state)?,
            DocumentType::Excel => {
                DocumentIoEngine::serialize_excel_package_to_bytes(&office_state)?
            }
            DocumentType::PowerPoint => {
                DocumentIoEngine::serialize_ppt_package_to_bytes(&office_state)?
            }
            DocumentType::Pdf => {
                let title = self
                    .session_manager
                    .get_session(session_id)
                    .map(|s| s.title.as_str())
                    .unwrap_or("Document.pdf");
                let body = if !office_state.word_content.is_empty() {
                    &office_state.word_content
                } else {
                    "OpenOfficeRS Native PDF Document"
                };
                DocumentIoEngine::build_iso_pdf_bytes(title, body)
            }
        };

        let _ = self.session_manager.mark_session_clean(session_id);
        Ok(bytes)
    }

    /// Generates a presentation SVG visualization for the specified session.
    pub fn export_svg(&self, session_id: &str) -> Result<String, String> {
        let session = self
            .session_manager
            .get_session(session_id)
            .ok_or_else(|| format!("Session '{session_id}' not found"))?;

        let office_state = self
            .session_manager
            .office_states
            .get(session_id)
            .cloned()
            .unwrap_or_else(GlobalOfficeState::new);

        let preview_text = match session.doc_type {
            DocumentType::Word => {
                if office_state.word_content.is_empty() {
                    "Document Body".to_string()
                } else {
                    office_state.word_content.chars().take(80).collect()
                }
            }
            DocumentType::Excel => format!(
                "Sheets: {} | Cells: {}",
                office_state.excel_sheets.join(", "),
                office_state.excel_cells.len()
            ),
            DocumentType::PowerPoint => {
                format!("Presentation ({} slides)", office_state.ppt_slides.len())
            }
            DocumentType::Pdf => {
                if office_state.word_content.is_empty() {
                    "PDF Document Preview".to_string()
                } else {
                    office_state.word_content.clone()
                }
            }
        };

        let escaped_text = preview_text.replace('&', "&amp;").replace('<', "&lt;");
        let escaped_title = session.title.replace('&', "&amp;").replace('<', "&lt;");

        Ok(format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 800 1100\" data-session-id=\"{session_id}\">\
                <rect width=\"100%\" height=\"100%\" fill=\"#ffffff\"/>\
                <rect x=\"30\" y=\"30\" width=\"740\" height=\"60\" rx=\"6\" fill=\"#f3f4f6\" stroke=\"#e5e7eb\"/>\
                <text x=\"50\" y=\"68\" font-family=\"Segoe UI, sans-serif\" font-size=\"18\" font-weight=\"600\" fill=\"#111827\">{escaped_title}</text>\
                <text x=\"50\" y=\"140\" font-family=\"Segoe UI, sans-serif\" font-size=\"14\" fill=\"#374151\">{escaped_text}</text>\
            </svg>"
        ))
    }

    /// Performs zero-loss OPC package roundtrip preservation.
    pub fn roundtrip_opc(
        &self,
        format: &str,
        original_bytes: &[u8],
        updates_json: &str,
    ) -> Result<Vec<u8>, String> {
        match format.to_ascii_lowercase().as_str() {
            "docx" | "word" => DocumentIoEngine::roundtrip_docx_bytes(original_bytes, updates_json),
            "xlsx" | "excel" => {
                let cell_map: std::collections::HashMap<String, String> =
                    serde_json::from_str(updates_json)
                        .map_err(|e| format!("Invalid cell updates JSON map: {e}"))?;
                DocumentIoEngine::roundtrip_xlsx_bytes(original_bytes, &cell_map)
            }
            "pptx" | "ppt" => DocumentIoEngine::roundtrip_pptx_bytes(original_bytes),
            _ => Err(format!("Unsupported OPC format: '{format}'")),
        }
    }

    /// Retrieves session metadata information.
    pub fn get_document_info(&self, session_id: &str) -> Result<DocumentMetadata, String> {
        DocumentIoEngine::get_document_info(&self.session_manager, session_id)
    }

    /// Closes an active document session.
    pub fn close_session(
        &mut self,
        session_id: &str,
        force: bool,
    ) -> Result<Option<String>, String> {
        self.session_manager
            .close_session(session_id, force)
            .map_err(|e| e.to_string())
    }

    /// Executes undo on the session command bus.
    pub fn undo(&mut self, session_id: &str) -> Result<(), String> {
        self.session_manager.undo_session(session_id)
    }

    /// Executes redo on the session command bus.
    pub fn redo(&mut self, session_id: &str) -> Result<(), String> {
        self.session_manager.redo_session(session_id)
    }
}

// -----------------------------------------------------------------------------
// Global WASM Bridge Single-Instance & JSON FFI API
// -----------------------------------------------------------------------------

static GLOBAL_WASM_ENGINE: OnceLock<Mutex<WasmOfficeEngine>> = OnceLock::new();

fn global_engine() -> Result<std::sync::MutexGuard<'static, WasmOfficeEngine>, String> {
    let mutex = GLOBAL_WASM_ENGINE.get_or_init(|| Mutex::new(WasmOfficeEngine::new()));
    mutex
        .lock()
        .map_err(|e| format!("Engine lock poisoned: {e}"))
}

/// Helper resolving string representations to typed DocumentType.
pub fn parse_document_type(doc_type_str: &str) -> DocumentType {
    let lower = doc_type_str.trim().to_ascii_lowercase();
    match lower.as_str() {
        "word" | "docx" | "doc" | "writer" => DocumentType::Word,
        "excel" | "xlsx" | "xls" | "calc" | "csv" => DocumentType::Excel,
        "powerpoint" | "pptx" | "ppt" | "slides" => DocumentType::PowerPoint,
        "pdf" => DocumentType::Pdf,
        _ => DocumentType::Word,
    }
}

/// Creates a new document session and returns serialized `SessionSummary` JSON.
pub fn wasm_create_session(doc_type_str: &str, title: &str) -> Result<String, String> {
    let mut engine = global_engine()?;
    let summary = engine.create_session(doc_type_str, title)?;
    serde_json::to_string(&summary)
        .map_err(|e| format!("Failed to serialize SessionSummary JSON: {e}"))
}

/// Ingests binary package bytes and returns serialized `SessionSummary` JSON.
pub fn wasm_open_document(bytes: &[u8], file_name: &str) -> Result<String, String> {
    let mut engine = global_engine()?;
    let summary = engine.open_document_bytes(bytes, file_name)?;
    serde_json::to_string(&summary)
        .map_err(|e| format!("Failed to serialize SessionSummary JSON: {e}"))
}

/// Dispatches a JSON-encoded `UiActionEvent` and returns serialized `UiDispatchResult` JSON.
pub fn wasm_dispatch_action(session_id: &str, action_json: &str) -> Result<String, String> {
    let action: UiActionEvent = serde_json::from_str(action_json)
        .map_err(|e| format!("Failed to deserialize UiActionEvent JSON: {e}"))?;
    let mut engine = global_engine()?;
    let result = engine.dispatch_action(session_id, action)?;
    serde_json::to_string(&result)
        .map_err(|e| format!("Failed to serialize UiDispatchResult JSON: {e}"))
}

/// Serializes the document session and returns package bytes.
pub fn wasm_save_document(session_id: &str) -> Result<Vec<u8>, String> {
    let mut engine = global_engine()?;
    engine.save_document_bytes(session_id)
}

/// Exports an SVG representation for the document session.
pub fn wasm_export_svg(session_id: &str) -> Result<String, String> {
    let engine = global_engine()?;
    engine.export_svg(session_id)
}

/// Performs zero-loss in-memory OPC package updates.
pub fn wasm_roundtrip_opc(
    format: &str,
    original_bytes: &[u8],
    updates_json: &str,
) -> Result<Vec<u8>, String> {
    let engine = global_engine()?;
    engine.roundtrip_opc(format, original_bytes, updates_json)
}

/// Retrieves document session metadata in JSON format.
pub fn wasm_get_document_info(session_id: &str) -> Result<String, String> {
    let engine = global_engine()?;
    let info = engine.get_document_info(session_id)?;
    serde_json::to_string(&info)
        .map_err(|e| format!("Failed to serialize DocumentMetadata JSON: {e}"))
}

/// Closes an active document session.
pub fn wasm_close_session(session_id: &str, force: bool) -> Result<Option<String>, String> {
    let mut engine = global_engine()?;
    engine.close_session(session_id, force)
}

/// Executes undo on the session.
pub fn wasm_undo(session_id: &str) -> Result<(), String> {
    let mut engine = global_engine()?;
    engine.undo(session_id)
}

/// Executes redo on the session.
pub fn wasm_redo(session_id: &str) -> Result<(), String> {
    let mut engine = global_engine()?;
    engine.redo(session_id)
}
