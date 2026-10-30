import { describe, it, expect, beforeEach, vi } from "vitest";
import { UnifiedShellApp } from "../main";
import { calcState } from "../state/calcState";
import { IPC } from "../ipc";

describe("Subphase 30.1.2: Excel Formulas, Data, Review & View Ribbons (100% Working Controls)", () => {
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
      closePath: vi.fn(),
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

  describe("Formulas Tab", () => {
    beforeEach(() => {
      app.getRibbon().setActiveTab("Formulas");
    });

    it("wires Insert Function wizard modal", () => {
      const btn = document.querySelector('[data-testid="calc-btn-insert-fn"]') as HTMLButtonElement;
      expect(btn).not.toBeNull();
      btn.click();
      expect(app.getFunctionWizardDialog().isOpen()).toBe(true);
      app.getFunctionWizardDialog().hide();
    });

    it("wires AutoSum and aggregate functions (Sum, Average, Count, Max, Min)", () => {
      calcState.setCellValue("A1", "10");
      calcState.setCellValue("A2", "20");
      calcState.setCellValue("A3", "30");
      calcState.selectedRanges = [{ minRow: 0, maxRow: 2, minCol: 0, maxCol: 0 }];

      const btnSum = document.querySelector('[data-testid="calc-btn-autosum"]') as HTMLButtonElement;
      btnSum.click();
      expect(calcState.getCellValue("A4")).toBe(60);

      const btnAvg = document.querySelector('[data-testid="calc-btn-avg"]') as HTMLButtonElement;
      btnAvg.click();
      expect(calcState.getCellValue("A4")).toBe(20);

      const btnCount = document.querySelector('[data-testid="calc-btn-count"]') as HTMLButtonElement;
      btnCount.click();
      expect(calcState.getCellValue("A4")).toBe(3);

      const btnMax = document.querySelector('[data-testid="calc-btn-max"]') as HTMLButtonElement;
      btnMax.click();
      expect(calcState.getCellValue("A4")).toBe(30);

      const btnMin = document.querySelector('[data-testid="calc-btn-min"]') as HTMLButtonElement;
      btnMin.click();
      expect(calcState.getCellValue("A4")).toBe(10);
    });

    it("wires Category buttons and evaluates Financial functions (PMT, FV, PV, NPV, IRR)", () => {
      const finBtn = document.querySelector('[data-testid="calc-btn-fn-financial"]') as HTMLButtonElement;
      expect(finBtn).not.toBeNull();
      finBtn.click();
      expect(app.getFunctionWizardDialog().isOpen()).toBe(true);
      app.getFunctionWizardDialog().hide();

      // Test Financial formula evaluations in calcState
      const pmtRes = calcState.evaluateFormula("=PMT(0.05/12, 60, 20000)");
      expect(typeof pmtRes).toBe("number");
      expect(pmtRes).toBeLessThan(0);

      const fvRes = calcState.evaluateFormula("=FV(0.05/12, 12, -100)");
      expect(typeof fvRes).toBe("number");
      expect(Number(fvRes)).toBeGreaterThan(1200);

      const pvRes = calcState.evaluateFormula("=PV(0.05/12, 12, -100)");
      expect(typeof pvRes).toBe("number");
      expect(Number(pvRes)).toBeGreaterThan(1100);

      calcState.setCellValue("A1", "-1000");
      calcState.setCellValue("A2", "400");
      calcState.setCellValue("A3", "400");
      calcState.setCellValue("A4", "400");
      const npvRes = calcState.evaluateFormula("=NPV(0.1, A2:A4)");
      expect(Number(npvRes)).toBeGreaterThan(900);

      const irrRes = calcState.evaluateFormula("=IRR(A1:A4)");
      expect(typeof irrRes).toBe("number");
      expect(Number(irrRes)).toBeGreaterThan(0.05);
    });

    it("wires Logical category button and evaluates IF, AND, OR, NOT, IFERROR, IFS", () => {
      const logBtn = document.querySelector('[data-testid="calc-btn-fn-logical"]') as HTMLButtonElement;
      expect(logBtn).not.toBeNull();
      logBtn.click();
      expect(app.getFunctionWizardDialog().isOpen()).toBe(true);
      app.getFunctionWizardDialog().hide();

      expect(calcState.evaluateFormula('=IF(10 > 5, "Pass", "Fail")')).toBe("Pass");
      expect(calcState.evaluateFormula("=AND(10 > 5, 20 > 10)")).toBe(true);
      expect(calcState.evaluateFormula("=OR(10 < 5, 20 > 10)")).toBe(true);
      expect(calcState.evaluateFormula("=NOT(10 < 5)")).toBe(true);
      expect(calcState.evaluateFormula("=IFERROR(#VALUE!, 99)")).toBe(99);
      expect(calcState.evaluateFormula('=IFS(1 > 2, "No", 2 > 1, "Yes")')).toBe("Yes");
    });

    it("wires Text category button and evaluates CONCAT, LEFT, RIGHT, MID, LEN, UPPER, LOWER, TRIM, SUBSTITUTE", () => {
      const textBtn = document.querySelector('[data-testid="calc-btn-fn-text"]') as HTMLButtonElement;
      expect(textBtn).not.toBeNull();
      textBtn.click();
      expect(app.getFunctionWizardDialog().isOpen()).toBe(true);
      app.getFunctionWizardDialog().hide();

      calcState.setCellValue("A1", "Open");
      calcState.setCellValue("B1", "Office");
      expect(calcState.evaluateFormula('=CONCAT(A1, " ", B1)')).toBe("Open Office");
      expect(calcState.evaluateFormula('=LEFT("Antigravity", 4)')).toBe("Anti");
      expect(calcState.evaluateFormula('=RIGHT("Antigravity", 7)')).toBe("gravity");
      expect(calcState.evaluateFormula('=MID("Antigravity", 5, 4)')).toBe("grav");
      expect(calcState.evaluateFormula('=LEN("AetherOffice")')).toBe(12);
      expect(calcState.evaluateFormula('=UPPER("office")')).toBe("OFFICE");
      expect(calcState.evaluateFormula('=LOWER("OFFICE")')).toBe("office");
      expect(calcState.evaluateFormula('=TRIM("  padded  ")')).toBe("padded");
      expect(calcState.evaluateFormula('=SUBSTITUTE("banana", "a", "o")')).toBe("bonono");
    });

    it("wires Date category button and evaluates TODAY, NOW, YEAR, MONTH, DAY, DATEDIF", () => {
      const dateBtn = document.querySelector('[data-testid="calc-btn-fn-date"]') as HTMLButtonElement;
      expect(dateBtn).not.toBeNull();
      dateBtn.click();
      expect(app.getFunctionWizardDialog().isOpen()).toBe(true);
      app.getFunctionWizardDialog().hide();

      expect(String(calcState.evaluateFormula("=TODAY()"))).toMatch(/^\d{4}-\d{2}-\d{2}$/);
      expect(String(calcState.evaluateFormula("=NOW()"))).toMatch(/^\d{4}-\d{2}-\d{2}/);
      expect(calcState.evaluateFormula('=YEAR("2026-10-30")')).toBe(2026);
      expect(calcState.evaluateFormula('=MONTH("2026-10-30")')).toBe(10);
      expect(calcState.evaluateFormula('=DAY("2026-10-30")')).toBe(30);
      expect(calcState.evaluateFormula('=DATEDIF("2026-01-01", "2026-01-15", "D")')).toBe(14);
    });

    it("wires Lookup & Ref button and evaluates VLOOKUP, INDEX, MATCH, XLOOKUP", () => {
      const lkBtn = document.querySelector('[data-testid="calc-btn-fn-lookup"]') as HTMLButtonElement;
      expect(lkBtn).not.toBeNull();
      lkBtn.click();
      expect(app.getFunctionWizardDialog().isOpen()).toBe(true);
      app.getFunctionWizardDialog().hide();

      calcState.setCellValue("A1", "Alpha");
      calcState.setCellValue("B1", "100");
      calcState.setCellValue("A2", "Beta");
      calcState.setCellValue("B2", "200");
      calcState.setCellValue("A3", "Gamma");
      calcState.setCellValue("B3", "300");

      expect(calcState.evaluateFormula('=VLOOKUP("Beta", A1:B3, 2)')).toBe(200);
      expect(calcState.evaluateFormula("=INDEX(A1:B3, 3, 1)")).toBe("Gamma");
      expect(calcState.evaluateFormula('=MATCH("Beta", A1:A3)')).toBe(2);
      expect(calcState.evaluateFormula('=XLOOKUP("Gamma", A1:A3, B1:B3)')).toBe(300);
    });

    it("wires Math & Trig button and evaluates ROUND, RAND, RANDBETWEEN, MEDIAN, STDEV, SUMIF, SUMIFS, COUNTIF, AVERAGEIF", () => {
      const mathBtn = document.querySelector('[data-testid="calc-btn-fn-math"]') as HTMLButtonElement;
      expect(mathBtn).not.toBeNull();
      mathBtn.click();
      expect(app.getFunctionWizardDialog().isOpen()).toBe(true);
      app.getFunctionWizardDialog().hide();

      expect(calcState.evaluateFormula("=ROUND(12.3456, 2)")).toBe(12.35);
      const randVal = calcState.evaluateFormula("=RAND()");
      expect(typeof randVal).toBe("number");
      expect(Number(randVal)).toBeGreaterThanOrEqual(0);
      expect(Number(randVal)).toBeLessThanOrEqual(1);

      const rbVal = calcState.evaluateFormula("=RANDBETWEEN(10, 20)");
      expect(Number(rbVal)).toBeGreaterThanOrEqual(10);
      expect(Number(rbVal)).toBeLessThanOrEqual(20);

      calcState.setCellValue("A1", "10");
      calcState.setCellValue("A2", "20");
      calcState.setCellValue("A3", "30");
      calcState.setCellValue("A4", "40");
      calcState.setCellValue("A5", "50");

      expect(calcState.evaluateFormula("=MEDIAN(A1:A5)")).toBe(30);
      expect(Number(calcState.evaluateFormula("=STDEV(A1:A5)"))).toBeGreaterThan(14);
      expect(calcState.evaluateFormula('=COUNTIF(A1:A5, ">25")')).toBe(3);

      calcState.setCellValue("B1", "Apples");
      calcState.setCellValue("B2", "Oranges");
      calcState.setCellValue("B3", "Apples");
      calcState.setCellValue("B4", "Bananas");
      calcState.setCellValue("B5", "Apples");

      expect(calcState.evaluateFormula('=SUMIF(B1:B5, "Apples", A1:A5)')).toBe(90);
      expect(calcState.evaluateFormula('=AVERAGEIF(B1:B5, "Apples", A1:A5)')).toBe(30);
      expect(calcState.evaluateFormula('=SUMIFS(A1:A5, B1:B5, "Apples", A1:A5, ">15")')).toBe(80);
    });

    it("wires Name Manager modal and defined names usable in formulas", () => {
      const nmBtn = document.querySelector('[data-testid="calc-btn-name-manager"]') as HTMLButtonElement;
      expect(nmBtn).not.toBeNull();
      nmBtn.click();

      const nmModal = app.getNameManagerDialog();
      expect(nmModal.isOpen()).toBe(true);

      // Define named variable
      calcState.defineName("TaxRate", 0.18);
      expect(calcState.getName("TaxRate")).toBe(0.18);

      calcState.setCellValue("A1", "100");
      expect(calcState.evaluateFormula("=A1 * TaxRate")).toBe(18);

      nmModal.hide();
      expect(nmModal.isOpen()).toBe(false);

      calcState.deleteName("TaxRate");
      expect(calcState.getName("TaxRate")).toBeUndefined();
    });

    it("wires Define Name button", () => {
      const defBtn = document.querySelector('[data-testid="calc-btn-define-name"]') as HTMLButtonElement;
      expect(defBtn).not.toBeNull();
      defBtn.click();
      expect(app.getNameManagerDialog().isOpen()).toBe(true);
      app.getNameManagerDialog().hide();
    });

    it("wires Formula Auditing (Trace Precedents, Dependents, Remove Arrows, Show Formulas, Evaluate)", () => {
      calcState.setCellValue("A1", "10");
      calcState.setCellValue("A2", "20");
      calcState.setCellValue("A3", "=A1 + A2");
      calcState.selectedCell = "A3";

      const btnPrec = document.querySelector('[data-testid="calc-btn-trace-precedents"]') as HTMLButtonElement;
      btnPrec.click();
      expect(calcState.formulaReferences.length).toBeGreaterThan(0);

      const btnRem = document.querySelector('[data-testid="calc-btn-remove-arrows"]') as HTMLButtonElement;
      btnRem.click();
      expect(calcState.formulaReferences.length).toBe(0);

      calcState.selectedCell = "A1";
      const btnDep = document.querySelector('[data-testid="calc-btn-trace-dependents"]') as HTMLButtonElement;
      btnDep.click();
      expect(calcState.formulaReferences.length).toBeGreaterThan(0);

      btnRem.click();
      expect(calcState.formulaReferences.length).toBe(0);

      const btnShow = document.querySelector('[data-testid="calc-btn-show-formulas"]') as HTMLButtonElement;
      btnShow.click();
      expect(calcState.showFormulas).toBe(true);
      btnShow.click();
      expect(calcState.showFormulas).toBe(false);

      const btnEval = document.querySelector('[data-testid="calc-btn-evaluate"]') as HTMLButtonElement;
      btnEval.click();
      expect(calcState.getCellValue("A3")).toBe(30);
    });

    it("wires Calculation buttons (Calculate Now, Calc Sheet)", () => {
      calcState.setCellValue("A1", "50");
      calcState.setCellValue("A2", "=A1 * 2");
      const btnCalcNow = document.querySelector('[data-testid="calc-btn-calculate-now"]') as HTMLButtonElement;
      btnCalcNow.click();
      expect(calcState.getCellValue("A2")).toBe(100);

      const btnCalcSheet = document.querySelector('[data-testid="calc-btn-calc-sheet"]') as HTMLButtonElement;
      btnCalcSheet.click();
      expect(calcState.getCellValue("A2")).toBe(100);
    });
  });

  describe("Data Tab", () => {
    beforeEach(() => {
      app.getRibbon().setActiveTab("Data");
    });

    it("wires From CSV and From Web import", () => {
      const csvBtn = document.querySelector('[data-testid="calc-btn-from-csv"]') as HTMLButtonElement;
      expect(csvBtn).not.toBeNull();
      csvBtn.click();
      expect(calcState.getCellValue("A1")).toBe("Item");
      expect(calcState.getCellValue("B1")).toBe("Qty");

      const webBtn = document.querySelector('[data-testid="calc-btn-from-web"]') as HTMLButtonElement;
      expect(webBtn).not.toBeNull();
      webBtn.click();
      expect(calcState.getCellValue("A1")).toBe("Region");
      expect(calcState.getCellValue("B1")).toBe("Sales");
    });

    it("wires Refresh All button", () => {
      const refBtn = document.querySelector('[data-testid="calc-btn-refresh-all"]') as HTMLButtonElement;
      expect(refBtn).not.toBeNull();
      refBtn.click();
    });

    it("wires Sort A-Z, Sort Z-A, and Custom Sort", () => {
      calcState.setCellValue("A1", "Zebra");
      calcState.setCellValue("A2", "Apple");
      calcState.setCellValue("A3", "Mango");
      calcState.selectedRanges = [{ minRow: 0, maxRow: 2, minCol: 0, maxCol: 0 }];

      const ascBtn = document.querySelector('[data-testid="calc-btn-sort-asc"]') as HTMLButtonElement;
      ascBtn.click();
      expect(calcState.getCellValue("A1")).toBe("Apple");
      expect(calcState.getCellValue("A3")).toBe("Zebra");

      const descBtn = document.querySelector('[data-testid="calc-btn-sort-desc"]') as HTMLButtonElement;
      descBtn.click();
      expect(calcState.getCellValue("A1")).toBe("Zebra");
      expect(calcState.getCellValue("A3")).toBe("Apple");

      const customSortBtn = document.querySelector('[data-testid="calc-btn-custom-sort"]') as HTMLButtonElement;
      customSortBtn.click();
      expect(calcState.getCellValue("A1")).toBe("Apple");
    });

    it("wires Filter and Clear Filter buttons", () => {
      calcState.setCellValue("A1", "City");
      calcState.setCellValue("A2", "Tokyo");
      calcState.setCellValue("A3", "Paris");
      calcState.selectedRanges = [{ minRow: 0, maxRow: 2, minCol: 0, maxCol: 0 }];

      const filterBtn = document.querySelector('[data-testid="calc-btn-toggle-filter"]') as HTMLButtonElement;
      filterBtn.click();
      expect(calcState.getActiveSheet().filterActive).toBe(true);

      const clearBtn = document.querySelector('[data-testid="calc-btn-clear-filter"]') as HTMLButtonElement;
      clearBtn.click();
      expect(calcState.getActiveSheet().hiddenRows?.size).toBe(0);

      filterBtn.click();
      expect(calcState.getActiveSheet().filterActive).toBe(false);
    });

    it("wires Text to Columns, Flash Fill, and Remove Duplicates", () => {
      calcState.setCellValue("A1", "First,Last");
      calcState.setCellValue("A2", "John,Doe");
      calcState.selectedRanges = [{ minRow: 0, maxRow: 1, minCol: 0, maxCol: 0 }];

      const t2cBtn = document.querySelector('[data-testid="calc-btn-text-to-columns"]') as HTMLButtonElement;
      t2cBtn.click();
      expect(calcState.getCellValue("A1")).toBe("First");
      expect(calcState.getCellValue("B1")).toBe("Last");
      expect(calcState.getCellValue("A2")).toBe("John");
      expect(calcState.getCellValue("B2")).toBe("Doe");

      // Flash Fill
      calcState.setCellValue("A1", "john.doe@example.com");
      calcState.setCellValue("B1", "john.doe");
      calcState.setCellValue("A2", "jane.smith@example.com");
      calcState.selectedRanges = [{ minRow: 0, maxRow: 1, minCol: 1, maxCol: 1 }];
      const ffBtn = document.querySelector('[data-testid="calc-btn-flash-fill"]') as HTMLButtonElement;
      ffBtn.click();
      expect(calcState.getCellValue("B2")).toBe("jane.smith");

      // Remove Duplicates
      calcState.setCellValue("A1", "Dup");
      calcState.setCellValue("A2", "Unique");
      calcState.setCellValue("A3", "Dup");
      calcState.selectedRanges = [{ minRow: 0, maxRow: 2, minCol: 0, maxCol: 0 }];
      const rdBtn = document.querySelector('[data-testid="calc-btn-remove-duplicates"]') as HTMLButtonElement;
      rdBtn.click();
      expect(calcState.getCellValue("A1")).toBe("Dup");
      expect(calcState.getCellValue("A2")).toBe("Unique");
      expect(calcState.getCellValue("A3")).toBeUndefined();
    });

    it("wires Data Validation", () => {
      const valBtn = document.querySelector('[data-testid="calc-btn-data-validation"]') as HTMLButtonElement;
      expect(valBtn).not.toBeNull();
      valBtn.click();

      const validationCheck = calcState.validateCellValue("A1", 50);
      expect(validationCheck.valid).toBe(true);

      const invalidCheck = calcState.validateCellValue("A1", 999);
      expect(invalidCheck.valid).toBe(false);
    });

    it("wires Outline & Solver (Group, Ungroup, Subtotal, Goal Seek)", () => {
      const grpBtn = document.querySelector('[data-testid="calc-btn-group"]') as HTMLButtonElement;
      grpBtn.click();
      expect(calcState.getActiveSheet().rowGroups?.length).toBe(1);

      const ungrpBtn = document.querySelector('[data-testid="calc-btn-ungroup"]') as HTMLButtonElement;
      ungrpBtn.click();
      expect(calcState.getActiveSheet().rowGroups?.length).toBe(0);

      // Subtotals
      calcState.setCellValue("A1", "North");
      calcState.setCellValue("B1", "100");
      calcState.setCellValue("A2", "North");
      calcState.setCellValue("B2", "200");
      calcState.setCellValue("A3", "South");
      calcState.setCellValue("B3", "300");
      calcState.selectedRanges = [{ minRow: 0, maxRow: 2, minCol: 0, maxCol: 1 }];

      const subBtn = document.querySelector('[data-testid="calc-btn-subtotal"]') as HTMLButtonElement;
      subBtn.click();
      expect(calcState.getCellValue("A3")).toBe("North Total");
      expect(calcState.getCellValue("B3")).toBe(300);

      // Goal Seek
      const gsBtn = document.querySelector('[data-testid="calc-btn-goal-seek"]') as HTMLButtonElement;
      gsBtn.click();
      expect(app.getGoalSeekDialog().isOpen()).toBe(true);
      app.getGoalSeekDialog().hide();
    });
  });

  describe("Review & View Tabs", () => {
    it("wires Review Tab: Spelling, Thesaurus, Protect Sheet, Protect Workbook, Allow Edit Ranges, Notes, Stats, Copilot", () => {
      app.getRibbon().setActiveTab("Review");

      const spellBtn = document.querySelector('[data-testid="calc-btn-spelling"]') as HTMLButtonElement;
      spellBtn.click();
      expect(document.getElementById("status-doc-info")?.textContent).toContain("Spelling");

      const theBtn = document.querySelector('[data-testid="calc-btn-thesaurus"]') as HTMLButtonElement;
      theBtn.click();
      expect(document.getElementById("status-doc-info")?.textContent).toContain("Thesaurus");

      // Protect Sheet
      const psBtn = document.querySelector('[data-testid="calc-btn-protect-sheet"]') as HTMLButtonElement;
      psBtn.click();
      expect(calcState.isSheetProtected()).toBe(true);
      // Protected sheet should disallow cell editing
      const editResult = calcState.setCellValue("A1", "Hacked");
      expect(editResult).toBe(false);
      psBtn.click();
      expect(calcState.isSheetProtected()).toBe(false);

      // Protect Workbook
      const pwBtn = document.querySelector('[data-testid="calc-btn-protect-workbook"]') as HTMLButtonElement;
      pwBtn.click();
      expect(calcState.isWorkbookProtected()).toBe(true);
      pwBtn.click();
      expect(calcState.isWorkbookProtected()).toBe(false);

      const allowBtn = document.querySelector('[data-testid="calc-btn-allow-edit-ranges"]') as HTMLButtonElement;
      allowBtn.click();
      expect(document.getElementById("status-doc-info")?.textContent).toContain("Allow Edit Ranges");

      // Note
      const noteBtn = document.querySelector('[data-testid="calc-btn-new-note"]') as HTMLButtonElement;
      noteBtn.click();
      expect(calcState.getCell(calcState.selectedCell).comment).toBe("Comment");

      // Show All Notes
      const showNotesBtn = document.querySelector('[data-testid="calc-btn-show-all-notes"]') as HTMLButtonElement;
      showNotesBtn.click();
      expect(calcState.showAllNotes).toBe(true);
      showNotesBtn.click();
      expect(calcState.showAllNotes).toBe(false);

      // Workbook Statistics
      const statsBtn = document.querySelector('[data-testid="calc-btn-workbook-stats"]') as HTMLButtonElement;
      statsBtn.click();
      expect(app.getWorkbookStatisticsDialog().isOpen()).toBe(true);
      app.getWorkbookStatisticsDialog().hide();

      // Copilot
      const copilotBtn = document.querySelector('[data-testid="calc-btn-copilot-review"]') as HTMLButtonElement;
      copilotBtn.click();
    });

    it("wires View Tab: Workbook Views, Show toggles, Zoom, Freeze Panes, Split", () => {
      app.getRibbon().setActiveTab("View");

      // Workbook Views
      const normalBtn = document.querySelector('[data-testid="calc-btn-view-normal"]') as HTMLButtonElement;
      const pbBtn = document.querySelector('[data-testid="calc-btn-view-pagebreak"]') as HTMLButtonElement;
      const plBtn = document.querySelector('[data-testid="calc-btn-view-pagelayout"]') as HTMLButtonElement;

      pbBtn.click();
      expect(calcState.viewMode).toBe("pagebreak");
      plBtn.click();
      expect(calcState.viewMode).toBe("pagelayout");
      normalBtn.click();
      expect(calcState.viewMode).toBe("normal");

      // Show Group
      const fbBtn = document.querySelector('[data-testid="calc-btn-toggle-formula-bar"]') as HTMLButtonElement;
      fbBtn.click();
      expect(calcState.getActiveSheet().showFormulaBar).toBe(false);
      fbBtn.click();
      expect(calcState.getActiveSheet().showFormulaBar).toBe(true);

      const gridBtn = document.querySelector('[data-testid="calc-btn-toggle-gridlines"]') as HTMLButtonElement;
      gridBtn.click();
      expect(calcState.getActiveSheet().showGridlines).toBe(false);
      gridBtn.click();
      expect(calcState.getActiveSheet().showGridlines).toBe(true);

      const headBtn = document.querySelector('[data-testid="calc-btn-toggle-headings"]') as HTMLButtonElement;
      headBtn.click();
      expect(calcState.getActiveSheet().showHeadings).toBe(false);
      headBtn.click();
      expect(calcState.getActiveSheet().showHeadings).toBe(true);

      // Zoom Group
      const zoom100Btn = document.querySelector('[data-testid="calc-btn-zoom-100"]') as HTMLButtonElement;
      const zoomInBtn = document.querySelector('[data-testid="calc-btn-zoom-in"]') as HTMLButtonElement;
      const zoomOutBtn = document.querySelector('[data-testid="calc-btn-zoom-out"]') as HTMLButtonElement;
      const zoomSelBtn = document.querySelector('[data-testid="calc-btn-zoom-selection"]') as HTMLButtonElement;

      zoomInBtn.click();
      expect(calcState.zoom).toBeCloseTo(1.1);
      zoomOutBtn.click();
      expect(calcState.zoom).toBeCloseTo(1.0);
      zoomSelBtn.click();
      expect(calcState.zoom).toBeGreaterThan(0.5);
      zoom100Btn.click();
      expect(calcState.zoom).toBe(1.0);

      // Window Group (Freeze Panes & Split)
      const fpBtn = document.querySelector('[data-testid="calc-btn-freeze-panes"]') as HTMLButtonElement;
      const topBtn = document.querySelector('[data-testid="calc-btn-freeze-top-row"]') as HTMLButtonElement;
      const colBtn = document.querySelector('[data-testid="calc-btn-freeze-first-col"]') as HTMLButtonElement;
      const ufpBtn = document.querySelector('[data-testid="calc-btn-unfreeze-panes"]') as HTMLButtonElement;

      topBtn.click();
      expect(calcState.getActiveSheet().freezePanes).toEqual({ row: 1, col: 0 });

      colBtn.click();
      expect(calcState.getActiveSheet().freezePanes).toEqual({ row: 0, col: 1 });

      calcState.selectedRanges = [{ minRow: 3, maxRow: 3, minCol: 2, maxCol: 2 }];
      fpBtn.click();
      expect(calcState.getActiveSheet().freezePanes).toEqual({ row: 3, col: 2 });

      ufpBtn.click();
      expect(calcState.getActiveSheet().freezePanes).toEqual({ row: 0, col: 0 });

      const splitBtn = document.querySelector('[data-testid="calc-btn-split-window"]') as HTMLButtonElement;
      splitBtn.click();
      expect(calcState.isSplitWindow).toBe(true);
      splitBtn.click();
      expect(calcState.isSplitWindow).toBe(false);
    });
  });
});
