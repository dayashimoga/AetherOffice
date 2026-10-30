import { describe, it, expect, beforeEach, vi } from "vitest";
import { WriterCanvas } from "../components/viewports/WriterCanvas";
import { WriterState, writerState } from "../state/writerState";
import { UnifiedShellApp } from "../main";

describe("Blank Editable Word Pages, Rich Typing, Selection & Multi-Page Engine (GEC-v3.0 Subphase 22.1.1)", () => {
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

  describe("1. 100% Blank Starting Word Document", () => {
    it("starts with exactly 1 clean completely blank page and zero words", () => {
      const state = new WriterState();
      expect(state.doc.pages.length).toBe(1);
      expect(state.doc.pages[0].pageNumber).toBe(1);

      const allText = state.doc.pages[0].paragraphs
        .flatMap((p) => p.runs)
        .map((r) => r.text)
        .join("");
      expect(allText).toBe("");
      expect(state.getWordCount()).toBe(0);
    });

    it("mounts WriterCanvas and renders exactly 1 blank paper sheet with caret", () => {
      const container = document.getElementById("viewport-container")!;
      const state = new WriterState();
      const canvas = new WriterCanvas(container, state);
      expect(canvas).toBeDefined();

      const pages = container.querySelectorAll(".word-page");
      expect(pages.length).toBe(1);

      const pageCanvases = container.querySelectorAll("canvas.word-page-canvas");
      expect(pageCanvases.length).toBe(1);

      // Caret is rendered on canvas
      expect(mockCtx.fillRect).toHaveBeenCalled();
    });
  });

  describe("2. Rich Interactive Typing, Cursor & Paragraph Splitting", () => {
    it("accepts typed input and updates document text and word count", () => {
      const container = document.getElementById("viewport-container")!;
      const state = new WriterState();
      const canvas = new WriterCanvas(container, state);

      canvas.typeText("Hello World");
      const text = state.doc.pages[0].paragraphs[0].runs[0].text;
      expect(text).toBe("Hello World");
      expect(state.getWordCount()).toBe(2);
      expect(state.cursor.offset).toBe(11);
    });

    it("splits paragraph on Enter and deletes backward on Backspace", () => {
      const container = document.getElementById("viewport-container")!;
      const state = new WriterState();
      const canvas = new WriterCanvas(container, state);

      canvas.typeText("First Paragraph");
      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "Enter" }));
      expect(state.doc.pages[0].paragraphs.length).toBe(2);

      canvas.typeText("Second Paragraph");
      expect(state.doc.pages[0].paragraphs[1].runs[0].text).toBe("Second Paragraph");

      // Backspace deletes characters
      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "Backspace" }));
      expect(state.doc.pages[0].paragraphs[1].runs[0].text).toBe("Second Paragrap");
    });

    it("supports Undo (Ctrl+Z) and Redo (Ctrl+Y)", () => {
      const container = document.getElementById("viewport-container")!;
      const state = new WriterState();
      const canvas = new WriterCanvas(container, state);

      canvas.typeText("Aether");
      expect(state.doc.pages[0].paragraphs[0].runs[0].text).toBe("Aether");

      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "z", ctrlKey: true }));
      expect(state.doc.pages[0].paragraphs[0].runs[0].text).toBe("Aethe");

      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "y", ctrlKey: true }));
      expect(state.doc.pages[0].paragraphs[0].runs[0].text).toBe("Aether");
    });
  });

  describe("3. Multi-Page Creation, Navigation & Deletion", () => {
    it("creates Page 2 via Ctrl+Enter shortcut", () => {
      const container = document.getElementById("viewport-container")!;
      const state = new WriterState();
      const canvas = new WriterCanvas(container, state);

      expect(state.doc.pages.length).toBe(1);
      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "Enter", ctrlKey: true }));

      expect(state.doc.pages.length).toBe(2);
      expect(state.cursor.pageIndex).toBe(1);
      const pages = container.querySelectorAll(".word-page");
      expect(pages.length).toBe(2);
    });

    it("appends Page 2 via bottom '+ New Page' action button", () => {
      const container = document.getElementById("viewport-container")!;
      const state = new WriterState();
      new WriterCanvas(container, state);

      const addPageBtn = container.querySelector<HTMLButtonElement>('[data-testid="btn-add-new-page"]');
      expect(addPageBtn).not.toBeNull();

      addPageBtn?.click();
      expect(state.doc.pages.length).toBe(2);
      expect(state.doc.pages[1].pageNumber).toBe(2);
    });

    it("deletes empty page via 'Delete Empty Page' button when total pages > 1", () => {
      const container = document.getElementById("viewport-container")!;
      const state = new WriterState();
      const canvas = new WriterCanvas(container, state);

      // Add a second page
      canvas.insertNewPage();
      expect(state.doc.pages.length).toBe(2);

      // Delete the empty 2nd page
      const delBtn = container.querySelector<HTMLButtonElement>('[data-testid="btn-delete-empty-page"]');
      expect(delBtn).not.toBeNull();

      delBtn?.click();
      expect(state.doc.pages.length).toBe(1);
    });

    it("automatically paginates and creates page 2 when content overflows page boundary", () => {
      const state = new WriterState();
      expect(state.doc.pages.length).toBe(1);

      // Add multiple large paragraphs exceeding 864px
      for (let i = 0; i < 30; i++) {
        state.doc.pages[0].paragraphs.push({
          id: `para-overflow-${i}`,
          style: "Normal",
          alignment: "left",
          runs: [
            {
              text: "AetherOffice enterprise word processor automatic pagination test content line filling paragraph budget.",
              font: "Calibri",
              size: 14,
            },
          ],
        });
      }

      const didSplit = state.autoPaginate(864);
      expect(didSplit).toBe(true);
      expect(state.doc.pages.length).toBeGreaterThan(1);
    });
  });

  describe("4. Editable Header & Footer Zones", () => {
    it("activates inline header editing on single click and persists edited text", () => {
      const container = document.getElementById("viewport-container")!;
      const state = new WriterState();
      new WriterCanvas(container, state);

      const headerEl = container.querySelector<HTMLElement>(".word-page-header");
      expect(headerEl).not.toBeNull();

      headerEl?.click();
      expect(headerEl?.contentEditable).toBe("true");
      expect(headerEl?.classList.contains("editing-header")).toBe(true);

      // Simulate typing new header and blur
      headerEl!.innerText = "Confidential Product Specification";
      headerEl?.dispatchEvent(new Event("blur"));

      expect(state.doc.headerText).toBe("Confidential Product Specification");
    });

    it("activates inline footer editing on single click and persists edited text", () => {
      const container = document.getElementById("viewport-container")!;
      const state = new WriterState();
      new WriterCanvas(container, state);

      const footerEl = container.querySelector<HTMLElement>(".word-page-footer");
      expect(footerEl).not.toBeNull();

      footerEl?.click();
      expect(footerEl?.contentEditable).toBe("true");
      expect(footerEl?.classList.contains("editing-footer")).toBe(true);

      // Simulate typing new footer and blur
      footerEl!.innerText = "Internal Review Only";
      footerEl?.dispatchEvent(new Event("blur"));

      expect(state.doc.footerText).toBe("Internal Review Only");
    });
  });

  describe("5. UnifiedShellApp Word Integration & Ribbon Wiring", () => {
    it("boots into Word with completely blank page and wires '+ New Page' ribbon buttons", () => {
      writerState.resetToBlank("TestDoc.docx");
      const app = new UnifiedShellApp(false);
      expect(app).toBeDefined();

      app.applyBootConfig({ app_mode: "writer" });
      const activeTab = app.getTabBar().getActiveTab();
      expect(activeTab?.docType).toBe("Word");

      const canvas = app.getWriterCanvas();
      expect(canvas).toBeDefined();
      expect(canvas?.getState().doc.pages.length).toBe(1);

      // Click Ribbon + New Page (Home tab or Insert tab)
      app.handleRibbonAction("InsertNewPage");
      expect(canvas?.getState().doc.pages.length).toBe(2);

      // Click Ribbon Delete Page
      app.handleRibbonAction("DeleteEmptyPage");
      expect(canvas?.getState().doc.pages.length).toBe(1);
    });
  });
});

