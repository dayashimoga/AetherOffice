//! Subphase 5.3.2 Integration Test Suite:
//! In-Place PDF Text/Image Editor, Page Organiser & Form Filler UI
//!
//! Validates:
//! - In-place text editor typing, backspace, delete, caret navigation, and font styling
//! - 530ms caret blink cycle state machine and typing reset
//! - 8-handle transform gizmo hit testing (corners, edges, rotation stem)
//! - Object translation, aspect-locked resizing, and 360° rotation
//! - Document Page Organiser (page reordering, rotation ±90°, insertion, deletion, duplication)
//! - Multi-page thumbnail filmstrip / grid layout computation
//! - Interactive Form Filler state machine (text input, checkbox toggles, dropdown selection, required validation)
//! - Transactional Command Bus integration with Undo/Redo and keystroke coalescing
//! - Sub-millisecond keystroke / dispatch latency SLA (< 1.0 ms)

use pdf_engine::types::PdfRect;
use render_core::compiler::pdf::{PdfDocumentLayout, PdfViewportConfig};
use render_core::compiler::pdf_editor::{
    PdfEditTextCommand, PdfEditorDisplayRenderer, PdfFormFiller, PdfGizmoHit, PdfMoveObjectCommand,
    PdfObjectEditor, PdfObjectGizmo, PdfPageOrganiser, PdfRotatePageCommand,
    PdfSetFormFieldCommand, PdfTextEditor, PdfTextStyle,
};
use render_core::interaction::{CommandBus, DocumentState, ResizeHandle};
use render_core::primitive::{DisplayList, Rect2D};
use std::time::Instant;

fn sample_rect(x: f64, y: f64, w: f64, h: f64) -> PdfRect {
    PdfRect::new(x, y, x + w, y + h)
}

// =========================================================================
// Tests
// =========================================================================

#[test]
fn test_01_text_editor_start_typing_and_reflow() {
    let mut editor = PdfTextEditor::new();
    assert!(!editor.is_editing());

    let initial_bbox = sample_rect(72.0, 700.0, 36.0, 14.0);
    editor.start_editing(
        0,
        0,
        "Hello".into(),
        5,
        initial_bbox,
        PdfTextStyle::new("Helvetica", 12.0, [0.0, 0.0, 0.0, 1.0]),
    );

    assert!(editor.is_editing());
    let st = editor.active_state().unwrap();
    assert_eq!(st.text, "Hello");
    assert_eq!(st.caret_offset, 5);

    // Type " World"
    editor.insert_text(" World");
    let st = editor.active_state().unwrap();
    assert_eq!(st.text, "Hello World");
    assert_eq!(st.caret_offset, 11);

    // Bounding box urx must expand to accommodate longer text
    assert!(st.bbox.urx > initial_bbox.urx);

    let committed = editor.commit().unwrap();
    assert_eq!(committed.text, "Hello World");
    assert!(!editor.is_editing());
}

#[test]
fn test_02_text_editor_backspace_and_delete() {
    let mut editor = PdfTextEditor::new();
    editor.start_editing(
        0,
        0,
        "RustOffice".into(),
        4, // Caret between "Rust" and "Office"
        sample_rect(50.0, 500.0, 100.0, 12.0),
        PdfTextStyle::new("Times-Roman", 12.0, [0.0, 0.0, 0.0, 1.0]),
    );

    // Backspace deletes 't' before caret -> "RusOffice", caret at 3
    assert!(editor.backspace());
    assert_eq!(editor.active_state().unwrap().text, "RusOffice");
    assert_eq!(editor.active_state().unwrap().caret_offset, 3);

    // Delete removes 'O' after caret -> "Rusffice", caret stays at 3
    assert!(editor.delete());
    assert_eq!(editor.active_state().unwrap().text, "Rusffice");
    assert_eq!(editor.active_state().unwrap().caret_offset, 3);

    // Move to beginning and backspace should safely return false without panic
    editor.move_caret_home();
    assert_eq!(editor.active_state().unwrap().caret_offset, 0);
    assert!(!editor.backspace());
}

#[test]
fn test_03_text_editor_caret_navigation_and_styling() {
    let mut editor = PdfTextEditor::new();
    editor.start_editing(
        0,
        0,
        "Antigravity".into(),
        5,
        sample_rect(100.0, 600.0, 120.0, 14.0),
        PdfTextStyle::new("Helvetica", 12.0, [0.0, 0.0, 0.0, 1.0]),
    );

    // Caret navigation
    editor.move_caret_left();
    assert_eq!(editor.active_state().unwrap().caret_offset, 4);
    editor.move_caret_right();
    assert_eq!(editor.active_state().unwrap().caret_offset, 5);
    editor.move_caret_end();
    assert_eq!(editor.active_state().unwrap().caret_offset, 11);

    // Typography styling
    editor.set_font_size(18.0);
    assert_eq!(editor.active_state().unwrap().style.font_size, 18.0);

    editor.set_font_name("Courier-Bold");
    assert_eq!(
        editor.active_state().unwrap().style.font_name,
        "Courier-Bold"
    );

    editor.set_color([1.0, 0.0, 0.0, 1.0]);
    assert_eq!(
        editor.active_state().unwrap().style.color,
        [1.0, 0.0, 0.0, 1.0]
    );
}

#[test]
fn test_04_text_editor_blinking_state_machine() {
    let mut editor = PdfTextEditor::new();
    editor.start_editing(
        0,
        0,
        "Blink".into(),
        0,
        sample_rect(50.0, 50.0, 50.0, 10.0),
        PdfTextStyle::new("Helvetica", 10.0, [0.0, 0.0, 0.0, 1.0]),
    );

    assert!(editor.active_state().unwrap().caret_visible);

    // Advance 530ms -> toggles to hidden
    editor.tick_blink(530.0);
    assert!(!editor.active_state().unwrap().caret_visible);

    // Advance another 530ms -> toggles back to visible
    editor.tick_blink(530.0);
    assert!(editor.active_state().unwrap().caret_visible);

    // Typing resets blink timer and makes caret immediately visible
    editor.tick_blink(530.0); // Caret is hidden
    assert!(!editor.active_state().unwrap().caret_visible);
    editor.insert_char('X');
    assert!(editor.active_state().unwrap().caret_visible);
}

#[test]
fn test_05_object_editor_selection_and_hit_testing() {
    let mut obj_editor = PdfObjectEditor::new();
    let img_id = obj_editor.add_image(
        0,
        sample_rect(100.0, 100.0, 200.0, 150.0),
        "logo.png",
        "image/png",
    );

    assert_eq!(obj_editor.selected_id, Some(img_id));

    // Gizmo bounding box at (100, 100, 200, 150)
    let gizmo = PdfObjectGizmo::new(Rect2D::new(100.0, 100.0, 200.0, 150.0));

    // Hit test rotation handle (centered horizontally at 200, 24px above top = 76)
    let rot_hit = gizmo.hit_test((200.0, 76.0), 2.0);
    assert_eq!(rot_hit, Some(PdfGizmoHit::Rotate));

    // Hit test TopLeft handle at (100, 100)
    let tl_hit = gizmo.hit_test((100.0, 100.0), 2.0);
    assert_eq!(tl_hit, Some(PdfGizmoHit::Handle(ResizeHandle::TopLeft)));

    // Hit test BottomRight handle at (300, 250)
    let br_hit = gizmo.hit_test((300.0, 250.0), 2.0);
    assert_eq!(br_hit, Some(PdfGizmoHit::Handle(ResizeHandle::BottomRight)));

    // Hit test Body at (150, 150)
    let body_hit = gizmo.hit_test((150.0, 150.0), 2.0);
    assert_eq!(body_hit, Some(PdfGizmoHit::Body));

    // Hit test outside
    let outside_hit = gizmo.hit_test((400.0, 400.0), 2.0);
    assert_eq!(outside_hit, None);
}

#[test]
fn test_06_object_editor_translation() {
    let mut obj_editor = PdfObjectEditor::new();
    let _id = obj_editor.add_image(
        0,
        sample_rect(50.0, 50.0, 100.0, 100.0),
        "diagram.png",
        "image/png",
    );

    // Translate by (+20, -10)
    assert!(obj_editor.translate_selected(20.0, -10.0));

    let obj = obj_editor.selected_object().unwrap();
    assert_eq!(obj.rect.llx, 70.0);
    assert_eq!(obj.rect.urx, 170.0);
    assert_eq!(obj.rect.lly, 40.0);
    assert_eq!(obj.rect.ury, 140.0);

    // Lock object and attempt translation
    obj_editor.selected_object_mut().unwrap().is_locked = true;
    assert!(!obj_editor.translate_selected(10.0, 10.0));
    assert_eq!(obj_editor.selected_object().unwrap().rect.llx, 70.0);
}

#[test]
fn test_07_object_editor_8_handle_resize_with_aspect_lock() {
    let mut obj_editor = PdfObjectEditor::new();
    let _id = obj_editor.add_image(
        0,
        sample_rect(100.0, 100.0, 200.0, 100.0), // 2:1 aspect ratio
        "banner.png",
        "image/png",
    );

    // Resize from BottomRight without aspect lock (+50 dx, +20 dy)
    assert!(obj_editor.resize_selected(ResizeHandle::BottomRight, 50.0, 20.0, false, 10.0));
    assert_eq!(obj_editor.selected_object().unwrap().rect.width(), 250.0);

    // Resize from TopLeft with aspect lock
    let prev_w = obj_editor.selected_object().unwrap().rect.width();
    let prev_h = obj_editor.selected_object().unwrap().rect.height();
    let prev_aspect = prev_w / prev_h;

    assert!(obj_editor.resize_selected(ResizeHandle::TopLeft, -20.0, 0.0, true, 10.0));

    let new_w = obj_editor.selected_object().unwrap().rect.width();
    let new_h = obj_editor.selected_object().unwrap().rect.height();
    let new_aspect = new_w / new_h;
    assert!((prev_aspect - new_aspect).abs() < 1e-2);
}

#[test]
fn test_08_object_editor_rotation() {
    let mut obj_editor = PdfObjectEditor::new();
    let _id = obj_editor.add_image(
        0,
        sample_rect(50.0, 50.0, 50.0, 50.0),
        "icon.png",
        "image/png",
    );

    assert_eq!(obj_editor.selected_object().unwrap().rotation_deg, 0.0);

    // Rotate +45 degrees
    assert!(obj_editor.rotate_selected(45.0));
    assert_eq!(obj_editor.selected_object().unwrap().rotation_deg, 45.0);

    // Rotate past 360 degrees
    assert!(obj_editor.rotate_selected(330.0));
    assert_eq!(obj_editor.selected_object().unwrap().rotation_deg, 15.0);

    // Rotate negative degrees
    assert!(obj_editor.rotate_selected(-30.0));
    assert_eq!(obj_editor.selected_object().unwrap().rotation_deg, 345.0);
}

#[test]
fn test_09_page_organiser_reordering() {
    let boxes = vec![
        PdfRect::new(0.0, 0.0, 100.0, 100.0), // Original Page 0
        PdfRect::new(0.0, 0.0, 200.0, 200.0), // Original Page 1
        PdfRect::new(0.0, 0.0, 300.0, 300.0), // Original Page 2
    ];
    let mut organiser = PdfPageOrganiser::new(&boxes);
    assert_eq!(organiser.page_count(), 3);

    // Move Page 0 to Index 2 (end of document)
    organiser.reorder_page(0, 2).unwrap();

    assert_eq!(organiser.get_page(0).unwrap().media_box.width(), 200.0);
    assert_eq!(organiser.get_page(1).unwrap().media_box.width(), 300.0);
    assert_eq!(organiser.get_page(2).unwrap().media_box.width(), 100.0);

    // Out of bounds checks
    assert!(organiser.reorder_page(0, 10).is_err());
}

#[test]
fn test_10_page_organiser_rotation() {
    let boxes = vec![PdfRect::letter()];
    let mut organiser = PdfPageOrganiser::new(&boxes);

    assert_eq!(organiser.get_page(0).unwrap().rotation, 0);

    // Clockwise: 0 -> 90 -> 180 -> 270 -> 0
    assert_eq!(organiser.rotate_page(0, true).unwrap(), 90);
    assert_eq!(organiser.rotate_page(0, true).unwrap(), 180);
    assert_eq!(organiser.rotate_page(0, true).unwrap(), 270);
    assert_eq!(organiser.rotate_page(0, true).unwrap(), 0);

    // Counter-clockwise: 0 -> 270
    assert_eq!(organiser.rotate_page(0, false).unwrap(), 270);
}

#[test]
fn test_11_page_organiser_insertion_deletion_and_duplication() {
    let boxes = vec![PdfRect::letter(), PdfRect::a4()];
    let mut organiser = PdfPageOrganiser::new(&boxes);
    assert_eq!(organiser.page_count(), 2);

    // Duplicate page 0
    let dup_idx = organiser.duplicate_page(0).unwrap();
    assert_eq!(dup_idx, 1);
    assert_eq!(organiser.page_count(), 3);

    // Delete duplicated page
    let removed = organiser.delete_page(1).unwrap();
    assert_eq!(removed.media_box, PdfRect::letter());
    assert_eq!(organiser.page_count(), 2);

    // Insert back
    organiser.insert_page(1, removed).unwrap();
    assert_eq!(organiser.page_count(), 3);

    // Attempting to delete until only 1 remains, then deleting last must fail
    organiser.delete_page(0).unwrap();
    organiser.delete_page(0).unwrap();
    assert_eq!(organiser.page_count(), 1);
    assert!(organiser.delete_page(0).is_err());
}

#[test]
fn test_12_page_organiser_thumbnail_grid_layout() {
    let boxes = vec![PdfRect::letter(); 6];
    let organiser = PdfPageOrganiser::new(&boxes);

    let thumbnails = organiser.compute_thumbnail_grid(800.0, 600.0, 120.0, 16.0);
    assert_eq!(thumbnails.len(), 6);

    // Verify grid layout structure
    assert_eq!(thumbnails[0].grid_row, 0);
    assert_eq!(thumbnails[0].grid_col, 0);
    assert_eq!(thumbnails[0].label, "Page 1 of 6");

    assert_eq!(thumbnails[5].page_index, 5);
    assert_eq!(thumbnails[5].label, "Page 6 of 6");
}

#[test]
fn test_13_form_filler_focus_typing_and_validation() {
    let mut filler = PdfFormFiller::new();

    filler.focus_field("Customer_Name");
    assert_eq!(filler.focused_field, Some("Customer_Name".into()));

    // Type "Alice"
    for ch in "Alice".chars() {
        assert!(filler.type_char(ch));
    }
    assert_eq!(filler.get_value("Customer_Name"), Some("Alice"));

    // Backspace
    assert!(filler.backspace());
    assert_eq!(filler.get_value("Customer_Name"), Some("Alic"));

    // Validation
    let req = vec!["Customer_Name".to_string(), "Required_Phone".to_string()];
    let res = filler.validate_required(&req);
    assert!(res.is_err());
    let missing = res.unwrap_err();
    assert_eq!(missing, vec!["Required_Phone"]);

    // Fill Required_Phone
    filler.set_value("Required_Phone", "+1-555-0199");
    assert!(filler.validate_required(&req).is_ok());
}

#[test]
fn test_14_form_filler_checkbox_and_dropdown_selection() {
    let mut filler = PdfFormFiller::new();

    // Checkbox toggling
    assert!(filler.toggle_checkbox("Agree_Terms")); // Off -> Yes
    assert_eq!(filler.get_value("Agree_Terms"), Some("Yes"));
    assert!(!filler.toggle_checkbox("Agree_Terms")); // Yes -> Off
    assert_eq!(filler.get_value("Agree_Terms"), Some("Off"));

    // Choice dropdown selection
    filler.select_choice("Country", "Canada");
    assert_eq!(filler.get_value("Country"), Some("Canada"));
    filler.select_choice("Country", "United Kingdom");
    assert_eq!(filler.get_value("Country"), Some("United Kingdom"));
}

#[test]
fn test_15_transactional_command_bus_undo_redo_and_coalescing() {
    let mut bus = CommandBus::new();
    let mut state = DocumentState::default();

    let cmd1 = Box::new(PdfEditTextCommand {
        page_index: 0,
        run_index: 0,
        old_text: "Original".into(),
        new_text: "Edit 1".into(),
    });

    bus.execute(&mut state, cmd1).unwrap();
    assert_eq!(state.text_content, "Edit 1");

    // Consecutive edit on same run merges into 1 command
    let cmd2 = Box::new(PdfEditTextCommand {
        page_index: 0,
        run_index: 0,
        old_text: "Edit 1".into(),
        new_text: "Edit 12".into(),
    });
    bus.execute(&mut state, cmd2).unwrap();
    assert_eq!(state.text_content, "Edit 12");
    assert_eq!(bus.undo_stack_len(), 1); // Coalesced

    // Undo reverts back to "Original"
    bus.undo(&mut state).unwrap();
    assert_eq!(state.text_content, "Original");

    // Redo restores to "Edit 12"
    bus.redo(&mut state).unwrap();
    assert_eq!(state.text_content, "Edit 12");
}

#[test]
fn test_16_performance_interactive_typing_and_drag_latency_sla() {
    let mut editor = PdfTextEditor::new();
    editor.start_editing(
        0,
        0,
        "Benchmark Performance SLA".into(),
        10,
        sample_rect(50.0, 50.0, 200.0, 14.0),
        PdfTextStyle::new("Helvetica", 12.0, [0.0, 0.0, 0.0, 1.0]),
    );

    let mut obj_editor = PdfObjectEditor::new();
    obj_editor.add_image(
        0,
        sample_rect(100.0, 100.0, 150.0, 150.0),
        "test.png",
        "image/png",
    );

    let start = Instant::now();

    // Perform 100 simulated keystrokes
    for i in 0..100 {
        editor.insert_char((b'a' + (i % 26) as u8) as char);
        obj_editor.translate_selected(0.5, 0.5);
    }

    let elapsed = start.elapsed();
    let per_op = elapsed / 100;

    // Hard SLA: Word/PDF typing < 1.0 ms per keystroke
    assert!(
        per_op.as_micros() < 1000,
        "SLA Violation: Expected per-keystroke latency < 1.0 ms, took {:?}",
        per_op
    );
}

#[test]
fn test_17_editor_overlay_display_list_rendering() {
    let mut text_editor = PdfTextEditor::new();
    text_editor.start_editing(
        0,
        0,
        "Editing Text".into(),
        4,
        sample_rect(72.0, 700.0, 100.0, 14.0),
        PdfTextStyle::new("Helvetica", 12.0, [0.0, 0.0, 0.0, 1.0]),
    );

    let mut obj_editor = PdfObjectEditor::new();
    obj_editor.add_image(
        0,
        sample_rect(100.0, 100.0, 150.0, 150.0),
        "image.jpg",
        "image/jpeg",
    );

    let mut form_filler = PdfFormFiller::new();
    form_filler.focus_field("User_Input");

    let page_boxes = vec![PdfRect::letter()];
    let config = PdfViewportConfig::new(1280.0, 800.0);
    let layout = PdfDocumentLayout::calculate(&page_boxes, &config);
    let page_layout = layout.get_page(0).unwrap();

    let mut dl = DisplayList::new();
    PdfEditorDisplayRenderer::render_editor_overlays(
        &mut dl,
        &text_editor,
        &obj_editor,
        &form_filler,
        page_layout,
        &config,
    );

    assert!(!dl.is_empty());
    // Must contain text editor box, caret line, gizmo outline, 8 handles, rotation stem/circle, form glow
    assert!(dl.quad_count >= 10);

    // Test remaining commands
    let mut bus = CommandBus::new();
    let mut state = DocumentState::default();
    let move_cmd = Box::new(PdfMoveObjectCommand {
        object_id: 1,
        old_rect: sample_rect(10.0, 10.0, 50.0, 50.0),
        new_rect: sample_rect(20.0, 20.0, 50.0, 50.0),
    });
    bus.execute(&mut state, move_cmd).unwrap();

    let rot_cmd = Box::new(PdfRotatePageCommand {
        page_index: 0,
        old_rotation: 0,
        new_rotation: 90,
    });
    bus.execute(&mut state, rot_cmd).unwrap();

    let form_cmd = Box::new(PdfSetFormFieldCommand {
        field_name: "Name".into(),
        old_value: "A".into(),
        new_value: "B".into(),
    });
    bus.execute(&mut state, form_cmd).unwrap();
    assert_eq!(bus.undo_stack_len(), 3);
}
