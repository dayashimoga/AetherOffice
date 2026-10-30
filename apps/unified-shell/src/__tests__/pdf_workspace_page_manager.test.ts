import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { PdfState, pdfState } from "../state/pdfState";
import { PdfWorkspace } from "../apps/pdf/PdfWorkspace";
import { UnifiedShellApp } from "../main";

describe("PDF Studio Workspace: Blank PDF, Open Local PDF & Page Management (Subphase 25.1.1)", () => {
  let state: PdfState;
  let container: HTMLElement;
  let workspace: PdfWorkspace;

  beforeEach(() => {
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
        <header id="tab-bar-container" class="tab-bar"></header>
        <header id="ribbon-container" class="ribbon-bar"></header>
        <div id="sub-header-container" class="sub-header-bar"></div>
        <div class="main-content" style="display:flex; height:600px;">
          <aside id="sidebar-container" class="sidebar-pane"></aside>
          <main id="viewport-container" style="flex:1;"></main>
        </div>
        <footer id="status-bar" class="status-bar">
          <div class="status-left">
            <span id="status-doc-info"></span>
            <span id="status-word-count"></span>
          </div>
        </footer>
      </div>
    `;

    state = new PdfState();
    container = document.getElementById("viewport-container")!;
    workspace = new PdfWorkspace(container, state);
  });

  afterEach(() => {
    document.body.innerHTML = "";
  });

  describe("1. Blank PDF Document & Open Local PDF Architecture", () => {
    it("resets to a 100% clean blank PDF page under GEC-v3.0 Article II", () => {
      state.resetToBlank("BlankDocument.pdf");
      expect(state.pages.length).toBe(1);
      expect(state.pageCount).toBe(1);
      expect(state.currentPage).toBe(1);
      expect(state.bookmarks.length).toBe(0);
      expect(state.formFields.length).toBe(0);

      const page1 = state.getPage(1)!;
      expect(page1.textBlocks).toHaveLength(0);
      expect(page1.imageBlocks).toHaveLength(0);
      expect(page1.annotations).toHaveLength(0);
      expect(page1.rotation).toBe(0);
      expect(page1.width).toBe(816);
      expect(page1.height).toBe(1056);
    });

    it("loads parsed local PDF document pages and outline bookmarks via loadPdfDocument", () => {
      state.loadPdfDocument(
        [
          { id: "page_1", pageNumber: 1, width: 816, height: 1056, rotation: 0, textBlocks: [], imageBlocks: [], formFields: [], annotations: [] },
          { id: "page_2", pageNumber: 2, width: 816, height: 1056, rotation: 90, textBlocks: [], imageBlocks: [], formFields: [], annotations: [] },
        ],
        [
          { id: "bm1", title: "Chapter 1", pageNumber: 1, level: 0 },
          { id: "bm2", title: "Chapter 2", pageNumber: 2, level: 0 },
        ]
      );

      expect(state.pages.length).toBe(2);
      expect(state.pageCount).toBe(2);
      expect(state.bookmarks.length).toBe(2);
      expect(state.bookmarks[0].title).toBe("Chapter 1");
      expect(state.getPage(2)?.rotation).toBe(90);
    });
  });

  describe("2. Page Management: Add, Delete, Rotate, and Reorder", () => {
    beforeEach(() => {
      state.resetToBlank("ManagePages.pdf");
      workspace.render();
    });

    it("adds blank pages and updates page indices", () => {
      expect(state.pages.length).toBe(1);
      state.execCommand("InsertBlankPage");
      expect(state.pages.length).toBe(2);
      expect(state.pages[0].pageNumber).toBe(1);
      expect(state.pages[1].pageNumber).toBe(2);

      state.execCommand("InsertBlankPage");
      expect(state.pages.length).toBe(3);
    });

    it("rotates pages 90 degrees clockwise and counter-clockwise", () => {
      expect(state.getPage(1)?.rotation).toBe(0);

      state.execCommand("RotatePageCW");
      expect(state.getPage(1)?.rotation).toBe(90);

      state.execCommand("RotatePageCW");
      expect(state.getPage(1)?.rotation).toBe(180);

      state.execCommand("RotatePageCCW");
      expect(state.getPage(1)?.rotation).toBe(90);

      state.execCommand("RotatePageCCW");
      expect(state.getPage(1)?.rotation).toBe(0);
    });

    it("moves pages up and down to reorder pages", () => {
      state.execCommand("InsertBlankPage");
      state.execCommand("InsertBlankPage");
      expect(state.pages.length).toBe(3);

      state.addTextBlock(1, 100, 100, "First Page Text");
      state.addTextBlock(2, 100, 100, "Second Page Text");

      state.setCurrentPage(2);
      state.execCommand("MovePageUp");
      expect(state.currentPage).toBe(1);
      expect(state.getPage(1)?.textBlocks?.[0].text).toBe("Second Page Text");
      expect(state.getPage(2)?.textBlocks?.[0].text).toBe("First Page Text");

      state.execCommand("MovePageDown");
      expect(state.currentPage).toBe(2);
      expect(state.getPage(1)?.textBlocks?.[0].text).toBe("First Page Text");
      expect(state.getPage(2)?.textBlocks?.[0].text).toBe("Second Page Text");
    });

    it("deletes a page and prevents deleting the last remaining page", () => {
      state.execCommand("InsertBlankPage");
      expect(state.pages.length).toBe(2);

      state.execCommand("DeletePage", 2);
      expect(state.pages.length).toBe(1);

      const delLast = state.execCommand("DeletePage", 1);
      expect(delLast).toBe(false);
      expect(state.pages.length).toBe(1);
    });

    it("renders page thumbnail cards in sidebar with rotate and delete controls", () => {
      state.execCommand("InsertBlankPage");
      workspace.render();

      const sidebar = container.querySelector('[data-testid="pdf-sidebar"]');
      expect(sidebar).not.toBeNull();

      const thumbs = container.querySelectorAll('[data-testid="pdf-thumb-card"]');
      expect(thumbs.length).toBe(2);

      const rotBtns = container.querySelectorAll('[data-testid="btn-thumb-rotate"]');
      expect(rotBtns.length).toBe(2);

      const delBtns = container.querySelectorAll('[data-testid="btn-thumb-delete"]');
      expect(delBtns.length).toBe(2);
    });
  });

  describe("3. Direct Text & Image Authoring on PDF", () => {
    beforeEach(() => {
      state.resetToBlank("Authoring.pdf");
      workspace.render();
    });

    it("adds movable editable text blocks and respects font family, size, color, bold and italic", () => {
      state.execCommand("SetPdfFontFamily", "Arial");
      state.execCommand("SetPdfFontSize", 20);
      state.execCommand("SetPdfTextColor", "#2563eb");
      state.execCommand("TogglePdfBold");
      state.execCommand("TogglePdfItalic");

      expect(state.activeFontFamily).toBe("Arial");
      expect(state.activeFontSize).toBe(20);
      expect(state.activeFontColor).toBe("#2563eb");
      expect(state.activeBold).toBe(true);
      expect(state.activeItalic).toBe(true);

      const tb = state.addTextBlock(1, 150, 220, "Contract Specification");
      expect(tb.text).toBe("Contract Specification");
      expect(tb.fontFamily).toBe("Arial");
      expect(tb.fontSize).toBe(20);
      expect(tb.fontColor).toBe("#2563eb");
      expect(tb.bold).toBe(true);
      expect(tb.italic).toBe(true);

      workspace.render();
      const textBlockEls = container.querySelectorAll('[data-testid="pdf-text-block-editable"]');
      expect(textBlockEls.length).toBe(1);
    });

    it("modifies existing text block in-place with undo/redo capability", () => {
      const tb = state.addTextBlock(1, 100, 100, "Initial Caption");
      expect(state.getPage(1)?.textBlocks?.[0].text).toBe("Initial Caption");

      state.updateTextBlock(1, tb.id, "Revised Title 2026");
      expect(state.getPage(1)?.textBlocks?.[0].text).toBe("Revised Title 2026");

      state.undo();
      expect(state.getPage(1)?.textBlocks?.[0].text).toBe("Initial Caption");

      state.redo();
      expect(state.getPage(1)?.textBlocks?.[0].text).toBe("Revised Title 2026");
    });

    it("adds and renders image blocks on PDF overlay", () => {
      state.execCommand("AddPdfImage", {
        x: 120,
        y: 180,
        width: 300,
        height: 200,
        title: "Architecture Diagram",
      });

      const page1 = state.getPage(1)!;
      expect(page1.imageBlocks?.length).toBe(1);
      expect(page1.imageBlocks?.[0].width).toBe(300);

      workspace.render();
      const imgBlockEls = container.querySelectorAll('[data-testid="pdf-image-block"]');
      expect(imgBlockEls.length).toBe(1);
    });
  });

  describe("4. Zoom & Viewport Display Controls", () => {
    beforeEach(() => {
      state.resetToBlank("ZoomTest.pdf");
      workspace.render();
    });

    it("controls zoom levels via ZoomIn, ZoomOut, Zoom100, ZoomFitPage, ZoomFitWidth", () => {
      expect(state.zoom).toBe(1.0);

      state.execCommand("ZoomIn");
      expect(state.zoom).toBe(1.25);

      state.execCommand("ZoomOut");
      expect(state.zoom).toBe(1.0);

      state.execCommand("ZoomFitPage");
      expect(state.zoom).toBe(0.8);

      state.execCommand("ZoomFitWidth");
      expect(state.zoom).toBe(1.25);

      state.execCommand("Zoom100");
      expect(state.zoom).toBe(1.0);
    });

    it("switches view modes between continuous, single, and twoPage", () => {
      expect(state.viewMode).toBe("continuous");

      state.execCommand("SetPageLayout", "single");
      expect(state.viewMode).toBe("single");

      state.execCommand("SetPageLayout", "two-page");
      expect(state.viewMode).toBe("twoPage");

      state.execCommand("SetPageLayout", "continuous");
      expect(state.viewMode).toBe("continuous");
    });
  });

  describe("5. UnifiedShellApp Ribbon Wiring & Integration", () => {
    it("opens PDF tab with clean blank document and mounts PdfWorkspace", () => {
      const app = new UnifiedShellApp();
      expect(app).toBeDefined();

      app.getTabBar().addTab({
        id: "tab-test-pdf",
        docType: "PDF",
        title: "TestPdf.pdf",
        isDirty: false,
      });
      app.getTabBar().switchTab("tab-test-pdf");

      const ws = app.getPdfWorkspace();
      expect(ws).toBeDefined();

      // Ribbon tab switching to Page Tools
      const ribbon = app.getRibbon();
      expect(ribbon.getVisibleTabs()).toContain("Page Tools");
      expect(ribbon.getVisibleTabs()).toContain("Edit PDF");
      expect(ribbon.getVisibleTabs()).toContain("Forms");

      ribbon.setActiveTab("Page Tools");
      const addPageBtn = document.querySelector('[data-testid="pdf-btn-add-page"]') as HTMLButtonElement | null;
      expect(addPageBtn).not.toBeNull();

      const initialPages = pdfState.pages.length;
      addPageBtn?.click();
      expect(pdfState.pages.length).toBe(initialPages + 1);

      // Ribbon Edit PDF tab controls
      ribbon.setActiveTab("Edit PDF");
      const addTextBtn = document.querySelector('[data-testid="pdf-btn-add-text"]') as HTMLButtonElement | null;
      expect(addTextBtn).not.toBeNull();
      addTextBtn?.click();
      expect(pdfState.activeTool).toBe("edit_text");
    });
  });
});
