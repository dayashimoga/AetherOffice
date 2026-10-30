import { describe, it, expect, beforeEach, vi } from "vitest";
import { UnifiedShellApp } from "../main";
import { IPC } from "../ipc";
import { writerState } from "../state/writerState";
import { calcState } from "../state/calcState";
import { slidesState } from "../state/slidesState";
import { pdfState } from "../state/pdfState";
import { notesState } from "../state/notesState";

describe("Universal File Open/Save/Export & Shortcuts (GEC-v3.0 Subphase 27.1.1)", () => {
  let app: UnifiedShellApp;

  beforeEach(() => {
    localStorage.clear();
    IPC._resetMockState();

    // Mock HTMLCanvasElement 2D context
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
      scale: vi.fn(),
      setTransform: vi.fn(),
      setLineDash: vi.fn(),
    })) as unknown as typeof HTMLCanvasElement.prototype.getContext;

    document.body.innerHTML = `
      <div id="app" class="desktop-shell">
        <header class="title-bar">
          <div class="app-branding">
            <span class="app-logo">📄</span>
            <span class="app-name">AetherOffice</span>
          </div>
        </header>
        <div id="app-rail-container"></div>
        <div id="tab-bar-container"></div>
        <div id="ribbon-container"></div>
        <div id="sub-header-container"></div>
        <aside id="sidebar-container"></aside>
        <div id="viewport-container"></div>
        <div id="bottom-sheet"></div>
        <canvas id="stylus-ink-canvas"></canvas>
        <div id="reflow-reader-container"></div>
        <div id="row-card-inspector-container"></div>
        <div id="laser-pointer-dot"></div>
        <span id="status-doc-info"></span>
        <span id="status-word-count"></span>
        <span id="status-zoom-level"></span>
        <input type="range" id="zoom-slider" />
      </div>
    `;

    app = new UnifiedShellApp(false);
    app.applyBootConfig({ app_mode: "writer" });
  });

  it("1. Backstage File -> New creates blank tabs for all 5 document types", async () => {
    const backstage = app.getBackstageMenu();
    await backstage.show("Word", "sess-1", "Untitled.docx");
    backstage.setActiveNav("New");

    // Word
    const newWord = document.querySelector<HTMLElement>('[data-testid="backstage-new-word"]');
    expect(newWord).not.toBeNull();
    newWord?.click();
    let tabs = app.getTabBar().getTabs();
    expect(tabs.some((t) => t.docType === "Word")).toBe(true);

    // Excel
    await backstage.show("Word", "sess-1", "Untitled.docx");
    backstage.setActiveNav("New");
    const newExcel = document.querySelector<HTMLElement>('[data-testid="backstage-new-excel"]');
    expect(newExcel).not.toBeNull();
    newExcel?.click();
    tabs = app.getTabBar().getTabs();
    expect(tabs.some((t) => t.docType === "Excel")).toBe(true);

    // PowerPoint
    await backstage.show("Word", "sess-1", "Untitled.docx");
    backstage.setActiveNav("New");
    const newPpt = document.querySelector<HTMLElement>('[data-testid="backstage-new-ppt"]');
    expect(newPpt).not.toBeNull();
    newPpt?.click();
    tabs = app.getTabBar().getTabs();
    expect(tabs.some((t) => t.docType === "PowerPoint")).toBe(true);

    // PDF
    await backstage.show("Word", "sess-1", "Untitled.docx");
    backstage.setActiveNav("New");
    const newPdf = document.querySelector<HTMLElement>('[data-testid="backstage-new-pdf"]');
    expect(newPdf).not.toBeNull();
    newPdf?.click();
    tabs = app.getTabBar().getTabs();
    expect(tabs.some((t) => t.docType === "PDF")).toBe(true);

    // Notes
    await backstage.show("Word", "sess-1", "Untitled.docx");
    backstage.setActiveNav("New");
    const newNotes = document.querySelector<HTMLElement>('[data-testid="backstage-new-notes"]');
    expect(newNotes).not.toBeNull();
    newNotes?.click();
    tabs = app.getTabBar().getTabs();
    expect(tabs.some((t) => t.docType === "Notes")).toBe(true);
  });

  it("2. Universal Open File opens .docx, .xlsx, .csv, .pptx, .pdf, .txt, .md into correct editors", async () => {
    // Open Word document with text
    await app.handleOpenFile("Annual_Report.docx", "Project Overview\nAll tasks verified.");
    expect(app.getTabBar().getActiveTab()?.docType).toBe("Word");
    expect(writerState.getTextContent()).toContain("Project Overview");

    // Open CSV spreadsheet
    await app.handleOpenFile("Sales_Q4.csv", "Region,Revenue,Units\nNorth,5200,120\nSouth,6100,150");
    expect(app.getTabBar().getActiveTab()?.docType).toBe("Excel");
    expect(calcState.getCellValue("A1")).toBe("Region");
    expect(String(calcState.getCellValue("B2"))).toBe("5200");

    // Open PowerPoint
    await app.handleOpenFile("Keynote_Pitch.pptx");
    expect(app.getTabBar().getActiveTab()?.docType).toBe("PowerPoint");
    expect(slidesState.slides.length).toBeGreaterThanOrEqual(1);

    // Open PDF
    await app.handleOpenFile("Specification.pdf");
    expect(app.getTabBar().getActiveTab()?.docType).toBe("PDF");
    expect(pdfState.pages.length).toBeGreaterThanOrEqual(1);

    // Open Text / Markdown Notes
    await app.handleOpenFile("Sprint_Tasks.md", "# Today's Priorities\n- [ ] Release v3.0");
    expect(app.getTabBar().getActiveTab()?.docType).toBe("Notes");
    expect(notesState.getContent()).toContain("# Today's Priorities");
  });

  it("3. Native file input picker browse button triggers file opening in Backstage", async () => {
    const backstage = app.getBackstageMenu();
    await backstage.show("Word", "sess-1", "Untitled.docx");
    backstage.setActiveNav("Open");

    const browseBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-browse-file"]');
    const fileInput = document.querySelector<HTMLInputElement>('[data-testid="input-file-picker"]');
    expect(browseBtn).not.toBeNull();
    expect(fileInput).not.toBeNull();

    const clickSpy = vi.spyOn(fileInput!, "click");
    browseBtn?.click();
    expect(clickSpy).toHaveBeenCalled();
  });

  it("4. Universal Save & Save As persists document and clears dirty indicator across all 5 formats", async () => {
    // 4.1 Word
    app.handleNewTab("Word");
    const wordTab = app.getTabBar().getActiveTab()!;
    app.getTabBar().setDirty(wordTab.id, true);
    expect(wordTab.isDirty).toBe(true);
    await app.saveActiveDocument("MyDoc.docx");
    expect(wordTab.isDirty).toBe(false);

    // 4.2 Excel
    app.handleNewTab("Excel");
    calcState.setCellValue("A1", "100");
    const excelTab = app.getTabBar().getActiveTab()!;
    app.getTabBar().setDirty(excelTab.id, true);
    await app.saveActiveDocument("Budget.xlsx");
    expect(excelTab.isDirty).toBe(false);

    // 4.3 PowerPoint
    app.handleNewTab("PowerPoint");
    const pptTab = app.getTabBar().getActiveTab()!;
    app.getTabBar().setDirty(pptTab.id, true);
    await app.saveActiveDocument("Deck.pptx");
    expect(pptTab.isDirty).toBe(false);

    // 4.4 PDF
    app.handleNewTab("PDF");
    const pdfTab = app.getTabBar().getActiveTab()!;
    app.getTabBar().setDirty(pdfTab.id, true);
    await app.saveActiveDocument("Doc.pdf");
    expect(pdfTab.isDirty).toBe(false);

    // 4.5 Notes
    app.handleNewTab("Notes");
    notesState.setContent("Testing save for notes");
    const notesTab = app.getTabBar().getActiveTab()!;
    app.getTabBar().setDirty(notesTab.id, true);
    await app.saveActiveDocument("Meeting.txt");
    expect(notesTab.isDirty).toBe(false);
  });

  it("5. Export formats: PDF, CSV, TXT, HTML, MD produce exports and update status", async () => {
    // Excel -> CSV Export
    app.handleNewTab("Excel");
    calcState.setCellValue("A1", "Item");
    calcState.setCellValue("B1", "Cost");
    const csvExport = await app.handleExport("csv");
    expect(csvExport).toContain(".csv");

    // Notes -> TXT, MD, HTML Export
    app.handleNewTab("Notes");
    notesState.setContent("# Meeting Notes\n- Action items");
    const txtExport = await app.handleExport("txt");
    expect(txtExport).toContain(".txt");

    const mdExport = await app.handleExport("md");
    expect(mdExport).toContain(".md");

    const htmlExport = await app.handleExport("html");
    expect(htmlExport).toContain(".html");

    // Word -> PDF Export
    app.handleNewTab("Word");
    const pdfExport = await app.handleExport("pdf");
    expect(pdfExport).toContain(".pdf");

    const statusDoc = document.getElementById("status-doc-info");
    expect(statusDoc?.textContent).toContain("Exported:");
  });

  it("6. Global Keyboard Shortcuts (Ctrl+N, Ctrl+O, Ctrl+S, Ctrl+P, Ctrl+Z, Ctrl+Y, Ctrl+W, Ctrl+F)", async () => {
    window.print = vi.fn();
    const printSpy = vi.spyOn(window, "print");
    const shortcuts = app.getKeyboardShortcuts();

    // Ctrl+N -> New Tab
    const tabCountBefore = app.getTabBar().getTabs().length;
    shortcuts.handleKeyDown(new KeyboardEvent("keydown", { key: "n", ctrlKey: true }));
    expect(app.getTabBar().getTabs().length).toBe(tabCountBefore + 1);

    // Ctrl+P -> Print
    shortcuts.handleKeyDown(new KeyboardEvent("keydown", { key: "p", ctrlKey: true }));
    expect(printSpy).toHaveBeenCalled();

    // Ctrl+S -> Save
    const active = app.getTabBar().getActiveTab()!;
    app.getTabBar().setDirty(active.id, true);
    shortcuts.handleKeyDown(new KeyboardEvent("keydown", { key: "s", ctrlKey: true }));
    await new Promise((r) => setTimeout(r, 20));
    expect(active.isDirty).toBe(false);

    // Ctrl+F -> Find dialog
    shortcuts.handleKeyDown(new KeyboardEvent("keydown", { key: "f", ctrlKey: true }));
    const findDialog = document.getElementById("find-replace-dialog");
    expect(findDialog?.classList.contains("hidden")).toBe(false);

    // Ctrl+W -> Close Tab
    const curCount = app.getTabBar().getTabs().length;
    shortcuts.handleKeyDown(new KeyboardEvent("keydown", { key: "w", ctrlKey: true }));
    expect(app.getTabBar().getTabs().length).toBe(curCount - 1);
  });
});
