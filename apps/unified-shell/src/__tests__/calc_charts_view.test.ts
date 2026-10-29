import { describe, it, expect, beforeEach } from "vitest";
import { CalcState, calcState } from "../state/calcState";
import { CalcChartOverlay } from "../components/CalcChartOverlay";
import { UnifiedShellApp } from "../main";

describe("Calc Insert & View Ribbons: Live Charts (Bar, Line, Pie), Conditional Formatting & Freeze Panes (GEC-v2.0 Subphase 10.2.2)", () => {
  let calc: CalcState;

  beforeEach(() => {
    const mockCtx = {
      fillRect: () => {},
      strokeRect: () => {},
      fillText: () => {},
      clearRect: () => {},
      beginPath: () => {},
      closePath: () => {},
      moveTo: () => {},
      lineTo: () => {},
      stroke: () => {},
      fill: () => {},
      save: () => {},
      restore: () => {},
      clip: () => {},
      scale: () => {},
      setTransform: () => {},
      measureText: (txt: string) => ({ width: txt.length * 7 }),
      setLineDash: () => {},
      font: "12px sans-serif",
      fillStyle: "#000",
      strokeStyle: "#000",
      lineWidth: 1,
      textAlign: "left",
    };
    HTMLCanvasElement.prototype.getContext = (() => mockCtx) as any;

    document.body.innerHTML = `
      <div id="app">
        <div id="titlebar"></div>
        <div id="tab-bar-container"></div>
        <div id="ribbon-container"></div>
        <div id="sub-header-container"></div>
        <div id="main-content">
          <div id="sidebar-container"></div>
          <div id="viewport-container">
            <div id="calc-grid-container"></div>
            <div id="calc-chart-overlay-container"></div>
            <div id="calc-sheet-tab-bar-container"></div>
          </div>
        </div>
        <div id="status-bar">
          <span id="status-doc-info"></span>
          <span id="status-calc-stats"></span>
          <span id="status-word-count"></span>
          <input id="zoom-slider" type="range" value="100" />
          <span id="status-zoom-level">100%</span>
        </div>
        <div id="backstage-container"></div>
        <div id="ink-layer-container"></div>
        <div id="reflow-reader-container"></div>
        <div id="row-card-inspector-container"></div>
        <div id="laser-pointer-dot"></div>
      </div>
    `;

    calc = new CalcState();
  });

  describe("1. Live Interactive Charts (Column, Bar, Line, Pie)", () => {
    it("inserts a Column Chart and extracts category and series data from a multi-column range", () => {
      // Setup sample sales table:
      // Row 0: Category | Q1 | Q2
      // Row 1: Electronics | 15000 | 18000
      // Row 2: Apparel | 8000 | 9500
      calc.cells.set("A1", { raw: "Category", computed: "Category" });
      calc.cells.set("B1", { raw: "Q1", computed: "Q1" });
      calc.cells.set("C1", { raw: "Q2", computed: "Q2" });

      calc.cells.set("A2", { raw: "Electronics", computed: "Electronics" });
      calc.cells.set("B2", { raw: "15000", computed: 15000 });
      calc.cells.set("C2", { raw: "18000", computed: 18000 });

      calc.cells.set("A3", { raw: "Apparel", computed: "Apparel" });
      calc.cells.set("B3", { raw: "8000", computed: 8000 });
      calc.cells.set("C3", { raw: "9500", computed: 9500 });

      const range = { minRow: 0, maxRow: 2, minCol: 0, maxCol: 2 };
      const chart = calc.insertChart("column", range, "Quarterly Sales");

      expect(chart).toBeDefined();
      expect(chart.type).toBe("column");
      expect(chart.title).toBe("Quarterly Sales");
      expect(calc.getCharts().length).toBe(1);

      const data = calc.getChartData(chart);
      expect(data.categories).toEqual(["Electronics", "Apparel"]);
      expect(data.series.length).toBe(2);
      expect(data.series[0].name).toBe("Q1");
      expect(data.series[0].values).toEqual([15000, 8000]);
      expect(data.series[1].name).toBe("Q2");
      expect(data.series[1].values).toEqual([18000, 9500]);
    });

    it("automatically updates chart data points when cell values in the sheet change", () => {
      calc.cells.set("A1", { raw: "Product", computed: "Product" });
      calc.cells.set("B1", { raw: "Units", computed: "Units" });
      calc.cells.set("A2", { raw: "Gadget", computed: "Gadget" });
      calc.cells.set("B2", { raw: "50", computed: 50 });

      const range = { minRow: 0, maxRow: 1, minCol: 0, maxCol: 1 };
      const chart = calc.insertChart("line", range);

      let data = calc.getChartData(chart);
      expect(data.series[0].values).toEqual([50]);

      // Edit cell in sheet
      calc.selectedCell = "B2";
      calc.commitFormula("125");

      data = calc.getChartData(chart);
      expect(data.series[0].values).toEqual([125]);
    });

    it("inserts and extracts data for Pie and Bar charts with single column ranges", () => {
      calc.cells.set("A1", { raw: "100", computed: 100 });
      calc.cells.set("A2", { raw: "200", computed: 200 });
      calc.cells.set("A3", { raw: "300", computed: 300 });

      const range = { minRow: 0, maxRow: 2, minCol: 0, maxCol: 0 };
      const pieChart = calc.insertChart("pie", range, "Budget Distribution");

      const data = calc.getChartData(pieChart);
      expect(data.series[0].values).toEqual([100, 200, 300]);
      expect(data.categories).toEqual(["Row 1", "Row 2", "Row 3"]);

      const deleted = calc.deleteChart(pieChart.id);
      expect(deleted).toBe(true);
      expect(calc.getCharts().length).toBe(0);
    });

    it("renders floating draggable chart cards and SVG elements via CalcChartOverlay", () => {
      calc.cells.set("A1", { raw: "A", computed: "A" });
      calc.cells.set("B1", { raw: "10", computed: 10 });
      calc.cells.set("A2", { raw: "B", computed: "B" });
      calc.cells.set("B2", { raw: "20", computed: 20 });

      calc.insertChart("column", { minRow: 0, maxRow: 1, minCol: 0, maxCol: 1 }, "Test Column");
      calc.insertChart("pie", { minRow: 0, maxRow: 1, minCol: 0, maxCol: 1 }, "Test Pie");

      const overlay = new CalcChartOverlay("calc-chart-overlay-container", calc);
      const container = document.getElementById("calc-chart-overlay-container")!;

      const cards = container.querySelectorAll(".calc-chart-card");
      expect(cards.length).toBe(2);

      const firstCard = cards[0];
      expect(firstCard.querySelector(".calc-chart-title")?.textContent).toContain("Test Column");
      expect(firstCard.querySelector("svg")).not.toBeNull();
      expect(firstCard.querySelector(".calc-chart-close-btn")).not.toBeNull();
      expect(firstCard.querySelector(".calc-chart-resize-handle")).not.toBeNull();

      overlay.refresh();
      expect(container.querySelectorAll(".calc-chart-card").length).toBe(2);
    });
  });

  describe("2. Conditional Formatting (Highlight Cells, Data Bars & Color Scales)", () => {
    it("applies Greater Than and Less Than rules to cells", () => {
      calc.cells.set("A1", { raw: "50", computed: 50 });
      calc.cells.set("A2", { raw: "150", computed: 150 });
      calc.cells.set("A3", { raw: "250", computed: 250 });

      const range = { minRow: 0, maxRow: 2, minCol: 0, maxCol: 0 };
      calc.addConditionalFormat({
        type: "greaterThan",
        range,
        value1: 200,
        bgColor: "#fee2e2",
        color: "#991b1b",
      });

      calc.addConditionalFormat({
        type: "lessThan",
        range,
        value1: 100,
        bgColor: "#dcfce7",
        color: "#15803d",
      });

      // A3 (250) > 200
      const styleA3 = calc.getConditionalStyleForCell("A3", 2, 0);
      expect(styleA3?.bgColor).toBe("#fee2e2");
      expect(styleA3?.color).toBe("#991b1b");

      // A1 (50) < 100
      const styleA1 = calc.getConditionalStyleForCell("A1", 0, 0);
      expect(styleA1?.bgColor).toBe("#dcfce7");
      expect(styleA1?.color).toBe("#15803d");

      // A2 (150) neither
      const styleA2 = calc.getConditionalStyleForCell("A2", 1, 0);
      expect(styleA2).toBeNull();
    });

    it("detects Duplicate Values within a range", () => {
      calc.cells.set("A1", { raw: "Alpha", computed: "Alpha" });
      calc.cells.set("A2", { raw: "Beta", computed: "Beta" });
      calc.cells.set("A3", { raw: "Alpha", computed: "Alpha" });

      const range = { minRow: 0, maxRow: 2, minCol: 0, maxCol: 0 };
      calc.addConditionalFormat({
        type: "duplicateValues",
        range,
        bgColor: "#fef3c7",
      });

      const styleA1 = calc.getConditionalStyleForCell("A1", 0, 0);
      expect(styleA1?.bgColor).toBe("#fef3c7");

      const styleA3 = calc.getConditionalStyleForCell("A3", 2, 0);
      expect(styleA3?.bgColor).toBe("#fef3c7");

      const styleA2 = calc.getConditionalStyleForCell("A2", 1, 0);
      expect(styleA2).toBeNull();
    });

    it("calculates proportional ratios for Visual Data Bars", () => {
      calc.cells.set("A1", { raw: "10", computed: 10 });
      calc.cells.set("A2", { raw: "55", computed: 55 });
      calc.cells.set("A3", { raw: "100", computed: 100 });

      const range = { minRow: 0, maxRow: 2, minCol: 0, maxCol: 0 };
      calc.addConditionalFormat({
        type: "dataBar",
        range,
        barColor: "#3b82f6",
      });

      // Min (10) -> ratio 0
      const styleA1 = calc.getConditionalStyleForCell("A1", 0, 0);
      expect(styleA1?.dataBarRatio).toBe(0);

      // Max (100) -> ratio 1
      const styleA3 = calc.getConditionalStyleForCell("A3", 2, 0);
      expect(styleA3?.dataBarRatio).toBe(1);

      // Mid (55) -> (55 - 10) / (100 - 10) = 45 / 90 = 0.5
      const styleA2 = calc.getConditionalStyleForCell("A2", 1, 0);
      expect(styleA2?.dataBarRatio).toBe(0.5);
      expect(styleA2?.barColor).toBe("#3b82f6");
    });

    it("clears conditional formatting rules selectively or completely", () => {
      const range = { minRow: 0, maxRow: 2, minCol: 0, maxCol: 0 };
      calc.addConditionalFormat({ type: "greaterThan", range, value1: 50 });
      calc.addConditionalFormat({ type: "lessThan", range, value1: 10 });

      expect(calc.getActiveSheet().conditionalFormats?.length).toBe(2);

      calc.clearConditionalFormats();
      expect(calc.getActiveSheet().conditionalFormats?.length).toBe(0);
    });
  });

  describe("3. View Tab & Freeze Panes", () => {
    it("toggles gridlines, row/column headings, and formula bar visibility", () => {
      expect(calc.getActiveSheet().showGridlines).toBe(true);
      const gridlinesOff = calc.toggleGridlines();
      expect(gridlinesOff).toBe(false);
      expect(calc.getActiveSheet().showGridlines).toBe(false);

      expect(calc.getActiveSheet().showHeadings).toBe(true);
      const headingsOff = calc.toggleHeadings();
      expect(headingsOff).toBe(false);
      expect(calc.getActiveSheet().showHeadings).toBe(false);

      expect(calc.getActiveSheet().showFormulaBar).toBe(true);
      const formulaBarOff = calc.toggleFormulaBar();
      expect(formulaBarOff).toBe(false);
      expect(calc.getActiveSheet().showFormulaBar).toBe(false);
    });

    it("locks top row with freezeTopRow and first column with freezeFirstColumn", () => {
      calc.freezeTopRow();
      expect(calc.getActiveSheet().freezePanes).toEqual({ row: 1, col: 0 });

      calc.freezeFirstColumn();
      expect(calc.getActiveSheet().freezePanes).toEqual({ row: 0, col: 1 });

      calc.unfreezePanes();
      expect(calc.getActiveSheet().freezePanes).toEqual({ row: 0, col: 0 });
    });

    it("freezes panes at active cell selection", () => {
      calc.selectedCell = "C4"; // Row 3 (0-indexed), Col 2 (0-indexed)
      calc.freezePanesAtSelection({ minRow: 3, maxRow: 3, minCol: 2, maxCol: 2 });
      expect(calc.getActiveSheet().freezePanes).toEqual({ row: 3, col: 2 });
    });
  });

  describe("4. UnifiedShellApp Ribbon Wiring for Charts, Conditional Formatting & View", () => {
    it("handles InsertChart, ConditionalFormatting, FreezePanes, and View toggles through ribbon", () => {
      const app = new UnifiedShellApp(true);
      const tabBar = app.getTabBar();
      tabBar.addTab({ id: "tab-calc-chart-test", docType: "Excel", title: "Charts.xlsx", isDirty: false });
      tabBar.switchTab("tab-calc-chart-test");

      const activeTab = tabBar.getActiveTab()!;

      // Insert Column Chart via Ribbon
      app.handleRibbonAction("InsertChart", "column");
      expect(calcState.getCharts().length).toBeGreaterThan(0);
      expect(activeTab.isDirty).toBe(true);

      // Conditional Formatting via Ribbon
      app.handleRibbonAction("ConditionalFormatting");
      expect(calcState.getActiveSheet().conditionalFormats?.length).toBeGreaterThan(0);

      // Freeze Panes via Ribbon
      app.handleRibbonAction("FreezeTopRow");
      expect(calcState.getActiveSheet().freezePanes?.row).toBe(1);

      app.handleRibbonAction("FreezeFirstColumn");
      expect(calcState.getActiveSheet().freezePanes?.col).toBe(1);

      app.handleRibbonAction("UnfreezePanes");
      expect(calcState.getActiveSheet().freezePanes).toEqual({ row: 0, col: 0 });

      // View toggles
      app.handleRibbonAction("ToggleGridlines");
      expect(calcState.getActiveSheet().showGridlines).toBe(false);

      app.handleRibbonAction("ToggleHeadings");
      expect(calcState.getActiveSheet().showHeadings).toBe(false);

      app.handleRibbonAction("ToggleFormulaBar");
      expect(calcState.getActiveSheet().showFormulaBar).toBe(false);
    });
  });
});

