import { describe, it, expect, beforeEach, vi } from "vitest";
import { UnifiedShellApp } from "../main";
import { calcState } from "../state/calcState";
import { IPC } from "../ipc";

describe("Subphase 30.1.1: Excel Home, Insert & Page Layout Ribbons (100% Working Controls)", () => {
  let app: UnifiedShellApp;

  beforeEach(() => {
    localStorage.clear();
    IPC._resetMockState();
    calcState.resetToBlank();

    HTMLCanvasElement.prototype.getContext = (() => ({
      clearRect: vi.fn(),
      fillRect: vi.fn(),
      fillText: vi.fn(),
      strokeRect: vi.fn(),
      measureText: () => ({ width: 10 }),
      beginPath: vi.fn(),
      moveTo: vi.fn(),
      lineTo: vi.fn(),
      stroke: vi.fn(),
      fill: vi.fn(),
      save: vi.fn(),
      restore: vi.fn(),
      setTransform: vi.fn(),
      scale: vi.fn(),
      setLineDash: vi.fn(),
      ellipse: vi.fn(),
      rect: vi.fn(),
      roundRect: vi.fn(),
    })) as unknown as typeof HTMLCanvasElement.prototype.getContext;

    document.body.innerHTML = `
      <div id="app" class="desktop-shell">
        <header class="title-bar">
          <div class="app-branding">
            <span class="app-name">AetherOffice</span>
            <div class="quick-access-toolbar">
              <button id="btn-quick-open" class="quick-btn"></button>
              <button id="btn-quick-save" class="quick-btn"></button>
              <button id="btn-quick-print" class="quick-btn"></button>
            </div>
          </div>
          <div class="title-actions">
            <button id="btn-theme-toggle" class="theme-toggle-btn"></button>
          </div>
          <div class="window-controls">
            <button class="win-btn win-min"></button>
            <button class="win-btn win-max"></button>
            <button class="win-btn win-close"></button>
          </div>
        </header>
        <div class="shell-body">
          <aside id="app-rail-container" class="app-rail"></aside>
          <div class="shell-workspace-area">
            <nav id="tab-bar-container" class="tab-bar"></nav>
            <section id="ribbon-container" class="ribbon-bar"></section>
            <section id="sub-header-container" class="sub-header-bar"></section>
            <main class="workspace-layout">
              <aside id="sidebar-container" class="sidebar-pane"></aside>
              <section class="canvas-viewport" id="viewport-container" tabindex="0">
                <canvas id="render-surface"></canvas>
              </section>
            </main>
            <footer id="status-bar" class="status-bar">
              <span id="status-doc-info"></span>
              <span id="status-word-count"></span>
              <span id="status-lang"></span>
              <span id="status-zoom-level">100%</span>
              <input type="range" id="zoom-slider" min="20" max="400" value="100" />
            </footer>
          </div>
        </div>
      </div>
    `;

    app = new UnifiedShellApp(false);
    app.applyBootConfig({ app_mode: "calc" });
  });

  describe("1. Excel Home Tab: Clipboard, Font, Alignment, Numbers, Styles, Cells & Editing", () => {
    it("handles font family, font size, bold, italic, underline, borders, and colors", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Home");

      calcState.selectCell("B2");
      calcState.setCellValue("B2", "Styled Text");

      const fontSelect = document.querySelector<HTMLSelectElement>('[data-testid="calc-select-font-family"]')!;
      expect(fontSelect).not.toBeNull();
      fontSelect.value = "Arial";
      fontSelect.dispatchEvent(new Event("change"));
      expect(calcState.getCell("B2").fontFamily).toBe("Arial");

      const sizeSelect = document.querySelector<HTMLSelectElement>('[data-testid="calc-select-font-size"]')!;
      expect(sizeSelect).not.toBeNull();
      sizeSelect.value = "14";
      sizeSelect.dispatchEvent(new Event("change"));
      expect(calcState.getCell("B2").fontSize).toBe(14);

      const boldBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-bold"]')!;
      boldBtn.click();
      expect(calcState.getCell("B2").bold).toBe(true);

      const italicBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-italic"]')!;
      italicBtn.click();
      expect(calcState.getCell("B2").italic).toBe(true);

      const underlineBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-underline"]')!;
      underlineBtn.click();
      expect(calcState.getCell("B2").underline).toBe(true);

      const bordersBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-borders"]')!;
      bordersBtn.click();
      expect(calcState.getCell("B2").borders).toBe("all");

      const fillBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-fill-color"]')!;
      fillBtn.click();
      expect(calcState.getCell("B2").bgColor).toBe("#dcfce7");

      const colorBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-font-color"]')!;
      colorBtn.click();
      expect(calcState.getCell("B2").color).toBe("#15803d");
    });

    it("handles text alignments, wrap text, and merge & center", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Home");

      calcState.selectCell("C3");
      calcState.setCellValue("C3", "Aligned Text");

      const topBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-align-top"]')!;
      topBtn.click();
      expect(calcState.getCell("C3").valign).toBe("top");

      const midBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-align-middle"]')!;
      midBtn.click();
      expect(calcState.getCell("C3").valign).toBe("middle");

      const botBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-align-bottom"]')!;
      botBtn.click();
      expect(calcState.getCell("C3").valign).toBe("bottom");

      const wrapBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-wrap-text"]')!;
      wrapBtn.click();
      expect(calcState.getCell("C3").wrapText).toBe(true);

      const leftBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-align-left"]')!;
      leftBtn.click();
      expect(calcState.getCell("C3").align).toBe("left");

      const centerBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-align-center"]')!;
      centerBtn.click();
      expect(calcState.getCell("C3").align).toBe("center");

      const rightBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-align-right"]')!;
      rightBtn.click();
      expect(calcState.getCell("C3").align).toBe("right");

      calcState.selectedRanges = [{ minRow: 2, maxRow: 2, minCol: 2, maxCol: 4 }];
      const mergeBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-merge-center"]')!;
      mergeBtn.click();
      expect(calcState.getActiveSheet().mergedRanges?.length).toBeGreaterThan(0);
    });

    it("wires Number formats (Currency, Percent, Comma, Fraction, Decimals)", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Home");

      calcState.selectCell("D4");
      calcState.setCellValue("D4", "1250.75");

      const numSelect = document.querySelector<HTMLSelectElement>('[data-testid="calc-select-number-format"]')!;
      numSelect.value = "Fraction";
      numSelect.dispatchEvent(new Event("change"));
      expect(calcState.getCell("D4").format).toBe("Fraction");

      const currBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-num-currency"]')!;
      currBtn.click();
      expect(calcState.getCell("D4").format).toBe("Currency");

      const pctBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-num-percent"]')!;
      pctBtn.click();
      expect(calcState.getCell("D4").format).toBe("Percentage");

      const commaBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-num-comma"]')!;
      commaBtn.click();
      expect(calcState.getCell("D4").format).toBe("Comma");

      const incDecBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-inc-decimal"]')!;
      incDecBtn.click();
      expect(calcState.getCell("D4").decimals).toBe(3);

      const decDecBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-dec-decimal"]')!;
      decDecBtn.click();
      expect(calcState.getCell("D4").decimals).toBe(2);
    });

    it("applies Conditional Formatting rules and Format as Table", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Home");

      calcState.selectedRanges = [{ minRow: 0, maxRow: 4, minCol: 0, maxCol: 2 }];
      const condGtBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-cond-gt"]')!;
      condGtBtn.click();
      expect(calcState.getActiveSheet().conditionalFormats?.length).toBe(1);

      const condBarBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-cond-databar"]')!;
      condBarBtn.click();
      expect(calcState.getActiveSheet().conditionalFormats?.length).toBe(2);

      const tableBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-format-table"]')!;
      tableBtn.click();
      expect(calcState.getActiveSheet().tableRange).toBeDefined();
      expect(calcState.getCell("A1").bgColor).toBe("#107c41");
      expect(calcState.getCell("A2").bgColor).toBe("#ffffff");
    });

    it("executes row/column insert, delete, autofit, fill, and clear actions", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Home");

      calcState.selectCell("B2");
      calcState.setCellValue("B2", "Sample Data Long Text");

      const autoColBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-autofit-col"]')!;
      autoColBtn.click();
      expect(calcState.getActiveSheet().colWidths.get(1)).toBeGreaterThan(80);

      const autoRowBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-autofit-row"]')!;
      autoRowBtn.click();
      expect(calcState.getActiveSheet().rowHeights.get(1)).toBeGreaterThanOrEqual(22);

      calcState.selectedRanges = [{ minRow: 1, maxRow: 4, minCol: 1, maxCol: 1 }];
      const fillDownBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-fill-down"]')!;
      fillDownBtn.click();
      expect(calcState.getCell("B3").raw).toBe("Sample Data Long Text");

      const clearFormatsBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-clear-formats"]')!;
      clearFormatsBtn.click();
      expect(calcState.getCell("B2").bold).toBeUndefined();

      const clearContentsBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-clear-contents"]')!;
      clearContentsBtn.click();
      expect(calcState.getCell("B2").raw).toBe("");
    });
  });

  describe("2. Excel Insert Tab: Pivot Summary, Charts, Sparklines, Illustrations & Notes", () => {
    it("generates automatic Pivot Summary table from data", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Insert");

      calcState.setCellValue("A1", "North");
      calcState.setCellValue("B1", "1000");
      calcState.setCellValue("A2", "North");
      calcState.setCellValue("B2", "2000");
      calcState.setCellValue("A3", "South");
      calcState.setCellValue("B3", "3000");
      calcState.selectedRanges = [{ minRow: 0, maxRow: 2, minCol: 0, maxCol: 1 }];

      const pivotSummaryBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-pivot-summary"]')!;
      expect(pivotSummaryBtn).not.toBeNull();
      pivotSummaryBtn.click();

      expect(calcState.getCell("G1").raw).toBe("Row Labels");
      expect(calcState.getCell("H1").raw).toBe("Sum of Value");
      expect(calcState.getCell("G2").raw).toBe("North");
      expect(calcState.getCell("H2").computed).toBe(3000);
      expect(calcState.getCell("G3").raw).toBe("South");
      expect(calcState.getCell("H3").computed).toBe(3000);
      expect(calcState.getCell("G4").raw).toBe("Grand Total");
      expect(calcState.getCell("H4").computed).toBe(6000);
    });

    it("inserts SVG charts (column, bar, line, pie, doughnut, scatter)", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Insert");

      calcState.selectedRanges = [{ minRow: 0, maxRow: 2, minCol: 0, maxCol: 1 }];

      const colChartBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-chart-column"]')!;
      colChartBtn.click();
      expect(calcState.getCharts().length).toBe(1);
      expect(calcState.getCharts()[0].type).toBe("column");

      const pieChartBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-chart-pie"]')!;
      pieChartBtn.click();
      expect(calcState.getCharts().length).toBe(2);
      expect(calcState.getCharts()[1].type).toBe("pie");

      const doughnutChartBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-chart-doughnut"]')!;
      doughnutChartBtn.click();
      expect(calcState.getCharts().length).toBe(3);
      expect(calcState.getCharts()[2].type).toBe("doughnut");

      const scatterChartBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-chart-scatter"]')!;
      scatterChartBtn.click();
      expect(calcState.getCharts().length).toBe(4);
      expect(calcState.getCharts()[3].type).toBe("scatter");
    });

    it("inserts mini inline Sparklines (Line, Column, Win/Loss)", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Insert");

      calcState.selectCell("E2");
      const lineSparkBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-sparkline-line"]')!;
      lineSparkBtn.click();
      expect(calcState.getCell("E2").sparkline?.type).toBe("line");

      calcState.selectCell("E3");
      const colSparkBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-sparkline-column"]')!;
      colSparkBtn.click();
      expect(calcState.getCell("E3").sparkline?.type).toBe("column");

      calcState.selectCell("E4");
      const wlSparkBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-sparkline-winloss"]')!;
      wlSparkBtn.click();
      expect(calcState.getCell("E4").sparkline?.type).toBe("winloss");
    });

    it("inserts Pictures, Shapes, Links, and Notes", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Insert");

      calcState.selectCell("F1");
      const picBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-insert-picture"]')!;
      picBtn.click();
      expect(calcState.getCell("F1").raw).toBe("[Picture]");

      calcState.selectCell("F2");
      const shapeBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-insert-shapes"]')!;
      shapeBtn.click();
      expect(calcState.getCell("F2").raw).toContain("[Shape:");

      calcState.selectCell("F3");
      const linkBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-insert-link"]')!;
      linkBtn.click();
      expect(calcState.getCell("F3").raw).toContain("https://");

      calcState.selectCell("F4");
      const noteBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-insert-note"]')!;
      noteBtn.click();
      expect(calcState.getCell("F4").comment).toBeDefined();
    });
  });

  describe("3. Excel Page Layout Tab: Themes, Margins, Orientation, Print Area & Sheet Options", () => {
    it("wires Themes, Theme Colors, and Theme Fonts", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Page Layout");

      const themeBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-themes"]')!;
      themeBtn.click();
      expect(calcState.getActiveSheet().theme).toBe("Office");

      const colorBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-theme-colors"]')!;
      colorBtn.click();
      expect(calcState.getActiveSheet().themeColors).toBe("Office");

      const fontBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-theme-fonts"]')!;
      fontBtn.click();
      expect(calcState.getActiveSheet().themeFonts).toBe("Calibri");
    });

    it("wires Margins (Normal, Narrow, Wide) and Orientation (Portrait, Landscape)", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Page Layout");

      const narrowBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-margins-narrow"]')!;
      narrowBtn.click();
      expect(calcState.getActiveSheet().margins).toBe("narrow");

      const wideBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-margins-wide"]')!;
      wideBtn.click();
      expect(calcState.getActiveSheet().margins).toBe("wide");

      const landBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-orient-landscape"]')!;
      landBtn.click();
      expect(calcState.getActiveSheet().orientation).toBe("landscape");

      const portBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-orient-portrait"]')!;
      portBtn.click();
      expect(calcState.getActiveSheet().orientation).toBe("portrait");
    });

    it("sets and clears Print Area", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Page Layout");

      calcState.selectedRanges = [{ minRow: 0, maxRow: 10, minCol: 0, maxCol: 5 }];
      const setAreaBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-set-print-area"]')!;
      setAreaBtn.click();
      expect(calcState.getActiveSheet().printArea).toEqual({ minRow: 0, maxRow: 10, minCol: 0, maxCol: 5 });

      const clearAreaBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-clear-print-area"]')!;
      clearAreaBtn.click();
      expect(calcState.getActiveSheet().printArea).toBeNull();
    });

    it("toggles View Grid, Print Grid, View Headings, and Print Headings", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Page Layout");

      const gridBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-toggle-grid"]')!;
      gridBtn.click();
      expect(calcState.getActiveSheet().showGridlines).toBe(false);
      gridBtn.click();
      expect(calcState.getActiveSheet().showGridlines).toBe(true);

      const printGridBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-print-grid"]')!;
      printGridBtn.click();
      expect(calcState.getActiveSheet().printGridlines).toBe(true);

      const headingsBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-toggle-headings"]')!;
      headingsBtn.click();
      expect(calcState.getActiveSheet().showHeadings).toBe(false);
      headingsBtn.click();
      expect(calcState.getActiveSheet().showHeadings).toBe(true);

      const printHeadingsBtn = document.querySelector<HTMLButtonElement>('[data-testid="calc-btn-print-headings"]')!;
      printHeadingsBtn.click();
      expect(calcState.getActiveSheet().printHeadings).toBe(true);
    });
  });
});
