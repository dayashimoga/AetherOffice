//! Sub-Phase 43.2.2 Test Suite: WASM Core Compilation for Browser/Test Fallback
//!
//! Strictly verifies GEC-v7.0 Article I, III, IV, and VI:
//! - Pure safe Rust WASM bridge API with zero panics.
//! - In-memory multi-document session orchestration across Word, Excel, PowerPoint, and PDF.
//! - Action dispatch, undo/redo, SVG visualization, and lossless binary serialization.

use excel_engine::package::ExcelWorkbookPackage;
use ooxml_opc::unparsed::UnparsedPart;
use render_core::wasm_bridge::{
    wasm_close_session, wasm_create_session, wasm_dispatch_action, wasm_export_svg,
    wasm_get_document_info, wasm_open_document, wasm_roundtrip_opc, wasm_save_document,
    WasmOfficeEngine,
};
use render_core::{DocumentType, SessionSummary, UiActionEvent};
use std::collections::HashMap;
use std::io::Cursor;
use word_engine::package::WordPackage;

#[test]
fn test_01_wasm_create_session_and_svg_export() {
    let mut engine = WasmOfficeEngine::new();

    let docx_summary = engine
        .create_session("word", "QuarterlyReport.docx")
        .expect("Word session creation succeeds");
    assert_eq!(docx_summary.doc_type, DocumentType::Word);
    assert_eq!(docx_summary.title, "QuarterlyReport.docx");
    assert!(!docx_summary.is_dirty);

    let docx_svg = engine
        .export_svg(&docx_summary.session_id)
        .expect("SVG export succeeds");
    assert!(docx_svg.contains("<svg"));
    assert!(docx_svg.contains(&docx_summary.session_id));
    assert!(docx_svg.contains("QuarterlyReport.docx"));

    let xlsx_summary = engine
        .create_session("excel", "Budget2026.xlsx")
        .expect("Excel session creation succeeds");
    assert_eq!(xlsx_summary.doc_type, DocumentType::Excel);

    let xlsx_svg = engine
        .export_svg(&xlsx_summary.session_id)
        .expect("Excel SVG succeeds");
    assert!(xlsx_svg.contains("Budget2026.xlsx"));

    let pptx_summary = engine
        .create_session("powerpoint", "Keynote.pptx")
        .expect("PPTX session creation succeeds");
    assert_eq!(pptx_summary.doc_type, DocumentType::PowerPoint);

    let pptx_svg = engine
        .export_svg(&pptx_summary.session_id)
        .expect("PPTX SVG succeeds");
    assert!(pptx_svg.contains("Keynote.pptx"));

    let pdf_summary = engine
        .create_session("pdf", "Specification.pdf")
        .expect("PDF session creation succeeds");
    assert_eq!(pdf_summary.doc_type, DocumentType::Pdf);

    let info = engine
        .get_document_info(&docx_summary.session_id)
        .expect("Get document info succeeds");
    assert_eq!(info.session_id, docx_summary.session_id);
    assert_eq!(info.title, "QuarterlyReport.docx");
}

#[test]
fn test_02_wasm_open_document_and_save_bytes() {
    let mut engine = WasmOfficeEngine::new();

    // 1. Create a Word package with unparsed macro part
    let mut word_pkg = WordPackage::new();
    let macro_bytes = vec![0xCA, 0xFE, 0xBA, 0xBE, 0x42];
    word_pkg.unparsed_store.insert(UnparsedPart::new(
        "word/vbaProject.bin",
        macro_bytes.clone(),
        Some("application/vnd.ms-office.vbaProject".to_string()),
        None,
    ));

    let mut buf = Cursor::new(Vec::new());
    word_pkg.save(&mut buf).expect("Save word package succeeds");
    let docx_bytes = buf.into_inner();

    // 2. Open through WASM engine
    let opened = engine
        .open_document_bytes(&docx_bytes, "MacroDoc.docx")
        .expect("WASM open succeeds");
    assert_eq!(opened.doc_type, DocumentType::Word);
    assert_eq!(opened.title, "MacroDoc.docx");

    // 3. Dispatch an edit action
    let dispatch_res = engine
        .dispatch_action(
            &opened.session_id,
            UiActionEvent::InsertText {
                text: "Edited in WASM environment.".to_string(),
            },
        )
        .expect("Dispatch action succeeds");
    assert!(dispatch_res.is_dirty);

    // 4. Serialize back to package bytes
    let saved_bytes = engine
        .save_document_bytes(&opened.session_id)
        .expect("Save document bytes succeeds");
    assert!(!saved_bytes.is_empty());

    // 5. Verify unparsed part is preserved
    let parsed_back = WordPackage::from_bytes(&saved_bytes).expect("Parsed saved package succeeds");
    let vba_part = parsed_back.unparsed_store.get("word/vbaProject.bin");
    assert!(vba_part.is_some());
    assert_eq!(vba_part.unwrap().content, macro_bytes);
}

#[test]
fn test_03_wasm_action_dispatch_and_undo_redo() {
    let mut engine = WasmOfficeEngine::new();

    let session = engine
        .create_session("word", "UndoTest.docx")
        .expect("Session creation succeeds");

    // Action 1: Toggle Bold
    let r1 = engine
        .dispatch_action(&session.session_id, UiActionEvent::ToggleBold)
        .expect("Toggle bold succeeds");
    assert_eq!(r1.undo_stack_depth, 1);
    assert_eq!(r1.redo_stack_depth, 0);

    // Action 2: Toggle Italic
    let r2 = engine
        .dispatch_action(&session.session_id, UiActionEvent::ToggleItalic)
        .expect("Toggle italic succeeds");
    assert_eq!(r2.undo_stack_depth, 2);

    // Undo Action 2
    engine.undo(&session.session_id).expect("Undo succeeds");

    // Redo Action 2
    engine.redo(&session.session_id).expect("Redo succeeds");

    // Close session
    let closed = engine
        .close_session(&session.session_id, true)
        .expect("Close session succeeds");
    assert!(closed.is_none());
}

#[test]
fn test_04_wasm_global_json_api_functions() {
    // 1. Create session via JSON API
    let summary_json =
        wasm_create_session("excel", "GlobalFinance.xlsx").expect("wasm_create_session succeeds");
    let summary: SessionSummary =
        serde_json::from_str(&summary_json).expect("Parse session summary JSON");
    assert_eq!(summary.doc_type, DocumentType::Excel);
    assert_eq!(summary.title, "GlobalFinance.xlsx");

    // 2. Export SVG via JSON API
    let svg = wasm_export_svg(&summary.session_id).expect("wasm_export_svg succeeds");
    assert!(svg.contains("GlobalFinance.xlsx"));

    // 3. Dispatch action via JSON API
    let action_json = serde_json::to_string(&UiActionEvent::SetCellValue {
        coord: "A1".to_string(),
        value: "1000000".to_string(),
    })
    .expect("Serialize action JSON");

    let dispatch_json = wasm_dispatch_action(&summary.session_id, &action_json)
        .expect("wasm_dispatch_action succeeds");
    assert!(dispatch_json.contains("Set Cell Value"));

    // 4. Get document info via JSON API
    let info_json =
        wasm_get_document_info(&summary.session_id).expect("wasm_get_document_info succeeds");
    assert!(info_json.contains("GlobalFinance.xlsx"));

    // 5. Save document via JSON API
    let saved_bytes = wasm_save_document(&summary.session_id).expect("wasm_save_document succeeds");
    assert!(!saved_bytes.is_empty());

    // 6. Test open document via bytes JSON API
    let opened_from_bytes_json = wasm_open_document(&saved_bytes, "ReopenedFinance.xlsx")
        .expect("wasm_open_document succeeds");
    assert!(opened_from_bytes_json.contains("ReopenedFinance.xlsx"));

    // 7. Test roundtrip OPC
    let orig_xlsx = ExcelWorkbookPackage {
        workbook: excel_engine::workbook::Workbook::new(),
        shared_strings: excel_engine::shared_strings::SharedStringTable::new(),
        styles: excel_engine::styles::StyleSheet::default(),
        worksheets: vec![(
            "Sheet1".to_string(),
            excel_engine::worksheet::Worksheet::default(),
        )],
        unparsed_store: ooxml_opc::UnparsedPartStore::new(),
    };
    let mut xlsx_buf = Cursor::new(Vec::new());
    orig_xlsx
        .save(&mut xlsx_buf)
        .expect("Save empty xlsx succeeds");
    let orig_bytes = xlsx_buf.into_inner();

    let mut cell_updates = HashMap::new();
    cell_updates.insert("B2".to_string(), "WASM_CELL_VALUE".to_string());
    let updates_json = serde_json::to_string(&cell_updates).expect("Serialize cell map");

    let roundtripped = wasm_roundtrip_opc("xlsx", &orig_bytes, &updates_json)
        .expect("wasm_roundtrip_opc succeeds");
    assert!(!roundtripped.is_empty());

    // 8. Close session
    let closed =
        wasm_close_session(&summary.session_id, true).expect("wasm_close_session succeeds");
    assert!(closed.is_some());
}
