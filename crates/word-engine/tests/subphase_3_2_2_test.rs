//! Subphase 3.2.2 Integration Test Suite: REA Table Auto-Fit & Floating Wrap Polygons
//!
//! Validates:
//! - Table column auto-fit solver (minContentWidth vs maxContentWidth with proportional interpolation).
//! - Unit conversions for cell & table widths (dxa, pct in 50ths of a percent, auto).
//! - Merged column slack distribution (w:gridSpan).
//! - Table alignments (left, center, right).
//! - Cell padding (w:tblCellMar, w:tcMar) and vertical alignments (top, center, bottom).
//! - Cross-page table row pagination with w:cantSplit and w:tblHeader repeating headers.
//! - DrawingML floating anchor text exclusion zones (wrapSquare, wrapTight, wrapThrough, wrapTopAndBottom).
//! - Full WordPackage integration and JSON test vectors manifest validation.

use std::path::PathBuf;

use word_engine::layout::{BoundingBox, FontMetricsProvider, LineBreaker, StandardFontMetrics};
use word_engine::model::{
    BodyElement, Paragraph, ParagraphChild, Run, RunContent, RunProperties, Table, TableCell,
    TableCellMargins, TableCellProperties, TableGrid, TableProperties, TableRow,
    TableRowProperties, TableWidth,
};
use word_engine::package::WordPackage;
use word_engine::styles::StyleSheet;
use word_engine::table_layout::{
    FloatingAnchor, FloatingWrapSolver, Point2D, TableAutoFitSolver, TablePaginationSolver,
    WrapType,
};

fn resolve_repo_path(rel: &str) -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut curr = manifest_dir.clone();
    for _ in 0..5 {
        let candidate = curr.join(rel);
        if candidate.exists() {
            return candidate;
        }
        if let Some(parent) = curr.parent() {
            curr = parent.to_path_buf();
        } else {
            break;
        }
    }
    manifest_dir.join(rel)
}

fn create_text_cell(text: &str, font_size_half_pts: u32) -> TableCell {
    let mut r = Run {
        properties: Some(RunProperties {
            size_half_points: Some(font_size_half_pts),
            font_ascii: Some("Calibri".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    r.content.push(RunContent::Text(text.to_string(), false));

    let mut p = Paragraph::default();
    p.children.push(ParagraphChild::Run(r));

    TableCell {
        elements: vec![BodyElement::Paragraph(p)],
        ..Default::default()
    }
}

// =========================================================================
// Tests 01-04: Width Units & Margins Resolution
// =========================================================================

#[test]
fn test_01_cell_width_conversion_dxa() {
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = TableAutoFitSolver::new(&metrics, &cascade);

    let tbl = Table {
        properties: Some(TableProperties {
            width: Some(TableWidth {
                width: 4320, // 3 inches in twips
                width_type: "dxa".to_string(),
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    let target = solver.resolve_target_table_width(&tbl, 10000);
    assert_eq!(target, 4320);
}

#[test]
fn test_02_cell_width_conversion_pct() {
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = TableAutoFitSolver::new(&metrics, &cascade);

    let mut tbl = Table {
        properties: Some(TableProperties {
            width: Some(TableWidth {
                width: 2500, // 50% in 50ths of a percent (5000 = 100%)
                width_type: "pct".to_string(),
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    // 50% of 10000 twips = 5000 twips
    let target = solver.resolve_target_table_width(&tbl, 10000);
    assert_eq!(target, 5000);

    // 100% of 9000 twips = 9000 twips
    tbl.properties
        .as_mut()
        .unwrap()
        .width
        .as_mut()
        .unwrap()
        .width = 5000;
    let target_100 = solver.resolve_target_table_width(&tbl, 9000);
    assert_eq!(target_100, 9000);
}

#[test]
fn test_03_cell_width_conversion_auto() {
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = TableAutoFitSolver::new(&metrics, &cascade);

    let tbl = Table {
        properties: Some(TableProperties {
            width: Some(TableWidth {
                width: 0,
                width_type: "auto".to_string(),
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    // Auto width uses entire available width
    let target = solver.resolve_target_table_width(&tbl, 8500);
    assert_eq!(target, 8500);
}

#[test]
fn test_04_cell_margins_resolution() {
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let breaker = LineBreaker::new(&metrics);
    let solver = TableAutoFitSolver::new(&metrics, &cascade);

    let cell = create_text_cell("Sample", 24);
    let padding_none = 0;
    let padding_custom = 200; // 100 left + 100 right

    let (min1, max1) = solver.measure_cell_content(&cell, &breaker, padding_none);
    let (min2, max2) = solver.measure_cell_content(&cell, &breaker, padding_custom);

    assert_eq!(min2 - min1, 200);
    assert_eq!(max2 - max1, 200);
}

// =========================================================================
// Tests 05-12: Auto-Fit Min/Max Content Width & Distribution
// =========================================================================

#[test]
fn test_05_min_content_width_unbreakable_word() {
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let breaker = LineBreaker::new(&metrics);
    let solver = TableAutoFitSolver::new(&metrics, &cascade);

    // Two words: "Short" and "Supercalifragilisticexpialidocious"
    let cell = create_text_cell("Short Supercalifragilisticexpialidocious", 24);
    let (min_w, max_w) = solver.measure_cell_content(&cell, &breaker, 0);

    // Min width should bound the longest unbreakable token
    let longest_token_w = metrics
        .measure_char('S', "Calibri", 24, false, false)
        .advance_width
        * 34;
    assert!(min_w >= longest_token_w / 2);
    assert!(max_w > min_w);
}

#[test]
fn test_06_max_content_width_unwrapped_paragraph() {
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let breaker = LineBreaker::new(&metrics);
    let solver = TableAutoFitSolver::new(&metrics, &cascade);

    let cell = create_text_cell("Hello World from OpenOfficeRS", 24);
    let (min_w, max_w) = solver.measure_cell_content(&cell, &breaker, 0);

    // Max content width is single-line unwrapped paragraph width
    assert!(max_w > min_w);
    assert!(max_w >= 2000);
}

#[test]
fn test_07_autofit_fixed_layout_uses_grid() {
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = TableAutoFitSolver::new(&metrics, &cascade);

    let tbl = Table {
        properties: Some(TableProperties {
            layout_type: Some("fixed".to_string()),
            ..Default::default()
        }),
        grid: Some(TableGrid {
            columns: vec![2000, 3000, 4000],
        }),
        rows: vec![TableRow {
            cells: vec![
                create_text_cell("A", 24),
                create_text_cell("B", 24),
                create_text_cell("C", 24),
            ],
            ..Default::default()
        }],
        ..Default::default()
    };

    let col_widths = solver.compute_column_widths(&tbl, 12000);
    assert_eq!(col_widths, vec![2000, 3000, 4000]);
}

#[test]
fn test_08_autofit_target_width_equal_min() {
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = TableAutoFitSolver::new(&metrics, &cascade);

    let min_widths = vec![1000, 1500, 2000];
    let max_widths = vec![2000, 3000, 4000];
    let target = 4500; // sum(min_widths)

    let distributed = solver.distribute_widths(&min_widths, &max_widths, target);
    assert_eq!(distributed, vec![1000, 1500, 2000]);
}

#[test]
fn test_09_autofit_target_width_equal_max() {
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = TableAutoFitSolver::new(&metrics, &cascade);

    let min_widths = vec![1000, 1500, 2000];
    let max_widths = vec![2000, 3000, 4000];
    let target = 9000; // sum(max_widths)

    let distributed = solver.distribute_widths(&min_widths, &max_widths, target);
    assert_eq!(distributed, vec![2000, 3000, 4000]);
}

#[test]
fn test_10_autofit_target_width_interpolated() {
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = TableAutoFitSolver::new(&metrics, &cascade);

    let min_widths = vec![1000, 1000];
    let max_widths = vec![2000, 2000];
    // Midpoint: sum_min = 2000, sum_max = 4000, target = 3000 (alpha = 0.5)
    let target = 3000;

    let distributed = solver.distribute_widths(&min_widths, &max_widths, target);
    assert_eq!(distributed, vec![1500, 1500]);
}

#[test]
fn test_11_autofit_slack_distribution_exceeds_max() {
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = TableAutoFitSolver::new(&metrics, &cascade);

    let min_widths = vec![1000, 2000];
    let max_widths = vec![2000, 4000]; // sum = 6000
    let target = 9000; // slack = 3000 distributed 1:2

    let distributed = solver.distribute_widths(&min_widths, &max_widths, target);
    // col 1 gets 2000 + (3000 * 2000 / 6000) = 3000
    // col 2 gets 4000 + (3000 * 4000 / 6000) = 6000
    assert_eq!(distributed, vec![3000, 6000]);
}

#[test]
fn test_12_autofit_compression_below_min() {
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = TableAutoFitSolver::new(&metrics, &cascade);

    let min_widths = vec![2000, 4000]; // sum = 6000
    let max_widths = vec![3000, 6000];
    let target = 3000; // scaled by 0.5

    let distributed = solver.distribute_widths(&min_widths, &max_widths, target);
    assert_eq!(distributed, vec![1000, 2000]);
}

// =========================================================================
// Tests 13-18: Merged Columns, Table & Cell Alignments
// =========================================================================

#[test]
fn test_13_gridspan_merged_columns_slack_distribution() {
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = TableAutoFitSolver::new(&metrics, &cascade);

    // Row 1: 2 individual cells
    // Row 2: 1 cell spanning 2 columns with a long string
    let mut cell_span = create_text_cell("Very long merged header text that requires width", 24);
    cell_span.properties = Some(TableCellProperties {
        grid_span: 2,
        ..Default::default()
    });

    let tbl = Table {
        properties: Some(TableProperties {
            layout_type: Some("autofit".to_string()),
            ..Default::default()
        }),
        rows: vec![
            TableRow {
                cells: vec![create_text_cell("A", 24), create_text_cell("B", 24)],
                ..Default::default()
            },
            TableRow {
                cells: vec![cell_span],
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let col_widths = solver.compute_column_widths(&tbl, 10000);
    assert_eq!(col_widths.len(), 2);
    // Both columns should have received excess width from the spanning cell
    assert!(col_widths[0] > 1000);
    assert!(col_widths[1] > 1000);
}

#[test]
fn test_14_table_alignment_left() {
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = TablePaginationSolver::new(&metrics, &cascade);

    let tbl = Table {
        properties: Some(TableProperties {
            width: Some(TableWidth {
                width: 4000,
                width_type: "dxa".to_string(),
            }),
            justification: Some("left".to_string()),
            ..Default::default()
        }),
        rows: vec![TableRow {
            cells: vec![create_text_cell("Col1", 24), create_text_cell("Col2", 24)],
            ..Default::default()
        }],
        ..Default::default()
    };

    let (layout, _, _) = solver.layout_table(&tbl, 1000, 1000, 15000, 1000, 8000);
    assert_eq!(layout.bounds.x, 1000);
}

#[test]
fn test_15_table_alignment_center() {
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = TablePaginationSolver::new(&metrics, &cascade);

    let tbl = Table {
        properties: Some(TableProperties {
            width: Some(TableWidth {
                width: 4000,
                width_type: "dxa".to_string(),
            }),
            justification: Some("center".to_string()),
            ..Default::default()
        }),
        rows: vec![TableRow {
            cells: vec![create_text_cell("Col1", 24), create_text_cell("Col2", 24)],
            ..Default::default()
        }],
        ..Default::default()
    };

    // available width = 8000, table width = 4000. Center offset = (8000 - 4000) / 2 = 2000.
    // col_x = 1000 -> tbl_x = 3000.
    let (layout, _, _) = solver.layout_table(&tbl, 1000, 1000, 15000, 1000, 8000);
    assert_eq!(layout.bounds.x, 3000);
}

#[test]
fn test_16_table_alignment_right() {
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = TablePaginationSolver::new(&metrics, &cascade);

    let tbl = Table {
        properties: Some(TableProperties {
            width: Some(TableWidth {
                width: 4000,
                width_type: "dxa".to_string(),
            }),
            justification: Some("right".to_string()),
            ..Default::default()
        }),
        rows: vec![TableRow {
            cells: vec![create_text_cell("Col1", 24), create_text_cell("Col2", 24)],
            ..Default::default()
        }],
        ..Default::default()
    };

    // available width = 8000, table width = 4000. Right offset = 8000 - 4000 = 4000.
    // col_x = 1000 -> tbl_x = 5000.
    let (layout, _, _) = solver.layout_table(&tbl, 1000, 1000, 15000, 1000, 8000);
    assert_eq!(layout.bounds.x, 5000);
}

#[test]
fn test_17_cell_vertical_alignment_center() {
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = TablePaginationSolver::new(&metrics, &cascade);
    let breaker = LineBreaker::new(&metrics);

    let mut cell = create_text_cell("Single Line", 24);
    cell.properties = Some(TableCellProperties {
        vertical_align: Some("center".to_string()),
        ..Default::default()
    });

    let row = TableRow {
        properties: Some(TableRowProperties {
            height: Some(2000), // explicitly large row height
            height_rule: Some("exact".to_string()),
            ..Default::default()
        }),
        cells: vec![cell],
        ..Default::default()
    };

    let col_widths = vec![4000];
    let (layout_row, _) = solver.layout_row(&row, &col_widths, 0, 1000, false, &breaker);

    let line_y = layout_row.cells[0].lines[0].bounds.y;
    // With row height 2000, line centered should be well below 1000 + top margin
    assert!(
        line_y > 1500,
        "Line should be vertically centered: y={line_y}"
    );
}

#[test]
fn test_18_cell_vertical_alignment_bottom() {
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = TablePaginationSolver::new(&metrics, &cascade);
    let breaker = LineBreaker::new(&metrics);

    let mut cell = create_text_cell("Single Line", 24);
    cell.properties = Some(TableCellProperties {
        vertical_align: Some("bottom".to_string()),
        ..Default::default()
    });

    let row = TableRow {
        properties: Some(TableRowProperties {
            height: Some(2000), // explicitly large row height
            height_rule: Some("exact".to_string()),
            ..Default::default()
        }),
        cells: vec![cell],
        ..Default::default()
    };

    let col_widths = vec![4000];
    let (layout_row, _) = solver.layout_row(&row, &col_widths, 0, 1000, false, &breaker);

    let line_bottom = layout_row.cells[0].lines[0].bounds.bottom();
    // Line bottom should be near the row bottom (1000 + 2000 = 3000)
    assert!(
        line_bottom >= 2800,
        "Line bottom should be near cell bottom: bottom={line_bottom}"
    );
}

// =========================================================================
// Tests 19-22: Cross-Page Table Row Splitting & Repeating Headers
// =========================================================================

#[test]
fn test_19_cantsplit_row_advances_to_next_page() {
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = TablePaginationSolver::new(&metrics, &cascade);

    // Row with height 800 and cantSplit=true
    let row = TableRow {
        properties: Some(TableRowProperties {
            cant_split: true,
            height: Some(800),
            height_rule: Some("exact".to_string()),
            ..Default::default()
        }),
        cells: vec![create_text_cell("No Split Content", 24)],
        ..Default::default()
    };

    let tbl = Table {
        rows: vec![row],
        ..Default::default()
    };

    // curr_y = 9500, page_content_bottom = 10000 (only 500 twips remaining, but row needs 800)
    let (layout, _, pages_spanned) = solver.layout_table(&tbl, 1000, 9500, 10000, 1440, 5000);
    assert_eq!(pages_spanned, 1);
    // Row was relocated to top of next page (1440)
    assert_eq!(layout.rows[0].bounds.y, 1440);
}

#[test]
fn test_20_splittable_row_fits_on_page() {
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = TablePaginationSolver::new(&metrics, &cascade);

    let row = TableRow {
        properties: Some(TableRowProperties {
            cant_split: false,
            height: Some(400),
            ..Default::default()
        }),
        cells: vec![create_text_cell("Fitting Content", 24)],
        ..Default::default()
    };

    let tbl = Table {
        rows: vec![row],
        ..Default::default()
    };

    // 1000 twips remaining, row needs 400 -> fits on current page
    let (layout, _, pages_spanned) = solver.layout_table(&tbl, 1000, 9000, 10000, 1440, 5000);
    assert_eq!(pages_spanned, 0);
    assert_eq!(layout.rows[0].bounds.y, 9000);
}

#[test]
fn test_21_tblheader_repeats_on_subsequent_pages() {
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = TablePaginationSolver::new(&metrics, &cascade);

    let header_row = TableRow {
        properties: Some(TableRowProperties {
            is_header: true,
            height: Some(500),
            height_rule: Some("exact".to_string()),
            ..Default::default()
        }),
        cells: vec![create_text_cell("Header Title", 24)],
        ..Default::default()
    };

    let data_row = TableRow {
        properties: Some(TableRowProperties {
            cant_split: true,
            height: Some(800),
            height_rule: Some("exact".to_string()),
            ..Default::default()
        }),
        cells: vec![create_text_cell("Data Row 1", 24)],
        ..Default::default()
    };

    let tbl = Table {
        rows: vec![header_row, data_row],
        ..Default::default()
    };

    // curr_y = 9000. Header fits (9000 to 9500).
    // Remaining = 500 twips. Data row (800) does not fit -> advances to page 2.
    let (layout, _, pages_spanned) = solver.layout_table(&tbl, 1000, 9000, 10000, 1440, 5000);
    assert_eq!(pages_spanned, 1);
    // Page 2 should have repeated header row followed by data row
    assert_eq!(layout.rows.len(), 3);
    assert!(layout.rows[0].is_header && !layout.rows[0].is_repeated_header);
    assert!(layout.rows[1].is_repeated_header);
    assert_eq!(layout.rows[1].bounds.y, 1440);
    assert_eq!(layout.rows[2].bounds.y, 1440 + 500);
}

#[test]
fn test_22_tblheader_multiple_rows_repeated() {
    let stylesheet = StyleSheet::new();
    let cascade = stylesheet.cascade_resolver();
    let metrics = StandardFontMetrics::new();
    let solver = TablePaginationSolver::new(&metrics, &cascade);

    let hdr1 = TableRow {
        properties: Some(TableRowProperties {
            is_header: true,
            height: Some(300),
            height_rule: Some("exact".to_string()),
            ..Default::default()
        }),
        cells: vec![create_text_cell("Header 1", 24)],
        ..Default::default()
    };

    let hdr2 = TableRow {
        properties: Some(TableRowProperties {
            is_header: true,
            height: Some(400),
            height_rule: Some("exact".to_string()),
            ..Default::default()
        }),
        cells: vec![create_text_cell("Header 2", 24)],
        ..Default::default()
    };

    let data = TableRow {
        properties: Some(TableRowProperties {
            cant_split: true,
            height: Some(800),
            height_rule: Some("exact".to_string()),
            ..Default::default()
        }),
        cells: vec![create_text_cell("Data", 24)],
        ..Default::default()
    };

    let tbl = Table {
        rows: vec![hdr1, hdr2, data],
        ..Default::default()
    };

    let (layout, _, pages_spanned) = solver.layout_table(&tbl, 1000, 9000, 10000, 1440, 5000);
    assert_eq!(pages_spanned, 1);
    // rows: hdr1, hdr2 on page 1; repeated hdr1, repeated hdr2, data on page 2
    assert_eq!(layout.rows.len(), 5);
    assert!(layout.rows[2].is_repeated_header);
    assert!(layout.rows[3].is_repeated_header);
}

// =========================================================================
// Tests 23-26: Floating DrawingML Anchor Wrapping Solvers
// =========================================================================

#[test]
fn test_23_drawingml_wrap_square_exclusion() {
    let mut anchor =
        FloatingAnchor::new(BoundingBox::new(2000, 1000, 3000, 2000), WrapType::Square);
    anchor.dist_left = 200;
    anchor.dist_right = 200;

    // Line at y_mid = 1500 (inside anchor vertical range 1000..3000)
    let exclusion = FloatingWrapSolver::compute_exclusion_interval(&anchor, 1500, 8000);
    assert_eq!(exclusion, Some((1800, 5200))); // 2000 - 200 to 5000 + 200

    let intervals = FloatingWrapSolver::available_text_intervals(&anchor, 1500, 0, 8000);
    assert_eq!(intervals.len(), 2);
    assert_eq!(intervals[0], (0, 1800));
    assert_eq!(intervals[1], (5200, 8000));
}

#[test]
fn test_24_drawingml_wrap_top_and_bottom_exclusion() {
    let anchor = FloatingAnchor::new(
        BoundingBox::new(2000, 1000, 3000, 2000),
        WrapType::TopAndBottom,
    );

    let exclusion = FloatingWrapSolver::compute_exclusion_interval(&anchor, 1500, 8000);
    assert_eq!(exclusion, Some((0, 8000)));

    let intervals = FloatingWrapSolver::available_text_intervals(&anchor, 1500, 0, 8000);
    assert!(
        intervals.is_empty(),
        "TopAndBottom leaves no intervals in exclusion zone"
    );
}

#[test]
fn test_25_drawingml_wrap_tight_polygon_raycast() {
    let mut anchor = FloatingAnchor::new(BoundingBox::new(1000, 1000, 4000, 4000), WrapType::Tight);
    anchor.dist_left = 0;
    anchor.dist_right = 0;

    // Diamond polygon: top (3000, 1000), right (5000, 3000), bottom (3000, 5000), left (1000, 3000)
    anchor.wrap_polygon = vec![
        Point2D::new(3000, 1000),
        Point2D::new(5000, 3000),
        Point2D::new(3000, 5000),
        Point2D::new(1000, 3000),
    ];

    // At y = 2000 (midway top half):
    // Left edge from (1000, 3000) to (3000, 1000): x = 1000 + (2000-3000)*(2000)/(-2000) = 2000
    // Right edge from (3000, 1000) to (5000, 3000): x = 3000 + (2000-1000)*(2000)/(2000) = 4000
    let exclusion = FloatingWrapSolver::compute_exclusion_interval(&anchor, 2000, 8000);
    assert_eq!(exclusion, Some((2000, 4000)));

    let intervals = FloatingWrapSolver::available_text_intervals(&anchor, 2000, 0, 8000);
    assert_eq!(intervals, vec![(0, 2000), (4000, 8000)]);
}

#[test]
fn test_26_drawingml_wrap_through_polygon_raycast() {
    let mut anchor =
        FloatingAnchor::new(BoundingBox::new(1000, 1000, 4000, 4000), WrapType::Through);
    anchor.dist_left = 100;
    anchor.dist_right = 100;

    // Triangular wedge
    anchor.wrap_polygon = vec![
        Point2D::new(3000, 1000),
        Point2D::new(5000, 5000),
        Point2D::new(1000, 5000),
    ];

    // At y = 3000 (halfway down height)
    let exclusion = FloatingWrapSolver::compute_exclusion_interval(&anchor, 3000, 8000);
    assert!(exclusion.is_some());
    let (ex_l, ex_r) = exclusion.unwrap();
    assert!(ex_l < 3000);
    assert!(ex_r > 3000);
}

// =========================================================================
// Tests 27-28: Package Integration & Manifest
// =========================================================================

#[test]
fn test_27_wordpackage_table_layout_integration() {
    let baseline_path = resolve_repo_path("specs/evidence/word/baseline.docx");
    let file = std::fs::File::open(&baseline_path).expect("Failed to open baseline.docx");
    let mut pkg = WordPackage::open(file).expect("Failed to parse baseline.docx");

    // Append a table to the document body
    let mut tbl = Table {
        properties: Some(TableProperties {
            layout_type: Some("autofit".to_string()),
            cell_margins: Some(TableCellMargins {
                top: 100,
                bottom: 100,
                left: 150,
                right: 150,
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    tbl.rows.push(TableRow {
        properties: Some(TableRowProperties {
            is_header: true,
            ..Default::default()
        }),
        cells: vec![
            create_text_cell("Product", 24),
            create_text_cell("Price", 24),
        ],
        ..Default::default()
    });

    tbl.rows.push(TableRow {
        cells: vec![
            create_text_cell("RustOffice Word", 24),
            create_text_cell("$0.00 Open Source", 24),
        ],
        ..Default::default()
    });

    pkg.document.body.elements.push(BodyElement::Table(tbl));

    let layout = pkg.layout().expect("Failed to lay out document with table");
    assert!(layout.page_count() >= 1);
    assert!(layout.total_lines >= 3);
}

#[test]
fn test_28_vectors_manifest_validation() {
    let path = resolve_repo_path("specs/evidence/word/3.2.2_vectors.json");
    let json_bytes =
        std::fs::read(&path).expect("Must read specs/evidence/word/3.2.2_vectors.json");
    let vectors: serde_json::Value =
        serde_json::from_slice(&json_bytes).expect("Valid JSON manifest");

    let count = vectors["vectors"].as_array().expect("vectors array").len();
    assert!(
        count >= 25,
        "Subphase 3.2.2 requires at least 25 test vectors, found: {}",
        count
    );

    assert_eq!(vectors["subphase_id"].as_str(), Some("3.2.2"));
    assert_eq!(
        vectors["title"].as_str(),
        Some("REA Table Auto-Fit Widths, Row Splitting & Floating Wrap Polygons")
    );
}
