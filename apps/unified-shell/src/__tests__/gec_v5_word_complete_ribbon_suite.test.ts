import { describe, it, expect, beforeEach, vi } from "vitest";
import { UnifiedShellApp } from "../main";
import { writerState } from "../state/writerState";
import { IPC } from "../ipc";

describe("GEC-v5.0 Subphase 29.1.2: Complete Word Ribbon & Selection Preservation Suite", () => {
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
      measureText: () => ({ width: 12 }),
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
        <div id="left-app-rail-container"></div>
        <div id="tab-bar-container"></div>
        <div id="ribbon-container"></div>
        <div id="sub-header-container" style="display: none;"></div>
        <div class="workbench">
          <div id="sidebar-container"></div>
          <main id="viewport-container" class="viewport"></main>
          <div id="copilot-sidebar-container"></div>
        </div>
        <footer id="status-bar-container">
          <span id="status-doc-info"></span>
          <span id="status-page-info"></span>
          <span id="status-word-count"></span>
          <input type="range" id="zoom-slider" min="50" max="200" value="100" />
          <span id="status-zoom-level">100%</span>
        </footer>
      </div>
    `;

    app = new UnifiedShellApp();
    app.init();
    app.getTabBar().switchTab("tab-writer-1");
  });

  describe("1. Font Family, Font Size & Selection Range Preservation", () => {
    it("tracks selection range and restores it when applying font family and font size", () => {
      // Setup paragraph with text "Antigravity Engineering"
      writerState.doc.pages[0].paragraphs = [
        {
          id: "p-test-1",
          style: "Normal",
          alignment: "left",
          runs: [
            { text: "Antigravity ", font: "Calibri", size: 11 },
            { text: "Engineering", font: "Calibri", size: 11 },
          ],
        },
      ];
      app.getWriterCanvas()?.render();

      // Simulate selection of run 1 ("Engineering")
      const bodyEl = document.querySelector(".word-page-body") as HTMLElement;
      expect(bodyEl).toBeDefined();

      const textNode = bodyEl.firstChild;
      if (textNode && typeof window.getSelection === "function") {
        const range = document.createRange();
        range.setStart(textNode, 12);
        range.setEnd(textNode, 23);
        window.getSelection()?.removeAllRanges();
        window.getSelection()?.addRange(range);
        app.getWriterCanvas()?.saveSelection();
        expect(app.getWriterCanvas()?.savedSelectionRangeRef).toBeDefined();
      } else {
        writerState.selection = {
          anchor: { pageIndex: 0, paragraphIndex: 0, runIndex: 1, offset: 0 },
          focus: { pageIndex: 0, paragraphIndex: 0, runIndex: 1, offset: 11 },
        };
      }

      // Change Font Family to "Inter"
      app.handleRibbonAction("SetFontFamily", "Inter");
      const p = writerState.doc.pages[0].paragraphs[0];
      const engineeringRun = p.runs.find((r) => r.text.includes("Engineering"));
      expect(engineeringRun).toBeDefined();
      expect(engineeringRun?.font).toBe("Inter");

      // Original unselected text remains Calibri
      const antigravityRun = p.runs.find((r) => r.text.includes("Antigravity"));
      expect(antigravityRun).toBeDefined();
      expect(antigravityRun?.font).toBe("Calibri");

      // Change Font Size to 24 on selection
      app.handleRibbonAction("SetFontSize", 24);
      const updatedRun = p.runs.find((r) => r.text.includes("Engineering"));
      expect(updatedRun?.size).toBe(24);
    });

    it("applies formatting to activeFormat and typing when selection is collapsed", () => {
      writerState.clearSelection();
      app.handleRibbonAction("SetFontFamily", "Georgia");
      app.handleRibbonAction("SetFontSize", 18);
      app.handleRibbonAction("ToggleBold");

      expect(writerState.activeFormat.font).toBe("Georgia");
      expect(writerState.activeFormat.size).toBe(18);
      expect(writerState.activeFormat.bold).toBe(true);

      // Typing new text inherits the new active format
      writerState.insertText("New Heading");
      const allRuns = writerState.doc.pages[0].paragraphs.flatMap((p) => p.runs);
      const typedRun = allRuns.find((r) => r.text.includes("New Heading"));
      expect(typedRun).toBeDefined();
      expect(typedRun?.font).toBe("Georgia");
      expect(typedRun?.size).toBe(18);
    });
  });

  describe("2. Insert Tab: Complete 100% Working Tools", () => {
    it("handles Cover Page, Blank Page, and Page Break", () => {
      app.getRibbon().selectTab("Insert");
      app.handleRibbonAction("InsertCoverPage");
      expect(writerState.doc.pages.length).toBeGreaterThanOrEqual(2);
      expect(writerState.doc.pages[0].paragraphs[0].style).toBe("Title");

      const initCount = writerState.doc.pages.length;
      app.handleRibbonAction("InsertBlankPage");
      expect(writerState.doc.pages.length).toBe(initCount + 1);
    });

    it("inserts interactive Table with + Row, + Col, Delete Table mini-bar", () => {
      app.handleRibbonAction("InsertTable", { rows: 2, cols: 2 });
      const tablePara = writerState.doc.pages[0].paragraphs.find((p) => p.table);
      expect(tablePara).toBeDefined();
      expect(tablePara?.table?.rows.length).toBe(2);

      // Verify interactive DOM Table element & mini toolbar
      const tableEl = document.querySelector("[data-testid='word-interactive-table']");
      expect(tableEl).toBeDefined();

      const addRowBtn = document.querySelector("[data-testid='btn-table-add-row']") as HTMLButtonElement;
      expect(addRowBtn).toBeDefined();
      addRowBtn.click();
      expect(tablePara?.table?.rows.length).toBe(3);

      const addColBtn = document.querySelector("[data-testid='btn-table-add-col']") as HTMLButtonElement;
      expect(addColBtn).toBeDefined();
      addColBtn.click();
      expect(tablePara?.table?.colCount).toBe(3);

      const delBtn = document.querySelector("[data-testid='btn-table-delete']") as HTMLButtonElement;
      expect(delBtn).toBeDefined();
      delBtn.click();
      expect(writerState.doc.pages[0].paragraphs.find((p) => p.table)).toBeUndefined();
    });

    it("inserts Picture with interactive resize slider and control bar", () => {
      app.handleRibbonAction("InsertPicture", {
        src: "data:image/svg+xml;utf8,<svg></svg>",
        width: 300,
        height: 200,
      });
      const imgPara = writerState.doc.pages[0].paragraphs.find((p) => p.image);
      expect(imgPara).toBeDefined();
      expect(imgPara?.image?.width).toBe(300);

      const imgContainer = document.querySelector("[data-testid='word-interactive-image']");
      expect(imgContainer).toBeDefined();

      const slider = document.querySelector("[data-testid='word-image-resize-slider']") as HTMLInputElement;
      expect(slider).toBeDefined();
      slider.value = "450";
      slider.dispatchEvent(new Event("input"));
      expect(imgPara?.image?.width).toBe(450);

      const delBtn = document.querySelector("[data-testid='btn-delete-image']") as HTMLButtonElement;
      expect(delBtn).toBeDefined();
      delBtn.click();
      expect(writerState.doc.pages[0].paragraphs.find((p) => p.image)).toBeUndefined();
    });

    it("inserts draggable and resizable Shape with text edit and delete button", () => {
      app.handleRibbonAction("InsertShape", "roundRect");
      const shapePara = writerState.doc.pages[0].paragraphs.find((p) => p.shape);
      expect(shapePara).toBeDefined();
      expect(shapePara?.shape?.shapeType).toBe("roundRect");

      const shapeEl = document.querySelector("[data-testid='word-floating-shape']");
      expect(shapeEl).toBeDefined();

      const delBtn = shapeEl?.querySelector("[data-testid='shape-delete-btn']") as HTMLButtonElement;
      expect(delBtn).toBeDefined();
      delBtn.click();
      expect(writerState.doc.pages[0].paragraphs.find((p) => p.shape)).toBeUndefined();
    });

    it("focuses Header and Footer on command", () => {
      const focusSpy = vi.spyOn(HTMLElement.prototype, "focus");
      app.handleRibbonAction("InsertHeader");
      expect(focusSpy).toHaveBeenCalled();

      focusSpy.mockClear();
      app.handleRibbonAction("InsertFooter");
      expect(focusSpy).toHaveBeenCalled();
      focusSpy.mockRestore();
    });

    it("inserts Hyperlink, Symbols, and Math Equations", () => {
      app.handleRibbonAction("InsertHyperlink", { text: "OpenOfficeRS", url: "https://rustoffice.org" });
      const linkRun = writerState.doc.pages[0].paragraphs.flatMap((p) => p.runs).find((r) => r.linkUrl);
      expect(linkRun).toBeDefined();
      expect(linkRun?.linkUrl).toBe("https://rustoffice.org");

      app.handleRibbonAction("InsertSymbol", "π");
      expect(writerState.getTextContent()).toContain("π");

      app.handleRibbonAction("InsertEquation", "f(x) = ∫ 2π r dr");
      expect(writerState.getTextContent()).toContain("∫ 2π r dr");
    });
  });

  describe("3. Layout Tab: Margins, Orientation, Size, Columns, Watermark", () => {
    it("updates margins including Custom Margins", () => {
      app.handleRibbonAction("SetMargins", "narrow");
      expect(writerState.doc.margins).toBe("narrow");
      expect(app.getWriterCanvas()?.marginX).toBe(48);

      app.handleRibbonAction("SetMargins", "wide");
      expect(writerState.doc.margins).toBe("wide");
      expect(app.getWriterCanvas()?.marginX).toBe(144);

      app.handleRibbonAction("SetMargins", "custom");
      expect(writerState.doc.margins).toBe("custom");
      expect(app.getWriterCanvas()?.marginX).toBe(64);
    });

    it("swaps page dimensions on Orientation toggle", () => {
      app.handleRibbonAction("SetOrientation", "landscape");
      expect(writerState.doc.pageOrientation).toBe("landscape");
      const dim = app.getWriterCanvas()?.getPageDimensions();
      expect(dim?.width).toBe(1056);
      expect(dim?.height).toBe(816);

      app.handleRibbonAction("SetOrientation", "portrait");
      expect(writerState.doc.pageOrientation).toBe("portrait");
      const dimP = app.getWriterCanvas()?.getPageDimensions();
      expect(dimP?.width).toBe(816);
      expect(dimP?.height).toBe(1056);
    });

    it("applies Multi-Column layout to page body", () => {
      app.handleRibbonAction("SetColumns", 3);
      expect(writerState.doc.columns).toBe(3);
      const bodyEl = document.querySelector(".word-page-body") as HTMLElement;
      expect(bodyEl.style.columnCount).toBe("3");
    });

    it("sets Page Color, Page Borders, and Watermark", () => {
      app.handleRibbonAction("SetPageBackgroundColor", "#f1f5f9");
      expect((writerState.doc as any).backgroundColor).toBe("#f1f5f9");

      app.handleRibbonAction("SetPageBorder", "2px solid #2563eb");
      expect(writerState.doc.pageBorder).toBe("2px solid #2563eb");

      app.handleRibbonAction("SetWatermark", "DO NOT COPY");
      expect(writerState.doc.watermark).toBe("DO NOT COPY");
      const wm = document.querySelector("[data-testid='word-watermark']");
      expect(wm?.textContent).toBe("DO NOT COPY");
    });
  });

  describe("4. References & Review Tabs: TOC, Footnotes, Comments, Track Changes", () => {
    it("generates Table of Contents from headings", () => {
      writerState.doc.pages[0].paragraphs.push(
        { id: "h1", style: "Heading 1", alignment: "left", runs: [{ text: "System Architecture" }] },
        { id: "h2", style: "Heading 2", alignment: "left", runs: [{ text: "Memory Layout" }] }
      );
      app.handleRibbonAction("InsertTOC");
      const tocPara = writerState.doc.pages[0].paragraphs.find((p) =>
        p.runs.some((r) => r.text.includes("TABLE OF CONTENTS"))
      );
      expect(tocPara).toBeDefined();
      expect(writerState.getTextContent()).toContain("System Architecture");
    });

    it("inserts footnotes and renders footnote area on page", () => {
      app.handleRibbonAction("InsertFootnote", "ECMA-376 Part 1 Specification");
      expect(writerState.doc.footnotes?.length).toBeGreaterThanOrEqual(1);
      const fnContainer = document.querySelector("[data-testid='word-page-footnotes']");
      expect(fnContainer?.textContent).toContain("ECMA-376 Part 1 Specification");
    });

    it("adds and deletes Comments in side gutter", () => {
      app.handleRibbonAction("NewComment", "Needs clarification on memory footprint");
      expect(writerState.doc.comments?.length).toBeGreaterThanOrEqual(1);
      const card = document.querySelector("[data-testid='word-comment-card']");
      expect(card?.textContent).toContain("Needs clarification");

      const commentId = writerState.doc.comments![0].id;
      app.handleRibbonAction("DeleteComment", commentId);
      expect(writerState.doc.comments?.length).toBe(0);
    });

    it("manages Track Changes, Accept All, and Reject All", () => {
      app.handleRibbonAction("ToggleTrackChanges");
      expect(writerState.doc.trackChanges).toBe(true);

      writerState.insertText(" Proposed Change");
      const inserted = writerState.doc.pages[0].paragraphs.flatMap((p) => p.runs).find((r) => r.insertion);
      expect(inserted).toBeDefined();

      app.handleRibbonAction("AcceptAll");
      const anyInsertion = writerState.doc.pages[0].paragraphs.flatMap((p) => p.runs).some((r) => r.insertion);
      expect(anyInsertion).toBe(false);

      app.handleRibbonAction("ProtectDocument", true);
      expect(writerState.doc.readOnly).toBe(true);
      const bodyEl = document.querySelector(".word-page-body") as HTMLElement;
      expect(bodyEl.contentEditable).toBe("false");
    });
  });

  describe("5. View Tab: Multi-Pages 2-Column Grid & Layout Presets", () => {
    it("displays pages side-by-side in a 2-column grid when ZoomMultiplePages is activated", () => {
      // Add second page
      writerState.insertPageBreak();
      app.getWriterCanvas()?.render();
      expect(writerState.doc.pages.length).toBeGreaterThanOrEqual(2);

      app.handleRibbonAction("ZoomMultiplePages");
      expect(app.getWriterCanvas()?.isMultiPage).toBe(true);
      expect(writerState.zoom).toBe(0.65);

      const pagesContainer = document.querySelector(".word-pages-container") as HTMLElement;
      expect(pagesContainer.style.flexDirection).toBe("row");
      expect(pagesContainer.style.flexWrap).toBe("wrap");

      // Switch back to One Page
      app.handleRibbonAction("ZoomOnePage");
      expect(app.getWriterCanvas()?.isMultiPage).toBe(false);
      const updatedContainer = document.querySelector(".word-pages-container") as HTMLElement;
      expect(updatedContainer.style.flexDirection).toBe("column");
    });
  });
});
