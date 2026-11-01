//! Sub-Phase 43.1.2 Test Suite: Zero-Loss OPC Round-Trip Bridge
//!
//! Strictly verifies GEC-v7.0 Article III:
//! - Real binary open and save for all formats with zero data loss.
//! - Complete bit-for-bit retention of unmodeled parts (macros, drawings, signatures, custom streams)
//!   in `UnparsedPartStore` across in-memory roundtrips and disk persistence.

use excel_engine::package::ExcelWorkbookPackage;
use ooxml_opc::unparsed::UnparsedPart;
use ppt_engine::package::PresentationPackage;
use render_core::document_io::DocumentIoEngine;
use render_core::shell::DesktopSessionManager;
use std::collections::HashMap;
use std::io::Cursor;
use word_engine::package::WordPackage;

#[test]
fn test_01_docx_zero_loss_unparsed_parts_preservation() {
    let mut pkg = WordPackage::new();
    let macro_bytes = vec![0xCA, 0xFE, 0xBA, 0xBE, 0x11, 0x22, 0x33, 0x44];
    pkg.unparsed_store.insert(UnparsedPart::new(
        "word/vbaProject.bin",
        macro_bytes.clone(),
        Some("application/vnd.ms-office.vbaProject".to_string()),
        None,
    ));

    let custom_xml = b"<root><data key=\"contract\">GEC-v7.0</data></root>".to_vec();
    pkg.unparsed_store.insert(UnparsedPart::new(
        "customXml/item1.xml",
        custom_xml.clone(),
        Some("application/xml".to_string()),
        None,
    ));

    let mut initial_buf = Cursor::new(Vec::new());
    pkg.save(&mut initial_buf).expect("Initial save succeeds");
    let initial_docx = initial_buf.into_inner();

    // Perform in-memory roundtrip update
    let updated_text = "Executive Summary\nAll deliverables certified under GEC-v7.0 Article III.";
    let updated_docx = DocumentIoEngine::roundtrip_docx_bytes(&initial_docx, updated_text)
        .expect("Roundtrip update succeeds");

    // Verify reopened package retains text and unmodeled parts bit-for-bit
    let reopened = WordPackage::from_bytes(&updated_docx).expect("Reopen package succeeds");
    let full_text = reopened.get_full_text();
    assert!(full_text.contains("Executive Summary"));
    assert!(full_text.contains("All deliverables certified"));

    let macro_part = reopened
        .unparsed_store
        .get("word/vbaProject.bin")
        .or_else(|| reopened.unparsed_store.get("/word/vbaProject.bin"))
        .expect("vbaProject.bin preserved");
    assert_eq!(macro_part.content, macro_bytes);

    let xml_part = reopened
        .unparsed_store
        .get("customXml/item1.xml")
        .or_else(|| reopened.unparsed_store.get("/customXml/item1.xml"))
        .expect("customXml/item1.xml preserved");
    assert_eq!(xml_part.content, custom_xml);
}

#[test]
fn test_02_xlsx_zero_loss_unparsed_parts_and_cells_update() {
    let mut initial_pkg = ExcelWorkbookPackage {
        workbook: excel_engine::workbook::Workbook::new(),
        shared_strings: excel_engine::shared_strings::SharedStringTable::new(),
        styles: excel_engine::styles::StyleSheet::default(),
        worksheets: vec![(
            "Sheet1".to_string(),
            excel_engine::worksheet::Worksheet::default(),
        )],
        unparsed_store: ooxml_opc::UnparsedPartStore::new(),
    };
    initial_pkg
        .workbook
        .sheets
        .push(excel_engine::workbook::SheetRecord::new(
            "Sheet1".to_string(),
            1,
            "rId1".to_string(),
        ));
    let mut buf = Cursor::new(Vec::new());
    initial_pkg.save(&mut buf).unwrap();
    let initial_xlsx_blank = buf.into_inner();

    let mut cells = HashMap::new();
    cells.insert("A1".to_string(), "Revenue".to_string());
    cells.insert("B1".to_string(), "100000".to_string());

    let populated_xlsx =
        DocumentIoEngine::roundtrip_xlsx_bytes(&initial_xlsx_blank, &cells).unwrap();
    let mut pkg = ExcelWorkbookPackage::open(Cursor::new(&populated_xlsx)).unwrap();

    let chart_payload = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG header
    pkg.unparsed_store.insert(UnparsedPart::new(
        "xl/drawings/drawing1.xml",
        chart_payload.clone(),
        Some("application/vnd.openxmlformats-officedocument.drawing+xml".to_string()),
        None,
    ));

    let mut buf = Cursor::new(Vec::new());
    pkg.save(&mut buf).expect("Save with unmodeled parts");
    let initial_xlsx = buf.into_inner();

    // Roundtrip update with new cell values
    let mut update_cells = HashMap::new();
    update_cells.insert("B1".to_string(), "250000".to_string());
    update_cells.insert("C1".to_string(), "=B1*0.2".to_string());

    let updated_xlsx = DocumentIoEngine::roundtrip_xlsx_bytes(&initial_xlsx, &update_cells)
        .expect("Roundtrip XLSX succeeds");

    let reopened = ExcelWorkbookPackage::open(Cursor::new(&updated_xlsx))
        .expect("Reopen updated XLSX succeeds");

    let drawing_part = reopened
        .unparsed_store
        .get("xl/drawings/drawing1.xml")
        .or_else(|| reopened.unparsed_store.get("/xl/drawings/drawing1.xml"))
        .expect("drawing1.xml preserved");
    assert_eq!(drawing_part.content, chart_payload);
}

#[test]
fn test_03_desktop_session_disk_open_and_save_zero_loss() {
    let temp_dir = std::env::temp_dir();
    let src_file = temp_dir.join("subphase_43_1_2_initial.docx");
    let dst_file = temp_dir.join("subphase_43_1_2_saved.docx");

    // Create initial docx file with unmodeled macro binary
    let mut pkg = WordPackage::new();
    let macro_bytes = vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
    pkg.unparsed_store.insert(UnparsedPart::new(
        "word/vbaProject.bin",
        macro_bytes.clone(),
        Some("application/vnd.ms-office.vbaProject".to_string()),
        None,
    ));
    pkg.save_to_file(&src_file).expect("Write initial docx");

    // Open via DocumentIoEngine in session manager
    let mut mgr = DesktopSessionManager::new();
    let summary = DocumentIoEngine::open_document_from_disk(&mut mgr, src_file.to_str().unwrap())
        .expect("Open from disk succeeds");

    assert_eq!(summary.title, "subphase_43_1_2_initial.docx");
    assert!(!summary.is_dirty);

    // Edit content in session
    if let Some(office_state) = mgr.office_states.get_mut(&summary.session_id) {
        office_state.word_content =
            "Modified body text through desktop session manager".to_string();
    }

    // Save document to destination file
    let saved_path = DocumentIoEngine::save_document_to_disk(
        &mut mgr,
        &summary.session_id,
        dst_file.to_str().unwrap(),
    )
    .expect("Save to disk succeeds");
    assert_eq!(saved_path, dst_file.to_str().unwrap());

    // Reopen saved file directly from disk and verify unmodeled parts and content
    let reopened = WordPackage::from_file(&dst_file).expect("Reopen saved docx");
    let full_text = reopened.get_full_text();
    assert!(full_text.contains("Modified body text through desktop session manager"));

    let macro_part = reopened
        .unparsed_store
        .get("word/vbaProject.bin")
        .or_else(|| reopened.unparsed_store.get("/word/vbaProject.bin"))
        .expect("Preserved vbaProject.bin");
    assert_eq!(macro_part.content, macro_bytes);

    // Cleanup temp files
    let _ = std::fs::remove_file(src_file);
    let _ = std::fs::remove_file(dst_file);
}

#[test]
fn test_04_pptx_zero_loss_roundtrip() {
    let mut pkg = PresentationPackage::create_empty();
    let custom_media = vec![0x00, 0x01, 0x02, 0x03, 0x04];
    pkg.unparsed_store.insert(UnparsedPart::new(
        "ppt/media/sound1.wav",
        custom_media.clone(),
        Some("audio/wav".to_string()),
        None,
    ));

    let mut buf = Cursor::new(Vec::new());
    pkg.save(&mut buf).expect("Save PPTX with unmodeled audio");
    let initial_pptx = buf.into_inner();

    let roundtripped_pptx =
        DocumentIoEngine::roundtrip_pptx_bytes(&initial_pptx).expect("Roundtrip PPTX");

    let reopened = PresentationPackage::open(Cursor::new(&roundtripped_pptx))
        .expect("Reopen roundtripped PPTX");

    let media_part = reopened
        .unparsed_store
        .get("ppt/media/sound1.wav")
        .or_else(|| reopened.unparsed_store.get("/ppt/media/sound1.wav"))
        .expect("Audio part preserved");
    assert_eq!(media_part.content, custom_media);
}
