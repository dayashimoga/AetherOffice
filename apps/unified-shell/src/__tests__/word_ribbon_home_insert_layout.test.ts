import { describe, it, expect, beforeEach, vi } from "vitest";
import { UnifiedShellApp } from "../main";
import { writerState } from "../state/writerState";
import { IPC } from "../ipc";

describe("Subphase 29.1.2: Fix Word Home, Insert & Layout Ribbons (100% Working Controls)", () => {
  let app: UnifiedShellApp;

  beforeEach(() => {
    localStorage.clear();
    IPC._resetMockState();
    writerState.resetToBlank();

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
    app.applyBootConfig({ app_mode: "writer" });
  });

  describe("1. Home Tab: 100% Working Controls & Selection Preservation", () => {
    it("preserves focus and editor selection when pressing mouse down on ribbon buttons", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Home");

      const boldBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-bold"]')!;
      expect(boldBtn).not.toBeNull();

      const mousedownEvent = new MouseEvent("mousedown", { cancelable: true });
      boldBtn.dispatchEvent(mousedownEvent);
      expect(mousedownEvent.defaultPrevented).toBe(true);
    });

    it("wires Font Family and Font Size selectors", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Home");

      const fontSelect = document.querySelector<HTMLSelectElement>('[data-testid="font-family-select"]')!;
      expect(fontSelect).not.toBeNull();
      fontSelect.value = "Georgia";
      fontSelect.dispatchEvent(new Event("change"));
      expect(writerState.activeFormat.font).toBe("Georgia");

      const sizeSelect = document.querySelector<HTMLSelectElement>('[data-testid="font-size-select"]')!;
      expect(sizeSelect).not.toBeNull();
      sizeSelect.value = "18";
      sizeSelect.dispatchEvent(new Event("change"));
      expect(writerState.activeFormat.size).toBe(18);
    });

    it("wires Grow Font, Shrink Font, Change Case, and Clear Formatting", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Home");

      writerState.activeFormat.size = 12;
      const incBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-increase-font"]')!;
      incBtn.click();
      expect(writerState.activeFormat.size).toBe(14);

      const decBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-decrease-font"]')!;
      decBtn.click();
      expect(writerState.activeFormat.size).toBe(12);

      // Change Case
      writerState.insertText("sample sentence");
      const caseBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-change-case"]')!;
      caseBtn.click();
      expect(writerState.getTextContent()).toContain("SAMPLE SENTENCE");

      // Clear Formatting
      writerState.activeFormat.bold = true;
      writerState.activeFormat.color = "#ff0000";
      const clearBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-clear-formatting"]')!;
      clearBtn.click();
      expect(writerState.activeFormat.bold).toBe(false);
      expect(writerState.activeFormat.color).toBe("#242424");
    });

    it("wires Bold, Italic, Underline, Strikethrough, Subscript, Superscript, and Colors", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Home");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-bold"]')!.click();
      expect(writerState.activeFormat.bold).toBe(true);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-italic"]')!.click();
      expect(writerState.activeFormat.italic).toBe(true);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-underline"]')!.click();
      expect(writerState.activeFormat.underline).toBe(true);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-strikethrough"]')!.click();
      expect(writerState.activeFormat.strikethrough).toBe(true);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-subscript"]')!.click();
      expect(writerState.doc.pages[0].paragraphs[0].runs[0].subscript).toBe(true);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-superscript"]')!.click();
      expect(writerState.doc.pages[0].paragraphs[0].runs[0].superscript).toBe(true);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-font-color"]')!.click();
      expect(writerState.activeFormat.color).toBe("#2563eb");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-highlight-color"]')!.click();
      expect(writerState.doc.pages[0].paragraphs[0].runs[0].highlight).toBe("#fef08a");
    });

    it("wires Bullets, Numbering, Multilevel, Indent, Sort, and Formatting Marks", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Home");

      writerState.doc.pages[0].paragraphs[0].runs[0].text = "Item Text";

      document.querySelector<HTMLButtonElement>('[data-testid="btn-bullet-list"]')!.click();
      expect(writerState.doc.pages[0].paragraphs[0].runs[0].text).toBe("• Item Text");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-numbered-list"]')!.click();
      expect(writerState.doc.pages[0].paragraphs[0].runs[0].text).toBe("1. • Item Text");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-multilevel-list"]')!.click();
      expect(writerState.doc.pages[0].paragraphs[0].runs[0].text).toBe("1.1 1. • Item Text");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-increase-indent"]')!.click();
      expect(writerState.doc.pages[0].paragraphs[0].indent).toBe(36);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-decrease-indent"]')!.click();
      expect(writerState.doc.pages[0].paragraphs[0].indent).toBe(0);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-show-formatting"]')!.click();
      expect(writerState.doc.showFormattingMarks).toBe(true);

      // Sort
      writerState.loadDocumentFromText("Zebra\nApple\nMango");
      document.querySelector<HTMLButtonElement>('[data-testid="btn-sort-paragraphs"]')!.click();
      const paras = writerState.doc.pages[0].paragraphs.map((p) => p.runs[0].text);
      expect(paras).toEqual(["Apple", "Mango", "Zebra"]);
    });

    it("wires Alignments, Line Spacing, Shading, Borders, and Styles", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Home");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-align-center"]')!.click();
      expect(writerState.activeFormat.alignment).toBe("center");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-align-right"]')!.click();
      expect(writerState.activeFormat.alignment).toBe("right");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-align-justify"]')!.click();
      expect(writerState.activeFormat.alignment).toBe("justify");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-align-left"]')!.click();
      expect(writerState.activeFormat.alignment).toBe("left");

      const spacingSelect = document.querySelector<HTMLSelectElement>('[data-testid="line-spacing-select"]')!;
      spacingSelect.value = "2.0";
      spacingSelect.dispatchEvent(new Event("change"));
      expect(writerState.activeFormat.lineSpacing).toBe(2.0);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-shading"]')!.click();
      expect(writerState.doc.pages[0].paragraphs[0].shading).toBe("#f3f4f6");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-paragraph-border"]')!.click();
      expect(writerState.doc.pages[0].paragraphs[0].border).toBe("1px solid #94a3b8");

      // Styles
      document.querySelector<HTMLButtonElement>('[data-testid="btn-style-h1"]')!.click();
      expect(writerState.doc.pages[0].paragraphs[0].style).toBe("Heading 1");
      expect(writerState.doc.pages[0].paragraphs[0].runs[0].bold).toBe(true);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-style-title"]')!.click();
      expect(writerState.doc.pages[0].paragraphs[0].style).toBe("Title");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-style-quote"]')!.click();
      expect(writerState.doc.pages[0].paragraphs[0].style).toBe("Quote");
    });
  });

  describe("2. Insert Tab: 100% Working Controls", () => {
    it("wires Cover Page, Blank Page, Page Break, and Delete Page", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Insert");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-insert-cover-page"]')!.click();
      expect(writerState.doc.pages.length).toBe(2);
      expect(writerState.doc.pages[0].paragraphs[0].style).toBe("Title");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-insert-blank-page"]')!.click();
      expect(writerState.doc.pages.length).toBe(3);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-insert-page-break"]')!.click();
      expect(writerState.doc.pages.length).toBe(4);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-delete-empty-page"]')!.click();
      expect(writerState.doc.pages.length).toBe(3);
    });

    it("wires Table Insertion, Add Row, Add Col, Del Row, and Del Col", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Insert");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-insert-table"]')!.click();
      const tblPara = writerState.doc.pages[0].paragraphs.find((p) => p.table);
      expect(tblPara).not.toBeUndefined();
      expect(tblPara!.table!.rows.length).toBe(3);
      expect(tblPara!.table!.colCount).toBe(3);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-add-table-row"]')!.click();
      expect(tblPara!.table!.rows.length).toBe(4);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-add-table-col"]')!.click();
      expect(tblPara!.table!.colCount).toBe(4);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-delete-table-row"]')!.click();
      expect(tblPara!.table!.rows.length).toBe(3);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-delete-table-col"]')!.click();
      expect(tblPara!.table!.colCount).toBe(3);
    });

    it("wires Picture, All Shapes (rect, roundRect, ellipse, triangle, arrow, star, callout), and Chart", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Insert");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-insert-picture"]')!.click();
      expect(writerState.doc.pages[0].paragraphs.some((p) => p.image)).toBe(true);

      const shapeTestIds = [
        "btn-shape-rect",
        "btn-shape-round",
        "btn-shape-ellipse",
        "btn-shape-triangle",
        "btn-shape-arrow",
        "btn-shape-star",
        "btn-shape-callout",
      ];
      shapeTestIds.forEach((id) => {
        document.querySelector<HTMLButtonElement>(`[data-testid="${id}"]`)!.click();
      });

      const shapes = writerState.doc.pages[0].paragraphs.filter((p) => p.shape);
      expect(shapes.length).toBe(7);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-insert-chart"]')!.click();
      expect(writerState.doc.pages[0].paragraphs.some((p) => p.shape?.text?.includes("Chart"))).toBe(true);
    });

    it("wires Header, Footer, Page Number, Text Box, WordArt, Drop Cap, Line, Date, and Symbol", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Insert");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-insert-header"]')!.click();
      expect(writerState.doc.headerText).toBe("Document Header");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-insert-footer"]')!.click();
      expect(writerState.doc.footerText).toBe("Document Footer");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-insert-page-number"]')!.click();
      expect(writerState.doc.footerText).toContain("Page");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-insert-textbox"]')!.click();
      expect(writerState.doc.pages[0].paragraphs.some((p) => p.shape?.text === "Text Box Content")).toBe(true);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-insert-wordart"]')!.click();
      expect(writerState.doc.pages[0].paragraphs.some((p) => p.runs[0].font === "Impact")).toBe(true);

      writerState.cursor.paragraphIndex = 0;
      writerState.doc.pages[0].paragraphs[0].runs[0].text = "Hello World";
      document.querySelector<HTMLButtonElement>('[data-testid="btn-insert-dropcap"]')!.click();
      expect(writerState.doc.pages[0].paragraphs[0].runs[0].text).toBe("H");
      expect(writerState.doc.pages[0].paragraphs[0].runs[0].size).toBe(32);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-insert-line"]')!.click();
      expect(writerState.doc.pages[0].paragraphs.some((p) => p.shape?.height === 2)).toBe(true);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-insert-symbol"]')!.click();
      expect(writerState.getTextContent()).toContain("©");
    });
  });

  describe("3. Layout Tab: 100% Working Controls", () => {
    it("wires Margins (Normal, Narrow, Moderate, Wide)", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Layout");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-margins-narrow"]')!.click();
      expect(writerState.doc.margins).toBe("narrow");
      expect(app.getWriterCanvas()?.marginX).toBe(48);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-margins-moderate"]')!.click();
      expect(writerState.doc.margins).toBe("moderate");
      expect(app.getWriterCanvas()?.marginX).toBe(72);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-margins-wide"]')!.click();
      expect(writerState.doc.margins).toBe("wide");
      expect(app.getWriterCanvas()?.marginX).toBe(144);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-margins-normal"]')!.click();
      expect(writerState.doc.margins).toBe("normal");
      expect(app.getWriterCanvas()?.marginX).toBe(96);
    });

    it("visibly swaps page dimensions between Portrait (816x1056) and Landscape (1056x816)", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Layout");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-orientation-landscape"]')!.click();
      expect(writerState.doc.pageOrientation).toBe("landscape");
      expect(app.getWriterCanvas()?.pageWidth).toBe(1056);
      expect(app.getWriterCanvas()?.pageHeight).toBe(816);

      const pageEl = document.querySelector<HTMLElement>(".word-page")!;
      expect(pageEl.style.width).toBe("1056px");
      expect(pageEl.style.height).toBe("816px");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-orientation-portrait"]')!.click();
      expect(writerState.doc.pageOrientation).toBe("portrait");
      expect(app.getWriterCanvas()?.pageWidth).toBe(816);
      expect(app.getWriterCanvas()?.pageHeight).toBe(1056);
    });

    it("wires Page Sizes (Letter, A4, Legal)", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Layout");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-pagesize-a4"]')!.click();
      expect(writerState.doc.pageSize).toBe("a4");
      expect(app.getWriterCanvas()?.pageWidth).toBe(794);
      expect(app.getWriterCanvas()?.pageHeight).toBe(1123);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-pagesize-legal"]')!.click();
      expect(writerState.doc.pageSize).toBe("legal");
      expect(app.getWriterCanvas()?.pageWidth).toBe(816);
      expect(app.getWriterCanvas()?.pageHeight).toBe(1296);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-pagesize-letter"]')!.click();
      expect(writerState.doc.pageSize).toBe("letter");
      expect(app.getWriterCanvas()?.pageWidth).toBe(816);
      expect(app.getWriterCanvas()?.pageHeight).toBe(1056);
    });

    it("applies Multi-Column CSS layout (One, Two, Three) to the page body", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Layout");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-cols-2"]')!.click();
      expect(writerState.doc.columns).toBe(2);
      let bodyEl = document.querySelector<HTMLElement>(".word-page-body")!;
      expect(bodyEl.style.columnCount).toBe("2");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-cols-3"]')!.click();
      expect(writerState.doc.columns).toBe(3);
      bodyEl = document.querySelector<HTMLElement>(".word-page-body")!;
      expect(bodyEl.style.columnCount).toBe("3");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-cols-1"]')!.click();
      expect(writerState.doc.columns).toBe(1);
    });

    it("wires Line Numbers, Hyphenation, Page Color, Page Borders, and Watermark", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Layout");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-line-numbers"]')!.click();
      expect(writerState.doc.lineNumbers).toBe(true);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-hyphenation"]')!.click();
      expect(writerState.doc.hyphenation).toBe(true);

      document.querySelector<HTMLButtonElement>('[data-testid="btn-page-color"]')!.click();
      expect(writerState.doc.backgroundColor).toBe("#f8fafc");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-page-borders"]')!.click();
      expect(writerState.doc.pageBorder).toBe("2px solid #2563eb");

      document.querySelector<HTMLButtonElement>('[data-testid="btn-watermark"]')!.click();
      expect(writerState.doc.watermark).toBe("CONFIDENTIAL");

      const watermarkEl = document.querySelector<HTMLElement>('[data-testid="word-watermark"]');
      expect(watermarkEl).not.toBeNull();
      expect(watermarkEl?.textContent).toBe("CONFIDENTIAL");
    });
  });
});
