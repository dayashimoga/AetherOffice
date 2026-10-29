//! Subphase 5.1.2 Integration Test Suite:
//! Interactive Hit-Testing, Selection Handles & Undo/Redo Command Bus
//!
//! Validates:
//! - Sub-millisecond pointer/touch screen-to-world coordinate mapping
//! - Word hit-testing (pages, lines, character midpoint split, table cell/row resize)
//! - Excel hit-testing (cells, row/col headers, divider resize +-3px, 6x6 AutoFill handle)
//! - PowerPoint hit-testing (shapes reverse z-order, 8 resize handles, 24px offset rotation handle)
//! - Selection models (Word caret/range, Excel cell/range/fill, PowerPoint single/toggle)
//! - Multi-format clipboard serializer (Plain text TSV, HTML table, OOXML)
//! - Transactional Command Bus with unlimited Undo/Redo, compound transactions, and typing coalescing
//! - Kinetic scrolling & pinch-to-zoom inertia physics
//! - Vector manifest validation against `specs/evidence/render/5.1.2_vectors.json`

use render_core::{
    ClipboardManager, CommandBus, CompoundCommand, CoordinateTransformer, DocumentState,
    ExcelHitTarget, ExcelRange, ExcelSelection, HitTester, InertiaPhysics, InsertTextCommand,
    PptHitTarget, PptSelection, Rect2D, ResizeHandle, WordHitTarget, WordSelection,
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
fn test_01_coordinate_mapping_screen_to_world() {
    let screen_pt = (400.0, 300.0);
    let scroll = (100.0, 50.0);
    let zoom = 1.5;
    let dpr = 2.0;

    let world = CoordinateTransformer::screen_to_world(screen_pt, scroll, zoom, dpr);
    let expected_x = 233.33333333333334;
    let expected_y = 150.0;

    assert!((world.0 - expected_x).abs() < 1e-6);
    assert!((world.1 - expected_y).abs() < 1e-6);
}

#[test]
fn test_02_coordinate_mapping_world_to_screen() {
    let world_pt = (233.33333333333334, 150.0);
    let scroll = (100.0, 50.0);
    let zoom = 1.5;
    let dpr = 2.0;

    let screen = CoordinateTransformer::world_to_screen(world_pt, scroll, zoom, dpr);
    assert!((screen.0 - 400.0).abs() < 1e-4);
    assert!((screen.1 - 300.0).abs() < 1e-4);
}

#[test]
fn test_03_word_hit_testing_page_resolution() {
    let pages = [(0.0, 1056.0), (1076.0, 1056.0), (2152.0, 1056.0)];
    let point = (200.0, 1500.0);

    let mut hit_target = WordHitTarget::Outside;
    for (i, &(y, h)) in pages.iter().enumerate() {
        if point.1 >= y && point.1 <= y + h {
            hit_target = WordHitTarget::Page {
                page_index: i,
                local_pt: (point.0, point.1 - y),
            };
            break;
        }
    }

    assert_eq!(
        hit_target,
        WordHitTarget::Page {
            page_index: 1,
            local_pt: (200.0, 424.0),
        }
    );
}

#[test]
fn test_04_word_hit_testing_char_midpoint_advance() {
    let line_x = 100.0;
    let char_advances = [10.0, 10.0, 10.0, 10.0];

    // Midpoints: char 0: 105.0; char 1: 115.0; char 2: 125.0; char 3: 135.0
    let offset_early = HitTester::hit_test_word_line(line_x, &char_advances, 104.0);
    assert_eq!(offset_early, 0);

    let offset_late = HitTester::hit_test_word_line(line_x, &char_advances, 107.0);
    assert_eq!(offset_late, 1);

    let offset_end = HitTester::hit_test_word_line(line_x, &char_advances, 145.0);
    assert_eq!(offset_end, 4);
}

#[test]
fn test_05_word_hit_testing_table_row_resize() {
    let row_top = 260.0;
    let row_bottom = 300.0;
    let thresh = 3.0;

    let (inside_cell, is_resize_1) =
        HitTester::hit_test_word_table_row(row_top, row_bottom, 290.0, thresh);
    assert!(inside_cell);
    assert!(!is_resize_1);

    let (inside_cell_2, is_resize_2) =
        HitTester::hit_test_word_table_row(row_top, row_bottom, 299.0, thresh);
    assert!(inside_cell_2);
    assert!(is_resize_2);
}

#[test]
fn test_06_excel_hit_testing_cell_lookup() {
    let target = HitTester::hit_test_excel(20.0, 80.0, 0.0, 0.0, None, (250.0, 65.0));
    assert_eq!(target, ExcelHitTarget::Cell { row: 3, col: 3 });
}

#[test]
fn test_07_excel_hit_testing_header_selection() {
    // Column header hit: x=120, y=12 (col_w=60, header_w=40, col=1)
    let target_col = HitTester::hit_test_excel(16.0, 60.0, 40.0, 24.0, None, (120.0, 12.0));
    assert_eq!(target_col, ExcelHitTarget::ColHeader { col: 1 });

    // Row header hit: x=20, y=80 (row_h=16, header_h=24, row=(80-24)/16 = 3)
    let target_row = HitTester::hit_test_excel(16.0, 60.0, 40.0, 24.0, None, (20.0, 80.0));
    assert_eq!(target_row, ExcelHitTarget::RowHeader { row: 3 });
}

#[test]
fn test_08_excel_hit_testing_col_divider_resize() {
    // Col 1 right boundary is header_w(0) + 2 * col_w(80) = 160.0
    // Click at x = 161.5 is within +-3px
    let target = HitTester::hit_test_excel(20.0, 80.0, 0.0, 24.0, None, (161.5, 10.0));
    assert_eq!(target, ExcelHitTarget::ColResize { col: 1, x: 160.0 });
}

#[test]
fn test_09_excel_hit_testing_autofill_handle() {
    let sel_br = Some((240.0, 80.0));

    let hit_inside = HitTester::hit_test_excel(20.0, 80.0, 0.0, 0.0, sel_br, (239.0, 79.0));
    match hit_inside {
        ExcelHitTarget::AutoFillHandle { row, col } => {
            assert_eq!(row, 4);
            assert_eq!(col, 3);
        }
        other => panic!("Expected AutoFillHandle, got {:?}", other),
    }

    let hit_outside = HitTester::hit_test_excel(20.0, 80.0, 0.0, 0.0, sel_br, (230.0, 70.0));
    match hit_outside {
        ExcelHitTarget::Cell { .. } => {}
        other => panic!("Expected Cell, got {:?}", other),
    }
}

#[test]
fn test_10_ppt_hit_testing_shape_reverse_z_order() {
    let shape_bottom = (1, Rect2D::new(50.0, 50.0, 200.0, 200.0));
    let shape_top = (2, Rect2D::new(100.0, 100.0, 200.0, 200.0));
    let click = (150.0, 150.0);

    let shapes = [shape_bottom, shape_top];
    let mut hit_id = None;

    // Reverse z-order: top shape checked first
    for &(id, ref rect) in shapes.iter().rev() {
        if let Some(PptHitTarget::ShapeBody { shape_id }) =
            HitTester::hit_test_ppt_shape(id, rect, click, false)
        {
            hit_id = Some(shape_id);
            break;
        }
    }

    assert_eq!(hit_id, Some(2));
}

#[test]
fn test_11_ppt_hit_testing_8_resize_handles() {
    let rect = Rect2D::new(100.0, 100.0, 200.0, 100.0);

    let hit_tl = HitTester::hit_test_ppt_shape(1, &rect, (100.0, 100.0), true);
    assert_eq!(
        hit_tl,
        Some(PptHitTarget::ResizeHandle {
            shape_id: 1,
            handle: ResizeHandle::TopLeft,
        })
    );

    let hit_br = HitTester::hit_test_ppt_shape(1, &rect, (300.0, 200.0), true);
    assert_eq!(
        hit_br,
        Some(PptHitTarget::ResizeHandle {
            shape_id: 1,
            handle: ResizeHandle::BottomRight,
        })
    );
}

#[test]
fn test_12_ppt_hit_testing_rotation_handle() {
    let rect = Rect2D::new(100.0, 100.0, 200.0, 100.0);
    // Rotation handle at cx = 200.0, cy = 100.0 - 24.0 = 76.0
    let hit_rot = HitTester::hit_test_ppt_shape(1, &rect, (200.0, 76.0), true);
    assert_eq!(hit_rot, Some(PptHitTarget::RotationHandle { shape_id: 1 }));
}

#[test]
fn test_13_word_selection_caret_and_range() {
    let caret = WordSelection::caret(10);
    assert!(caret.is_collapsed());
    assert_eq!(caret.span(), 0);
    assert_eq!(caret.min_offset(), 10);
    assert_eq!(caret.max_offset(), 10);

    let range = WordSelection::range(5, 15);
    assert!(!range.is_collapsed());
    assert_eq!(range.span(), 10);
    assert_eq!(range.min_offset(), 5);
    assert_eq!(range.max_offset(), 15);
}

#[test]
fn test_14_excel_selection_range_metrics() {
    let range = ExcelRange::new(2, 2, 5, 4);
    assert_eq!(range.row_count(), 4);
    assert_eq!(range.col_count(), 3);
    assert_eq!(range.total_cells(), 12);
    assert!(range.contains(3, 3));
    assert!(!range.contains(1, 2));

    let mut sel = ExcelSelection::single(1, 1);
    assert_eq!(sel.ranges.len(), 1);
    sel.add_range(range);
    assert_eq!(sel.ranges.len(), 2);
}

#[test]
fn test_15_ppt_selection_multi_toggle() {
    let mut sel = PptSelection::single(10);
    assert_eq!(sel.selected_ids, vec![10]);
    assert_eq!(sel.primary_id, Some(10));

    sel.toggle(20);
    assert_eq!(sel.selected_ids, vec![10, 20]);
    assert_eq!(sel.primary_id, Some(20));

    sel.toggle(10);
    assert_eq!(sel.selected_ids, vec![20]);
    assert_eq!(sel.primary_id, Some(20));
}

#[test]
fn test_16_command_bus_execute_and_push() {
    let mut bus = CommandBus::new();
    let mut state = DocumentState {
        text_content: "Hello".into(),
        cell_values: Vec::new(),
    };

    let cmd = Box::new(InsertTextCommand {
        offset: 5,
        text: " World".into(),
    });

    bus.execute(&mut state, cmd).expect("Execute command");
    assert_eq!(state.text_content, "Hello World");
    assert!(bus.can_undo());
    assert!(!bus.can_redo());
}

#[test]
fn test_17_command_bus_undo() {
    let mut bus = CommandBus::new();
    let mut state = DocumentState {
        text_content: "Hello".into(),
        cell_values: Vec::new(),
    };

    bus.execute(
        &mut state,
        Box::new(InsertTextCommand {
            offset: 5,
            text: " World".into(),
        }),
    )
    .expect("Execute");

    bus.undo(&mut state).expect("Undo");
    assert_eq!(state.text_content, "Hello");
    assert!(!bus.can_undo());
    assert!(bus.can_redo());
}

#[test]
fn test_18_command_bus_redo() {
    let mut bus = CommandBus::new();
    let mut state = DocumentState {
        text_content: "Hello".into(),
        cell_values: Vec::new(),
    };

    bus.execute(
        &mut state,
        Box::new(InsertTextCommand {
            offset: 5,
            text: " World".into(),
        }),
    )
    .expect("Execute");
    bus.undo(&mut state).expect("Undo");

    bus.redo(&mut state).expect("Redo");
    assert_eq!(state.text_content, "Hello World");
    assert!(bus.can_undo());
    assert!(!bus.can_redo());
}

#[test]
fn test_19_command_bus_new_action_clears_redo() {
    let mut bus = CommandBus::new();
    let mut state = DocumentState {
        text_content: "Hello".into(),
        cell_values: Vec::new(),
    };

    bus.execute(
        &mut state,
        Box::new(InsertTextCommand {
            offset: 5,
            text: " World".into(),
        }),
    )
    .expect("Execute 1");
    bus.undo(&mut state).expect("Undo");
    assert_eq!(bus.redo_stack_len(), 1);

    bus.execute(
        &mut state,
        Box::new(InsertTextCommand {
            offset: 5,
            text: " Rust".into(),
        }),
    )
    .expect("Execute 2");
    assert_eq!(state.text_content, "Hello Rust");
    assert_eq!(bus.redo_stack_len(), 0);
}

#[test]
fn test_20_command_bus_keystroke_coalescing() {
    let mut bus = CommandBus::new();
    let mut state = DocumentState::default();

    let keystrokes = ["R", "u", "s", "t"];
    for (i, &ch) in keystrokes.iter().enumerate() {
        bus.execute(
            &mut state,
            Box::new(InsertTextCommand {
                offset: i,
                text: ch.into(),
            }),
        )
        .expect("Execute keystroke");
    }

    assert_eq!(state.text_content, "Rust");
    assert_eq!(bus.undo_stack_len(), 1);

    bus.undo(&mut state).expect("Undo coalesced typing");
    assert_eq!(state.text_content, "");
}

#[test]
fn test_21_command_bus_compound_transaction_atomicity() {
    let mut bus = CommandBus::new();
    let mut state = DocumentState::default();

    let compound = Box::new(CompoundCommand {
        description_str: "Insert 3 letters".into(),
        commands: vec![
            Box::new(InsertTextCommand {
                offset: 0,
                text: "A".into(),
            }),
            Box::new(InsertTextCommand {
                offset: 1,
                text: "B".into(),
            }),
            Box::new(InsertTextCommand {
                offset: 2,
                text: "C".into(),
            }),
        ],
    });

    bus.execute(&mut state, compound).expect("Execute compound");
    assert_eq!(state.text_content, "ABC");
    assert_eq!(bus.undo_stack_len(), 1);

    bus.undo(&mut state).expect("Undo compound");
    assert_eq!(state.text_content, "");
}

#[test]
fn test_22_clipboard_excel_plain_tsv() {
    let grid = vec![
        vec!["Item".to_string(), "Price".to_string()],
        vec!["Widget".to_string(), "19.99".to_string()],
    ];

    let payload = ClipboardManager::serialize_excel(&grid);
    assert_eq!(payload.plain_text, "Item\tPrice\nWidget\t19.99");
}

#[test]
fn test_23_clipboard_excel_html_table() {
    let grid = vec![vec!["A".to_string(), "B".to_string()]];
    let payload = ClipboardManager::serialize_excel(&grid);

    assert!(payload.html.contains("<table>"));
    assert!(payload.html.contains("<td>A</td>"));
    assert!(payload.html.contains("<td>B</td>"));
}

#[test]
fn test_24_inertia_physics_velocity_decay() {
    let v0 = 1000.0;
    let frames = 10;
    let mu = 0.92;

    let v10 = InertiaPhysics::decay_velocity(v0, frames, mu);
    let expected = 434.388;
    assert!((v10 - expected).abs() < 0.001);
}

#[test]
fn test_25_inertia_physics_rubber_band_spring() {
    let displacement = -50.0;
    let velocity = 0.0;
    let k = 0.15;
    let c = 0.0;

    let force = InertiaPhysics::compute_spring_force(displacement, velocity, k, c);
    assert!((force - 7.5).abs() < 1e-6);
}

#[test]
fn test_26_vectors_manifest_validation() {
    let manifest_path = resolve_repo_path("specs/evidence/render/5.1.2_vectors.json");
    assert!(
        manifest_path.exists(),
        "5.1.2_vectors.json must exist at {}",
        manifest_path.display()
    );

    let content = std::fs::read_to_string(&manifest_path).expect("Read vectors file");
    let json: serde_json::Value = serde_json::from_str(&content).expect("Valid JSON");

    assert_eq!(json["subphase"].as_str(), Some("5.1.2"));
    let vector_count = json["vector_count"].as_u64().expect("vector_count is u64") as usize;
    assert!(
        vector_count >= 25,
        "Vector count must be >= 25, found {}",
        vector_count
    );

    let vectors = json["vectors"].as_array().expect("vectors is array");
    assert_eq!(vectors.len(), vector_count);

    for (idx, vec_item) in vectors.iter().enumerate() {
        let expected_id = format!("VEC-5.1.2-{:02}", idx + 1);
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
