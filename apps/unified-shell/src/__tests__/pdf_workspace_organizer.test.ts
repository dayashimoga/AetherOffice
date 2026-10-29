import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { PdfState, pdfState } from "../state/pdfState";
import { PdfWorkspace } from "../apps/pdf/PdfWorkspace";
import { UnifiedShellApp } from "../main";

describe("Standalone PDF Studio Workspace, Multi-Page Renderer & Page Organizer (GEC-v2.0 Subphase 12.1.1)", () => {
  let state: PdfState;

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

    state = new PdfState();
    document.body.innerHTML = `
      <div id="app" class="desktop-shell">
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
          <div class="status-left">
            <span id="status-doc-info">Page 1 of 1</span>
          </div>
        </footer>
      </div>
    `;
  });

  afterEach(() => {
    document.body.innerHTML = "";
  });

  describe("1. Multi-Page PDF Model & Page Organizer Operations", () => {
    it("initializes with 3 default pages, bookmarks, and form fields", () => {
      expect(state.pages.length).toBe(3);
      expect(state.pageCount).toBe(3);
      expect(state.currentPage).toBe(1);
      expect(state.bookmarks.length).toBeGreaterThanOrEqual(3);
      expect(state.formFields.length).toBeGreaterThanOrEqual(2);
    });

    it("inserts a new blank page after specified page and renumbers pages", () => {
      const newPage = state.insertBlankPage(1);
      expect(newPage.pageNumber).toBe(2);
      expect(state.pages.length).toBe(4);
      expect(state.currentPage).toBe(2);
      expect(state.pages.map((p) => p.pageNumber)).toEqual([1, 2, 3, 4]);
    });

    it("deletes a page and renumbers remaining pages", () => {
      expect(state.pages.length).toBe(3);
      const res = state.deletePage(2);
      expect(res).toBe(true);
      expect(state.pages.length).toBe(2);
      expect(state.pages.map((p) => p.pageNumber)).toEqual([1, 2]);
    });

    it("prevents deleting the last remaining page in a document", () => {
      state.deletePage(1);
      state.deletePage(1);
      expect(state.pages.length).toBe(1);

      const failedDel = state.deletePage(1);
      expect(failedDel).toBe(false);
      expect(state.pages.length).toBe(1);
    });

    it("rotates pages 90 degrees clockwise and counter-clockwise", () => {
      const page1 = state.getPage(1)!;
      expect(page1.rotation).toBe(0);

      // CW: 0 -> 90 -> 180 -> 270 -> 0
      state.rotatePageClockwise(1);
      expect(page1.rotation).toBe(90);

      state.rotatePageClockwise(1);
      expect(page1.rotation).toBe(180);

      // CCW: 180 -> 90 -> 0
      state.rotatePageCounterClockwise(1);
      expect(page1.rotation).toBe(90);

      state.rotatePageCounterClockwise(1);
      expect(page1.rotation).toBe(0);
    });

    it("rotates all pages simultaneously across the document", () => {
      state.rotateAllPages(90);
      state.pages.forEach((p) => {
        expect(p.rotation).toBe(90);
      });

      state.rotateAllPages(-90);
      state.pages.forEach((p) => {
        expect(p.rotation).toBe(0);
      });
    });

    it("moves pages up and down, reordering document sequence", () => {
      const page1Id = state.pages[0].id;
      const page2Id = state.pages[1].id;

      // Move page 2 up to page 1
      state.movePageUp(2);
      expect(state.pages[0].id).toBe(page2Id);
      expect(state.pages[1].id).toBe(page1Id);
      expect(state.pages[0].pageNumber).toBe(1);
      expect(state.pages[1].pageNumber).toBe(2);

      // Move page 1 down
      state.movePageDown(1);
      expect(state.pages[0].id).toBe(page1Id);
      expect(state.pages[1].id).toBe(page2Id);
    });

    it("extracts page and merges external pages", () => {
      const extracted = state.extractPage(1);
      expect(extracted).not.toBeNull();
      expect(extracted?.pageNumber).toBe(1);

      // Merge extracted page back as new page
      const newTotal = state.mergePdf([extracted!]);
      expect(newTotal).toBe(4);
      expect(state.pages.length).toBe(4);
      expect(state.pages[3].pageNumber).toBe(4);
    });

    it("creates blank PDF document with specified page count", () => {
      state.createBlankPdf(5);
      expect(state.pages.length).toBe(5);
      expect(state.currentPage).toBe(1);
      expect(state.pages[4].pageNumber).toBe(5);
    });

    it("supports Undo and Redo across page organizer operations", () => {
      expect(state.pages.length).toBe(3);
      state.insertBlankPage(3);
      expect(state.pages.length).toBe(4);

      // Undo insertion
      state.undo();
      expect(state.pages.length).toBe(3);

      // Redo insertion
      state.redo();
      expect(state.pages.length).toBe(4);
    });
  });

  describe("2. Dedicated PDF Studio Sidebar (Thumbnails, Bookmarks, Annotations)", () => {
    let workspace: PdfWorkspace;

    beforeEach(() => {
      const viewport = document.getElementById("viewport-container")!;
      workspace = new PdfWorkspace(viewport, state);
    });

    it("renders 3 sidebar tabs: Pages, Outline, Notes", () => {
      const tabBtns = workspace.getSidebar().querySelectorAll(".pdf-sidebar-tab-btn");
      expect(tabBtns.length).toBe(3);
      expect(tabBtns[0].textContent).toContain("Pages");
      expect(tabBtns[1].textContent).toContain("Outline");
      expect(tabBtns[2].textContent).toContain("Notes");
    });

    it("renders thumbnail cards for all pages with rotation badges and mini canvases", () => {
      const cards = workspace.getSidebar().querySelectorAll(".pdf-thumbnail-card");
      expect(cards.length).toBe(3);

      // Mini canvas on each card
      expect(cards[0].querySelector("canvas")).not.toBeNull();
      expect(cards[0].textContent).toContain("Page 1");

      // Rotate page 2 and check card label
      state.rotatePageClockwise(2);
      workspace.render();
      const updatedCards = workspace.getSidebar().querySelectorAll(".pdf-thumbnail-card");
      expect(updatedCards[1].textContent).toContain("Page 2 (90°)");
    });

    it("switches to Bookmarks tab and renders document outline tree", () => {
      state.setSidebarTab("bookmarks");
      workspace.render();

      const bookmarks = workspace.getSidebar().querySelectorAll(".pdf-bookmark-item");
      expect(bookmarks.length).toBeGreaterThanOrEqual(3);
      expect(bookmarks[0].textContent).toContain("1. ISO 32000-2 Specification");
      expect(bookmarks[1].textContent).toContain("2. Security & Signatures");
    });

    it("switches to Annotations tab and lists document notes", () => {
      state.setSidebarTab("annotations");
      workspace.render();

      const annots = workspace.getSidebar().querySelectorAll(".pdf-annotation-item");
      expect(annots.length).toBeGreaterThanOrEqual(2);
      expect(annots[0].textContent).toContain("highlight");
      expect(annots[0].textContent).toContain("Certified Standard Reference");
    });
  });

  describe("3. Multi-Page Viewport & Display Modes", () => {
    let workspace: PdfWorkspace;

    beforeEach(() => {
      const viewport = document.getElementById("viewport-container")!;
      workspace = new PdfWorkspace(viewport, state);
    });

    it("renders continuous scroll layout with stacked page canvases", () => {
      state.setViewMode("continuous");
      workspace.render();

      const canvases = workspace.getViewport().querySelectorAll(".pdf-page-canvas");
      expect(canvases.length).toBe(3);
    });

    it("renders single-page fit mode showing only the active page", () => {
      state.setViewMode("single");
      state.setCurrentPage(2);
      workspace.render();

      const canvases = workspace.getViewport().querySelectorAll(".pdf-page-canvas");
      expect(canvases.length).toBe(1);
      expect(canvases[0].id).toBe("pdf-canvas-p2");
    });

    it("renders two-page spread layout", () => {
      state.setViewMode("twoPage");
      workspace.render();

      const spread = workspace.getViewport().querySelector(".pdf-spread-container");
      expect(spread).not.toBeNull();
      const canvases = spread?.querySelectorAll(".pdf-page-canvas");
      expect(canvases?.length).toBe(3);
    });

    it("scales canvas dimensions when adjusting zoom", () => {
      state.setZoom(2.0);
      workspace.render();

      const canvas = workspace.getViewport().querySelector(".pdf-page-canvas") as HTMLCanvasElement;
      expect(canvas.style.width).toBe(`${816 * 2}px`);
      expect(canvas.style.height).toBe(`${1056 * 2}px`);
    });
  });

  describe("4. UnifiedShellApp Ribbon & Keyboard Integration", () => {
    let app: UnifiedShellApp;

    beforeEach(() => {
      app = new UnifiedShellApp(false);
      app.getTabBar().addTab({
        id: "tab-pdf-1",
        docType: "PDF",
        title: "Document.pdf",
        isDirty: false,
      });
      app.getTabBar().switchTab("tab-pdf-1");
    });

    it("mounts PdfWorkspace when switching to PDF document tab", () => {
      const ws = app.getPdfWorkspace();
      expect(ws).toBeDefined();
      expect(ws?.getRoot().className).toContain("pdf-studio-workspace");
    });

    it("dispatches Page Tools Ribbon actions: InsertBlankPage, RotatePageCW, DeletePage", () => {
      expect(pdfState.pages.length).toBe(3);

      // Insert blank page
      app.handleRibbonAction("InsertBlankPage");
      expect(pdfState.pages.length).toBe(4);

      // Rotate CW
      app.handleRibbonAction("RotatePageCW");
      expect(pdfState.getPage(pdfState.currentPage)?.rotation).toBe(90);

      // Delete page
      app.handleRibbonAction("DeletePage");
      expect(pdfState.pages.length).toBe(3);
    });

    it("dispatches MovePageUp, MovePageDown, and ExtractPage actions", () => {
      pdfState.setCurrentPage(2);
      app.handleRibbonAction("MovePageUp");
      expect(pdfState.currentPage).toBe(1);

      app.handleRibbonAction("MovePageDown");
      expect(pdfState.currentPage).toBe(2);

      app.handleRibbonAction("ExtractPage");
      expect(pdfState.extractPage()).not.toBeNull();
    });

    it("dispatches View Mode and Zoom ribbon actions", () => {
      app.handleRibbonAction("SetViewMode", "twoPage");
      expect(pdfState.viewMode).toBe("twoPage");

      app.handleRibbonAction("ZoomIn");
      expect(pdfState.zoom).toBe(1.25);

      app.handleRibbonAction("Zoom100");
      expect(pdfState.zoom).toBe(1.0);
    });

    it("synchronizes status bar with current page and page count", () => {
      const docInfo = document.getElementById("status-doc-info");
      expect(docInfo?.textContent).toBe(`Page ${pdfState.currentPage} of ${pdfState.pageCount}`);

      app.handleRibbonAction("InsertBlankPage");
      expect(docInfo?.textContent).toBe(`Page ${pdfState.currentPage} of ${pdfState.pageCount}`);
    });
  });
});

