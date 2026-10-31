import { describe, it, expect, beforeEach, vi } from "vitest";
import { UnifiedShellApp } from "../main";
import { binaryPackageService } from "../services/binaryPackageService";
import { fileIoService } from "../services/fileIoService";
import { IPC } from "../ipc";

describe("Subphase 28.1.2: Multi-Document Management, Split View & Real Binary File I/O", () => {
  beforeEach(() => {
    localStorage.clear();
    IPC._resetMockState();

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
            <span class="app-name">AetherOffice</span>
            <div class="quick-access-toolbar">
              <button id="btn-quick-open">📂</button>
              <button id="btn-quick-save">💾</button>
              <button id="btn-quick-print">🖨️</button>
            </div>
          </div>
          <div class="title-actions">
            <button id="btn-theme-toggle" class="theme-toggle-btn">
              <span class="theme-icon">☀️</span>
              <span class="theme-label">Light</span>
            </button>
          </div>
        </header>
        <div class="shell-body" style="display:flex; flex:1;">
          <aside id="app-rail-container" class="app-rail"></aside>
          <div class="shell-workspace-area" style="display:flex; flex-direction:column; flex:1;">
            <nav id="tab-bar-container" class="tab-bar"></nav>
            <section id="ribbon-container" class="ribbon-bar"></section>
            <section id="sub-header-container" class="sub-header-bar"></section>
            <main class="workspace-layout">
              <aside id="sidebar-container" class="sidebar-pane"></aside>
              <section class="canvas-viewport" id="viewport-container">
                <canvas id="render-surface"></canvas>
              </section>
            </main>
            <footer id="status-bar" class="status-bar">
              <span id="status-doc-info">Page 1 of 1</span>
              <span id="status-word-count">0 words</span>
            </footer>
          </div>
        </div>
      </div>
    `;
    localStorage.clear();
  });

  // =========================================================================
  // 1. Multi-Document Simultaneous Tabs & App Rail Badge Counts
  // =========================================================================
  it("1. Manages multiple simultaneous documents per format with accurate App Rail badges and filter toggle", () => {
    const app = new UnifiedShellApp(false);
    app.init(false);

    // Open 3 simultaneous Word documents
    app.handleNewTab("Word"); // Untitled 1.docx
    app.handleNewTab("Word"); // Untitled 2.docx
    app.handleNewTab("Word"); // Untitled 3.docx

    // Open 2 simultaneous Excel documents
    app.handleNewTab("Excel"); // Book1.xlsx
    app.handleNewTab("Excel"); // Book2.xlsx

    const tabBar = app.getTabBar();
    const tabs = tabBar.getTabs();
    expect(tabs.length).toBeGreaterThanOrEqual(5);

    const wordTabs = tabs.filter((t) => t.docType === "Word");
    const excelTabs = tabs.filter((t) => t.docType === "Excel");
    expect(wordTabs.length).toBe(3);
    expect(excelTabs.length).toBe(2);

    // Verify AppRail badges
    const rail = app.getAppRail()!;
    expect(rail).toBeDefined();
    const counts = rail.getCounts();
    expect(counts.word).toBe(3);
    expect(counts.excel).toBe(2);

    const wordBadge = document.querySelector('[data-testid="rail-badge-word"]');
    expect(wordBadge).not.toBeNull();
    expect(wordBadge?.textContent).toBe("3");

    const excelBadge = document.querySelector('[data-testid="rail-badge-excel"]');
    expect(excelBadge).not.toBeNull();
    expect(excelBadge?.textContent).toBe("2");

    // Test Tab Filter Toggle: Current App Tabs vs All Open Tabs (Defaults to current under GEC-v6.0)
    expect(tabBar.getFilterMode()).toBe("current");
    const filterBtn = document.querySelector<HTMLButtonElement>('[data-testid="tab-filter-toggle"]');
    expect(filterBtn).not.toBeNull();
    filterBtn?.click();
    expect(tabBar.getFilterMode()).toBe("all");
    filterBtn?.click();
    expect(tabBar.getFilterMode()).toBe("current");
  });

  // =========================================================================
  // 2. Side-by-Side Split View Workspace
  // =========================================================================
  it("2. Toggles Side-by-Side Split View with independent document view and pane controls", () => {
    const app = new UnifiedShellApp(false);
    app.init(false);

    app.handleNewTab("Word");
    app.handleNewTab("PDF");

    expect(app.isSplitView()).toBe(false);

    // Toggle Split View ON
    const splitActive = app.toggleSplitView();
    expect(splitActive).toBe(true);
    expect(app.isSplitView()).toBe(true);

    const layout = document.querySelector(".workspace-layout")!;
    expect(layout.classList.contains("split-view-active")).toBe(true);

    const splitPane = document.getElementById("workspace-split-pane");
    expect(splitPane).not.toBeNull();

    const select = splitPane!.querySelector<HTMLSelectElement>('[data-testid="split-pane-select"]');
    expect(select).not.toBeNull();
    expect(select?.options.length).toBeGreaterThan(0);

    // Toggle Split View OFF
    app.toggleSplitView();
    expect(app.isSplitView()).toBe(false);
    expect(layout.classList.contains("split-view-active")).toBe(false);
    expect(document.getElementById("workspace-split-pane")).toBeNull();
  });

  // =========================================================================
  // 3. Real Binary File Open & Save Round-Trip (All 5 Formats)
  // =========================================================================
  it("3. Performs real round-trip binary packaging and parsing for .docx, .xlsx, .pptx, .pdf, and .txt", async () => {
    // 3.1 DOCX Round-Trip
    const docxBytes = await binaryPackageService.buildDocx("Executive Summary\nAetherOffice Enterprise", [
      { text: "Executive Summary", bold: true },
      { text: "AetherOffice Enterprise", italic: true },
    ]);
    expect(docxBytes.length).toBeGreaterThan(100);
    const parsedDocx = await binaryPackageService.parseDocx(docxBytes);
    expect(parsedDocx.paragraphs).toContain("Executive Summary");
    expect(parsedDocx.paragraphs).toContain("AetherOffice Enterprise");

    // 3.2 XLSX Round-Trip
    const xlsxBytes = await binaryPackageService.buildXlsx({
      A1: "Quarter",
      B1: "Revenue",
      A2: "Q1",
      B2: "150000",
    });
    expect(xlsxBytes.length).toBeGreaterThan(100);
    const parsedXlsx = await binaryPackageService.parseXlsx(xlsxBytes);
    expect(parsedXlsx.cells["A1"]).toBe("Quarter");
    expect(parsedXlsx.cells["B1"]).toBe("Revenue");
    expect(parsedXlsx.cells["A2"]).toBe("Q1");
    expect(parsedXlsx.cells["B2"]).toBe("150000");

    // 3.3 PPTX Round-Trip
    const pptxBytes = await binaryPackageService.buildPptx([
      { title: "Keynote 2026", shapes: [{ text: "Product Roadmap" }] },
      { title: "Financials", shapes: [{ text: "Gross Margin 82%" }] },
    ]);
    expect(pptxBytes.length).toBeGreaterThan(100);
    const parsedPptx = await binaryPackageService.parsePptx(pptxBytes);
    expect(parsedPptx.slides.length).toBe(2);
    expect(parsedPptx.slides[0].title).toBe("Keynote 2026");
    expect(parsedPptx.slides[1].title).toBe("Financials");

    // 3.4 PDF Binary Round-Trip
    const pdfBytes = binaryPackageService.buildPdf(["ISO 32000-2 Certified PDF Studio Document"]);
    expect(pdfBytes.length).toBeGreaterThan(100);
    const parsedPdf = binaryPackageService.parsePdf(pdfBytes);
    expect(parsedPdf.pageCount).toBe(1);
    expect(parsedPdf.text).toContain("ISO 32000-2 Certified PDF Studio Document");

    // 3.5 TXT / Markdown Round-Trip
    const mdBytes = binaryPackageService.buildText("# System Notes\n- Verified feature");
    const parsedMd = binaryPackageService.parseText(mdBytes);
    expect(parsedMd).toBe("# System Notes\n- Verified feature");
  });

  it("4. Loads binary files through FileIoService into application state without errors", async () => {
    // Test loading .docx buffer into Writer
    const docxBytes = await binaryPackageService.buildDocx("Loaded Document Content");
    const okDocx = await fileIoService.loadFileContent(docxBytes, "Word", "Contract.docx");
    expect(okDocx).toBe(true);

    // Test loading .xlsx buffer into Calc
    const xlsxBytes = await binaryPackageService.buildXlsx({ A1: "LoadedValue" });
    const okXlsx = await fileIoService.loadFileContent(xlsxBytes, "Excel", "Data.xlsx");
    expect(okXlsx).toBe(true);
  });
});
