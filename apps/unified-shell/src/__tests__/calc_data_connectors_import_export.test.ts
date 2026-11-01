import { describe, it, expect, beforeEach } from "vitest";
import {
  CalcDataConnectorCoordinator,
  calcDataConnectorCoordinator,
} from "../state/CalcDataConnectorCoordinator";
import { calcState } from "../state/calcState";

describe("Sub-Phase 47.2.2: CalcDataConnectorCoordinator", () => {
  beforeEach(() => {
    // Clear active sheet cells
    const sheet = calcState.getActiveSheet();
    sheet.cells.clear();
  });

  it("test_01_coordinate_conversion_and_range_parsing", () => {
    const coord = new CalcDataConnectorCoordinator();

    expect(coord.coordsToA1(0, 0)).toBe("A1");
    expect(coord.coordsToA1(9, 2)).toBe("C10");
    expect(coord.coordsToA1(0, 26)).toBe("AA1");

    expect(coord.a1ToCoords("A1")).toEqual({ row: 0, col: 0 });
    expect(coord.a1ToCoords("C10")).toEqual({ row: 9, col: 2 });
    expect(coord.a1ToCoords("AA1")).toEqual({ row: 0, col: 26 });
    expect(coord.a1ToCoords("INVALID")).toBeNull();

    const range = coord.parseCellRange("B2:D5");
    expect(range).toEqual({
      startRow: 1,
      startCol: 1,
      endRow: 4,
      endCol: 3,
    });
  });

  it("test_02_import_csv_hydrates_grid", async () => {
    const csvData = "Region,Revenue,Active\nNorth,50000,true\nSouth,62000,false\n";
    const report = await calcDataConnectorCoordinator.importDataset(
      "csv",
      csvData,
      { start_row: 0, start_col: 0, write_headers: true },
      calcState
    );

    expect(report.rows_ingested).toBeGreaterThanOrEqual(2);
    expect(report.columns_ingested).toBe(3);

    const sheet = calcState.getActiveSheet();
    expect(sheet.cells.get("A1")?.computed).toBe("Region");
    expect(sheet.cells.get("B1")?.computed).toBe("Revenue");
    expect(sheet.cells.get("C1")?.computed).toBe("Active");
  });

  it("test_03_import_json_hydrates_grid", async () => {
    const jsonData = JSON.stringify([
      { SKU: "W-01", Stock: 100, Price: 19.99 },
      { SKU: "W-02", Stock: 250, Price: 34.50 },
    ]);

    const report = await calcDataConnectorCoordinator.importDataset(
      "json",
      jsonData,
      { start_row: 1, start_col: 1, write_headers: true },
      calcState
    );

    expect(report.rows_ingested).toBe(2);
    expect(report.columns_ingested).toBe(3);

    const sheet = calcState.getActiveSheet();
    // Headers at row 1 (B2, C2, D2)
    expect(sheet.cells.get("B2")?.computed).toBe("SKU");
    expect(sheet.cells.get("C2")?.computed).toBe("Stock");
    expect(sheet.cells.get("D2")?.computed).toBe("Price");
  });

  it("test_04_import_without_headers", async () => {
    const csvData = "Alpha,10\nBeta,20\n";
    const report = await calcDataConnectorCoordinator.importDataset(
      "csv",
      csvData,
      { start_row: 0, start_col: 0, write_headers: false },
      calcState
    );

    expect(report.rows_ingested).toBe(2);
    const sheet = calcState.getActiveSheet();
    // First data row placed at A1 since write_headers is false
    expect(sheet.cells.get("A1")?.computed).toBe("Alpha");
  });

  it("test_05_extract_table_from_grid_bounding_box", () => {
    calcState.setCellValue("A1", "Item");
    calcState.setCellValue("B1", "Qty");
    calcState.setCellValue("C1", "Price");

    calcState.setCellValue("A2", "Gadget");
    calcState.setCellValue("B2", "15");
    calcState.setCellValue("C2", "29.95");

    const table = calcDataConnectorCoordinator.extractTableFromGrid(
      undefined,
      true,
      calcState
    );

    expect(table.schema.fields.length).toBe(3);
    expect(table.schema.fields[0].name).toBe("Item");
    expect(table.schema.fields[1].name).toBe("Qty");
    expect(table.schema.fields[2].name).toBe("Price");

    expect(table.rows.length).toBe(1);
    expect(table.rows[0][0]).toEqual({ Text: "Gadget" });
  });

  it("test_06_export_dataset_to_csv", async () => {
    calcState.setCellValue("A1", "City");
    calcState.setCellValue("B1", "Population");
    calcState.setCellValue("A2", "Tokyo");
    calcState.setCellValue("B2", "14000000");

    const bytes = await calcDataConnectorCoordinator.exportDataset(
      "csv",
      { range: "A1:B2", hasHeaders: true },
      calcState
    );

    expect(bytes).toBeInstanceOf(Uint8Array);
    const text = new TextDecoder().decode(bytes);
    expect(text.length).toBeGreaterThan(0);
  });

  it("test_07_export_dataset_to_json", async () => {
    calcState.setCellValue("A1", "Code");
    calcState.setCellValue("B1", "Status");
    calcState.setCellValue("A2", "200");
    calcState.setCellValue("B2", "OK");

    const bytes = await calcDataConnectorCoordinator.exportDataset(
      "json",
      { range: "A1:B2", hasHeaders: true },
      calcState
    );

    expect(bytes).toBeInstanceOf(Uint8Array);
    expect(bytes.length).toBeGreaterThan(0);
  });

  it("test_08_save_export_to_file", async () => {
    calcState.setCellValue("A1", "Key");
    calcState.setCellValue("B1", "Val");
    calcState.setCellValue("A2", "Host");
    calcState.setCellValue("B2", "localhost");

    const filePath = await calcDataConnectorCoordinator.saveExportToFile(
      "csv",
      "export_test.csv",
      { range: "A1:B2" },
      calcState
    );

    expect(typeof filePath).toBe("string");
    expect(filePath.length).toBeGreaterThan(0);
  });

  it("test_09_unwrap_data_value_variants", () => {
    const coord = new CalcDataConnectorCoordinator();

    expect(coord.unwrapDataValueToString(null)).toBe("");
    expect(coord.unwrapDataValueToString(undefined as any)).toBe("");
    expect(coord.unwrapDataValueToString("plain")).toBe("plain");
    expect(coord.unwrapDataValueToString(123)).toBe("123");
    expect(coord.unwrapDataValueToString(true)).toBe("true");

    expect(coord.unwrapDataValueToString({ Text: "Hello" })).toBe("Hello");
    expect(coord.unwrapDataValueToString({ Int64: 42 })).toBe("42");
    expect(coord.unwrapDataValueToString({ Float64: 3.1415 })).toBe("3.1415");
    expect(coord.unwrapDataValueToString({ Boolean: true })).toBe("TRUE");
    expect(coord.unwrapDataValueToString({ Boolean: false })).toBe("FALSE");
    expect(coord.unwrapDataValueToString({ Date: 19800 })).toBe("19800");
    expect(coord.unwrapDataValueToString({ DateTime: 1710000000 })).toBe("1710000000");
    expect(coord.unwrapDataValueToString({ Binary: [1, 2, 3] })).toBe("<BLOB 3 bytes>");
  });

  it("test_10_empty_sheet_graceful_handling", async () => {
    const sheet = calcState.getActiveSheet();
    sheet.cells.clear();

    const table = calcDataConnectorCoordinator.extractTableFromGrid(
      undefined,
      true,
      calcState
    );
    expect(table.schema.fields.length).toBe(0);
    expect(table.rows.length).toBe(0);

    const bytes = await calcDataConnectorCoordinator.exportDataset(
      "csv",
      undefined,
      calcState
    );
    expect(bytes).toBeInstanceOf(Uint8Array);
  });
});

