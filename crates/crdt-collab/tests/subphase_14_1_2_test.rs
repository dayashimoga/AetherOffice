use crdt_collab::{CoordinateShifter, GridCellValue, GridMatrixCrdt};

#[test]
fn test_01_basic_cell_value_and_formula_storage() {
    let mut grid = GridMatrixCrdt::new(1);
    assert_eq!(grid.cell_count(), 0);

    // Set A1 (row 0, col 0) = 100
    grid.set_cell_value("Sheet1", 0, 0, GridCellValue::Number(100.0));
    // Set A2 (row 1, col 0) = 200
    grid.set_cell_value("Sheet1", 1, 0, GridCellValue::Number(200.0));
    // Set A3 (row 2, col 0) = =SUM(A1:A2)
    grid.set_cell_formula("Sheet1", 2, 0, "=SUM(A1:A2)", GridCellValue::Number(300.0));

    assert_eq!(grid.cell_count(), 3);

    let cell_a1 = grid.get_cell("Sheet1", 0, 0).expect("A1");
    assert_eq!(cell_a1.value, GridCellValue::Number(100.0));

    let cell_a3 = grid.get_cell("Sheet1", 2, 0).expect("A3");
    assert_eq!(cell_a3.formula.as_deref(), Some("=SUM(A1:A2)"));
    assert_eq!(cell_a3.value, GridCellValue::Number(300.0));
}

#[test]
fn test_02_concurrent_edits_distinct_cells_converge() {
    let mut client_a = GridMatrixCrdt::new(1);
    let mut client_b = GridMatrixCrdt::new(2);

    // Client A edits A1 = 42
    let delta_a = client_a.set_cell_value("Sheet1", 0, 0, GridCellValue::Number(42.0));

    // Client B edits B1 = "Revenue"
    let delta_b =
        client_b.set_cell_value("Sheet1", 0, 1, GridCellValue::Text("Revenue".to_string()));

    // Cross-sync
    client_a.apply_delta(delta_b).expect("A applies B");
    client_b.apply_delta(delta_a).expect("B applies A");

    assert_eq!(client_a.cell_count(), 2);
    assert_eq!(client_b.cell_count(), 2);

    assert_eq!(
        client_a.get_cell("Sheet1", 0, 0).unwrap().value,
        GridCellValue::Number(42.0)
    );
    assert_eq!(
        client_b.get_cell("Sheet1", 0, 0).unwrap().value,
        GridCellValue::Number(42.0)
    );

    assert_eq!(
        client_a.get_cell("Sheet1", 0, 1).unwrap().value,
        GridCellValue::Text("Revenue".to_string())
    );
    assert_eq!(
        client_b.get_cell("Sheet1", 0, 1).unwrap().value,
        GridCellValue::Text("Revenue".to_string())
    );
}

#[test]
fn test_03_concurrent_edits_same_cell_lww_resolution() {
    let mut c1 = GridMatrixCrdt::new(1);
    let mut c2 = GridMatrixCrdt::new(2);

    // Initial value A1 = 10
    let init = c1.set_cell_value("Sheet1", 0, 0, GridCellValue::Number(10.0));
    c2.apply_delta(init).expect("c2 init");

    // Concurrent edit: C1 writes 20, C2 writes 99
    let d1 = c1.set_cell_value("Sheet1", 0, 0, GridCellValue::Number(20.0));
    let d2 = c2.set_cell_value("Sheet1", 0, 0, GridCellValue::Number(99.0));

    // Apply in opposite orders
    c1.apply_delta(d2).expect("c1 <- c2");
    c2.apply_delta(d1).expect("c2 <- c1");

    // Both must converge to the higher OpId winner
    let v1 = &c1.get_cell("Sheet1", 0, 0).unwrap().value;
    let v2 = &c2.get_cell("Sheet1", 0, 0).unwrap().value;
    assert_eq!(v1, v2);
}

#[test]
fn test_04_row_insertion_coordinate_shifting() {
    let mut grid = GridMatrixCrdt::new(1);
    // Row 0 (A1) = 10
    grid.set_cell_value("Sheet1", 0, 0, GridCellValue::Number(10.0));
    // Row 4 (A5) = 50
    grid.set_cell_value("Sheet1", 4, 0, GridCellValue::Number(50.0));

    // Insert row at index 2
    grid.insert_row("Sheet1", 2);

    // Row 0 should be unchanged
    assert_eq!(
        grid.get_cell("Sheet1", 0, 0).unwrap().value,
        GridCellValue::Number(10.0)
    );

    // Row 4 should have shifted to Row 5 (A6)
    assert!(grid.get_cell("Sheet1", 4, 0).is_none());
    assert_eq!(
        grid.get_cell("Sheet1", 5, 0).unwrap().value,
        GridCellValue::Number(50.0)
    );
}

#[test]
fn test_05_formula_reference_rewriting_on_row_shift() {
    let shifted = CoordinateShifter::shift_formula_references("=SUM(A1:A10)", "Sheet1", true, 4, 1);
    // A1 (row 0) < 4: unchanged
    // A10 (row 9) >= 4: shifted by 1 -> row 10 (A11)
    assert_eq!(shifted, "=SUM(A1:A11)");

    let shifted_del =
        CoordinateShifter::shift_formula_references("=AVERAGE(B5:B20)", "Sheet1", true, 10, -1);
    // B5 (row 4) < 10: unchanged
    // B20 (row 19) >= 10: shifted by -1 -> row 18 (B19)
    assert_eq!(shifted_del, "=AVERAGE(B5:B19)");
}

#[test]
fn test_06_row_deletion_with_expungement() {
    let mut grid = GridMatrixCrdt::new(1);
    grid.set_cell_value("Sheet1", 0, 0, GridCellValue::Text("Header".to_string()));
    grid.set_cell_value("Sheet1", 1, 0, GridCellValue::Number(100.0));
    grid.set_cell_value("Sheet1", 2, 0, GridCellValue::Number(200.0));

    // Delete row 1 (the 100.0 row)
    grid.delete_row("Sheet1", 1);

    assert_eq!(grid.cell_count(), 2);
    // Row 0 remains Header
    assert_eq!(
        grid.get_cell("Sheet1", 0, 0).unwrap().value,
        GridCellValue::Text("Header".to_string())
    );
    // Row 2 shifts to Row 1 with value 200.0
    assert_eq!(
        grid.get_cell("Sheet1", 1, 0).unwrap().value,
        GridCellValue::Number(200.0)
    );
    assert!(grid.get_cell("Sheet1", 2, 0).is_none());
}

#[test]
fn test_07_concurrent_cell_edit_and_row_insert_convergence() {
    let mut c1 = GridMatrixCrdt::new(1);
    let mut c2 = GridMatrixCrdt::new(2);

    let init = c1.set_cell_value("Sheet1", 5, 0, GridCellValue::Number(500.0));
    c2.apply_delta(init).expect("sync init");

    // C1 writes to Row 5 (updates value to 555)
    let d1 = c1.set_cell_value("Sheet1", 5, 0, GridCellValue::Number(555.0));

    // C2 inserts a row at index 2
    let d2 = c2.insert_row("Sheet1", 2);

    // Sync
    c1.apply_delta(d2).expect("c1 applies d2");
    c2.apply_delta(d1).expect("c2 applies d1");

    // Both should have shifted to row 6 with value 555
    assert_eq!(
        c1.get_cell("Sheet1", 6, 0).unwrap().value,
        GridCellValue::Number(555.0)
    );
    assert_eq!(
        c2.get_cell("Sheet1", 6, 0).unwrap().value,
        GridCellValue::Number(555.0)
    );
}

#[test]
fn test_08_multi_sheet_isolation() {
    let mut grid = GridMatrixCrdt::new(1);
    grid.set_cell_value("Sheet1", 0, 0, GridCellValue::Text("S1".to_string()));
    grid.set_cell_value("Sheet2", 0, 0, GridCellValue::Text("S2".to_string()));

    // Row shift on Sheet1 only
    grid.insert_row("Sheet1", 0);

    // Sheet1 cell shifted to row 1
    assert_eq!(
        grid.get_cell("Sheet1", 1, 0).unwrap().value,
        GridCellValue::Text("S1".to_string())
    );
    // Sheet2 cell remains at row 0!
    assert_eq!(
        grid.get_cell("Sheet2", 0, 0).unwrap().value,
        GridCellValue::Text("S2".to_string())
    );
}

#[test]
fn test_09_export_to_excel_engine_cspg() {
    let mut grid = GridMatrixCrdt::new(1);
    grid.set_cell_value("Sheet1", 0, 0, GridCellValue::Number(123.45));
    grid.set_cell_value("Sheet1", 0, 1, GridCellValue::Text("Tax Rate".to_string()));
    grid.set_cell_value("Sheet1", 0, 2, GridCellValue::Boolean(true));
    grid.set_cell_formula("Sheet1", 1, 0, "=A1*0.2", GridCellValue::Number(24.69));

    // Export to CompressedSparsePageGrid
    let cspg = grid.to_cspg("Sheet1").expect("to_cspg");

    // Verify cell contents in CSPG
    let c0 = cspg.get_value(0, 0);
    assert!(matches!(c0, excel_engine::CellValue::Number(n) if (n - 123.45).abs() < 1e-6));

    let c1 = cspg.get_value(0, 1);
    assert!(matches!(c1, excel_engine::CellValue::InlineString(ref s) if s == "Tax Rate"));

    let f = cspg.get_formula_str(1, 0).expect("formula at A2");
    assert_eq!(f, "=A1*0.2");
}

#[test]
fn test_10_formula_ref_error_on_deleted_row() {
    let shifted = CoordinateShifter::shift_formula_references("=A5+10", "Sheet1", true, 4, -1);
    // A5 (row 4) was deleted! Should become #REF!
    assert_eq!(shifted, "=#REF!+10");
}
