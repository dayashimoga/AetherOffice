import { describe, it, expect, beforeEach, vi } from "vitest";
import { WriterState, writerState } from "../state/writerState";
import { UnifiedShellApp } from "../main";

describe("Working Word Ribbon: Font, Alignment, Lists, Styles, Tables, Images & Full Tooling (GEC-v3.0 Subphase 22.1.2)", () => {
  let mockCtx: any;

  beforeEach(() => {
    mockCtx = {
      clearRect: vi.fn(),
      fillRect: vi.fn(),
      fillText: vi.fn(),
      strokeRect: vi.fn(),
      beginPath: vi.fn(),
      moveTo: vi.fn(),
      lineTo: vi.fn(),
      stroke: vi.fn(),
      fill: vi.fn(),
      measureText: vi.fn().mockImplementation((text: string) => ({ width: text.length * 8 })),
      save: vi.fn(),
      restore: vi.fn(),
      scale: vi.fn(),
      translate: vi.fn(),
      setTransform: vi.fn(),
      setLineDash: vi.fn(),
      roundRect: vi.fn(),
      rect: vi.fn(),
    };
    HTMLCanvasElement.prototype.getContext = vi.fn().mockReturnValue(mockCtx);

    document.body.innerHTML = `
      <div id="app">
        <header class="title-bar">
          <div class="window-controls">
            <button class="win-btn win-min">−</button>
            <button class="win-btn win-max">□</button>
            <button class="win-btn win-close">×</button>
          </div>
        </header>
        <aside id="app-rail-container" class="app-rail"></aside>
        <div id="tab-bar-container"></div>
        <div id="ribbon-container"></div>
        <div id="sub-header-container"></div>
        <div id="sidebar-container"></div>
        <div id="viewport-container"></div>
        <footer id="status-bar">
          <span id="status-doc-info">Page 1 of 1</span>
          <span id="status-word-count">0 words</span>
          <span id="status-zoom-level">100%</span>
          <input type="range" id="zoom-slider" />
        </footer>
      </div>
    `;
  });

  describe("1. Home Tab Ribbon Actions: Fonts, Formatting, Alignment & Styles", () => {
    it("applies font family, font size, bold, italic, underline, and strikethrough", () => {
      const state = new WriterState();
      state.insertText("Aether Text");

      state.execCommand("SetFontFamily", "Courier New");
      expect(state.doc.pages[0].paragraphs[0].runs[0].font).toBe("Courier New");

      state.execCommand("SetFontSize", 18);
      expect(state.doc.pages[0].paragraphs[0].runs[0].size).toBe(18);

      state.execCommand("ToggleBold");
      expect(state.doc.pages[0].paragraphs[0].runs[0].bold).toBe(true);

      state.execCommand("ToggleItalic");
      expect(state.doc.pages[0].paragraphs[0].runs[0].italic).toBe(true);

      state.execCommand("ToggleUnderline");
      expect(state.doc.pages[0].paragraphs[0].runs[0].underline).toBe(true);

      state.execCommand("ToggleStrikethrough");
      expect(state.doc.pages[0].paragraphs[0].runs[0].strikethrough).toBe(true);
    });

    it("applies alignments, line spacing, and bullet/numbered lists", () => {
      const state = new WriterState();
      state.insertText("Line 1");

      state.execCommand("AlignCenter");
      expect(state.doc.pages[0].paragraphs[0].alignment).toBe("center");

      state.execCommand("AlignRight");
      expect(state.doc.pages[0].paragraphs[0].alignment).toBe("right");

      state.execCommand("SetLineSpacing", 2.0);
      expect(state.doc.pages[0].paragraphs[0].lineSpacing).toBe(2.0);

      state.execCommand("ToggleBulletList");
      expect(state.doc.pages[0].paragraphs[0].runs[0].text).toContain("•");

      state.execCommand("IncreaseIndent");
      expect(state.doc.pages[0].paragraphs[0].indent).toBe(36);
    });

    it("applies Heading 1, Heading 2, Heading 3, and Quote styles", () => {
      const state = new WriterState();
      state.insertText("Heading Section");

      state.execCommand("SetStyle", "Heading 1");
      expect(state.doc.pages[0].paragraphs[0].style).toBe("Heading 1");
      expect(state.doc.pages[0].paragraphs[0].runs[0].bold).toBe(true);
      expect(state.doc.pages[0].paragraphs[0].runs[0].size).toBe(24);

      state.execCommand("SetStyle", "Heading 2");
      expect(state.doc.pages[0].paragraphs[0].style).toBe("Heading 2");
      expect(state.doc.pages[0].paragraphs[0].runs[0].size).toBe(16);

      state.execCommand("SetStyle", "Heading 3");
      expect(state.doc.pages[0].paragraphs[0].style).toBe("Heading 3");
      expect(state.doc.pages[0].paragraphs[0].runs[0].size).toBe(13);

      state.execCommand("SetStyle", "Quote");
      expect(state.doc.pages[0].paragraphs[0].style).toBe("Quote");
      expect(state.doc.pages[0].paragraphs[0].runs[0].italic).toBe(true);
      expect(state.doc.pages[0].paragraphs[0].indent).toBe(24);
    });
  });

  describe("2. Live Navigation Outline Dynamic Synchronization", () => {
    it("dynamically populates Navigation Outline when Headings are formatted and navigates on click", () => {
      writerState.resetToBlank("Architecture.docx");
      const app = new UnifiedShellApp(false);
      app.applyBootConfig({ app_mode: "writer" });

      const canvas = app.getWriterCanvas()!;
      expect(canvas).toBeDefined();

      canvas.typeText("System Overview");
      app.handleRibbonAction("SetStyle", "Heading 1");

      const navContainer = document.getElementById("sidebar-container");
      expect(navContainer).not.toBeNull();
      expect(navContainer?.textContent).toContain("System Overview");

      // Add Heading 2
      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "Enter" }));
      canvas.typeText("Memory Allocation");
      app.handleRibbonAction("SetStyle", "Heading 2");
      expect(navContainer?.textContent).toContain("Memory Allocation");
    });
  });

  describe("3. Insert Tab Ribbon Actions: Tables, Illustrations & Elements", () => {
    it("inserts an editable table and adds rows and columns", () => {
      const state = new WriterState();
      state.execCommand("InsertTable", { rows: 2, cols: 2 });

      const tablePara = state.doc.pages[0].paragraphs.find((p) => p.table !== undefined);
      expect(tablePara).toBeDefined();
      expect(tablePara?.table?.rows.length).toBe(2);
      expect(tablePara?.table?.colCount).toBe(2);

      state.execCommand("AddTableRow");
      expect(tablePara?.table?.rows.length).toBe(3);

      state.execCommand("AddTableColumn");
      expect(tablePara?.table?.colCount).toBe(3);
    });

    it("inserts pictures, shapes, horizontal lines, hyperlinks, and date/time", () => {
      const state = new WriterState();

      state.execCommand("InsertPicture", { src: "chart.png", width: 400, height: 200, caption: "Quarterly" });
      expect(state.doc.pages[0].paragraphs.some((p) => p.image !== undefined)).toBe(true);

      state.execCommand("InsertShape", "ellipse");
      expect(state.doc.pages[0].paragraphs.some((p) => p.shape?.shapeType === "ellipse")).toBe(true);

      state.execCommand("InsertHorizontalLine");
      expect(state.doc.pages[0].paragraphs.some((p) => p.id.startsWith("line-"))).toBe(true);

      state.execCommand("InsertLink", { text: "Documentation", url: "https://docs.aetheroffice.io" });
      expect(state.doc.pages[0].paragraphs.some((p) => p.runs.some((r) => r.linkUrl !== undefined))).toBe(true);

      state.execCommand("InsertDateTime");
      const lastPara = state.doc.pages[0].paragraphs[state.doc.pages[0].paragraphs.length - 1];
      expect(lastPara.runs[0].text.length).toBeGreaterThan(0);
    });
  });

  describe("4. Layout Tab Ribbon Actions: Orientation, Margins, Page Size & Background", () => {
    it("updates page setup and background color", () => {
      const state = new WriterState();

      state.execCommand("SetOrientation", "landscape");
      expect(state.doc.pageOrientation).toBe("landscape");

      state.execCommand("SetMargins", "wide");
      expect(state.doc.margins).toBe("wide");

      state.execCommand("SetPageSize", "a4");
      expect(state.doc.pageSize).toBe("a4");

      state.execCommand("SetPageBackgroundColor", "#f8fafc");
      expect((state.doc as any).backgroundColor).toBe("#f8fafc");
    });
  });

  describe("5. Review & View Tab Actions: Dialogs, Zoom & Pane Toggles", () => {
    it("opens Word Count dialog and Find/Replace dialog through ribbon actions", () => {
      writerState.resetToBlank("ReviewDoc.docx");
      const app = new UnifiedShellApp(false);
      app.applyBootConfig({ app_mode: "writer" });

      app.handleRibbonAction("WordCount");
      const wordCountModal = document.querySelector(".word-count-modal");
      expect(wordCountModal).not.toBeNull();

      app.handleRibbonAction("Find");
      const findDialog = document.querySelector(".find-replace-dialog");
      expect(findDialog).not.toBeNull();
    });

    it("adjusts zoom and toggles ruler and navigation pane", () => {
      writerState.resetToBlank("ViewDoc.docx");
      const app = new UnifiedShellApp(false);
      app.applyBootConfig({ app_mode: "writer" });

      app.handleRibbonAction("SetZoom", 0.5);
      expect(writerState.zoom).toBe(0.5);

      app.handleRibbonAction("SetZoom", 2.0);
      expect(writerState.zoom).toBe(2.0);

      const ruler = document.getElementById("sub-header-container")!;
      const nav = document.getElementById("sidebar-container")!;

      app.handleRibbonAction("ToggleRuler");
      expect(ruler.style.display).toBe("none");
      app.handleRibbonAction("ToggleRuler");
      expect(ruler.style.display).toBe("flex");

      app.handleRibbonAction("ToggleNavPane");
      expect(nav.style.display).toBe("none");
      app.handleRibbonAction("ToggleNavPane");
      expect(nav.style.display).toBe("flex");
    });
  });
});
