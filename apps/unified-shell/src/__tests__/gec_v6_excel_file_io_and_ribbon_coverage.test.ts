import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { binaryPackageService } from "../services/binaryPackageService";
import { fileCodec } from "../services/fileCodec";
import { calcState } from "../state/calcState";
import { CalcRibbon } from "../components/ribbon/CalcRibbon";
import { UnifiedShellApp } from "../main";
import { IPC } from "../ipc";
import JSZip from "jszip";

describe("Subphase 37.1.1: Excel Save/Open (.xlsx & .csv) Fix & Complete 7-Tab Ribbon Functionality", () => {
  beforeEach(() => {
    (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ = {
      invoke: vi.fn(),
    };

    const mockCtx = {
      clearRect: vi.fn(),
      fillRect: vi.fn(),
      strokeRect: vi.fn(),
      fillText: vi.fn(),
      measureText: vi.fn().mockReturnValue({ width: 50 }),
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
      textBaseline: "alphabetic",
    };
    HTMLCanvasElement.prototype.getContext = (() => mockCtx) as unknown as typeof HTMLCanvasElement.prototype.getContext;

    document.body.innerHTML = `
      <div id="app">
        <header id="title-bar"><div class="title-actions"></div></header>
        <div id="app-rail-container"></div>
        <div id="tab-bar-container"></div>
        <div id="ribbon-container"></div>
        <div id="sub-header-container"></div>
        <div id="main-content-layout">
          <div id="sidebar-container"></div>
          <div id="viewport-container"></div>
        </div>
        <footer id="status-bar">
          <span id="status-doc-info">Ready</span>
          <span id="status-zoom-level">100%</span>
        </footer>
      </div>
    `;
    calcState.resetToBlank("Sheet1");
  });

  afterEach(() => {
    delete (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    vi.restoreAllMocks();
  });

  it("1. Real multi-sheet .xlsx file save and open round-trip with numbers, text, formulas & booleans", async () => {
    // Populate Sheet1
    calcState.setCellValue("A1", "Department");
    calcState.setCellValue("B1", "Revenue");
    calcState.setCellValue("C1", "Target");
    calcState.setCellValue("D1", "Status");
    calcState.setCellValue("A2", "Engineering");
    calcState.setCellValue("B2", "150000");
    calcState.setCellValue("C2", "=B2*1.1");
    calcState.setCellValue("D2", "TRUE");

    // Add Sheet2 and populate
    calcState.addSheet("Expenses");
    calcState.setCellValue("A1", "Expense Item");
    calcState.setCellValue("B1", "Cost");
    calcState.setCellValue("A2", "Infrastructure");
    calcState.setCellValue("B2", "42000");

    // Switch back to Sheet1
    calcState.switchSheet(calcState.sheets[0].id);

    // Save as .xlsx
    let savedBytes: Uint8Array | number[] | undefined;
    vi.spyOn(IPC, "saveFileBytes").mockImplementation(async (_filename, bytes) => {
      savedBytes = bytes;
      return "C:\\Users\\Test\\Documents\\Financials.xlsx";
    });

    const savedPath = await fileCodec.saveFile("Excel", { filename: "Financials.xlsx" });
    expect(savedPath).toContain("Financials.xlsx");
    expect(savedBytes).toBeDefined();

    const u8 = savedBytes instanceof Uint8Array ? savedBytes : new Uint8Array(savedBytes!);

    // Verify ZIP contains valid OOXML structure
    const zip = await JSZip.loadAsync(u8);
    expect(zip.file("[Content_Types].xml")).not.toBeNull();
    expect(zip.file("_rels/.rels")).not.toBeNull();
    expect(zip.file("xl/workbook.xml")).not.toBeNull();
    expect(zip.file("xl/_rels/workbook.xml.rels")).not.toBeNull();
    expect(zip.file("xl/styles.xml")).not.toBeNull();
    expect(zip.file("xl/sharedStrings.xml")).not.toBeNull();
    expect(zip.file("xl/worksheets/sheet1.xml")).not.toBeNull();
    expect(zip.file("xl/worksheets/sheet2.xml")).not.toBeNull();

    // Reset state to empty blank
    calcState.resetToBlank("EmptyBlank");
    expect(calcState.sheets.length).toBe(1);
    expect(calcState.getCellValue("A1")).toBeUndefined();

    // Open saved .xlsx file
    const opened = await fileCodec.openFile(u8, "Financials.xlsx", "Excel");
    expect(opened).toBe(true);

    // Verify both sheets were restored
    expect(calcState.sheets.length).toBe(2);
    expect(calcState.sheets[0].name).toBe("Sheet1");
    expect(calcState.sheets[1].name).toBe("Expenses");

    // Verify Sheet1 content and formulas
    calcState.switchSheet(calcState.sheets[0].id);
    expect(calcState.getCellValue("A1")).toBe("Department");
    expect(calcState.getCellValue("B1")).toBe("Revenue");
    expect(calcState.getCellValue("A2")).toBe("Engineering");
    expect(Number(calcState.getCellValue("B2"))).toBe(150000);
    expect(Number(calcState.getCellValue("C2"))).toBe(165000);
    expect(calcState.getCellValue("D2")).toBe("TRUE");

    // Verify Sheet2 content
    calcState.switchSheet(calcState.sheets[1].id);
    expect(calcState.getCellValue("A1")).toBe("Expense Item");
    expect(Number(calcState.getCellValue("B2"))).toBe(42000);
  });

  it("2. Real .csv file save and open round-trip", async () => {
    calcState.setCellValue("A1", "Product");
    calcState.setCellValue("B1", "Qty");
    calcState.setCellValue("C1", "Price");
    calcState.setCellValue("A2", "Mechanical Keyboard");
    calcState.setCellValue("B2", "25");
    calcState.setCellValue("C2", "149.99");

    let savedCsvBytes: Uint8Array | number[] | undefined;
    vi.spyOn(IPC, "saveFileBytes").mockImplementation(async (_filename, bytes) => {
      savedCsvBytes = bytes;
      return "C:\\Users\\Test\\Documents\\Inventory.csv";
    });

    const savedPath = await fileCodec.saveFile("Excel", { filename: "Inventory.csv" });
    expect(savedPath).toContain("Inventory.csv");
    expect(savedCsvBytes).toBeDefined();

    const csvU8 = savedCsvBytes instanceof Uint8Array ? savedCsvBytes : new Uint8Array(savedCsvBytes!);
    const csvText = new TextDecoder().decode(csvU8);
    expect(csvText).toContain("Mechanical Keyboard");
    expect(csvText).toContain("149.99");

    // Reset and open CSV
    calcState.resetToBlank("BlankSheet");
    const opened = await fileCodec.openFile(csvU8, "Inventory.csv", "Excel");
    expect(opened).toBe(true);

    expect(calcState.getCellValue("A1")).toBe("Product");
    expect(calcState.getCellValue("A2")).toBe("Mechanical Keyboard");
    expect(Number(calcState.getCellValue("B2"))).toBe(25);
  });

  it("3. Opening existing .xlsx via UnifiedShellApp renders cells immediately without empty sheets", async () => {
    vi.spyOn(IPC, "getBootConfig").mockResolvedValue({
      app_mode: "calc",
    });

    const app = new UnifiedShellApp(false);
    app.applyBootConfig({ app_mode: "calc" });
    await new Promise((r) => setTimeout(r, 60));

    // Create a real .xlsx payload
    const testCells = {
      A1: "Quarter",
      B1: "Profit",
      A2: "Q1",
      B2: "75000",
      A3: "Q2",
      B3: "82000",
    };
    const xlsxBytes = await binaryPackageService.buildXlsx(testCells);

    // Open the file through UnifiedShellApp
    await app.handleOpenFile("Quarterly_Report.xlsx", xlsxBytes);

    expect(calcState.getCellValue("A1")).toBe("Quarter");
    expect(calcState.getCellValue("B1")).toBe("Profit");
    expect(calcState.getCellValue("A2")).toBe("Q1");
    expect(Number(calcState.getCellValue("B2"))).toBe(75000);

    const canvas = app.getCalcGridCanvas();
    expect(canvas).toBeDefined();

    // Verify status doc info shows Opened: Quarterly_Report.xlsx
    const docInfo = document.getElementById("status-doc-info");
    expect(docInfo?.textContent).toContain("Quarterly_Report.xlsx");
  });

  it("4. All 7 Excel Ribbon Tabs: Home, Insert, Page Layout, Formulas, Data, Review, View emit valid actions", () => {
    const ribbonContainer = document.createElement("div");
    document.body.appendChild(ribbonContainer);
    const emittedActions: Array<{ action: string; payload?: unknown }> = [];
    const ribbon = new CalcRibbon();

    const render = (tab: string) => {
      ribbon.renderTab(tab, ribbonContainer, (action, payload) => {
        emittedActions.push({ action, payload });
      });
    };

    // 1. Home Tab
    render("Home");
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-paste"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-copy"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-bold"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-italic"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-align-center"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-merge-center"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-num-currency"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-autosum"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-copilot"]')?.click();

    expect(emittedActions.some((a) => a.action === "Paste")).toBe(true);
    expect(emittedActions.some((a) => a.action === "Copy")).toBe(true);
    expect(emittedActions.some((a) => a.action === "ToggleBold")).toBe(true);
    expect(emittedActions.some((a) => a.action === "ToggleItalic")).toBe(true);
    expect(emittedActions.some((a) => a.action === "AlignCenter")).toBe(true);
    expect(emittedActions.some((a) => a.action === "MergeAndCenter")).toBe(true);
    expect(emittedActions.some((a) => a.action === "SetNumberFormat" && a.payload === "Currency")).toBe(true);
    expect(emittedActions.some((a) => a.action === "AutoSum")).toBe(true);
    expect(emittedActions.some((a) => a.action === "ToggleCopilot")).toBe(true);

    // 2. Insert Tab
    render("Insert");
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-pivottable"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-pivot-summary"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-insert-table"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-chart-column"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-chart-line"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-chart-pie"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-sparkline-line"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-insert-link"]')?.click();

    expect(emittedActions.some((a) => a.action === "InsertPivotTable")).toBe(true);
    expect(emittedActions.some((a) => a.action === "PivotSummary")).toBe(true);
    expect(emittedActions.some((a) => a.action === "InsertTable")).toBe(true);
    expect(emittedActions.some((a) => a.action === "InsertChart" && a.payload === "column")).toBe(true);
    expect(emittedActions.some((a) => a.action === "InsertChart" && a.payload === "line")).toBe(true);
    expect(emittedActions.some((a) => a.action === "InsertChart" && a.payload === "pie")).toBe(true);
    expect(emittedActions.some((a) => a.action === "InsertSparkline" && a.payload === "line")).toBe(true);
    expect(emittedActions.some((a) => a.action === "InsertLink")).toBe(true);

    // 3. Page Layout Tab
    render("Page Layout");
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-themes"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-margins-normal"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-orient-landscape"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-set-print-area"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-toggle-grid"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-toggle-headings"]')?.click();

    expect(emittedActions.some((a) => a.action === "SetTheme")).toBe(true);
    expect(emittedActions.some((a) => a.action === "SetMargins")).toBe(true);
    expect(emittedActions.some((a) => a.action === "SetOrientation")).toBe(true);
    expect(emittedActions.some((a) => a.action === "SetPrintArea")).toBe(true);
    expect(emittedActions.some((a) => a.action === "ToggleGridlines")).toBe(true);
    expect(emittedActions.some((a) => a.action === "ToggleHeadings")).toBe(true);

    // 4. Formulas Tab
    render("Formulas");
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-insert-fn"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-name-manager"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-define-name"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-show-formulas"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-calculate-now"]')?.click();

    expect(emittedActions.some((a) => a.action === "InsertFunction")).toBe(true);
    expect(emittedActions.some((a) => a.action === "NameManager")).toBe(true);
    expect(emittedActions.some((a) => a.action === "DefineName")).toBe(true);
    expect(emittedActions.some((a) => a.action === "ShowFormulas")).toBe(true);
    expect(emittedActions.some((a) => a.action === "Recalculate")).toBe(true);

    // 5. Data Tab
    render("Data");
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-from-csv"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-sort-asc"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-sort-desc"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-toggle-filter"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-text-to-columns"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-flash-fill"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-remove-duplicates"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-data-validation"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-subtotal"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-goal-seek"]')?.click();

    expect(emittedActions.some((a) => a.action === "ImportCsv")).toBe(true);
    expect(emittedActions.some((a) => a.action === "SortAsc")).toBe(true);
    expect(emittedActions.some((a) => a.action === "SortDesc")).toBe(true);
    expect(emittedActions.some((a) => a.action === "ToggleFilter")).toBe(true);
    expect(emittedActions.some((a) => a.action === "TextToColumns")).toBe(true);
    expect(emittedActions.some((a) => a.action === "FlashFill")).toBe(true);
    expect(emittedActions.some((a) => a.action === "RemoveDuplicates")).toBe(true);
    expect(emittedActions.some((a) => a.action === "DataValidation")).toBe(true);
    expect(emittedActions.some((a) => a.action === "Subtotal")).toBe(true);
    expect(emittedActions.some((a) => a.action === "GoalSeek")).toBe(true);

    // 6. Review Tab
    render("Review");
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-spelling"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-protect-sheet"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-workbook-stats"]')?.click();

    expect(emittedActions.some((a) => a.action === "CheckSpelling")).toBe(true);
    expect(emittedActions.some((a) => a.action === "ProtectSheet")).toBe(true);
    expect(emittedActions.some((a) => a.action === "WorkbookStatistics")).toBe(true);

    // 7. View Tab
    render("View");
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-view-normal"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-view-pagebreak"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-toggle-formula-bar"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-zoom-100"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-freeze-panes"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-freeze-top-row"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-freeze-first-col"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="calc-btn-unfreeze-panes"]')?.click();

    expect(emittedActions.some((a) => a.action === "ViewNormal")).toBe(true);
    expect(emittedActions.some((a) => a.action === "ViewPageBreak")).toBe(true);
    expect(emittedActions.some((a) => a.action === "ToggleFormulaBar")).toBe(true);
    expect(emittedActions.some((a) => a.action === "Zoom100")).toBe(true);
    expect(emittedActions.some((a) => a.action === "FreezePanes")).toBe(true);
    expect(emittedActions.some((a) => a.action === "FreezeTopRow")).toBe(true);
    expect(emittedActions.some((a) => a.action === "FreezeFirstColumn")).toBe(true);
    expect(emittedActions.some((a) => a.action === "UnfreezePanes")).toBe(true);
  });

  it("5. Single cells, ranges (A1:D10), whole columns (A:C) and rows (1:5) format correctly via state operations", () => {
    // 5.1 Single cell formatting
    calcState.setCellValue("A1", "TestVal");
    calcState.selectRangeByName("A1");
    calcState.toggleBold();
    expect(calcState.getCell("A1").bold).toBe(true);

    // 5.2 Cell range A1:D10 formatting
    calcState.selectRangeByName("A1:D10");
    calcState.toggleItalic();
    calcState.setFillColor("#fee2e2");
    expect(calcState.getCell("A1").italic).toBe(true);
    expect(calcState.getCell("D10").italic).toBe(true);
    expect(calcState.getCell("C5").bgColor).toBe("#fee2e2");

    // 5.3 Whole columns A:C formatting
    calcState.selectColumnRange(0, 2);
    calcState.setFontColor("#15803d");
    calcState.setNumberFormat("Currency");
    expect(calcState.getCell("B2").color).toBe("#15803d");
    expect(calcState.getCell("B2").format).toBe("Currency");

    // 5.4 Whole rows 1:5 formatting
    calcState.selectRowRange(0, 4);
    calcState.toggleUnderline();
    expect(calcState.getCell("A1").underline).toBe(true);
    expect(calcState.getCell("A5").underline).toBe(true);
  });
});
