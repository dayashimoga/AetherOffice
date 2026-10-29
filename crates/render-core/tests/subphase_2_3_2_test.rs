//! Subphase 2.3.2 Integration Test Suite:
//! Interactive Formula Bar, Multi-Range Cell Selection & Auto-Fill
//!
//! Validates:
//! - Interactive Formula Bar Name Box parsing (single cells, ranges, inverted ranges)
//! - Live formula syntax reference extraction with distinct visual highlight colors
//! - Multi-range cell selection model (single, extend, add disjoint, row, col, select-all)
//! - Auto-fill series prediction (numeric linear, date/alphanumeric, formula relative translation)
//! - Fixed `$` coordinate preservation in auto-fill formula adjustment
//! - Visual display compilation for multi-range selections, auto-fill preview, and formula references
//! - 120fps compilation SLA check (< 2.0 ms for full interactive viewport)
//! - Command Bus dispatching for formula commits and recalculation

use render_core::{
    AutoFillEngine, ColorRgba, CommandDispatcher, DisplayCommand, ExcelAction,
    ExcelDisplayCompiler, ExcelSelectionRange, FormulaBarEngine, GridDimensionMap,
    MultiRangeSelectionModel, OfficeCommand,
};
use std::time::Instant;

#[test]
fn test_01_name_box_parsing_single_and_ranges() {
    let single = FormulaBarEngine::parse_name_box("B4");
    assert_eq!(single, Some(ExcelSelectionRange::single(3, 1)));

    let range = FormulaBarEngine::parse_name_box("A1:C10");
    assert_eq!(
        range,
        Some(ExcelSelectionRange {
            min_row: 0,
            min_col: 0,
            max_row: 9,
            max_col: 2,
        })
    );

    // Inverted range order should be normalized
    let inverted = FormulaBarEngine::parse_name_box("C10:A1");
    assert_eq!(
        inverted,
        Some(ExcelSelectionRange {
            min_row: 0,
            min_col: 0,
            max_row: 9,
            max_col: 2,
        })
    );

    // Invalids
    assert_eq!(FormulaBarEngine::parse_name_box(""), None);
    assert_eq!(FormulaBarEngine::parse_name_box("Invalid"), None);
}

#[test]
fn test_02_formula_colored_reference_extraction() {
    let formula = "=SUM(A1:B10) + C5 * D1:D2";
    let refs = FormulaBarEngine::extract_colored_references(formula);

    assert_eq!(refs.len(), 3);
    assert_eq!(
        refs[0].0,
        ExcelSelectionRange {
            min_row: 0,
            min_col: 0,
            max_row: 9,
            max_col: 1,
        }
    );
    assert_eq!(refs[1].0, ExcelSelectionRange::single(4, 2)); // C5
    assert_eq!(
        refs[2].0,
        ExcelSelectionRange {
            min_row: 0,
            min_col: 3,
            max_row: 1,
            max_col: 3,
        }
    ); // D1:D2

    // All 3 colors must be distinct
    assert_ne!(refs[0].1, refs[1].1);
    assert_ne!(refs[1].1, refs[2].1);
    assert_ne!(refs[0].1, refs[2].1);
}

#[test]
fn test_03_multirange_selection_model_initialization() {
    let model = MultiRangeSelectionModel::new(2, 3);
    assert_eq!(model.active_cell, (2, 3));
    assert_eq!(model.anchor_cell, (2, 3));
    assert_eq!(model.ranges.len(), 1);
    assert!(model.is_selected(2, 3));
    assert!(!model.is_selected(0, 0));
    assert_eq!(model.total_cell_count(), 1);
    assert_eq!(model.to_range_string(), "D3");
}

#[test]
fn test_04_multirange_selection_extend() {
    let mut model = MultiRangeSelectionModel::new(2, 3);
    model.extend_selection(5, 6);

    assert_eq!(model.ranges.len(), 1);
    assert_eq!(model.active_cell, (5, 6));
    assert_eq!(model.anchor_cell, (2, 3)); // Anchor stays fixed
    assert_eq!(model.total_cell_count(), 16); // 4 rows x 4 cols
    assert!(model.is_selected(3, 4));
    assert!(model.is_selected(5, 6));
    assert_eq!(model.to_range_string(), "D3:G6");
}

#[test]
fn test_05_multirange_selection_disjoint_ranges() {
    let mut model = MultiRangeSelectionModel::new(0, 0);
    model.extend_selection(1, 1); // A1:B2

    // Add disjoint range D4:E5
    model.add_range(ExcelSelectionRange {
        min_row: 3,
        min_col: 3,
        max_row: 4,
        max_col: 4,
    });

    assert_eq!(model.ranges.len(), 2);
    assert!(model.is_selected(0, 0));
    assert!(model.is_selected(1, 1));
    assert!(model.is_selected(3, 3));
    assert!(model.is_selected(4, 4));
    assert!(!model.is_selected(2, 2));

    assert_eq!(model.total_cell_count(), 8);
    assert_eq!(model.to_range_string(), "A1:B2, D4:E5");

    let bbox = model.bounding_box();
    assert_eq!(
        bbox,
        Some(ExcelSelectionRange {
            min_row: 0,
            min_col: 0,
            max_row: 4,
            max_col: 4,
        })
    );
}

#[test]
fn test_06_multirange_selection_row_col_all() {
    let mut model = MultiRangeSelectionModel::new(0, 0);

    // Select row 5 across 100 columns
    model.select_row(5, 100);
    assert_eq!(model.total_cell_count(), 100);
    assert!(model.is_selected(5, 0));
    assert!(model.is_selected(5, 99));
    assert!(!model.is_selected(4, 0));

    // Select col 2 across 50 rows
    model.select_col(2, 50);
    assert_eq!(model.total_cell_count(), 50);
    assert!(model.is_selected(0, 2));
    assert!(model.is_selected(49, 2));
    assert!(!model.is_selected(0, 1));

    // Select all 1000 x 50
    model.select_all(1000, 50);
    assert_eq!(model.total_cell_count(), 50_000);
}

#[test]
fn test_07_autofill_translate_relative_references() {
    let f1 = "=A1+B1";
    assert_eq!(AutoFillEngine::translate_formula(f1, 1, 0), "=A2+B2");
    assert_eq!(AutoFillEngine::translate_formula(f1, 0, 1), "=B1+C1");
    assert_eq!(AutoFillEngine::translate_formula(f1, 3, 2), "=C4+D4");
}

#[test]
fn test_08_autofill_translate_absolute_fixed_references() {
    let f = "=$A$1+B$2+$C3";
    // Shift row by +2, col by +1
    let translated = AutoFillEngine::translate_formula(f, 2, 1);
    // $A$1 unchanged; B$2 -> C$2; $C3 -> $C5
    assert_eq!(translated, "=$A$1+C$2+$C5");
}

#[test]
fn test_09_autofill_translate_ranges() {
    // Running sum pattern
    let f_sum = "=SUM($A$1:A1)";
    let step1 = AutoFillEngine::translate_formula(f_sum, 1, 0);
    assert_eq!(step1, "=SUM($A$1:A2)");

    let step2 = AutoFillEngine::translate_formula(f_sum, 2, 0);
    assert_eq!(step2, "=SUM($A$1:A3)");

    let f_avg = "=AVERAGE(B2:D2)";
    assert_eq!(
        AutoFillEngine::translate_formula(f_avg, 3, 0),
        "=AVERAGE(B5:D5)"
    );
}

#[test]
fn test_10_autofill_predict_linear_numeric_series() {
    // Arithmetic step 1
    let s1 = vec!["1".to_string(), "2".to_string()];
    let p1 = AutoFillEngine::predict_series(&s1, 3, false);
    assert_eq!(p1, vec!["3", "4", "5"]);

    // Arithmetic step 10
    let s2 = vec!["10".to_string(), "20".to_string()];
    let p2 = AutoFillEngine::predict_series(&s2, 2, false);
    assert_eq!(p2, vec!["30", "40"]);

    // Single number duplicate
    let s3 = vec!["42".to_string()];
    let p3 = AutoFillEngine::predict_series(&s3, 3, false);
    assert_eq!(p3, vec!["42", "42", "42"]);
}

#[test]
fn test_11_autofill_predict_alphanumeric_series() {
    let s1 = vec!["Day 1".to_string()];
    let p1 = AutoFillEngine::predict_series(&s1, 3, false);
    assert_eq!(p1, vec!["Day 2", "Day 3", "Day 4"]);

    let s2 = vec!["Quarter 1".to_string(), "Quarter 2".to_string()];
    let p2 = AutoFillEngine::predict_series(&s2, 2, false);
    assert_eq!(p2, vec!["Quarter 3", "Quarter 4"]);

    let s3 = vec!["Item 008".to_string(), "Item 009".to_string()];
    let p3 = AutoFillEngine::predict_series(&s3, 2, false);
    assert_eq!(p3, vec!["Item 010", "Item 011"]);
}

#[test]
fn test_12_autofill_predict_formulas() {
    let s = vec!["=A1*2".to_string()];
    // Vertical auto-fill
    let pv = AutoFillEngine::predict_series(&s, 2, false);
    assert_eq!(pv, vec!["=A2*2", "=A3*2"]);

    // Horizontal auto-fill
    let ph = AutoFillEngine::predict_series(&s, 2, true);
    assert_eq!(ph, vec!["=B1*2", "=C1*2"]);
}

#[test]
fn test_13_autofill_compute_target_direction() {
    let src = ExcelSelectionRange::single(2, 2);

    // Drag down to row 5
    let (t_v, is_h1) = AutoFillEngine::compute_autofill_target(&src, (5, 2));
    assert!(!is_h1);
    assert_eq!(
        t_v,
        ExcelSelectionRange {
            min_row: 2,
            min_col: 2,
            max_row: 5,
            max_col: 2,
        }
    );

    // Drag right to col 6
    let (t_h, is_h2) = AutoFillEngine::compute_autofill_target(&src, (2, 6));
    assert!(is_h2);
    assert_eq!(
        t_h,
        ExcelSelectionRange {
            min_row: 2,
            min_col: 2,
            max_row: 2,
            max_col: 6,
        }
    );
}

#[test]
fn test_14_compile_autofill_preview_display_command() {
    let dims = GridDimensionMap::new(20.0, 60.0);
    let target = ExcelSelectionRange {
        min_row: 1,
        min_col: 1,
        max_row: 4,
        max_col: 1,
    };

    let list = ExcelDisplayCompiler::compile_autofill_preview(&target, &dims, 0.0, 0.0, 50.0, 26.0);
    assert_eq!(list.commands.len(), 1);

    if let DisplayCommand::DrawQuad(q) = &list.commands[0] {
        assert_eq!(q.border_width, 1.5);
    } else {
        panic!("Expected DrawQuad for autofill preview");
    }
}

#[test]
fn test_15_compile_formula_references_display_commands() {
    let dims = GridDimensionMap::new(20.0, 60.0);
    let refs = vec![
        (
            ExcelSelectionRange::single(0, 0),
            ColorRgba::new(0.1, 0.5, 0.9, 1.0),
        ),
        (
            ExcelSelectionRange::single(1, 1),
            ColorRgba::new(0.1, 0.8, 0.2, 1.0),
        ),
    ];

    let list = ExcelDisplayCompiler::compile_formula_references(&refs, &dims, 0.0, 0.0, 50.0, 26.0);
    assert_eq!(list.commands.len(), 2);
}

#[test]
fn test_16_compile_multi_selection_display_commands() {
    let dims = GridDimensionMap::new(20.0, 60.0);
    let mut model = MultiRangeSelectionModel::new(0, 0);
    model.add_range(ExcelSelectionRange::single(3, 3));

    let list = ExcelDisplayCompiler::compile_multi_selection(&model, &dims, 0.0, 0.0, 50.0, 26.0);

    // 2 selection quads + 1 fill handle on primary range = 3 quads
    assert_eq!(list.commands.len(), 3);
}

#[test]
fn test_17_command_bus_set_cell_formula_and_recalc() {
    let cmd = OfficeCommand::Excel(ExcelAction::SetCellFormula {
        coord: "C4".to_string(),
        formula: "=SUM(C1:C3)".to_string(),
    });
    let res = CommandDispatcher::dispatch(cmd);
    assert!(res.is_ok());

    let cmd_recalc = OfficeCommand::Excel(ExcelAction::Recalculate);
    assert!(CommandDispatcher::dispatch(cmd_recalc).is_ok());
}

#[test]
fn test_18_120fps_multi_selection_and_formula_highlight_sla() {
    let dims = GridDimensionMap::new(20.0, 64.0);
    let mut model = MultiRangeSelectionModel::new(100, 20);
    model.extend_selection(150, 40);
    model.add_range(ExcelSelectionRange {
        min_row: 200,
        min_col: 10,
        max_row: 250,
        max_col: 30,
    });

    let refs = vec![
        (
            ExcelSelectionRange::single(100, 20),
            ColorRgba::new(0.1, 0.5, 0.9, 1.0),
        ),
        (
            ExcelSelectionRange::single(150, 40),
            ColorRgba::new(0.1, 0.8, 0.2, 1.0),
        ),
    ];

    let start = Instant::now();
    let iterations = 100;
    for _ in 0..iterations {
        let _ = ExcelDisplayCompiler::compile_multi_selection(
            &model, &dims, 1000.0, 2000.0, 50.0, 26.0,
        );
        let _ = ExcelDisplayCompiler::compile_formula_references(
            &refs, &dims, 1000.0, 2000.0, 50.0, 26.0,
        );
    }
    let elapsed = start.elapsed();
    let per_frame_ms = (elapsed.as_secs_f64() * 1000.0) / (iterations as f64);

    println!(
        "Multi-selection & formula highlights SLA per frame: {:.3} ms",
        per_frame_ms
    );
    assert!(
        per_frame_ms < 1.0,
        "SLA violated: took {:.3} ms (must be < 1.0 ms)",
        per_frame_ms
    );
}

#[test]
fn test_19_zero_panics_scan_excel_and_shell() {
    let excel_content = include_str!("../src/compiler/excel.rs");
    for line in excel_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }
        assert!(!trimmed.contains(".unwrap()"));
        assert!(!trimmed.contains(".expect("));
        assert!(!trimmed.contains("panic!("));
        assert!(!trimmed.contains("todo!("));
        assert!(!trimmed.contains("unimplemented!("));
    }
}

#[test]
fn test_20_stage_tracker_subphase_2_3_2_verification() {
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
    let found = subphases.iter().find(|s| s["id"] == "2.3.2");
    assert!(
        found.is_some(),
        "Subphase 2.3.2 must exist in stage tracker"
    );
}
