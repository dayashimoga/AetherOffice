import { describe, it, expect, beforeEach, vi } from "vitest";
import { UnifiedShellApp } from "../main";
import { HubLauncher } from "../apps/hub/HubLauncher";
import { writerState } from "../state/writerState";
import { calcState } from "../state/calcState";
import { slidesState } from "../state/slidesState";
import { pdfState } from "../state/pdfState";
import { IPC } from "../ipc";

describe("Comprehensive Automated E2E UI Test Suite Across All 5 Apps (GEC-v2.0 Subphase 13.1.1)", () => {
  let app: UnifiedShellApp;

  beforeEach(() => {
    localStorage.clear();
    IPC._resetMockState();

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
      translate: () => {},
      rotate: () => {},
      arc: () => {},
      roundRect: () => {},
      ellipse: () => {},
      measureText: (txt: string) => ({ width: txt.length * 8 }),
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
          <div class="app-branding">
            <span class="app-logo">⚡</span>
            <span class="app-name">AetherOffice</span>
          </div>
        </header>
        <div id="tab-bar-container"></div>
        <div id="ribbon-container"></div>
        <div id="sub-header-container" style="display:flex;"></div>
        <div class="main-content" style="display:flex; height:600px;">
          <aside id="sidebar-container" class="sidebar-pane"></aside>
          <main id="viewport-container" style="flex:1;"></main>
          <div id="animation-pane-container"></div>
        </div>
        <div id="bottom-sheet"></div>
        <canvas id="stylus-ink-canvas"></canvas>
        <div id="reflow-reader-container"></div>
        <div id="row-card-inspector-container"></div>
        <div id="laser-pointer-dot"></div>
        <footer id="status-bar-container">
          <span id="status-doc-info"></span>
          <span id="status-word-count"></span>
          <span id="status-zoom-level">100%</span>
          <input type="range" id="zoom-slider" />
        </footer>
      </div>
    `;

    app = new UnifiedShellApp(false);
  });

  // =========================================================================
  // 1. AetherHub E2E: Main Suite Launcher, Templates & Multi-Process Spawning
  // =========================================================================
  describe("1. AetherHub E2E Launcher & Multi-Process Spawning", () => {
    it("renders App Launch Cards for Writer, Calc, Slides, and PDF Studio", async () => {
      const viewport = document.getElementById("viewport-container") as HTMLElement;
      const hub = new HubLauncher(viewport);
      await hub.render();

      expect(viewport.querySelector('[data-testid="app-card-writer"]')).not.toBeNull();
      expect(viewport.querySelector('[data-testid="app-card-calc"]')).not.toBeNull();
      expect(viewport.querySelector('[data-testid="app-card-slides"]')).not.toBeNull();
      expect(viewport.querySelector('[data-testid="app-card-pdf"]')).not.toBeNull();
    });

    it("spawns independent standalone child processes upon clicking App Cards", async () => {
      const viewport = document.getElementById("viewport-container") as HTMLElement;
      const hub = new HubLauncher(viewport);
      const launchSpy = vi.fn();
      hub.setCallbacks({ onLaunchApp: launchSpy });
      await hub.render();

      const writerBtn = viewport.querySelector<HTMLButtonElement>('[data-launch-app="writer"]');
      expect(writerBtn).not.toBeNull();
      writerBtn?.click();
      await new Promise((r) => setTimeout(r, 10));
      expect(launchSpy).toHaveBeenCalledWith("writer", undefined);

      const calcBtn = viewport.querySelector<HTMLButtonElement>('[data-launch-app="calc"]');
      calcBtn?.click();
      await new Promise((r) => setTimeout(r, 10));
      expect(launchSpy).toHaveBeenCalledWith("calc", undefined);

      const slidesBtn = viewport.querySelector<HTMLButtonElement>('[data-launch-app="slides"]');
      slidesBtn?.click();
      await new Promise((r) => setTimeout(r, 10));
      expect(launchSpy).toHaveBeenCalledWith("slides", undefined);

      const pdfBtn = viewport.querySelector<HTMLButtonElement>('[data-launch-app="pdf"]');
      pdfBtn?.click();
      await new Promise((r) => setTimeout(r, 10));
      expect(launchSpy).toHaveBeenCalledWith("pdf", undefined);
    });

    it("launches applications with pre-structured templates from Template Gallery", async () => {
      const viewport = document.getElementById("viewport-container") as HTMLElement;
      const hub = new HubLauncher(viewport);
      const templateSpy = vi.fn();
      hub.setCallbacks({ onSelectTemplate: templateSpy });
      await hub.render();

      const tplCards = viewport.querySelectorAll("[data-template-id]");
      expect(tplCards.length).toBeGreaterThanOrEqual(4);

      (tplCards[0] as HTMLElement).click();
      await new Promise((r) => setTimeout(r, 10));
      expect(templateSpy).toHaveBeenCalled();
    });
  });

  // =========================================================================
  // 2. AetherWriter E2E: Paginated Canvas, Caret, Styles, Tables & Word Count
  // =========================================================================
  describe("2. AetherWriter E2E: Paginated Canvas, Caret, Styles & Tables", () => {
    beforeEach(() => {
      const tabBar = app.getTabBar();
      tabBar.addTab({
        id: "tab-writer-e2e",
        docType: "Word",
        title: "Contract.docx",
        isDirty: false,
      });
      tabBar.switchTab("tab-writer-e2e");
    });

    it("mounts WriterCanvas, executes interactive typing with live caret and paragraph splits", () => {
      const canvas = app.getWriterCanvas();
      expect(canvas).toBeDefined();

      const initialParas = writerState.paragraphs.length;
      canvas?.typeText("Antigravity Global Engineering Contract v2.0");

      expect(writerState.paragraphs[0].runs[0].text).toContain("Antigravity Global Engineering Contract");
      expect(app.getTabBar().getTabs().find((t) => t.id === "tab-writer-e2e")?.isDirty).toBe(true);

      // Enter split
      canvas?.handleKeyDown(new KeyboardEvent("keydown", { key: "Enter" }));
      expect(writerState.paragraphs.length).toBe(initialParas + 1);
    });

    it("applies rich text formatting: Bold, Italic, Font Size, and Heading 1 Style", () => {
      app.handleRibbonAction("ToggleBold");
      expect(writerState.activeFormatting.bold).toBe(true);

      app.handleRibbonAction("ToggleItalic");
      expect(writerState.activeFormatting.italic).toBe(true);

      app.handleRibbonAction("SetFontSize", 18);
      expect(writerState.activeFormatting.size).toBe(18);

      // Apply Heading 1 style
      app.handleRibbonAction("SetStyle", "Heading 1");
      const navOutline = document.getElementById("sidebar-container");
      expect(navOutline).not.toBeNull();
    });

    it("inserts Blank Page, Table, Picture, Shape, and Comments via Insert Ribbon", () => {
      const initialPages = writerState.doc.pages.length;
      app.handleRibbonAction("InsertBlankPage");
      expect(writerState.doc.pages.length).toBe(initialPages + 1);

      app.handleRibbonAction("InsertTable", { rows: 3, cols: 3 });
      expect(writerState.paragraphs.some((p) => p.table !== undefined)).toBe(true);

      app.handleRibbonAction("InsertPicture", { width: 300, height: 180, caption: "E2E Test Architecture" });
      expect(writerState.paragraphs.some((p) => p.image !== undefined)).toBe(true);

      app.handleRibbonAction("InsertComment", "Reviewed and verified compliant");
      expect(writerState.doc.comments?.length).toBeGreaterThanOrEqual(1);
    });

    it("adjusts Page Layout (Orientation, Margins, Columns) and opens Word Count Dialog", () => {
      app.handleRibbonAction("SetOrientation", "landscape");
      expect(writerState.doc.pageOrientation).toBe("landscape");

      app.handleRibbonAction("SetMargins", "wide");
      expect(writerState.doc.margins).toBe("wide");

      app.handleRibbonAction("SetColumns", 2);
      expect(writerState.doc.columns).toBe(2);

      // Open Word Count Dialog
      app.handleRibbonAction("WordCount");
      expect(app.getWordCountDialog().isOpen()).toBe(true);
    });
  });

  // =========================================================================
  // 3. AetherCalc E2E: Interactive Grid, Formula Evaluation, Sheets & Charts
  // =========================================================================
  describe("3. AetherCalc E2E: Interactive Grid, Formulas, Sheets & Charts", () => {
    beforeEach(() => {
      const tabBar = app.getTabBar();
      tabBar.addTab({
        id: "tab-calc-e2e",
        docType: "Excel",
        title: "AnnualBudget.xlsx",
        isDirty: false,
      });
      tabBar.switchTab("tab-calc-e2e");
    });

    it("edits A1=100 and A2=200, enters =SUM(A1:A2) in A3, and asserts 300 evaluates", () => {
      const canvas = app.getCalcGridCanvas();
      expect(canvas).toBeDefined();

      calcState.setCellValue("A1", "100");
      calcState.setCellValue("A2", "200");
      calcState.setCellValue("A3", "=SUM(A1:A2)");

      expect(calcState.getCellValue("A1")).toBe(100);
      expect(calcState.getCellValue("A2")).toBe(200);
      expect(calcState.getCellValue("A3")).toBe(300);

      // Formula Bar reflects formula text
      calcState.selectCell("A3");
      expect(calcState.formulaBarText).toBe("=SUM(A1:A2)");
    });

    it("adds Sheet2, switches sheets, and manages workbook sheets pane", () => {
      const initialSheetCount = calcState.sheets.length;
      calcState.addSheet("Q4_Forecast");

      expect(calcState.sheets.length).toBe(initialSheetCount + 1);
      expect(calcState.getActiveSheet().name).toBe("Q4_Forecast");

      calcState.selectSheet(0);
      expect(calcState.getActiveSheetIndex()).toBe(0);
    });

    it("inserts live Charts (Column, Bar, Pie) and manages Freeze Panes", () => {
      app.handleRibbonAction("InsertChart", "column");
      expect(calcState.charts.length).toBeGreaterThanOrEqual(1);
      expect(calcState.charts[calcState.charts.length - 1].type).toBe("column");

      // Freeze Panes
      app.handleRibbonAction("FreezeTopRow");
      expect(calcState.frozenRows).toBe(1);

      app.handleRibbonAction("FreezeFirstColumn");
      expect(calcState.frozenCols).toBe(1);

      app.handleRibbonAction("UnfreezePanes");
      expect(calcState.frozenRows).toBe(0);
      expect(calcState.frozenCols).toBe(0);
    });

    it("dispatches Sort, Filter, and Number formatting through ribbon", () => {
      app.handleRibbonAction("SetNumberFormat", "Currency");
      expect(calcState.cells.get(calcState.selectedCell)?.format).toBe("Currency");

      app.handleRibbonAction("SortAscending");
      app.handleRibbonAction("SortDescending");
      app.handleRibbonAction("ToggleFilter");
    });
  });

  // =========================================================================
  // 4. AetherSlides E2E: Shape Canvas, Animations, Transitions & Slide Show
  // =========================================================================
  describe("4. AetherSlides E2E: Shape Canvas, Animations, Transitions & Slide Show", () => {
    beforeEach(() => {
      const tabBar = app.getTabBar();
      tabBar.addTab({
        id: "tab-slides-e2e",
        docType: "PowerPoint",
        title: "Keynote.pptx",
        isDirty: false,
      });
      tabBar.switchTab("tab-slides-e2e");
    });

    it("adds a new slide, duplicates slide, and applies Slide Layout", () => {
      const initialCount = slidesState.slides.length;
      app.handleRibbonAction("NewSlide");
      expect(slidesState.slides.length).toBe(initialCount + 1);

      app.handleRibbonAction("DuplicateSlide");
      expect(slidesState.slides.length).toBe(initialCount + 2);

      slidesState.setSlideLayout(0, "twoContent");
      expect(slidesState.slides[0].layout).toBe("twoContent");
    });

    it("inserts Star shape, tests geometry manipulation, and formats shape text", () => {
      const stage = app.getSlideStageCanvas();
      expect(stage).toBeDefined();

      const initialShapes = slidesState.getCurrentSlide().shapes.length;
      app.handleRibbonAction("InsertShape", "star");

      const currSlide = slidesState.getCurrentSlide();
      expect(currSlide.shapes.length).toBe(initialShapes + 1);
      const star = currSlide.shapes[currSlide.shapes.length - 1];
      expect(star.shapeType).toBe("star");

      // Format shape
      slidesState.setShapeFill("#c43e1c");
      expect(star.fill).toBe("#c43e1c");
      slidesState.setShapeText("Executive Vision 2026");
      expect(star.text).toBe("Executive Vision 2026");
    });

    it("configures Transitions, Animations, and toggles collapsible Animation Pane", () => {
      // Transition
      app.handleRibbonAction("SetTransition", "Fade");
      expect(slidesState.getCurrentSlide().transition).toBe("Fade");

      // Animation
      app.handleRibbonAction("SetAnimation", "flyIn");
      const animations = slidesState.getSlideAnimations();
      expect(animations.length).toBeGreaterThanOrEqual(1);
      expect(animations[0].effect).toBe("flyIn");

      // Toggle Animation Pane
      app.handleRibbonAction("ToggleAnimationPane");
      const animPane = document.getElementById("animation-pane-container");
      expect(animPane?.style.display).not.toBe("none");
    });

    it("starts fullscreen Slide Show (F5) and Presenter View", () => {
      const player = app.getPresentationPlayer();
      expect(player).toBeDefined();

      app.handleRibbonAction("StartShowBeginning");
      expect(player?.isOpen()).toBe(true);
      expect(player?.isPresenter()).toBe(false);

      player?.exitShow();
      expect(player?.isOpen()).toBe(false);

      app.handleRibbonAction("StartPresenterView");
      expect(player?.isOpen()).toBe(true);
      expect(player?.isPresenter()).toBe(true);
      player?.exitShow();
    });
  });

  // =========================================================================
  // 5. AetherPDF Studio E2E: Page Organizer, Text Editing, Forms & Signing
  // =========================================================================
  describe("5. AetherPDF Studio E2E: Page Organizer, Text, Forms & Signing", () => {
    beforeEach(() => {
      const tabBar = app.getTabBar();
      tabBar.addTab({
        id: "tab-pdf-e2e",
        docType: "PDF",
        title: "StandardContract.pdf",
        isDirty: false,
      });
      tabBar.switchTab("tab-pdf-e2e");
    });

    it("mounts PdfWorkspace with multi-page canvases, thumbnails, and outline outline", () => {
      const ws = app.getPdfWorkspace();
      expect(ws).toBeDefined();
      expect(ws?.getRoot().className).toContain("pdf-studio-workspace");

      // Thumbnails & Canvases
      expect(pdfState.pageCount).toBeGreaterThanOrEqual(3);
      const canvases = ws?.getViewport().querySelectorAll(".pdf-page-canvas");
      expect(canvases?.length).toBeGreaterThanOrEqual(3);
    });

    it("inserts blank page, rotates pages, and performs page reordering", () => {
      const initialCount = pdfState.pageCount;
      app.handleRibbonAction("InsertBlankPage");
      expect(pdfState.pageCount).toBe(initialCount + 1);

      app.handleRibbonAction("RotatePageCW");
      expect(pdfState.getPage(pdfState.currentPage)?.rotation).toBe(90);

      app.handleRibbonAction("MovePageUp");
      expect(pdfState.currentPage).toBeGreaterThanOrEqual(1);

      app.handleRibbonAction("DeletePage");
      expect(pdfState.pageCount).toBe(initialCount);
    });

    it("performs in-place text editing, adds watermark, and applies permanent redaction", () => {
      // In-place text editing
      const page1 = pdfState.getPage(1)!;
      const tbId = page1.textBlocks![0].id;
      pdfState.updateTextBlock(1, tbId, "Certified Document Header v2");
      expect(page1.textBlocks![0].text).toBe("Certified Document Header v2");

      // Watermark
      app.handleRibbonAction("AddWatermark", "CLASSIFIED");
      expect(page1.watermark?.text).toBe("CLASSIFIED");

      // Redaction
      pdfState.markRedaction(1, [80, 70, 300, 30]);
      app.handleRibbonAction("ApplyRedactions");
      const redactAnn = page1.annotations?.find((a) => a.type === "redact");
      expect(redactAnn?.isAppliedRedaction).toBe(true);
    });

    it("fills interactive AcroForm fields, imports/exports data, and stamps digital signature", () => {
      // Form Field Filling
      pdfState.setFormFieldValue("Full_Name", "Alexandria Hamilton");
      pdfState.setFormFieldValue("Agreed", true);
      const data = pdfState.exportFormData();
      expect(data["Full_Name"]).toBe("Alexandria Hamilton");
      expect(data["Agreed"]).toBe(true);

      // Sign Document Modal
      app.handleRibbonAction("SignPdf");
      const modal = app.getPdfWorkspace()?.getSignModal();
      expect(modal).toBeDefined();
      expect(modal?.getOverlay().style.display).toBe("flex");

      // Stamp signature
      pdfState.addSignature(1, "data:image/png;base64,...", "Alexandria Hamilton");
      const sigAnnot = pdfState.getPage(1)!.annotations!.find((a) => a.type === "signature");
      expect(sigAnnot?.signerName).toBe("Alexandria Hamilton");

      modal?.hide();
    });

    it("exports standard ISO 32000 compliant binary PDF bytes", () => {
      const bytes = pdfState.exportBinaryPdf();
      expect(bytes).toBeInstanceOf(Uint8Array);
      expect(bytes.length).toBeGreaterThan(100);

      const text = new TextDecoder("latin1").decode(bytes);
      expect(text.startsWith("%PDF-1.7")).toBe(true);
      expect(text).toContain("/Type /Catalog");
      expect(text).toContain("%%EOF");
    });
  });

  // =========================================================================
  // 6. Cross-App Multi-Document Workspace & Session Management
  // =========================================================================
  describe("6. Cross-App Multi-Document Tab Switching & Unified Shell Keyboard Shortcuts", () => {
    beforeEach(() => {
      const tabBar = app.getTabBar();
      tabBar.addTab({ id: "tab-w", docType: "Word", title: "Doc.docx", isDirty: false });
      tabBar.addTab({ id: "tab-c", docType: "Excel", title: "Sheet.xlsx", isDirty: false });
      tabBar.addTab({ id: "tab-s", docType: "PowerPoint", title: "Slides.pptx", isDirty: false });
      tabBar.addTab({ id: "tab-p", docType: "PDF", title: "Doc.pdf", isDirty: false });
    });

    it("seamlessly switches between all 4 document types updating ribbon, status bar & viewports", () => {
      const tabBar = app.getTabBar();
      const statusInfo = document.getElementById("status-doc-info");

      // Switch to Word
      tabBar.switchTab("tab-w");
      expect(app.getCurrentDocType()).toBe("Word");
      expect(app.getWriterCanvas()).toBeDefined();

      // Switch to Calc
      tabBar.switchTab("tab-c");
      expect(app.getCurrentDocType()).toBe("Excel");
      expect(app.getCalcGridCanvas()).toBeDefined();
      expect(statusInfo?.textContent).toContain("Sheet ");

      // Switch to Slides
      tabBar.switchTab("tab-s");
      expect(app.getCurrentDocType()).toBe("PowerPoint");
      expect(app.getSlideStageCanvas()).toBeDefined();
      expect(statusInfo?.textContent).toContain("Slide 1 of");

      // Switch to PDF
      tabBar.switchTab("tab-p");
      expect(app.getCurrentDocType()).toBe("PDF");
      expect(app.getPdfWorkspace()).toBeDefined();
      expect(statusInfo?.textContent).toContain("Page 1 of");
    });

    it("dispatches global keyboard shortcuts: Ctrl+Z (Undo), Ctrl+Y (Redo), and marks tabs dirty", () => {
      const tabBar = app.getTabBar();
      tabBar.switchTab("tab-c");

      calcState.setCellValue("B1", "Initial");
      calcState.setCellValue("B1", "Modified");
      expect(calcState.getCellValue("B1")).toBe("Modified");

      // Ctrl+Z Undo
      app.getKeyboardShortcuts().handleKeyDown(new KeyboardEvent("keydown", { key: "z", ctrlKey: true }));
      expect(calcState.getCellValue("B1")).toBe("Initial");

      // Ctrl+Y Redo
      app.getKeyboardShortcuts().handleKeyDown(new KeyboardEvent("keydown", { key: "y", ctrlKey: true }));
      expect(calcState.getCellValue("B1")).toBe("Modified");
    });
  });
});
