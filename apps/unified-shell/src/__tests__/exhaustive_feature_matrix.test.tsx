import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { UnifiedShellApp } from "../main";
import { writerState } from "../state/writerState";
import { calcState } from "../state/calcState";
import { slidesState } from "../state/slidesState";
import { pdfState } from "../state/pdfState";
import { notesState } from "../state/notesState";
import { themeManager } from "../components/ThemeManager";
import { IPC } from "../ipc";

describe("Exhaustive Automated Feature Matrix Gate (GEC-v4.0 Subphase 34.1.1)", () => {
  let app: UnifiedShellApp;

  beforeEach(() => {
    localStorage.clear();
    IPC._resetMockState();

    // 1. Mock 2D Canvas Context with full rendering support
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
    if (!HTMLCanvasElement.prototype.scrollIntoView) {
      HTMLCanvasElement.prototype.scrollIntoView = () => {};
    }

    document.body.className = "";
    document.documentElement.className = "";

    // 2. DOM Frame Setup
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

    // 3. Reset all module states to blank documents
    writerState.resetToBlank("Document1.docx");
    calcState.resetToBlank("Sheet1");
    slidesState.resetToBlank("Presentation1.pptx");
    pdfState.resetToBlank("Document1.pdf");
    notesState.resetToBlank("Scratchpad1.txt");

    app = new UnifiedShellApp(false);
  });

  afterEach(() => {
    document.body.innerHTML = "";
  });

  // =========================================================================
  // SUITE 1: GLOBAL SHELL, THEME CASCADE, GHOST SIDEBARS & MULTI-WINDOW HUB
  // =========================================================================
  describe("Suite 1: Global Shell, Theme Cascade, Zero Ghost Sidebars & Session Hub", () => {
    it("1.1 enforces Full-App Dark/Light Theme Cascade across all elements", () => {
      themeManager.init("#btn-theme-toggle");
      expect(themeManager.getTheme()).toBe("light");

      // Toggle to dark
      const next = themeManager.toggleTheme();
      expect(next).toBe("dark");
      expect(document.documentElement.classList.contains("theme-dark")).toBe(true);
      expect(document.body.classList.contains("theme-dark")).toBe(true);
      expect(document.getElementById("app")?.classList.contains("theme-dark")).toBe(true);

      // Toggle back to light
      const light = themeManager.toggleTheme();
      expect(light).toBe("light");
      expect(document.documentElement.classList.contains("theme-light")).toBe(true);
      expect(document.body.classList.contains("theme-light")).toBe(true);
      expect(document.getElementById("app")?.classList.contains("theme-light")).toBe(true);
    });

    it("1.2 prevents ghost Navigation sidebars across Excel, PowerPoint, PDF, and Notes (Article II.1)", () => {
      // Switch to Word and toggle nav pane ON
      app.getTabBar().addTab({ id: "tab-w", docType: "Word", title: "WordDoc.docx", isDirty: false });
      app.getTabBar().switchTab("tab-w");
      (app as any).showNavPane = true;
      app.updateSidebarForActiveApp();

      const sidebar = document.getElementById("sidebar-container")!;
      expect(sidebar.style.display).not.toBe("none");

      // Switch to Excel: sidebar MUST NOT show Word navigation
      app.getTabBar().addTab({ id: "tab-x", docType: "Excel", title: "Sheet.xlsx", isDirty: false });
      app.getTabBar().switchTab("tab-x");
      app.updateSidebarForActiveApp();
      expect(sidebar.textContent).not.toContain("Headings");
      expect(sidebar.textContent).not.toContain("Pages");

      // Switch to PowerPoint
      app.getTabBar().addTab({ id: "tab-p", docType: "PowerPoint", title: "Slides.pptx", isDirty: false });
      app.getTabBar().switchTab("tab-p");
      app.updateSidebarForActiveApp();
      expect(sidebar.textContent).not.toContain("Headings");

      // Switch to PDF
      app.getTabBar().addTab({ id: "tab-pdf", docType: "PDF", title: "Doc.pdf", isDirty: false });
      app.getTabBar().switchTab("tab-pdf");
      app.updateSidebarForActiveApp();
      expect(sidebar.textContent).not.toContain("Headings");

      // Switch to Notes
      app.getTabBar().addTab({ id: "tab-n", docType: "Notes", title: "Note.txt", isDirty: false });
      app.getTabBar().switchTab("tab-n");
      app.updateSidebarForActiveApp();
      expect(sidebar.textContent).not.toContain("Headings");
    });

    it("1.3 integrates Open Documents Session Hub drawer and multi-window popout", async () => {
      const drawer = app.getOpenDocumentsDrawer();
      expect(drawer).toBeDefined();

      const allFilesBtn = document.getElementById("btn-all-open-files");
      expect(allFilesBtn).not.toBeNull();
      allFilesBtn?.click();
      expect(drawer.getIsOpen()).toBe(true);

      allFilesBtn?.click();
      expect(drawer.getIsOpen()).toBe(false);

      // Verify popout
      const popoutRes = await app.handlePopoutTab({ id: "tab-pop", docType: "Word", title: "Test.docx", isDirty: false });
      expect(popoutRes).toBeTruthy();
    });

    it("1.4 dynamic status bar reflects active application metrics accurately (Article II.4)", () => {
      // Word metrics
      app.getTabBar().addTab({ id: "tab-w2", docType: "Word", title: "Doc.docx", isDirty: false });
      app.getTabBar().switchTab("tab-w2");
      writerState.resetToBlank("Doc.docx");
      writerState.insertText("Antigravity builds reliable office tools.");
      app.updateStatusBar();

      const docInfo = document.getElementById("status-doc-info")!;
      const wordCount = document.getElementById("status-word-count")!;
      const lang = document.getElementById("status-lang")!;
      expect(docInfo.textContent).toContain("Page 1 of 1");
      expect(wordCount.textContent).toContain("words");
      expect(lang.textContent).toBe("English (US)");

      // Excel metrics
      app.getTabBar().addTab({ id: "tab-x2", docType: "Excel", title: "Book.xlsx", isDirty: false });
      app.getTabBar().switchTab("tab-x2");
      calcState.selectedCell = "B4";
      app.updateStatusBar();
      expect(docInfo.textContent).toContain("Sheet 1 of 1");
      expect(wordCount.textContent).toContain("Selected: B4");

      // PowerPoint metrics
      app.getTabBar().addTab({ id: "tab-p2", docType: "PowerPoint", title: "Deck.pptx", isDirty: false });
      app.getTabBar().switchTab("tab-p2");
      slidesState.currentSlideIndex = 0;
      slidesState.setTheme("Emerald");
      app.updateStatusBar();
      expect(docInfo.textContent).toContain("Slide 1 of");
      expect(lang.textContent).toContain("Theme: Emerald");

      // PDF metrics
      app.getTabBar().addTab({ id: "tab-pdf2", docType: "PDF", title: "File.pdf", isDirty: false });
      app.getTabBar().switchTab("tab-pdf2");
      pdfState.createBlankPdf(3);
      pdfState.currentPage = 2;
      app.updateStatusBar();
      expect(docInfo.textContent).toBe("Page 2 of 3");

      // Notes metrics
      app.getTabBar().addTab({ id: "tab-n2", docType: "Notes", title: "Note.txt", isDirty: false });
      app.getTabBar().switchTab("tab-n2");
      notesState.setContent("Line one\nLine two with five words");
      notesState.setCursor(2, 5);
      app.updateStatusBar();
      expect(docInfo.textContent).toBe("Ln 2, Col 5");
      expect(wordCount.textContent).toContain("2 lines | 7 words");
      expect(lang.textContent).toBe("UTF-8");
    });
  });

  // =========================================================================
  // SUITE 2: WORD (.DOCX) - FORMATTING, TABLES, STYLES, TOC & PAGE LAYOUT
  // =========================================================================
  describe("Suite 2: Word (.docx) Feature Matrix", () => {
    beforeEach(() => {
      app.getTabBar().addTab({ id: "tab-word-test", docType: "Word", title: "Article.docx", isDirty: false });
      app.getTabBar().switchTab("tab-word-test");
      writerState.resetToBlank("Article.docx");
    });

    it("2.1 handles Document Typing, Pages, Typography, and Formatting Toggles", () => {
      writerState.insertText("Modern office productivity suite in Rust.");
      expect(writerState.getWordCount()).toBeGreaterThan(0);

      // Add Page
      app.handleRibbonAction("InsertNewPage");
      expect(writerState.doc.pages.length).toBeGreaterThanOrEqual(2);

      // Font Family & Size
      app.handleRibbonAction("SetFontFamily", "Georgia");
      expect(writerState.activeFormat.font).toBe("Georgia");

      app.handleRibbonAction("SetFontSize", 16);
      expect(writerState.activeFormat.size).toBe(16);

      // Formatting toggles
      app.handleRibbonAction("ToggleBold");
      expect(writerState.activeFormat.bold).toBe(true);

      app.handleRibbonAction("ToggleItalic");
      expect(writerState.activeFormat.italic).toBe(true);

      app.handleRibbonAction("ToggleUnderline");
      expect(writerState.activeFormat.underline).toBe(true);

      // Alignment
      app.handleRibbonAction("AlignCenter");
      expect(writerState.activeFormat.alignment).toBe("center");
    });

    it("2.2 handles Tables: Insert Table, Add/Delete Rows and Columns", () => {
      app.handleRibbonAction("InsertTable", { rows: 3, cols: 3 });
      const tablePara = writerState.paragraphs.find((p) => p.table);
      expect(tablePara).toBeDefined();
      expect(tablePara?.table?.rows.length).toBe(3);
      expect(tablePara?.table?.colCount).toBe(3);

      // Add Table Row
      writerState.cursor.paragraphIndex = 1;
      app.handleRibbonAction("AddTableRow");
      expect(tablePara?.table?.rows.length).toBe(4);

      // Add Table Column
      app.handleRibbonAction("AddTableColumn");
      expect(tablePara?.table?.colCount).toBe(4);

      // Delete Table Row
      app.handleRibbonAction("DeleteTableRow");
      expect(tablePara?.table?.rows.length).toBe(3);

      // Delete Table Column
      app.handleRibbonAction("DeleteTableColumn");
      expect(tablePara?.table?.colCount).toBe(3);
    });

    it("2.3 handles Headings, TOC, Page Layout, Shapes & Track Changes", () => {
      // Style / Heading
      app.handleRibbonAction("SetStyle", "Heading 1");
      const currentPara = writerState.paragraphs[0];
      expect(currentPara.style).toBe("Heading 1");

      // Resizable Shape
      app.handleRibbonAction("InsertShape", "roundRect");
      const shapePara = writerState.paragraphs.find((p) => p.shape);
      expect(shapePara).toBeDefined();
      expect(shapePara?.shape?.shapeType).toBe("roundRect");

      // TOC
      app.handleRibbonAction("InsertTOC");
      const tocPara = writerState.paragraphs.find((p) => p.id.startsWith("toc-"));
      expect(tocPara).toBeDefined();

      // Page Layout: Orientation & Margins
      app.handleRibbonAction("SetOrientation", "landscape");
      expect(writerState.doc.pageOrientation).toBe("landscape");

      app.handleRibbonAction("SetMargins", "narrow");
      expect(writerState.doc.margins).toBe("narrow");

      // Footnote
      app.handleRibbonAction("InsertFootnote", "Citation footnote note");
      expect(writerState.footnotes.length).toBeGreaterThan(0);

      // Track Changes
      app.handleRibbonAction("ToggleTrackChanges");
      expect(writerState.doc.trackChanges).toBe(true);

      // Comments
      app.handleRibbonAction("InsertComment", "Review note for chapter 1");
      expect(writerState.doc.comments?.length).toBeGreaterThan(0);

      // Save Document
      app.handleRibbonAction("SaveDocument");
      expect(document.getElementById("status-doc-info")?.textContent).toContain("Saved");
    });
  });

  // =========================================================================
  // SUITE 3: EXCEL (.XLSX) - FORMULAS, SHEETS, FORMATTING, DATA & VIEW
  // =========================================================================
  describe("Suite 3: Excel (.xlsx) Feature Matrix", () => {
    beforeEach(() => {
      app.getTabBar().addTab({ id: "tab-calc-test", docType: "Excel", title: "Spreadsheet.xlsx", isDirty: false });
      app.getTabBar().switchTab("tab-calc-test");
      calcState.resetToBlank("Sheet1");
    });

    it("3.1 handles Formula calculation, Cell evaluation, and Multi-Sheet", () => {
      calcState.setCellValue("A1", "100");
      calcState.setCellValue("A2", "200");
      calcState.setCellValue("A3", "=SUM(A1:A2)");
      expect(calcState.getCellValue("A3")).toBe(300);

      calcState.setCellValue("A4", "=AVERAGE(A1:A2)");
      expect(calcState.getCellValue("A4")).toBe(150);

      calcState.setCellValue("A5", '=IF(A1>50, "High", "Low")');
      expect(calcState.getCellValue("A5")).toBe("High");

      // Add Sheet2 & Switch
      const sheet2 = calcState.addSheet("Sheet2");
      expect(calcState.sheets.length).toBe(2);
      expect(sheet2.name).toBe("Sheet2");

      calcState.switchSheet("Sheet2");
      expect(calcState.getActiveSheet().name).toBe("Sheet2");
    });

    it("3.2 handles Number Formatting, Borders, and Colors", () => {
      calcState.selectCell("B2");
      app.handleRibbonAction("SetNumberFormat", "Currency");
      expect(calcState.getCell("B2").format).toBe("Currency");

      app.handleRibbonAction("SetBorders", "all");
      expect(calcState.getCell("B2").borders).toBe("all");

      app.handleRibbonAction("SetFillColor", "#dcfce7");
      expect(calcState.getCell("B2").bgColor).toBe("#dcfce7");

      app.handleRibbonAction("SetFontColor", "#15803d");
      expect(calcState.getCell("B2").color).toBe("#15803d");
    });

    it("3.3 handles Sorting, Filtering, Charts, Goal Seek, and Freeze Panes", () => {
      // Sorting
      app.handleRibbonAction("SortAsc");
      app.handleRibbonAction("SortDesc");

      const activeSheet = calcState.getActiveSheet();
      expect(activeSheet).toBeDefined();
      if (!activeSheet) return;

      // Filter
      app.handleRibbonAction("ToggleFilter");
      expect(activeSheet.filterActive).toBe(true);

      // Insert Chart
      app.handleRibbonAction("InsertChart", "column");
      expect(activeSheet.charts?.length ?? 0).toBeGreaterThan(0);

      // Goal Seek
      calcState.setCellValue("B1", "50");
      calcState.setCellValue("B2", "=B1*2");
      const goalRes = calcState.goalSeek("B2", 150, "B1");
      expect(goalRes.success).toBe(true);
      expect(Math.round(Number(calcState.getCellValue("B1")))).toBe(75);

      // Gridlines & Headings
      const gridBefore = activeSheet.showGridlines;
      app.handleRibbonAction("ToggleGridlines");
      expect(activeSheet.showGridlines).not.toBe(gridBefore);

      // Freeze Panes
      app.handleRibbonAction("FreezeTopRow");
      expect(activeSheet.freezePanes?.row).toBe(1);

      app.handleRibbonAction("FreezeFirstColumn");
      expect(activeSheet.freezePanes?.col).toBe(1);

      app.handleRibbonAction("UnfreezePanes");
      expect(activeSheet.freezePanes?.row).toBe(0);
      expect(activeSheet.freezePanes?.col).toBe(0);

      // Save Document
      app.handleRibbonAction("SaveDocument");
      expect(document.getElementById("status-doc-info")?.textContent).toContain("Saved");
    });
  });

  // =========================================================================
  // SUITE 4: POWERPOINT (.PPTX) - SLIDES, SHAPES, THEMES, ANIMATIONS, SHOW
  // =========================================================================
  describe("Suite 4: PowerPoint (.pptx) Feature Matrix", () => {
    beforeEach(() => {
      app.getTabBar().addTab({ id: "tab-slides-test", docType: "PowerPoint", title: "Deck.pptx", isDirty: false });
      app.getTabBar().switchTab("tab-slides-test");
      slidesState.resetToBlank("Deck.pptx");
    });

    it("4.1 manages Slides: Insert Slide, Duplicate, Delete, Hide, Inline Edit", () => {
      const initialSlides = slidesState.slides.length;

      // Add Slide
      app.handleRibbonAction("NewSlide");
      expect(slidesState.slides.length).toBe(initialSlides + 1);

      // Duplicate Slide
      app.handleRibbonAction("DuplicateSlide");
      expect(slidesState.slides.length).toBe(initialSlides + 2);

      // Inline Edit Title
      slidesState.getCurrentSlide().title = "Roadmap Overview";
      expect(slidesState.getCurrentSlide().title).toBe("Roadmap Overview");

      // Hide Slide
      app.handleRibbonAction("ToggleHideSlide");
      expect(slidesState.getCurrentSlide().isHidden).toBe(true);

      // Unhide
      app.handleRibbonAction("ToggleHideSlide");
      expect(slidesState.getCurrentSlide().isHidden).toBe(false);

      // Delete Slide
      app.handleRibbonAction("DeleteSlide");
      expect(slidesState.slides.length).toBe(initialSlides + 1);
    });

    it("4.2 manages Shapes, Themes, Transitions, Animations, and Fullscreen Slide Show", () => {
      // Insert Shape
      app.handleRibbonAction("InsertShape", "rect");
      const currentSlide = slidesState.getCurrentSlide();
      expect(currentSlide.shapes?.length).toBeGreaterThan(0);

      // Change Design Theme & Size
      app.handleRibbonAction("SetTheme", "Slate");
      expect(slidesState.currentTheme).toBe("Slate");

      app.handleRibbonAction("SetSlideSize", "4:3");
      expect(slidesState.slideAspectRatio).toBe("4:3");

      // Transitions & Animations
      app.handleRibbonAction("SetTransition", "fade");
      expect(slidesState.getCurrentSlide().transition).toBe("Fade");

      const shapeId = currentSlide.shapes![0].id;
      slidesState.selectShape(shapeId);
      app.handleRibbonAction("AddAnimation", "fadeIn");
      expect(slidesState.getCurrentSlide().animations?.length).toBeGreaterThan(0);

      // Slide Show
      app.handleRibbonAction("StartShowBeginning");
      expect(slidesState.isShowActive).toBe(true);

      app.handleRibbonAction("StepShowForward");
      app.handleRibbonAction("ExitShow");
      expect(slidesState.isShowActive).toBe(false);

      // Save Document
      app.handleRibbonAction("SaveDocument");
      expect(document.getElementById("status-doc-info")?.textContent).toContain("Saved");
    });
  });

  // =========================================================================
  // SUITE 5: PDF STUDIO (.PDF) - PAGES, RESIZE, SECURITY, ANNOTATIONS, FORMS
  // =========================================================================
  describe("Suite 5: PDF Studio (.pdf) Feature Matrix", () => {
    beforeEach(() => {
      app.getTabBar().addTab({ id: "tab-pdf-test", docType: "PDF", title: "Document.pdf", isDirty: false });
      app.getTabBar().switchTab("tab-pdf-test");
      pdfState.resetToBlank("Document.pdf");
    });

    it("5.1 organizes Pages: Blank PDF, Insert, Rotate CW/CCW, Delete, Move", () => {
      app.handleRibbonAction("CreateBlankPdf");
      expect(pdfState.pageCount).toBe(1);

      // Insert Page
      app.handleRibbonAction("InsertBlankPage");
      expect(pdfState.pageCount).toBe(2);

      // Rotate CW
      const curPage = pdfState.getPage(pdfState.currentPage);
      const rotBefore = curPage?.rotation || 0;
      app.handleRibbonAction("RotatePageCW");
      expect(curPage?.rotation).toBe((rotBefore + 90) % 360);

      // Rotate CCW
      app.handleRibbonAction("RotatePageCCW");
      expect(curPage?.rotation).toBe(rotBefore);

      // Move Page Down & Up
      app.handleRibbonAction("MovePageDown");
      app.handleRibbonAction("MovePageUp");

      // Delete Page
      app.handleRibbonAction("DeletePage");
      expect(pdfState.pageCount).toBe(1);
    });

    it("5.2 resizes Page Dimensions and manages Password Security & Encryption", () => {
      // Resize to Letter
      app.handleRibbonAction("SetPageSize", "Letter");
      expect(pdfState.pages[0].width).toBe(816);
      expect(pdfState.pages[0].height).toBe(1056);

      // Password Encryption
      app.handleRibbonAction("EncryptPassword", { openPassword: "secret-key-123" });
      expect(pdfState.isEncrypted).toBe(true);

      // Decryption / Remove Security
      app.handleRibbonAction("DecryptPassword", "secret-key-123");
      expect(pdfState.isEncrypted).toBe(false);

      // Compression
      app.handleRibbonAction("CompressPdf");
      expect(pdfState.isCompressed).toBe(true);
    });

    it("5.3 adds Annotations, Redactions, Form Fields, and Digital Signature", () => {
      // In-place text editing
      app.handleRibbonAction("AddPdfText");
      expect(pdfState.pages[0].textBlocks?.length).toBeGreaterThan(0);

      // Annotations: Tool selection & Highlight
      app.handleRibbonAction("SetAnnotTool", "note");
      expect(pdfState.activeTool).toBe("note");

      pdfState.addAnnotation({ type: "highlight", pageNumber: 1, rect: [100, 100, 150, 20], color: "#fef08a" });
      expect(pdfState.pages[0].annotations?.some((a) => a.type === "highlight")).toBe(true);

      // Redaction
      app.handleRibbonAction("MarkRedaction");
      expect(pdfState.pages[0].annotations?.some((a) => a.type === "redact")).toBe(true);

      app.handleRibbonAction("ApplyRedactions");

      // Form Field: Checkbox
      app.handleRibbonAction("AddFormField", "checkbox");
      expect(pdfState.pages[0].formFields?.some((f) => f.type === "checkbox")).toBe(true);

      // Digital Signature / Initials Stamp
      app.handleRibbonAction("AddInitials");
      expect(pdfState.pages[0].annotations?.some((a) => a.type === "signature")).toBe(true);

      // Save PDF
      app.handleRibbonAction("SaveDocument");
      expect(document.getElementById("status-doc-info")?.textContent).toContain("Saved");

      // Export PDF
      app.handleRibbonAction("ExportPdf");
      expect(document.getElementById("status-doc-info")?.textContent).toContain("Exported");
    });
  });

  // =========================================================================
  // SUITE 6: NOTEPAD / NOTES (.TXT/.MD) - EDITING, TOOLS, FIND & MARKDOWN
  // =========================================================================
  describe("Suite 6: Notepad / Notes (.txt/.md) Feature Matrix", () => {
    beforeEach(() => {
      app.getTabBar().addTab({ id: "tab-notes-test", docType: "Notes", title: "Note.txt", isDirty: false });
      app.getTabBar().switchTab("tab-notes-test");
      notesState.resetToBlank("Note.txt");
    });

    it("6.1 manages Scratchpads, Text Power Tools, and View Modes", () => {
      // New Scratchpad
      const initialNotes = notesState.notes.length;
      app.handleRibbonAction("NewNote");
      expect(notesState.notes.length).toBe(initialNotes + 1);

      // Text Power Tools: Upper, Lower, Sort A-Z, De-dup, Trim
      notesState.setContent("cherry  \napple  \ncherry  \nbanana  ");

      app.handleRibbonAction("TransformText", "upper");
      expect(notesState.content).toContain("CHERRY");

      app.handleRibbonAction("TransformText", "sort_az");
      expect(notesState.content.startsWith("APPLE")).toBe(true);

      app.handleRibbonAction("TransformText", "remove_duplicates");
      expect(notesState.content.split("\n").length).toBe(3);

      app.handleRibbonAction("TransformText", "trim_whitespace");
      expect(notesState.content.endsWith(" ")).toBe(false);

      // View Modes: Classic, Split, Preview
      app.handleRibbonAction("SetNotesViewMode", "split");
      expect(notesState.viewMode).toBe("split");

      app.handleRibbonAction("SetNotesViewMode", "preview");
      expect(notesState.viewMode).toBe("preview");

      app.handleRibbonAction("SetNotesViewMode", "editor");
      expect(notesState.viewMode).toBe("editor");
    });

    it("6.2 inserts Markdown syntax, executes Find & Replace, and Saves .md/.txt", () => {
      notesState.setContent("");

      // Markdown Syntax: H1, Bold, Code, Checklist, Table, Link, HR
      app.handleRibbonAction("InsertMarkdownSnippet", "h1");
      expect(notesState.content).toContain("# Heading 1");

      app.handleRibbonAction("InsertMarkdownSnippet", "bold");
      expect(notesState.content).toContain("**bold text**");

      app.handleRibbonAction("InsertMarkdownSnippet", "code");
      expect(notesState.content).toContain("```ts");

      app.handleRibbonAction("InsertMarkdownSnippet", "checklist");
      expect(notesState.content).toContain("- [ ] Task item");

      app.handleRibbonAction("InsertMarkdownSnippet", "table");
      expect(notesState.content).toContain("| Item | Description | Status |");

      app.handleRibbonAction("InsertMarkdownSnippet", "link");
      expect(notesState.content).toContain("[Link](https://example.com)");

      app.handleRibbonAction("InsertMarkdownSnippet", "hr");
      expect(notesState.content).toContain("---");

      // Find & Replace
      notesState.setContent("Antigravity builds office applications.");
      app.handleRibbonAction("ReplaceText", { find: "office", replace: "modern office" });
      expect(notesState.content).toBe("Antigravity builds modern office applications.");

      // Save as Markdown
      app.handleRibbonAction("SaveDocumentMd");
      expect(notesState.title.endsWith(".md")).toBe(true);

      // Save as TXT
      app.handleRibbonAction("SaveDocument");
      expect(document.getElementById("status-doc-info")?.textContent).toContain("Saved");
    });
  });
});
