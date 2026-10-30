//! Subphase 18.1.2 Integration Tests:
//! Fast In-Memory OLAP Aggregation Cube Engine.
//! Covers complex pivots, multi-level nesting, date hierarchies, grand total recalculations,
//! GETPIVOTDATA simulation, and direct Grid rendering.

use data_connectors::olap::{
    AggregationType, DateGroupingLevel, FilterField, GrandTotalMode, MeasureField, OlapCube,
    PivotConfig, PivotField, PivotTableGridBridge,
};
use data_connectors::schema::{DataType, Field, Schema};
use data_connectors::value::DataValue;
use excel_engine::grid::{CellValue, CompressedSparsePageGrid};
use std::time::Instant;

fn sample_sales_dataset() -> (Schema, Vec<Vec<DataValue>>) {
    let schema = Schema::new(vec![
        Field::new("Region", DataType::Utf8, false),
        Field::new("Category", DataType::Utf8, false),
        Field::new("Date", DataType::Date, false),
        Field::new("Sales", DataType::Float64, false),
        Field::new("Quantity", DataType::Int64, false),
        Field::new("CustomerID", DataType::Utf8, false),
    ]);

    let rows = vec![
        vec![
            DataValue::Text("North".into()),
            DataValue::Text("Electronics".into()),
            DataValue::Text("2026-01-15".into()),
            DataValue::Float64(1000.0),
            DataValue::Int64(2),
            DataValue::Text("CUST-1".into()),
        ],
        vec![
            DataValue::Text("North".into()),
            DataValue::Text("Furniture".into()),
            DataValue::Text("2026-02-10".into()),
            DataValue::Float64(500.0),
            DataValue::Int64(1),
            DataValue::Text("CUST-2".into()),
        ],
        vec![
            DataValue::Text("South".into()),
            DataValue::Text("Electronics".into()),
            DataValue::Text("2026-05-20".into()),
            DataValue::Float64(750.0),
            DataValue::Int64(3),
            DataValue::Text("CUST-1".into()),
        ],
        vec![
            DataValue::Text("South".into()),
            DataValue::Text("Furniture".into()),
            DataValue::Text("2026-08-14".into()),
            DataValue::Float64(250.0),
            DataValue::Int64(5),
            DataValue::Text("CUST-3".into()),
        ],
        vec![
            DataValue::Text("North".into()),
            DataValue::Text("Electronics".into()),
            DataValue::Text("2026-11-05".into()),
            DataValue::Float64(1500.0),
            DataValue::Int64(4),
            DataValue::Text("CUST-1".into()),
        ],
    ];

    (schema, rows)
}

#[test]
fn test_01_single_dimension_pivot_aggregations() {
    let (schema, rows) = sample_sales_dataset();

    let config = PivotConfig {
        row_fields: vec![PivotField::new("Region")],
        col_fields: vec![],
        value_fields: vec![
            MeasureField::new("Sales", AggregationType::Sum),
            MeasureField::new("Sales", AggregationType::Count),
            MeasureField::new("Sales", AggregationType::Average),
            MeasureField::new("Sales", AggregationType::Min),
            MeasureField::new("Sales", AggregationType::Max),
        ],
        filter_fields: vec![],
        grand_totals: GrandTotalMode::Both,
        show_subtotals: true,
    };

    let cube = OlapCube::build(&schema, &rows, config).unwrap();

    // North has 3 records: 1000, 500, 1500. Total = 3000, Avg = 1000, Min = 500, Max = 1500.
    // South has 2 records: 750, 250. Total = 1000, Avg = 500, Min = 250, Max = 750.
    assert_eq!(cube.row_coords.len(), 2);
    assert_eq!(cube.row_coords[0], vec!["North".to_string()]);
    assert_eq!(cube.row_coords[1], vec!["South".to_string()]);

    // North metrics
    assert_eq!(cube.get_value(0, 0, 0), 3000.0); // Sum
    assert_eq!(cube.get_value(0, 0, 1), 3.0); // Count
    assert_eq!(cube.get_value(0, 0, 2), 1000.0); // Avg
    assert_eq!(cube.get_value(0, 0, 3), 500.0); // Min
    assert_eq!(cube.get_value(0, 0, 4), 1500.0); // Max

    // South metrics
    assert_eq!(cube.get_value(1, 0, 0), 1000.0);
    assert_eq!(cube.get_value(1, 0, 1), 2.0);
    assert_eq!(cube.get_value(1, 0, 2), 500.0);

    // Grand Totals: Sum = 4000, Count = 5, Avg = 800
    assert_eq!(cube.get_grand_total(0), 4000.0);
    assert_eq!(cube.get_grand_total(1), 5.0);
    assert_eq!(cube.get_grand_total(2), 800.0);
}

#[test]
fn test_02_multi_dimensional_hierarchical_grouping() {
    let (schema, rows) = sample_sales_dataset();

    let config = PivotConfig {
        row_fields: vec![PivotField::new("Region"), PivotField::new("Category")],
        col_fields: vec![],
        value_fields: vec![MeasureField::new("Sales", AggregationType::Sum)],
        filter_fields: vec![],
        grand_totals: GrandTotalMode::Both,
        show_subtotals: true,
    };

    let cube = OlapCube::build(&schema, &rows, config).unwrap();

    // Row tuples:
    // ["North", "Electronics"] -> 1000 + 1500 = 2500
    // ["North", "Furniture"] -> 500
    // ["South", "Electronics"] -> 750
    // ["South", "Furniture"] -> 250
    assert_eq!(cube.row_coords.len(), 4);
    assert_eq!(
        cube.row_coords[0],
        vec!["North".to_string(), "Electronics".to_string()]
    );
    assert_eq!(cube.get_value(0, 0, 0), 2500.0);
    assert_eq!(cube.get_value(1, 0, 0), 500.0);
    assert_eq!(cube.get_value(2, 0, 0), 750.0);
    assert_eq!(cube.get_value(3, 0, 0), 250.0);
}

#[test]
fn test_03_statistical_measures_variance_stddev_product() {
    let schema = Schema::new(vec![
        Field::new("Group", DataType::Utf8, false),
        Field::new("Score", DataType::Float64, false),
    ]);

    // Data: 2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0 (mean = 5.0, sum = 40)
    // Variance = ((2-5)^2 + 3*(4-5)^2 + 2*(5-5)^2 + (7-5)^2 + (9-5)^2) / (8 - 1)
    // = (9 + 3 + 0 + 4 + 16) / 7 = 32 / 7 = 4.571428571428571
    // StdDev = sqrt(32/7) = 2.138089935299395
    let rows = vec![
        vec![DataValue::Text("A".into()), DataValue::Float64(2.0)],
        vec![DataValue::Text("A".into()), DataValue::Float64(4.0)],
        vec![DataValue::Text("A".into()), DataValue::Float64(4.0)],
        vec![DataValue::Text("A".into()), DataValue::Float64(4.0)],
        vec![DataValue::Text("A".into()), DataValue::Float64(5.0)],
        vec![DataValue::Text("A".into()), DataValue::Float64(5.0)],
        vec![DataValue::Text("A".into()), DataValue::Float64(7.0)],
        vec![DataValue::Text("A".into()), DataValue::Float64(9.0)],
    ];

    let config = PivotConfig {
        row_fields: vec![PivotField::new("Group")],
        col_fields: vec![],
        value_fields: vec![
            MeasureField::new("Score", AggregationType::Variance),
            MeasureField::new("Score", AggregationType::StdDev),
        ],
        filter_fields: vec![],
        grand_totals: GrandTotalMode::None,
        show_subtotals: false,
    };

    let cube = OlapCube::build(&schema, &rows, config).unwrap();
    let variance = cube.get_value(0, 0, 0);
    let stddev = cube.get_value(0, 0, 1);

    assert!((variance - (32.0 / 7.0)).abs() < 1e-10);
    assert!((stddev - (32.0 / 7.0f64).sqrt()).abs() < 1e-10);
}

#[test]
fn test_04_distinct_count_and_null_handling() {
    let (schema, rows) = sample_sales_dataset();

    let config = PivotConfig {
        row_fields: vec![PivotField::new("Region")],
        col_fields: vec![],
        value_fields: vec![MeasureField::new(
            "CustomerID",
            AggregationType::DistinctCount,
        )],
        filter_fields: vec![],
        grand_totals: GrandTotalMode::Both,
        show_subtotals: false,
    };

    let cube = OlapCube::build(&schema, &rows, config).unwrap();

    // North has customers: CUST-1, CUST-2, CUST-1 -> Distinct = 2
    // South has customers: CUST-1, CUST-3 -> Distinct = 2
    assert_eq!(cube.get_value(0, 0, 0), 2.0);
    assert_eq!(cube.get_value(1, 0, 0), 2.0);

    // Total distinct across entire dataset: CUST-1, CUST-2, CUST-3 = 3
    assert_eq!(cube.get_grand_total(0), 3.0);
}

#[test]
fn test_05_percentage_of_total_and_row_totals() {
    let (schema, rows) = sample_sales_dataset();

    let config = PivotConfig {
        row_fields: vec![PivotField::new("Region")],
        col_fields: vec![],
        value_fields: vec![MeasureField::new("Sales", AggregationType::PercentOfTotal)],
        filter_fields: vec![],
        grand_totals: GrandTotalMode::Both,
        show_subtotals: false,
    };

    let cube = OlapCube::build(&schema, &rows, config).unwrap();

    // Grand total sales = 4000.0. North = 3000.0 (75%), South = 1000.0 (25%)
    let pct_north = cube.get_value(0, 0, 0);
    let pct_south = cube.get_value(1, 0, 0);

    assert_eq!(pct_north, 75.0);
    assert_eq!(pct_south, 25.0);
}

#[test]
fn test_06_automatic_date_grouping_hierarchy() {
    let (schema, rows) = sample_sales_dataset();

    let config = PivotConfig {
        row_fields: vec![
            PivotField::new("Date").with_date_grouping(DateGroupingLevel::Year),
            PivotField::new("Date").with_date_grouping(DateGroupingLevel::Quarter),
        ],
        col_fields: vec![],
        value_fields: vec![MeasureField::new("Sales", AggregationType::Sum)],
        filter_fields: vec![],
        grand_totals: GrandTotalMode::Both,
        show_subtotals: true,
    };

    let cube = OlapCube::build(&schema, &rows, config).unwrap();

    // Dates in sample:
    // 2026-01-15 -> 2026, Q1 (1000)
    // 2026-02-10 -> 2026, Q1 (500)   -> Q1 sum = 1500
    // 2026-05-20 -> 2026, Q2 (750)   -> Q2 sum = 750
    // 2026-08-14 -> 2026, Q3 (250)   -> Q3 sum = 250
    // 2026-11-05 -> 2026, Q4 (1500)  -> Q4 sum = 1500
    assert_eq!(cube.row_coords.len(), 4);
    assert_eq!(
        cube.row_coords[0],
        vec!["2026".to_string(), "Q1".to_string()]
    );
    assert_eq!(cube.get_value(0, 0, 0), 1500.0);
    assert_eq!(cube.get_value(1, 0, 0), 750.0);
    assert_eq!(cube.get_value(2, 0, 0), 250.0);
    assert_eq!(cube.get_value(3, 0, 0), 1500.0);
}

#[test]
fn test_07_page_report_filter_evaluation() {
    let (schema, rows) = sample_sales_dataset();

    // Include only North region
    let config = PivotConfig {
        row_fields: vec![PivotField::new("Category")],
        col_fields: vec![],
        value_fields: vec![MeasureField::new("Sales", AggregationType::Sum)],
        filter_fields: vec![FilterField::include(
            "Region",
            vec![DataValue::Text("North".into())],
        )],
        grand_totals: GrandTotalMode::Both,
        show_subtotals: false,
    };

    let cube = OlapCube::build(&schema, &rows, config).unwrap();

    // Electronics: 1000 + 1500 = 2500, Furniture: 500
    assert_eq!(cube.row_coords.len(), 2);
    assert_eq!(cube.get_value(0, 0, 0), 2500.0);
    assert_eq!(cube.get_value(1, 0, 0), 500.0);
    assert_eq!(cube.get_grand_total(0), 3000.0);
}

#[test]
fn test_08_pivot_table_grid_rendering_and_layout() {
    let (schema, rows) = sample_sales_dataset();

    let config = PivotConfig {
        row_fields: vec![PivotField::new("Region")],
        col_fields: vec![PivotField::new("Category")],
        value_fields: vec![MeasureField::new("Sales", AggregationType::Sum)],
        filter_fields: vec![],
        grand_totals: GrandTotalMode::Both,
        show_subtotals: true,
    };

    let cube = OlapCube::build(&schema, &rows, config).unwrap();
    let mut grid = CompressedSparsePageGrid::new();

    let report = PivotTableGridBridge::render_to_grid(&cube, &mut grid, 2, 1).unwrap();

    assert_eq!(report.start_row, 2);
    assert_eq!(report.start_col, 1);
    assert_eq!(report.row_labels_count, 2);
    assert_eq!(report.col_labels_count, 2);

    // Row 2 is Header: Col 1 is "Region", Col 2 is "Electronics | Sum of Sales", Col 3 is "Furniture | Sum of Sales", Col 4 is "Total Sum of Sales"
    assert_eq!(
        grid.get_value(2, 1),
        CellValue::InlineString("Region".to_string())
    );
    assert_eq!(
        grid.get_value(2, 2),
        CellValue::InlineString("Electronics | Sum of Sales".to_string())
    );

    // Row 3: North: Electronics = 2500, Furniture = 500, Total = 3000
    assert_eq!(
        grid.get_value(3, 1),
        CellValue::InlineString("North".to_string())
    );
    assert_eq!(grid.get_value(3, 2), CellValue::Number(2500.0));
    assert_eq!(grid.get_value(3, 3), CellValue::Number(500.0));
    assert_eq!(grid.get_value(3, 4), CellValue::Number(3000.0));

    // Row 4: South: Electronics = 750, Furniture = 250, Total = 1000
    assert_eq!(grid.get_value(4, 2), CellValue::Number(750.0));
    assert_eq!(grid.get_value(4, 3), CellValue::Number(250.0));
    assert_eq!(grid.get_value(4, 4), CellValue::Number(1000.0));

    // Row 5: Grand Total: Electronics = 3250, Furniture = 750, Grand Total = 4000
    assert_eq!(
        grid.get_value(5, 1),
        CellValue::InlineString("Grand Total".to_string())
    );
    assert_eq!(grid.get_value(5, 2), CellValue::Number(3250.0));
    assert_eq!(grid.get_value(5, 3), CellValue::Number(750.0));
    assert_eq!(grid.get_value(5, 4), CellValue::Number(4000.0));
}

#[test]
fn test_09_getpivotdata_formula_lookup_simulation() {
    let (schema, rows) = sample_sales_dataset();

    let config = PivotConfig {
        row_fields: vec![PivotField::new("Region")],
        col_fields: vec![PivotField::new("Category")],
        value_fields: vec![MeasureField::new("Sales", AggregationType::Sum)],
        filter_fields: vec![],
        grand_totals: GrandTotalMode::Both,
        show_subtotals: true,
    };

    let cube = OlapCube::build(&schema, &rows, config).unwrap();

    // Query 1: GETPIVOTDATA("Sales", Region="North", Category="Electronics") -> 2500
    let val1 = cube
        .get_pivot_data("Sales", &[("Region", "North"), ("Category", "Electronics")])
        .unwrap();
    assert_eq!(val1, 2500.0);

    // Query 2: GETPIVOTDATA("Sales", Region="South") -> Row Total = 1000
    let val2 = cube
        .get_pivot_data("Sales", &[("Region", "South")])
        .unwrap();
    assert_eq!(val2, 1000.0);

    // Query 3: GETPIVOTDATA("Sales") -> Grand Total = 4000
    let val3 = cube.get_pivot_data("Sales", &[]).unwrap();
    assert_eq!(val3, 4000.0);
}

#[test]
fn test_10_large_cube_performance_benchmark() {
    let total_rows = 50_000usize;
    let schema = Schema::new(vec![
        Field::new("Region", DataType::Utf8, false),
        Field::new("Quarter", DataType::Utf8, false),
        Field::new("Channel", DataType::Utf8, false),
        Field::new("Revenue", DataType::Float64, false),
    ]);

    let regions = ["North", "South", "East", "West"];
    let quarters = ["Q1", "Q2", "Q3", "Q4"];
    let channels = ["Direct", "Partner", "Online"];

    let mut rows = Vec::with_capacity(total_rows);
    for i in 0..total_rows {
        rows.push(vec![
            DataValue::Text(regions[i % 4].to_string()),
            DataValue::Text(quarters[i % 4].to_string()),
            DataValue::Text(channels[i % 3].to_string()),
            DataValue::Float64((i % 100) as f64 + 1.0),
        ]);
    }

    let config = PivotConfig {
        row_fields: vec![PivotField::new("Region"), PivotField::new("Quarter")],
        col_fields: vec![PivotField::new("Channel")],
        value_fields: vec![
            MeasureField::new("Revenue", AggregationType::Sum),
            MeasureField::new("Revenue", AggregationType::Count),
        ],
        filter_fields: vec![],
        grand_totals: GrandTotalMode::Both,
        show_subtotals: true,
    };

    let start = Instant::now();
    let cube = OlapCube::build(&schema, &rows, config).unwrap();
    let elapsed = start.elapsed();

    println!(
        "50k-row OLAP cube calculated in {:.2} ms",
        elapsed.as_secs_f64() * 1000.0
    );

    assert_eq!(cube.row_coords.len(), 4);
    assert_eq!(cube.col_coords.len(), 3);
    assert_eq!(cube.get_grand_total(1), 50_000.0); // Total count = 50,000
}
