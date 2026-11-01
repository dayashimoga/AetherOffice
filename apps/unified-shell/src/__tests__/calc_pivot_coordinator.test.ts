import { describe, it, expect, beforeEach } from "vitest";
import {
  CalcPivotCoordinator,
  PivotConfigDto,
  TableDataset,
} from "../state/CalcPivotCoordinator";
import { calcState } from "../state/calcState";

describe("CalcPivotCoordinator (Sub-Phase 47.2.1)", () => {
  beforeEach(() => {
    const sheet = calcState.getActiveSheet();
    sheet.cells.clear();
    sheet.charts = [];
  });

  it("extracts structured table datasets and infers schema types from grid cells", () => {
    // Populate headers at row 1 (A1..E1)
    calcState.setCellValue("A1", "Region");
    calcState.setCellValue("B1", "Category");
    calcState.setCellValue("C1", "Date");
    calcState.setCellValue("D1", "Sales");
    calcState.setCellValue("E1", "Quantity");

    // Populate rows 2..4
    calcState.setCellValue("A2", "North");
    calcState.setCellValue("B2", "Electronics");
    calcState.setCellValue("C2", "2026-01-15");
    calcState.setCellValue("D2", "1000");
    calcState.setCellValue("E2", "2");

    calcState.setCellValue("A3", "South");
    calcState.setCellValue("B3", "Furniture");
    calcState.setCellValue("C3", "2026-02-20");
    calcState.setCellValue("D3", "500");
    calcState.setCellValue("E3", "5");

    calcState.setCellValue("A4", "North");
    calcState.setCellValue("B4", "Furniture");
    calcState.setCellValue("C4", "2026-03-10");
    calcState.setCellValue("D4", "800");
    calcState.setCellValue("E4", "1");

    const coordinator = new CalcPivotCoordinator();
    const dataset = coordinator.extractDatasetFromGrid(calcState, "A1:E4", true);

    expect(dataset.headers).toEqual(["Region", "Category", "Date", "Sales", "Quantity"]);
    expect(dataset.types).toEqual(["Utf8", "Utf8", "Date", "Float64", "Float64"]);
    expect(dataset.records.length).toBe(3);
    expect(dataset.records[0]).toEqual(["North", "Electronics", "2026-01-15", 1000, 2]);
  });

  it("collects distinct sorted unique values for interactive slicers", () => {
    const dataset: TableDataset = {
      headers: ["Region", "Category", "Sales"],
      types: ["Utf8", "Utf8", "Float64"],
      records: [
        ["West", "Electronics", 500],
        ["North", "Furniture", 200],
        ["South", "Electronics", 300],
        ["North", "Electronics", 400],
      ],
    };

    const coordinator = new CalcPivotCoordinator();
    const unique = coordinator.collectUniqueValues(dataset);

    expect(unique["Region"]).toEqual(["North", "South", "West"]);
    expect(unique["Category"]).toEqual(["Electronics", "Furniture"]);
    expect(unique["Sales"]).toEqual(["200", "300", "400", "500"]);
  });

  it("computes multi-dimensional OLAP aggregations with row and column dimensions", async () => {
    const dataset: TableDataset = {
      headers: ["Region", "Category", "Sales", "Units"],
      types: ["Utf8", "Utf8", "Float64", "Float64"],
      records: [
        ["North", "Electronics", 1000, 2],
        ["North", "Furniture", 500, 1],
        ["South", "Electronics", 750, 3],
        ["South", "Furniture", 250, 5],
        ["North", "Electronics", 1200, 4],
      ],
    };

    const config: PivotConfigDto = {
      rowFields: [{ sourceColumn: "Region" }],
      colFields: [{ sourceColumn: "Category" }],
      valueFields: [{ sourceColumn: "Sales", aggregation: "Sum" }],
      filterFields: [],
      grandTotals: "Both",
      showSubtotals: true,
    };

    const coordinator = new CalcPivotCoordinator();
    const result = await coordinator.computePivotTable(dataset, config, 0, 0);

    expect(result.row_headers).toEqual([["North"], ["South"]]);
    expect(result.col_headers).toEqual([["Electronics"], ["Furniture"]]);

    // North Electronics: 1000 + 1200 = 2200
    const northElec = result.cells.find((c) => c.row === 1 && c.col === 1);
    expect(northElec?.value).toBe(2200);

    // North Furniture: 500
    const northFurn = result.cells.find((c) => c.row === 1 && c.col === 2);
    expect(northFurn?.value).toBe(500);

    // North Row Total: 2700
    const northTotal = result.cells.find((c) => c.row === 1 && c.col === 3);
    expect(northTotal?.value).toBe(2700);
    expect(northTotal?.is_grand_total).toBe(true);

    // Grand Total: 2200 + 500 + 750 + 250 = 3700
    expect(result.grand_totals[0]).toBe(3700);
  });

  it("supports automatic date hierarchy grouping by Quarter", async () => {
    const dataset: TableDataset = {
      headers: ["Date", "Amount"],
      types: ["Date", "Float64"],
      records: [
        ["2026-01-10", 100],
        ["2026-02-15", 200],
        ["2026-05-20", 400],
      ],
    };

    const config: PivotConfigDto = {
      rowFields: [{ sourceColumn: "Date", dateGrouping: "Quarter" }],
      colFields: [],
      valueFields: [{ sourceColumn: "Amount", aggregation: "Sum" }],
      filterFields: [],
      grandTotals: "Both",
      showSubtotals: false,
    };

    const coordinator = new CalcPivotCoordinator();
    const result = await coordinator.computePivotTable(dataset, config, 0, 0);

    expect(result.row_headers).toEqual([["Q1"], ["Q2"]]);
    // Q1 Amount = 100 + 200 = 300
    const q1Cell = result.cells.find((c) => c.row === 1 && c.col === 1);
    expect(q1Cell?.value).toBe(300);

    // Q2 Amount = 400
    const q2Cell = result.cells.find((c) => c.row === 2 && c.col === 1);
    expect(q2Cell?.value).toBe(400);
  });

  it("applies interactive slicer filtering to dataset records", async () => {
    const dataset: TableDataset = {
      headers: ["Region", "Category", "Sales"],
      types: ["Utf8", "Utf8", "Float64"],
      records: [
        ["North", "Electronics", 1000],
        ["North", "Furniture", 500],
        ["South", "Electronics", 750],
        ["South", "Furniture", 250],
      ],
    };

    const config: PivotConfigDto = {
      rowFields: [{ sourceColumn: "Category" }],
      colFields: [],
      valueFields: [{ sourceColumn: "Sales", aggregation: "Sum" }],
      filterFields: [{ sourceColumn: "Region", includedValues: ["South"] }],
      grandTotals: "Both",
      showSubtotals: false,
    };

    const coordinator = new CalcPivotCoordinator();
    const result = await coordinator.computePivotTable(dataset, config, 0, 0);

    // Only South records present: Electronics: 750, Furniture: 250 -> Total 1000
    expect(result.grand_totals[0]).toBe(1000);
    expect(result.row_headers.length).toBe(2);
  });

  it("renders evaluated pivot table headers and values directly into calcState", async () => {
    const dataset: TableDataset = {
      headers: ["Region", "Sales"],
      types: ["Utf8", "Float64"],
      records: [
        ["North", 1000],
        ["South", 500],
      ],
    };

    const config: PivotConfigDto = {
      rowFields: [{ sourceColumn: "Region" }],
      colFields: [],
      valueFields: [{ sourceColumn: "Sales", aggregation: "Sum" }],
      filterFields: [],
      grandTotals: "Both",
      showSubtotals: false,
    };

    const coordinator = new CalcPivotCoordinator();
    const result = await coordinator.computePivotTable(dataset, config, 5, 2); // Start at C6

    coordinator.renderPivotTableToState(calcState, result, config);

    // Title at C6
    expect(calcState.getCellValue("C6")).toContain("Region");
    // Row 1: North at C7, value 1000 at D7
    expect(calcState.getCellValue("C7")).toBe("North");
    expect(Number(calcState.getCellValue("D7"))).toBe(1000);

    // Row 2: South at C8, value 500 at D8
    expect(calcState.getCellValue("C8")).toBe("South");
    expect(Number(calcState.getCellValue("D8"))).toBe(500);

    // Grand total row at C9
    expect(calcState.getCellValue("C9")).toBe("Grand Total");
    expect(Number(calcState.getCellValue("D9"))).toBe(1500);
  });

  it("synchronizes linked PivotCharts with updated pivot table categories and series", async () => {
    const chart = calcState.insertChart("column", { minRow: 0, maxRow: 5, minCol: 0, maxCol: 5 }, "Sales Chart");

    const dataset: TableDataset = {
      headers: ["Region", "Sales"],
      types: ["Utf8", "Float64"],
      records: [
        ["North", 1000],
        ["South", 500],
      ],
    };

    const config: PivotConfigDto = {
      rowFields: [{ sourceColumn: "Region" }],
      colFields: [],
      valueFields: [{ sourceColumn: "Sales", aggregation: "Sum" }],
      filterFields: [],
      grandTotals: "Both",
      showSubtotals: false,
    };

    const coordinator = new CalcPivotCoordinator();
    const result = await coordinator.computePivotTable(dataset, config, 0, 0);

    coordinator.syncPivotChart(calcState, chart.id, result);

    const anyChart = chart as any;
    expect(anyChart.categories).toEqual(["North", "South"]);
    expect(anyChart.series[0].name).toBe("Measure 1");
    expect(anyChart.series[0].data).toEqual([1000, 500]);
  });
});
