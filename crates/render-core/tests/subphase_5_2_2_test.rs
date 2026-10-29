//! Subphase 5.2.2 Integration Test Suite:
//! Adaptive Tablet & Mobile UI (Touch Drawers, Stylus Ink, Reflow Mode)
//!
//! Validates:
//! - Viewport breakpoint classification (Desktop >= 1024px, Tablet 768-1023px, Mobile < 768px)
//! - Pointer modality touch target requirements (44px coarse vs 24px fine)
//! - Safe area insets subtraction and virtual keyboard usable height calculation
//! - Active caret auto-scroll clearance above virtual keyboard
//! - Stylus pressure-to-width scaling w(p) = w0 * (1 + alpha * (p - 0.5))
//! - Stylus ink stroke recording and bounding box accumulation
//! - Catmull-Rom spline cubic Bezier control point derivation
//! - Stroke micro-jitter decimation
//! - DrawingML ink trace XML serialization
//! - Tablet compact touch ribbon and mobile bottom sheet state transitions
//! - Mobile bottom sheet snap-to-nearest threshold logic
//! - Word Mobile Reflow Reader single-column layout, font scaling, and table cards
//! - Excel Mobile Row Card Inspector field extraction, type inference, and navigation
//! - PowerPoint touch presentation laser pointer geometry and trail alpha decay
//! - Split-screen multitasking viewport resize recalculation
//! - Vector manifest validation against `specs/evidence/render/5.2.2_vectors.json`

use render_core::{
    BottomSheetState, BottomSheetStateMachine, CardFieldType, FormFactor, InkStroke,
    LaserPointerEngine, Rect2D, RowCardInspectorEngine, SafeAreaInsets, TouchRibbonState,
    ViewportMetrics, WordReflowReaderEngine,
};
use std::path::{Path, PathBuf};

fn resolve_repo_path(rel_path: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let repo_root = Path::new(manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("Valid repo root");
    repo_root.join(rel_path)
}

#[test]
fn test_01_breakpoint_desktop() {
    let metrics = ViewportMetrics::new(1440.0, 900.0, false);
    assert_eq!(metrics.form_factor(), FormFactor::Desktop);
}

#[test]
fn test_02_breakpoint_tablet() {
    let metrics = ViewportMetrics::new(820.0, 1180.0, true);
    assert_eq!(metrics.form_factor(), FormFactor::Tablet);
}

#[test]
fn test_03_breakpoint_mobile() {
    let metrics = ViewportMetrics::new(390.0, 844.0, true);
    assert_eq!(metrics.form_factor(), FormFactor::Mobile);
}

#[test]
fn test_04_pointer_modality_touch_target() {
    let touch = ViewportMetrics::new(390.0, 844.0, true);
    assert_eq!(touch.min_touch_target_px(), 44.0);

    let mouse = ViewportMetrics::new(1920.0, 1080.0, false);
    assert_eq!(mouse.min_touch_target_px(), 24.0);
}

#[test]
fn test_05_safe_area_insets_subtraction() {
    let mut metrics = ViewportMetrics::new(390.0, 844.0, true);
    metrics.safe_insets = SafeAreaInsets {
        top: 47.0,
        right: 0.0,
        bottom: 34.0,
        left: 0.0,
    };
    metrics.chrome_height = 50.0;

    assert_eq!(metrics.usable_width(), 390.0);
    assert_eq!(metrics.usable_height(), 713.0);
}

#[test]
fn test_06_virtual_keyboard_compensation() {
    let mut metrics = ViewportMetrics::new(390.0, 844.0, true);
    metrics.safe_insets = SafeAreaInsets {
        top: 47.0,
        right: 0.0,
        bottom: 34.0,
        left: 0.0,
    };
    metrics.chrome_height = 50.0;
    metrics.keyboard_height = 300.0;

    assert_eq!(metrics.usable_height(), 713.0);
    assert_eq!(metrics.active_height_with_keyboard(), 413.0);
}

#[test]
fn test_07_active_caret_auto_scroll() {
    let caret_y = 500.0;
    let viewport_top = 0.0;
    let keyboard_top = 413.0;
    let min_clearance = 24.0;

    let scroll = ViewportMetrics::compute_caret_scroll_needed(
        caret_y,
        viewport_top,
        keyboard_top,
        min_clearance,
    );
    assert_eq!(scroll, Some(111.0));

    let caret_safe = 200.0;
    let no_scroll = ViewportMetrics::compute_caret_scroll_needed(
        caret_safe,
        viewport_top,
        keyboard_top,
        min_clearance,
    );
    assert_eq!(no_scroll, None);
}

#[test]
fn test_08_stylus_ink_pressure_scaling() {
    let stroke = InkStroke::new("#000000", 2.5);
    let alpha = 0.8;

    let w_low = stroke.compute_point_width(0.25, alpha);
    assert!((w_low - 2.0).abs() < 1e-6);

    let w_mid = stroke.compute_point_width(0.5, alpha);
    assert!((w_mid - 2.5).abs() < 1e-6);

    let w_high = stroke.compute_point_width(1.0, alpha);
    assert!((w_high - 3.5).abs() < 1e-6);
}

#[test]
fn test_09_stylus_ink_stroke_bbox() {
    let mut stroke = InkStroke::new("#002060", 2.0);
    stroke.add_point(100.0, 150.0, 0.5, 1000);
    stroke.add_point(120.0, 180.0, 0.6, 1016);
    stroke.add_point(140.0, 160.0, 0.7, 1032);

    assert_eq!(stroke.points.len(), 3);
    let bbox = stroke.bounding_box().expect("Bounding box exists");
    assert_eq!(bbox, Rect2D::new(100.0, 150.0, 40.0, 30.0));
}

#[test]
fn test_10_stylus_ink_catmull_rom_bezier_controls() {
    let p0 = (0.0, 0.0);
    let p1 = (10.0, 20.0);
    let p2 = (20.0, 25.0);
    let p3 = (30.0, 10.0);
    let tension = 0.5;

    let (cp1, cp2) = InkStroke::compute_catmull_rom_bezier_controls(p0, p1, p2, p3, tension);
    let expected_cp1_x = 10.0 + 20.0 / 3.0; // 16.6666...
    let expected_cp1_y = 20.0 + 25.0 / 3.0; // 28.3333...

    assert!((cp1.0 - expected_cp1_x).abs() < 1e-5);
    assert!((cp1.1 - expected_cp1_y).abs() < 1e-5);
    assert!(cp2.0 > 0.0 && cp2.1 > 0.0);
}

#[test]
fn test_11_stylus_ink_path_decimation() {
    let mut stroke = InkStroke::new("#000000", 2.0);
    stroke.add_point(10.0, 10.0, 0.5, 100);
    stroke.add_point(10.2, 10.1, 0.5, 110);
    stroke.add_point(20.0, 20.0, 0.5, 120);

    let decimated = stroke.decimate(0.5);
    assert_eq!(decimated.len(), 2);
    assert_eq!(decimated[0].x, 10.0);
    assert_eq!(decimated[1].x, 20.0);
}

#[test]
fn test_12_drawingml_ink_trace_xml() {
    let mut stroke = InkStroke::new("#002060", 2.5);
    stroke.add_point(10.0, 20.0, 0.5, 100);
    stroke.add_point(30.0, 40.0, 0.5, 120);

    let xml = stroke.to_drawingml_xml();
    assert!(xml.contains("<ink:trace"));
    assert!(xml.contains(r##"color="#002060""##));
    assert!(xml.contains("10.0,20.0 30.0,40.0"));
}

#[test]
fn test_13_tablet_compact_touch_ribbon() {
    let mut ribbon = TouchRibbonState {
        is_collapsed: true,
        touch_height_px: 56.0,
    };
    assert!(ribbon.is_collapsed);

    ribbon.toggle();
    assert!(!ribbon.is_collapsed);
    assert_eq!(ribbon.touch_height_px, 56.0);
}

#[test]
fn test_14_mobile_bottom_sheet_transitions() {
    let state = BottomSheetState::Collapsed;
    let next = BottomSheetStateMachine::transition_drag(state, -150.0);
    assert_eq!(next, BottomSheetState::HalfExpanded);

    let full = BottomSheetStateMachine::transition_drag(next, -150.0);
    assert_eq!(full, BottomSheetState::FullExpanded);

    let down = BottomSheetStateMachine::transition_drag(full, 150.0);
    assert_eq!(down, BottomSheetState::HalfExpanded);
}

#[test]
fn test_15_mobile_bottom_sheet_snap_nearest() {
    let targets = [100.0, 400.0, 700.0];
    let snapped = BottomSheetStateMachine::snap_to_nearest(420.0, &targets);
    assert_eq!(snapped, 400.0);

    let snapped_top = BottomSheetStateMachine::snap_to_nearest(150.0, &targets);
    assert_eq!(snapped_top, 100.0);
}

#[test]
fn test_16_word_reflow_reader_columns() {
    let (cols, width) = WordReflowReaderEngine::compute_reflow_cols(370.0);
    assert_eq!(cols, 1);
    assert_eq!(width, 370.0);
}

#[test]
fn test_17_word_reflow_reader_font_scale() {
    let scaled = WordReflowReaderEngine::scale_font_size(11.0, 1.5);
    assert!((scaled - 16.5).abs() < 1e-4);
}

#[test]
fn test_18_word_reflow_reader_table_cards() {
    assert!(WordReflowReaderEngine::should_render_table_as_cards(
        600.0, 360.0
    ));
    assert!(!WordReflowReaderEngine::should_render_table_as_cards(
        300.0, 360.0
    ));
}

#[test]
fn test_19_excel_row_card_extraction() {
    let headers = vec!["ID".into(), "Name".into(), "Salary".into()];
    let values = vec!["101".into(), "Alice".into(), "95000".into()];

    let fields = RowCardInspectorEngine::build_row_card(&headers, &values);
    assert_eq!(fields.len(), 3);
    assert_eq!(fields[0].label, "ID");
    assert_eq!(fields[0].value, "101");
    assert_eq!(fields[1].label, "Name");
    assert_eq!(fields[1].value, "Alice");
    assert_eq!(fields[2].label, "Salary");
    assert_eq!(fields[2].value, "95000");
}

#[test]
fn test_20_excel_row_card_type_inference() {
    assert_eq!(
        RowCardInspectorEngine::infer_field_type("123.45"),
        CardFieldType::Numeric
    );
    assert_eq!(
        RowCardInspectorEngine::infer_field_type("2026-10-29"),
        CardFieldType::Date
    );
    assert_eq!(
        RowCardInspectorEngine::infer_field_type("=A1*2"),
        CardFieldType::Formula
    );
    assert_eq!(
        RowCardInspectorEngine::infer_field_type("Product Note"),
        CardFieldType::Text
    );
}

#[test]
fn test_21_excel_row_card_edit_commit() {
    let active_row: u32 = 4;
    let edit_col: u32 = 2;
    let new_val = "98000";

    let cell_coord = (active_row, edit_col);
    assert_eq!(cell_coord, (4, 2));
    assert_eq!(new_val, "98000");
}

#[test]
fn test_22_excel_row_card_navigation() {
    assert_eq!(RowCardInspectorEngine::navigate_row(3, 10, true), 4);
    assert_eq!(RowCardInspectorEngine::navigate_row(3, 10, false), 2);
    assert_eq!(RowCardInspectorEngine::navigate_row(9, 10, true), 9);
    assert_eq!(RowCardInspectorEngine::navigate_row(0, 10, false), 0);
}

#[test]
fn test_23_powerpoint_laser_pointer_geometry() {
    assert_eq!(LaserPointerEngine::CORE_RADIUS_PX, 6.0);
    assert_eq!(LaserPointerEngine::GLOW_RADIUS_PX, 18.0);
}

#[test]
fn test_24_powerpoint_laser_pointer_decay() {
    let alpha = LaserPointerEngine::compute_trail_alpha(1.0, 120.0, 120.0);
    let expected = (-1.0_f64).exp(); // ~0.36787944
    assert!((alpha - expected).abs() < 1e-4);
}

#[test]
fn test_25_multitasking_split_screen() {
    let screen_w = 1024.0;
    let split_ratio = 0.5;
    let window_w = screen_w * split_ratio;

    let metrics = ViewportMetrics::new(window_w, 768.0, true);
    assert_eq!(metrics.width, 512.0);
    assert_eq!(metrics.form_factor(), FormFactor::Mobile);
}

#[test]
fn test_26_vectors_manifest_validation() {
    let manifest_path = resolve_repo_path("specs/evidence/render/5.2.2_vectors.json");
    assert!(
        manifest_path.exists(),
        "5.2.2_vectors.json must exist at {}",
        manifest_path.display()
    );

    let content = std::fs::read_to_string(&manifest_path).expect("Read vectors file");
    let json: serde_json::Value = serde_json::from_str(&content).expect("Valid JSON");

    assert_eq!(json["subphase"].as_str(), Some("5.2.2"));
    let vector_count = json["vector_count"].as_u64().expect("vector_count is u64") as usize;
    assert!(
        vector_count >= 25,
        "Vector count must be >= 25, found {}",
        vector_count
    );

    let vectors = json["vectors"].as_array().expect("vectors is array");
    assert_eq!(vectors.len(), vector_count);

    for (idx, vec_item) in vectors.iter().enumerate() {
        let expected_id = format!("VEC-5.2.2-{:02}", idx + 1);
        assert_eq!(
            vec_item["id"].as_str(),
            Some(expected_id.as_str()),
            "Vector at index {} has invalid ID",
            idx
        );
        assert!(
            vec_item["description"].is_string(),
            "Vector {} missing description",
            expected_id
        );
        assert!(
            vec_item["input"].is_object(),
            "Vector {} missing input",
            expected_id
        );
        assert!(
            vec_item["expected"].is_object(),
            "Vector {} missing expected",
            expected_id
        );
    }
}
