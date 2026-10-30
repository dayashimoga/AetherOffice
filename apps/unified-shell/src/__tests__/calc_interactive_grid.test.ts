import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { CalcGridCanvas } from "../components/viewports/CalcGridCanvas";
import { CalcState } from "../state/calcState";
import { FormulaBar } from "../components/FormulaBar";
import { SheetTabBar } from "../components/SheetTabBar";
import { WorkbookSheetsPane } from "../components/WorkbookSheetsPane";
import { UnifiedShellApp } from "../main";

describe("Editable Spreadsheet Grid, In-Cell Input, Multi-Sheet Tabs & Column Resizing (GEC-v2.0 Subphase 10.1.1)", () => {
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

  describe("1. In-Cell Input Editor & Real-Time Formula Bar Synchronization", () => {
    it("activates in-cell editor on pressing F2 or double clicking", () => {
      canvas.selectCell("A1");
      expect(canvas.isCellEditing()).toBe(false);

      canvas.startEditing();
      expect(canvas.isCellEditing()).toBe(true);

      const editor = canvas.getEditorInput();
      expect(editor.style.display).toBe("block");
      expect(editor.value).toBe("");

      canvas.cancelEditing();
      expect(canvas.isCellEditing()).toBe(false);
      expect(editor.style.display).toBe("none");
    });

    it("activates in-cell editor with printable keystroke on selected cell", () => {
      canvas.selectCell("B2");
      const canvasEl = container.querySelector("canvas")!;

      canvasEl.dispatchEvent(new KeyboardEvent("keydown", { key: "9" }));
      expect(canvas.isCellEditing()).toBe(true);

      const editor = canvas.getEditorInput();
      expect(editor.value).toBe("9");
    });

    it("synchronizes in-cell editor input in real time with FormulaBar", () => {
      const formulaBar = new FormulaBar("sub-header-container");
      formulaBar.setCallbacks({
        onInput: (val) => canvas.setEditingValue(val),
        onCommit: (_val) => canvas.commitEditing("down"),
      });

      canvas.setOnCellEditing((coord, val) => {
        formulaBar.setActiveCell(coord, val);
      });

      canvas.selectCell("A2");
      canvas.startEditing();

      const editor = canvas.getEditorInput();
      editor.value = "17500";
      editor.dispatchEvent(new Event("input"));

      // FormulaBar receives live update
      expect(formulaBar.getFormulaInput().value).toBe("17500");

      // Modifying FormulaBar updates in-cell editor
      formulaBar.getFormulaInput().value = "=SUM(B2:B3)";
      formulaBar.getFormulaInput().dispatchEvent(new Event("input"));
      expect(editor.value).toBe("=SUM(B2:B3)");
    });

    it("commits cell value and navigates down on Enter, up on Shift+Enter, right on Tab, left on Shift+Tab", () => {
      canvas.selectCell("C1");
      canvas.startEditing("TestValue");

      // Enter commits and moves down to C2
      canvas.commitEditing("down");
      expect(state.cells.get("C1")?.raw).toBe("TestValue");
      expect(state.selectedCell).toBe("C2");

      // Shift+Enter commits and moves up to C1
      canvas.startEditing("Val2");
      canvas.commitEditing("up");
      expect(state.cells.get("C2")?.raw).toBe("Val2");
      expect(state.selectedCell).toBe("C1");

      // Tab commits and moves right to D1
      canvas.startEditing("Val3");
      canvas.commitEditing("right");
      expect(state.selectedCell).toBe("D1");

      // Shift+Tab commits and moves left to C1
      canvas.startEditing("Val4");
      canvas.commitEditing("left");
      expect(state.selectedCell).toBe("C1");
    });

    it("cancels in-cell editing on Escape without mutating cell data", () => {
      canvas.selectCell("A1");
      const initialVal = state.cells.get("A1")?.raw;

      canvas.startEditing();
      const editor = canvas.getEditorInput();
      editor.value = "MUTATED_VALUE";

      canvas.cancelEditing();
      expect(canvas.isCellEditing()).toBe(false);
      expect(state.cells.get("A1")?.raw).toBe(initialVal);
    });
  });

  describe("2. Formula Evaluation & Dynamic Dependency Recalculation", () => {
    it("evaluates =SUM(A1:A3) and updates when dependent values change", () => {
      state.selectCell("A1");
      state.commitFormula("10");

      state.selectCell("A2");
      state.commitFormula("20");

      state.selectCell("A3");
      state.commitFormula("30");

      state.selectCell("A4");
      state.commitFormula("=SUM(A1:A3)");

      expect(state.cells.get("A4")?.computed).toBe(60);

      // Mutate A1 from 10 to 50
      state.selectCell("A1");
      state.commitFormula("50");

      // DAG recalculation automatically updates A4 to 100
      expect(state.cells.get("A4")?.computed).toBe(100);
    });

    it("evaluates AVERAGE, MIN, MAX, and COUNT functions", () => {
      state.selectCell("D1");
      state.commitFormula("10");
      state.selectCell("D2");
      state.commitFormula("20");
      state.selectCell("D3");
      state.commitFormula("30");

      expect(state.evaluateFormula("=AVERAGE(D1:D3)")).toBe(20);
      expect(state.evaluateFormula("=MIN(D1:D3)")).toBe(10);
      expect(state.evaluateFormula("=MAX(D1:D3)")).toBe(30);
      expect(state.evaluateFormula("=COUNT(D1:D3)")).toBe(3);
    });

    it("evaluates arithmetic expressions with cell coordinates", () => {
      state.selectCell("E1");
      state.commitFormula("100");
      state.selectCell("E2");
      state.commitFormula("25");

      expect(state.evaluateFormula("=E1+E2")).toBe(125);
      expect(state.evaluateFormula("=E1-E2")).toBe(75);
      expect(state.evaluateFormula("=E1*E2")).toBe(2500);
      expect(state.evaluateFormula("=E1/E2")).toBe(4);
    });
  });

  describe("3. Multi-Sheet Management (SheetTabBar & WorkbookSheetsPane)", () => {
    it("renders default Sheet1 and adds Sheet2 independently", () => {
      const tabBarContainer = document.createElement("div");
      document.body.appendChild(tabBarContainer);
      const sheetBar = new SheetTabBar(tabBarContainer, state);
      expect(sheetBar).toBeDefined();

      expect(state.sheets.length).toBe(1);
      expect(state.sheets[0].name).toBe("Sheet1");

      // Set cell in Sheet1 first
      state.selectCell("A1");
      state.commitFormula("Sheet1 Data");

      const addBtn = tabBarContainer.querySelector<HTMLButtonElement>(".btn-add-sheet")!;
      expect(addBtn).not.toBeNull();
      addBtn.click();

      expect(state.sheets.length).toBe(2);
      expect(state.sheets[1].name).toBe("Sheet2");
      expect(state.activeSheetId).toBe(state.sheets[1].id);

      // Sheet2 is an independent grid
      state.selectCell("A1");
      state.commitFormula("Sheet2 Data");
      expect(state.sheets[1].cells.get("A1")?.raw).toBe("Sheet2 Data");

      // Switch back to Sheet1
      state.switchSheet("sheet-1");
      expect(state.sheets[0].cells.get("A1")?.raw).toBe("Sheet1 Data");
    });

    it("renames sheet via double click or rename method", () => {
      const tabBarContainer = document.createElement("div");
      document.body.appendChild(tabBarContainer);
      const sheetBar = new SheetTabBar(tabBarContainer, state);
      expect(sheetBar).toBeDefined();

      const renamed = state.renameSheet("sheet-1", "Budget2026");
      expect(renamed).toBe(true);
      expect(state.sheets[0].name).toBe("Budget2026");

      // Rejects empty or duplicate names
      expect(state.renameSheet("sheet-1", "")).toBe(false);
    });

    it("deletes sheet while preserving minimum 1 sheet", () => {
      const newSheet = state.addSheet("Temporary");
      expect(state.sheets.length).toBe(2);

      const deleted = state.deleteSheet(newSheet.id);
      expect(deleted).toBe(true);
      expect(state.sheets.length).toBe(1);

      // Prevent deleting last sheet
      const cantDelete = state.deleteSheet(state.sheets[0].id);
      expect(cantDelete).toBe(false);
      expect(state.sheets.length).toBe(1);
    });

    it("synchronizes left WorkbookSheetsPane with sheet actions", () => {
      const pane = new WorkbookSheetsPane("sidebar-container", state);
      expect(document.querySelector(".sheets-list")).not.toBeNull();
      expect(document.querySelectorAll(".sheet-item").length).toBe(1);

      state.addSheet("Quarterly");
      pane.refresh();
      expect(document.querySelectorAll(".sheet-item").length).toBe(2);
    });
  });

  describe("4. Row & Column Resizing & Context Menu Manipulation", () => {
    it("resizes column width and row height dynamically", () => {
      expect(state.getColWidth(0)).toBe(96);
      state.setColWidth(0, 150);
      expect(state.getColWidth(0)).toBe(150);

      expect(state.getRowHeight(0)).toBe(24);
      state.setRowHeight(0, 48);
      expect(state.getRowHeight(0)).toBe(48);

      expect(() => canvas.render()).not.toThrow();
    });

    it("inserts and deletes rows shifting existing cell coordinates", () => {
      state.selectCell("A1");
      state.commitFormula("Row 1");
      state.selectCell("A2");
      state.commitFormula("Row 2");

      // Insert row at row 1 (0-indexed row 0)
      state.insertRow(0);
      expect(state.cells.get("A2")?.raw).toBe("Row 1");
      expect(state.cells.get("A3")?.raw).toBe("Row 2");

      // Delete inserted row
      state.deleteRow(0);
      expect(state.cells.get("A1")?.raw).toBe("Row 1");
      expect(state.cells.get("A2")?.raw).toBe("Row 2");
    });

    it("inserts and deletes columns shifting existing cell coordinates", () => {
      state.selectCell("A1");
      state.commitFormula("Col A");
      state.selectCell("B1");
      state.commitFormula("Col B");

      // Insert column at col 0 (Col A)
      state.insertColumn(0);
      expect(state.cells.get("B1")?.raw).toBe("Col A");
      expect(state.cells.get("C1")?.raw).toBe("Col B");

      // Delete inserted column
      state.deleteColumn(0);
      expect(state.cells.get("A1")?.raw).toBe("Col A");
      expect(state.cells.get("B1")?.raw).toBe("Col B");
    });

    it("clears row and column contents cleanly", () => {
      state.selectCell("A5");
      state.commitFormula("Target5");
      state.clearRow(4); // row 5 is index 4
      expect(state.cells.get("A5")?.raw).toBe("");

      state.selectCell("F1");
      state.commitFormula("TargetF");
      state.clearColumn(5); // col F is index 5
      expect(state.cells.get("F1")?.raw).toBe("");
    });
  });

  describe("5. UnifiedShellApp Calc Mode Full Integration", () => {
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

    it("boots directly into Calc mode mounting FormulaBar, GridCanvas, SheetTabBar, and WorkbookSheetsPane", () => {
      expect(app.getCurrentDocType()).toBe("Excel");
      expect(app.getFormulaBar()).toBeDefined();
      expect(app.getCalcGridCanvas()).toBeDefined();
      expect(app.getSheetTabBar()).toBeDefined();
      expect(app.getWorkbookSheetsPane()).toBeDefined();

      const docInfo = document.getElementById("status-doc-info");
      expect(docInfo?.textContent).toContain("Sheet 1 of");
    });

    it("marks active document tab dirty when editing cells in Calc", () => {
      const activeTab = app.getTabBar().getActiveTab();
      expect(activeTab).toBeDefined();

      const calcCv = app.getCalcGridCanvas()!;
      calcCv.selectCell("A2");
      calcCv.startEditing("99999");
      calcCv.commitEditing("down");

      expect(activeTab?.isDirty).toBe(true);
    });

    it("applies Ribbon actions (ToggleBold, SetFontColor, InsertRow) in Calc mode", () => {
      const activeTab = app.getTabBar().getActiveTab()!;

      (app as any).handleRibbonAction("ToggleBold");
      expect(activeTab.isDirty).toBe(true);

      (app as any).handleRibbonAction("SetFontColor", "#2563eb");
      expect(activeTab.isDirty).toBe(true);

      (app as any).handleRibbonAction("InsertRow");
      expect(activeTab.isDirty).toBe(true);
    });
  });
});
