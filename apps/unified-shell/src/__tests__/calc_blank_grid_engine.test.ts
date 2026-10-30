import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { CalcGridCanvas } from "../components/viewports/CalcGridCanvas";
import { CalcState } from "../state/calcState";
import { SheetTabBar } from "../components/SheetTabBar";
import { WorkbookSheetsPane } from "../components/WorkbookSheetsPane";
import { UnifiedShellApp } from "../main";

describe("Blank Virtualized Excel Grid, Instant Typing, Formula Engine & Multi-Sheet Bar (GEC-v3.0 Subphase 23.1.1)", () => {
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
      <div id="viewport-container" style="width: 800px; height: 600px; position: relative;"></div>
      <div id="status-bar">
        <span id="status-doc-info"></span>
      </div>
    `;

    container = document.getElementById("viewport-container")!;
    state = new CalcState();
    canvas = new CalcGridCanvas(container, state);
  });

  afterEach(() => {
    document.body.innerHTML = "";
  });

  describe("1. Blank Boot & Zero-Fake-Data Contract Verification", () => {
    it("initializes CalcState with a single 100% blank sheet and zero prepopulated cells", () => {
      expect(state.sheets.length).toBe(1);
      expect(state.sheets[0].name).toBe("Sheet1");
      expect(state.sheets[0].cells.size).toBe(0);
      expect(state.formulaBarText).toBe("");
      expect(state.selectedCell).toBe("A1");
    });

    it("resets to a clean blank sheet when resetToBlank is called", () => {
      state.setCellValue("A1", "Test1");
      state.setCellValue("B1", "Test2");
      expect(state.cells.size).toBe(2);

      state.resetToBlank("Sheet1");
      expect(state.sheets.length).toBe(1);
      expect(state.sheets[0].name).toBe("Sheet1");
      expect(state.sheets[0].cells.size).toBe(0);
      expect(state.formulaBarText).toBe("");
      expect(state.selectedCell).toBe("A1");
    });
  });

  describe("2. Instant Typing, In-Cell Editor & Navigation Keys", () => {
    it("activates in-cell editor immediately on any printable key on selected cell", () => {
      canvas.selectCell("A1");
      const canvasEl = container.querySelector("canvas")!;

      canvasEl.dispatchEvent(new KeyboardEvent("keydown", { key: "4" }));
      expect(canvas.isCellEditing()).toBe(true);

      const editor = canvas.getEditorInput();
      expect(editor.style.display).toBe("block");
      expect(editor.value).toBe("4");
    });

    it("commits cell value and navigates down on Enter, right on Tab", () => {
      canvas.selectCell("A1");
      canvas.startEditing("100");
      canvas.commitEditing("down");

      expect(state.cells.get("A1")?.computed).toBe(100);
      expect(state.selectedCell).toBe("A2");

      canvas.startEditing("200");
      canvas.commitEditing("right");

      expect(state.cells.get("A2")?.computed).toBe(200);
      expect(state.selectedCell).toBe("B2");
    });

    it("navigates selection with Tab and Shift+Tab when not editing", () => {
      canvas.selectCell("A1");
      const canvasEl = container.querySelector("canvas")!;

      // Tab moves to B1
      canvasEl.dispatchEvent(new KeyboardEvent("keydown", { key: "Tab" }));
      expect(state.selectedCell).toBe("B1");

      // Shift+Tab moves back to A1
      canvasEl.dispatchEvent(new KeyboardEvent("keydown", { key: "Tab", shiftKey: true }));
      expect(state.selectedCell).toBe("A1");
    });

    it("clears cell on Delete or Backspace key", () => {
      state.setCellValue("A1", "Content");
      expect(state.cells.get("A1")?.raw).toBe("Content");

      canvas.selectCell("A1");
      const canvasEl = container.querySelector("canvas")!;
      canvasEl.dispatchEvent(new KeyboardEvent("keydown", { key: "Delete" }));

      expect(state.cells.get("A1")?.raw).toBe("");
    });

    it("cancels in-cell editing on Escape key without mutating cell value", () => {
      state.setCellValue("A1", "Original");
      canvas.selectCell("A1");
      canvas.startEditing();

      const editor = canvas.getEditorInput();
      editor.value = "NewModifiedValue";

      canvas.cancelEditing();
      expect(canvas.isCellEditing()).toBe(false);
      expect(state.cells.get("A1")?.raw).toBe("Original");
    });
  });

  describe("3. Live Formula Engine & Dynamic Recalculation", () => {
    it("evaluates =SUM(A1:A2) in A3 and automatically recalculates when A1 or A2 changes", () => {
      state.setCellValue("A1", "150");
      state.setCellValue("A2", "250");
      state.setCellValue("A3", "=SUM(A1:A2)");

      expect(state.cells.get("A3")?.computed).toBe(400);

      // Mutate A1 -> dynamic update
      state.setCellValue("A1", "500");
      expect(state.cells.get("A3")?.computed).toBe(750);
    });

    it("evaluates statistical formulas: AVERAGE, MIN, MAX, and COUNT", () => {
      state.setCellValue("B1", "10");
      state.setCellValue("B2", "20");
      state.setCellValue("B3", "30");

      expect(state.evaluateFormula("=AVERAGE(B1:B3)")).toBe(20);
      expect(state.evaluateFormula("=MIN(B1:B3)")).toBe(10);
      expect(state.evaluateFormula("=MAX(B1:B3)")).toBe(30);
      expect(state.evaluateFormula("=COUNT(B1:B3)")).toBe(3);
    });

    it("evaluates logical =IF(condition, trueVal, falseVal) formula", () => {
      state.setCellValue("C1", "75");
      expect(state.evaluateFormula('=IF(C1>50, "Pass", "Fail")')).toBe("Pass");

      state.setCellValue("C1", "25");
      expect(state.evaluateFormula('=IF(C1>50, "Pass", "Fail")')).toBe("Fail");

      state.setCellValue("C2", "OK");
      expect(state.evaluateFormula('=IF(C2="OK", 100, 0)')).toBe(100);
      expect(state.evaluateFormula('=IF(C1<=10, "Low", "High")')).toBe("High");
    });

    it("evaluates lookup =VLOOKUP(lookupVal, tableRange, colIdx, approx) formula", () => {
      // Build small catalog table in E1:F3
      state.setCellValue("E1", "101");
      state.setCellValue("F1", "Apples");
      state.setCellValue("E2", "102");
      state.setCellValue("F2", "Bananas");
      state.setCellValue("E3", "103");
      state.setCellValue("F3", "Cherries");

      expect(state.evaluateFormula("=VLOOKUP(102, E1:F3, 2, FALSE)")).toBe("Bananas");
      expect(state.evaluateFormula("=VLOOKUP(101, E1:F3, 2, FALSE)")).toBe("Apples");
      expect(state.evaluateFormula("=VLOOKUP(999, E1:F3, 2, FALSE)")).toBe("#N/A");
      expect(state.evaluateFormula("=VLOOKUP(102, E1:F3, 5, FALSE)")).toBe("#REF!");
    });

    it("evaluates arithmetic expressions including powers and coordinates", () => {
      state.setCellValue("D1", "5");
      state.setCellValue("D2", "4");

      expect(state.evaluateFormula("=D1*D2+100")).toBe(120);
      expect(state.evaluateFormula("=D1^2")).toBe(25);
      expect(state.evaluateFormula("=(D1+D2)*2")).toBe(18);
    });
  });

  describe("4. Multi-Sheet Management (SheetTabBar & WorkbookSheetsPane)", () => {
    it("renders default Sheet1 and adds Sheet2 on clicking + button", () => {
      const tabBarContainer = document.createElement("div");
      document.body.appendChild(tabBarContainer);
      const sheetBar = new SheetTabBar(tabBarContainer, state);
      expect(sheetBar).toBeDefined();

      expect(state.sheets.length).toBe(1);
      expect(state.sheets[0].name).toBe("Sheet1");

      const addBtn = tabBarContainer.querySelector<HTMLButtonElement>(".btn-add-sheet")!;
      expect(addBtn).not.toBeNull();
      addBtn.click();

      expect(state.sheets.length).toBe(2);
      expect(state.sheets[1].name).toBe("Sheet2");
      expect(state.activeSheetId).toBe(state.sheets[1].id);
      expect(state.sheets[1].cells.size).toBe(0);
    });

    it("allows double-click inline renaming and prevents invalid names", () => {
      const tabBarContainer = document.createElement("div");
      document.body.appendChild(tabBarContainer);
      const sheetBar = new SheetTabBar(tabBarContainer, state);
      expect(sheetBar).toBeDefined();

      const ok = state.renameSheet("sheet-1", "Budget2026");
      expect(ok).toBe(true);
      expect(state.sheets[0].name).toBe("Budget2026");

      expect(state.renameSheet("sheet-1", "")).toBe(false);
    });

    it("deletes sheet while preserving minimum 1 sheet", () => {
      const newSheet = state.addSheet("TempSheet");
      expect(state.sheets.length).toBe(2);

      const deleted = state.deleteSheet(newSheet.id);
      expect(deleted).toBe(true);
      expect(state.sheets.length).toBe(1);

      // Cannot delete the only remaining sheet
      const cantDelete = state.deleteSheet(state.sheets[0].id);
      expect(cantDelete).toBe(false);
      expect(state.sheets.length).toBe(1);
    });

    it("synchronizes WorkbookSheetsPane with sheet additions and removals", () => {
      const pane = new WorkbookSheetsPane("sidebar-container", state);
      expect(document.querySelector(".sheets-list")).not.toBeNull();
      expect(document.querySelectorAll(".sheet-item").length).toBe(1);

      state.addSheet("QuarterlyPlan");
      pane.refresh();
      expect(document.querySelectorAll(".sheet-item").length).toBe(2);
    });
  });

  describe("5. UnifiedShellApp Full Calc Lifecycle Integration", () => {
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
          </div>
        </div>
      `;

      app = new UnifiedShellApp(false);
      app.applyBootConfig({ app_mode: "calc" });
    });

    it("boots directly into Calc mode with 100% blank Sheet1 and empty FormulaBar", () => {
      expect(app.getCurrentDocType()).toBe("Excel");
      const fBar = app.getFormulaBar();
      expect(fBar).toBeDefined();
      expect(fBar?.getFormulaInput().value).toBe("");

      const calcCanvas = app.getCalcGridCanvas();
      expect(calcCanvas).toBeDefined();

      const docInfo = document.getElementById("status-doc-info");
      expect(docInfo?.textContent).toContain("Sheet 1 of");
    });

    it("supports entering data in A1 and A2 and calculating =SUM(A1:A2) in A3 through FormulaBar", () => {
      const fBar = app.getFormulaBar()!;
      const calcCanvas = app.getCalcGridCanvas()!;

      calcCanvas.selectCell("A1");
      fBar.getFormulaInput().value = "45";
      fBar.getFormulaInput().dispatchEvent(new Event("input"));
      const commitBtn = document.querySelector(".btn-formula-commit") as HTMLButtonElement;
      commitBtn.click();

      calcCanvas.selectCell("A2");
      fBar.getFormulaInput().value = "55";
      commitBtn.click();

      calcCanvas.selectCell("A3");
      fBar.getFormulaInput().value = "=SUM(A1:A2)";
      commitBtn.click();

      expect(fBar.getFormulaInput().value).toBe("=SUM(A1:A2)");
      const activeTab = app.getTabBar().getActiveTab();
      expect(activeTab?.isDirty).toBe(true);
    });
  });
});
