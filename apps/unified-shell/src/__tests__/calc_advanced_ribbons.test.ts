import { describe, it, expect, beforeEach } from "vitest";
import { CalcState, calcState, formatCellValue, CellData } from "../state/calcState";
import { FunctionWizardDialog, EXCEL_FUNCTIONS } from "../components/FunctionWizardDialog";
import { UnifiedShellApp } from "../main";

describe("Calc Home, Formulas & Data Ribbons: Number Formats, Function Wizard & Sort/Filter (GEC-v2.0 Subphase 10.2.1)", () => {
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

  describe("1. Cell Number Formats & Decimal Control", () => {
    it("formats numbers as Currency ($) with standard 2 decimal places and handles negative values", () => {
      const cellPos: CellData = { raw: "15000", computed: 15000, format: "Currency" };
      expect(formatCellValue(cellPos)).toBe("$15,000.00");

      const cellNeg: CellData = { raw: "-525.75", computed: -525.75, format: "Currency" };
      expect(formatCellValue(cellNeg)).toBe("-$525.75");

      const cellZero: CellData = { raw: "0", computed: 0, format: "Currency" };
      expect(formatCellValue(cellZero)).toBe("$0.00");
    });

    it("formats numbers as Percentage (%) multiplying by 100", () => {
      const cell: CellData = { raw: "0.155", computed: 0.155, format: "Percentage" };
      expect(formatCellValue(cell)).toBe("15.50%");

      const cellWhole: CellData = { raw: "1", computed: 1, format: "Percentage" };
      expect(formatCellValue(cellWhole)).toBe("100.00%");
    });

    it("formats numbers with Comma separators and Accounting style", () => {
      const cellComma: CellData = { raw: "1234567.89", computed: 1234567.89, format: "Comma" };
      expect(formatCellValue(cellComma)).toBe("1,234,567.89");

      const cellAcct: CellData = { raw: "-2500", computed: -2500, format: "Accounting" };
      expect(formatCellValue(cellAcct)).toBe("($ 2,500.00)");
    });

    it("increases and decreases decimal places dynamically", () => {
      calc.cells.set("A1", { raw: "42", computed: 42, format: "Currency", decimals: 2 });
      calc.selectedRanges = [{ minRow: 0, maxRow: 0, minCol: 0, maxCol: 0 }];

      calc.increaseDecimal();
      expect(calc.cells.get("A1")?.decimals).toBe(3);
      expect(formatCellValue(calc.cells.get("A1")!)).toBe("$42.000");

      calc.decreaseDecimal();
      calc.decreaseDecimal();
      expect(calc.cells.get("A1")?.decimals).toBe(1);
      expect(formatCellValue(calc.cells.get("A1")!)).toBe("$42.0");

      // Cannot decrease below 0
      calc.decreaseDecimal();
      calc.decreaseDecimal();
      expect(calc.cells.get("A1")?.decimals).toBe(0);
      expect(formatCellValue(calc.cells.get("A1")!)).toBe("$42");
    });

    it("applies setNumberFormat across a multi-cell range", () => {
      calc.cells.set("B1", { raw: "100", computed: 100 });
      calc.cells.set("B2", { raw: "200", computed: 200 });
      calc.selectedRanges = [{ minRow: 0, maxRow: 1, minCol: 1, maxCol: 1 }];

      calc.setNumberFormat("Currency");
      expect(calc.cells.get("B1")?.format).toBe("Currency");
      expect(calc.cells.get("B2")?.format).toBe("Currency");
      expect(formatCellValue(calc.cells.get("B1")!)).toBe("$100.00");
      expect(formatCellValue(calc.cells.get("B2")!)).toBe("$200.00");
    });
  });

  describe("2. Alignment, Borders, Wrap Text, and Merge & Center", () => {
    it("sets horizontal and vertical text alignments across cells", () => {
      calc.cells.set("A1", { raw: "Title", computed: "Title" });
      calc.selectedRanges = [{ minRow: 0, maxRow: 0, minCol: 0, maxCol: 0 }];

      calc.setAlignment("center");
      calc.setVerticalAlignment("middle");
      expect(calc.cells.get("A1")?.align).toBe("center");
      expect(calc.cells.get("A1")?.valign).toBe("middle");

      calc.setAlignment("right");
      expect(calc.cells.get("A1")?.align).toBe("right");
    });

    it("toggles wrapText and cell borders", () => {
      calc.cells.set("A1", { raw: "Long text wrapping sample", computed: "Long text wrapping sample" });
      calc.selectedRanges = [{ minRow: 0, maxRow: 0, minCol: 0, maxCol: 0 }];

      calc.toggleWrapText();
      expect(calc.cells.get("A1")?.wrapText).toBe(true);
      calc.toggleWrapText();
      expect(calc.cells.get("A1")?.wrapText).toBe(false);

      calc.setBorders("all");
      expect(calc.cells.get("A1")?.borders).toBe("all");
      calc.setBorders("bottom");
      expect(calc.cells.get("A1")?.borders).toBe("bottom");
    });

    it("applies Merge & Center across selected cells", () => {
      calc.cells.set("A1", { raw: "Sales Report", computed: "Sales Report" });
      calc.cells.set("B1", { raw: "junk", computed: "junk" });
      calc.cells.set("C1", { raw: "junk2", computed: "junk2" });
      calc.selectedRanges = [{ minRow: 0, maxRow: 0, minCol: 0, maxCol: 2 }];

      calc.mergeAndCenter();
      expect(calc.cells.get("A1")?.computed).toBe("Sales Report");
      expect(calc.cells.get("A1")?.align).toBe("center");
      expect(calc.cells.has("B1")).toBe(false);
      expect(calc.cells.has("C1")).toBe(false);

      const merged = calc.getActiveSheet().mergedRanges;
      expect(merged).toBeDefined();
      expect(merged?.length).toBeGreaterThan(0);
      expect(merged?.[0]).toEqual({ minRow: 0, maxRow: 0, minCol: 0, maxCol: 2 });
    });
  });

  describe("3. AutoSum Automatic Range Detection & Formula Generation", () => {
    it("inserts =SUM() below a multi-cell selection", () => {
      calc.cells.set("A1", { raw: "10", computed: 10 });
      calc.cells.set("A2", { raw: "20", computed: 20 });
      calc.cells.set("A3", { raw: "30", computed: 30 });
      calc.selectedRanges = [{ minRow: 0, maxRow: 2, minCol: 0, maxCol: 0 }];

      const formula = calc.insertAutoSum("SUM");
      expect(formula).toBe("=SUM(A1:A3)");
      expect(calc.selectedCell).toBe("A4");
      expect(calc.cells.get("A4")?.raw).toBe("=SUM(A1:A3)");
      expect(calc.cells.get("A4")?.computed).toBe(60);
    });

    it("detects contiguous numeric cells above a single selected empty cell", () => {
      calc.cells.set("B1", { raw: "100", computed: 100 });
      calc.cells.set("B2", { raw: "250", computed: 250 });
      calc.cells.set("B3", { raw: "150", computed: 150 });
      calc.selectedCell = "B4";
      calc.selectedRanges = [{ minRow: 3, maxRow: 3, minCol: 1, maxCol: 1 }];

      const formula = calc.insertAutoSum("AVERAGE");
      expect(formula).toBe("=AVERAGE(B1:B3)");
      expect(calc.cells.get("B4")?.raw).toBe("=AVERAGE(B1:B3)");
      expect(calc.cells.get("B4")?.computed).toBe((100 + 250 + 150) / 3);
    });

    it("detects contiguous numeric cells to the left when no cells are above", () => {
      calc.cells.set("A5", { raw: "5", computed: 5 });
      calc.cells.set("B5", { raw: "10", computed: 10 });
      calc.cells.set("C5", { raw: "15", computed: 15 });
      calc.selectedCell = "D5";
      calc.selectedRanges = [{ minRow: 4, maxRow: 4, minCol: 3, maxCol: 3 }];

      const formula = calc.insertAutoSum("MAX");
      expect(formula).toBe("=MAX(A5:C5)");
      expect(calc.cells.get("D5")?.raw).toBe("=MAX(A5:C5)");
      expect(calc.cells.get("D5")?.computed).toBe(15);
    });
  });

  describe("4. Function Wizard Dialog (fx Modal)", () => {
    it("initializes with standard functions across categories", () => {
      const dialog = new FunctionWizardDialog();
      expect(EXCEL_FUNCTIONS.length).toBeGreaterThanOrEqual(25);
      expect(EXCEL_FUNCTIONS.some((f) => f.name === "SUM")).toBe(true);
      expect(EXCEL_FUNCTIONS.some((f) => f.name === "VLOOKUP")).toBe(true);
      expect(EXCEL_FUNCTIONS.some((f) => f.name === "PMT")).toBe(true);
      expect(dialog.isOpen()).toBe(false);
    });

    it("shows and filters functions by search query and category", () => {
      const dialog = new FunctionWizardDialog();
      dialog.show("Financial");
      expect(dialog.isOpen()).toBe(true);

      const modalEl = document.getElementById("function-wizard-modal");
      expect(modalEl).not.toBeNull();
      expect(modalEl?.style.display).toBe("flex");

      const catSelect = modalEl?.querySelector<HTMLSelectElement>(".fn-category-select");
      expect(catSelect?.value).toBe("Financial");

      const searchInput = modalEl?.querySelector<HTMLInputElement>(".fn-search-input");
      if (searchInput) {
        searchInput.value = "PMT";
        searchInput.dispatchEvent(new Event("input"));
      }

      const listItems = modalEl?.querySelectorAll(".fn-item");
      expect(listItems?.length).toBeGreaterThan(0);
      expect(listItems?.[0].textContent).toBe("PMT");

      dialog.hide();
      expect(dialog.isOpen()).toBe(false);
      expect(modalEl?.style.display).toBe("none");
    });

    it("inserts formula example when Insert button is clicked", () => {
      const dialog = new FunctionWizardDialog();
      let insertedFormula = "";
      dialog.setOnInsert((formula) => {
        insertedFormula = formula;
      });

      dialog.show("Math & Trig");
      const modalEl = document.getElementById("function-wizard-modal");
      const insertBtn = modalEl?.querySelector<HTMLButtonElement>(".btn-insert-fn");
      insertBtn?.click();

      expect(insertedFormula).toContain("=");
      expect(dialog.isOpen()).toBe(false);
    });
  });

  describe("5. Data Ribbon: Sort, Filter, Text to Columns, Flash Fill & Remove Duplicates", () => {
    it("sorts rows ascending and descending based on active column values", () => {
      calc.cells.set("A1", { raw: "Banana", computed: "Banana" });
      calc.cells.set("B1", { raw: "30", computed: 30 });

      calc.cells.set("A2", { raw: "Apple", computed: "Apple" });
      calc.cells.set("B2", { raw: "10", computed: 10 });

      calc.cells.set("A3", { raw: "Cherry", computed: "Cherry" });
      calc.cells.set("B3", { raw: "20", computed: 20 });

      calc.selectedRanges = [{ minRow: 0, maxRow: 2, minCol: 0, maxCol: 1 }];

      // Sort Ascending by Column A (alphabetical)
      calc.sortRange(true, 0);
      expect(calc.cells.get("A1")?.computed).toBe("Apple");
      expect(calc.cells.get("B1")?.computed).toBe(10);
      expect(calc.cells.get("A2")?.computed).toBe("Banana");
      expect(calc.cells.get("B2")?.computed).toBe(30);
      expect(calc.cells.get("A3")?.computed).toBe("Cherry");
      expect(calc.cells.get("B3")?.computed).toBe(20);

      // Sort Descending by Column B (numeric)
      calc.sortRange(false, 1);
      expect(calc.cells.get("A1")?.computed).toBe("Banana");
      expect(calc.cells.get("B1")?.computed).toBe(30);
      expect(calc.cells.get("A2")?.computed).toBe("Cherry");
      expect(calc.cells.get("B2")?.computed).toBe(20);
      expect(calc.cells.get("A3")?.computed).toBe("Apple");
      expect(calc.cells.get("B3")?.computed).toBe(10);
    });

    it("toggles and clears filter state on active sheet", () => {
      calc.selectedRanges = [{ minRow: 0, maxRow: 0, minCol: 0, maxCol: 3 }];

      const isFilterActive = calc.toggleFilter();
      expect(isFilterActive).toBe(true);
      expect(calc.getActiveSheet().filterActive).toBe(true);
      expect(calc.getActiveSheet().filterRange).toEqual({ minRow: 0, maxRow: 0, minCol: 0, maxCol: 3 });

      calc.filterColumn(0, (val) => val === "Active");
      expect(calc.getActiveSheet().hiddenRows).toBeDefined();

      calc.clearFilter();
      expect(calc.getActiveSheet().hiddenRows?.size).toBe(0);

      const toggledOff = calc.toggleFilter();
      expect(toggledOff).toBe(false);
      expect(calc.getActiveSheet().filterActive).toBe(false);
    });

    it("splits delimited cell strings across adjacent columns using textToColumns", () => {
      calc.cells.set("A1", { raw: "John,Doe,35", computed: "John,Doe,35" });
      calc.cells.set("A2", { raw: "Jane,Smith,28", computed: "Jane,Smith,28" });
      calc.selectedRanges = [{ minRow: 0, maxRow: 1, minCol: 0, maxCol: 0 }];

      calc.textToColumns(",");

      expect(calc.cells.get("A1")?.computed).toBe("John");
      expect(calc.cells.get("B1")?.computed).toBe("Doe");
      expect(calc.cells.get("C1")?.computed).toBe(35);

      expect(calc.cells.get("A2")?.computed).toBe("Jane");
      expect(calc.cells.get("B2")?.computed).toBe("Smith");
      expect(calc.cells.get("C2")?.computed).toBe(28);
    });

    it("applies flashFill pattern from adjacent source column", () => {
      calc.cells.set("A1", { raw: "first.last@company.com", computed: "first.last@company.com" });
      calc.cells.set("B1", { raw: "first.last", computed: "first.last" });

      calc.cells.set("A2", { raw: "alice.wonder@company.com", computed: "alice.wonder@company.com" });
      calc.cells.set("A3", { raw: "bob.builder@company.com", computed: "bob.builder@company.com" });

      // Target selection is Column B rows 0..2
      calc.selectedRanges = [{ minRow: 0, maxRow: 2, minCol: 1, maxCol: 1 }];

      calc.flashFill();

      expect(calc.cells.get("B2")?.computed).toBe("alice.wonder");
      expect(calc.cells.get("B3")?.computed).toBe("bob.builder");
    });

    it("removes duplicate rows within the selected range", () => {
      calc.cells.set("A1", { raw: "North", computed: "North" });
      calc.cells.set("B1", { raw: "100", computed: 100 });

      calc.cells.set("A2", { raw: "South", computed: "South" });
      calc.cells.set("B2", { raw: "200", computed: 200 });

      calc.cells.set("A3", { raw: "North", computed: "North" });
      calc.cells.set("B3", { raw: "100", computed: 100 });

      calc.selectedRanges = [{ minRow: 0, maxRow: 2, minCol: 0, maxCol: 1 }];

      const removed = calc.removeDuplicates();
      expect(removed).toBe(1);
      expect(calc.cells.has("A1")).toBe(true);
      expect(calc.cells.has("A2")).toBe(true);
      expect(calc.cells.has("A3")).toBe(false);
      expect(calc.cells.has("B3")).toBe(false);
    });
  });

  describe("6. UnifiedShellApp Ribbon & Keyboard Integration", () => {
    it("handles Calc ribbon actions for Number Formats, Alignment, AutoSum, and Sort/Filter", () => {
      const app = new UnifiedShellApp(true);
      const tabBar = app.getTabBar();
      tabBar.addTab({ id: "tab-calc-test", docType: "Excel", title: "Test.xlsx", isDirty: false });
      tabBar.switchTab("tab-calc-test");

      const ribbon = app.getRibbon();

      // Number formatting through ribbon
      ribbon.getActiveAppRibbon().renderTab("Home", document.createElement("div"), (action, payload) => {
        app.handleRibbonAction(action, payload);
      });

      app.handleRibbonAction("SetNumberFormat", "Currency");
      expect(calcState.cells.get(calcState.selectedCell)?.format).toBe("Currency");

      app.handleRibbonAction("IncreaseDecimal");
      expect(calcState.cells.get(calcState.selectedCell)?.decimals).toBe(3);

      // Alignment actions
      app.handleRibbonAction("AlignCenter");
      expect(calcState.cells.get(calcState.selectedCell)?.align).toBe("center");

      // AutoSum action
      app.handleRibbonAction("AutoSum");
      expect(calcState.cells.get(calcState.selectedCell)?.raw).toContain("=");

      // Data actions
      app.handleRibbonAction("ToggleFilter");
      expect(calcState.getActiveSheet().filterActive).toBe(true);

      app.handleRibbonAction("ClearFilter");
      expect(calcState.getActiveSheet().hiddenRows?.size).toBe(0);

      // Function Wizard action opens dialog
      app.handleRibbonAction("InsertFunction");
      expect(app.getFunctionWizardDialog().isOpen()).toBe(true);
      app.getFunctionWizardDialog().hide();
    });
  });
});
