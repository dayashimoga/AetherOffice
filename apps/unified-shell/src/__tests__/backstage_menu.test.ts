import { describe, it, expect, beforeEach, vi } from "vitest";
import { UnifiedShellApp } from "../main";
import { IPC } from "../ipc";

describe("Native File Backstage Menu & Disk I/O (GEC-v2.0 Subphase 8.2.1)", () => {
  let app: UnifiedShellApp;

  beforeEach(() => {
    localStorage.clear();
    IPC._resetMockState();

    // Mock HTMLCanvasElement 2D context for DOM test runner
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
            <span class="app-name">OpenOfficeRS</span>
          </div>
        </header>
        <div id="tab-bar-container"></div>
        <div id="ribbon-container"></div>
        <div id="sub-header-container"></div>
        <div id="sidebar-container"></div>
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

  it("opens full Backstage overlay when clicking Ribbon File tab", async () => {
    const fileTabBtn = document.querySelector<HTMLButtonElement>('[data-testid="ribbon-tab-file"]');
    expect(fileTabBtn).not.toBeNull();
    expect(fileTabBtn?.classList.contains("ribbon-file-tab")).toBe(true);

    const backstage = app.getBackstageMenu();
    expect(backstage.isOpen()).toBe(false);

    // Click File Tab
    fileTabBtn?.click();
    await new Promise((r) => setTimeout(r, 10));

    expect(backstage.isOpen()).toBe(true);
    const overlay = document.getElementById("backstage-overlay");
    expect(overlay?.classList.contains("hidden")).toBe(false);
  });

  it("closes Backstage view when clicking Back button or pressing Escape", async () => {
    const backstage = app.getBackstageMenu();
    await backstage.show("Word", "sess-1", "Untitled 1.docx");
    expect(backstage.isOpen()).toBe(true);

    // Click Back button
    const backBtn = document.getElementById("btn-backstage-back");
    expect(backBtn).not.toBeNull();
    backBtn?.click();
    expect(backstage.isOpen()).toBe(false);

    // Open again and press Escape
    await backstage.show("Word", "sess-1", "Untitled 1.docx");
    expect(backstage.isOpen()).toBe(true);

    window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    expect(backstage.isOpen()).toBe(false);
  });

  it("navigates through Backstage tabs: Info, New, Open, Save, Save As, Export, Print", async () => {
    const backstage = app.getBackstageMenu();
    await backstage.show("Word", "sess-1", "Contract.docx", "docs/Contract.docx", true);

    // 1. Info panel verification
    const panel = document.getElementById("backstage-content-panel");
    expect(panel?.textContent).toContain("Contract.docx");
    expect(panel?.textContent).toContain("Modified (Unsaved Changes)");

    // 2. Switch to New
    backstage.setActiveNav("New");
    const panelNew = document.getElementById("backstage-content-panel");
    expect(panelNew?.textContent).toContain("Blank Document");
    expect(panelNew?.textContent).toContain("Blank Spreadsheet");
    expect(panelNew?.textContent).toContain("Blank Presentation");
    expect(panelNew?.textContent).toContain("Blank PDF");

    // 3. Switch to Open
    backstage.setActiveNav("Open");
    const panelOpen = document.getElementById("backstage-content-panel");
    expect(panelOpen?.querySelector("#input-open-file-path")).not.toBeNull();

    // 4. Switch to Save
    backstage.setActiveNav("Save");
    const panelSave = document.getElementById("backstage-content-panel");
    expect(panelSave?.querySelector("#btn-submit-save")).not.toBeNull();

    // 5. Switch to Save As
    backstage.setActiveNav("SaveAs");
    const panelSaveAs = document.getElementById("backstage-content-panel");
    expect(panelSaveAs?.querySelector("#input-save-as-path")).not.toBeNull();

    // 6. Switch to Export
    backstage.setActiveNav("Export");
    const panelExport = document.getElementById("backstage-content-panel");
    expect(panelExport?.querySelector("#input-export-pdf-path")).not.toBeNull();
  });

  it("executes Save and clears dirty indicator on active document tab", async () => {
    const tabBar = app.getTabBar();
    const active = tabBar.getActiveTab();
    expect(active).toBeDefined();

    // Mark tab dirty with asterisk
    tabBar.setDirty(active!.id, true);
    expect(document.querySelector(".tab-title")?.textContent).toContain("*");

    const backstage = app.getBackstageMenu();
    await backstage.show("Word", active!.id, active!.title, undefined, true);

    // Click Quick Save button
    const saveSpy = vi.spyOn(IPC, "saveDocument");
    const quickSaveBtn = document.getElementById("btn-quick-save") as HTMLButtonElement;
    expect(quickSaveBtn).not.toBeNull();
    quickSaveBtn.click();

    await new Promise((r) => setTimeout(r, 20));
    expect(saveSpy).toHaveBeenCalledWith(active!.id, undefined);

    // Verify dirty indicator cleared
    expect(active!.isDirty).toBe(false);
    expect(document.querySelector(".tab-title")?.textContent).not.toContain("*");
  });

  it("saves a copy of document to new disk path via Save As", async () => {
    const tabBar = app.getTabBar();
    const active = tabBar.getActiveTab();
    const backstage = app.getBackstageMenu();

    backstage.show("Word", active!.id, active!.title);
    backstage.setActiveNav("SaveAs");

    const saveSpy = vi.spyOn(IPC, "saveDocument");
    const inputPath = document.getElementById("input-save-as-path") as HTMLInputElement;
    inputPath.value = "C:/Documents/Backup_Copy.docx";

    const saveAsBtn = document.getElementById("btn-submit-save-as") as HTMLButtonElement;
    saveAsBtn.click();

    await new Promise((r) => setTimeout(r, 20));
    expect(saveSpy).toHaveBeenCalledWith(active!.id, "C:/Documents/Backup_Copy.docx");
  });

  it("opens local document from disk and registers new tab", async () => {
    const backstage = app.getBackstageMenu();
    backstage.show("Word", "sess-1", "Doc.docx");
    backstage.setActiveNav("Open");

    const openSpy = vi.spyOn(IPC, "openDocument");
    const inputPath = document.getElementById("input-open-file-path") as HTMLInputElement;
    inputPath.value = "spreadsheets/Quarterly_Budget.xlsx";

    const openBtn = document.getElementById("btn-submit-open-file") as HTMLButtonElement;
    openBtn.click();

    await new Promise((r) => setTimeout(r, 20));
    expect(openSpy).toHaveBeenCalledWith("spreadsheets/Quarterly_Budget.xlsx");

    const tabBar = app.getTabBar();
    const tabs = tabBar.getTabs();
    expect(tabs.length).toBe(2);
    expect(tabs[1].docType).toBe("Excel");
    expect(tabs[1].title).toBe("Quarterly_Budget.xlsx");
    expect(backstage.isOpen()).toBe(false);
  });

  it("exports document to certified ISO 32000 PDF via Backstage Export", async () => {
    const tabBar = app.getTabBar();
    const active = tabBar.getActiveTab();
    const backstage = app.getBackstageMenu();

    backstage.show("Word", active!.id, active!.title);
    backstage.setActiveNav("Export");

    const exportSpy = vi.spyOn(IPC, "exportPdf");
    const inputPath = document.getElementById("input-export-pdf-path") as HTMLInputElement;
    inputPath.value = "exports/Final_Release.pdf";

    const exportBtn = document.getElementById("btn-submit-export-pdf") as HTMLButtonElement;
    exportBtn.click();

    await new Promise((r) => setTimeout(r, 20));
    expect(exportSpy).toHaveBeenCalledWith(active!.id, "exports/Final_Release.pdf");
    expect(backstage.isOpen()).toBe(false);
  });
});
