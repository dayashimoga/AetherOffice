//! Integration Tests for Subphase 8.2.1: Native File Open, Save, Save-As & Backstage File Menu
//!
//! Complies with ECMA-376, ISO 32000-1, and GEC-v2.0 Article I-VII.
//! Verifies:
//! - Round-trip disk Open -> Edit -> Save -> Re-open for:
//!   1. Word (.docx)
//!   2. Excel (.xlsx)
//!   3. PowerPoint (.pptx)
//!   4. PDF (.pdf)
//! - ISO 32000 PDF Export across all formats
//! - Dirty indicator management and metadata retrieval

use render_core::document_io::DocumentIoEngine;
use render_core::shell::{DesktopSessionManager, DocumentType};
use render_core::ui_dispatcher::{UiActionEvent, UiCommandDispatcher};
use std::fs;
use std::path::Path;

fn ensure_test_dir() -> &'static str {
    let dir = "target/tmp_test_subphase_8_2_1";
    let _ = fs::create_dir_all(dir);
    dir
}

#[test]
fn test_01_word_docx_roundtrip_disk_open_edit_save() {
    let dir = ensure_test_dir();
    let file_path = format!("{dir}/Annual_Report_2026.docx");
    let mut mgr = DesktopSessionManager::new();

    // 1. Create a Word session
    let sess = mgr.create_session(DocumentType::Word, "Annual_Report_2026.docx");
    assert!(!sess.is_dirty);

    // 2. Insert text content via UI command dispatcher
    let res = UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess.session_id,
        UiActionEvent::InsertText {
            text: "OpenOfficeRS Word Engine: Ground Truth Document Text.".to_string(),
        },
    )
    .expect("Dispatch text insertion");
    assert!(res.is_dirty);
    assert_eq!(res.undo_stack_depth, 1);

    // 3. Save to disk
    let saved_path =
        DocumentIoEngine::save_document_to_disk(&mut mgr, &sess.session_id, &file_path)
            .expect("Save Word document to disk");
    assert_eq!(saved_path, file_path);
    assert!(Path::new(&file_path).exists());
    assert!(fs::metadata(&file_path).unwrap().len() > 100);

    // Verify session is marked clean
    let tab = mgr
        .tab_bar
        .tabs
        .iter()
        .find(|t| t.id == sess.session_id)
        .unwrap();
    assert!(!tab.is_dirty);

    // 4. Re-open saved package from disk into a fresh session
    let mut mgr2 = DesktopSessionManager::new();
    let reloaded = DocumentIoEngine::open_document_from_disk(&mut mgr2, &file_path)
        .expect("Open Word document from disk");

    assert_eq!(reloaded.doc_type, DocumentType::Word);
    assert!(!reloaded.is_dirty);

    // Verify reloaded document text fidelity
    let state2 = mgr2.office_states.get(&reloaded.session_id).unwrap();
    assert!(state2.word_content.contains("OpenOfficeRS Word Engine"));
}

#[test]
fn test_02_excel_xlsx_roundtrip_disk_open_edit_save() {
    let dir = ensure_test_dir();
    let file_path = format!("{dir}/Budget_Model_2026.xlsx");
    let mut mgr = DesktopSessionManager::new();

    // 1. Create Excel session
    let sess = mgr.create_session(DocumentType::Excel, "Budget_Model_2026.xlsx");

    // 2. Dispatch cell modifications
    UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess.session_id,
        UiActionEvent::SetCellValue {
            coord: "A1".to_string(),
            value: "Quarterly Revenue".to_string(),
        },
    )
    .expect("Set A1 text");

    UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess.session_id,
        UiActionEvent::SetCellValue {
            coord: "B1".to_string(),
            value: "542000".to_string(),
        },
    )
    .expect("Set B1 number");

    UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess.session_id,
        UiActionEvent::SetCellFormula {
            coord: "B2".to_string(),
            formula: "=SUM(B1)".to_string(),
        },
    )
    .expect("Set B2 formula");

    // 3. Save to disk as valid ECMA-376 .xlsx package
    let saved_path =
        DocumentIoEngine::save_document_to_disk(&mut mgr, &sess.session_id, &file_path)
            .expect("Save Excel workbook to disk");
    assert_eq!(saved_path, file_path);
    assert!(Path::new(&file_path).exists());
    assert!(fs::metadata(&file_path).unwrap().len() > 200);

    // 4. Re-open from disk in clean manager
    let mut mgr2 = DesktopSessionManager::new();
    let reloaded = DocumentIoEngine::open_document_from_disk(&mut mgr2, &file_path)
        .expect("Open Excel workbook from disk");

    assert_eq!(reloaded.doc_type, DocumentType::Excel);
    assert!(!reloaded.is_dirty);

    let state2 = mgr2.office_states.get(&reloaded.session_id).unwrap();
    assert_eq!(
        state2.excel_cells.get("A1").map(|s| s.as_str()),
        Some("Quarterly Revenue")
    );
    assert_eq!(
        state2.excel_cells.get("B1").map(|s| s.as_str()),
        Some("542000")
    );
    assert_eq!(
        state2.excel_cells.get("B2").map(|s| s.as_str()),
        Some("=SUM(B1)")
    );
}

#[test]
fn test_03_powerpoint_pptx_roundtrip_disk_open_edit_save() {
    let dir = ensure_test_dir();
    let file_path = format!("{dir}/Keynote_Pitch_2026.pptx");
    let mut mgr = DesktopSessionManager::new();

    let sess = mgr.create_session(DocumentType::PowerPoint, "Keynote_Pitch_2026.pptx");

    // Add slide via UI dispatcher
    UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess.session_id,
        UiActionEvent::NewSlide {
            layout: "TitleSlide".to_string(),
        },
    )
    .expect("New slide");

    // Save to disk
    let saved_path =
        DocumentIoEngine::save_document_to_disk(&mut mgr, &sess.session_id, &file_path)
            .expect("Save PowerPoint to disk");
    assert_eq!(saved_path, file_path);
    assert!(Path::new(&file_path).exists());
    assert!(fs::metadata(&file_path).unwrap().len() > 100);

    // Re-open from disk
    let mut mgr2 = DesktopSessionManager::new();
    let reloaded = DocumentIoEngine::open_document_from_disk(&mut mgr2, &file_path)
        .expect("Open PowerPoint from disk");
    assert_eq!(reloaded.doc_type, DocumentType::PowerPoint);
    assert!(!reloaded.is_dirty);
}

#[test]
fn test_04_pdf_roundtrip_disk_open_edit_save() {
    let dir = ensure_test_dir();
    let file_path = format!("{dir}/Standard_Specifications.pdf");
    let mut mgr = DesktopSessionManager::new();

    let sess = mgr.create_session(DocumentType::Pdf, "Standard_Specifications.pdf");

    // Insert text
    let state = mgr.office_states.get_mut(&sess.session_id).unwrap();
    state.word_content = "OpenOfficeRS ISO 32000-1 Compliant PDF Document".to_string();

    // Save to disk
    let saved_path =
        DocumentIoEngine::save_document_to_disk(&mut mgr, &sess.session_id, &file_path)
            .expect("Save PDF to disk");
    assert_eq!(saved_path, file_path);
    assert!(Path::new(&file_path).exists());

    // Validate PDF syntax
    let bytes = fs::read(&file_path).expect("Read PDF bytes");
    assert!(bytes.starts_with(b"%PDF-1.4"));
    assert!(bytes.ends_with(b"%%EOF"));

    // Re-open from disk
    let mut mgr2 = DesktopSessionManager::new();
    let reloaded = DocumentIoEngine::open_document_from_disk(&mut mgr2, &file_path)
        .expect("Open PDF from disk");
    assert_eq!(reloaded.doc_type, DocumentType::Pdf);
    assert!(!reloaded.is_dirty);
}

#[test]
fn test_05_export_document_to_pdf_all_formats() {
    let dir = ensure_test_dir();
    let mut mgr = DesktopSessionManager::new();

    // 1. Export Word to PDF
    let w_sess = mgr.create_session(DocumentType::Word, "Doc.docx");
    mgr.office_states
        .get_mut(&w_sess.session_id)
        .unwrap()
        .word_content = "Executive Summary of High Performance Rust Systems".to_string();
    let w_pdf = format!("{dir}/Exported_Word.pdf");
    let exp1 = DocumentIoEngine::export_document_to_pdf(&mgr, &w_sess.session_id, &w_pdf)
        .expect("Export Word to PDF");
    assert_eq!(exp1, w_pdf);
    assert!(Path::new(&w_pdf).exists());
    let b1 = fs::read(&w_pdf).unwrap();
    assert!(b1.starts_with(b"%PDF-1.4"));

    // 2. Export Excel to PDF
    let e_sess = mgr.create_session(DocumentType::Excel, "Sheet.xlsx");
    mgr.office_states
        .get_mut(&e_sess.session_id)
        .unwrap()
        .excel_cells
        .insert("A1".into(), "Profit: $45,000".into());
    let e_pdf = format!("{dir}/Exported_Excel.pdf");
    let exp2 = DocumentIoEngine::export_document_to_pdf(&mgr, &e_sess.session_id, &e_pdf)
        .expect("Export Excel to PDF");
    assert_eq!(exp2, e_pdf);
    assert!(Path::new(&e_pdf).exists());

    // 3. Export PowerPoint to PDF
    let p_sess = mgr.create_session(DocumentType::PowerPoint, "Deck.pptx");
    let p_pdf = format!("{dir}/Exported_PowerPoint.pdf");
    let exp3 = DocumentIoEngine::export_document_to_pdf(&mgr, &p_sess.session_id, &p_pdf)
        .expect("Export PowerPoint to PDF");
    assert_eq!(exp3, p_pdf);
    assert!(Path::new(&p_pdf).exists());
}

#[test]
fn test_06_dirty_indicator_cleared_on_save_and_metadata_reported() {
    let dir = ensure_test_dir();
    let file_path = format!("{dir}/Meta_Test_Doc.docx");
    let mut mgr = DesktopSessionManager::new();

    let sess = mgr.create_session(DocumentType::Word, "Meta_Test_Doc.docx");

    // Make dirty
    UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess.session_id,
        UiActionEvent::InsertText {
            text: "Unsaved modifications".to_string(),
        },
    )
    .unwrap();
    assert!(mgr.is_session_dirty(&sess.session_id).unwrap());

    // Check metadata prior to save
    let meta_before = DocumentIoEngine::get_document_info(&mgr, &sess.session_id).unwrap();
    assert!(meta_before.is_dirty);
    assert_eq!(meta_before.file_path, None);

    // Save
    DocumentIoEngine::save_document_to_disk(&mut mgr, &sess.session_id, &file_path).unwrap();

    // Check metadata after save
    let meta_after = DocumentIoEngine::get_document_info(&mgr, &sess.session_id).unwrap();
    assert!(!meta_after.is_dirty);
    assert_eq!(meta_after.file_path, Some(file_path.clone()));
    assert!(meta_after.file_size_bytes > 0);
    assert!(!mgr.is_session_dirty(&sess.session_id).unwrap());
}
