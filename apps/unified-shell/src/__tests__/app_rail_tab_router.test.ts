import { describe, it, expect, beforeEach } from "vitest";
import { UnifiedShellApp } from "../main";

describe("Unified Left App Switcher Rail, Home Hub & Multi-Document Tab Router (GEC-v3.0 Subphase 21.1.2)", () => {
  beforeEach(() => {
    const mockCtx: Partial<CanvasRenderingContext2D> = {
      fillRect: () => {},
      strokeRect: () => {},
      fillText: () => {},
      strokeText: () => {},
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
      rect: () => {},
      scale: () => {},
      setTransform: () => {},
      translate: () => {},
      rotate: () => {},
      arc: () => {},
      roundRect: () => {},
      ellipse: () => {},
      measureText: (txt: string) => ({ width: txt.length * 8 } as TextMetrics),
      setLineDash: () => {},
      font: "14px Segoe UI, sans-serif",
      fillStyle: "#000",
      strokeStyle: "#000",
      lineWidth: 1,
      textAlign: "left",
      textBaseline: "top",
    };
    HTMLCanvasElement.prototype.getContext = (() => mockCtx) as any;
    if (!HTMLCanvasElement.prototype.scrollIntoView) {
      HTMLCanvasElement.prototype.scrollIntoView = () => {};
    }

    document.body.innerHTML = `
      <div id="app" class="desktop-shell">
        <header class="title-bar">
          <div class="window-controls">
            <button class="win-btn win-min">−</button>
            <button class="win-btn win-max">□</button>
            <button class="win-btn win-close">×</button>
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
              <span id="status-zoom-level"></span>
              <input type="range" id="zoom-slider" />
            </footer>
          </div>
        </div>
        <div id="bottom-sheet"></div>
        <canvas id="stylus-ink-canvas"></canvas>
        <div id="reflow-reader-container"></div>
        <div id="row-card-inspector-container"></div>
        <div id="laser-pointer-dot"></div>
      </div>
    `;
  });

  it("renders 6 Left App Switcher Rail icons (Home, Word, Excel, PPT, PDF, Notes)", () => {
    const app = new UnifiedShellApp(false);
    expect(app).toBeDefined();

    const rail = app.getAppRail();
    expect(rail).toBeDefined();

    const railContainer = document.getElementById("app-rail-container");
    expect(railContainer).not.toBeNull();

    const railBtns = railContainer!.querySelectorAll<HTMLButtonElement>(".rail-btn");
    expect(railBtns.length).toBe(6);

    const apps = Array.from(railBtns).map((btn) => btn.dataset.railApp);
    expect(apps).toEqual(["home", "word", "excel", "powerpoint", "pdf", "notes"]);
  });

  it("launches Excel from App Rail, rendering CalcGridCanvas and NEVER Word", () => {
    const app = new UnifiedShellApp(false);
    app.applyBootConfig({ app_mode: "writer" });

    // Initial state is Word
    expect(app.getCurrentDocType()).toBe("Word");

    // Click Excel on Left App Rail
    const excelRailBtn = document.querySelector<HTMLButtonElement>('[data-testid="rail-btn-excel"]');
    expect(excelRailBtn).not.toBeNull();
    excelRailBtn!.click();

    // Must be Excel
    expect(app.getCurrentDocType()).toBe("Excel");
    const activeTab = app.getTabBar().getActiveTab();
    expect(activeTab?.docType).toBe("Excel");
    expect(activeTab?.title).toContain("Book");

    // Must render calc grid, NEVER writer canvas
    expect(document.getElementById("calc-grid-container")).not.toBeNull();
    expect(document.querySelector(".calc-viewport-wrapper")).not.toBeNull();
  });

  it("launches PowerPoint from App Rail, rendering SlideStageCanvas and NEVER Word", () => {
    const app = new UnifiedShellApp(false);
    app.applyBootConfig({ app_mode: "writer" });

    // Click PowerPoint on Left App Rail
    const pptRailBtn = document.querySelector<HTMLButtonElement>('[data-testid="rail-btn-powerpoint"]');
    expect(pptRailBtn).not.toBeNull();
    pptRailBtn!.click();

    // Must be PowerPoint
    expect(app.getCurrentDocType()).toBe("PowerPoint");
    const activeTab = app.getTabBar().getActiveTab();
    expect(activeTab?.docType).toBe("PowerPoint");
    expect(activeTab?.title).toContain("Presentation");

    // Must render Slide Stage Canvas
    const canvas = document.querySelector<HTMLCanvasElement>("#viewport-container canvas");
    expect(canvas).not.toBeNull();
  });

  it("launches PDF Studio from App Rail, rendering PDF workspace", () => {
    const app = new UnifiedShellApp(false);
    app.applyBootConfig({ app_mode: "writer" });

    // Click PDF Studio on Left App Rail
    const pdfRailBtn = document.querySelector<HTMLButtonElement>('[data-testid="rail-btn-pdf"]');
    expect(pdfRailBtn).not.toBeNull();
    pdfRailBtn!.click();

    expect(app.getCurrentDocType()).toBe("PDF");
    const activeTab = app.getTabBar().getActiveTab();
    expect(activeTab?.docType).toBe("PDF");
    expect(activeTab?.title).toContain("Document");

    // Must render PDF workspace
    expect(document.querySelector(".pdf-studio-workspace")).not.toBeNull();
  });

  it("launches Notepad / Notes from App Rail, rendering notes textarea", () => {
    const app = new UnifiedShellApp(false);
    app.applyBootConfig({ app_mode: "writer" });

    // Click Notes on Left App Rail
    const notesRailBtn = document.querySelector<HTMLButtonElement>('[data-testid="rail-btn-notes"]');
    expect(notesRailBtn).not.toBeNull();
    notesRailBtn!.click();

    expect(app.getCurrentDocType()).toBe("Notes");
    const activeTab = app.getTabBar().getActiveTab();
    expect(activeTab?.docType).toBe("Notes");
    expect(activeTab?.title).toContain("Untitled");

    // Must render notes workspace
    expect(document.querySelector('[data-testid="notes-workspace"]')).not.toBeNull();
    const textarea = document.getElementById("notes-editor-textarea") as HTMLTextAreaElement | null;
    expect(textarea).not.toBeNull();
  });

  it("switches to Home Hub when clicking Home icon on App Rail, showing 5 launch cards", async () => {
    const app = new UnifiedShellApp(false);
    app.applyBootConfig({ app_mode: "writer" });

    const homeRailBtn = document.querySelector<HTMLButtonElement>('[data-testid="rail-btn-home"]');
    expect(homeRailBtn).not.toBeNull();
    homeRailBtn!.click();

    for (let i = 0; i < 50; i++) {
      if (document.querySelector('[data-testid="hub-launcher"]')) break;
      await new Promise((resolve) => setTimeout(resolve, 20));
    }

    const appContainer = document.getElementById("app");
    expect(appContainer?.classList.contains("shell-hub-mode")).toBe(true);

    const hubEl = document.querySelector('[data-testid="hub-launcher"]');
    expect(hubEl).not.toBeNull();

    // Assert 5 standalone app cards: Writer, Calc, Slides, PDF, Notes
    expect(document.querySelector('[data-testid="app-card-writer"]')).not.toBeNull();
    expect(document.querySelector('[data-testid="app-card-calc"]')).not.toBeNull();
    expect(document.querySelector('[data-testid="app-card-slides"]')).not.toBeNull();
    expect(document.querySelector('[data-testid="app-card-pdf"]')).not.toBeNull();
    expect(document.querySelector('[data-testid="app-card-notes"]')).not.toBeNull();
  });

  it("opens quick dropdown when clicking + New Tab button and launches selected tab type", () => {
    const app = new UnifiedShellApp(false);
    app.applyBootConfig({ app_mode: "writer" });

    const addBtn = document.querySelector<HTMLButtonElement>('[data-testid="tab-add-btn"]');
    expect(addBtn).not.toBeNull();

    const dropdown = document.querySelector<HTMLElement>('[data-testid="tab-new-dropdown"]');
    expect(dropdown).not.toBeNull();
    expect(dropdown!.style.display).toBe("none");

    // Click + to open dropdown
    addBtn!.click();
    expect(dropdown!.style.display).toBe("block");

    // Click "Blank Excel Workbook"
    const newExcelItem = document.querySelector<HTMLButtonElement>('[data-testid="dropdown-new-excel"]');
    expect(newExcelItem).not.toBeNull();
    newExcelItem!.click();

    // Dropdown closes and Excel tab is added
    expect(dropdown!.style.display).toBe("none");
    expect(app.getCurrentDocType()).toBe("Excel");
    expect(app.getTabBar().getTabs().length).toBe(2);
  });

  it("closes tab cleanly when clicking × button", () => {
    const app = new UnifiedShellApp(false);
    app.applyBootConfig({ app_mode: "writer" });

    // Add a second tab
    app.handleNewTab("Excel");
    expect(app.getTabBar().getTabs().length).toBe(2);

    // Close the second tab
    const closeBtns = document.querySelectorAll<HTMLButtonElement>(".tab-close-btn");
    expect(closeBtns.length).toBe(2);
    closeBtns[1].click();

    expect(app.getTabBar().getTabs().length).toBe(1);
    expect(app.getTabBar().getTabs()[0].docType).toBe("Word");
  });
});
