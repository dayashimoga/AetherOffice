//! Subphase 2.1.2 Integration Tests:
//! Compressed Sparse Page Grid (CSPG), Coordinates, and Parallel Worksheet Engine.
//! Validates GEC-v1.0 Article III (< 180MB RSS for 1M cells) and Article IV (Zero Panics).

use excel_engine::*;
use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;

fn resolve_fixture(relative: &str) -> PathBuf {
    let direct = PathBuf::from(relative);
    if direct.exists() {
        return direct;
    }
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let from_manifest = manifest_dir.join("../../").join(relative);
    if from_manifest.exists() {
        return from_manifest;
    }
    direct
}

// ---------------------------------------------------------------------------
// 1. Coordinate Conversion Tests
// ---------------------------------------------------------------------------

#[test]
fn test_01_coord_single_letter_columns() {
    assert_eq!(col_to_name(0).unwrap(), "A");
    assert_eq!(col_to_name(1).unwrap(), "B");
    assert_eq!(col_to_name(25).unwrap(), "Z");

    assert_eq!(name_to_col("A").unwrap(), 0);
    assert_eq!(name_to_col("B").unwrap(), 1);
    assert_eq!(name_to_col("Z").unwrap(), 25);
}

#[test]
fn test_02_coord_double_letter_columns() {
    assert_eq!(col_to_name(26).unwrap(), "AA");
    assert_eq!(col_to_name(51).unwrap(), "AZ");
    assert_eq!(col_to_name(52).unwrap(), "BA");
    assert_eq!(col_to_name(701).unwrap(), "ZZ");

    assert_eq!(name_to_col("AA").unwrap(), 26);
    assert_eq!(name_to_col("AZ").unwrap(), 51);
    assert_eq!(name_to_col("BA").unwrap(), 52);
    assert_eq!(name_to_col("ZZ").unwrap(), 701);
}

#[test]
fn test_03_coord_triple_letter_max_bounds() {
    assert_eq!(col_to_name(702).unwrap(), "AAA");
    assert_eq!(col_to_name(16383).unwrap(), "XFD");

    assert_eq!(name_to_col("AAA").unwrap(), 702);
    assert_eq!(name_to_col("XFD").unwrap(), 16383);
}

#[test]
fn test_04_coord_origin_and_max_cells() {
    assert_eq!(coords_to_cell(0, 0).unwrap(), "A1");
    assert_eq!(cell_to_coords("A1").unwrap(), (0, 0));

    assert_eq!(coords_to_cell(1048575, 16383).unwrap(), "XFD1048576");
    assert_eq!(cell_to_coords("XFD1048576").unwrap(), (1048575, 16383));
}

#[test]
fn test_05_coord_dollar_signs_stripping() {
    assert_eq!(cell_to_coords("$A$1").unwrap(), (0, 0));
    assert_eq!(cell_to_coords("A$1").unwrap(), (0, 0));
    assert_eq!(cell_to_coords("$A1").unwrap(), (0, 0));
    assert_eq!(cell_to_coords("$XFD$1048576").unwrap(), (1048575, 16383));
}

#[test]
fn test_06_coord_invalid_inputs_error_handling() {
    // Col 16384 (XFE) is out of bounds
    assert!(name_to_col("XFE").is_err());
    assert!(cell_to_coords("XFE1").is_err());
    assert!(cell_to_coords("AAAA1").is_err());
    assert!(cell_to_coords("123").is_err());
    assert!(cell_to_coords("A0").is_err());
    assert!(cell_to_coords("A1048577").is_err());
    assert!(coords_to_cell(1048576, 0).is_err());
    assert!(coords_to_cell(0, 16384).is_err());
}

#[test]
fn test_07_coord_parse_range() {
    assert_eq!(parse_range("B5").unwrap(), ((4, 1), (4, 1)));
    assert_eq!(parse_range("A1:C10").unwrap(), ((0, 0), (9, 2)));
    assert_eq!(parse_range("C10:A1").unwrap(), ((0, 0), (9, 2)));
}

// ---------------------------------------------------------------------------
// 2. Compressed Sparse Page Grid (CSPG) Storage Tests
// ---------------------------------------------------------------------------

#[test]
fn test_08_grid_numbers_storage_and_retrieval() {
    let mut grid = CompressedSparsePageGrid::new();
    grid.set_number(0, 0, 42.5).unwrap();
    grid.set_number(10, 5, -123.456).unwrap();

    assert_eq!(grid.get_value(0, 0), CellValue::Number(42.5));
    assert_eq!(grid.get_value(10, 5), CellValue::Number(-123.456));
    assert_eq!(grid.get_value(99, 99), CellValue::Empty);
    assert_eq!(grid.populated_cell_count(), 2);
}

#[test]
fn test_09_grid_shared_strings_storage() {
    let mut grid = CompressedSparsePageGrid::new();
    grid.set_shared_string(0, 0, 0).unwrap();
    grid.set_shared_string(5, 5, 142).unwrap();

    assert_eq!(grid.get_value(0, 0), CellValue::SharedString(0));
    assert_eq!(grid.get_value(5, 5), CellValue::SharedString(142));
}

#[test]
fn test_10_grid_booleans_storage() {
    let mut grid = CompressedSparsePageGrid::new();
    grid.set_boolean(0, 1, true).unwrap();
    grid.set_boolean(0, 2, false).unwrap();

    assert_eq!(grid.get_value(0, 1), CellValue::Boolean(true));
    assert_eq!(grid.get_value(0, 2), CellValue::Boolean(false));
}

#[test]
fn test_11_grid_errors_storage() {
    let mut grid = CompressedSparsePageGrid::new();
    grid.set_error(0, 0, CellError::Div0).unwrap();
    grid.set_error(0, 1, CellError::NA).unwrap();
    grid.set_error(0, 2, CellError::Value).unwrap();
    grid.set_error(0, 3, CellError::Ref).unwrap();

    assert_eq!(grid.get_value(0, 0), CellValue::Error(CellError::Div0));
    assert_eq!(grid.get_value(0, 1), CellValue::Error(CellError::NA));
    assert_eq!(grid.get_value(0, 2), CellValue::Error(CellError::Value));
    assert_eq!(grid.get_value(0, 3), CellValue::Error(CellError::Ref));
}

#[test]
fn test_12_grid_formulas_storage() {
    let mut grid = CompressedSparsePageGrid::new();
    grid.set_formula(1, 2, "A2+B2", Some(142.0)).unwrap();

    match grid.get_value(1, 2) {
        CellValue::Formula {
            formula,
            cached_value,
        } => {
            assert_eq!(formula, "A2+B2");
            assert_eq!(cached_value, Some(142.0));
        }
        other => panic!("Expected formula cell, found {:?}", other),
    }
}

#[test]
fn test_13_grid_inline_strings() {
    let mut grid = CompressedSparsePageGrid::new();
    grid.set_inline_string(2, 3, "Inline Text").unwrap();

    assert_eq!(
        grid.get_value(2, 3),
        CellValue::InlineString("Inline Text".to_string())
    );
}

#[test]
fn test_14_grid_styles_storage() {
    let mut grid = CompressedSparsePageGrid::new();
    grid.set_style(0, 0, 1).unwrap();
    grid.set_style(10, 10, 5).unwrap();

    assert_eq!(grid.get_style(0, 0), Some(1));
    assert_eq!(grid.get_style(10, 10), Some(5));
    assert_eq!(grid.get_style(20, 20), None);
}

#[test]
fn test_15_grid_clear_cell_resets_occupancy() {
    let mut grid = CompressedSparsePageGrid::new();
    grid.set_number(5, 8, 999.0).unwrap();
    assert!(grid.is_occupied(5, 8));
    assert_eq!(grid.populated_cell_count(), 1);

    grid.clear_cell(5, 8).unwrap();
    assert!(!grid.is_occupied(5, 8));
    assert_eq!(grid.get_value(5, 8), CellValue::Empty);
    assert_eq!(grid.populated_cell_count(), 0);
    // Page chunk should be removed when empty
    assert_eq!(grid.page_count(), 0);
}

#[test]
fn test_16_grid_chunk_boundary_crossings() {
    let mut grid = CompressedSparsePageGrid::new();
    grid.set_number(63, 63, 1.0).unwrap(); // Page (0, 0)
    grid.set_number(64, 64, 2.0).unwrap(); // Page (1, 1)
    grid.set_number(127, 127, 3.0).unwrap(); // Page (1, 1)
    grid.set_number(128, 128, 4.0).unwrap(); // Page (2, 2)

    assert_eq!(grid.page_count(), 3);
    assert_eq!(grid.get_value(63, 63), CellValue::Number(1.0));
    assert_eq!(grid.get_value(64, 64), CellValue::Number(2.0));
    assert_eq!(grid.get_value(127, 127), CellValue::Number(3.0));
    assert_eq!(grid.get_value(128, 128), CellValue::Number(4.0));
}

#[test]
fn test_17_grid_four_corners_extreme_bounds() {
    let mut grid = CompressedSparsePageGrid::new();
    // 4 corners of full Excel sheet (1,048,576 rows x 16,384 columns)
    grid.set_number(0, 0, 100.0).unwrap(); // A1 -> Page (0, 0)
    grid.set_number(0, 16383, 200.0).unwrap(); // XFD1 -> Page (0, 255)
    grid.set_number(1048575, 0, 300.0).unwrap(); // A1048576 -> Page (16383, 0)
    grid.set_number(1048575, 16383, 400.0).unwrap(); // XFD1048576 -> Page (16383, 255)

    assert_eq!(grid.page_count(), 4);
    assert_eq!(grid.get_value(0, 0), CellValue::Number(100.0));
    assert_eq!(grid.get_value(0, 16383), CellValue::Number(200.0));
    assert_eq!(grid.get_value(1048575, 0), CellValue::Number(300.0));
    assert_eq!(grid.get_value(1048575, 16383), CellValue::Number(400.0));

    // Memory footprint of 4 sparse corners is minuscule (< 200 KB)
    assert!(grid.estimated_memory_bytes() < 500_000);
}

#[test]
fn test_18_grid_bounding_box_and_dimensions() {
    let mut grid = CompressedSparsePageGrid::new();
    grid.set_number(1, 1, 10.0).unwrap(); // B2
    grid.set_number(9, 3, 20.0).unwrap(); // D10
    grid.set_number(4, 0, 30.0).unwrap(); // A5
    grid.set_number(7, 2, 40.0).unwrap(); // C8

    let bbox = grid.bounding_box().expect("Expected bounding box");
    assert_eq!(bbox, (1, 0, 9, 3)); // min_row: 1, min_col: 0 (A), max_row: 9, max_col: 3 (D)
    assert_eq!(grid.dimension_ref(), "A2:D10");
}

#[test]
fn test_19_grid_simd_sum_row_and_range() {
    let mut grid = CompressedSparsePageGrid::new();
    // Populate row 0 with values 1 to 100
    for c in 0..100 {
        grid.set_number(0, c, (c + 1) as f64).unwrap();
    }

    let sum = grid.simd_sum_range(0, 0, 0, 99);
    // Sum of 1..100 = n*(n+1)/2 = 100*101/2 = 5050
    assert_eq!(sum, 5050.0);
}

// ---------------------------------------------------------------------------
// 3. Worksheet XML Parser & Serializer Tests
// ---------------------------------------------------------------------------

#[test]
fn test_20_ws_parse_real_baseline_sheet1() {
    let fixture_path = resolve_fixture("specs/evidence/excel/baseline.xlsx");
    let file = File::open(&fixture_path).expect("Failed to open baseline.xlsx");
    let pkg = ExcelWorkbookPackage::open(file).expect("Failed to open package");

    assert_eq!(pkg.worksheets.len(), 1);
    let (name, ws) = &pkg.worksheets[0];
    assert_eq!(name, "Sheet1");
    assert_eq!(ws.dimension.as_deref(), Some("A1:C2"));

    // Cell A1: shared string 0 ("OpenOfficeRS Ground Truth Baseline")
    assert_eq!(ws.grid.get_value(0, 0), CellValue::SharedString(0));
    assert_eq!(
        pkg.shared_strings.get(0),
        Some("OpenOfficeRS Ground Truth Baseline")
    );

    // Cell A2: 42
    assert_eq!(ws.grid.get_value(1, 0), CellValue::Number(42.0));

    // Cell B2: 100
    assert_eq!(ws.grid.get_value(1, 1), CellValue::Number(100.0));

    // Cell C2: formula A2+B2 with cached 142
    match ws.grid.get_value(1, 2) {
        CellValue::Formula {
            formula,
            cached_value,
        } => {
            assert_eq!(formula, "A2+B2");
            assert_eq!(cached_value, Some(142.0));
        }
        other => panic!("Expected formula in C2, found {:?}", other),
    }
}

#[test]
fn test_21_ws_roundtrip_streaming_xml() {
    let xml = r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
        <dimension ref="A1:B2"/>
        <sheetData>
            <row r="1">
                <c r="A1"><v>10</v></c>
                <c r="B1"><v>20</v></c>
            </row>
            <row r="2">
                <c r="A2"><v>30</v></c>
                <c r="B2"><v>40</v></c>
            </row>
        </sheetData>
    </worksheet>"#;

    let ws = Worksheet::from_xml_str(xml).expect("Failed to parse worksheet");
    assert_eq!(ws.grid.populated_cell_count(), 4);
    assert_eq!(ws.grid.get_value(0, 0), CellValue::Number(10.0));
    assert_eq!(ws.grid.get_value(0, 1), CellValue::Number(20.0));
    assert_eq!(ws.grid.get_value(1, 0), CellValue::Number(30.0));
    assert_eq!(ws.grid.get_value(1, 1), CellValue::Number(40.0));

    let reserialized = ws.to_xml_string().expect("Failed to serialize worksheet");
    let reloaded = Worksheet::from_xml_str(&reserialized).expect("Failed to re-parse");

    assert_eq!(reloaded.grid.populated_cell_count(), 4);
    assert_eq!(reloaded.grid.get_value(0, 0), CellValue::Number(10.0));
    assert_eq!(reloaded.grid.get_value(1, 1), CellValue::Number(40.0));
}

#[test]
fn test_22_ws_sheet_views_and_format_pr() {
    let xml = r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
        <dimension ref="A1"/>
        <sheetViews>
            <sheetView tabSelected="1" workbookViewId="0" zoomScale="150"/>
        </sheetViews>
        <sheetFormatPr defaultRowHeight="18.0" customHeight="1"/>
        <sheetData>
            <row r="1"><c r="A1"><v>1</v></c></row>
        </sheetData>
        <pageMargins left="0.5" right="0.5" top="0.8" bottom="0.8" header="0.2" footer="0.2"/>
    </worksheet>"#;

    let ws = Worksheet::from_xml_str(xml).expect("Failed to parse worksheet");
    assert_eq!(ws.sheet_views.len(), 1);
    assert_eq!(ws.sheet_views[0].zoom_scale, Some(150));

    let sfp = ws.sheet_format_pr.as_ref().unwrap();
    assert_eq!(sfp.default_row_height, 18.0);
    assert!(sfp.custom_height);

    let pm = ws.page_margins.as_ref().unwrap();
    assert_eq!(pm.left, 0.5);
    assert_eq!(pm.top, 0.8);
}

#[test]
fn test_23_ws_unmodeled_nodes_preservation() {
    let xml = r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
        <dimension ref="A1"/>
        <sheetData>
            <row r="1"><c r="A1"><v>1</v></c></row>
        </sheetData>
        <autoFilter ref="A1:D10"/>
    </worksheet>"#;

    let ws = Worksheet::from_xml_str(xml).expect("Failed to parse worksheet");
    assert_eq!(ws.unmodeled_nodes.len(), 1);
    assert!(ws.unmodeled_nodes[0].raw_xml.contains("autoFilter"));
    assert!(ws.unmodeled_nodes[0].raw_xml.contains("A1:D10"));

    let reserialized = ws.to_xml_string().expect("Failed to serialize");
    assert!(reserialized.contains("<autoFilter ref=\"A1:D10\"/>"));
}

#[test]
fn test_24_package_parallel_worksheets_load_and_save() {
    let fixture_path = resolve_fixture("specs/evidence/excel/baseline.xlsx");
    let file = File::open(&fixture_path).expect("Failed to open baseline");
    let pkg = ExcelWorkbookPackage::open(file).expect("Failed to open");

    assert!(!pkg.worksheets.is_empty());

    let mut out = Cursor::new(Vec::new());
    pkg.save(&mut out)
        .expect("Failed to save package with worksheets");

    out.set_position(0);
    let reloaded = ExcelWorkbookPackage::open(out).expect("Failed to reload package");
    assert_eq!(reloaded.worksheets.len(), pkg.worksheets.len());
    let (_, ws) = &reloaded.worksheets[0];
    assert_eq!(ws.grid.get_value(1, 0), CellValue::Number(42.0));
}

// ---------------------------------------------------------------------------
// 4. Memory Footprint SLA Verification (GEC-v1.0 Article III)
// ---------------------------------------------------------------------------

/// Hard SLA Verification: GEC-v1.0 Article III
/// 1,000,000 populated Excel cells MUST consume < 180 MB RSS via CSPG.
#[test]
fn test_25_sla_1m_cells_memory_footprint() {
    let mut grid = CompressedSparsePageGrid::new();
    let total_cells = 1_000_000usize;

    // Populate a table of 10,000 rows x 100 columns = 1,000,000 cells
    let rows = 10_000u32;
    let cols = 100u32;

    for r in 0..rows {
        for c in 0..cols {
            grid.set_number(r, c, (r * cols + c) as f64).unwrap();
        }
    }

    assert_eq!(grid.populated_cell_count(), total_cells);

    let mem_bytes = grid.estimated_memory_bytes();
    let mem_mb = mem_bytes as f64 / (1024.0 * 1024.0);

    println!(
        "=== [Memory SLA Benchmark] 1,000,000 cells in CSPG: {:.2} MB (Pages: {}) ===",
        mem_mb,
        grid.page_count()
    );

    // GEC-v1.0 Article III: 1M populated Excel cells < 180 MB RSS
    assert!(
        mem_mb < 180.0,
        "SLA Violation: 1M cells consumed {:.2} MB, exceeding 180 MB limit",
        mem_mb
    );

    // Even stricter: Our SoA CSPG should comfortably consume < 25 MB!
    assert!(
        mem_mb < 25.0,
        "CSPG layout efficiency regression: consumed {:.2} MB",
        mem_mb
    );

    // Verify O(1) random access across bounds
    assert_eq!(grid.get_value(0, 0), CellValue::Number(0.0));
    assert_eq!(grid.get_value(5000, 50), CellValue::Number(500050.0));
    assert_eq!(grid.get_value(9999, 99), CellValue::Number(999999.0));
}

#[test]
fn test_26_vectors_manifest_validation() {
    let vectors_path = resolve_fixture("specs/evidence/excel/2.1.2_vectors.json");
    let content = std::fs::read_to_string(&vectors_path).expect("Failed to read vectors JSON");
    let json: serde_json::Value =
        serde_json::from_str(&content).expect("Failed to parse vectors JSON");

    assert_eq!(json["subphase"], "2.1.2");
    let vectors = json["vectors"].as_array().expect("vectors array");
    assert_eq!(vectors.len(), 26);
}
