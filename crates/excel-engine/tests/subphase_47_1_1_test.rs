//! Sub-Phase 47.1.1: Modern Dynamic Array & Lambda Functions Test Suite
//! Strictly conforms to GEC-v7.0 (Zero Panics, Typed Results, Clean-Room Standards).

use excel_engine::calc_dag::{evaluate_ast_node, CalculationDag, CellKey};
use excel_engine::formula::parse_formula;
use excel_engine::grid::{CellError, CellValue, CompressedSparsePageGrid};

fn eval_expr(formula: &str, grid: &CompressedSparsePageGrid) -> CellValue {
    let ast = match parse_formula(formula) {
        Ok(a) => a,
        Err(e) => panic!("Failed to parse formula {formula}: {e:?}"),
    };
    match evaluate_ast_node(&ast, grid, true) {
        Ok(v) => v,
        Err(e) => panic!("Failed to evaluate formula {formula}: {e:?}"),
    }
}

// =========================================================================
// 1. SEQUENCE & RANDARRAY
// =========================================================================

#[test]
fn test_01_sequence_vector_and_matrix() {
    let grid = CompressedSparsePageGrid::default();

    // 1D vertical sequence
    let res = eval_expr("=SEQUENCE(4)", &grid);
    assert_eq!(
        res,
        CellValue::Array(vec![
            vec![CellValue::Number(1.0)],
            vec![CellValue::Number(2.0)],
            vec![CellValue::Number(3.0)],
            vec![CellValue::Number(4.0)],
        ])
    );

    // 2D sequence with start and step
    let res2 = eval_expr("=SEQUENCE(2, 3, 10, 5)", &grid);
    assert_eq!(
        res2,
        CellValue::Array(vec![
            vec![
                CellValue::Number(10.0),
                CellValue::Number(15.0),
                CellValue::Number(20.0)
            ],
            vec![
                CellValue::Number(25.0),
                CellValue::Number(30.0),
                CellValue::Number(35.0)
            ],
        ])
    );
}

#[test]
fn test_02_sequence_errors() {
    let grid = CompressedSparsePageGrid::default();
    assert_eq!(
        eval_expr("=SEQUENCE(0)", &grid),
        CellValue::Error(CellError::Value)
    );
    assert_eq!(
        eval_expr("=SEQUENCE(-5)", &grid),
        CellValue::Error(CellError::Value)
    );
    assert_eq!(
        eval_expr("=SEQUENCE(2, -1)", &grid),
        CellValue::Error(CellError::Value)
    );
}

#[test]
fn test_03_randarray_bounds_and_types() {
    let grid = CompressedSparsePageGrid::default();

    let res = eval_expr("=RANDARRAY(3, 4, 1, 10, TRUE)", &grid);
    if let CellValue::Array(rows) = res {
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].len(), 4);
        for row in rows {
            for cell in row {
                if let CellValue::Number(n) = cell {
                    assert!((1.0..=10.0).contains(&n));
                    assert_eq!(n.fract(), 0.0);
                } else {
                    panic!("Expected Number in RANDARRAY, got {cell:?}");
                }
            }
        }
    } else {
        panic!("Expected Array result, got {res:?}");
    }

    // Min > Max error
    assert_eq!(
        eval_expr("=RANDARRAY(2, 2, 10, 5)", &grid),
        CellValue::Error(CellError::Value)
    );
}

// =========================================================================
// 2. CHOOSEROWS & CHOOSECOLS
// =========================================================================

#[test]
fn test_04_chooserows_positive_and_negative() {
    let grid = CompressedSparsePageGrid::default();

    // Positive indices
    let res = eval_expr("=CHOOSEROWS({10, 20; 30, 40; 50, 60}, 1, 3)", &grid);
    assert_eq!(
        res,
        CellValue::Array(vec![
            vec![CellValue::Number(10.0), CellValue::Number(20.0)],
            vec![CellValue::Number(50.0), CellValue::Number(60.0)],
        ])
    );

    // Negative indexing: -1 is last row (row 3), -2 is second from last (row 2)
    let res_neg = eval_expr("=CHOOSEROWS({10, 20; 30, 40; 50, 60}, -1, -2)", &grid);
    assert_eq!(
        res_neg,
        CellValue::Array(vec![
            vec![CellValue::Number(50.0), CellValue::Number(60.0)],
            vec![CellValue::Number(30.0), CellValue::Number(40.0)],
        ])
    );

    // Zero or out of bounds returns #VALUE!
    assert_eq!(
        eval_expr("=CHOOSEROWS({10, 20; 30, 40}, 0)", &grid),
        CellValue::Error(CellError::Value)
    );
    assert_eq!(
        eval_expr("=CHOOSEROWS({10, 20; 30, 40}, 5)", &grid),
        CellValue::Error(CellError::Value)
    );
}

#[test]
fn test_05_choosecols_positive_and_negative() {
    let grid = CompressedSparsePageGrid::default();

    let res = eval_expr("=CHOOSECOLS({1, 2, 3; 4, 5, 6}, 1, 3)", &grid);
    assert_eq!(
        res,
        CellValue::Array(vec![
            vec![CellValue::Number(1.0), CellValue::Number(3.0)],
            vec![CellValue::Number(4.0), CellValue::Number(6.0)],
        ])
    );

    // Negative indexing: -1 is column 3
    let res_neg = eval_expr("=CHOOSECOLS({1, 2, 3; 4, 5, 6}, -1)", &grid);
    assert_eq!(
        res_neg,
        CellValue::Array(vec![
            vec![CellValue::Number(3.0)],
            vec![CellValue::Number(6.0)],
        ])
    );
}

// =========================================================================
// 3. TOCOL & TOROW
// =========================================================================

#[test]
fn test_06_tocol_and_torow() {
    let grid = CompressedSparsePageGrid::default();

    // TOCOL row-by-row
    let res_col = eval_expr("=TOCOL({1, 2; 3, 4})", &grid);
    assert_eq!(
        res_col,
        CellValue::Array(vec![
            vec![CellValue::Number(1.0)],
            vec![CellValue::Number(2.0)],
            vec![CellValue::Number(3.0)],
            vec![CellValue::Number(4.0)],
        ])
    );

    // TOCOL scan by column
    let res_col_scan = eval_expr("=TOCOL({1, 2; 3, 4}, 0, TRUE)", &grid);
    assert_eq!(
        res_col_scan,
        CellValue::Array(vec![
            vec![CellValue::Number(1.0)],
            vec![CellValue::Number(3.0)],
            vec![CellValue::Number(2.0)],
            vec![CellValue::Number(4.0)],
        ])
    );

    // TOROW
    let res_row = eval_expr("=TOROW({1, 2; 3, 4})", &grid);
    assert_eq!(
        res_row,
        CellValue::Array(vec![vec![
            CellValue::Number(1.0),
            CellValue::Number(2.0),
            CellValue::Number(3.0),
            CellValue::Number(4.0),
        ]])
    );
}

// =========================================================================
// 4. TAKE, DROP, EXPAND & SORTBY
// =========================================================================

#[test]
fn test_07_take_and_drop() {
    let grid = CompressedSparsePageGrid::default();

    // TAKE top 2 rows, 2 columns
    let res_take = eval_expr("=TAKE({1, 2, 3; 4, 5, 6; 7, 8, 9}, 2, 2)", &grid);
    assert_eq!(
        res_take,
        CellValue::Array(vec![
            vec![CellValue::Number(1.0), CellValue::Number(2.0)],
            vec![CellValue::Number(4.0), CellValue::Number(5.0)],
        ])
    );

    // TAKE from end (last 1 row, last 2 cols)
    let res_take_end = eval_expr("=TAKE({1, 2, 3; 4, 5, 6; 7, 8, 9}, -1, -2)", &grid);
    assert_eq!(
        res_take_end,
        CellValue::Array(vec![vec![CellValue::Number(8.0), CellValue::Number(9.0)]])
    );

    // DROP first row
    let res_drop = eval_expr("=DROP({1, 2; 3, 4; 5, 6}, 1)", &grid);
    assert_eq!(
        res_drop,
        CellValue::Array(vec![
            vec![CellValue::Number(3.0), CellValue::Number(4.0)],
            vec![CellValue::Number(5.0), CellValue::Number(6.0)],
        ])
    );

    // DROP all rows -> #CALC!
    assert_eq!(
        eval_expr("=DROP({1, 2; 3, 4}, 2)", &grid),
        CellValue::Error(CellError::Calc)
    );
}

#[test]
fn test_08_expand_padding() {
    let grid = CompressedSparsePageGrid::default();

    let res = eval_expr("=EXPAND({1, 2; 3, 4}, 3, 3, 0)", &grid);
    assert_eq!(
        res,
        CellValue::Array(vec![
            vec![
                CellValue::Number(1.0),
                CellValue::Number(2.0),
                CellValue::Number(0.0)
            ],
            vec![
                CellValue::Number(3.0),
                CellValue::Number(4.0),
                CellValue::Number(0.0)
            ],
            vec![
                CellValue::Number(0.0),
                CellValue::Number(0.0),
                CellValue::Number(0.0)
            ],
        ])
    );
}

#[test]
fn test_09_sortby_secondary_keys() {
    let grid = CompressedSparsePageGrid::default();

    // Sort names based on secondary score criteria
    let res = eval_expr(
        "=SORTBY({\"Alice\"; \"Bob\"; \"Charlie\"}, {85; 95; 80}, 1)",
        &grid,
    );
    assert_eq!(
        res,
        CellValue::Array(vec![
            vec![CellValue::InlineString("Charlie".to_string())],
            vec![CellValue::InlineString("Alice".to_string())],
            vec![CellValue::InlineString("Bob".to_string())],
        ])
    );
}

// =========================================================================
// 5. ENHANCED UNIQUE, SORT, FILTER & XLOOKUP
// =========================================================================

#[test]
fn test_10_unique_by_col_and_exactly_once() {
    let grid = CompressedSparsePageGrid::default();

    // Deduplicate columns (col 1 and col 3 are identical: [1; 3])
    let res_by_col = eval_expr("=UNIQUE({1, 2, 1; 3, 4, 3}, TRUE)", &grid);
    assert_eq!(
        res_by_col,
        CellValue::Array(vec![
            vec![CellValue::Number(1.0), CellValue::Number(2.0)],
            vec![CellValue::Number(3.0), CellValue::Number(4.0)],
        ])
    );

    // Exactly once (returns only rows that appear exactly 1 time)
    let res_once = eval_expr("=UNIQUE({10; 20; 20; 30}, FALSE, TRUE)", &grid);
    assert_eq!(
        res_once,
        CellValue::Array(vec![
            vec![CellValue::Number(10.0)],
            vec![CellValue::Number(30.0)],
        ])
    );
}

#[test]
fn test_11_sort_by_col() {
    let grid = CompressedSparsePageGrid::default();

    // Sort columns ascending based on row 1
    let res = eval_expr("=SORT({30, 10, 20; 3, 1, 2}, 1, 1, TRUE)", &grid);
    assert_eq!(
        res,
        CellValue::Array(vec![
            vec![
                CellValue::Number(10.0),
                CellValue::Number(20.0),
                CellValue::Number(30.0)
            ],
            vec![
                CellValue::Number(1.0),
                CellValue::Number(2.0),
                CellValue::Number(3.0)
            ],
        ])
    );
}

#[test]
fn test_12_xlookup_2d_return_array() {
    let grid = CompressedSparsePageGrid::default();

    // Looking up "B" in column, returning row from 2D array
    let res = eval_expr(
        "=XLOOKUP(\"B\", {\"A\"; \"B\"; \"C\"}, {10, 20; 30, 40; 50, 60})",
        &grid,
    );
    assert_eq!(
        res,
        CellValue::Array(vec![vec![CellValue::Number(30.0), CellValue::Number(40.0)]])
    );
}

// =========================================================================
// 6. FIRST-CLASS LEXICAL CLOSURES (LET, LAMBDA, MAKEARRAY, BYROW, BYCOL)
// =========================================================================

#[test]
fn test_13_let_with_named_lambda_call() {
    let grid = CompressedSparsePageGrid::default();

    // Define lambda inside LET and call it
    let res = eval_expr("=LET(Double, LAMBDA(x, x * 2), Double(21))", &grid);
    assert_eq!(res, CellValue::Number(42.0));

    // Two-parameter lambda inside LET
    let res2 = eval_expr(
        "=LET(AddMult, LAMBDA(a, b, (a + b) * 2), AddMult(3, 7))",
        &grid,
    );
    assert_eq!(res2, CellValue::Number(20.0));
}

#[test]
fn test_14_makearray_computed_matrix() {
    let grid = CompressedSparsePageGrid::default();

    let res = eval_expr("=MAKEARRAY(2, 3, LAMBDA(r, c, r * 10 + c))", &grid);
    assert_eq!(
        res,
        CellValue::Array(vec![
            vec![
                CellValue::Number(11.0),
                CellValue::Number(12.0),
                CellValue::Number(13.0)
            ],
            vec![
                CellValue::Number(21.0),
                CellValue::Number(22.0),
                CellValue::Number(23.0)
            ],
        ])
    );
}

#[test]
fn test_15_byrow_and_bycol() {
    let grid = CompressedSparsePageGrid::default();

    // BYROW row sums
    let res_byrow = eval_expr("=BYROW({1, 2, 3; 4, 5, 6}, LAMBDA(row, SUM(row)))", &grid);
    assert_eq!(
        res_byrow,
        CellValue::Array(vec![
            vec![CellValue::Number(6.0)],
            vec![CellValue::Number(15.0)],
        ])
    );

    // BYCOL column sums
    let res_bycol = eval_expr("=BYCOL({1, 2, 3; 4, 5, 6}, LAMBDA(col, SUM(col)))", &grid);
    assert_eq!(
        res_bycol,
        CellValue::Array(vec![vec![
            CellValue::Number(5.0),
            CellValue::Number(7.0),
            CellValue::Number(9.0)
        ]])
    );
}

#[test]
fn test_16_map_multiple_arrays() {
    let grid = CompressedSparsePageGrid::default();

    let res = eval_expr("=MAP({1, 2, 3}, {10, 20, 30}, LAMBDA(a, b, a + b))", &grid);
    assert_eq!(
        res,
        CellValue::Array(vec![vec![
            CellValue::Number(11.0),
            CellValue::Number(22.0),
            CellValue::Number(33.0)
        ]])
    );
}

#[test]
fn test_17_isomitted_in_lambda() {
    let grid = CompressedSparsePageGrid::default();

    let res = eval_expr(
        "=LET(Check, LAMBDA(x, IF(ISOMITTED(x), 999, x)), Check(42))",
        &grid,
    );
    assert_eq!(res, CellValue::Number(42.0));
}

// =========================================================================
// 7. CALCULATION DAG SPILL EXECUTION
// =========================================================================

#[test]
fn test_18_dag_spill_sequence_into_grid() {
    let mut grid = CompressedSparsePageGrid::default();
    let mut dag = CalculationDag::default();

    let root_cell = CellKey::new(1, 0, 0); // A1
    let ast = parse_formula("=SEQUENCE(3, 2, 10, 1)").expect("parse sequence");
    dag.register_formula(root_cell, &ast);
    grid.set_formula(0, 0, "=SEQUENCE(3, 2, 10, 1)", None)
        .expect("set formula");
    dag.mark_dirty(root_cell);

    let evaluated = dag.evaluate_dirty(&mut grid).expect("evaluate dirty");
    assert_eq!(evaluated, 1);

    // Verify root cell cached value
    assert_eq!(grid.get_formula_cached_value(0, 0), Some(10.0));
    // Verify spilled cells
    assert_eq!(grid.get_value(0, 1), CellValue::Number(11.0)); // B1
    assert_eq!(grid.get_value(1, 0), CellValue::Number(12.0)); // A2
    assert_eq!(grid.get_value(1, 1), CellValue::Number(13.0)); // B2
    assert_eq!(grid.get_value(2, 0), CellValue::Number(14.0)); // A3
    assert_eq!(grid.get_value(2, 1), CellValue::Number(15.0)); // B3
}
