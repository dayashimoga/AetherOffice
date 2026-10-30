import { describe, it, expect, beforeEach, vi } from "vitest";
import { UnifiedShellApp } from "../main";
import { IPC } from "../ipc";
import { writerState } from "../state/writerState";
import { calcState } from "../state/calcState";
import { slidesState } from "../state/slidesState";
import { pdfState } from "../state/pdfState";
import { notesState } from "../state/notesState";
import { themeManager } from "../components/ThemeManager";

describe("Global Engineering Contract (GEC-v3.0) Suite Overhaul Final Audit", () => {
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
          <div class="title-actions">
            <button id="btn-theme-toggle" class="theme-toggle-btn">
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
        <div class="shell-body" style="display: flex; flex: 1;">
          <aside id="app-rail-container" class="app-rail"></aside>
          <div class="shell-workspace-area">
            <nav id="tab-bar-container"></nav>
            <section id="ribbon-container"></section>
            <section id="sub-header-container"></section>
            <main class="workspace-layout">
              <aside id="sidebar-container"></aside>
              <section id="viewport-container" tabindex="0">
                <canvas id="render-surface"></canvas>
              </section>
            </main>
            <footer id="status-bar">
              <span id="status-doc-info"></span>
              <span id="status-word-count"></span>
              <span id="status-zoom-level">100%</span>
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

    app = new UnifiedShellApp(false);
    app.applyBootConfig({ app_mode: "writer" });
  });

  it("1. Absolute Ban on Prepopulated Fake Strings across newly created documents", () => {
    // 1.1 Word: newly created document must be blank
    app.handleNewTab("Word");
    expect(writerState.getTextContent().trim()).toBe("");
    expect(writerState.getTotalWords()).toBe(0);

    // 1.2 Excel: newly created sheet must be clean with 0 cell data
    app.handleNewTab("Excel");
    expect(calcState.cells.size).toBe(0);
    expect(calcState.getCellValue("A1")).toBeUndefined();
    expect(calcState.getActiveSheet().name).toBe("Sheet1");

    // 1.3 PowerPoint: newly created slide deck has clean slides without fake marketing text
    app.handleNewTab("PowerPoint");
    expect(slidesState.slides.length).toBeGreaterThanOrEqual(1);
    const firstSlide = slidesState.slides[0];
    const slideText = firstSlide.shapes.map((s) => s.text || "").join(" ");
    expect(slideText).not.toContain("Executive");
    expect(slideText).not.toContain("Performance SLAs");
    expect(slideText).not.toContain("Live Demo");

    // 1.4 PDF Studio: newly created PDF starts blank
    app.handleNewTab("PDF");
    expect(pdfState.pages.length).toBe(1);
    expect(pdfState.pages[0]?.textBlocks?.length ?? 0).toBe(0);

    // 1.5 Notepad: newly created note starts with 100% empty string
    app.handleNewTab("Notes");
    expect(notesState.getContent()).toBe("");
    expect(notesState.getStats().words).toBe(0);
  });

  it("2. Left App Rail accurately routes and creates Word, Excel, PowerPoint, PDF, and Notes tabs", () => {
    const rail = app.getAppRail();
    expect(rail).toBeDefined();

    // 2.1 Route to Excel
    app.handleRailAppSelect("excel");
    let active = app.getTabBar().getActiveTab();
    expect(active?.docType).toBe("Excel");
    expect(document.querySelector(".formula-input")).not.toBeNull();

    // 2.2 Route to PowerPoint
    app.handleRailAppSelect("powerpoint");
    active = app.getTabBar().getActiveTab();
    expect(active?.docType).toBe("PowerPoint");
    expect(document.querySelector(".slide-stage-wrapper") || document.querySelector(".slide-stage-canvas")).not.toBeNull();

    // 2.3 Route to PDF
    app.handleRailAppSelect("pdf");
    active = app.getTabBar().getActiveTab();
    expect(active?.docType).toBe("PDF");
    expect(document.querySelector(".pdf-studio-workspace") || document.querySelector('[data-testid="pdf-sidebar"]')).not.toBeNull();

    // 2.4 Route to Notes
    app.handleRailAppSelect("notes");
    active = app.getTabBar().getActiveTab();
    expect(active?.docType).toBe("Notes");
    expect(document.querySelector('[data-testid="notes-workspace"]')).not.toBeNull();

    // 2.5 Route to Word
    app.handleRailAppSelect("word");
    active = app.getTabBar().getActiveTab();
    expect(active?.docType).toBe("Word");
    expect(document.querySelector(".word-page") || document.querySelector(".word-pages-container")).not.toBeNull();
  });

  it("3. Word Module: supports typing, text formatting, and creating Page 2 (+ New Page)", () => {
    app.handleNewTab("Word");

    // Typing
    writerState.insertText("AetherOffice Writer Clean Typography");
    expect(writerState.getTextContent()).toContain("AetherOffice Writer Clean Typography");

    // Formatting: Bold & Italic
    writerState.selectAll();
    writerState.execCommand("ToggleBold");
    expect(writerState.activeFormat.bold).toBe(true);

    writerState.execCommand("ToggleItalic");
    expect(writerState.activeFormat.italic).toBe(true);

    // Font family & size
    writerState.execCommand("SetFontFamily", "Segoe UI");
    expect(writerState.activeFormat.font).toBe("Segoe UI");

    writerState.execCommand("SetFontSize", 14);
    expect(writerState.activeFormat.size).toBe(14);

    // Insert Page 2
    expect(writerState.getTotalPages()).toBe(1);
    writerState.insertNewPage();
    expect(writerState.getTotalPages()).toBe(2);

    // Typing on Page 2
    writerState.insertText("Content on Page Two");
    expect(writerState.getTextContent()).toContain("Content on Page Two");
  });

  it("4. Excel Module: supports cell typing, formula evaluation =SUM(A1:A2), and adding Sheet2", () => {
    app.handleNewTab("Excel");

    // In-cell typing
    calcState.setCellValue("A1", "150");
    calcState.setCellValue("A2", "350");
    expect(calcState.getCellValue("A1")).toBe(150);
    expect(calcState.getCellValue("A2")).toBe(350);

    // Formula evaluation
    calcState.setCellValue("A3", "=SUM(A1:A2)");
    expect(calcState.getCellValue("A3")).toBe(500);

    // Multi-sheet addition
    expect(calcState.sheets.length).toBe(1);
    calcState.addSheet("Sheet2");
    expect(calcState.sheets.length).toBe(2);
    expect(calcState.getActiveSheet().name).toBe("Sheet2");

    // Data in Sheet2 is isolated from Sheet1
    expect(calcState.cells.size).toBe(0);
    calcState.setCellValue("B1", "Q1 Revenue");
    expect(calcState.getCellValue("B1")).toBe("Q1 Revenue");
  });

  it("5. PowerPoint Module: supports adding slides, slide text typing, and inserting vector shapes", () => {
    app.handleNewTab("PowerPoint");

    // Initial blank slide
    expect(slidesState.slides.length).toBe(1);

    // Add Slide 2
    slidesState.addSlide();
    expect(slidesState.slides.length).toBe(2);
    expect(slidesState.currentSlideIndex).toBe(1);

    // Type in text box
    const textShape = slidesState.addShape("text", 100, 100, 200, 50);
    textShape.text = "Quarterly Strategy Presentation";
    const activeSlide = slidesState.slides[slidesState.currentSlideIndex];
    expect(activeSlide.shapes.some((s) => s.text === "Quarterly Strategy Presentation")).toBe(true);

    // Insert geometric shapes (rect, ellipse)
    slidesState.addShape("rect");
    slidesState.addShape("ellipse");
    expect(activeSlide.shapes.some((s) => s.type === "rect")).toBe(true);
    expect(activeSlide.shapes.some((s) => s.type === "ellipse")).toBe(true);
  });

  it("6. PDF Studio: supports adding pages, text blocks, freehand ink, and AcroForms", () => {
    app.handleNewTab("PDF");

    // Add Page
    expect(pdfState.pages.length).toBe(1);
    pdfState.insertBlankPage();
    expect(pdfState.pages.length).toBe(2);

    // Add and edit text box
    const block = pdfState.addTextBlock(1, 100, 200, "Invoice Notice #2026-001");
    expect(block.text).toBe("Invoice Notice #2026-001");
    pdfState.updateTextBlock(1, block.id, "Invoice Notice #2026-001 - PAID");
    expect(pdfState.pages[0]?.textBlocks?.[0]?.text).toContain("PAID");

    // Add Annotation
    const annot = pdfState.addAnnotation({
      type: "highlight",
      pageNumber: 1,
      rect: [50, 50, 100, 20],
      color: "#ff0",
    });
    expect(annot.type).toBe("highlight");
    expect(pdfState.pages[0]?.annotations?.length ?? 0).toBe(1);

    // Add Form field
    const field = pdfState.addFormField(1, { type: "text", name: "UserField", value: "Jane Doe" });
    expect(field.type).toBe("text");
    pdfState.setFormFieldValue(field.id, "Jane Doe");
    expect(pdfState.getFormFieldValue(field.id)).toBe("Jane Doe");
  });

  it("7. Notepad & Markdown Notes: supports live editing, markdown parsing, and Dark/Light theme toggle", () => {
    app.handleNewTab("Notes");

    // Editing Notes
    notesState.setContent("# Quarterly Objectives\n- [x] Consolidate binaries\n- [ ] Release Suite v3.0");
    expect(notesState.getContent()).toContain("# Quarterly Objectives");

    // Markdown HTML parsing
    const html = notesState.renderMarkdownToHtml(notesState.getContent());
    expect(html).toContain("<h1>Quarterly Objectives</h1>");
    expect(html).toContain('<input type="checkbox" checked');

    // Theme Engine: Light to Dark toggle
    themeManager.init();
    expect(themeManager.getTheme()).toBe("light");
    themeManager.toggleTheme();
    expect(themeManager.getTheme()).toBe("dark");
    expect(document.documentElement.getAttribute("data-theme")).toBe("dark");

    // Dark to Light toggle
    themeManager.toggleTheme();
    expect(themeManager.getTheme()).toBe("light");
    expect(document.documentElement.getAttribute("data-theme")).toBe("light");
  });
});
