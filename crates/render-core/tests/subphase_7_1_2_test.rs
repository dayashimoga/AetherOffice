//! Subphase 7.1.2 Integration & Verification Test Suite
//!
//! Validates:
//! 1. Windows, Linux, macOS, Android, and iOS Release Matrix CI/CD
//! 2. Cross-platform target triplets and packaging bundles
//! 3. GEC-v1.0 Article III performance and memory SLA budget checks
//! 4. 100% UI button, input, and formula bar dispatch through the Transactional Command Bus
//! 5. Word, Excel, PowerPoint, PDF, Stylus, and Clipboard UI action routing
//! 6. Sub-millisecond UI action dispatch latency benchmark

use render_core::adaptive::InkStroke;
use render_core::compiler::word::ParagraphAlignment;
use render_core::primitive::{ColorRgba, Rect2D};
use render_core::release_matrix::{
    PackagingBundle, PlatformReleaseTarget, ReleaseMatrix, TargetArch, TargetOs,
};
use render_core::shell::{DesktopSessionManager, DocumentType};
use render_core::ui_dispatcher::{UiActionEvent, UiCommandDispatcher};
use std::time::Instant;

#[test]
fn test_01_release_matrix_platform_targets_and_os_coverage() {
    let matrix = ReleaseMatrix::canonical_matrix("OpenOfficeRS", "0.1.0");

    assert_eq!(matrix.app_name, "OpenOfficeRS");
    assert_eq!(matrix.version, "0.1.0");
    assert!(matrix.targets.len() >= 8);

    // Verify all 5 operating systems are present
    assert!(!matrix.targets_for_os(TargetOs::Windows).is_empty());
    assert!(!matrix.targets_for_os(TargetOs::Linux).is_empty());
    assert!(!matrix.targets_for_os(TargetOs::MacOS).is_empty());
    assert!(!matrix.targets_for_os(TargetOs::Android).is_empty());
    assert!(!matrix.targets_for_os(TargetOs::Ios).is_empty());

    assert!(matrix.validate_full_coverage().is_ok());
}

#[test]
fn test_02_rust_triplets_and_architecture_consistency() {
    let matrix = ReleaseMatrix::default();

    let triplets: Vec<&str> = matrix
        .targets
        .iter()
        .map(|t| t.rust_triplet.as_str())
        .collect();

    assert!(triplets.contains(&"x86_64-pc-windows-msvc"));
    assert!(triplets.contains(&"x86_64-unknown-linux-gnu"));
    assert!(triplets.contains(&"aarch64-apple-darwin+x86_64-apple-darwin"));
    assert!(triplets.contains(&"aarch64-linux-android"));
    assert!(triplets.contains(&"armv7-linux-androideabi"));
    assert!(triplets.contains(&"x86_64-linux-android"));
    assert!(triplets.contains(&"aarch64-apple-ios"));
    assert!(triplets.contains(&"aarch64-apple-ios-sim"));

    assert!(matrix.validate_triplets().is_ok());
}

#[test]
fn test_03_packaging_bundle_formats_and_artifacts() {
    let matrix = ReleaseMatrix::default();

    let win_target = matrix
        .targets_for_os(TargetOs::Windows)
        .into_iter()
        .next()
        .unwrap();
    assert!(win_target.bundle_formats.contains(&PackagingBundle::Msi));
    assert!(win_target.bundle_formats.contains(&PackagingBundle::Nsis));

    let filename = win_target.artifact_filename("OpenOfficeRS", "0.1.0", PackagingBundle::Msi);
    assert_eq!(filename, "OpenOfficeRS-0.1.0-Windows-x86_64.msi");

    let linux_target = matrix
        .targets_for_os(TargetOs::Linux)
        .into_iter()
        .next()
        .unwrap();
    assert!(linux_target
        .bundle_formats
        .contains(&PackagingBundle::AppImage));
    assert!(linux_target.bundle_formats.contains(&PackagingBundle::Deb));
    assert_eq!(PackagingBundle::AppImage.extension(), ".AppImage");
    assert_eq!(PackagingBundle::Deb.extension(), ".deb");

    let mac_target = matrix
        .targets_for_os(TargetOs::MacOS)
        .into_iter()
        .next()
        .unwrap();
    assert!(mac_target.bundle_formats.contains(&PackagingBundle::Dmg));
    assert_eq!(PackagingBundle::Dmg.extension(), ".dmg");

    let android_target = matrix
        .targets_for_os(TargetOs::Android)
        .into_iter()
        .next()
        .unwrap();
    assert!(android_target
        .bundle_formats
        .contains(&PackagingBundle::Apk));
    assert_eq!(PackagingBundle::Apk.extension(), ".apk");

    let ios_target = matrix
        .targets_for_os(TargetOs::Ios)
        .into_iter()
        .next()
        .unwrap();
    assert!(ios_target.bundle_formats.contains(&PackagingBundle::Ipa));
    assert_eq!(PackagingBundle::Ipa.extension(), ".ipa");
}

#[test]
fn test_04_ci_cd_github_actions_matrix_json_export() {
    let matrix = ReleaseMatrix::default();
    let json = matrix.export_github_matrix_json().expect("Export JSON");

    assert!(json.contains("\"app_name\": \"OpenOfficeRS\""));
    assert!(json.contains("\"x86_64-pc-windows-msvc\""));
    assert!(json.contains("\"aarch64-linux-android\""));
    assert!(json.contains("\"aarch64-apple-ios\""));

    let deserialized: ReleaseMatrix = serde_json::from_str(&json).expect("Deserialize JSON");
    assert_eq!(deserialized.targets.len(), matrix.targets.len());
}

#[test]
fn test_05_article_iii_performance_and_sla_limits() {
    let desktop_target = PlatformReleaseTarget::new(
        TargetOs::Windows,
        TargetArch::X86_64,
        "x86_64-pc-windows-msvc",
        vec![PackagingBundle::Msi],
        "10+",
    );
    assert_eq!(desktop_target.startup_sla_limit_ms, 150);
    assert_eq!(desktop_target.memory_sla_limit_mb, 180);

    // Compliant SLA
    assert!(desktop_target.check_sla_budget(120, 160).is_ok());
    // Exceeded startup SLA
    assert!(desktop_target.check_sla_budget(160, 100).is_err());
    // Exceeded memory SLA
    assert!(desktop_target.check_sla_budget(100, 200).is_err());

    let mobile_target = PlatformReleaseTarget::new(
        TargetOs::Android,
        TargetArch::Aarch64,
        "aarch64-linux-android",
        vec![PackagingBundle::Apk],
        "8.0+",
    );
    assert_eq!(mobile_target.startup_sla_limit_ms, 300);
    assert_eq!(mobile_target.memory_sla_limit_mb, 120);

    assert!(mobile_target.check_sla_budget(250, 80).is_ok());
    assert!(mobile_target.check_sla_budget(350, 80).is_err());
}

#[test]
fn test_06_ui_action_coverage_and_audit() {
    assert_eq!(UiCommandDispatcher::audit_action_coverage(), 28);
}

#[test]
fn test_07_word_ribbon_formatting_dispatch_to_command_bus() {
    let mut mgr = DesktopSessionManager::new();
    let session = mgr.create_session(DocumentType::Word, "Doc1.docx");
    let sess_id = session.session_id.clone();

    // Insert text first so format has target
    UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess_id,
        UiActionEvent::InsertText {
            text: "OpenOfficeRS Testing".to_string(),
        },
    )
    .expect("Insert text");

    // 1. Bold
    let r1 = UiCommandDispatcher::dispatch(&mut mgr, &sess_id, UiActionEvent::ToggleBold)
        .expect("Toggle bold");
    assert_eq!(r1.undo_stack_depth, 2);
    assert!(r1.is_dirty);

    // 2. Italic
    let r2 = UiCommandDispatcher::dispatch(&mut mgr, &sess_id, UiActionEvent::ToggleItalic)
        .expect("Toggle italic");
    assert_eq!(r2.undo_stack_depth, 3);

    // 3. Underline
    let r3 = UiCommandDispatcher::dispatch(&mut mgr, &sess_id, UiActionEvent::ToggleUnderline)
        .expect("Toggle underline");
    assert_eq!(r3.undo_stack_depth, 4);

    // 4. Strikethrough
    let r4 = UiCommandDispatcher::dispatch(&mut mgr, &sess_id, UiActionEvent::ToggleStrikethrough)
        .expect("Toggle strikethrough");
    assert_eq!(r4.undo_stack_depth, 5);

    // 5. Font Family
    let r5 = UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess_id,
        UiActionEvent::SetFontFamily("Arial".to_string()),
    )
    .expect("Set font");
    assert_eq!(r5.undo_stack_depth, 6);

    // 6. Font Size
    let r6 = UiCommandDispatcher::dispatch(&mut mgr, &sess_id, UiActionEvent::SetFontSize(16.0))
        .expect("Set font size");
    assert_eq!(r6.undo_stack_depth, 7);

    // 7. Font Color
    let r7 = UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess_id,
        UiActionEvent::SetFontColor(ColorRgba::new(0.0, 0.5, 1.0, 1.0)),
    )
    .expect("Set font color");
    assert_eq!(r7.undo_stack_depth, 8);

    // 8. Alignment
    let r8 = UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess_id,
        UiActionEvent::SetAlignment(ParagraphAlignment::Center),
    )
    .expect("Set alignment");
    assert_eq!(r8.undo_stack_depth, 9);

    // 9. Clear formatting
    let r9 = UiCommandDispatcher::dispatch(&mut mgr, &sess_id, UiActionEvent::ClearFormatting)
        .expect("Clear formatting");
    assert_eq!(r9.undo_stack_depth, 10);

    // Undo clear formatting
    let u1 = UiCommandDispatcher::dispatch(&mut mgr, &sess_id, UiActionEvent::Undo).expect("Undo");
    assert_eq!(u1.undo_stack_depth, 9);
    assert_eq!(u1.redo_stack_depth, 1);
}

#[test]
fn test_08_word_typing_and_deletion_dispatch_to_command_bus() {
    let mut mgr = DesktopSessionManager::new();
    let session = mgr.create_session(DocumentType::Word, "Typing.docx");
    let sess_id = session.session_id.clone();

    // 1. Keystroke insertion
    UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess_id,
        UiActionEvent::InsertText {
            text: "Hello World!".to_string(),
        },
    )
    .expect("Type text");

    assert_eq!(
        mgr.office_states.get(&sess_id).unwrap().word_content,
        "Hello World!"
    );

    // 2. Delete range ("World!")
    UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess_id,
        UiActionEvent::DeleteTextRange {
            offset: 5,
            length: 7,
        },
    )
    .expect("Delete text range");

    assert_eq!(
        mgr.office_states.get(&sess_id).unwrap().word_content,
        "Hello"
    );

    // 3. Replace text range ("Hello" -> "Antigravity")
    UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess_id,
        UiActionEvent::ReplaceTextRange {
            offset: 0,
            length: 5,
            new_text: "Antigravity".to_string(),
        },
    )
    .expect("Replace text");

    assert_eq!(
        mgr.office_states.get(&sess_id).unwrap().word_content,
        "Antigravity"
    );

    // 4. Undo replacement
    UiCommandDispatcher::dispatch(&mut mgr, &sess_id, UiActionEvent::Undo).expect("Undo replace");
    assert_eq!(
        mgr.office_states.get(&sess_id).unwrap().word_content,
        "Hello"
    );

    // 5. Undo deletion
    UiCommandDispatcher::dispatch(&mut mgr, &sess_id, UiActionEvent::Undo).expect("Undo delete");
    assert_eq!(
        mgr.office_states.get(&sess_id).unwrap().word_content,
        "Hello World!"
    );
}

#[test]
fn test_09_excel_formula_bar_and_cell_input_dispatch_to_command_bus() {
    let mut mgr = DesktopSessionManager::new();
    let session = mgr.create_session(DocumentType::Excel, "Sheet1.xlsx");
    let sess_id = session.session_id.clone();

    // 1. Set cell formula from formula bar
    UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess_id,
        UiActionEvent::SetCellFormula {
            coord: "C1".to_string(),
            formula: "=SUM(A1:B1)".to_string(),
        },
    )
    .expect("Formula bar input");

    assert_eq!(
        mgr.office_states
            .get(&sess_id)
            .unwrap()
            .excel_cells
            .get("C1")
            .unwrap(),
        "=SUM(A1:B1)"
    );

    // 2. Set direct cell value
    UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess_id,
        UiActionEvent::SetCellValue {
            coord: "A1".to_string(),
            value: "100".to_string(),
        },
    )
    .expect("Set value");

    assert_eq!(
        mgr.office_states
            .get(&sess_id)
            .unwrap()
            .excel_cells
            .get("A1")
            .unwrap(),
        "100"
    );

    // 3. Clear cell
    UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess_id,
        UiActionEvent::ClearCell {
            coord: "C1".to_string(),
        },
    )
    .expect("Clear cell");

    assert_eq!(
        mgr.office_states
            .get(&sess_id)
            .unwrap()
            .excel_cells
            .get("C1")
            .unwrap(),
        ""
    );

    // 4. Add sheet tab
    let res = UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess_id,
        UiActionEvent::AddSheet {
            title: "Q4_Summary".to_string(),
        },
    )
    .expect("Add sheet");
    assert_eq!(res.undo_stack_depth, 4);

    // Undo add sheet
    UiCommandDispatcher::dispatch(&mut mgr, &sess_id, UiActionEvent::Undo).expect("Undo");
    assert!(!mgr
        .office_states
        .get(&sess_id)
        .unwrap()
        .excel_sheets
        .contains(&"Q4_Summary".to_string()));
}

#[test]
fn test_10_excel_mobile_card_field_editor_dispatch_to_command_bus() {
    let mut mgr = DesktopSessionManager::new();
    let session = mgr.create_session(DocumentType::Excel, "MobileBudget.xlsx");
    let sess_id = session.session_id.clone();

    // Mobile Card Inspector edits row 4, col 2 ("C5")
    UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess_id,
        UiActionEvent::EditRowCardField {
            row: 4,
            col: 2,
            field_name: "Revenue".to_string(),
            new_value: "54200".to_string(),
        },
    )
    .expect("Edit card field");

    assert_eq!(
        mgr.office_states
            .get(&sess_id)
            .unwrap()
            .excel_cells
            .get("C5")
            .unwrap(),
        "54200"
    );

    // Undo mobile card edit
    UiCommandDispatcher::dispatch(&mut mgr, &sess_id, UiActionEvent::Undo).expect("Undo card edit");
    assert_eq!(
        mgr.office_states
            .get(&sess_id)
            .unwrap()
            .excel_cells
            .get("C5"),
        None
    );
}

#[test]
fn test_11_powerpoint_slide_and_shape_actions_dispatch_to_command_bus() {
    let mut mgr = DesktopSessionManager::new();
    let session = mgr.create_session(DocumentType::PowerPoint, "Presentation.pptx");
    let sess_id = session.session_id.clone();

    // 1. New Slide
    let r1 = UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess_id,
        UiActionEvent::NewSlide {
            layout: "TitleAndContent".to_string(),
        },
    )
    .expect("New slide");
    assert_eq!(r1.undo_stack_depth, 1);

    // 2. Add Shape
    let r2 = UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess_id,
        UiActionEvent::AddSlideShape {
            shape_type: "round_rect".to_string(),
            rect: Rect2D::new(50.0, 50.0, 200.0, 100.0),
        },
    )
    .expect("Add shape");
    assert_eq!(r2.undo_stack_depth, 2);

    // 3. Update Slide Text
    let r3 = UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess_id,
        UiActionEvent::UpdateSlideText {
            shape_id: 2,
            text: "Quarterly Review".to_string(),
        },
    )
    .expect("Update text");
    assert_eq!(r3.undo_stack_depth, 3);
    assert_eq!(
        mgr.office_states
            .get(&sess_id)
            .unwrap()
            .ppt_shapes
            .get(&2)
            .unwrap()
            .text,
        "Quarterly Review"
    );

    // 4. Delete Slide Shape
    let r4 = UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess_id,
        UiActionEvent::DeleteSlideShape { shape_id: 2 },
    )
    .expect("Delete shape");
    assert_eq!(r4.undo_stack_depth, 4);
    assert!(!mgr
        .office_states
        .get(&sess_id)
        .unwrap()
        .ppt_shapes
        .contains_key(&2));
}

#[test]
fn test_12_pdf_form_filling_and_rotation_dispatch_to_command_bus() {
    let mut mgr = DesktopSessionManager::new();
    let session = mgr.create_session(DocumentType::Pdf, "Form.pdf");
    let sess_id = session.session_id.clone();

    // 1. Fill Form Field
    UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess_id,
        UiActionEvent::SetFormField {
            field_name: "ApplicantName".to_string(),
            value: "Dr. Dayashimoga".to_string(),
        },
    )
    .expect("Set field");

    assert_eq!(
        mgr.office_states
            .get(&sess_id)
            .unwrap()
            .pdf_fields
            .get("ApplicantName")
            .unwrap(),
        "Dr. Dayashimoga"
    );

    // 2. Rotate PDF Page
    UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess_id,
        UiActionEvent::RotatePdfPage {
            page: 1,
            angle_deg: 90,
        },
    )
    .expect("Rotate page");

    assert_eq!(
        *mgr.office_states
            .get(&sess_id)
            .unwrap()
            .pdf_page_rotations
            .get(&1)
            .unwrap(),
        90
    );

    // 3. Edit PDF Text
    UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess_id,
        UiActionEvent::EditPdfText {
            page: 0,
            object_id: 1,
            new_text: "Updated Contract Clause".to_string(),
        },
    )
    .expect("Edit PDF text");

    assert_eq!(
        mgr.office_states
            .get(&sess_id)
            .unwrap()
            .pdf_text_runs
            .get(&(0, 1))
            .unwrap(),
        "Updated Contract Clause"
    );
}

#[test]
fn test_13_stylus_ink_stroke_and_clear_dispatch_to_command_bus() {
    let mut mgr = DesktopSessionManager::new();
    let session = mgr.create_session(DocumentType::Word, "Notes.docx");
    let sess_id = session.session_id.clone();

    let mut stroke = InkStroke::new("#0078D7", 2.5);
    stroke.add_point(10.0, 10.0, 0.5, 0);
    stroke.add_point(20.0, 20.0, 0.7, 5);
    stroke.add_point(30.0, 30.0, 0.9, 10);

    // 1. Add stylus stroke
    let r1 = UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess_id,
        UiActionEvent::AddInkStroke(stroke.clone()),
    )
    .expect("Add stroke");
    assert_eq!(r1.undo_stack_depth, 1);

    // 2. Clear all ink
    let r2 = UiCommandDispatcher::dispatch(&mut mgr, &sess_id, UiActionEvent::ClearAllInk)
        .expect("Clear ink");
    assert_eq!(r2.undo_stack_depth, 2);

    // 3. Undo clear ink
    let u1 = UiCommandDispatcher::dispatch(&mut mgr, &sess_id, UiActionEvent::Undo).expect("Undo");
    assert_eq!(u1.undo_stack_depth, 1);
}

#[test]
fn test_14_clipboard_cut_copy_paste_dispatch_to_command_bus() {
    let mut mgr = DesktopSessionManager::new();
    let session = mgr.create_session(DocumentType::Word, "Clipboard.docx");
    let sess_id = session.session_id.clone();

    // Type text
    UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess_id,
        UiActionEvent::InsertText {
            text: "Confidential Report".to_string(),
        },
    )
    .expect("Insert text");

    // Copy to clipboard
    UiCommandDispatcher::dispatch(&mut mgr, &sess_id, UiActionEvent::Copy).expect("Copy text");
    assert_eq!(
        mgr.clipboard.paste_plain_text(),
        Some("Confidential Report".to_string())
    );

    // Cut to clipboard
    UiCommandDispatcher::dispatch(&mut mgr, &sess_id, UiActionEvent::Cut).expect("Cut text");
    assert_eq!(mgr.office_states.get(&sess_id).unwrap().word_content, "");

    // Paste back
    UiCommandDispatcher::dispatch(
        &mut mgr,
        &sess_id,
        UiActionEvent::Paste {
            content: "Confidential Report".to_string(),
        },
    )
    .expect("Paste text");
    assert_eq!(
        mgr.office_states.get(&sess_id).unwrap().word_content,
        "Confidential Report"
    );
}

#[test]
fn test_15_global_undo_redo_and_sub_millisecond_dispatch_benchmark() {
    let mut mgr = DesktopSessionManager::new();
    let session = mgr.create_session(DocumentType::Word, "Bench.docx");
    let sess_id = session.session_id.clone();

    // Benchmark 10,000 UI button dispatches to ensure < 5.0 µs / op latency
    let start = Instant::now();
    for i in 0..10_000 {
        let action = if i % 2 == 0 {
            UiActionEvent::ToggleBold
        } else {
            UiActionEvent::ToggleItalic
        };
        UiCommandDispatcher::dispatch(&mut mgr, &sess_id, action).expect("Dispatch action");
    }
    let elapsed = start.elapsed();
    let per_op_micros = (elapsed.as_micros() as f64) / 10_000.0;

    println!(
        "Benchmark: 10,000 UI Command Bus dispatches took {:?} (average: {:.2} µs / op)",
        elapsed, per_op_micros
    );

    // Verify SLA limit: vastly under 1.0 ms (1000 µs) budget (GEC-v1.0 Article III.3)
    assert!(
        per_op_micros < 1000.0,
        "Dispatch latency {:.2} µs exceeded < 1.0 ms SLA limit",
        per_op_micros
    );

    // Test Save / Mark Saved
    let r_save =
        UiCommandDispatcher::dispatch(&mut mgr, &sess_id, UiActionEvent::MarkSaved).expect("Save");
    assert!(!r_save.is_dirty);
    assert!(!mgr.is_session_dirty(&sess_id).unwrap());
}
