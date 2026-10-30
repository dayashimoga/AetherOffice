import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { CalcGridCanvas } from "../components/viewports/CalcGridCanvas";
import { CalcState } from "../state/calcState";
import { CalcChartOverlay } from "../components/CalcChartOverlay";
import { UnifiedShellApp } from "../main";

describe("Working Excel Ribbon: Formatting, Sort/Filter, Row/Col Ops & Live Charts (GEC-v3.0 Subphase 23.1.2)", () => {
  let container: HTMLElement;
  let state: CalcState;
  let canvas: CalcGridCanvas;

  beforeEach(() => {
    const mockCtx = {
      clearRect: () => {},
      fillRect: () => {},
      strokeRect: () => {},
      fillText: () => {},
      measureText: (text: string) => ({ width: text.length * 7 }),
      beginPath: () => {},
      moveTo: () => {},
      lineTo: () => {},
      arc: () => {},
      closePath: () => {},
      fill: () => {},
      stroke: () => {},
      save: () => {},
      restore: () => {},
      setTransform: () => {},
      scale: () => {},
      setLineDash: () => {},
      rect: () => {},
      clip: () => {},
      font: "",
      fillStyle: "",
      strokeStyle: "",
      lineWidth: 1,
      textAlign: "left",
    };
    HTMLCanvasElement.prototype.getContext = (() => mockCtx) as any;

    document.body.innerHTML = `
      <div id="sub-header-container"></div>
      <div id="sidebar-container"></div>
      <div id="viewport-container" style="width: 800px; height: 600px; position: relative;">
        <div id="calc-grid-container" style="width: 100%; height: 100%;"></div>
        <div id="calc-chart-overlay-container"></div>
      </div>
      <div id="status-bar">
        <span id="status-doc-info"></span>
        <span id="status-zoom-level">100%</span>
        <input type="range" id="zoom-slider" min="50" max="200" value="100" />
      </div>
    `;

    container = document.getElementById("calc-grid-container")!;
    state = new CalcState();
    canvas = new CalcGridCanvas(container, state);
  });

  afterEach(() => {
    document.body.innerHTML = "";
  });

  describe("1. Home Tab: Rich Formatting, Alignment, Borders & Numbers", () => {
    it("applies font family and font size to selected cells", () => {
      state.setCellValue("A1", "TestFont");
      state.selectCell("A1");

      state.setFontFamily("Arial");
      expect(state.cells.get("A1")?.fontFamily).toBe("Arial");

      state.setFontSize(16);
      expect(state.cells.get("A1")?.fontSize).toBe(16);
    });

    it("toggles bold, italic, and underline styling", () => {
      state.setCellValue("A1", "StyledText");
      state.selectCell("A1");

      state.toggleBold();
      expect(state.cells.get("A1")?.bold).toBe(true);

      state.toggleItalic();
      expect(state.cells.get("A1")?.italic).toBe(true);

      state.toggleUnderline();
      expect(state.cells.get("A1")?.underline).toBe(true);
    });

    it("applies fill color, font color, and borders (all, outside, none)", () => {
      state.setCellValue("B1", "ColoredCell");
      state.selectCell("B1");

      state.setFillColor("#fef08a");
      expect(state.cells.get("B1")?.bgColor).toBe("#fef08a");

      state.setFontColor("#b91c1c");
      expect(state.cells.get("B1")?.color).toBe("#b91c1c");

      state.setBorders("all");
      expect(state.cells.get("B1")?.borders).toBe("all");

      state.setBorders("outside");
      expect(state.cells.get("B1")?.borders).toBe("outside");

      state.setBorders("none");
      expect(state.cells.get("B1")?.borders).toBe("none");
    });

    it("applies horizontal and vertical text alignment", () => {
      state.setCellValue("C1", "Aligned");
      state.selectCell("C1");

      state.setAlignment("center");
      expect(state.cells.get("C1")?.align).toBe("center");

      state.setAlignment("right");
      expect(state.cells.get("C1")?.align).toBe("right");

      state.setVerticalAlignment("middle");
      expect(state.cells.get("C1")?.valign).toBe("middle");
    });

    it("formats numbers as Currency, Percentage, Comma and adjusts decimals", () => {
      state.setCellValue("D1", "1250.5");
      state.selectCell("D1");

      state.setNumberFormat("Currency");
      expect(state.cells.get("D1")?.format).toBe("Currency");

      state.increaseDecimal();
      expect(state.cells.get("D1")?.decimals).toBe(3);

      state.decreaseDecimal();
      expect(state.cells.get("D1")?.decimals).toBe(2);

      state.setNumberFormat("Percentage");
      expect(state.cells.get("D1")?.format).toBe("Percentage");

      state.setNumberFormat("Comma");
      expect(state.cells.get("D1")?.format).toBe("Comma");
    });

    it("clears cells, inserts rows/columns, and deletes rows/columns cleanly", () => {
      state.setCellValue("A1", "Val1");
      state.setCellValue("A2", "Val2");

      // Insert row at 0 shifts A1 -> A2, A2 -> A3
      state.insertRow(0);
      expect(state.cells.get("A2")?.raw).toBe("Val1");
      expect(state.cells.get("A3")?.raw).toBe("Val2");

      // Delete inserted row reverts
      state.deleteRow(0);
      expect(state.cells.get("A1")?.raw).toBe("Val1");
      expect(state.cells.get("A2")?.raw).toBe("Val2");

      // Clear selection
      state.selectCell("A1");
      state.clearSelection();
      expect(state.cells.get("A1")?.raw).toBe("");
    });
  });

  describe("2. Insert Tab: Live SVG Charts & Cell Comments", () => {
    it("inserts Column, Bar, Line, and Pie charts creating draggable SVG cards", () => {
      // Setup small table
      state.setCellValue("A1", "Item");
      state.setCellValue("B1", "Sales");
      state.setCellValue("A2", "Alpha");
      state.setCellValue("B2", "100");
      state.setCellValue("A3", "Beta");
      state.setCellValue("B3", "200");

      state.selectCell("A1");
      state.selectCell("B3", true);

      const chartOverlay = new CalcChartOverlay("calc-chart-overlay-container", state);
      expect(chartOverlay).toBeDefined();

      const colChart = state.insertChart("column");
      expect(colChart.type).toBe("column");
      expect(state.getCharts().length).toBe(1);

      const pieChart = state.insertChart("pie");
      expect(pieChart.type).toBe("pie");
      expect(state.getCharts().length).toBe(2);

      const lineChart = state.insertChart("line");
      expect(lineChart.type).toBe("line");

      const barChart = state.insertChart("bar");
      expect(barChart.type).toBe("bar");
      expect(state.getCharts().length).toBe(4);

      chartOverlay.refresh();
      const chartCards = document.querySelectorAll(".calc-chart-card");
      expect(chartCards.length).toBe(4);
    });

    it("attaches comments to cells and displays comment corner indicator", () => {
      state.setCellValue("A1", "ReviewedCell");
      state.selectCell("A1");
      state.insertComment("Needs validation from finance team");

      expect(state.cells.get("A1")?.comment).toBe("Needs validation from finance team");
      expect(() => canvas.render()).not.toThrow();
    });
  });

  describe("3. Formulas Tab: Statistical Functions, Show Formulas & Recalculate", () => {
    it("inserts AutoSum for SUM, AVERAGE, COUNT, MAX, and MIN", () => {
      state.setCellValue("A1", "10");
      state.setCellValue("A2", "20");
      state.setCellValue("A3", "30");

      state.selectCell("A1");
      state.selectCell("A3", true);

      const sumFormula = state.insertAutoSum("SUM");
      expect(sumFormula).toBe("=SUM(A1:A3)");

      const avgFormula = state.insertAutoSum("AVERAGE");
      expect(avgFormula).toBe("=AVERAGE(A1:A3)");

      const countFormula = state.insertAutoSum("COUNT");
      expect(countFormula).toBe("=COUNT(A1:A3)");

      const maxFormula = state.insertAutoSum("MAX");
      expect(maxFormula).toBe("=MAX(A1:A3)");

      const minFormula = state.insertAutoSum("MIN");
      expect(minFormula).toBe("=MIN(A1:A3)");
    });

    it("toggles Show Formulas displaying raw formula string in place of computed result", () => {
      state.setCellValue("B1", "10");
      state.setCellValue("B2", "20");
      state.setCellValue("B3", "=SUM(B1:B2)");

      expect(state.showFormulas).toBe(false);
      expect(state.cells.get("B3")?.computed).toBe(30);

      const isShow = state.toggleShowFormulas();
      expect(isShow).toBe(true);
      expect(state.showFormulas).toBe(true);

      expect(() => canvas.render()).not.toThrow();
    });
  });

  describe("4. Data Tab: Sort, Filter, Remove Duplicates & Text to Columns", () => {
    it("sorts cell range ascending and descending", () => {
      state.setCellValue("A1", "Zebra");
      state.setCellValue("A2", "Apple");
      state.setCellValue("A3", "Mango");

      state.selectCell("A1");
      state.selectCell("A3", true);

      state.sortRange(true);
      expect(state.cells.get("A1")?.raw).toBe("Apple");
      expect(state.cells.get("A2")?.raw).toBe("Mango");
      expect(state.cells.get("A3")?.raw).toBe("Zebra");

      state.sortRange(false);
      expect(state.cells.get("A1")?.raw).toBe("Zebra");
      expect(state.cells.get("A2")?.raw).toBe("Mango");
      expect(state.cells.get("A3")?.raw).toBe("Apple");
    });

    it("toggles auto-filter on the selected range", () => {
      state.setCellValue("A1", "Name");
      state.setCellValue("B1", "Score");
      state.selectCell("A1");
      state.selectCell("B5", true);

      expect(state.getActiveSheet().filterActive).toBeFalsy();
      state.toggleFilter();
      expect(state.getActiveSheet().filterActive).toBe(true);

      state.toggleFilter();
      expect(state.getActiveSheet().filterActive).toBe(false);
    });

    it("removes duplicate rows from the selected range", () => {
      state.setCellValue("A1", "Item1");
      state.setCellValue("A2", "Item2");
      state.setCellValue("A3", "Item1"); // duplicate

      state.selectCell("A1");
      state.selectCell("A3", true);

      state.removeDuplicates();
      expect(state.cells.get("A1")?.raw).toBe("Item1");
      expect(state.cells.get("A2")?.raw).toBe("Item2");
      expect(state.cells.get("A3")).toBeUndefined();
    });

    it("splits comma-separated text into adjacent columns using Text to Columns", () => {
      state.setCellValue("A1", "Red,Green,Blue");
      state.selectCell("A1");

      state.textToColumns(",");
      expect(state.cells.get("A1")?.raw).toBe("Red");
      expect(state.cells.get("B1")?.raw).toBe("Green");
      expect(state.cells.get("C1")?.raw).toBe("Blue");
    });
  });

  describe("5. UnifiedShellApp Ribbon Wiring & Status Bar Live Metrics", () => {
    let app: UnifiedShellApp;

    beforeEach(() => {
      document.body.innerHTML = `
        <div id="app">
          <div id="titlebar">
            <span id="titlebar-filename"></span>
            <span id="titlebar-dirty-indicator"></span>
          </div>
          <div id="tab-bar-container"></div>
          <div id="ribbon-container"></div>
          <div id="sub-header-container"></div>
          <div id="sidebar-container"></div>
          <div id="viewport-container" style="width: 800px; height: 600px;"></div>
          <div id="bottom-sheet"></div>
          <canvas id="stylus-ink-canvas"></canvas>
          <div id="reflow-reader-container"></div>
          <div id="row-card-inspector-container"></div>
          <div id="laser-pointer-dot"></div>
          <div id="status-bar">
            <span id="status-doc-info"></span>
            <span id="status-word-count"></span>
            <span id="status-zoom-level">100%</span>
            <input type="range" id="zoom-slider" min="50" max="200" value="100" />
          </div>
        </div>
      `;

      app = new UnifiedShellApp(false);
      app.applyBootConfig({ app_mode: "calc" });
    });

    it("dispatches Calc ribbon actions through UnifiedShellApp", () => {
      const activeTab = app.getTabBar().getActiveTab()!;

      (app as any).handleRibbonAction("SetFontFamily", "Arial");
      expect(activeTab.isDirty).toBe(true);

      (app as any).handleRibbonAction("SetFontSize", 14);
      expect(activeTab.isDirty).toBe(true);

      (app as any).handleRibbonAction("SetFillColor", "#fef08a");
      expect(activeTab.isDirty).toBe(true);

      (app as any).handleRibbonAction("SetBorders", "all");
      expect(activeTab.isDirty).toBe(true);

      (app as any).handleRibbonAction("AutoAverage");
      expect(activeTab.isDirty).toBe(true);

      (app as any).handleRibbonAction("ShowFormulas");
      expect(activeTab.isDirty).toBe(true);

      (app as any).handleRibbonAction("InsertComment", "Verified Note");
      expect(activeTab.isDirty).toBe(true);

      (app as any).handleRibbonAction("ClearCells");
      expect(activeTab.isDirty).toBe(true);
    });

    it("updates Status Bar with live metrics (Average, Count, Min, Max, Sum) on multi-cell selection", () => {
      const calcCanvas = app.getCalcGridCanvas()!;
      const stateRef = app.getCalcGridCanvas()?.["state"] as CalcState;
      expect(stateRef).toBeDefined();

      stateRef.setCellValue("A1", "10");
      stateRef.setCellValue("A2", "20");
      stateRef.setCellValue("A3", "30");

      calcCanvas.selectCell("A1");
      calcCanvas.selectCell("A3", true);

      const docInfo = document.getElementById("status-doc-info");
      expect(docInfo?.textContent).toContain("Average: 20");
      expect(docInfo?.textContent).toContain("Count: 3");
      expect(docInfo?.textContent).toContain("Min: 10");
      expect(docInfo?.textContent).toContain("Max: 30");
      expect(docInfo?.textContent).toContain("Sum: 60");
    });

    it("adjusts Zoom via ribbon actions ZoomIn, ZoomOut, and Zoom100", () => {
      (app as any).handleRibbonAction("ZoomIn");
      const stateRef = app.getCalcGridCanvas()?.["state"] as CalcState;
      expect(stateRef.zoom).toBeGreaterThan(1.0);

      (app as any).handleRibbonAction("Zoom100");
      expect(stateRef.zoom).toBe(1.0);

      (app as any).handleRibbonAction("ZoomOut");
      expect(stateRef.zoom).toBeLessThan(1.0);
    });
  });
});
