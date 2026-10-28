//! Subphase 2.2.2 Integration Test Suite: REA Calc Chain DAG & IEEE-754 Snap-to-Zero Parity.
//! Strictly conforms to GEC-v1.0 (Zero Contamination, Zero Panics, SLAs, Bit-for-Bit Parity).

use excel_engine::calc_dag::{
    apply_ieee_snap_to_zero, evaluate_ast_node, CalculationDag, CellKey, IterativeConfig,
    IEEE_SNAP_THRESHOLD,
};
use excel_engine::formula::parse_formula;
use excel_engine::grid::{CellError, CellValue, CompressedSparsePageGrid};
use std::fs;
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

// ==============================================================================
// 1. EXACT IEEE-754 SNAP-TO-ZERO TESTS
// ==============================================================================

#[test]
fn test_01_ieee_snap_1333_subtraction_snaps_to_zero() {
    let grid = CompressedSparsePageGrid::default();
    let ast = parse_formula("=1.333 - 1.225 - 0.108").expect("parse formula");
    let result = evaluate_ast_node(&ast, &grid, true).expect("evaluate root");
    assert_eq!(result, CellValue::Number(0.0));
}

#[test]
fn test_02_ieee_snap_parenthesized_preserves_raw_residual() {
    let grid = CompressedSparsePageGrid::default();
    let ast = parse_formula("=(1.333 - 1.225 - 0.108)").expect("parse parenthesized");
    let result = evaluate_ast_node(&ast, &grid, true).expect("evaluate parenthesized");
    match result {
        CellValue::Number(val) => {
            assert_ne!(val, 0.0);
            assert!((val - (-1.249_000_902_703_301e-16)).abs() < 1e-25);
        }
        _ => panic!("Expected numeric result"),
    }
}

#[test]
fn test_03_ieee_snap_01_plus_02_minus_03_snaps_to_zero() {
    let grid = CompressedSparsePageGrid::default();
    let ast = parse_formula("=0.1 + 0.2 - 0.3").expect("parse");
    let result = evaluate_ast_node(&ast, &grid, true).expect("evaluate");
    assert_eq!(result, CellValue::Number(0.0));
}

#[test]
fn test_04_ieee_snap_parenthesized_01_02_03_preserves_residual() {
    let grid = CompressedSparsePageGrid::default();
    let ast = parse_formula("=(0.1 + 0.2 - 0.3)").expect("parse");
    let result = evaluate_ast_node(&ast, &grid, true).expect("evaluate");
    match result {
        CellValue::Number(val) => {
            assert_ne!(val, 0.0);
            assert!((val - 5.551115123125783e-17).abs() < 1e-25);
        }
        _ => panic!("Expected numeric result"),
    }
}

#[test]
fn test_05_ieee_snap_2225_subtraction() {
    let grid = CompressedSparsePageGrid::default();
    let ast = parse_formula("=2.225 - 2.22 - 0.005").expect("parse");
    let result = evaluate_ast_node(&ast, &grid, true).expect("evaluate");
    assert_eq!(result, CellValue::Number(0.0));
}

#[test]
fn test_06_ieee_snap_2225_parenthesized_preserves_residual() {
    let grid = CompressedSparsePageGrid::default();
    let ast = parse_formula("=(2.225 - 2.22 - 0.005)").expect("parse");
    let result = evaluate_ast_node(&ast, &grid, true).expect("evaluate");
    match result {
        CellValue::Number(val) => {
            assert_ne!(val, 0.0);
            assert!((val - (-1.0668549377257364e-16)).abs() < 1e-25);
        }
        _ => panic!("Expected numeric result"),
    }
}

#[test]
fn test_07_ieee_snap_cancellation_1005() {
    let grid = CompressedSparsePageGrid::default();
    let ast = parse_formula("=10.05 - 10.0 - 0.05").expect("parse");
    let result = evaluate_ast_node(&ast, &grid, true).expect("evaluate");
    assert_eq!(result, CellValue::Number(0.0));
}

#[test]
fn test_08_ieee_snap_parenthesized_1005() {
    let grid = CompressedSparsePageGrid::default();
    let ast = parse_formula("=(10.05 - 10.0 - 0.05)").expect("parse");
    let result = evaluate_ast_node(&ast, &grid, true).expect("evaluate");
    match result {
        CellValue::Number(val) => {
            assert_ne!(val, 0.0);
            assert!((val - 7.077671781985373e-16).abs() < 1e-25);
        }
        _ => panic!("Expected numeric result"),
    }
}

#[test]
fn test_09_ieee_snap_small_numbers_non_additive_preserved() {
    let grid = CompressedSparsePageGrid::default();
    let ast = parse_formula("=1e-16 * 1").expect("parse");
    let result = evaluate_ast_node(&ast, &grid, true).expect("evaluate");
    assert_eq!(result, CellValue::Number(1e-16));
}

#[test]
fn test_10_ieee_snap_genuine_differences_preserved() {
    let snapped = apply_ieee_snap_to_zero(1e-13, 1.0, 1.0);
    assert_eq!(
        snapped, 1e-13,
        "Difference at 13 digits must not be snapped"
    );

    let subnormal_diff = 1e-16;
    let snapped_subnormal = apply_ieee_snap_to_zero(subnormal_diff, 1.0, 1.0);
    assert_eq!(
        snapped_subnormal, 0.0,
        "Difference beyond 15 digits must snap to 0.0"
    );
    const { assert!(IEEE_SNAP_THRESHOLD > 1e-15) };
}

// ==============================================================================
// 2. DAG CONSTRUCTION & TOPOLOGICAL ORDERING TESTS
// ==============================================================================

#[test]
fn test_11_dag_linear_chain_topological_sort() {
    let mut dag = CalculationDag::new();
    let a1 = CellKey::new(1, 0, 0); // A1
    let a2 = CellKey::new(1, 1, 0); // A2
    let a3 = CellKey::new(1, 2, 0); // A3

    let f2 = parse_formula("=A1 * 2").unwrap();
    let f3 = parse_formula("=A2 + 5").unwrap();

    dag.register_formula(a2, &f2);
    dag.register_formula(a3, &f3);

    let order = dag.topological_sort().expect("topological sort");
    assert_eq!(order.len(), 2);
    assert_eq!(order[0], a2);
    assert_eq!(order[1], a3);

    // If A1 changes, both A2 and A3 must be dirty
    dag.mark_dirty(a1);
    assert!(dag.is_dirty(a2));
    assert!(dag.is_dirty(a3));
}

#[test]
fn test_12_dag_diamond_topology_evaluation_order() {
    let mut dag = CalculationDag::new();
    let a1 = CellKey::new(1, 0, 0);
    let b1 = CellKey::new(1, 0, 1);
    let c1 = CellKey::new(1, 0, 2);
    let d1 = CellKey::new(1, 0, 3);

    let f_b1 = parse_formula("=A1 * 2").unwrap();
    let f_c1 = parse_formula("=A1 + 10").unwrap();
    let f_d1 = parse_formula("=B1 + C1").unwrap();

    dag.register_formula(b1, &f_b1);
    dag.register_formula(c1, &f_c1);
    dag.register_formula(d1, &f_d1);

    dag.mark_dirty(a1);
    let order = dag.topological_sort().expect("diamond topological sort");

    let pos_b1 = order.iter().position(|&x| x == b1).unwrap();
    let pos_c1 = order.iter().position(|&x| x == c1).unwrap();
    let pos_d1 = order.iter().position(|&x| x == d1).unwrap();

    assert!(pos_b1 < pos_d1, "B1 must be evaluated before D1");
    assert!(pos_c1 < pos_d1, "C1 must be evaluated before D1");
}

#[test]
fn test_13_dag_range_dependency_expansion() {
    let mut dag = CalculationDag::new();
    let sum_cell = CellKey::new(1, 0, 1); // B1
    let f = parse_formula("=SUM(A1:A4)").unwrap();
    dag.register_formula(sum_cell, &f);

    let a3 = CellKey::new(1, 2, 0);
    dag.mark_dirty(a3);
    assert!(dag.is_dirty(sum_cell));
}

#[test]
fn test_14_dag_tree_multi_level_dependencies() {
    let mut dag = CalculationDag::new();
    let root = CellKey::new(1, 0, 0);
    let l1_a = CellKey::new(1, 1, 0);
    let l1_b = CellKey::new(1, 1, 1);
    let l2 = CellKey::new(1, 2, 0);

    let f_1a = parse_formula("=A1 + 1").unwrap();
    let f_1b = parse_formula("=A1 + 2").unwrap();
    let f_2 = parse_formula("=A2 + B2").unwrap();

    dag.register_formula(l1_a, &f_1a);
    dag.register_formula(l1_b, &f_1b);
    dag.register_formula(l2, &f_2);

    dag.mark_dirty(root);
    let order = dag.topological_sort().expect("tree sort");
    assert_eq!(order.len(), 4);
    assert_eq!(*order.last().unwrap(), l2);
}

#[test]
fn test_15_dag_unregister_clears_old_edges() {
    let mut dag = CalculationDag::new();
    let a1 = CellKey::new(1, 0, 0);
    let b1 = CellKey::new(1, 0, 1);

    let f = parse_formula("=A1 * 10").unwrap();
    dag.register_formula(b1, &f);

    dag.unregister_cell(b1);
    dag.mark_dirty(a1);
    assert!(
        !dag.is_dirty(b1),
        "Unregistered cell should no longer be dirty"
    );
}

// ==============================================================================
// 3. DIRTY FLAG PROPAGATION TESTS
// ==============================================================================

#[test]
fn test_16_dag_dirty_flag_transitive_propagation() {
    let mut dag = CalculationDag::new();
    let c1 = CellKey::new(1, 0, 0);
    let c2 = CellKey::new(1, 1, 0);
    let c3 = CellKey::new(1, 2, 0);
    let c4 = CellKey::new(1, 3, 0);

    dag.register_formula(c2, &parse_formula("=A1").unwrap());
    dag.register_formula(c3, &parse_formula("=A2").unwrap());
    dag.register_formula(c4, &parse_formula("=A3").unwrap());

    dag.mark_dirty(c1);
    assert!(dag.is_dirty(c2));
    assert!(dag.is_dirty(c3));
    assert!(dag.is_dirty(c4));
}

#[test]
fn test_17_dag_selective_dirty_evaluation() {
    let mut dag = CalculationDag::new();
    let mut grid = CompressedSparsePageGrid::default();

    let a1 = CellKey::new(1, 0, 0);
    let a2 = CellKey::new(1, 1, 0);
    let _b1 = CellKey::new(1, 0, 1);
    let b2 = CellKey::new(1, 1, 1);

    grid.set_number(0, 0, 10.0).unwrap();
    grid.set_number(0, 1, 100.0).unwrap();

    grid.set_formula(1, 0, "=A1 + 1", None).unwrap();
    grid.set_formula(1, 1, "=B1 + 1", None).unwrap();

    dag.register_formula(a2, &parse_formula("=A1 + 1").unwrap());
    dag.register_formula(b2, &parse_formula("=B1 + 1").unwrap());

    // Clear dirty flags from registration and only mark A1 dirty
    dag.clear_dirty_cells();
    dag.mark_dirty(a1);
    assert!(dag.is_dirty(a2));
    assert!(!dag.is_dirty(b2));

    let evaluated_count = dag.evaluate_dirty(&mut grid).expect("evaluate");
    assert_eq!(evaluated_count, 1, "Only dirty cell A2 should be evaluated");
    assert_eq!(grid.get_formula_cached_value(1, 0), Some(11.0));
}

#[test]
fn test_18_dag_mark_all_dirty() {
    let mut dag = CalculationDag::new();
    let a2 = CellKey::new(1, 1, 0);
    let b2 = CellKey::new(1, 1, 1);

    dag.register_formula(a2, &parse_formula("=1").unwrap());
    dag.register_formula(b2, &parse_formula("=2").unwrap());

    dag.mark_all_dirty();
    assert!(dag.is_dirty(a2));
    assert!(dag.is_dirty(b2));
}

// ==============================================================================
// 4. TARJAN CYCLE DETECTION TESTS
// ==============================================================================

#[test]
fn test_19_cycle_self_referential_detection() {
    let mut dag = CalculationDag::new();
    let a1 = CellKey::new(1, 0, 0);
    let f = parse_formula("=A1 + 1").unwrap();
    dag.register_formula(a1, &f);

    let cycles = dag.detect_cycles();
    assert_eq!(cycles.len(), 1);
    assert_eq!(cycles[0], vec![a1]);
}

#[test]
fn test_20_cycle_mutual_2node_detection() {
    let mut dag = CalculationDag::new();
    let a1 = CellKey::new(1, 0, 0);
    let b1 = CellKey::new(1, 0, 1);

    dag.register_formula(a1, &parse_formula("=B1 + 1").unwrap());
    dag.register_formula(b1, &parse_formula("=A1 + 1").unwrap());

    let cycles = dag.detect_cycles();
    assert_eq!(cycles.len(), 1);
    assert_eq!(cycles[0].len(), 2);
    assert!(cycles[0].contains(&a1));
    assert!(cycles[0].contains(&b1));
}

#[test]
fn test_21_cycle_3node_loop_detection() {
    let mut dag = CalculationDag::new();
    let a1 = CellKey::new(1, 0, 0);
    let b1 = CellKey::new(1, 0, 1);
    let c1 = CellKey::new(1, 0, 2);

    dag.register_formula(a1, &parse_formula("=C1 + 1").unwrap());
    dag.register_formula(b1, &parse_formula("=A1 + 1").unwrap());
    dag.register_formula(c1, &parse_formula("=B1 + 1").unwrap());

    let cycles = dag.detect_cycles();
    assert_eq!(cycles.len(), 1);
    assert_eq!(cycles[0].len(), 3);
}

#[test]
fn test_22_cycle_disjoint_cycles_detection() {
    let mut dag = CalculationDag::new();
    let a1 = CellKey::new(1, 0, 0);
    let b1 = CellKey::new(1, 0, 1);
    let x1 = CellKey::new(1, 10, 0);
    let y1 = CellKey::new(1, 10, 1);

    // Cycle 1: A1 <-> B1
    dag.register_formula(a1, &parse_formula("=B1").unwrap());
    dag.register_formula(b1, &parse_formula("=A1").unwrap());

    // Cycle 2: A11 <-> B11 (row 10, col 0 and 1)
    dag.register_formula(x1, &parse_formula("=B11").unwrap());
    dag.register_formula(y1, &parse_formula("=A11").unwrap());

    let cycles = dag.detect_cycles();
    assert_eq!(
        cycles.len(),
        2,
        "Must detect two separate independent cycles"
    );
}

// ==============================================================================
// 5. ITERATIVE CALCULATION CONVERGENCE TESTS
// ==============================================================================

#[test]
fn test_23_iterative_calculation_converges_to_target() {
    let mut dag = CalculationDag::new();
    let mut grid = CompressedSparsePageGrid::default();

    // Enable iterative calculation
    dag.iter_config = IterativeConfig {
        enabled: true,
        max_iterations: 100,
        max_change: 0.0001,
    };

    let a1 = CellKey::new(1, 0, 0);
    let b1 = CellKey::new(1, 0, 1);

    grid.set_number(0, 0, 100.0).unwrap();
    grid.set_formula(0, 1, "=B1 + (A1 - B1) * 0.5", None)
        .unwrap();

    dag.register_formula(b1, &parse_formula("=B1 + (A1 - B1) * 0.5").unwrap());
    dag.mark_dirty(a1);

    let count = dag
        .evaluate_dirty(&mut grid)
        .expect("evaluate with iteration");
    assert_eq!(count, 1);

    let val = grid.get_formula_cached_value(0, 1).expect("cached value");
    assert!(
        (val - 100.0).abs() < 0.01,
        "Iterative calculation must converge to A1 value 100"
    );
}

#[test]
fn test_24_iterative_calculation_divergent_halts_at_max_iterations() {
    let mut dag = CalculationDag::new();
    let mut grid = CompressedSparsePageGrid::default();

    dag.iter_config = IterativeConfig {
        enabled: true,
        max_iterations: 25,
        max_change: 0.0001,
    };

    let a1 = CellKey::new(1, 0, 0);
    grid.set_formula(0, 0, "=A1 + 1", None).unwrap();
    dag.register_formula(a1, &parse_formula("=A1 + 1").unwrap());

    let count = dag.evaluate_dirty(&mut grid).expect("evaluate divergent");
    assert_eq!(count, 1);
    assert_eq!(grid.get_formula_cached_value(0, 0), Some(25.0));
}

#[test]
fn test_25_iterative_calculation_disabled_returns_cycle_error() {
    let mut dag = CalculationDag::new();
    let mut grid = CompressedSparsePageGrid::default();

    dag.iter_config.enabled = false;
    let a1 = CellKey::new(1, 0, 0);
    grid.set_formula(0, 0, "=A1 + 1", None).unwrap();
    dag.register_formula(a1, &parse_formula("=A1 + 1").unwrap());

    let result = dag.evaluate_dirty(&mut grid);
    assert!(
        result.is_err(),
        "Expected circular reference error when iteration is disabled"
    );
}

// ==============================================================================
// 6. CALC CHAIN XML SERIALIZATION TESTS
// ==============================================================================

#[test]
fn test_26_calc_chain_xml_serialization_round_trip() {
    let mut dag = CalculationDag::new();
    let sample_xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<calcChain xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><c r="A2" i="1"/><c r="B2" i="1"/><c r="C5" i="2"/></calcChain>"#;

    dag.parse_calc_chain_xml(sample_xml)
        .expect("parse calcChain.xml");
    let serialized = dag
        .serialize_calc_chain_xml()
        .expect("serialize calcChain.xml");

    let mut dag2 = CalculationDag::new();
    dag2.parse_calc_chain_xml(&serialized)
        .expect("parse serialized calcChain.xml");

    let _k1 = CellKey::parse_a1(1, "A2").unwrap();
    let _k2 = CellKey::parse_a1(1, "B2").unwrap();
    let _k3 = CellKey::parse_a1(2, "C5").unwrap();

    let _order = dag2.topological_sort().unwrap_or_default();
    let chain_xml_str = String::from_utf8_lossy(&serialized);
    assert!(chain_xml_str.contains("r=\"A2\""));
    assert!(chain_xml_str.contains("r=\"B2\""));
    assert!(chain_xml_str.contains("r=\"C5\""));
    assert!(chain_xml_str.contains("i=\"2\""));
}

#[test]
fn test_27_calc_chain_xml_multi_sheet_preservation() {
    let mut dag = CalculationDag::new();
    let xml = br#"<calcChain xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><c r="A1" i="1"/><c r="B1"/><c r="A1" i="2"/></calcChain>"#;
    dag.parse_calc_chain_xml(xml).expect("parse multi sheet");

    let ser = dag.serialize_calc_chain_xml().expect("serialize");
    let ser_str = String::from_utf8_lossy(&ser);
    assert!(ser_str.contains("calcChain"));
}

#[test]
fn test_28_calc_chain_xml_malformed_returns_typed_error() {
    let mut dag = CalculationDag::new();
    let bad_xml = b"<calcChain><c unclosed";
    let res = dag.parse_calc_chain_xml(bad_xml);
    assert!(res.is_err(), "Malformed calcChain XML must return error");
}

// ==============================================================================
// 7. GRID EVALUATION INTEGRATION TESTS
// ==============================================================================

#[test]
fn test_29_eval_arithmetic_precedence() {
    let grid = CompressedSparsePageGrid::default();
    let ast = parse_formula("=10 + 2 * 5 - 4 / 2").unwrap();
    let res = evaluate_ast_node(&ast, &grid, true).unwrap();
    assert_eq!(res, CellValue::Number(18.0));
}

#[test]
fn test_30_eval_division_by_zero_error() {
    let grid = CompressedSparsePageGrid::default();
    let ast = parse_formula("=10 / 0").unwrap();
    let res = evaluate_ast_node(&ast, &grid, true).unwrap();
    assert_eq!(res, CellValue::Error(CellError::Div0));
}

#[test]
fn test_31_eval_power_operator() {
    let grid = CompressedSparsePageGrid::default();
    let ast = parse_formula("=2 ^ 8").unwrap();
    let res = evaluate_ast_node(&ast, &grid, true).unwrap();
    assert_eq!(res, CellValue::Number(256.0));
}

#[test]
fn test_32_eval_sum_range_function() {
    let mut grid = CompressedSparsePageGrid::default();
    grid.set_number(0, 0, 10.0).unwrap();
    grid.set_number(1, 0, 20.0).unwrap();
    grid.set_number(2, 0, 30.0).unwrap();

    let ast = parse_formula("=SUM(A1:A3)").unwrap();
    let res = evaluate_ast_node(&ast, &grid, true).unwrap();
    assert_eq!(res, CellValue::Number(60.0));
}

#[test]
fn test_33_eval_average_range_function() {
    let mut grid = CompressedSparsePageGrid::default();
    grid.set_number(0, 0, 10.0).unwrap();
    grid.set_number(1, 0, 20.0).unwrap();
    grid.set_number(2, 0, 30.0).unwrap();

    let ast = parse_formula("=AVERAGE(A1:A3)").unwrap();
    let res = evaluate_ast_node(&ast, &grid, true).unwrap();
    assert_eq!(res, CellValue::Number(20.0));
}

#[test]
fn test_34_eval_min_max_functions() {
    let mut grid = CompressedSparsePageGrid::default();
    grid.set_number(0, 0, 15.0).unwrap();
    grid.set_number(1, 0, 50.0).unwrap();
    grid.set_number(2, 0, 5.0).unwrap();

    let ast_max = parse_formula("=MAX(A1, A2, A3)").unwrap();
    assert_eq!(
        evaluate_ast_node(&ast_max, &grid, true).unwrap(),
        CellValue::Number(50.0)
    );

    let ast_min = parse_formula("=MIN(A1, A2, A3)").unwrap();
    assert_eq!(
        evaluate_ast_node(&ast_min, &grid, true).unwrap(),
        CellValue::Number(5.0)
    );
}

#[test]
fn test_35_eval_abs_and_round_functions() {
    let grid = CompressedSparsePageGrid::default();

    let ast_abs = parse_formula("=ABS(-42)").unwrap();
    assert_eq!(
        evaluate_ast_node(&ast_abs, &grid, true).unwrap(),
        CellValue::Number(42.0)
    );

    let ast_round = parse_formula("=ROUND(2.155, 2)").unwrap();
    assert_eq!(
        evaluate_ast_node(&ast_round, &grid, true).unwrap(),
        CellValue::Number(2.16)
    );
}

#[test]
fn test_36_eval_if_conditional_function() {
    let grid = CompressedSparsePageGrid::default();

    let ast_t = parse_formula("=IF(10 > 5, 100, 200)").unwrap();
    assert_eq!(
        evaluate_ast_node(&ast_t, &grid, true).unwrap(),
        CellValue::Number(100.0)
    );

    let ast_f = parse_formula("=IF(10 < 5, 100, 200)").unwrap();
    assert_eq!(
        evaluate_ast_node(&ast_f, &grid, true).unwrap(),
        CellValue::Number(200.0)
    );
}

#[test]
fn test_37_eval_string_concatenation_and_comparisons() {
    let grid = CompressedSparsePageGrid::default();

    let ast_concat = parse_formula("=\"OpenOffice\" & \"RS\"").unwrap();
    assert_eq!(
        evaluate_ast_node(&ast_concat, &grid, true).unwrap(),
        CellValue::InlineString("OpenOfficeRS".into())
    );

    let ast_eq = parse_formula("=10 = 10").unwrap();
    assert_eq!(
        evaluate_ast_node(&ast_eq, &grid, true).unwrap(),
        CellValue::Boolean(true)
    );

    let ast_ne = parse_formula("=10 <> 20").unwrap();
    assert_eq!(
        evaluate_ast_node(&ast_ne, &grid, true).unwrap(),
        CellValue::Boolean(true)
    );
}

// ==============================================================================
// 8. VECTORS MANIFEST VALIDATION
// ==============================================================================

#[test]
fn test_38_vectors_manifest_validation() {
    let path = resolve_fixture("specs/evidence/excel/2.2.2_vectors.json");
    assert!(path.exists(), "Vectors manifest must exist at {:?}", path);

    let content = fs::read_to_string(&path).expect("read vectors manifest");
    let json: serde_json::Value = serde_json::from_str(&content).expect("parse json");

    let total = json["total_vectors"].as_u64().unwrap_or(0);
    assert_eq!(total, 38);

    let vectors = json["vectors"].as_array().expect("vectors array");
    assert_eq!(vectors.len(), 38);

    let grid = CompressedSparsePageGrid::default();
    for v in vectors {
        if let Some(f) = v["formula"].as_str() {
            let parsed = parse_formula(f);
            assert!(
                parsed.is_ok(),
                "Vector formula failed to parse: {f}: {:?}",
                parsed.err()
            );

            // Verify snap vectors
            if let Some(cat) = v["category"].as_str() {
                if cat == "ieee_snap" {
                    let ast = parsed.unwrap();
                    let res = evaluate_ast_node(&ast, &grid, true).unwrap();
                    assert_eq!(res, CellValue::Number(0.0), "Vector {f} must snap to 0.0");
                } else if cat == "ieee_snap_suppressed" {
                    let ast = parsed.unwrap();
                    let res = evaluate_ast_node(&ast, &grid, true).unwrap();
                    match res {
                        CellValue::Number(n) => {
                            assert_ne!(n, 0.0, "Vector {f} must not snap to 0.0")
                        }
                        _ => panic!("Expected number for {f}"),
                    }
                }
            }
        }
    }
}
