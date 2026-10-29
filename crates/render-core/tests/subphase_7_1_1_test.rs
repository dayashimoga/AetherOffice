//! Subphase 7.1.1 Integration Test Suite:
//! Desktop / Tablet / Mobile Responsive Breakpoint Adaptation
//!
//! Validates:
//! - Viewport breakpoint classification (Desktop >= 1024px, Tablet 768px-1023px, Mobile < 768px)
//! - Safe area insets (notches, home bars) and usable viewport dimensions
//! - Virtual keyboard inset avoidance and caret auto-scroll clearance calculation
//! - Device orientation detection (Portrait vs Landscape)
//! - AdaptiveShellController dynamic resizing, mode toggles, and UI adaptations
//! - Tablet touch ribbon 56px touch target height and toggles
//! - Mobile bottom sheet state machine (Collapsed, HalfExpanded, FullExpanded) and drag transitions
//! - Word Mobile Reflow Reader single-column layout, dynamic font scaling, and table-to-card triggers
//! - Excel Mobile Row Card Inspector data type inference (Numeric, Date, Formula, Text) and row navigation
//! - Interactive CardEditorSession generating `ExcelSetCellTransaction` dispatched through Command Bus
//! - Pressure-sensitive stylus freehand ink layer, Catmull-Rom smoothing, decimation, and DrawingML/SVG export
//! - Stylus AddInkStrokeTransaction and ClearInkTransaction on GlobalCommandBus
//! - PowerPoint touch presentation laser pointer physics with exponential decay
//! - Multitasking side-by-side and stacked split-view geometry computation
//! - Sub-millisecond calculation latency SLA (< 1.0 ms)

use render_core::adaptive::{
    AdaptiveShellController, AddInkStrokeTransaction, BottomSheetState, BottomSheetStateMachine,
    CardEditorSession, CardFieldType, ClearInkTransaction, DeviceOrientation, FormFactor,
    InkStroke, LaserPointerEngine, PointerModality, RowCardInspectorEngine, SafeAreaInsets,
    SplitOrientation, SplitViewConfig, TouchRibbonState, ViewportMetrics, WordReflowReaderEngine,
};
use render_core::commands::{GlobalCommandBus, GlobalOfficeState, OfficeTransaction};
use render_core::primitive::Rect2D;
use render_core::shell::DocumentType;
use std::time::Instant;

// =========================================================================
// Tests
// =========================================================================

#[test]
fn test_01_viewport_metrics_breakpoint_classification() {
    // Desktop: >= 1024px
    let desktop_uhd = ViewportMetrics::new(3840.0, 2160.0, false);
    assert_eq!(desktop_uhd.form_factor(), FormFactor::Desktop);
    assert_eq!(desktop_uhd.min_touch_target_px(), 24.0);

    let desktop_fhd = ViewportMetrics::new(1920.0, 1080.0, false);
    assert_eq!(desktop_fhd.form_factor(), FormFactor::Desktop);

    let desktop_min = ViewportMetrics::new(1024.0, 768.0, false);
    assert_eq!(desktop_min.form_factor(), FormFactor::Desktop);

    // Tablet: 768px <= width < 1024px
    let tablet_ipad_pro = ViewportMetrics::new(1023.0, 1366.0, true);
    assert_eq!(tablet_ipad_pro.form_factor(), FormFactor::Tablet);
    assert_eq!(tablet_ipad_pro.min_touch_target_px(), 44.0);

    let tablet_ipad_air = ViewportMetrics::new(820.0, 1180.0, true);
    assert_eq!(tablet_ipad_air.form_factor(), FormFactor::Tablet);

    let tablet_min = ViewportMetrics::new(768.0, 1024.0, true);
    assert_eq!(tablet_min.form_factor(), FormFactor::Tablet);

    // Mobile: < 768px
    let mobile_large = ViewportMetrics::new(767.0, 1000.0, true);
    assert_eq!(mobile_large.form_factor(), FormFactor::Mobile);

    let mobile_pixel = ViewportMetrics::new(412.0, 915.0, true);
    assert_eq!(mobile_pixel.form_factor(), FormFactor::Mobile);

    let mobile_iphone = ViewportMetrics::new(390.0, 844.0, true);
    assert_eq!(mobile_iphone.form_factor(), FormFactor::Mobile);
    assert_eq!(mobile_iphone.min_touch_target_px(), 44.0);
}

#[test]
fn test_02_safe_area_insets_and_usable_dimensions() {
    let mut metrics = ViewportMetrics::new(390.0, 844.0, true);
    metrics.safe_insets = SafeAreaInsets {
        top: 47.0, // Dynamic Island / notch
        right: 0.0,
        bottom: 34.0, // Home indicator bar
        left: 0.0,
    };
    metrics.chrome_height = 44.0; // Header app bar

    assert_eq!(metrics.usable_width(), 390.0);
    // 844 - 47 - 34 - 44 = 719px
    assert_eq!(metrics.usable_height(), 719.0);
}

#[test]
fn test_03_virtual_keyboard_inset_avoidance_and_auto_scroll() {
    let mut metrics = ViewportMetrics::new(390.0, 844.0, true);
    metrics.keyboard_height = 300.0; // Virtual keyboard takes 300px
    metrics.chrome_height = 44.0;

    // Active height available for document with keyboard open:
    // 844 - 44 - 300 = 500px
    assert_eq!(metrics.active_height_with_keyboard(), 500.0);

    // Caret at y=600 when keyboard starts at y=544 (844 - 300):
    // Caret is hidden behind keyboard!
    // Auto-scroll needed with 24px clearance:
    let scroll_needed = ViewportMetrics::compute_caret_scroll_needed(
        600.0, // caret_y
        0.0,   // viewport_top
        544.0, // keyboard_top
        24.0,  // min_clearance
    );

    // Target max y = 544 - 24 = 520. Caret is at 600. Delta = 600 - 520 = 80px scroll needed!
    assert_eq!(scroll_needed, Some(80.0));

    // Caret at y=300 (above keyboard):
    let no_scroll = ViewportMetrics::compute_caret_scroll_needed(300.0, 0.0, 544.0, 24.0);
    assert_eq!(no_scroll, None);
}

#[test]
fn test_04_device_orientation_detection() {
    let mut ctrl = AdaptiveShellController::new(1280.0, 800.0, PointerModality::Fine);
    assert_eq!(ctrl.orientation(), DeviceOrientation::Landscape);

    // Portrait tablet (e.g. iPad Air upright)
    ctrl.resize(820.0, 1180.0);
    assert_eq!(ctrl.orientation(), DeviceOrientation::Portrait);
    assert_eq!(ctrl.form_factor(), FormFactor::Tablet);

    // Portrait phone
    ctrl.resize(390.0, 844.0);
    assert_eq!(ctrl.orientation(), DeviceOrientation::Portrait);
    assert_eq!(ctrl.form_factor(), FormFactor::Mobile);

    // Landscape phone
    ctrl.resize(844.0, 390.0);
    assert_eq!(ctrl.orientation(), DeviceOrientation::Landscape);
    assert_eq!(ctrl.form_factor(), FormFactor::Tablet); // Width 844 >= 768
}

#[test]
fn test_05_adaptive_shell_controller_dynamic_resizing() {
    let mut ctrl = AdaptiveShellController::new(1920.0, 1080.0, PointerModality::Fine);

    // 1. Desktop mode
    assert_eq!(ctrl.form_factor(), FormFactor::Desktop);
    assert!(ctrl.should_show_full_ribbon());
    assert!(!ctrl.should_show_touch_ribbon());
    assert!(!ctrl.should_show_bottom_sheet());
    assert!(ctrl.should_show_formula_bar());
    assert!(!ctrl.word_reflow_mode);
    assert!(!ctrl.excel_card_mode);

    // 2. Resize to Tablet (820px)
    ctrl.resize(820.0, 1180.0);
    assert_eq!(ctrl.form_factor(), FormFactor::Tablet);
    assert!(!ctrl.should_show_full_ribbon());
    assert!(ctrl.should_show_touch_ribbon());
    assert!(!ctrl.should_show_bottom_sheet());
    assert!(ctrl.should_show_formula_bar());
    assert!(!ctrl.word_reflow_mode);
    assert!(!ctrl.excel_card_mode);

    // 3. Resize to Mobile (390px)
    ctrl.resize(390.0, 844.0);
    assert_eq!(ctrl.form_factor(), FormFactor::Mobile);
    assert!(!ctrl.should_show_full_ribbon());
    assert!(!ctrl.should_show_touch_ribbon());
    assert!(ctrl.should_show_bottom_sheet());
    assert!(!ctrl.should_show_formula_bar());
    assert!(ctrl.word_reflow_mode);
    assert!(ctrl.excel_card_mode);
}

#[test]
fn test_06_touch_ribbon_state_collapsing() {
    let mut ribbon = TouchRibbonState::default();
    assert_eq!(ribbon.touch_height_px, 56.0);
    assert!(!ribbon.is_collapsed);

    // Toggle collapse
    assert!(ribbon.toggle());
    assert!(ribbon.is_collapsed);

    // Toggle expand
    assert!(!ribbon.toggle());
    assert!(!ribbon.is_collapsed);
}

#[test]
fn test_07_mobile_bottom_sheet_state_machine_and_gestures() {
    let state = BottomSheetState::Collapsed;

    // Drag up (-150px) -> HalfExpanded
    let state = BottomSheetStateMachine::transition_drag(state, -150.0);
    assert_eq!(state, BottomSheetState::HalfExpanded);

    // Drag up again (-120px) -> FullExpanded
    let state = BottomSheetStateMachine::transition_drag(state, -120.0);
    assert_eq!(state, BottomSheetState::FullExpanded);

    // Small drag (-30px, under threshold) -> retains FullExpanded
    let state = BottomSheetStateMachine::transition_drag(state, -30.0);
    assert_eq!(state, BottomSheetState::FullExpanded);

    // Drag down (150px) -> HalfExpanded
    let state = BottomSheetStateMachine::transition_drag(state, 150.0);
    assert_eq!(state, BottomSheetState::HalfExpanded);

    // Drag down (150px) -> Collapsed
    let state = BottomSheetStateMachine::transition_drag(state, 150.0);
    assert_eq!(state, BottomSheetState::Collapsed);

    // Snap to nearest target height: targets [64.0 (peek), 400.0 (half), 800.0 (full)]
    let targets = [64.0, 400.0, 800.0];
    assert_eq!(
        BottomSheetStateMachine::snap_to_nearest(80.0, &targets),
        64.0
    );
    assert_eq!(
        BottomSheetStateMachine::snap_to_nearest(350.0, &targets),
        400.0
    );
    assert_eq!(
        BottomSheetStateMachine::snap_to_nearest(750.0, &targets),
        800.0
    );
}

#[test]
fn test_08_word_reflow_reader_fluid_column_and_font_scaling() {
    // 1. Fluid single column matching usable width
    let (cols, width) = WordReflowReaderEngine::compute_reflow_cols(390.0);
    assert_eq!(cols, 1);
    assert_eq!(width, 390.0);

    // 2. Dynamic font scaling
    let normal_font = WordReflowReaderEngine::scale_font_size(12.0, 1.0);
    assert_eq!(normal_font, 12.0);

    let large_font = WordReflowReaderEngine::scale_font_size(12.0, 1.5);
    assert_eq!(large_font, 18.0);

    let min_clamped = WordReflowReaderEngine::scale_font_size(4.0, 1.0);
    assert_eq!(min_clamped, 8.0); // Min clamp at 8.0 pt

    // 3. Table to card trigger
    assert!(WordReflowReaderEngine::should_render_table_as_cards(
        500.0, 390.0
    ));
    assert!(!WordReflowReaderEngine::should_render_table_as_cards(
        350.0, 390.0
    ));
}

#[test]
fn test_09_excel_mobile_row_card_inspector_type_inference() {
    // Type inference
    assert_eq!(
        RowCardInspectorEngine::infer_field_type("=SUM(B1:B10)"),
        CardFieldType::Formula
    );
    assert_eq!(
        RowCardInspectorEngine::infer_field_type("2026-10-29"),
        CardFieldType::Date
    );
    assert_eq!(
        RowCardInspectorEngine::infer_field_type("12345.67"),
        CardFieldType::Numeric
    );
    assert_eq!(
        RowCardInspectorEngine::infer_field_type("OpenOfficeRS Suite"),
        CardFieldType::Text
    );

    // Build row card
    let headers = vec![
        "ID".to_string(),
        "Date".to_string(),
        "Amount".to_string(),
        "Formula".to_string(),
        "Description".to_string(),
    ];
    let row_values = vec![
        "101".to_string(),
        "2026-10-29".to_string(),
        "499.95".to_string(),
        "=B2*1.2".to_string(),
        "Pro License".to_string(),
    ];

    let fields = RowCardInspectorEngine::build_row_card(&headers, &row_values);
    assert_eq!(fields.len(), 5);
    assert_eq!(fields[0].field_type, CardFieldType::Numeric);
    assert_eq!(fields[1].field_type, CardFieldType::Date);
    assert_eq!(fields[2].field_type, CardFieldType::Numeric);
    assert_eq!(fields[3].field_type, CardFieldType::Formula);
    assert_eq!(fields[4].field_type, CardFieldType::Text);

    // Row navigation
    assert_eq!(RowCardInspectorEngine::navigate_row(0, 10, true), 1);
    assert_eq!(RowCardInspectorEngine::navigate_row(9, 10, true), 9); // Max clamp
    assert_eq!(RowCardInspectorEngine::navigate_row(5, 10, false), 4);
    assert_eq!(RowCardInspectorEngine::navigate_row(0, 10, false), 0); // Min clamp
}

#[test]
fn test_10_excel_card_editor_session_transaction_dispatch() {
    let mut bus = GlobalCommandBus::new();
    let mut state = GlobalOfficeState::new();

    let headers = vec!["Product".to_string(), "Price".to_string()];
    let values = vec!["Laptop".to_string(), "999.00".to_string()];
    let fields = RowCardInspectorEngine::build_row_card(&headers, &values);

    let mut session = CardEditorSession::new("Sheet1", 1, fields); // Row 1 (A2, B2)

    // Edit price field (col_index 1 -> B2)
    let tx = session
        .update_field_value(1, "1299.00")
        .expect("Update field value");
    assert_eq!(tx.coord, "B2");
    assert_eq!(tx.new_value, "1299.00");

    // Dispatch transaction through Command Bus!
    bus.execute(&mut state, Box::new(tx))
        .expect("Dispatch card edit transaction");
    assert_eq!(state.get_excel_cell("B2"), Some("1299.00"));
    assert!(bus.is_dirty());

    // Undo reverts cell value
    bus.undo(&mut state).expect("Undo card edit");
    assert_eq!(state.get_excel_cell("B2"), None);
    assert!(!bus.is_dirty());
}

#[test]
fn test_11_stylus_drawingml_freehand_ink_and_pressure() {
    let mut stroke = InkStroke::new("#0078D7", 2.0);

    // Add 4 points with varying pressures
    stroke.add_point(10.0, 10.0, 0.2, 100);
    stroke.add_point(20.0, 15.0, 0.5, 120);
    stroke.add_point(30.0, 25.0, 0.8, 140);
    stroke.add_point(40.0, 40.0, 1.0, 160);

    assert_eq!(stroke.points.len(), 4);

    // Pressure to width scaling: base = 2.0, alpha = 0.5
    // p=0.5 -> 2.0 * (1 + 0.5 * 0) = 2.0
    assert_eq!(stroke.compute_point_width(0.5, 0.5), 2.0);
    // p=1.0 -> 2.0 * (1 + 0.5 * 0.5) = 2.5
    assert_eq!(stroke.compute_point_width(1.0, 0.5), 2.5);

    // Bounding box
    let bbox = stroke.bounding_box().unwrap();
    assert_eq!(bbox.x, 10.0);
    assert_eq!(bbox.y, 10.0);
    assert_eq!(bbox.width, 30.0);
    assert_eq!(bbox.height, 30.0);

    // Catmull-Rom to Cubic Bézier controls
    let ((cp1_x, _cp1_y), (cp2_x, _cp2_y)) = InkStroke::compute_catmull_rom_bezier_controls(
        (10.0, 10.0),
        (20.0, 15.0),
        (30.0, 25.0),
        (40.0, 40.0),
        1.0,
    );
    assert!(cp1_x > 20.0 && cp1_x < 30.0);
    assert!(cp2_x > 20.0 && cp2_x < 30.0);

    // DrawingML XML export
    let dml = stroke.to_drawingml_xml();
    assert!(dml.contains(r##"color="#0078D7""##));
    assert!(dml.contains(r#"width="2.0""#));
    assert!(dml.contains("10.0,10.0"));

    // SVG Path export
    let svg = stroke.to_svg_path();
    assert!(svg.starts_with("M 10.0 10.0"));
    assert!(svg.contains(" C "));
}

#[test]
fn test_12_stylus_ink_transactions_on_command_bus() {
    let mut bus = GlobalCommandBus::new();
    let mut state = GlobalOfficeState::new();

    let stroke = InkStroke::new("#FF0000", 3.0);
    let add_tx = Box::new(AddInkStrokeTransaction::new(0, stroke.clone()));
    assert_eq!(add_tx.description(), "Stylus: Add Ink Stroke");
    assert_eq!(add_tx.target_app(), DocumentType::PowerPoint);

    bus.execute(&mut state, add_tx).expect("Execute Add Ink");
    assert_eq!(bus.undo_stack_len(), 1);

    let clear_tx = Box::new(ClearInkTransaction::new(0, vec![stroke]));
    assert_eq!(clear_tx.description(), "Stylus: Clear All Ink");
    bus.execute(&mut state, clear_tx)
        .expect("Execute Clear Ink");
    assert_eq!(bus.undo_stack_len(), 2);

    bus.undo(&mut state).expect("Undo clear ink");
    assert_eq!(bus.undo_stack_len(), 1);

    bus.undo(&mut state).expect("Undo add ink");
    assert_eq!(bus.undo_stack_len(), 0);
}

#[test]
fn test_13_powerpoint_laser_pointer_trail_decay() {
    assert_eq!(LaserPointerEngine::CORE_RADIUS_PX, 6.0);
    assert_eq!(LaserPointerEngine::GLOW_RADIUS_PX, 18.0);

    // At t=0, alpha = alpha0
    let a0 = LaserPointerEngine::compute_trail_alpha(1.0, 0.0, 200.0);
    assert!((a0 - 1.0).abs() < 1e-5);

    // At t=tau (200ms), alpha = 1.0 / e ~= 0.3678
    let a_tau = LaserPointerEngine::compute_trail_alpha(1.0, 200.0, 200.0);
    assert!((a_tau - 0.367879).abs() < 1e-4);

    // At t=1000ms, alpha is decayed below 0.01
    let a_late = LaserPointerEngine::compute_trail_alpha(1.0, 1000.0, 200.0);
    assert!(a_late < 0.01);
}

#[test]
fn test_14_multitasking_split_view_geometry() {
    let workspace_rect = Rect2D::new(0.0, 0.0, 1920.0, 1080.0);

    // 1. Horizontal 50/50 split
    let split_h = SplitViewConfig::new("sess-1", "sess-2", SplitOrientation::Horizontal);
    let (pane1, pane2) = split_h.compute_split_rects(&workspace_rect);
    assert_eq!(pane1.x, 0.0);
    assert_eq!(pane1.width, 960.0);
    assert_eq!(pane1.height, 1080.0);

    assert_eq!(pane2.x, 960.0);
    assert_eq!(pane2.width, 960.0);
    assert_eq!(pane2.height, 1080.0);

    // 2. Vertical 70/30 split
    let mut split_v = SplitViewConfig::new("sess-1", "sess-2", SplitOrientation::Vertical);
    split_v.ratio = 0.70;
    let (pane_top, pane_bot) = split_v.compute_split_rects(&workspace_rect);
    assert_eq!(pane_top.y, 0.0);
    assert_eq!(pane_top.height, 756.0); // 1080 * 0.70

    assert_eq!(pane_bot.y, 756.0);
    assert_eq!(pane_bot.height, 324.0); // 1080 * 0.30
}

#[test]
fn test_15_sub_millisecond_adaptive_calculation_sla_benchmark() {
    let iterations = 10_000;
    let start = Instant::now();

    for i in 0..iterations {
        let w = 320.0 + ((i % 1600) as f64);
        let h = 480.0 + ((i % 800) as f64);

        let mut ctrl = AdaptiveShellController::new(w, h, PointerModality::Coarse);
        let _ff = ctrl.form_factor();
        let _ori = ctrl.orientation();
        let _ = ctrl.should_show_bottom_sheet();

        ctrl.resize(w + 10.0, h + 10.0);
    }

    let elapsed = start.elapsed();
    let avg_micros = elapsed.as_micros() as f64 / iterations as f64;
    println!(
        "Benchmark: 10,000 adaptive shell layout calculations took {:?} (average: {:.2} µs / op)",
        elapsed, avg_micros
    );

    // SLA: Calculation latency must be strictly < 1.0 ms (1,000 µs)
    assert!(
        avg_micros < 1000.0,
        "Adaptive layout calculation latency {:.2} µs exceeded 1.0 ms SLA",
        avg_micros
    );
}
