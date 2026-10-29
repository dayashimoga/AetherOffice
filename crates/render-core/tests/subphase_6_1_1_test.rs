//! Subphase 6.1.1 Integration Test Suite:
//! Transactional Command Bus with Global Undo/Redo & Universal Clipboard
//!
//! Validates:
//! - Global office state initialization across Word, Excel, PowerPoint, and PDF
//! - Reversible transactions with undo, redo, and state rollback
//! - Keystroke typing coalescing and object drag coalescing
//! - Monotonic revision clean-point save tracking (`is_dirty()`, `mark_clean()`)
//! - History inspection and bounded undo stack depth
//! - Compound atomic multi-app transactions
//! - Multi-format W3C-compliant clipboard payloads (Word, Excel, PDF, PNG)
//! - Cross-app lossless conversion (TSV <-> HTML Table <-> Word Table XML)
//! - Universal clipboard ring-buffer history
//! - DesktopSessionManager multi-document PDF lifecycle and ribbon tab sets
//! - CommandDispatcher bus integration
//! - Sub-millisecond transaction execution and undo/redo latency SLA (< 1.0 ms)

use render_core::commands::{
    grid_to_tsv, html_table_to_tsv, tsv_to_grid, tsv_to_html_table, tsv_to_word_table_xml,
    word_table_xml_to_tsv, CommandDispatcher, CompoundOfficeTransaction, ExcelAction,
    ExcelAddSheetTransaction, ExcelPasteRangeTransaction, ExcelSetCellTransaction, GlobalClipboard,
    GlobalClipboardPayload, GlobalCommandBus, GlobalOfficeState, OfficeCommand, PdfAction,
    PdfEditTextTransaction, PdfMoveObjectTransaction, PdfRotatePageTransaction,
    PdfSetFormFieldTransaction, PptAction, PptAddShapeTransaction, PptDeleteShapeTransaction,
    PptMoveShapeTransaction, PptShapeData, WordAction, WordDeleteTextTransaction,
    WordFormatRangeTransaction, WordInsertTextTransaction, WordRangeStyle,
};
use render_core::primitive::Rect2D;
use render_core::shell::{DesktopSessionManager, DocumentType, RibbonModel, RibbonTabId};
use std::time::Instant;

// =========================================================================
// Tests
// =========================================================================

#[test]
fn test_01_global_office_state_initialization() {
    let state = GlobalOfficeState::new();

    // Word state
    assert!(state.word_content.is_empty());
    assert!(state.word_styles.is_empty());

    // Excel state
    assert_eq!(state.excel_sheets, vec!["Sheet1".to_string()]);
    assert!(state.excel_cells.is_empty());
    assert_eq!(state.get_excel_cell("A1"), None);

    // PPT state
    assert_eq!(state.ppt_slides, vec![1]);
    assert!(state.ppt_shapes.is_empty());

    // PDF state
    assert!(state.pdf_text_runs.is_empty());
    assert!(state.pdf_fields.is_empty());
    assert_eq!(state.get_pdf_rotation(0), 0);
    assert_eq!(state.get_pdf_field("FirstName"), None);
}

#[test]
fn test_02_word_insert_and_delete_transactions_with_undo_redo() {
    let mut bus = GlobalCommandBus::new();
    let mut state = GlobalOfficeState::new();

    // 1. Insert "Hello" at offset 0
    let tx1 = Box::new(WordInsertTextTransaction::new(0, "Hello"));
    bus.execute(&mut state, tx1).expect("Insert Hello");
    assert_eq!(state.word_content, "Hello");

    // 2. Insert " World" at offset 5
    let tx2 = Box::new(WordInsertTextTransaction::new(5, " World"));
    bus.execute(&mut state, tx2).expect("Insert World");
    assert_eq!(state.word_content, "Hello World");

    // 3. Delete " World" (offset 5, len 6)
    let tx3 = Box::new(WordDeleteTextTransaction::new(5, " World"));
    bus.execute(&mut state, tx3).expect("Delete World");
    assert_eq!(state.word_content, "Hello");

    // Undo delete -> "Hello World"
    bus.undo(&mut state).expect("Undo delete");
    assert_eq!(state.word_content, "Hello World");

    // Undo insert " World" -> "Hello"
    // (Note: tx2 and tx1 were merged via coalescing if contiguous!)
    // Let's verify undo reverts back to empty
    bus.undo(&mut state).expect("Undo insert");
    assert_eq!(state.word_content, "");

    // Redo restores "Hello World"
    bus.redo(&mut state).expect("Redo insert");
    assert_eq!(state.word_content, "Hello World");
}

#[test]
fn test_03_word_typing_coalescing() {
    let mut bus = GlobalCommandBus::new();
    let mut state = GlobalOfficeState::new();

    // Type "R", "u", "s", "t" character by character
    let chars = ["R", "u", "s", "t"];
    let mut curr_offset = 0;
    for ch in chars {
        let tx = Box::new(WordInsertTextTransaction::new(curr_offset, ch));
        curr_offset += ch.len();
        bus.execute(&mut state, tx).expect("Type char");
    }

    assert_eq!(state.word_content, "Rust");
    // All 4 adjacent keystrokes should coalesce into 1 single transaction on the undo stack!
    assert_eq!(bus.undo_stack_len(), 1);

    // Single undo reverts all coalesced typing
    bus.undo(&mut state).expect("Undo typed word");
    assert_eq!(state.word_content, "");
    assert_eq!(bus.redo_stack_len(), 1);

    // Redo restores entire typed word
    bus.redo(&mut state).expect("Redo typed word");
    assert_eq!(state.word_content, "Rust");
}

#[test]
fn test_04_word_backspace_and_delete_coalescing() {
    let mut bus = GlobalCommandBus::new();
    let mut state = GlobalOfficeState::new();
    state.word_content = "Antigravity".to_string();

    // Backspace 'y' then 't'
    let tx1 = Box::new(WordDeleteTextTransaction::new(10, "y"));
    bus.execute(&mut state, tx1).expect("Backspace y");
    assert_eq!(state.word_content, "Antigravit");

    let tx2 = Box::new(WordDeleteTextTransaction::new(9, "t"));
    bus.execute(&mut state, tx2).expect("Backspace t");
    assert_eq!(state.word_content, "Antigravi");

    // Both backspaces coalesce into 1 transaction
    assert_eq!(bus.undo_stack_len(), 1);

    bus.undo(&mut state).expect("Undo backspaces");
    assert_eq!(state.word_content, "Antigravity");
}

#[test]
fn test_05_word_format_range_transaction() {
    let mut bus = GlobalCommandBus::new();
    let mut state = GlobalOfficeState::new();
    state.word_content = "Bold text here".to_string();

    let style = WordRangeStyle {
        start: 0,
        end: 4,
        font_name: Some("Calibri".into()),
        font_size: Some(12.0),
        bold: Some(true),
        italic: Some(false),
        underline: None,
        color_hex: Some("#FF0000".into()),
    };

    let tx = Box::new(WordFormatRangeTransaction::new(style.clone()));
    bus.execute(&mut state, tx).expect("Apply style");

    assert_eq!(state.word_styles.len(), 1);
    assert_eq!(state.word_styles[0], style);

    // Undo restores previous styles (empty)
    bus.undo(&mut state).expect("Undo style");
    assert!(state.word_styles.is_empty());

    // Redo re-applies style
    bus.redo(&mut state).expect("Redo style");
    assert_eq!(state.word_styles.len(), 1);
}

#[test]
fn test_06_excel_set_cell_transaction_with_undo_redo() {
    let mut bus = GlobalCommandBus::new();
    let mut state = GlobalOfficeState::new();

    // 1. Set A1 = "100"
    let tx1 = Box::new(ExcelSetCellTransaction::new("A1", "100"));
    bus.execute(&mut state, tx1).expect("Set A1=100");
    assert_eq!(state.get_excel_cell("A1"), Some("100"));

    // 2. Set B1 = "200"
    let tx2 = Box::new(ExcelSetCellTransaction::new("B1", "200"));
    bus.execute(&mut state, tx2).expect("Set B1=200");
    assert_eq!(state.get_excel_cell("B1"), Some("200"));

    // 3. Set C1 = "=A1+B1"
    let tx3 = Box::new(ExcelSetCellTransaction::new("C1", "=A1+B1"));
    bus.execute(&mut state, tx3).expect("Set C1");
    assert_eq!(state.get_excel_cell("C1"), Some("=A1+B1"));

    // Undo C1
    bus.undo(&mut state).expect("Undo C1");
    assert_eq!(state.get_excel_cell("C1"), None);
    assert_eq!(state.get_excel_cell("B1"), Some("200"));

    // Undo B1
    bus.undo(&mut state).expect("Undo B1");
    assert_eq!(state.get_excel_cell("B1"), None);
    assert_eq!(state.get_excel_cell("A1"), Some("100"));

    // Redo B1 and C1
    bus.redo(&mut state).expect("Redo B1");
    assert_eq!(state.get_excel_cell("B1"), Some("200"));
    bus.redo(&mut state).expect("Redo C1");
    assert_eq!(state.get_excel_cell("C1"), Some("=A1+B1"));
}

#[test]
fn test_07_excel_cell_coalescing_and_paste_range() {
    let mut bus = GlobalCommandBus::new();
    let mut state = GlobalOfficeState::new();

    // Consecutive edits to same cell coalesce
    bus.execute(
        &mut state,
        Box::new(ExcelSetCellTransaction::new("A1", "1")),
    )
    .unwrap();
    bus.execute(
        &mut state,
        Box::new(ExcelSetCellTransaction::new("A1", "12")),
    )
    .unwrap();
    bus.execute(
        &mut state,
        Box::new(ExcelSetCellTransaction::new("A1", "123")),
    )
    .unwrap();

    assert_eq!(state.get_excel_cell("A1"), Some("123"));
    assert_eq!(bus.undo_stack_len(), 1);

    // Paste range
    let paste_cells = vec![
        ("A2".to_string(), "10".to_string()),
        ("B2".to_string(), "20".to_string()),
        ("C2".to_string(), "30".to_string()),
    ];
    bus.execute(
        &mut state,
        Box::new(ExcelPasteRangeTransaction::new(paste_cells)),
    )
    .unwrap();

    assert_eq!(state.get_excel_cell("A2"), Some("10"));
    assert_eq!(state.get_excel_cell("B2"), Some("20"));
    assert_eq!(state.get_excel_cell("C2"), Some("30"));

    // Undo paste range removes all 3 pasted cells
    bus.undo(&mut state).expect("Undo paste");
    assert_eq!(state.get_excel_cell("A2"), None);
    assert_eq!(state.get_excel_cell("B2"), None);
    assert_eq!(state.get_excel_cell("C2"), None);
    assert_eq!(state.get_excel_cell("A1"), Some("123"));
}

#[test]
fn test_08_excel_add_sheet_transaction() {
    let mut bus = GlobalCommandBus::new();
    let mut state = GlobalOfficeState::new();

    let tx = Box::new(ExcelAddSheetTransaction::new("Q4_Summary"));
    bus.execute(&mut state, tx).expect("Add Sheet");
    assert!(state.excel_sheets.contains(&"Q4_Summary".to_string()));

    bus.undo(&mut state).expect("Undo Add Sheet");
    assert!(!state.excel_sheets.contains(&"Q4_Summary".to_string()));

    bus.redo(&mut state).expect("Redo Add Sheet");
    assert!(state.excel_sheets.contains(&"Q4_Summary".to_string()));
}

#[test]
fn test_09_ppt_add_and_delete_shape_transaction() {
    let mut bus = GlobalCommandBus::new();
    let mut state = GlobalOfficeState::new();

    let shape = PptShapeData {
        id: 101,
        shape_type: "roundRect".into(),
        rect: Rect2D::new(100.0, 100.0, 200.0, 100.0),
        text: "Title Box".into(),
    };

    // Add shape
    bus.execute(
        &mut state,
        Box::new(PptAddShapeTransaction::new(shape.clone())),
    )
    .expect("Add shape");
    assert!(state.ppt_shapes.contains_key(&101));

    // Delete shape
    bus.execute(
        &mut state,
        Box::new(PptDeleteShapeTransaction::new(shape.clone())),
    )
    .expect("Delete shape");
    assert!(!state.ppt_shapes.contains_key(&101));

    // Undo delete -> restores shape
    bus.undo(&mut state).expect("Undo delete");
    assert!(state.ppt_shapes.contains_key(&101));

    // Undo add -> removes shape
    bus.undo(&mut state).expect("Undo add");
    assert!(!state.ppt_shapes.contains_key(&101));
}

#[test]
fn test_10_ppt_move_shape_drag_coalescing() {
    let mut bus = GlobalCommandBus::new();
    let mut state = GlobalOfficeState::new();

    let shape = PptShapeData {
        id: 1,
        shape_type: "rect".into(),
        rect: Rect2D::new(0.0, 0.0, 100.0, 100.0),
        text: String::new(),
    };
    state.ppt_shapes.insert(1, shape);

    // Continuous dragging simulation: (0,0) -> (10,0) -> (20,0) -> (50,0)
    bus.execute(
        &mut state,
        Box::new(PptMoveShapeTransaction::new(
            1,
            Rect2D::new(0.0, 0.0, 100.0, 100.0),
            Rect2D::new(10.0, 0.0, 100.0, 100.0),
        )),
    )
    .unwrap();

    bus.execute(
        &mut state,
        Box::new(PptMoveShapeTransaction::new(
            1,
            Rect2D::new(10.0, 0.0, 100.0, 100.0),
            Rect2D::new(20.0, 0.0, 100.0, 100.0),
        )),
    )
    .unwrap();

    bus.execute(
        &mut state,
        Box::new(PptMoveShapeTransaction::new(
            1,
            Rect2D::new(20.0, 0.0, 100.0, 100.0),
            Rect2D::new(50.0, 0.0, 100.0, 100.0),
        )),
    )
    .unwrap();

    assert_eq!(state.ppt_shapes.get(&1).unwrap().rect.x, 50.0);
    // Drag gestures coalesce into 1 transaction
    assert_eq!(bus.undo_stack_len(), 1);

    // Single undo reverts back to (0,0)
    bus.undo(&mut state).expect("Undo drag");
    assert_eq!(state.ppt_shapes.get(&1).unwrap().rect.x, 0.0);
}

#[test]
fn test_11_pdf_edit_text_and_form_field_transactions() {
    let mut bus = GlobalCommandBus::new();
    let mut state = GlobalOfficeState::new();

    // 1. PDF edit text run on page 0
    bus.execute(
        &mut state,
        Box::new(PdfEditTextTransaction::new(
            0,
            0,
            "Original Text",
            "Edited Text",
        )),
    )
    .expect("Edit PDF text");
    assert_eq!(
        state.pdf_text_runs.get(&(0, 0)).unwrap().as_str(),
        "Edited Text"
    );

    // 2. PDF form field filling with live typing coalescing
    bus.execute(
        &mut state,
        Box::new(PdfSetFormFieldTransaction::new("FullName", "", "John")),
    )
    .unwrap();
    bus.execute(
        &mut state,
        Box::new(PdfSetFormFieldTransaction::new(
            "FullName", "John", "John Doe",
        )),
    )
    .unwrap();

    assert_eq!(state.get_pdf_field("FullName"), Some("John Doe"));
    assert_eq!(bus.undo_stack_len(), 2);

    // Undo form field typing
    bus.undo(&mut state).expect("Undo form typing");
    assert_eq!(state.get_pdf_field("FullName"), Some(""));

    // 3. PDF move object with drag coalescing
    bus.execute(
        &mut state,
        Box::new(PdfMoveObjectTransaction::new(
            10,
            Rect2D::new(0.0, 0.0, 50.0, 50.0),
            Rect2D::new(10.0, 10.0, 50.0, 50.0),
        )),
    )
    .unwrap();
    bus.execute(
        &mut state,
        Box::new(PdfMoveObjectTransaction::new(
            10,
            Rect2D::new(10.0, 10.0, 50.0, 50.0),
            Rect2D::new(25.0, 25.0, 50.0, 50.0),
        )),
    )
    .unwrap();
    assert_eq!(state.pdf_objects.get(&10).unwrap().x, 25.0);
    // Move object coalesced
    bus.undo(&mut state).expect("Undo object move");
    assert_eq!(state.pdf_objects.get(&10).unwrap().x, 0.0);

    // Undo text edit
    bus.undo(&mut state).expect("Undo text edit");
    assert_eq!(
        state.pdf_text_runs.get(&(0, 0)).unwrap().as_str(),
        "Original Text"
    );
}

#[test]
fn test_12_pdf_rotate_page_transaction_modulo_360() {
    let mut bus = GlobalCommandBus::new();
    let mut state = GlobalOfficeState::new();

    // Rotate page 0 by +90°
    bus.execute(&mut state, Box::new(PdfRotatePageTransaction::new(0, 90)))
        .unwrap();
    assert_eq!(state.get_pdf_rotation(0), 90);

    // Rotate page 0 by +90° again -> 180°
    bus.execute(&mut state, Box::new(PdfRotatePageTransaction::new(0, 90)))
        .unwrap();
    assert_eq!(state.get_pdf_rotation(0), 180);

    // Rotate page 0 by +180° -> 360 = 0°
    bus.execute(&mut state, Box::new(PdfRotatePageTransaction::new(0, 180)))
        .unwrap();
    assert_eq!(state.get_pdf_rotation(0), 0);

    // Undo 180 -> back to 180°
    bus.undo(&mut state).unwrap();
    assert_eq!(state.get_pdf_rotation(0), 180);

    // Undo 90 -> back to 90°
    bus.undo(&mut state).unwrap();
    assert_eq!(state.get_pdf_rotation(0), 90);

    // Undo 90 -> back to 0°
    bus.undo(&mut state).unwrap();
    assert_eq!(state.get_pdf_rotation(0), 0);
}

#[test]
fn test_13_compound_atomic_multi_app_transaction() {
    let mut bus = GlobalCommandBus::new();
    let mut state = GlobalOfficeState::new();

    // Compound transaction executing Word + Excel + PPT simultaneously
    let compound = CompoundOfficeTransaction::new(
        "Multi-App Report Generation",
        DocumentType::Word,
        vec![
            Box::new(WordInsertTextTransaction::new(0, "Executive Summary")),
            Box::new(ExcelSetCellTransaction::new("A1", "Total Revenue")),
            Box::new(PptAddShapeTransaction::new(PptShapeData {
                id: 1,
                shape_type: "chart".into(),
                rect: Rect2D::new(0.0, 0.0, 400.0, 300.0),
                text: "Q1 Results".into(),
            })),
        ],
    );

    bus.execute(&mut state, Box::new(compound))
        .expect("Execute compound transaction");

    assert_eq!(state.word_content, "Executive Summary");
    assert_eq!(state.get_excel_cell("A1"), Some("Total Revenue"));
    assert!(state.ppt_shapes.contains_key(&1));

    // Single undo reverts all 3 actions in reverse order atomically
    bus.undo(&mut state).expect("Undo compound transaction");

    assert_eq!(state.word_content, "");
    assert_eq!(state.get_excel_cell("A1"), None);
    assert!(!state.ppt_shapes.contains_key(&1));

    // Redo restores all 3 actions
    bus.redo(&mut state).expect("Redo compound transaction");
    assert_eq!(state.word_content, "Executive Summary");
    assert_eq!(state.get_excel_cell("A1"), Some("Total Revenue"));
    assert!(state.ppt_shapes.contains_key(&1));
}

#[test]
fn test_14_save_point_clean_dirty_state_tracking() {
    let mut bus = GlobalCommandBus::new();
    let mut state = GlobalOfficeState::new();

    // Initially clean
    assert!(!bus.is_dirty());

    // Execute edit -> dirty
    bus.execute(&mut state, Box::new(WordInsertTextTransaction::new(0, "A")))
        .unwrap();
    assert!(bus.is_dirty());

    // Undo -> clean again! (Revision matches clean_counter)
    bus.undo(&mut state).unwrap();
    assert!(!bus.is_dirty());

    // Redo -> dirty
    bus.redo(&mut state).unwrap();
    assert!(bus.is_dirty());

    // User hits Ctrl+S / Save -> mark clean
    bus.mark_clean();
    assert!(!bus.is_dirty());

    // Another edit -> dirty
    bus.execute(&mut state, Box::new(WordInsertTextTransaction::new(1, "B")))
        .unwrap();
    assert!(bus.is_dirty());

    // Undo -> back to the save point -> clean!
    bus.undo(&mut state).unwrap();
    assert!(!bus.is_dirty());
}

#[test]
fn test_15_history_inspection_and_depth_bounding() {
    let mut bus = GlobalCommandBus::with_max_depth(3);
    let mut state = GlobalOfficeState::new();

    // Execute 5 non-coalescing transactions
    for i in 1..=5 {
        bus.execute(
            &mut state,
            Box::new(ExcelSetCellTransaction::new(format!("A{i}"), "val")),
        )
        .unwrap();
    }

    // Depth bounded to 3
    assert_eq!(bus.undo_stack_len(), 3);
    let history = bus.undo_history();
    assert_eq!(history.len(), 3);
    for desc in history {
        assert_eq!(desc, "Excel: Set Cell Value");
    }

    // Can undo up to 3 times
    assert!(bus.undo(&mut state).is_ok());
    assert!(bus.undo(&mut state).is_ok());
    assert!(bus.undo(&mut state).is_ok());
    assert!(bus.undo(&mut state).is_err()); // Stack exhausted
}

#[test]
fn test_16_clipboard_multi_mime_payload_generation() {
    // 1. Word payload
    let word_payload = GlobalClipboardPayload::from_word(
        "Quarterly Financials",
        true,
        true,
        Some("Arial"),
        Some(16.0),
    );
    assert_eq!(
        word_payload.plain_text.as_deref(),
        Some("Quarterly Financials")
    );
    assert!(word_payload
        .html
        .as_ref()
        .unwrap()
        .contains("font-weight:bold;"));
    assert!(word_payload
        .html
        .as_ref()
        .unwrap()
        .contains("font-style:italic;"));
    assert!(word_payload
        .html
        .as_ref()
        .unwrap()
        .contains("font-family:'Arial';"));
    assert!(word_payload
        .html
        .as_ref()
        .unwrap()
        .contains("font-size:16pt;"));
    assert!(word_payload.ooxml.as_ref().unwrap().contains("<w:b/>"));
    assert!(word_payload.ooxml.as_ref().unwrap().contains("<w:i/>"));
    assert!(word_payload
        .ooxml
        .as_ref()
        .unwrap()
        .contains(r#"w:ascii="Arial""#));
    assert!(word_payload
        .ooxml
        .as_ref()
        .unwrap()
        .contains(r#"w:sz w:val="32""#));

    // 2. Excel payload
    let grid = vec![
        vec!["Product".into(), "Price".into()],
        vec!["Laptop".into(), "$1200".into()],
    ];
    let excel_payload = GlobalClipboardPayload::from_excel_grid(&grid);
    assert_eq!(
        excel_payload.plain_text.as_deref(),
        Some("Product\tPrice\nLaptop\t$1200")
    );
    assert!(excel_payload.html.as_ref().unwrap().contains("<table>"));
    assert!(excel_payload
        .html
        .as_ref()
        .unwrap()
        .contains("<td>Laptop</td>"));
    assert!(excel_payload.ooxml.as_ref().unwrap().contains("<w:tbl"));

    // 3. PDF payload
    let pdf_payload = GlobalClipboardPayload::from_pdf_text("Contract Page 1");
    assert_eq!(pdf_payload.plain_text.as_deref(), Some("Contract Page 1"));
    assert!(pdf_payload.html.as_ref().unwrap().contains("pdf-extracted"));

    // 4. PNG payload
    let dummy_png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let png_payload = GlobalClipboardPayload::from_image_png(dummy_png.clone());
    assert_eq!(png_payload.png_bytes, Some(dummy_png));
}

#[test]
fn test_17_cross_app_clipboard_conversions_lossless() {
    let original_grid = vec![
        vec!["A".to_string(), "B".to_string(), "C".to_string()],
        vec!["1".to_string(), "2".to_string(), "3".to_string()],
    ];

    // Grid -> TSV
    let tsv = grid_to_tsv(&original_grid);
    assert_eq!(tsv, "A\tB\tC\n1\t2\t3");

    // TSV -> HTML Table -> TSV
    let html = tsv_to_html_table(&tsv);
    assert!(html.starts_with("<table>"));
    let tsv_from_html = html_table_to_tsv(&html);
    assert_eq!(tsv_from_html, tsv);

    // TSV -> Word Table XML -> TSV
    let word_xml = tsv_to_word_table_xml(&tsv);
    assert!(word_xml.contains("<w:tbl"));
    assert!(word_xml.contains("<w:tr>"));
    assert!(word_xml.contains("<w:tc>"));
    let tsv_from_word = word_table_xml_to_tsv(&word_xml);
    assert_eq!(tsv_from_word, tsv);

    // TSV back to 2D grid
    let parsed_grid = tsv_to_grid(&tsv_from_word);
    assert_eq!(parsed_grid, original_grid);
}

#[test]
fn test_18_global_clipboard_ring_buffer_history() {
    let mut clipboard = GlobalClipboard::with_capacity(3);

    assert_eq!(clipboard.current(), None);
    assert!(clipboard.history().is_empty());

    // Push 4 items into capacity 3
    clipboard.copy(GlobalClipboardPayload::from_plain_text("Item 1"));
    clipboard.copy(GlobalClipboardPayload::from_plain_text("Item 2"));
    clipboard.copy(GlobalClipboardPayload::from_plain_text("Item 3"));
    clipboard.copy(GlobalClipboardPayload::from_plain_text("Item 4"));

    assert_eq!(clipboard.paste_plain_text().as_deref(), Some("Item 4"));
    // History contains [Item 2, Item 3, Item 4]
    assert_eq!(clipboard.history().len(), 3);
    assert_eq!(clipboard.history()[0].plain_text.as_deref(), Some("Item 2"));
    assert_eq!(clipboard.history()[2].plain_text.as_deref(), Some("Item 4"));

    clipboard.clear();
    assert_eq!(clipboard.current(), None);
    assert!(clipboard.history().is_empty());
}

#[test]
fn test_19_desktop_session_manager_pdf_lifecycle_and_ribbon() {
    let mut mgr = DesktopSessionManager::new();

    // 1. Detect file extensions
    assert_eq!(
        DesktopSessionManager::detect_doc_type("manual.pdf"),
        DocumentType::Pdf
    );
    assert_eq!(
        DesktopSessionManager::detect_doc_type("SHEET.XLSX"),
        DocumentType::Excel
    );
    assert_eq!(
        DesktopSessionManager::detect_doc_type("slides.pptx"),
        DocumentType::PowerPoint
    );
    assert_eq!(
        DesktopSessionManager::detect_doc_type("letter.docx"),
        DocumentType::Word
    );

    // 2. Create PDF session
    let session = mgr.create_session(DocumentType::Pdf, "UserGuide.pdf");
    assert_eq!(session.doc_type, DocumentType::Pdf);
    assert!(!session.is_dirty);

    // 3. Ribbon tabs for PDF
    let ribbon = RibbonModel::for_document_type(DocumentType::Pdf);
    let tabs = ribbon.visible_tabs();
    assert_eq!(
        tabs,
        vec![
            RibbonTabId::File,
            RibbonTabId::Home,
            RibbonTabId::Insert,
            RibbonTabId::Review,
            RibbonTabId::View,
        ]
    );

    // 4. Execute transaction in session
    let tx = Box::new(PdfRotatePageTransaction::new(0, 90));
    mgr.execute_office_transaction(&session.session_id, tx)
        .expect("Rotate page");
    assert!(mgr.is_session_dirty(&session.session_id).unwrap());

    // 5. Mark clean
    mgr.mark_session_clean(&session.session_id).unwrap();
    assert!(!mgr.is_session_dirty(&session.session_id).unwrap());

    // 6. Undo in session
    mgr.undo_session(&session.session_id).unwrap();
    assert!(mgr.is_session_dirty(&session.session_id).unwrap());

    // 7. Redo in session
    mgr.redo_session(&session.session_id).unwrap();
    assert!(!mgr.is_session_dirty(&session.session_id).unwrap());
}

#[test]
fn test_20_command_dispatcher_to_bus_integration() {
    let mut bus = GlobalCommandBus::new();
    let mut state = GlobalOfficeState::new();

    // Dispatch Word insert
    CommandDispatcher::dispatch_to_bus(
        &mut bus,
        &mut state,
        OfficeCommand::Word(WordAction::InsertText {
            text: "Hello".into(),
        }),
    )
    .unwrap();
    assert_eq!(state.word_content, "Hello");

    // Dispatch Excel formula
    CommandDispatcher::dispatch_to_bus(
        &mut bus,
        &mut state,
        OfficeCommand::Excel(ExcelAction::SetCellFormula {
            coord: "B5".into(),
            formula: "=AVERAGE(A1:A10)".into(),
        }),
    )
    .unwrap();
    assert_eq!(state.get_excel_cell("B5"), Some("=AVERAGE(A1:A10)"));

    // Dispatch PPT shape
    CommandDispatcher::dispatch_to_bus(
        &mut bus,
        &mut state,
        OfficeCommand::PowerPoint(PptAction::InsertShape {
            shape_type: "star5".into(),
            x: 50.0,
            y: 50.0,
            w: 100.0,
            h: 100.0,
        }),
    )
    .unwrap();
    assert_eq!(state.ppt_shapes.len(), 1);

    // Dispatch PDF rotation
    CommandDispatcher::dispatch_to_bus(
        &mut bus,
        &mut state,
        OfficeCommand::Pdf(PdfAction::RotatePage {
            page: 1,
            angle_deg: 90,
        }),
    )
    .unwrap();
    assert_eq!(state.get_pdf_rotation(1), 90);

    // Undo all 4 dispatched actions
    assert!(bus.undo(&mut state).is_ok()); // Undo PDF
    assert_eq!(state.get_pdf_rotation(1), 0);
    assert!(bus.undo(&mut state).is_ok()); // Undo PPT
    assert!(state.ppt_shapes.is_empty());
    assert!(bus.undo(&mut state).is_ok()); // Undo Excel
    assert_eq!(state.get_excel_cell("B5"), None);
    assert!(bus.undo(&mut state).is_ok()); // Undo Word
    assert_eq!(state.word_content, "");
}

#[test]
fn test_21_sub_millisecond_latency_sla_benchmark() {
    let mut bus = GlobalCommandBus::new();
    let mut state = GlobalOfficeState::new();

    let iterations = 10_000;
    let start = Instant::now();

    for i in 0..iterations {
        let tx = Box::new(ExcelSetCellTransaction::new(
            format!("Cell_{i}"),
            format!("Val_{i}"),
        ));
        bus.execute(&mut state, tx).unwrap();
    }

    let execute_elapsed = start.elapsed();
    let avg_exec_micros = execute_elapsed.as_micros() as f64 / iterations as f64;
    println!(
        "Benchmark: 10,000 transaction executes took {:?} (average: {:.2} µs / op)",
        execute_elapsed, avg_exec_micros
    );
    // SLA: Execution must be strictly < 1.0 ms (1,000 µs)
    assert!(
        avg_exec_micros < 1000.0,
        "Transaction execute average latency {:.2} µs exceeded 1.0 ms SLA",
        avg_exec_micros
    );

    // Benchmark Undo latency
    let undo_start = Instant::now();
    for _ in 0..1000 {
        bus.undo(&mut state).unwrap();
    }
    let undo_elapsed = undo_start.elapsed();
    let avg_undo_micros = undo_elapsed.as_micros() as f64 / 1000.0;
    println!(
        "Benchmark: 1,000 transaction undos took {:?} (average: {:.2} µs / op)",
        undo_elapsed, avg_undo_micros
    );
    assert!(
        avg_undo_micros < 1000.0,
        "Transaction undo average latency {:.2} µs exceeded 1.0 ms SLA",
        avg_undo_micros
    );
}
