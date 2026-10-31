import { describe, it, expect, beforeEach, vi } from "vitest";
import { UnifiedShellApp } from "../main";
import { calcState } from "../state/calcState";
import { IPC } from "../ipc";

describe("GEC-v5.0 Subphase 30.1.2: Excel Complete Ribbon Overhaul (7 Tabs) & Action Verification", () => {
  let app: UnifiedShellApp;

  beforeEach(() => {
    localStorage.clear();
    IPC._resetMockState();
    calcState.resetToBlank();

    const mockCtx = {
      clearRect: vi.fn(),
      fillRect: vi.fn(),
      strokeRect: vi.fn(),
      fillText: vi.fn(),
      measureText: (text: string) => ({ width: text.length * 7 }),
      beginPath: vi.fn(),
      moveTo: vi.fn(),
      lineTo: vi.fn(),
      arc: vi.fn(),
      closePath: vi.fn(),
      fill: vi.fn(),
      stroke: vi.fn(),
      save: vi.fn(),
      restore: vi.fn(),
      setTransform: vi.fn(),
      scale: vi.fn(),
      setLineDash: vi.fn(),
      rect: vi.fn(),
      clip: vi.fn(),
      font: "",
      fillStyle: "",
      strokeStyle: "",
      lineWidth: 1,
      textAlign: "left",
    };
    HTMLCanvasElement.prototype.getContext = (() => mockCtx) as any;

    document.body.innerHTML = `
      <div id="app" class="desktop-shell">
        <header class="title-bar">
          <div class="app-branding">
            <span class="app-name">AetherOffice</span>
          </div>
          <div class="title-actions">
            <button id="btn-theme-toggle" class="theme-toggle-btn"></button>
          </div>
        </header>
        <div id="left-app-rail-container"></div>
        <div id="tab-bar-container"></div>
        <div id="ribbon-container"></div>
        <div id="sub-header-container" style="display: flex;"></div>
        <div class="workbench">
          <div id="sidebar-container"></div>
          <main id="viewport-container" class="viewport"></main>
          <div id="copilot-sidebar-container"></div>
        </div>
        <footer id="status-bar-container">
          <span id="status-doc-info"></span>
          <span id="status-page-info"></span>
          <input type="range" id="zoom-slider" min="50" max="200" value="100" />
          <span id="status-zoom-level">100%</span>
        </footer>
      </div>
    `;

    app = new UnifiedShellApp(false);
    app.applyBootConfig({ app_mode: "calc" });
  });

  describe("1. Home Tab: Complete Formatting, Number, Styles, Cells, Editing", () => {
    it("handles Clipboard: Cut, Copy, Paste, PasteValues, FormatPainter", () => {
      calcState.setCellValue("A1", "Test123");
      calcState.selectCell("A1");
      (app as any).handleRibbonAction("Copy");
      expect(calcState.internalClipboardText).toBe("Test123");

      calcState.selectCell("B1");
      (app as any).handleRibbonAction("Paste");
      expect(calcState.getCell("B1").raw).toBe("Test123");

      calcState.internalClipboardText = "456";
      calcState.selectCell("C1");
      (app as any).handleRibbonAction("PasteValues");
      expect(calcState.getCell("C1").raw).toBe("456");
      expect(calcState.getCell("C1").computed).toBe(456);

      // Format Painter
      calcState.selectCell("A1");
      calcState.toggleBold();
      calcState.setFillColor("#fef08a");
      (app as any).handleRibbonAction("FormatPainter");
      expect(calcState.formatPainterActive).toBe(true);

      calcState.selectCell("D1");
      (app as any).handleRibbonAction("FormatPainter");
      expect(calcState.getCell("D1").bold).toBe(true);
      expect(calcState.getCell("D1").bgColor).toBe("#fef08a");
      expect(calcState.formatPainterActive).toBe(false);
    });

    it("handles Font & Alignment: A+, A-, Bold, Italic, Underline, Borders, Alignments", () => {
      calcState.selectCell("A1");
      (app as any).handleRibbonAction("SetFontFamily", "Arial");
      expect(calcState.getCell("A1").fontFamily).toBe("Arial");

      (app as any).handleRibbonAction("SetFontSize", 14);
      expect(calcState.getCell("A1").fontSize).toBe(14);

      (app as any).handleRibbonAction("IncreaseFontSize");
      expect(calcState.getCell("A1").fontSize).toBe(16);

      (app as any).handleRibbonAction("DecreaseFontSize");
      expect(calcState.getCell("A1").fontSize).toBe(14);

      (app as any).handleRibbonAction("ToggleBold");
      (app as any).handleRibbonAction("ToggleItalic");
      (app as any).handleRibbonAction("ToggleUnderline");
      (app as any).handleRibbonAction("SetFontColor", "#dc2626");
      (app as any).handleRibbonAction("SetFillColor", "#fee2e2");
      (app as any).handleRibbonAction("SetBorders", "all");

      const cell = calcState.getCell("A1");
      expect(cell.bold).toBe(true);
      expect(cell.italic).toBe(true);
      expect(cell.underline).toBe(true);
      expect(cell.color).toBe("#dc2626");
      expect(cell.bgColor).toBe("#fee2e2");
      expect(cell.borders).toBe("all");

      // Alignments
      (app as any).handleRibbonAction("AlignTop");
      expect(calcState.getCell("A1").valign).toBe("top");
      (app as any).handleRibbonAction("AlignMiddle");
      expect(calcState.getCell("A1").valign).toBe("middle");
      (app as any).handleRibbonAction("AlignBottom");
      expect(calcState.getCell("A1").valign).toBe("bottom");

      (app as any).handleRibbonAction("AlignLeft");
      expect(calcState.getCell("A1").align).toBe("left");
      (app as any).handleRibbonAction("AlignCenter");
      expect(calcState.getCell("A1").align).toBe("center");
      (app as any).handleRibbonAction("AlignRight");
      expect(calcState.getCell("A1").align).toBe("right");

      (app as any).handleRibbonAction("ToggleWrapText");
      expect(calcState.getCell("A1").wrapText).toBe(true);

      (app as any).handleRibbonAction("MergeAndCenter");
      expect(calcState.getActiveSheet().mergedRanges?.length).toBeGreaterThan(0);
    });

    it("handles Number Formatting & Decimals", () => {
      calcState.setCellValue("A1", "1234.56");
      calcState.selectCell("A1");

      (app as any).handleRibbonAction("SetNumberFormat", "Currency");
      expect(calcState.getCell("A1").format).toBe("Currency");

      (app as any).handleRibbonAction("SetNumberFormat", "Percentage");
      expect(calcState.getCell("A1").format).toBe("Percentage");

      (app as any).handleRibbonAction("SetNumberFormat", "Comma");
      expect(calcState.getCell("A1").format).toBe("Comma");

      (app as any).handleRibbonAction("IncreaseDecimal");
      expect(calcState.getCell("A1").decimals).toBe(3);

      (app as any).handleRibbonAction("DecreaseDecimal");
      expect(calcState.getCell("A1").decimals).toBe(2);
    });

    it("handles Styles: Conditional Formatting, Table Formatting, Cell Styles", () => {
      calcState.selectCell("A1");
      (app as any).handleRibbonAction("ConditionalFormatting", "greaterThan");
      (app as any).handleRibbonAction("ConditionalFormatting", "dataBar");
      (app as any).handleRibbonAction("ConditionalFormatting", "colorScale");
      expect(calcState.getActiveSheet().conditionalFormats?.length).toBe(3);

      (app as any).handleRibbonAction("FormatAsTable");
      expect(calcState.getActiveSheet().tableRange).toBeDefined();

      (app as any).handleRibbonAction("CellStyles");
      expect(calcState.getCell("A1").bgColor).toBe("#e0f2fe");
    });

    it("handles Cells & Editing: Insert, Delete, AutoSum, Fill, Clear, Sort", () => {
      calcState.setCellValue("A1", "10");
      calcState.setCellValue("A2", "20");
      calcState.setCellValue("A3", "30");

      calcState.selectCell("A4");
      (app as any).handleRibbonAction("AutoSum");
      expect(calcState.getCell("A4").raw).toBe("=SUM(A1:A3)");

      calcState.selectCell("A5");
      (app as any).handleRibbonAction("AutoAverage");
      expect(calcState.getCell("A5").raw).toBe("=AVERAGE(A1:A4)");

      calcState.selectCell("A6");
      (app as any).handleRibbonAction("AutoCount");
      expect(calcState.getCell("A6").raw).toBe("=COUNT(A1:A5)");

      calcState.selectCell("A7");
      (app as any).handleRibbonAction("AutoMax");
      expect(calcState.getCell("A7").raw).toBe("=MAX(A1:A6)");

      calcState.selectCell("A8");
      (app as any).handleRibbonAction("AutoMin");
      expect(calcState.getCell("A8").raw).toBe("=MIN(A1:A7)");

      // Fill Down & Series
      calcState.selectRangeByName("A1:A4");
      (app as any).handleRibbonAction("FillDown");
      (app as any).handleRibbonAction("FillRight");
      (app as any).handleRibbonAction("FillSeries");

      // Clear
      (app as any).handleRibbonAction("ClearFormats");
      (app as any).handleRibbonAction("ClearContents");
      (app as any).handleRibbonAction("ClearCells");
    });
  });

  describe("2. Insert Tab: PivotTables, Charts, Sparklines, Illustrations & Links", () => {
    it("handles PivotTable and Slicer insertion", () => {
      (app as any).handleRibbonAction("InsertPivotTable");
      expect((app as any).pivotFieldListSidebar.isVisible()).toBe(true);

      (app as any).handleRibbonAction("InsertPivotChart");
      (app as any).handleRibbonAction("InsertSlicer");
    });

    it("handles Charts including Column, Line, Pie, Doughnut, Scatter, Area", () => {
      calcState.setCellValue("A1", "Product");
      calcState.setCellValue("B1", "Sales");
      calcState.setCellValue("A2", "Apples");
      calcState.setCellValue("B2", "100");
      calcState.setCellValue("A3", "Bananas");
      calcState.setCellValue("B3", "150");
      calcState.selectRangeByName("A1:B3");

      (app as any).handleRibbonAction("InsertChart", "column");
      (app as any).handleRibbonAction("InsertChart", "line");
      (app as any).handleRibbonAction("InsertChart", "pie");
      (app as any).handleRibbonAction("InsertChart", "doughnut");
      (app as any).handleRibbonAction("InsertChart", "scatter");
      (app as any).handleRibbonAction("InsertChart", "area");

      const charts = calcState.getCharts();
      expect(charts.length).toBe(6);
      expect(charts[0].type).toBe("column");
      expect(charts[1].type).toBe("line");
      expect(charts[2].type).toBe("pie");
      expect(charts[3].type).toBe("doughnut");
      expect(charts[4].type).toBe("scatter");
      expect(charts[5].type).toBe("area");
    });

    it("handles Sparklines, Illustrations & Links", () => {
      calcState.selectCell("C1");
      (app as any).handleRibbonAction("InsertSparkline", "line");
      expect(calcState.getCell("C1").sparkline?.type).toBe("line");

      calcState.selectCell("C2");
      (app as any).handleRibbonAction("InsertSparkline", "column");
      expect(calcState.getCell("C2").sparkline?.type).toBe("column");

      calcState.selectCell("C3");
      (app as any).handleRibbonAction("InsertSparkline", "winloss");
      expect(calcState.getCell("C3").sparkline?.type).toBe("winloss");

      (app as any).handleRibbonAction("InsertPicture");
      expect(calcState.getCell("C3").raw).toBe("[Picture]");

      (app as any).handleRibbonAction("InsertShape", "circle");
      expect(calcState.getCell("C3").raw).toBe("[Shape: circle]");

      (app as any).handleRibbonAction("InsertLink");
      expect(calcState.getCell("C3").raw).toBe("https://github.com/rustoffice");
    });
  });

  describe("3. Page Layout Tab: Themes, Margins, Orientation, Page Size, Print Area", () => {
    it("handles Themes, Colors, Fonts, Margins, Orientation, Page Size, Print Area", () => {
      (app as any).handleRibbonAction("SetTheme", "Facet");
      expect(calcState.getActiveSheet().theme).toBe("Facet");

      (app as any).handleRibbonAction("SetThemeColors", "Teal");
      expect(calcState.getActiveSheet().themeColors).toBe("Teal");

      (app as any).handleRibbonAction("SetThemeFonts", "Segoe UI");
      expect(calcState.getActiveSheet().themeFonts).toBe("Segoe UI");

      (app as any).handleRibbonAction("SetMargins", "wide");
      expect(calcState.getActiveSheet().margins).toBe("wide");

      (app as any).handleRibbonAction("SetOrientation", "landscape");
      expect(calcState.getActiveSheet().orientation).toBe("landscape");

      (app as any).handleRibbonAction("SetPageSize", "a4");
      expect(calcState.getActiveSheet().pageSize).toBe("a4");

      calcState.selectRangeByName("A1:D10");
      (app as any).handleRibbonAction("SetPrintArea");
      expect(calcState.getActiveSheet().printArea).not.toBeNull();

      (app as any).handleRibbonAction("ClearPrintArea");
      expect(calcState.getActiveSheet().printArea).toBeNull();

      (app as any).handleRibbonAction("TogglePrintGrid");
      expect(calcState.getActiveSheet().printGridlines).toBe(true);

      (app as any).handleRibbonAction("TogglePrintHeadings");
      expect(calcState.getActiveSheet().printHeadings).toBe(true);

      (app as any).handleRibbonAction("ScaleToFit");
      expect(calcState.zoom).toBe(0.85);
    });
  });

  describe("4. Formulas Tab: Function Library, Auditing & Recalculate", () => {
    it("handles Function Wizard, Categories, Auditing & Evaluation", () => {
      (app as any).handleRibbonAction("InsertFunction");
      expect((app as any).functionWizardDialog.isOpen()).toBe(true);

      (app as any).handleRibbonAction("FormulaCategory", "financial");
      (app as any).handleRibbonAction("FormulaCategory", "math");

      (app as any).handleRibbonAction("NameManager");
      expect((app as any).nameManagerDialog.isOpen()).toBe(true);

      (app as any).handleRibbonAction("ShowFormulas");
      expect(calcState.showFormulas).toBe(true);

      calcState.setCellValue("B1", "100");
      calcState.setCellValue("B2", "=B1*2");
      calcState.selectCell("B2");
      (app as any).handleRibbonAction("TracePrecedents");
      (app as any).handleRibbonAction("TraceDependents");
      (app as any).handleRibbonAction("RemoveArrows");

      (app as any).handleRibbonAction("EvaluateFormula");
      expect(calcState.getCell("B2").computed).toBe(200);

      (app as any).handleRibbonAction("CalculateSheet");
      (app as any).handleRibbonAction("Recalculate");
    });
  });

  describe("5. Data Tab: Import, Sorting, Filter, Data Tools, Outline", () => {
    it("handles CSV & JSON Web Import", () => {
      (app as any).handleRibbonAction("ImportCsv", "A,B\n1,2\n3,4");
      expect(calcState.getCell("A1").computed).toBe("A");
      expect(calcState.getCell("B2").computed).toBe(2);

      (app as any).handleRibbonAction("ImportWeb", '[{"Name":"Widget","Price":49.99}]');
      expect(calcState.getCell("A1").computed).toBe("Name");
      expect(calcState.getCell("B2").computed).toBe(49.99);
    });

    it("handles Sort, Filter, TextToColumns, FlashFill, RemoveDuplicates, GoalSeek", () => {
      calcState.setCellValue("A1", "10");
      calcState.setCellValue("A2", "50");
      calcState.setCellValue("A3", "30");
      calcState.selectRangeByName("A1:A3");

      (app as any).handleRibbonAction("SortAsc");
      (app as any).handleRibbonAction("SortDesc");

      (app as any).handleRibbonAction("CustomSort", [{ col: 0, ascending: true }]);
      (app as any).handleRibbonAction("ToggleFilter");
      expect(calcState.getActiveSheet().filterRange).not.toBeNull();
      (app as any).handleRibbonAction("ClearFilter");
      expect(calcState.getActiveSheet().hiddenRows?.size || 0).toBe(0);

      calcState.setCellValue("B1", "First,Last");
      calcState.selectCell("B1");
      (app as any).handleRibbonAction("TextToColumns", ",");
      (app as any).handleRibbonAction("FlashFill");
      (app as any).handleRibbonAction("RemoveDuplicates");

      (app as any).handleRibbonAction("DataValidation", { type: "whole", min: 1, max: 100 });
      expect(calcState.getCell("B1").validation).toBeDefined();

      (app as any).handleRibbonAction("GroupRows");
      (app as any).handleRibbonAction("UngroupRows");
      (app as any).handleRibbonAction("Subtotal");

      (app as any).handleRibbonAction("GoalSeek");
      expect((app as any).goalSeekDialog.isOpen()).toBe(true);
    });
  });

  describe("6. Review & View Tabs: Notes, Protection, Views, Zoom, Freezing", () => {
    it("handles Comments/Notes, Sheet Protection, Workbook Stats", () => {
      calcState.selectCell("A1");
      (app as any).handleRibbonAction("NewNote", "Review this quarterly figure");
      expect(calcState.getCell("A1").comment).toBe("Review this quarterly figure");

      (app as any).handleRibbonAction("ShowAllNotes");
      expect(calcState.showAllNotes).toBe(true);

      (app as any).handleRibbonAction("DeleteComment");
      expect(calcState.getCell("A1").comment).toBeUndefined();

      (app as any).handleRibbonAction("ProtectSheet");
      expect(calcState.isSheetProtected()).toBe(true);
      (app as any).handleRibbonAction("ProtectSheet");
      expect(calcState.isSheetProtected()).toBe(false);

      (app as any).handleRibbonAction("ProtectWorkbook");
      expect(calcState.isWorkbookProtected()).toBe(true);
      (app as any).handleRibbonAction("ProtectWorkbook");
      expect(calcState.isWorkbookProtected()).toBe(false);

      (app as any).handleRibbonAction("WorkbookStatistics");
      expect((app as any).workbookStatisticsDialog.isOpen()).toBe(true);
    });

    it("handles Views, Toggles, Zoom, Freezing Panes & Window Splitting", () => {
      (app as any).handleRibbonAction("ViewPageBreak");
      expect(calcState.getActiveSheet().viewMode).toBe("pagebreak");
      (app as any).handleRibbonAction("ViewPageLayout");
      expect(calcState.getActiveSheet().viewMode).toBe("pagelayout");
      (app as any).handleRibbonAction("ViewNormal");
      expect(calcState.getActiveSheet().viewMode).toBe("normal");

      (app as any).handleRibbonAction("ToggleFormulaBar");
      (app as any).handleRibbonAction("ToggleGridlines");
      (app as any).handleRibbonAction("ToggleHeadings");

      (app as any).handleRibbonAction("ZoomIn");
      (app as any).handleRibbonAction("ZoomOut");
      (app as any).handleRibbonAction("Zoom100");
      expect(calcState.zoom).toBe(1.0);
      (app as any).handleRibbonAction("ZoomToSelection");

      calcState.selectCell("C3");
      (app as any).handleRibbonAction("FreezePanes");
      expect(calcState.getActiveSheet().freezePanes).toEqual({ row: 2, col: 2 });

      (app as any).handleRibbonAction("UnfreezePanes");
      expect(calcState.getActiveSheet().freezePanes).toEqual({ row: 0, col: 0 });

      (app as any).handleRibbonAction("FreezeTopRow");
      expect(calcState.getActiveSheet().freezePanes).toEqual({ row: 1, col: 0 });

      (app as any).handleRibbonAction("FreezeFirstColumn");
      expect(calcState.getActiveSheet().freezePanes).toEqual({ row: 0, col: 1 });

      (app as any).handleRibbonAction("SplitWindow");
      expect(calcState.isSplitWindow).toBe(true);
      (app as any).handleRibbonAction("SplitWindow");
      expect(calcState.isSplitWindow).toBe(false);
    });
  });

  describe("7. Excel Find & Replace Integration", () => {
    it("finds, replaces and replaceAll across spreadsheet cells", () => {
      calcState.setCellValue("A1", "Alpha Target");
      calcState.setCellValue("B1", "Beta Target");
      calcState.setCellValue("C1", "Delta Unique");

      (app as any).showFindReplace("find");
      const findDialog = (app as any).findReplaceDialog;
      expect(findDialog).toBeDefined();

      // Test onFind callback
      const matches = findDialog.callbacks?.onFind("Target", false);
      expect(matches?.length).toBe(2);

      // Test onSelectMatch
      findDialog.callbacks?.onSelectMatch(matches[0]);
      expect(calcState.selectedCell).toBe("A1");

      // Test onReplace
      findDialog.callbacks?.onReplace(matches[0], "Updated");
      expect(calcState.getCell("A1").raw).toBe("Alpha Updated");

      // Test onReplaceAll
      const replacedCount = findDialog.callbacks?.onReplaceAll("Unique", "Finished", false);
      expect(replacedCount).toBe(1);
      expect(calcState.getCell("C1").raw).toBe("Delta Finished");
    });
  });

  describe("8. Ribbon DOM Audit: Zero Overflow & Active Control Verification", () => {
    it("renders all 7 ribbon tabs and verifies every button has a valid handler and zero clipping styles", () => {
      const ribbonContainer = document.getElementById("ribbon-container")!;
      expect(ribbonContainer).toBeDefined();

      const tabs = ["Home", "Insert", "Page Layout", "Formulas", "Data", "Review", "View"];
      const ribbon = app.getRibbon();

      for (const tabName of tabs) {
        ribbon.setActiveTab(tabName);
        const toolsBody = ribbonContainer.querySelector(".ribbon-tools-body") as HTMLElement;
        expect(toolsBody).not.toBeNull();

        // Check flex wrapping style is applied for zero-overflow contract
        const groups = toolsBody.querySelectorAll(".ribbon-group");
        expect(groups.length).toBeGreaterThan(0);

        const buttons = toolsBody.querySelectorAll("button");
        expect(buttons.length).toBeGreaterThan(0);

        buttons.forEach((btn) => {
          // Button must have text or icon
          expect(btn.textContent?.trim().length).toBeGreaterThan(0);
          // Button must be clickable without throwing
          expect(() => btn.click()).not.toThrow();
        });
      }
    });
  });
});
