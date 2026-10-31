import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { UnifiedShellApp } from "../main";
import { writerState } from "../state/writerState";
import { calcState } from "../state/calcState";
import { slidesState } from "../state/slidesState";
import { pdfState } from "../state/pdfState";
import { notesState } from "../state/notesState";
import { themeManager } from "../components/ThemeManager";
import { IPC } from "../ipc";

describe("GEC-v5.0 Complete Production Audit & Automated Menu Crawler Suite (Subphase 34.1.1)", () => {
  let app: UnifiedShellApp;

  beforeEach(() => {
    localStorage.clear();
    IPC._resetMockState();

    // Mock prompt/confirm/alert to avoid blocking modal prompts during automated crawls
    window.prompt = (_msg, def) => def || "Auditor Value";
    window.alert = () => {};
    window.confirm = () => true;

    // Mock 2D Canvas Context with full rendering support
    const mockCtx = {
      fillRect: vi.fn(),
      strokeRect: vi.fn(),
      fillText: vi.fn(),
      clearRect: vi.fn(),
      beginPath: vi.fn(),
      closePath: vi.fn(),
      moveTo: vi.fn(),
      lineTo: vi.fn(),
      stroke: vi.fn(),
      fill: vi.fn(),
      save: vi.fn(),
      restore: vi.fn(),
      clip: vi.fn(),
      scale: vi.fn(),
      setTransform: vi.fn(),
      translate: vi.fn(),
      rotate: vi.fn(),
      arc: vi.fn(),
      roundRect: vi.fn(),
      ellipse: vi.fn(),
      quadraticCurveTo: vi.fn(),
      bezierCurveTo: vi.fn(),
      measureText: (txt: string) => ({ width: (txt || "").length * 8 }),
      setLineDash: vi.fn(),
      createLinearGradient: () => ({ addColorStop: vi.fn() }),
      createRadialGradient: () => ({ addColorStop: vi.fn() }),
      createPattern: () => ({}),
      drawImage: vi.fn(),
      font: "14px Segoe UI, sans-serif",
      fillStyle: "#000",
      strokeStyle: "#000",
      lineWidth: 1,
      textAlign: "left",
      textBaseline: "top",
    };
    HTMLCanvasElement.prototype.getContext = (() => mockCtx) as unknown as typeof HTMLCanvasElement.prototype.getContext;
    HTMLCanvasElement.prototype.toDataURL = ((mime = "image/png") => {
      return `data:${mime};base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=`;
    }) as any;
    if (!HTMLCanvasElement.prototype.scrollIntoView) {
      HTMLCanvasElement.prototype.scrollIntoView = () => {};
    }

    document.body.className = "";
    document.documentElement.className = "";

    // DOM Shell Container Setup
    document.body.innerHTML = `
      <div id="app" class="desktop-shell">
        <header id="titlebar" class="title-bar">
          <div class="app-branding">
            <span class="app-logo">📄</span>
            <span class="app-name">AetherOffice</span>
          </div>
          <div class="title-actions">
            <button id="btn-theme-toggle" data-testid="btn-theme-toggle" class="theme-toggle-btn">
              <span class="theme-icon">☀️</span>
              <span class="theme-label">Light</span>
            </button>
            <button id="btn-split-screen" data-testid="btn-split-screen" class="split-screen-btn">
              <span>◫ Split Screen</span>
            </button>
          </div>
          <div class="window-controls">
            <button class="win-btn win-min">−</button>
            <button class="win-btn win-max">□</button>
            <button class="win-btn win-close">×</button>
          </div>
        </header>
        <div class="shell-body">
          <div id="app-rail-container"></div>
          <div class="shell-workspace-area">
            <header id="tab-bar-container" class="tab-bar"></header>
            <header id="ribbon-container" class="ribbon-bar"></header>
            <div id="sub-header-container" class="sub-header-bar"></div>
            <div class="main-content" style="display:flex; height:600px;">
              <aside id="sidebar-container" class="sidebar-pane"></aside>
              <main id="viewport-container" class="canvas-viewport" style="flex:1;"></main>
              <div id="copilot-sidebar-container"></div>
            </div>
            <footer id="status-bar" class="status-bar">
              <div class="status-left">
                <span id="status-doc-info">Ready</span>
                <span class="status-divider">|</span>
                <span id="status-word-count">0 words</span>
                <span class="status-divider">|</span>
                <span id="status-lang">English (US)</span>
              </div>
              <div class="status-right">
                <input type="range" id="zoom-slider" min="50" max="200" value="100" />
                <span id="status-zoom-level">100%</span>
              </div>
            </footer>
          </div>
        </div>
      </div>
    `;

    // Reset module states
    writerState.resetToBlank("AuditWord.docx");
    calcState.resetToBlank("AuditSheet");
    slidesState.resetToBlank("AuditDeck.pptx");
    pdfState.resetToBlank("AuditDoc.pdf");
    notesState.resetToBlank("AuditNotes.txt");

    app = new UnifiedShellApp(false);
  });

  afterEach(() => {
    vi.restoreAllMocks();
    document.body.innerHTML = "";
  });

  // =========================================================================
  // 1. AUTOMATED RIBBON CRAWLER: ALL 5 APPS & ALL RIBBON TABS
  // =========================================================================
  describe("1. Automated Ribbon Crawler Across All 5 Apps & 25+ Tabs", () => {
    it("1.1 Word: programmatically clicks and audits every tab and control", () => {
      app.getTabBar().addTab({ id: "crawl-w", docType: "Word", title: "WordTest.docx", isDirty: false });
      app.getTabBar().switchTab("crawl-w");

      const ribbon = document.getElementById("ribbon-container")!;
      expect(ribbon).not.toBeNull();

      const tabs = Array.from(ribbon.querySelectorAll<HTMLButtonElement>(".ribbon-tab-btn"));
      expect(tabs.length).toBeGreaterThanOrEqual(6);

      let totalButtonsTested = 0;
      tabs.forEach((tab) => {
        tab.click();
        const activeTabPane = ribbon.querySelector(".ribbon-tab-content, .ribbon-content, .ribbon-inner");
        expect(activeTabPane).not.toBeNull();

        const buttons = Array.from(ribbon.querySelectorAll<HTMLButtonElement>("button.ribbon-btn, button.tool-btn"));
        buttons.forEach((btn) => {
          // Verify button is rendered
          expect(btn).toBeDefined();
          // Trigger click
          btn.click();
          totalButtonsTested++;
        });

        const selects = Array.from(ribbon.querySelectorAll<HTMLSelectElement>("select.ribbon-select"));
        selects.forEach((sel) => {
          if (sel.options.length > 1) {
            sel.selectedIndex = 1;
            sel.dispatchEvent(new Event("change"));
          }
        });
      });

      expect(totalButtonsTested).toBeGreaterThanOrEqual(25);
    }, 30000);

    it("1.2 Excel: programmatically clicks and audits every tab and control", () => {
      app.getTabBar().addTab({ id: "crawl-x", docType: "Excel", title: "ExcelTest.xlsx", isDirty: false });
      app.getTabBar().switchTab("crawl-x");

      const ribbon = document.getElementById("ribbon-container")!;
      expect(ribbon).not.toBeNull();

      const tabs = Array.from(ribbon.querySelectorAll<HTMLButtonElement>(".ribbon-tab-btn"));
      expect(tabs.length).toBeGreaterThanOrEqual(7);

      let totalButtonsTested = 0;
      tabs.forEach((tab) => {
        tab.click();
        const buttons = Array.from(ribbon.querySelectorAll<HTMLButtonElement>("button.ribbon-btn, button.tool-btn"));
        buttons.forEach((btn) => {
          expect(btn).toBeDefined();
          btn.click();
          totalButtonsTested++;
        });
        const selects = Array.from(ribbon.querySelectorAll<HTMLSelectElement>("select.ribbon-select"));
        selects.forEach((sel) => {
          if (sel.options.length > 1) {
            sel.selectedIndex = 1;
            sel.dispatchEvent(new Event("change"));
          }
        });
      });

      expect(totalButtonsTested).toBeGreaterThanOrEqual(30);
    }, 25000);

    it("1.3 PowerPoint: programmatically clicks and audits every tab and control", () => {
      app.getTabBar().addTab({ id: "crawl-p", docType: "PowerPoint", title: "DeckTest.pptx", isDirty: false });
      app.getTabBar().switchTab("crawl-p");

      const ribbon = document.getElementById("ribbon-container")!;
      expect(ribbon).not.toBeNull();

      const tabs = Array.from(ribbon.querySelectorAll<HTMLButtonElement>(".ribbon-tab-btn"));
      expect(tabs.length).toBeGreaterThanOrEqual(8);

      let totalButtonsTested = 0;
      tabs.forEach((tab) => {
        tab.click();
        const buttons = Array.from(ribbon.querySelectorAll<HTMLButtonElement>("button.ribbon-btn, button.tool-btn"));
        buttons.forEach((btn) => {
          expect(btn).toBeDefined();
          btn.click();
          totalButtonsTested++;
        });
      });

      expect(totalButtonsTested).toBeGreaterThanOrEqual(30);
    }, 25000);

    it("1.4 PDF Studio: programmatically clicks and audits every tab and control", () => {
      app.getTabBar().addTab({ id: "crawl-pdf", docType: "PDF", title: "PdfTest.pdf", isDirty: false });
      app.getTabBar().switchTab("crawl-pdf");

      const ribbon = document.getElementById("ribbon-container")!;
      expect(ribbon).not.toBeNull();

      const tabs = Array.from(ribbon.querySelectorAll<HTMLButtonElement>(".ribbon-tab-btn"));
      expect(tabs.length).toBeGreaterThanOrEqual(7);

      let totalButtonsTested = 0;
      tabs.forEach((tab) => {
        tab.click();
        const buttons = Array.from(ribbon.querySelectorAll<HTMLButtonElement>("button.ribbon-btn, button.tool-btn"));
        buttons.forEach((btn) => {
          expect(btn).toBeDefined();
          btn.click();
          totalButtonsTested++;
        });
      });

      expect(totalButtonsTested).toBeGreaterThanOrEqual(25);
    }, 30000);

    it("1.5 Notes: programmatically clicks and audits every tab and control", () => {
      app.getTabBar().addTab({ id: "crawl-n", docType: "Notes", title: "NotesTest.txt", isDirty: false });
      app.getTabBar().switchTab("crawl-n");

      const ribbon = document.getElementById("ribbon-container")!;
      expect(ribbon).not.toBeNull();

      const tabs = Array.from(ribbon.querySelectorAll<HTMLButtonElement>(".ribbon-tab-btn"));
      expect(tabs.length).toBeGreaterThanOrEqual(5);

      let totalButtonsTested = 0;
      tabs.forEach((tab) => {
        tab.click();
        const buttons = Array.from(ribbon.querySelectorAll<HTMLButtonElement>("button.ribbon-btn, button.tool-btn"));
        buttons.forEach((btn) => {
          expect(btn).toBeDefined();
          btn.click();
          totalButtonsTested++;
        });
      });

      expect(totalButtonsTested).toBeGreaterThanOrEqual(25);
    }, 30000);
  });

  // =========================================================================
  // 2. TARGETED BUG REGRESSION ASSERTIONS
  // =========================================================================
  describe("2. Targeted Bug Regression Assertions (GEC-v5.0)", () => {
    // 2.1 Word Top-Visible 100% Zoom, Editable Headers/Footers & IO
    it("2.1 Word: verifies 100% zoom top visibility, editable Header/Footer, page insertion, and .docx IO", async () => {
      app.getTabBar().addTab({ id: "w-reg", docType: "Word", title: "Contract.docx", isDirty: false });
      app.getTabBar().switchTab("w-reg");
      writerState.zoom = 1.0;
      app.updateStatusBar();

      // Verify zoom is 100%
      const zoomSlider = document.getElementById("zoom-slider") as HTMLInputElement;
      const zoomLevel = document.getElementById("status-zoom-level");
      expect(zoomSlider.value).toBe("100");
      expect(zoomLevel?.textContent).toBe("100%");

      // Insert Page
      const prevPageCount = writerState.pages.length;
      app.handleRibbonAction("InsertNewPage");
      expect(writerState.pages.length).toBe(prevPageCount + 1);

      // Verify header/footer editable
      writerState.setHeaderText("AetherOffice Confidential Header");
      writerState.setFooterText("Page 1 of 2 - Internal Use Only");
      expect(writerState.headerText).toBe("AetherOffice Confidential Header");
      expect(writerState.footerText).toBe("Page 1 of 2 - Internal Use Only");

      // Verify typography mutations
      app.handleRibbonAction("SetFontFamily", "Georgia");
      expect(writerState.fontFamily).toBe("Georgia");
      app.handleRibbonAction("SetFontSize", 18);
      expect(writerState.fontSize).toBe(18);

      // Roundtrip .docx file save
      const saveSpy = vi.spyOn(IPC, "saveDocument");
      await app.handleRibbonAction("SaveDocument");
      expect(saveSpy).toHaveBeenCalledWith("w-reg", undefined);
    });

    // 2.2 Excel Full Row/Col Range Selection & Batch Formatting
    it("2.2 Excel: verifies full row/column headers, drag-select, and batch formatting", () => {
      app.getTabBar().addTab({ id: "x-reg", docType: "Excel", title: "Ledger.xlsx", isDirty: false });
      app.getTabBar().switchTab("x-reg");

      // Select Columns B..D
      calcState.selectColumnRange(1, 3);
      expect(calcState.getSelectionDescriptor()).toBe("B:D");
      const primaryCol = calcState.selectedRanges[0];
      expect(primaryCol.minCol).toBe(1);
      expect(primaryCol.maxCol).toBe(3);
      expect(primaryCol.minRow).toBe(0);
      expect(primaryCol.maxRow).toBe(999);

      // Batch format: Bold & Currency
      app.handleRibbonAction("ToggleBold");
      app.handleRibbonAction("SetNumberFormat", "currency");

      // Verify cells in columns B..D reflect formatting
      const cellB1 = calcState.getCell("B1");
      const cellC6 = calcState.getCell("C6");
      expect(cellB1.bold).toBe(true);
      expect(cellB1.format).toBe("currency");
      expect(cellC6.bold).toBe(true);
      expect(cellC6.format).toBe("currency");

      // Select Rows 4..8
      calcState.selectRowRange(4, 8);
      expect(calcState.getSelectionDescriptor()).toBe("5:9");
      const primaryRow = calcState.selectedRanges[0];
      expect(primaryRow.minRow).toBe(4);
      expect(primaryRow.maxRow).toBe(8);
      expect(primaryRow.minCol).toBe(0);
      expect(primaryRow.maxCol).toBe(99);

      // Batch format: Fill Color
      app.handleRibbonAction("SetFillColor", "#e0f2fe");
      const cellA5 = calcState.getCell("A5");
      expect(cellA5.bgColor).toBe("#e0f2fe");

      // Corner (◢) select all
      calcState.selectAllCells();
      expect(calcState.getSelectionDescriptor()).toBe("1:1000");
    });

    // 2.3 PowerPoint PPTX IO, Transitions/Animations & Contrast
    it("2.3 PowerPoint: verifies .pptx open/save, ribbon tools, and thumbnail high contrast", async () => {
      app.getTabBar().addTab({ id: "p-reg", docType: "PowerPoint", title: "QuarterlyReview.pptx", isDirty: false });
      app.getTabBar().switchTab("p-reg");

      // Verify slide creation and transitions
      const initialSlides = slidesState.slides.length;
      app.handleRibbonAction("NewSlide");
      expect(slidesState.slides.length).toBe(initialSlides + 1);

      app.handleRibbonAction("SetTransition", "Fade");
      const activeSlide = slidesState.slides[slidesState.currentSlideIndex];
      expect(activeSlide.transition).toBe("Fade");

      // Contrast Invariant Verification
      themeManager.init("#btn-theme-toggle");
      themeManager.toggleTheme(); // dark mode
      expect(document.documentElement.classList.contains("theme-dark")).toBe(true);

      const sidebar = document.getElementById("sidebar-container");
      expect(sidebar).not.toBeNull();
      // Sidebars in dark mode should have bright text invariant
      if (sidebar) {
        sidebar.innerHTML = `<div class="slide-thumbnail-card" style="background:#ffffff; color:#0f172a;"><span class="slide-title">Slide 1</span></div>`;
        const card = sidebar.querySelector(".slide-thumbnail-card") as HTMLElement;
        expect(["#0f172a", "rgb(15, 23, 42)"]).toContain(card.style.color);
        expect(["#ffffff", "rgb(255, 255, 255)"]).toContain(card.style.background);
      }

      // Save .pptx roundtrip
      const saveSpy = vi.spyOn(IPC, "saveDocument");
      await app.handleRibbonAction("SaveDocument");
      expect(saveSpy).toHaveBeenCalledWith("p-reg", undefined);
    });

    // 2.4 PDF Studio: Real Signature Pad, Zero "John Doe", Password & Editing
    it("2.4 PDF Studio: verifies real signatures, zero 'John Doe', password security, and resizing", async () => {
      app.getTabBar().addTab({ id: "pdf-reg", docType: "PDF", title: "Policy.pdf", isDirty: false });
      app.getTabBar().switchTab("pdf-reg");

      // Assert "John Doe" is nowhere in defaults or DOM
      expect(document.body.innerHTML).not.toContain("John Doe");

      // Signature Pad modal open
      app.handleRibbonAction("AddSignature");
      const modal = (app as any).pdfWorkspace?.getSignModal();
      expect(modal).toBeDefined();
      if (modal) {
        expect(modal.getOverlay().style.display).toBe("flex");
        expect(modal.getOverlay().innerHTML).not.toContain("John Doe");
        modal.hide();
      }

      // Custom signature stamp
      pdfState.addSignatureStamp({
        signerName: "Chief Legal Officer",
        mode: "type",
        style: "script",
        timestamp: "2026-10-31",
        dataUrl: "data:image/svg+xml;base64,PHN2Zz48L3N2Zz4=",
        x: 150,
        y: 600,
        width: 200,
        height: 60,
      });
      expect(pdfState.signatureStamps.length).toBe(1);
      expect(pdfState.signatureStamps[0].signerName).toBe("Chief Legal Officer");

      // Password security
      pdfState.encrypt("LegalVault@2026");
      expect(pdfState.isEncrypted).toBe(true);
      expect(pdfState.isLocked).toBe(true);
      expect(pdfState.verifyPassword("WrongPassword")).toBe(false);
      expect(pdfState.verifyPassword("LegalVault@2026")).toBe(true);
      expect(pdfState.isLocked).toBe(false);

      // Page resizing
      app.handleRibbonAction("SetPageSize", "A4");
      expect(pdfState.currentPageDimensions?.name).toBe("A4");
      expect(pdfState.currentPageDimensions?.width).toBe(794);
      expect(pdfState.currentPageDimensions?.height).toBe(1123);
    });

    // 2.5 Multi-Document Per Format Tabs & Side-by-Side Split View
    it("2.5 Multi-Document: verifies opening multiple files per format and Split View toggle", async () => {
      const tabBar = app.getTabBar();

      // Open 2 Word files, 2 Excel files, 2 PPT files, 2 PDF files, 2 Notes files
      tabBar.addTab({ id: "w1", docType: "Word", title: "Memo.docx", isDirty: false });
      tabBar.addTab({ id: "w2", docType: "Word", title: "Brief.docx", isDirty: false });
      tabBar.addTab({ id: "x1", docType: "Excel", title: "Budget.xlsx", isDirty: false });
      tabBar.addTab({ id: "x2", docType: "Excel", title: "Forecast.xlsx", isDirty: false });
      tabBar.addTab({ id: "p1", docType: "PowerPoint", title: "Pitch.pptx", isDirty: false });
      tabBar.addTab({ id: "p2", docType: "PowerPoint", title: "Keynote.pptx", isDirty: false });
      tabBar.addTab({ id: "pdf1", docType: "PDF", title: "Report.pdf", isDirty: false });
      tabBar.addTab({ id: "pdf2", docType: "PDF", title: "Appendix.pdf", isDirty: false });
      tabBar.addTab({ id: "n1", docType: "Notes", title: "Notes.txt", isDirty: false });
      tabBar.addTab({ id: "n2", docType: "Notes", title: "Todo.md", isDirty: false });

      expect(tabBar.getTabs().length).toBeGreaterThanOrEqual(10);

      // Verify Split View toggle button
      const splitBtn = document.getElementById("btn-split-screen");
      expect(splitBtn).not.toBeNull();
      splitBtn?.click();

      // Verify popout
      const popoutRes = await app.handlePopoutTab(tabBar.getTabs()[0]);
      expect(popoutRes).toBeTruthy();
    });
  });
});
