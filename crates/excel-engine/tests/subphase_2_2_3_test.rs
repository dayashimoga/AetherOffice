//! Subphase 2.2.3 Test Suite: Parallel Evaluation of Core & Financial/Lookup Functions
//! Strictly conforms to GEC-v1.0 (Zero Panics in library, Typed Result, Clean-Room Standards).

use excel_engine::calc_dag::{evaluate_ast_node, CalculationDag, CellKey};
use excel_engine::formula::parse_formula;
use excel_engine::grid::{CellError, CellValue, CompressedSparsePageGrid};
use std::fs;
use std::path::PathBuf;

fn eval_expr(formula: &str, grid: &CompressedSparsePageGrid) -> CellValue {
    let ast = parse_formula(formula)
        .unwrap_or_else(|e| panic!("Failed to parse formula {formula}: {e:?}"));
    evaluate_ast_node(&ast, grid, true)
        .unwrap_or_else(|e| panic!("Failed to evaluate formula {formula}: {e:?}"))
}

// =========================================================================
// 1. FINANCIAL FUNCTIONS
// =========================================================================

#[test]
fn test_01_financial_pmt_standard() {
    let grid = CompressedSparsePageGrid::default();
    let res = eval_expr("=PMT(0.08 / 12, 10, 10000)", &grid);
    if let CellValue::Number(n) = res {
        assert!(
            (n - (-1037.0320893591544)).abs() < 1e-6,
            "PMT expected ~-1037.032, got {n}"
        );
    } else {
        panic!("Expected Number, got {res:?}");
    }
}

#[test]
fn test_02_financial_pmt_zero_rate() {
    let grid = CompressedSparsePageGrid::default();
    let res = eval_expr("=PMT(0, 10, 10000)", &grid);
    assert_eq!(res, CellValue::Number(-1000.0));
}

#[test]
fn test_03_financial_ipmt_and_ppmt_sum_to_pmt() {
    let grid = CompressedSparsePageGrid::default();
    let pmt = match eval_expr("=PMT(0.08 / 12, 10, 10000)", &grid) {
        CellValue::Number(n) => n,
        other => panic!("Expected number, got {other:?}"),
    };
    let ipmt = match eval_expr("=IPMT(0.08 / 12, 1, 10, 10000)", &grid) {
        CellValue::Number(n) => n,
        other => panic!("Expected number, got {other:?}"),
    };
    let ppmt = match eval_expr("=PPMT(0.08 / 12, 1, 10, 10000)", &grid) {
        CellValue::Number(n) => n,
        other => panic!("Expected number, got {other:?}"),
    };

    assert!((ipmt - (-66.66666666666667)).abs() < 1e-6);
    assert!(
        ((ipmt + ppmt) - pmt).abs() < 1e-10,
        "IPMT + PPMT must equal PMT"
    );
}

#[test]
fn test_04_financial_npv() {
    let grid = CompressedSparsePageGrid::default();
    let res = eval_expr("=NPV(0.1, 100, 200, 300)", &grid);
    if let CellValue::Number(n) = res {
        assert!(
            (n - 481.5927873779112).abs() < 1e-6,
            "NPV expected ~481.59, got {n}"
        );
    } else {
        panic!("Expected Number, got {res:?}");
    }
}

#[test]
fn test_05_financial_xnpv() {
    let grid = CompressedSparsePageGrid::default();
    let res = eval_expr(
        "=XNPV(0.09, {-10000, 2750, 4250, 3250, 2750}, {43101, 43160, 43221, 43282, 43343})",
        &grid,
    );
    if let CellValue::Number(n) = res {
        assert!(
            (n - 2554.550886601569).abs() < 1e-4,
            "XNPV result {n} expected ~2554.55"
        );
    } else {
        panic!("Expected Number, got {res:?}");
    }
}

#[test]
fn test_06_financial_irr_convergence() {
    let grid = CompressedSparsePageGrid::default();
    let res = eval_expr("=IRR({-10000, 3000, 4200, 6800})", &grid);
    if let CellValue::Number(r) = res {
        assert!(
            (r - 0.1633547).abs() < 1e-4,
            "IRR expected ~16.34%, got {r}"
        );
    } else {
        panic!("Expected Number, got {res:?}");
    }
}

#[test]
fn test_07_financial_xirr_convergence() {
    let grid = CompressedSparsePageGrid::default();
    let res = eval_expr(
        "=XIRR({-10000, 2750, 4250, 3250, 2750}, {40000, 40100, 40200, 40300, 40400})",
        &grid,
    );
    if let CellValue::Number(r) = res {
        assert!(
            r > 0.1 && r < 0.5,
            "XIRR expected reasonable positive return, got {r}"
        );
    } else {
        panic!("Expected Number, got {res:?}");
    }
}

// =========================================================================
// 2. DATE & TIME FUNCTIONS (EXCEL 1900 LEAP-YEAR BUG PARITY)
// =========================================================================

#[test]
fn test_08_date_1900_system_jan1() {
    let grid = CompressedSparsePageGrid::default();
    let res = eval_expr("=DATE(1900, 1, 1)", &grid);
    assert_eq!(res, CellValue::Number(1.0));
}

#[test]
fn test_09_date_1900_system_feb28() {
    let grid = CompressedSparsePageGrid::default();
    let res = eval_expr("=DATE(1900, 2, 28)", &grid);
    assert_eq!(res, CellValue::Number(59.0));
}

#[test]
fn test_10_date_1900_system_leap_year_bug_serial_60() {
    let grid = CompressedSparsePageGrid::default();
    // Excel intentionally treats 1900 as a leap year: 1900-02-29 is serial 60!
    let res = eval_expr("=DATE(1900, 2, 29)", &grid);
    assert_eq!(res, CellValue::Number(60.0));
}

#[test]
fn test_11_date_1900_system_mar1() {
    let grid = CompressedSparsePageGrid::default();
    let res = eval_expr("=DATE(1900, 3, 1)", &grid);
    assert_eq!(res, CellValue::Number(61.0));
}

#[test]
fn test_12_date_serial_year_month_day_extraction() {
    let grid = CompressedSparsePageGrid::default();
    assert_eq!(eval_expr("=YEAR(46023)", &grid), CellValue::Number(2026.0));
    assert_eq!(eval_expr("=MONTH(46023)", &grid), CellValue::Number(1.0));
    assert_eq!(eval_expr("=DAY(46023)", &grid), CellValue::Number(1.0));
}

#[test]
fn test_13_date_edate() {
    let grid = CompressedSparsePageGrid::default();
    assert_eq!(
        eval_expr("=EDATE(46023, 3)", &grid),
        CellValue::Number(46113.0)
    );
    assert_eq!(
        eval_expr("=EDATE(46023, -12)", &grid),
        CellValue::Number(45658.0)
    );
}

#[test]
fn test_14_date_eomonth() {
    let grid = CompressedSparsePageGrid::default();
    assert_eq!(
        eval_expr("=EOMONTH(46023, 0)", &grid),
        CellValue::Number(46053.0)
    );
    assert_eq!(
        eval_expr("=EOMONTH(46023, 1)", &grid),
        CellValue::Number(46081.0)
    );
}

#[test]
fn test_15_date_workday_intl() {
    let grid = CompressedSparsePageGrid::default();
    let res = eval_expr("=WORKDAY.INTL(46023, 5, 1)", &grid);
    assert_eq!(res, CellValue::Number(46030.0));
}

#[test]
fn test_16_date_networkdays_intl() {
    let grid = CompressedSparsePageGrid::default();
    let res = eval_expr("=NETWORKDAYS.INTL(46023, 46032, 1)", &grid);
    assert_eq!(res, CellValue::Number(7.0));
}

// =========================================================================
// 3. MATH & STATISTICAL FUNCTIONS
// =========================================================================

#[test]
fn test_17_math_sumifs_single_condition() {
    let grid = CompressedSparsePageGrid::default();
    let res = eval_expr(
        "=SUMIFS({10, 20, 30, 40}, {\"A\", \"B\", \"A\", \"B\"}, \"A\")",
        &grid,
    );
    assert_eq!(res, CellValue::Number(40.0));
}

#[test]
fn test_18_math_sumifs_multi_numeric_criteria() {
    let grid = CompressedSparsePageGrid::default();
    let res = eval_expr(
        "=SUMIFS({10, 20, 30, 40}, {1, 2, 3, 4}, \">=2\", {1, 2, 3, 4}, \"<=3\")",
        &grid,
    );
    assert_eq!(res, CellValue::Number(50.0));
}

#[test]
fn test_19_stat_countifs() {
    let grid = CompressedSparsePageGrid::default();
    assert_eq!(
        eval_expr("=COUNTIFS({\"A\", \"B\", \"A\", \"B\"}, \"A\")", &grid),
        CellValue::Number(2.0)
    );
    assert_eq!(
        eval_expr("=COUNTIFS({10, 20, 30, 40}, \">=25\")", &grid),
        CellValue::Number(2.0)
    );
}

#[test]
fn test_20_stat_averageifs() {
    let grid = CompressedSparsePageGrid::default();
    let res = eval_expr(
        "=AVERAGEIFS({10, 20, 30, 40}, {\"A\", \"B\", \"A\", \"B\"}, \"A\")",
        &grid,
    );
    assert_eq!(res, CellValue::Number(20.0));
}

#[test]
fn test_21_stat_subtotal() {
    let grid = CompressedSparsePageGrid::default();
    assert_eq!(
        eval_expr("=SUBTOTAL(9, {10, 20, 30, 40})", &grid),
        CellValue::Number(100.0)
    );
    assert_eq!(
        eval_expr("=SUBTOTAL(1, {10, 20, 30, 40})", &grid),
        CellValue::Number(25.0)
    );
    assert_eq!(
        eval_expr("=SUBTOTAL(4, {10, 20, 30, 40})", &grid),
        CellValue::Number(40.0)
    );
    assert_eq!(
        eval_expr("=SUBTOTAL(5, {10, 20, 30, 40})", &grid),
        CellValue::Number(10.0)
    );
}

#[test]
fn test_22_stat_aggregate() {
    let grid = CompressedSparsePageGrid::default();
    assert_eq!(
        eval_expr("=AGGREGATE(4, 6, {10, 20, 40})", &grid),
        CellValue::Number(40.0)
    );
    assert_eq!(
        eval_expr("=AGGREGATE(14, 6, {10, 50, 30, 40}, 2)", &grid),
        CellValue::Number(40.0)
    );
    assert_eq!(
        eval_expr("=AGGREGATE(15, 6, {10, 50, 30, 40}, 2)", &grid),
        CellValue::Number(30.0)
    );
}

#[test]
fn test_23_math_round_family() {
    let grid = CompressedSparsePageGrid::default();
    assert_eq!(eval_expr("=ROUND(2.15, 1)", &grid), CellValue::Number(2.2));
    assert_eq!(eval_expr("=ROUND(2.14, 1)", &grid), CellValue::Number(2.1));
    assert_eq!(
        eval_expr("=ROUNDUP(2.11, 1)", &grid),
        CellValue::Number(2.2)
    );
    assert_eq!(
        eval_expr("=ROUNDDOWN(2.19, 1)", &grid),
        CellValue::Number(2.1)
    );
}

#[test]
fn test_24_math_mround() {
    let grid = CompressedSparsePageGrid::default();
    assert_eq!(eval_expr("=MROUND(10, 3)", &grid), CellValue::Number(9.0));
    assert_eq!(eval_expr("=MROUND(11, 3)", &grid), CellValue::Number(12.0));
}

#[test]
fn test_25_stat_stdevs() {
    let grid = CompressedSparsePageGrid::default();
    let res = eval_expr("=STDEV.S({2, 4, 4, 4, 5, 5, 7, 9})", &grid);
    if let CellValue::Number(s) = res {
        assert!((s - 2.138089935299395).abs() < 1e-6);
    } else {
        panic!("Expected Number, got {res:?}");
    }
}

#[test]
fn test_26_stat_percentile_inc() {
    let grid = CompressedSparsePageGrid::default();
    assert_eq!(
        eval_expr("=PERCENTILE.INC({1, 2, 3, 4, 5}, 0.5)", &grid),
        CellValue::Number(3.0)
    );
    assert_eq!(
        eval_expr("=PERCENTILE.INC({1, 2, 3, 4, 5}, 0.25)", &grid),
        CellValue::Number(2.0)
    );
}

// =========================================================================
// 4. LOOKUP & REFERENCE FUNCTIONS
// =========================================================================

#[test]
fn test_27_lookup_xlookup_exact_and_fallback() {
    let grid = CompressedSparsePageGrid::default();
    assert_eq!(
        eval_expr(
            "=XLOOKUP(\"B\", {\"A\", \"B\", \"C\"}, {10, 20, 30})",
            &grid
        ),
        CellValue::Number(20.0)
    );
    assert_eq!(
        eval_expr(
            "=XLOOKUP(\"D\", {\"A\", \"B\", \"C\"}, {10, 20, 30}, \"NOT_FOUND\")",
            &grid
        ),
        CellValue::InlineString("NOT_FOUND".to_string())
    );
}

#[test]
fn test_28_lookup_xmatch() {
    let grid = CompressedSparsePageGrid::default();
    assert_eq!(
        eval_expr("=XMATCH(\"B\", {\"A\", \"B\", \"C\"})", &grid),
        CellValue::Number(2.0)
    );
}

#[test]
fn test_29_lookup_vlookup() {
    let grid = CompressedSparsePageGrid::default();
    assert_eq!(
        eval_expr(
            "=VLOOKUP(\"B\", {\"A\", 10; \"B\", 20; \"C\", 30}, 2, FALSE)",
            &grid
        ),
        CellValue::Number(20.0)
    );
}

#[test]
fn test_30_lookup_hlookup() {
    let grid = CompressedSparsePageGrid::default();
    assert_eq!(
        eval_expr(
            "=HLOOKUP(\"B\", {\"A\", \"B\", \"C\"; 10, 20, 30}, 2, FALSE)",
            &grid
        ),
        CellValue::Number(20.0)
    );
}

#[test]
fn test_31_lookup_index_and_match() {
    let grid = CompressedSparsePageGrid::default();
    assert_eq!(
        eval_expr("=INDEX({10, 20; 30, 40}, 2, 1)", &grid),
        CellValue::Number(30.0)
    );
    assert_eq!(
        eval_expr("=MATCH(20, {10, 20, 30}, 0)", &grid),
        CellValue::Number(2.0)
    );
}

#[test]
fn test_32_lookup_offset_and_indirect() {
    let mut grid = CompressedSparsePageGrid::default();
    grid.set_number(0, 0, 100.0).unwrap(); // A1
    grid.set_number(1, 1, 200.0).unwrap(); // B2
    grid.set_inline_string(0, 1, "B2").unwrap(); // B1 = "B2"

    assert_eq!(
        eval_expr("=OFFSET(A1, 1, 1)", &grid),
        CellValue::Number(200.0)
    );
    assert_eq!(eval_expr("=INDIRECT(B1)", &grid), CellValue::Number(200.0));
}

#[test]
fn test_33_lookup_filter_dynamic() {
    let grid = CompressedSparsePageGrid::default();
    let res = eval_expr("=FILTER({1, 10; 2, 20; 3, 30}, {1; 0; 1})", &grid);
    assert_eq!(
        res,
        CellValue::Array(vec![
            vec![CellValue::Number(1.0), CellValue::Number(10.0)],
            vec![CellValue::Number(3.0), CellValue::Number(30.0)],
        ])
    );
}

#[test]
fn test_34_lookup_unique_dynamic() {
    let grid = CompressedSparsePageGrid::default();
    let res = eval_expr("=UNIQUE({1; 2; 2; 3; 1})", &grid);
    assert_eq!(
        res,
        CellValue::Array(vec![
            vec![CellValue::Number(1.0)],
            vec![CellValue::Number(2.0)],
            vec![CellValue::Number(3.0)],
        ])
    );
}

#[test]
fn test_35_lookup_sort_dynamic() {
    let grid = CompressedSparsePageGrid::default();
    let res = eval_expr("=SORT({3; 1; 2})", &grid);
    assert_eq!(
        res,
        CellValue::Array(vec![
            vec![CellValue::Number(1.0)],
            vec![CellValue::Number(2.0)],
            vec![CellValue::Number(3.0)],
        ])
    );
}

// =========================================================================
// 5. MODERN DYNAMIC LAMBDA / LET FUNCTIONS
// =========================================================================

#[test]
fn test_36_lambda_let_basic() {
    let grid = CompressedSparsePageGrid::default();
    assert_eq!(
        eval_expr("=_xlfn.LET(x, 10, y, 20, x + y)", &grid),
        CellValue::Number(30.0)
    );
}

#[test]
fn test_37_lambda_let_chained_variables() {
    let grid = CompressedSparsePageGrid::default();
    assert_eq!(
        eval_expr("=_xlfn.LET(a, 5, b, a * 2, b + 10)", &grid),
        CellValue::Number(20.0)
    );
}

#[test]
fn test_38_lambda_reduce_sum() {
    let grid = CompressedSparsePageGrid::default();
    assert_eq!(
        eval_expr(
            "=_xlfn.REDUCE(0, {1, 2, 3, 4}, LAMBDA(acc, x, acc + x))",
            &grid
        ),
        CellValue::Number(10.0)
    );
}

#[test]
fn test_39_lambda_map() {
    let grid = CompressedSparsePageGrid::default();
    let res = eval_expr("=_xlfn.MAP({1, 2, 3}, LAMBDA(x, x * 10))", &grid);
    assert_eq!(
        res,
        CellValue::Array(vec![vec![
            CellValue::Number(10.0),
            CellValue::Number(20.0),
            CellValue::Number(30.0)
        ]])
    );
}

#[test]
fn test_40_lambda_scan() {
    let grid = CompressedSparsePageGrid::default();
    let res = eval_expr("=_xlfn.SCAN(0, {1, 2, 3}, LAMBDA(acc, x, acc + x))", &grid);
    assert_eq!(
        res,
        CellValue::Array(vec![vec![
            CellValue::Number(1.0),
            CellValue::Number(3.0),
            CellValue::Number(6.0)
        ]])
    );
}

// =========================================================================
// 6. RAYON LEVEL-PARALLEL DAG EVALUATION
// =========================================================================

#[test]
fn test_41_rayon_parallel_dag_leveled_evaluation() {
    let mut grid = CompressedSparsePageGrid::default();
    let mut dag = CalculationDag::default();

    // Populate base independent inputs: A1 = 10, A2 = 20, A3 = 30
    grid.set_number(0, 0, 10.0).unwrap();
    grid.set_number(1, 0, 20.0).unwrap();
    grid.set_number(2, 0, 30.0).unwrap();

    let _a1 = CellKey::new(1, 0, 0);
    let _a2 = CellKey::new(1, 1, 0);
    let _a3 = CellKey::new(1, 2, 0);

    // Generation 1 (all mutually independent):
    // B1 = A1 * 2 (= 20)
    // B2 = A2 * 2 (= 40)
    // B3 = A3 * 2 (= 60)
    let b1 = CellKey::new(1, 0, 1);
    let b2 = CellKey::new(1, 1, 1);
    let b3 = CellKey::new(1, 2, 1);

    grid.set_formula(0, 1, "=A1 * 2", None).unwrap();
    grid.set_formula(1, 1, "=A2 * 2", None).unwrap();
    grid.set_formula(2, 1, "=A3 * 2", None).unwrap();

    let ast_b1 = parse_formula("=A1 * 2").unwrap();
    let ast_b2 = parse_formula("=A2 * 2").unwrap();
    let ast_b3 = parse_formula("=A3 * 2").unwrap();

    dag.register_formula(b1, &ast_b1);
    dag.register_formula(b2, &ast_b2);
    dag.register_formula(b3, &ast_b3);

    // Generation 2 (dependent on Generation 1):
    // C1 = B1 + B2 + B3 (= 120)
    let c1 = CellKey::new(1, 0, 2);
    grid.set_formula(0, 2, "=B1 + B2 + B3", None).unwrap();
    let ast_c1 = parse_formula("=B1 + B2 + B3").unwrap();
    dag.register_formula(c1, &ast_c1);

    // Check DAG topological levels structure
    let levels = dag.topological_levels().expect("topological levels");
    assert_eq!(
        levels.len(),
        2,
        "Must contain exactly 2 dependency generations"
    );
    assert_eq!(
        levels[0].len(),
        3,
        "Generation 1 must contain 3 independent B cells"
    );
    assert_eq!(
        levels[1].len(),
        1,
        "Generation 2 must contain 1 dependent C cell"
    );

    // Execute parallel Rayon dirty evaluation
    let eval_count = dag.evaluate_dirty(&mut grid).expect("evaluate dirty");
    assert_eq!(eval_count, 4, "Must evaluate all 4 formula cells");

    assert_eq!(grid.get_formula_cached_value(0, 1), Some(20.0));
    assert_eq!(grid.get_formula_cached_value(1, 1), Some(40.0));
    assert_eq!(grid.get_formula_cached_value(2, 1), Some(60.0));
    assert_eq!(grid.get_formula_cached_value(0, 2), Some(120.0));
}

#[test]
fn test_42_dynamic_spill_in_dag_evaluation() {
    let mut grid = CompressedSparsePageGrid::default();
    let mut dag = CalculationDag::default();

    // A1 = SORT({3; 1; 2}) -> should set A1 = 1, and spill into A2 = 2, A3 = 3
    let a1 = CellKey::new(1, 0, 0);
    grid.set_formula(0, 0, "=SORT({3; 1; 2})", None).unwrap();
    dag.mark_dirty(a1);

    let count = dag.evaluate_dirty(&mut grid).expect("spill eval");
    assert_eq!(count, 1);

    assert_eq!(grid.get_formula_cached_value(0, 0), Some(1.0));
    assert_eq!(grid.get_value(1, 0), CellValue::Number(2.0));
    assert_eq!(grid.get_value(2, 0), CellValue::Number(3.0));
}

// =========================================================================
// 7. VECTORS MANIFEST VALIDATION (>= 50 VECTORS)
// =========================================================================

#[test]
fn test_43_vectors_manifest_validation() {
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../specs/evidence/excel/2.2.3_vectors.json");

    assert!(
        manifest_path.exists(),
        "Vectors manifest must exist at specs/evidence/excel/2.2.3_vectors.json"
    );

    let raw = fs::read_to_string(&manifest_path).expect("read vectors manifest");
    let val: serde_json::Value = serde_json::from_str(&raw).expect("parse json");

    assert_eq!(val["subphase"], "2.2.3");
    let vectors = val["vectors"].as_array().expect("vectors array");
    assert!(
        vectors.len() >= 50,
        "Subphase 2.2.3 requires >= 50 vectors, found {}",
        vectors.len()
    );

    let grid = CompressedSparsePageGrid::default();
    let mut passed = 0;

    for vec in vectors {
        let formula = vec["formula"].as_str().expect("formula");
        let id = vec["id"].as_str().expect("id");
        let ast = parse_formula(formula)
            .unwrap_or_else(|e| panic!("Failed vector {id}: {formula} - {e:?}"));
        let res = evaluate_ast_node(&ast, &grid, true)
            .unwrap_or_else(|e| panic!("Eval vector {id}: {formula} - {e:?}"));

        assert!(
            !matches!(res, CellValue::Error(CellError::Name)),
            "Vector {id} produced #NAME?: {formula}"
        );
        passed += 1;
    }

    assert_eq!(passed, vectors.len());
}
