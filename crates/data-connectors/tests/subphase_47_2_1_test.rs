//! Sub-Phase 47.2.1 Integration Tests:
//! End-to-End Rust OLAP Cube to Pivot Table UI Binding.
//! Validates:
//! 1. Grid extraction to strongly-typed datasets.
//! 2. Multi-dimensional hierarchical OLAP aggregation.
//! 3. Slicer and page report multi-value filtering.
//! 4. Structured `PivotViewResult` generation.
//! 5. Direct `CompressedSparsePageGrid` rendering.
//! 6. Comprehensive aggregation metrics & GETPIVOTDATA simulation.

use data_connectors::olap::{
    extract_dataset_from_grid, AggregationType, DateGroupingLevel, FilterField, GrandTotalMode,
    MeasureField, OlapCube, PivotConfig, PivotField, PivotTableGridBridge,
};
use data_connectors::schema::{DataType, Field, Schema};
use data_connectors::value::DataValue;
use excel_engine::grid::{CellValue, CompressedSparsePageGrid};

fn build_test_sales_grid() -> CompressedSparsePageGrid {
    let mut grid = CompressedSparsePageGrid::new();

    // Headers at row 0
    let _ = grid.set_inline_string(0, 0, "Region");
    let _ = grid.set_inline_string(0, 1, "Category");
    let _ = grid.set_inline_string(0, 2, "Date");
    let _ = grid.set_inline_string(0, 3, "Sales");
    let _ = grid.set_inline_string(0, 4, "Units");

    // Data rows 1..=5
    let records = [
        ("North", "Electronics", "2026-01-15", 1000.0, 2.0),
        ("North", "Furniture", "2026-02-10", 500.0, 1.0),
        ("South", "Electronics", "2026-05-20", 750.0, 3.0),
        ("South", "Furniture", "2026-08-14", 250.0, 5.0),
        ("North", "Electronics", "2026-03-01", 1200.0, 4.0),
    ];

    for (r_idx, (reg, cat, dt, sales, units)) in records.iter().enumerate() {
        let r = (r_idx + 1) as u32;
        let _ = grid.set_inline_string(r, 0, reg);
        let _ = grid.set_inline_string(r, 1, cat);
        let _ = grid.set_inline_string(r, 2, dt);
        let _ = grid.set_number(r, 3, *sales);
        let _ = grid.set_number(r, 4, *units);
    }

    grid
}

#[test]
fn test_01_grid_extraction_into_dataset() {
    let grid = build_test_sales_grid();
    let (schema, records) = extract_dataset_from_grid(&grid, 0, 0, 5, 4, true).unwrap();

    assert_eq!(schema.fields.len(), 5);
    assert_eq!(schema.fields[0].name, "Region");
    assert_eq!(schema.fields[1].name, "Category");
    assert_eq!(schema.fields[2].name, "Date");
    assert_eq!(schema.fields[3].name, "Sales");
    assert_eq!(schema.fields[4].name, "Units");

    assert_eq!(schema.fields[0].data_type, DataType::Utf8);
    assert_eq!(schema.fields[2].data_type, DataType::Date);
    assert_eq!(schema.fields[3].data_type, DataType::Int64);

    assert_eq!(records.len(), 5);
    assert_eq!(records[0][0], DataValue::Text("North".into()));
    assert_eq!(records[0][3], DataValue::Int64(1000));
}

#[test]
fn test_02_grid_extraction_invalid_bounds() {
    let grid = CompressedSparsePageGrid::new();
    let res = extract_dataset_from_grid(&grid, 5, 2, 2, 2, true);
    assert!(res.is_err());
}

#[test]
fn test_03_olap_cube_collect_unique_values() {
    let grid = build_test_sales_grid();
    let (schema, records) = extract_dataset_from_grid(&grid, 0, 0, 5, 4, true).unwrap();

    let unique_map = OlapCube::collect_unique_values(&records, &schema);

    let regions = unique_map.get("Region").unwrap();
    assert_eq!(regions, &vec!["North".to_string(), "South".to_string()]);

    let categories = unique_map.get("Category").unwrap();
    assert_eq!(
        categories,
        &vec!["Electronics".to_string(), "Furniture".to_string()]
    );
}

#[test]
fn test_04_to_view_result_structure() {
    let grid = build_test_sales_grid();
    let (schema, records) = extract_dataset_from_grid(&grid, 0, 0, 5, 4, true).unwrap();

    let config = PivotConfig {
        row_fields: vec![PivotField::new("Region")],
        col_fields: vec![PivotField::new("Category")],
        value_fields: vec![MeasureField::new("Sales", AggregationType::Sum)],
        filter_fields: vec![],
        grand_totals: GrandTotalMode::Both,
        show_subtotals: true,
    };

    let cube = OlapCube::build(&schema, &records, config).unwrap();
    let view = cube.to_view_result(10, 5);

    assert_eq!(view.layout.start_row, 10);
    assert_eq!(view.layout.start_col, 5);
    assert_eq!(view.row_headers.len(), 2); // North, South
    assert_eq!(view.col_headers.len(), 2); // Electronics, Furniture

    // North Electronics: 1000 + 1200 = 2200
    // North Furniture: 500
    // South Electronics: 750
    // South Furniture: 250
    let north_elec = view
        .cells
        .iter()
        .find(|c| c.row == 11 && c.col == 6)
        .unwrap();
    assert_eq!(north_elec.value, 2200.0);
    assert!(!north_elec.is_grand_total);

    // Row total for North: 2200 + 500 = 2700
    let north_total = view
        .cells
        .iter()
        .find(|c| c.row == 11 && c.col == 8)
        .unwrap();
    assert_eq!(north_total.value, 2700.0);
    assert!(north_total.is_grand_total);

    // Grand total: 2700 + 1000 = 3700
    assert_eq!(view.grand_totals, vec![3700.0]);
}

#[test]
fn test_05_multidimensional_nested_hierarchy() {
    let grid = build_test_sales_grid();
    let (schema, records) = extract_dataset_from_grid(&grid, 0, 0, 5, 4, true).unwrap();

    let config = PivotConfig {
        row_fields: vec![PivotField::new("Region"), PivotField::new("Category")],
        col_fields: vec![],
        value_fields: vec![
            MeasureField::new("Sales", AggregationType::Sum),
            MeasureField::new("Units", AggregationType::Average),
        ],
        filter_fields: vec![],
        grand_totals: GrandTotalMode::RowsOnly,
        show_subtotals: true,
    };

    let cube = OlapCube::build(&schema, &records, config).unwrap();
    assert_eq!(cube.row_coords.len(), 4); // (North, Electronics), (North, Furniture), (South, Electronics), (South, Furniture)

    // Check measure 0 (Sales sum) for (North, Electronics) = 2200
    assert_eq!(cube.get_value(0, 0, 0), 2200.0);
    // Check measure 1 (Units avg) for (North, Electronics) = (2 + 4) / 2 = 3.0
    assert_eq!(cube.get_value(0, 0, 1), 3.0);
}

#[test]
fn test_06_slicer_multi_value_filtering() {
    let grid = build_test_sales_grid();
    let (schema, records) = extract_dataset_from_grid(&grid, 0, 0, 5, 4, true).unwrap();

    // Slicer filters Region to only "South"
    let config = PivotConfig {
        row_fields: vec![PivotField::new("Category")],
        col_fields: vec![],
        value_fields: vec![MeasureField::new("Sales", AggregationType::Sum)],
        filter_fields: vec![FilterField::include(
            "Region",
            vec![DataValue::Text("South".into())],
        )],
        grand_totals: GrandTotalMode::Both,
        show_subtotals: false,
    };

    let cube = OlapCube::build(&schema, &records, config).unwrap();
    // Only South records (Electronics: 750, Furniture: 250) -> total 1000
    assert_eq!(cube.get_grand_total(0), 1000.0);
    assert_eq!(cube.row_coords.len(), 2);
}

#[test]
fn test_07_date_grouping_hierarchy() {
    let schema = Schema::new(vec![
        Field::new("Date", DataType::Date, false),
        Field::new("Amount", DataType::Float64, false),
    ]);

    let records = vec![
        vec![
            DataValue::Text("2026-01-10".into()),
            DataValue::Float64(100.0),
        ],
        vec![
            DataValue::Text("2026-02-15".into()),
            DataValue::Float64(200.0),
        ],
        vec![
            DataValue::Text("2026-05-20".into()),
            DataValue::Float64(400.0),
        ],
    ];

    let config = PivotConfig {
        row_fields: vec![PivotField::new("Date").with_date_grouping(DateGroupingLevel::Quarter)],
        col_fields: vec![],
        value_fields: vec![MeasureField::new("Amount", AggregationType::Sum)],
        filter_fields: vec![],
        grand_totals: GrandTotalMode::Both,
        show_subtotals: false,
    };

    let cube = OlapCube::build(&schema, &records, config).unwrap();
    assert_eq!(cube.row_coords.len(), 2); // Q1, Q2
    assert_eq!(cube.row_coords[0], vec!["Q1".to_string()]);
    assert_eq!(cube.row_coords[1], vec!["Q2".to_string()]);
    assert_eq!(cube.get_value(0, 0, 0), 300.0); // 100 + 200
    assert_eq!(cube.get_value(1, 0, 0), 400.0);
}

#[test]
fn test_08_aggregation_kinds_comprehensive() {
    let schema = Schema::new(vec![
        Field::new("Group", DataType::Utf8, false),
        Field::new("Val", DataType::Float64, false),
    ]);

    let records = vec![
        vec![DataValue::Text("A".into()), DataValue::Float64(10.0)],
        vec![DataValue::Text("A".into()), DataValue::Float64(20.0)],
        vec![DataValue::Text("A".into()), DataValue::Float64(30.0)],
    ];

    let aggs = [
        (AggregationType::Sum, 60.0),
        (AggregationType::Count, 3.0),
        (AggregationType::Average, 20.0),
        (AggregationType::Min, 10.0),
        (AggregationType::Max, 30.0),
        (AggregationType::Product, 6000.0),
        (AggregationType::DistinctCount, 3.0),
    ];

    for (agg, expected) in aggs {
        let config = PivotConfig {
            row_fields: vec![PivotField::new("Group")],
            col_fields: vec![],
            value_fields: vec![MeasureField::new("Val", agg)],
            filter_fields: vec![],
            grand_totals: GrandTotalMode::Both,
            show_subtotals: false,
        };

        let cube = OlapCube::build(&schema, &records, config).unwrap();
        assert_eq!(cube.get_value(0, 0, 0), expected);
    }
}

#[test]
fn test_09_render_to_grid_and_verify_cells() {
    let mut grid = build_test_sales_grid();
    let (schema, records) = extract_dataset_from_grid(&grid, 0, 0, 5, 4, true).unwrap();

    let config = PivotConfig {
        row_fields: vec![PivotField::new("Region")],
        col_fields: vec![PivotField::new("Category")],
        value_fields: vec![MeasureField::new("Sales", AggregationType::Sum)],
        filter_fields: vec![],
        grand_totals: GrandTotalMode::Both,
        show_subtotals: true,
    };

    let cube = OlapCube::build(&schema, &records, config).unwrap();
    // Render pivot table starting at row 10, col 0
    let report = PivotTableGridBridge::render_to_grid(&cube, &mut grid, 10, 0).unwrap();

    assert_eq!(report.start_row, 10);
    assert_eq!(report.measures_count, 1);

    // Verify written headers
    let header_cell = grid.get_value(10, 0);
    assert_eq!(header_cell, CellValue::InlineString("Region".to_string()));

    // Verify Grand Total row
    let gt_label = grid.get_value(13, 0);
    assert_eq!(gt_label, CellValue::InlineString("Grand Total".to_string()));

    let gt_val = grid.get_value(13, 3);
    assert_eq!(gt_val, CellValue::Number(3700.0));
}

#[test]
fn test_10_get_pivot_data_queries() {
    let grid = build_test_sales_grid();
    let (schema, records) = extract_dataset_from_grid(&grid, 0, 0, 5, 4, true).unwrap();

    let config = PivotConfig {
        row_fields: vec![PivotField::new("Region")],
        col_fields: vec![PivotField::new("Category")],
        value_fields: vec![MeasureField::new("Sales", AggregationType::Sum)],
        filter_fields: vec![],
        grand_totals: GrandTotalMode::Both,
        show_subtotals: true,
    };

    let cube = OlapCube::build(&schema, &records, config).unwrap();

    // Query North Electronics
    let val = cube.get_pivot_data("Sales", &[("Region", "North"), ("Category", "Electronics")]);
    assert_eq!(val, Some(2200.0));

    // Query South row total
    let south_total = cube.get_pivot_data("Sales", &[("Region", "South")]);
    assert_eq!(south_total, Some(1000.0));

    // Query grand total
    let grand = cube.get_pivot_data("Sales", &[]);
    assert_eq!(grand, Some(3700.0));
}
