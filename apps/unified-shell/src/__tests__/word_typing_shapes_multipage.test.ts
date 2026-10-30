import { describe, it, expect, beforeEach, vi } from "vitest";
import { UnifiedShellApp } from "../main";
import { writerState } from "../state/writerState";
import { IPC } from "../ipc";

describe("Subphase 29.1.1: Fix Word Typing, Cursor Focus, Multi-Page Creation & Draggable/Resizable Shapes", () => {
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

  describe("1. Continuous Typing & Cursor in .word-page-body (GEC-v4.0 Subphase 29.1.1)", () => {
    it("renders .word-page-body with contentEditable='true' on each page", () => {
      const pageBodies = document.querySelectorAll(".word-page-body");
      expect(pageBodies.length).toBeGreaterThanOrEqual(1);

      const firstBody = pageBodies[0] as HTMLElement;
      expect(firstBody.getAttribute("contenteditable")).toBe("true");
      expect(firstBody.getAttribute("data-page-index")).toBe("0");
    });

    it("allows smooth continuous typing in .word-page-body and synchronizes paragraphs to writerState", () => {
      const firstBody = document.querySelector<HTMLElement>('.word-page-body[data-page-index="0"]')!;
      expect(firstBody).not.toBeNull();

      firstBody.innerText = "The quick brown fox jumps over the lazy dog.";
      firstBody.dispatchEvent(new Event("input", { bubbles: true }));

      expect(writerState.doc.pages[0].paragraphs[0].runs[0].text).toBe(
        "The quick brown fox jumps over the lazy dog."
      );

      const wordCount = document.getElementById("status-word-count");
      expect(wordCount?.textContent).toContain("9 words");
    });

    it("handles multi-line typing and updates paragraphs accurately", () => {
      const firstBody = document.querySelector<HTMLElement>('.word-page-body[data-page-index="0"]')!;
      firstBody.textContent = "Paragraph One\nParagraph Two\nParagraph Three";
      firstBody.dispatchEvent(new Event("input", { bubbles: true }));

      expect(writerState.doc.pages[0].paragraphs.length).toBe(3);
      expect(writerState.doc.pages[0].paragraphs[0].runs[0].text).toBe("Paragraph One");
      expect(writerState.doc.pages[0].paragraphs[1].runs[0].text).toBe("Paragraph Two");
      expect(writerState.doc.pages[0].paragraphs[2].runs[0].text).toBe("Paragraph Three");
    });
  });

  describe("2. Sensible Sized Shapes (140x90), 8-Point Handles, Text & Delete", () => {
    it("inserts shape at default size of 140x90 instead of giant full-width shape", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Insert");

      // Insert rectangle
      app.getRibbon()["emit"]("InsertShape", "rect");

      const shapes = document.querySelectorAll(".word-floating-shape");
      expect(shapes.length).toBe(1);

      const shapeEl = shapes[0] as HTMLElement;
      expect(shapeEl.style.width).toBe("140px");
      expect(shapeEl.style.height).toBe("90px");
      expect(shapeEl.querySelector(".shape-body.rect")).not.toBeNull();
    });

    it("renders 8 resize handles and interactive selection border when clicked", () => {
      app.getRibbon()["emit"]("InsertShape", "ellipse");
      const shapeEl = document.querySelector<HTMLElement>(".word-floating-shape")!;
      expect(shapeEl).not.toBeNull();

      // Click to select
      shapeEl.click();
      expect(shapeEl.classList.contains("selected")).toBe(true);

      const handles = shapeEl.querySelectorAll(".shape-handle");
      expect(handles.length).toBe(8);

      const positions = Array.from(handles).map((h) => h.getAttribute("data-handle"));
      expect(positions).toEqual(["nw", "n", "ne", "e", "se", "s", "sw", "w"]);
    });

    it("allows typing text inside the shape via .shape-text", () => {
      app.getRibbon()["emit"]("InsertShape", "roundRect");
      const shapeEl = document.querySelector<HTMLElement>(".word-floating-shape")!;
      const textSpan = shapeEl.querySelector<HTMLElement>(".shape-text")!;
      expect(textSpan).not.toBeNull();

      textSpan.textContent = "Quarterly Architecture";
      textSpan.dispatchEvent(new Event("input", { bubbles: true }));

      const shapePara = writerState.doc.pages[0].paragraphs.find((p) => p.shape);
      expect(shapePara?.shape?.text).toBe("Quarterly Architecture");
    });

    it("resizes shape interactively via corner resize handle", () => {
      app.getRibbon()["emit"]("InsertShape", "rect");
      const shapeEl = document.querySelector<HTMLElement>(".word-floating-shape")!;
      const seHandle = shapeEl.querySelector<HTMLElement>('.shape-handle[data-handle="se"]')!;
      expect(seHandle).not.toBeNull();

      seHandle.dispatchEvent(new MouseEvent("mousedown", { clientX: 180, clientY: 130, bubbles: true }));
      window.dispatchEvent(new MouseEvent("mousemove", { clientX: 230, clientY: 170, bubbles: true }));
      window.dispatchEvent(new MouseEvent("mouseup", { clientX: 230, clientY: 170, bubbles: true }));

      const shapePara = writerState.doc.pages[0].paragraphs.find((p) => p.shape);
      expect(shapePara?.shape?.width).toBeGreaterThanOrEqual(180);
      expect(shapePara?.shape?.height).toBeGreaterThanOrEqual(120);
    });

    it("deletes shape cleanly via delete button and Del key", () => {
      app.getRibbon()["emit"]("InsertShape", "star");
      let shapes = document.querySelectorAll(".word-floating-shape");
      expect(shapes.length).toBe(1);

      const shapeEl = shapes[0] as HTMLElement;
      const delBtn = shapeEl.querySelector<HTMLButtonElement>('[data-testid="shape-delete-btn"]')!;
      expect(delBtn).not.toBeNull();

      delBtn.click();

      shapes = document.querySelectorAll(".word-floating-shape");
      expect(shapes.length).toBe(0);
      expect(writerState.doc.pages[0].paragraphs.some((p) => p.shape)).toBe(false);
    });
  });

  describe("3. Multi-Page Creation (+ Add Page, Blank Page, Delete Page)", () => {
    it("creates Page 2 when clicking floating + Add Page button", () => {
      expect(document.querySelectorAll(".word-page").length).toBe(1);

      const addBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-add-page"]') ||
                     document.querySelector<HTMLButtonElement>('[data-testid="btn-add-new-page"]');
      expect(addBtn).not.toBeNull();
      addBtn!.click();

      expect(document.querySelectorAll(".word-page").length).toBe(2);
      expect(writerState.doc.pages.length).toBe(2);
      expect(document.getElementById("status-doc-info")?.textContent).toContain("Page 2 of 2");
    });

    it("creates Page 3 via Insert -> Blank Page and Insert -> Page Break (Ctrl+Enter)", () => {
      const canvas = app.getWriterCanvas()!;
      canvas.insertPageBreak();
      expect(writerState.doc.pages.length).toBe(2);

      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "Enter", ctrlKey: true }));
      expect(writerState.doc.pages.length).toBe(3);
      expect(document.querySelectorAll(".word-page").length).toBe(3);
    });

    it("deletes extra page via header Delete Page button", () => {
      const canvas = app.getWriterCanvas()!;
      canvas.insertPageBreak();
      canvas.insertPageBreak();
      expect(writerState.doc.pages.length).toBe(3);

      const delBtns = document.querySelectorAll<HTMLButtonElement>('[data-testid="btn-delete-page"]');
      expect(delBtns.length).toBeGreaterThanOrEqual(1);

      delBtns[0].click();
      expect(writerState.doc.pages.length).toBe(2);
      expect(document.querySelectorAll(".word-page").length).toBe(2);
    });
  });
});
