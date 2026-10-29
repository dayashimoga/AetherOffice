import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { WriterCanvas } from "../components/viewports/WriterCanvas";
import { WriterState } from "../state/writerState";
import { UnifiedShellApp } from "../main";

describe("Interactive Paginated Word Canvas, Real Caret & Keyboard Typing (GEC-v2.0 Subphase 9.1.1)", () => {
  let container: HTMLElement;
  let state: WriterState;
  let canvas: WriterCanvas;

  beforeEach(() => {
    // Setup Canvas 2D mock for headless happy-dom
    const mockCtx = {
      clearRect: () => {},
      fillRect: () => {},
      strokeRect: () => {},
      fillText: () => {},
      measureText: (text: string) => ({ width: text.length * 7 }),
      beginPath: () => {},
      moveTo: () => {},
      lineTo: () => {},
      stroke: () => {},
      save: () => {},
      restore: () => {},
      setTransform: () => {},
      scale: () => {},
      setLineDash: () => {},
      font: "",
      fillStyle: "",
      strokeStyle: "",
      lineWidth: 1,
      textAlign: "left",
      shadowColor: "",
      shadowBlur: 0,
      shadowOffsetY: 0,
    };
    HTMLCanvasElement.prototype.getContext = (() => mockCtx) as any;

    document.body.innerHTML = `
      <div id="viewport-container"></div>
      <div id="status-bar">
        <span id="status-doc-info"></span>
        <span id="status-word-count"></span>
      </div>
    `;

    container = document.getElementById("viewport-container")!;
    state = new WriterState();
    // Initialize with a clean 1-page document
    state.doc = {
      id: "test-doc-1",
      title: "Untitled.docx",
      pageOrientation: "portrait",
      margins: "normal",
      pageSize: "letter",
      pages: [
        {
          pageNumber: 1,
          paragraphs: [
            {
              id: "p0",
              style: "Normal",
              alignment: "left",
              runs: [{ text: "", font: "Calibri", size: 11 }],
            },
          ],
        },
      ],
    };
    state.cursor = { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 0 };
    state.clearSelection();

    canvas = new WriterCanvas(container, state);
  });

  afterEach(() => {
    canvas.destroy();
  });

  describe("1. Interactive Keyboard Input & 3-Paragraph Typing Simulation", () => {
    it("simulates typing a 3-paragraph document with live caret advancement and Enter splits", () => {
      let modifiedEvents: string[] = [];
      canvas.setOnDocModified((action) => {
        modifiedEvents.push(action);
      });

      // --- Paragraph 1 ---
      const p1Text = "The quick brown fox jumps over the lazy dog.";
      for (const char of p1Text) {
        canvas.handleKeyDown(new KeyboardEvent("keydown", { key: char }));
      }

      expect(state.doc.pages[0].paragraphs[0].runs[0].text).toBe(p1Text);
      expect(state.cursor.offset).toBe(p1Text.length);
      expect(state.cursor.paragraphIndex).toBe(0);

      // Press Enter to create Paragraph 2
      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "Enter" }));
      expect(state.doc.pages[0].paragraphs.length).toBe(2);
      expect(state.cursor.paragraphIndex).toBe(1);
      expect(state.cursor.offset).toBe(0);

      // --- Paragraph 2 ---
      const p2Text = "Rust delivers memory safety without garbage collection.";
      for (const char of p2Text) {
        canvas.handleKeyDown(new KeyboardEvent("keydown", { key: char }));
      }

      expect(state.doc.pages[0].paragraphs[1].runs[0].text).toBe(p2Text);
      expect(state.cursor.offset).toBe(p2Text.length);

      // Press Enter to create Paragraph 3
      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "Enter" }));
      expect(state.doc.pages[0].paragraphs.length).toBe(3);
      expect(state.cursor.paragraphIndex).toBe(2);
      expect(state.cursor.offset).toBe(0);

      // --- Paragraph 3 ---
      const p3Text = "AetherOffice provides 100% ECMA-376 fidelity.";
      for (const char of p3Text) {
        canvas.handleKeyDown(new KeyboardEvent("keydown", { key: char }));
      }

      expect(state.doc.pages[0].paragraphs[2].runs[0].text).toBe(p3Text);
      expect(state.doc.pages[0].paragraphs.length).toBe(3);
      expect(state.getTotalWords()).toBe(21);
      expect(modifiedEvents.length).toBeGreaterThan(30);
    });

    it("supports Shift+Enter for soft line breaks without splitting paragraphs", () => {
      const pText1 = "First line of paragraph";
      for (const char of pText1) {
        canvas.handleKeyDown(new KeyboardEvent("keydown", { key: char }));
      }

      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "Enter", shiftKey: true }));

      const pText2 = "Second line of same paragraph";
      for (const char of pText2) {
        canvas.handleKeyDown(new KeyboardEvent("keydown", { key: char }));
      }

      expect(state.doc.pages[0].paragraphs.length).toBe(1);
      expect(state.doc.pages[0].paragraphs[0].runs[0].text).toContain("\n");
      expect(state.doc.pages[0].paragraphs[0].runs[0].text).toBe(
        "First line of paragraph\nSecond line of same paragraph"
      );
    });

    it("supports international IME and InputEvent data composition", () => {
      const canvasEl = canvas.getCanvasElement();
      canvasEl.dispatchEvent(
        new InputEvent("beforeinput", { data: "こんにちは" })
      );

      expect(state.doc.pages[0].paragraphs[0].runs[0].text).toBe("こんにちは");
      expect(state.cursor.offset).toBe(5);
    });
  });

  describe("2. Mouse Selection (Click, Double-Click, Triple-Click, Click-Drag)", () => {
    beforeEach(() => {
      state.doc.pages[0].paragraphs = [
        {
          id: "p1",
          style: "Normal",
          alignment: "left",
          runs: [{ text: "The quick brown fox jumps over the lazy dog." }],
        },
        {
          id: "p2",
          style: "Normal",
          alignment: "left",
          runs: [{ text: "Second paragraph for testing text selections." }],
        },
      ];
      canvas.render();
    });

    it("selects exact word under cursor on double-click", () => {
      // Offset 6 is inside the word "quick" (chars 4..9)
      const targetPos = { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 6 };
      state.selectWord(targetPos);

      expect(state.selection).not.toBeNull();
      expect(state.selection?.anchor.offset).toBe(4); // "q"
      expect(state.selection?.focus.offset).toBe(9); // "k" + 1
      expect(state.isSelectionCollapsed()).toBe(false);
    });

    it("selects entire paragraph under cursor on triple-click", () => {
      const targetPos = { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 12 };
      state.selectParagraph(targetPos);

      expect(state.selection).not.toBeNull();
      expect(state.selection?.anchor.offset).toBe(0);
      expect(state.selection?.focus.offset).toBe(44); // full length of paragraph 1
      expect(state.isSelectionCollapsed()).toBe(false);
    });

    it("supports click and drag text selection across character ranges", () => {
      // Simulate click at offset 4
      state.cursor = { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 4 };
      state.selection = {
        anchor: { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 4 },
        focus: { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 19 },
      };

      expect(state.isSelectionCollapsed()).toBe(false);
      expect(state.selection?.anchor.offset).toBe(4);
      expect(state.selection?.focus.offset).toBe(19);

      // Render runs without error and draws selection
      expect(() => canvas.render()).not.toThrow();
    });
  });

  describe("3. Deletion, Selection Replacement & Backspace Paragraph Merging", () => {
    beforeEach(() => {
      state.doc.pages[0].paragraphs = [
        {
          id: "p1",
          style: "Normal",
          alignment: "left",
          runs: [{ text: "Alpha Beta Gamma" }],
        },
        {
          id: "p2",
          style: "Normal",
          alignment: "left",
          runs: [{ text: "Delta Epsilon" }],
        },
      ];
      state.cursor = { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 0 };
      state.clearSelection();
      canvas.render();
    });

    it("deletes selected text range on Backspace", () => {
      // Select "Beta " (offsets 6 to 11)
      state.selection = {
        anchor: { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 6 },
        focus: { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 11 },
      };

      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "Backspace" }));

      expect(state.doc.pages[0].paragraphs[0].runs[0].text).toBe("Alpha Gamma");
      expect(state.selection).toBeNull();
      expect(state.cursor.offset).toBe(6);
    });

    it("replaces selected text range with typed character immediately", () => {
      // Select "Beta" (offsets 6 to 10)
      state.selection = {
        anchor: { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 6 },
        focus: { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 10 },
      };

      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "Z" }));

      expect(state.doc.pages[0].paragraphs[0].runs[0].text).toBe("Alpha Z Gamma");
      expect(state.selection).toBeNull();
      expect(state.cursor.offset).toBe(7);
    });

    it("merges paragraph into preceding paragraph on Backspace at offset 0", () => {
      // Place caret at start of paragraph 2 (offset 0)
      state.cursor = { pageIndex: 0, paragraphIndex: 1, runIndex: 0, offset: 0 };

      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "Backspace" }));

      // Paragraph 2 should be merged into paragraph 1
      expect(state.doc.pages[0].paragraphs.length).toBe(1);
      expect(state.cursor.paragraphIndex).toBe(0);
      expect(state.cursor.offset).toBe(16); // merged at end of "Alpha Beta Gamma"
      expect(state.doc.pages[0].paragraphs[0].runs.map((r) => r.text).join("")).toBe(
        "Alpha Beta GammaDelta Epsilon"
      );
    });

    it("merges following paragraph on Delete at end of paragraph", () => {
      // Place caret at end of paragraph 1
      state.cursor = { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 16 };

      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "Delete" }));

      expect(state.doc.pages[0].paragraphs.length).toBe(1);
      expect(state.doc.pages[0].paragraphs[0].runs.map((r) => r.text).join("")).toBe(
        "Alpha Beta GammaDelta Epsilon"
      );
    });
  });

  describe("4. Cursor Navigation (Arrows, Word Jumps, Home/End, PageUp/Down)", () => {
    beforeEach(() => {
      state.doc.pages[0].paragraphs = [
        {
          id: "p1",
          style: "Normal",
          alignment: "left",
          runs: [{ text: "Hello beautiful world" }],
        },
      ];
      state.cursor = { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 0 };
      state.clearSelection();
      canvas.render();
    });

    it("moves by single character on ArrowRight and ArrowLeft", () => {
      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "ArrowRight" }));
      expect(state.cursor.offset).toBe(1);

      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "ArrowRight" }));
      expect(state.cursor.offset).toBe(2);

      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "ArrowLeft" }));
      expect(state.cursor.offset).toBe(1);
    });

    it("moves by whole words on Ctrl+ArrowRight and Ctrl+ArrowLeft", () => {
      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "ArrowRight", ctrlKey: true }));
      expect(state.cursor.offset).toBe(6); // after "Hello "

      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "ArrowRight", ctrlKey: true }));
      expect(state.cursor.offset).toBe(16); // after "beautiful "

      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "ArrowLeft", ctrlKey: true }));
      expect(state.cursor.offset).toBe(6); // before "beautiful "
    });

    it("jumps to start and end of line/paragraph on Home and End keys", () => {
      state.cursor.offset = 7;
      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "Home" }));
      expect(state.cursor.offset).toBe(0);

      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "End" }));
      expect(state.cursor.offset).toBe(21); // length of "Hello beautiful world"
    });

    it("expands selection when holding Shift with arrow navigation", () => {
      state.cursor.offset = 0;
      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "ArrowRight", shiftKey: true }));
      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "ArrowRight", shiftKey: true }));
      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "ArrowRight", shiftKey: true }));

      expect(state.selection).not.toBeNull();
      expect(state.selection?.anchor.offset).toBe(0);
      expect(state.selection?.focus.offset).toBe(3);
      expect(state.cursor.offset).toBe(3);
    });
  });

  describe("5. UnifiedShellApp Integration & Tab Dirty Indicator Synchronization", () => {
    it("marks active document tab dirty when typing text in WriterCanvas", () => {
      document.body.innerHTML = `
        <div id="app">
          <header class="title-bar"></header>
          <nav id="tab-bar-container" class="tab-bar"></nav>
          <section id="ribbon-container" class="ribbon-bar"></section>
          <section id="sub-header-container" class="sub-header-bar"></section>
          <aside id="sidebar-container" class="sidebar-pane"></aside>
          <main class="workspace-layout">
            <section class="canvas-viewport" id="viewport-container" tabindex="0">
              <canvas id="render-surface"></canvas>
            </section>
          </main>
          <footer id="status-bar" class="status-bar">
            <span id="status-doc-info"></span>
            <span id="status-word-count"></span>
          </footer>
        </div>
      `;

      const app = new UnifiedShellApp(false);
      app.applyBootConfig({ app_mode: "writer" });

      const activeTab = app.getTabBar().getActiveTab();
      expect(activeTab).not.toBeNull();
      expect(activeTab?.isDirty).toBe(false);

      const writer = app.getWriterCanvas();
      expect(writer).toBeDefined();

      // Simulate typing in active canvas
      writer?.handleKeyDown(new KeyboardEvent("keydown", { key: "X" }));

      // Tab should immediately reflect dirty status *
      const updatedTab = app.getTabBar().getActiveTab();
      expect(updatedTab?.isDirty).toBe(true);
    });
  });
});
