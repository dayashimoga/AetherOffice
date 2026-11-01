//! subphase_52_2_1_test.rs
//! GEC-v7.0 Stage 52 (Sub-Phase 52.2.1)
//!
//! Rust Integration Tests for:
//! - Virtualized ARIA Shadow DOM accessibility tree hierarchy and point query hit-testing
//! - Semantic HTML Shadow DOM overlay generation for screen readers
//! - CJK IME composition state management, pre-edit string buffering, and candidate box placement
//! - Unicode BiDi paragraph base direction detection and multi-run segmentation
//! - Keyboard navigation F6 / Shift+F6 global pane cycling
//! - Lossless Serde JSON roundtrip of accessibility and composition state structures

use render_core::{
    AriaBoundingBox, AriaNode, AriaRole, AriaTree, BidiAnalyzer, BidiDirection,
    ImeCompositionState, KeyboardNavigationMap, NavFocusPane,
};

#[test]
fn test_aria_node_creation_and_hierarchy() {
    let mut doc_root = AriaNode::new("doc-1", AriaRole::Document, "Main Document Viewport")
        .with_bounds(AriaBoundingBox::new(0.0, 0.0, 1024.0, 768.0));

    let heading = AriaNode::new("h-1", AriaRole::Heading, "Executive Summary")
        .with_level(1)
        .with_bounds(AriaBoundingBox::new(50.0, 50.0, 400.0, 40.0));

    let p1 = AriaNode::new(
        "p-1",
        AriaRole::Paragraph,
        "Quarterly financial results were positive.",
    )
    .with_bounds(AriaBoundingBox::new(50.0, 100.0, 600.0, 60.0));

    let mut table = AriaNode::new("tbl-1", AriaRole::Grid, "Revenue Breakdown Table")
        .with_bounds(AriaBoundingBox::new(50.0, 180.0, 500.0, 150.0));

    let mut row = AriaNode::new("row-1", AriaRole::Row, "Header Row");
    let cell1 = AriaNode::new("c-1", AriaRole::Cell, "Region")
        .with_value("Region")
        .with_selected(true);
    let cell2 = AriaNode::new("c-2", AriaRole::Cell, "Revenue").with_value("Revenue");
    row.add_child(cell1);
    row.add_child(cell2);
    table.add_child(row);

    doc_root.add_child(heading);
    doc_root.add_child(p1);
    doc_root.add_child(table);

    assert_eq!(doc_root.node_count(), 7);
    assert_eq!(doc_root.children.len(), 3);
}

#[test]
fn test_aria_tree_search_and_hit_testing() {
    let mut doc_root = AriaNode::new("doc-root", AriaRole::Document, "Editor")
        .with_bounds(AriaBoundingBox::new(0.0, 0.0, 800.0, 600.0));

    let btn = AriaNode::new("btn-submit", AriaRole::Button, "Submit Feedback")
        .with_bounds(AriaBoundingBox::new(100.0, 200.0, 120.0, 36.0));

    let shape = AriaNode::new("shape-star", AriaRole::Shape, "Callout Star")
        .with_bounds(AriaBoundingBox::new(300.0, 300.0, 80.0, 80.0));

    doc_root.add_child(btn);
    doc_root.add_child(shape);

    let tree = AriaTree::new(doc_root);
    assert_eq!(tree.total_nodes, 3);

    // Node lookup by ID
    let found_btn = tree.find_node("btn-submit");
    assert!(found_btn.is_some());
    assert_eq!(found_btn.map(|n| n.role), Some(AriaRole::Button));

    let found_missing = tree.find_node("non-existent");
    assert!(found_missing.is_none());

    // Hit-testing points
    let hit_btn = tree.find_node_at_point(110.0, 210.0);
    assert!(hit_btn.is_some());
    assert_eq!(hit_btn.map(|n| n.id.as_str()), Some("btn-submit"));

    let hit_shape = tree.find_node_at_point(350.0, 320.0);
    assert!(hit_shape.is_some());
    assert_eq!(hit_shape.map(|n| n.id.as_str()), Some("shape-star"));

    let hit_bg = tree.find_node_at_point(10.0, 10.0);
    assert!(hit_bg.is_some());
    assert_eq!(hit_bg.map(|n| n.id.as_str()), Some("doc-root"));

    let hit_outside = tree.find_node_at_point(9999.0, 9999.0);
    assert!(hit_outside.is_none());
}

#[test]
fn test_aria_shadow_dom_html_serialization() {
    let mut doc_root = AriaNode::new("doc-1", AriaRole::Document, "Document Canvas");
    let heading = AriaNode::new("h1", AriaRole::Heading, "Chapter 1").with_level(1);
    let p = AriaNode::new("p1", AriaRole::Paragraph, "Hello World Content");
    doc_root.add_child(heading);
    doc_root.add_child(p);

    let tree = AriaTree::new(doc_root);
    let html = tree.to_html_shadow_dom();

    assert!(html.contains("role=\"document\""));
    assert!(html.contains("id=\"aria-doc-1\""));
    assert!(html.contains("<h2 id=\"aria-h1\" role=\"heading\" aria-label=\"Chapter 1\" aria-level=\"1\">Chapter 1</h2>"));
    assert!(html.contains("<p id=\"aria-p1\" role=\"paragraph\" aria-label=\"Hello World Content\">Hello World Content</p>"));
}

#[test]
fn test_ime_composition_lifecycle() {
    let mut ime = ImeCompositionState::new();
    assert!(!ime.is_composing);
    assert!(ime.preedit_string.is_empty());

    // Start composition at caret (120, 240) with height 20
    ime.start(120.0, 240.0, 20.0);
    assert!(ime.is_composing);
    assert_eq!(ime.caret_x, 120.0);
    assert_eq!(ime.caret_y, 240.0);
    assert_eq!(ime.candidate_box.x, 120.0);
    assert_eq!(ime.candidate_box.y, 260.0); // caret_y + caret_h

    // Intermediate typing (e.g. Japanese "nihon" -> "にほん")
    ime.update("にほん", 3, 0, 3, 150.0, 240.0, 20.0);
    assert!(ime.is_composing);
    assert_eq!(ime.preedit_string, "にほん");
    assert_eq!(ime.cursor_position, 3);
    assert_eq!(ime.candidate_box.x, 150.0);

    // Commit committed string (e.g. "日本")
    ime.update("日本", 2, 0, 2, 160.0, 240.0, 20.0);
    let committed = ime.finish();
    assert_eq!(committed, "日本");
    assert!(!ime.is_composing);
    assert!(ime.preedit_string.is_empty());
}

#[test]
fn test_ime_composition_cancellation() {
    let mut ime = ImeCompositionState::new();
    ime.start(100.0, 100.0, 18.0);
    ime.update("test", 4, 0, 4, 130.0, 100.0, 18.0);
    assert!(ime.is_composing);

    ime.cancel();
    assert!(!ime.is_composing);
    assert!(ime.preedit_string.is_empty());
}

#[test]
fn test_bidi_base_direction_detection() {
    // English -> LTR
    assert_eq!(
        BidiAnalyzer::detect_base_direction("Hello World 2026"),
        BidiDirection::Ltr
    );

    // Arabic -> RTL
    assert_eq!(
        BidiAnalyzer::detect_base_direction("مرحبا بالعالم"),
        BidiDirection::Rtl
    );

    // Hebrew -> RTL
    assert_eq!(
        BidiAnalyzer::detect_base_direction("שלום עולם"),
        BidiDirection::Rtl
    );

    // Leading punctuation with Arabic -> RTL
    assert_eq!(
        BidiAnalyzer::detect_base_direction("(2026) مرحبا"),
        BidiDirection::Rtl
    );

    // Digits / symbols only -> Neutral
    assert_eq!(
        BidiAnalyzer::detect_base_direction("123456 !!!"),
        BidiDirection::Neutral
    );
}

#[test]
fn test_bidi_multi_run_segmentation() {
    // Mixed English and Hebrew: "Report שלום 2026"
    let runs = BidiAnalyzer::analyze_runs("Report שלום 2026");
    assert!(!runs.is_empty());

    let has_ltr = runs.iter().any(|r| r.direction == BidiDirection::Ltr);
    let has_rtl = runs.iter().any(|r| r.direction == BidiDirection::Rtl);

    assert!(has_ltr);
    assert!(has_rtl);
}

#[test]
fn test_keyboard_navigation_f6_cycling() {
    let mut nav = KeyboardNavigationMap::new();
    assert_eq!(nav.active_pane, NavFocusPane::Canvas);

    // Canvas -> StatusBar -> TitleBar -> Ribbon -> SidePane -> Canvas
    assert_eq!(nav.cycle_next_pane(), NavFocusPane::StatusBar);
    assert_eq!(nav.cycle_next_pane(), NavFocusPane::TitleBar);
    assert_eq!(nav.cycle_next_pane(), NavFocusPane::Ribbon);
    assert_eq!(nav.cycle_next_pane(), NavFocusPane::SidePane);
    assert_eq!(nav.cycle_next_pane(), NavFocusPane::Canvas);
}

#[test]
fn test_keyboard_navigation_shift_f6_reverse_cycling() {
    let mut nav = KeyboardNavigationMap::new();
    assert_eq!(nav.active_pane, NavFocusPane::Canvas);

    // Canvas -> SidePane -> Ribbon -> TitleBar -> StatusBar -> Canvas
    assert_eq!(nav.cycle_prev_pane(), NavFocusPane::SidePane);
    assert_eq!(nav.cycle_prev_pane(), NavFocusPane::Ribbon);
    assert_eq!(nav.cycle_prev_pane(), NavFocusPane::TitleBar);
    assert_eq!(nav.cycle_prev_pane(), NavFocusPane::StatusBar);
    assert_eq!(nav.cycle_prev_pane(), NavFocusPane::Canvas);
}

#[test]
fn test_accessibility_serde_roundtrip() {
    let node = AriaNode::new("grid-1", AriaRole::Grid, "Sample Spreadsheet Grid")
        .with_bounds(AriaBoundingBox::new(0.0, 0.0, 500.0, 300.0));
    let tree = AriaTree::new(node);

    let json = serde_json::to_string(&tree).expect("serialize tree");
    let deserialized: AriaTree = serde_json::from_str(&json).expect("deserialize tree");
    assert_eq!(tree, deserialized);

    let ime = ImeCompositionState::default();
    let ime_json = serde_json::to_string(&ime).expect("serialize ime");
    let ime_deserialized: ImeCompositionState =
        serde_json::from_str(&ime_json).expect("deserialize ime");
    assert_eq!(ime, ime_deserialized);

    let nav = KeyboardNavigationMap::default();
    let nav_json = serde_json::to_string(&nav).expect("serialize nav");
    let nav_deserialized: KeyboardNavigationMap =
        serde_json::from_str(&nav_json).expect("deserialize nav");
    assert_eq!(nav, nav_deserialized);
}
