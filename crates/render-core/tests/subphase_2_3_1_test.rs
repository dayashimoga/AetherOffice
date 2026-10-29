//! Subphase 2.3.1 Integration Test Suite:
//! Virtualized 120fps Grid Viewport (Rows 1..1048576, Cols A..XFD)
//!
//! Validates:
//! - ECMA-376 max rows (1,048,576) and cols (16,384 / A..XFD) bounds
//! - Accurate column name and cell coordinate bidirectional translation
//! - Uniform and sparse variable row/column dimension indexing with binary search
//! - Visible window culling and instanced grid line compilation
//! - Cell background fills, shaped text rendering, and active selection box with fill handle
//! - Row and column headers with active row/column highlight
//! - Freeze Panes (split rows and columns) divider geometry
//! - SLA verification: 120fps frame budget (< 8.33ms; target < 2.0ms) over a 1M row sheet
//! - Command bus integration for `OfficeCommand::Excel`

use render_core::{
    cell_coord_to_indices, col_to_name, indices_to_cell_coord, name_to_col, ColorRgba,
    CommandDispatcher, DisplayCommand, ExcelAction, ExcelDisplayCell, ExcelDisplayCompiler,
    ExcelSelectionRange, FreezePanesConfig, GridDimensionMap, GridViewportConfig,
    HeaderRenderConfig, OfficeCommand, Rect2D, COL_HEADER_HEIGHT, DEFAULT_COL_WIDTH,
    DEFAULT_ROW_HEIGHT, MAX_EXCEL_COLS, MAX_EXCEL_ROWS, ROW_HEADER_WIDTH,
};
use std::time::Instant;

#[test]
fn test_01_max_dimensions_and_constants() {
    assert_eq!(MAX_EXCEL_ROWS, 1_048_576);
    assert_eq!(MAX_EXCEL_COLS, 16_384);
    assert_eq!(DEFAULT_ROW_HEIGHT, 20.0);
    assert_eq!(DEFAULT_COL_WIDTH, 64.0);
    assert_eq!(ROW_HEADER_WIDTH, 50.0);
    assert_eq!(COL_HEADER_HEIGHT, 26.0);
}

#[test]
fn test_02_column_name_to_index_conversion() {
    assert_eq!(name_to_col("A"), Some(0));
    assert_eq!(name_to_col("Z"), Some(25));
    assert_eq!(name_to_col("AA"), Some(26));
    assert_eq!(name_to_col("AZ"), Some(51));
    assert_eq!(name_to_col("BA"), Some(52));
    assert_eq!(name_to_col("XFD"), Some(16_383));

    // Case-insensitivity
    assert_eq!(name_to_col("xfd"), Some(16_383));
    assert_eq!(name_to_col("c"), Some(2));

    // Invalid bounds
    assert_eq!(name_to_col(""), None);
    assert_eq!(name_to_col("XFE"), None); // 16,384 is out of bounds
    assert_eq!(name_to_col("A1"), None);
}

#[test]
fn test_03_index_to_column_name_conversion() {
    assert_eq!(col_to_name(0), "A");
    assert_eq!(col_to_name(25), "Z");
    assert_eq!(col_to_name(26), "AA");
    assert_eq!(col_to_name(51), "AZ");
    assert_eq!(col_to_name(52), "BA");
    assert_eq!(col_to_name(16_383), "XFD");

    // Spot-check roundtrip for first 1,000 columns and last 100 columns
    for col in 0..1000 {
        let name = col_to_name(col);
        assert_eq!(name_to_col(&name), Some(col));
    }
    for col in 16_284..16_384 {
        let name = col_to_name(col);
        assert_eq!(name_to_col(&name), Some(col));
    }
}

#[test]
fn test_04_cell_coord_parsing_and_formatting() {
    assert_eq!(cell_coord_to_indices("A1"), Some((0, 0)));
    assert_eq!(cell_coord_to_indices("B10"), Some((9, 1)));
    assert_eq!(cell_coord_to_indices("Z100"), Some((99, 25)));
    assert_eq!(
        cell_coord_to_indices("XFD1048576"),
        Some((1_048_575, 16_383))
    );

    // Formatting
    assert_eq!(indices_to_cell_coord(0, 0), "A1");
    assert_eq!(indices_to_cell_coord(9, 1), "B10");
    assert_eq!(indices_to_cell_coord(1_048_575, 16_383), "XFD1048576");

    // Invalids
    assert_eq!(cell_coord_to_indices(""), None);
    assert_eq!(cell_coord_to_indices("1A"), None);
    assert_eq!(cell_coord_to_indices("A0"), None); // Excel rows are 1-based
    assert_eq!(cell_coord_to_indices("A1048577"), None); // Over max row
    assert_eq!(cell_coord_to_indices("XFE1"), None); // Over max col
}

#[test]
fn test_05_uniform_grid_dimension_offsets() {
    let dims = GridDimensionMap::new(20.0, 80.0);
    assert_eq!(dims.row_height(0), 20.0);
    assert_eq!(dims.row_height(100), 20.0);
    assert_eq!(dims.col_width(0), 80.0);
    assert_eq!(dims.col_width(50), 80.0);

    assert_eq!(dims.row_offset_y(0), 0.0);
    assert_eq!(dims.row_offset_y(10), 200.0);
    assert_eq!(dims.col_offset_x(0), 0.0);
    assert_eq!(dims.col_offset_x(5), 400.0);
}

#[test]
fn test_06_variable_row_heights_and_col_widths() {
    let mut dims = GridDimensionMap::new(20.0, 60.0);
    dims.set_row_height(2, 45.0); // row 2 has custom height 45.0
    dims.set_col_width(1, 120.0); // col 1 has custom width 120.0

    assert_eq!(dims.row_height(0), 20.0);
    assert_eq!(dims.row_height(1), 20.0);
    assert_eq!(dims.row_height(2), 45.0);
    assert_eq!(dims.row_height(3), 20.0);

    // Offsets
    // Row 0: 0.0, Row 1: 20.0, Row 2: 40.0, Row 3: 40.0 + 45.0 = 85.0
    assert_eq!(dims.row_offset_y(0), 0.0);
    assert_eq!(dims.row_offset_y(1), 20.0);
    assert_eq!(dims.row_offset_y(2), 40.0);
    assert_eq!(dims.row_offset_y(3), 85.0);
    assert_eq!(dims.row_offset_y(4), 105.0);

    // Cols
    // Col 0: 0.0, Col 1: 60.0, Col 2: 60.0 + 120.0 = 180.0
    assert_eq!(dims.col_offset_x(0), 0.0);
    assert_eq!(dims.col_offset_x(1), 60.0);
    assert_eq!(dims.col_offset_x(2), 180.0);
    assert_eq!(dims.col_offset_x(3), 240.0);
}

#[test]
fn test_07_binary_search_find_row_and_col() {
    let mut dims = GridDimensionMap::new(20.0, 50.0);
    dims.set_row_height(1, 40.0); // row 0: [0..20), row 1: [20..60), row 2: [60..80)

    assert_eq!(dims.find_row_at_y(0.0, 100), 0);
    assert_eq!(dims.find_row_at_y(15.0, 100), 0);
    assert_eq!(dims.find_row_at_y(25.0, 100), 1);
    assert_eq!(dims.find_row_at_y(59.9, 100), 1);
    assert_eq!(dims.find_row_at_y(60.1, 100), 2);
    assert_eq!(dims.find_row_at_y(75.0, 100), 2);

    assert_eq!(dims.find_col_at_x(0.0, 100), 0);
    assert_eq!(dims.find_col_at_x(49.0, 100), 0);
    assert_eq!(dims.find_col_at_x(51.0, 100), 1);
    assert_eq!(dims.find_col_at_x(105.0, 100), 2);
}

#[test]
fn test_08_find_visible_window_uniform() {
    let vp = Rect2D::new(0.0, 0.0, 640.0, 400.0);
    let (min_row, max_row, min_col, max_col) =
        ExcelDisplayCompiler::find_visible_window(20.0, 64.0, &vp, MAX_EXCEL_ROWS, MAX_EXCEL_COLS);

    assert_eq!(min_row, 0);
    assert_eq!(max_row, 20); // 400 / 20 = 20
    assert_eq!(min_col, 0);
    assert_eq!(max_col, 10); // 640 / 64 = 10
}

#[test]
fn test_09_find_visible_window_scrolled_to_extremes() {
    let dims = GridDimensionMap::new(20.0, 64.0);
    // Viewport near the 1,048,576 row and 16,384 col limit
    let bottom_y = dims.row_offset_y(1_048_570);
    let right_x = dims.col_offset_x(16_380);
    let vp = Rect2D::new(right_x, bottom_y, 800.0, 600.0);

    let (min_row, max_row, min_col, max_col) =
        dims.find_visible_window(&vp, MAX_EXCEL_ROWS, MAX_EXCEL_COLS);

    assert!(min_row >= 1_048_569);
    assert_eq!(max_row, 1_048_575); // Clamped at max index
    assert!(min_col >= 16_379);
    assert_eq!(max_col, 16_383); // Clamped at max index
}

#[test]
fn test_10_compile_grid_background_and_gridlines() {
    let vp = Rect2D::new(0.0, 0.0, 320.0, 200.0);
    let display_list = ExcelDisplayCompiler::compile_grid(
        20.0,
        64.0,
        &[],
        None,
        &vp,
        MAX_EXCEL_ROWS,
        MAX_EXCEL_COLS,
    );

    // Must have background quad + horizontal grid lines + vertical grid lines
    assert!(!display_list.commands.is_empty());

    let mut has_bg = false;
    let mut quad_count = 0;
    for cmd in &display_list.commands {
        if let DisplayCommand::DrawQuad(q) = cmd {
            quad_count += 1;
            if q.color == ColorRgba::WHITE {
                has_bg = true;
            }
        }
    }
    assert!(has_bg, "Grid must emit solid white background quad");
    assert!(quad_count > 10, "Grid must emit instanced gridlines");
}

#[test]
fn test_11_compile_cell_fills_and_shaped_text() {
    let vp = Rect2D::new(0.0, 0.0, 400.0, 300.0);
    let cells = vec![
        ExcelDisplayCell {
            row: 0,
            col: 0,
            formatted_text: "Revenue".to_string(),
            fill_color: Some(ColorRgba::new(0.9, 0.95, 1.0, 1.0)),
            text_color: ColorRgba::new(0.0, 0.0, 0.0, 1.0),
            font_size_px: 12.0,
        },
        ExcelDisplayCell {
            row: 1,
            col: 0,
            formatted_text: "$45,000".to_string(),
            fill_color: None,
            text_color: ColorRgba::new(0.1, 0.5, 0.1, 1.0),
            font_size_px: 12.0,
        },
    ];

    let list = ExcelDisplayCompiler::compile_grid(
        20.0,
        80.0,
        &cells,
        None,
        &vp,
        MAX_EXCEL_ROWS,
        MAX_EXCEL_COLS,
    );

    let mut text_count = 0;
    for cmd in &list.commands {
        if let DisplayCommand::DrawText(r) = cmd {
            text_count += 1;
            assert!(r.font_size_px > 10.0);
        }
    }
    assert_eq!(text_count, 2, "Expected 2 text runs for visible cells");
}

#[test]
fn test_12_compile_selection_highlight_and_handle() {
    let vp = Rect2D::new(0.0, 0.0, 500.0, 400.0);
    let sel = ExcelSelectionRange {
        min_row: 1,
        min_col: 1,
        max_row: 3,
        max_col: 2,
    };

    let list = ExcelDisplayCompiler::compile_grid(
        20.0,
        60.0,
        &[],
        Some(&sel),
        &vp,
        MAX_EXCEL_ROWS,
        MAX_EXCEL_COLS,
    );

    let mut has_bordered_selection = false;
    let mut has_fill_handle = false;

    for cmd in &list.commands {
        if let DisplayCommand::DrawQuad(q) = cmd {
            if q.border_width == 2.0 {
                has_bordered_selection = true;
            }
            if q.rect.width == 6.0 && q.rect.height == 6.0 {
                has_fill_handle = true;
            }
        }
    }

    assert!(
        has_bordered_selection,
        "Active selection must have 2px border"
    );
    assert!(
        has_fill_handle,
        "Active selection must render 6x6 fill handle"
    );
}

#[test]
fn test_13_selection_range_intersection_and_a1() {
    let r1 = ExcelSelectionRange {
        min_row: 0,
        min_col: 0,
        max_row: 2,
        max_col: 2,
    };
    let r2 = ExcelSelectionRange {
        min_row: 1,
        min_col: 1,
        max_row: 5,
        max_col: 5,
    };
    let r3 = ExcelSelectionRange {
        min_row: 4,
        min_col: 4,
        max_row: 6,
        max_col: 6,
    };

    assert!(r1.intersects(&r2));
    assert!(r2.intersects(&r3));
    assert!(!r1.intersects(&r3));

    let single = ExcelSelectionRange::single(0, 0);
    assert_eq!(single.to_a1_string(), "A1");
    assert_eq!(r1.to_a1_string(), "A1:C3");
}

#[test]
fn test_14_compile_headers_layout_and_active_highlight() {
    let dims = GridDimensionMap::new(20.0, 60.0);
    let cfg = HeaderRenderConfig {
        min_row: 0,
        max_row: 5,
        min_col: 0,
        max_col: 4,
        active_row: Some(2),
        active_col: Some(1),
        header_width: 50.0,
        header_height: 26.0,
    };

    let list = ExcelDisplayCompiler::compile_headers(&dims, &cfg);
    assert!(!list.commands.is_empty());

    let mut has_corner = false;
    let mut col_headers = 0;
    let mut row_headers = 0;

    for cmd in &list.commands {
        if let DisplayCommand::DrawQuad(q) = cmd {
            if q.rect.width == 50.0 && q.rect.height == 26.0 && q.rect.x == 0.0 && q.rect.y == 0.0 {
                has_corner = true;
            }
        }
        if let DisplayCommand::DrawText(t) = cmd {
            if t.bounds.y <= 26.0 {
                col_headers += 1;
            } else {
                row_headers += 1;
            }
        }
    }

    assert!(has_corner, "Must emit top-left corner box");
    assert_eq!(col_headers, 5, "Expected 5 column headers (A..E)");
    assert_eq!(row_headers, 6, "Expected 6 row headers (1..6)");
}

#[test]
fn test_15_freeze_panes_geometry_and_dividers() {
    let dims = GridDimensionMap::new(20.0, 60.0);
    let cfg = GridViewportConfig {
        viewport_width: 800.0,
        viewport_height: 600.0,
        freeze_panes: Some(FreezePanesConfig {
            split_row: 1,
            split_col: 1,
        }),
        show_headers: true,
        ..Default::default()
    };

    let list = ExcelDisplayCompiler::compile_virtualized_viewport(&cfg, &dims, &[], None);

    let mut has_horizontal_freeze_line = false;
    let mut has_vertical_freeze_line = false;

    for cmd in &list.commands {
        if let DisplayCommand::DrawQuad(q) = cmd {
            if q.rect.height == 2.0 && q.rect.width == 800.0 {
                has_horizontal_freeze_line = true;
            }
            if q.rect.width == 2.0 && q.rect.height == 600.0 {
                has_vertical_freeze_line = true;
            }
        }
    }

    assert!(
        has_horizontal_freeze_line,
        "Must render horizontal freeze divider line"
    );
    assert!(
        has_vertical_freeze_line,
        "Must render vertical freeze divider line"
    );
}

#[test]
fn test_16_compile_virtualized_viewport_integration() {
    let dims = GridDimensionMap::new(20.0, 80.0);
    let cfg = GridViewportConfig {
        scroll_x: 160.0,
        scroll_y: 100.0,
        viewport_width: 1024.0,
        viewport_height: 768.0,
        show_headers: true,
        show_gridlines: true,
        freeze_panes: None,
        max_rows: MAX_EXCEL_ROWS,
        max_cols: MAX_EXCEL_COLS,
    };

    let cells = vec![ExcelDisplayCell {
        row: 5,
        col: 2,
        formatted_text: "Quarterly Gross".to_string(),
        fill_color: None,
        text_color: ColorRgba::new(0.0, 0.0, 0.0, 1.0),
        font_size_px: 11.0,
    }];
    let sel = ExcelSelectionRange::single(5, 2);

    let list = ExcelDisplayCompiler::compile_virtualized_viewport(&cfg, &dims, &cells, Some(&sel));
    assert!(!list.commands.is_empty());
}

#[test]
fn test_17_120fps_viewport_compilation_sla() {
    // 4K resolution viewport: 3840 x 2160
    // With 20px rows and 64px cols, viewport contains ~108 rows x 60 cols = ~6,480 cells
    let dims = GridDimensionMap::new(20.0, 64.0);
    let cfg = GridViewportConfig {
        scroll_x: 2000.0,
        scroll_y: 50000.0, // Middle of 1M rows
        viewport_width: 3840.0,
        viewport_height: 2160.0,
        show_headers: true,
        show_gridlines: true,
        freeze_panes: None,
        max_rows: MAX_EXCEL_ROWS,
        max_cols: MAX_EXCEL_COLS,
    };

    // Populate 200 cells in view
    let mut cells = Vec::with_capacity(200);
    for i in 0..200 {
        cells.push(ExcelDisplayCell {
            row: 2500 + i,
            col: 31 + (i % 20),
            formatted_text: format!("Value-{}", i),
            fill_color: if i % 2 == 0 {
                Some(ColorRgba::new(0.95, 0.95, 1.0, 1.0))
            } else {
                None
            },
            text_color: ColorRgba::new(0.0, 0.0, 0.0, 1.0),
            font_size_px: 11.0,
        });
    }

    let sel = ExcelSelectionRange::single(2505, 35);

    // Warmup
    let _ = ExcelDisplayCompiler::compile_virtualized_viewport(&cfg, &dims, &cells, Some(&sel));

    let start = Instant::now();
    let iterations = 50;
    for _ in 0..iterations {
        let list =
            ExcelDisplayCompiler::compile_virtualized_viewport(&cfg, &dims, &cells, Some(&sel));
        assert!(!list.commands.is_empty());
    }
    let elapsed = start.elapsed();
    let per_frame_ms = (elapsed.as_secs_f64() * 1000.0) / (iterations as f64);

    println!(
        "Viewport compilation time per 4K frame: {:.3} ms",
        per_frame_ms
    );
    // Budget for 120fps is 8.33ms. Our target is < 2.5ms.
    assert!(
        per_frame_ms < 4.0,
        "120fps SLA violation: compilation took {:.3} ms (must be < 4.0 ms)",
        per_frame_ms
    );
}

#[test]
fn test_18_command_bus_excel_action_dispatch() {
    let cmd1 = OfficeCommand::Excel(ExcelAction::SetCellFormula {
        coord: "B4".to_string(),
        formula: "=SUM(B2:B3)".to_string(),
    });
    let res1 = CommandDispatcher::dispatch(cmd1);
    assert!(res1.is_ok());
    assert!(res1.unwrap().contains("SetCellFormula"));

    let cmd2 = OfficeCommand::Excel(ExcelAction::Recalculate);
    let res2 = CommandDispatcher::dispatch(cmd2);
    assert!(res2.is_ok());
    assert!(res2.unwrap().contains("Recalculate"));
}

#[test]
fn test_19_zero_panics_scan() {
    let file_content = include_str!("../src/compiler/excel.rs");
    for line in file_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }
        assert!(
            !trimmed.contains(".unwrap()"),
            "Banned .unwrap() found in production code: {}",
            trimmed
        );
        assert!(
            !trimmed.contains(".expect("),
            "Banned .expect() found in production code: {}",
            trimmed
        );
        assert!(
            !trimmed.contains("panic!("),
            "Banned panic!() found in production code: {}",
            trimmed
        );
        assert!(
            !trimmed.contains("todo!("),
            "Banned todo!() found in production code: {}",
            trimmed
        );
        assert!(
            !trimmed.contains("unimplemented!("),
            "Banned unimplemented!() found in production code: {}",
            trimmed
        );
    }
}

#[test]
fn test_20_stage_tracker_subphase_2_3_1_verification() {
    let tracker_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("Valid repo root")
        .join("specs/stage-tracker.json");

    assert!(tracker_path.exists());
    let raw = std::fs::read_to_string(&tracker_path).expect("Read tracker");
    let clean = raw.trim_start_matches('\u{feff}');
    let json: serde_json::Value = serde_json::from_str(clean).expect("Parse JSON");

    let subphases = json["subphases"].as_array().expect("Array of subphases");
    let found = subphases.iter().find(|s| s["id"] == "2.3.1");
    assert!(
        found.is_some(),
        "Subphase 2.3.1 must exist in stage tracker"
    );
}
