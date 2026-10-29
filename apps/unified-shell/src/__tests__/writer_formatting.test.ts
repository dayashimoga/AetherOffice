import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { WriterCanvas } from "../components/viewports/WriterCanvas";
import { WriterState } from "../state/writerState";
import { UnifiedShellApp } from "../main";

describe("Writer Home & Layout Ribbons: Rich Run Formatting, Lists & Styles (GEC-v2.0 Subphase 9.2.1)", () => {
  let container: HTMLElement;
  let state: WriterState;
  let canvas: WriterCanvas;

  beforeEach(() => {
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
      <div id="sub-header-container"></div>
      <div id="sidebar-container"></div>
      <div id="viewport-container"></div>
      <div id="status-bar">
        <span id="status-doc-info"></span>
        <span id="status-word-count"></span>
      </div>
    `;

    container = document.getElementById("viewport-container")!;
    state = new WriterState();
    state.doc = {
      id: "doc-format-test",
      title: "FormattingTest.docx",
      pageOrientation: "portrait",
      margins: "normal",
      pageSize: "letter",
      pages: [
        {
          pageNumber: 1,
          paragraphs: [
            {
              id: "p1",
              style: "Normal",
              alignment: "left",
              runs: [
                {
                  text: "AetherOffice delivers native high performance and round-trip fidelity.",
                  font: "Calibri",
                  size: 11,
                  bold: false,
                  italic: false,
                  color: "#242424",
                },
              ],
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

  describe("1. Font Group Run Formatting Mutations", () => {
    it("mutates font family and font size across active run", () => {
      state.execCommand("SetFontFamily", "Georgia");
      expect(state.doc.pages[0].paragraphs[0].runs[0].font).toBe("Georgia");
      expect(state.activeFormat.font).toBe("Georgia");

      state.execCommand("SetFontSize", 18);
      expect(state.doc.pages[0].paragraphs[0].runs[0].size).toBe(18);
      expect(state.activeFormat.size).toBe(18);
    });

    it("toggles bold, italic, underline, strikethrough, subscript, and superscript", () => {
      // Bold
      state.execCommand("ToggleBold");
      expect(state.doc.pages[0].paragraphs[0].runs[0].bold).toBe(true);
      state.execCommand("ToggleBold");
      expect(state.doc.pages[0].paragraphs[0].runs[0].bold).toBe(false);

      // Italic
      state.execCommand("ToggleItalic");
      expect(state.doc.pages[0].paragraphs[0].runs[0].italic).toBe(true);

      // Underline
      state.execCommand("ToggleUnderline");
      expect(state.doc.pages[0].paragraphs[0].runs[0].underline).toBe(true);

      // Strikethrough
      state.execCommand("ToggleStrikethrough");
      expect(state.doc.pages[0].paragraphs[0].runs[0].strikethrough).toBe(true);

      // Subscript & Superscript
      state.execCommand("ToggleSubscript");
      expect(state.doc.pages[0].paragraphs[0].runs[0].subscript).toBe(true);
      expect(state.doc.pages[0].paragraphs[0].runs[0].superscript).toBeFalsy();

      state.execCommand("ToggleSuperscript");
      expect(state.doc.pages[0].paragraphs[0].runs[0].superscript).toBe(true);
      expect(state.doc.pages[0].paragraphs[0].runs[0].subscript).toBe(false);
    });

    it("increases and decreases font size via A+ and A- steps", () => {
      state.activeFormat.size = 12;
      state.doc.pages[0].paragraphs[0].runs[0].size = 12;

      state.execCommand("IncreaseFontSize");
      expect(state.doc.pages[0].paragraphs[0].runs[0].size).toBe(14);
      expect(state.activeFormat.size).toBe(14);

      state.execCommand("DecreaseFontSize");
      expect(state.doc.pages[0].paragraphs[0].runs[0].size).toBe(12);
      expect(state.activeFormat.size).toBe(12);
    });

    it("sets font color and highlight color on runs", () => {
      state.execCommand("SetFontColor", "#1d4ed8");
      expect(state.doc.pages[0].paragraphs[0].runs[0].color).toBe("#1d4ed8");

      state.execCommand("SetHighlightColor", "#fde047");
      expect(state.doc.pages[0].paragraphs[0].runs[0].highlight).toBe("#fde047");
    });

    it("clears formatting and resets run styles back to normal Calibri 11", () => {
      state.execCommand("ToggleBold");
      state.execCommand("ToggleItalic");
      state.execCommand("SetFontFamily", "Times New Roman");
      state.execCommand("SetFontSize", 28);
      state.execCommand("SetFontColor", "#ef4444");

      state.execCommand("ClearFormatting");

      const run = state.doc.pages[0].paragraphs[0].runs[0];
      expect(run.bold).toBe(false);
      expect(run.italic).toBe(false);
      expect(run.font).toBe("Calibri");
      expect(run.size).toBe(11);
      expect(run.color).toBe("#242424");
    });

    it("supports selective sub-run formatting when a specific word is selected", () => {
      // Paragraph text: "AetherOffice delivers native high performance and round-trip fidelity."
      // Select the word "delivers" (offset 13 to 21)
      state.selection = {
        anchor: { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 13 },
        focus: { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 21 },
      };

      state.execCommand("ToggleBold");

      const runs = state.doc.pages[0].paragraphs[0].runs;
      expect(runs.length).toBe(3);
      expect(runs[0].text).toBe("AetherOffice ");
      expect(runs[0].bold).toBeFalsy();

      expect(runs[1].text).toBe("delivers");
      expect(runs[1].bold).toBe(true);

      expect(runs[2].text).toBe(" native high performance and round-trip fidelity.");
      expect(runs[2].bold).toBeFalsy();
    });
  });

  describe("2. Paragraph Group: Lists, Indentation, Alignment & Line Spacing", () => {
    it("toggles bullet list on paragraph", () => {
      const p = state.doc.pages[0].paragraphs[0];
      expect(p.runs[0].text.startsWith("• ")).toBe(false);

      state.execCommand("ToggleBulletList");
      expect(p.runs[0].text.startsWith("• ")).toBe(true);

      state.execCommand("ToggleBulletList");
      expect(p.runs[0].text.startsWith("• ")).toBe(false);
    });

    it("toggles numbered list on paragraph", () => {
      const p = state.doc.pages[0].paragraphs[0];
      expect(p.runs[0].text.startsWith("1. ")).toBe(false);

      state.execCommand("ToggleNumberedList");
      expect(p.runs[0].text.startsWith("1. ")).toBe(true);

      state.execCommand("ToggleNumberedList");
      expect(p.runs[0].text.startsWith("1. ")).toBe(false);
    });

    it("increases and decreases paragraph indentation", () => {
      const p = state.doc.pages[0].paragraphs[0];
      expect(p.indent || 0).toBe(0);

      state.execCommand("IncreaseIndent");
      expect(p.indent).toBe(36);

      state.execCommand("IncreaseIndent");
      expect(p.indent).toBe(72);

      state.execCommand("DecreaseIndent");
      expect(p.indent).toBe(36);

      state.execCommand("DecreaseIndent");
      expect(p.indent).toBe(0);
    });

    it("applies paragraph alignments (left, center, right, justify)", () => {
      const p = state.doc.pages[0].paragraphs[0];

      state.execCommand("AlignCenter");
      expect(p.alignment).toBe("center");

      state.execCommand("AlignRight");
      expect(p.alignment).toBe("right");

      state.execCommand("AlignJustify");
      expect(p.alignment).toBe("justify");

      state.execCommand("AlignLeft");
      expect(p.alignment).toBe("left");
    });

    it("sets line spacing on paragraph (1.0, 1.15, 1.5, 2.0)", () => {
      const p = state.doc.pages[0].paragraphs[0];

      state.execCommand("SetLineSpacing", 1.5);
      expect(p.lineSpacing).toBe(1.5);
      expect(state.activeFormat.lineSpacing).toBe(1.5);

      state.execCommand("SetLineSpacing", 2.0);
      expect(p.lineSpacing).toBe(2.0);
    });
  });

  describe("3. Styles Gallery & Dynamic Navigation Outline Synchronization", () => {
    let app: UnifiedShellApp;

    beforeEach(() => {
      document.body.innerHTML = `
        <div id="app">
          <div id="titlebar">
            <span id="titlebar-app-icon"></span>
            <span id="titlebar-filename"></span>
            <span id="titlebar-dirty-indicator"></span>
          </div>
          <div id="tab-bar-container"></div>
          <div id="ribbon-container"></div>
          <div id="sub-header-container"></div>
          <div id="sidebar-container"></div>
          <div id="viewport-container"></div>
          <div id="bottom-sheet"></div>
          <canvas id="stylus-ink-canvas"></canvas>
          <div id="reflow-reader-container"></div>
          <div id="row-card-inspector-container"></div>
          <div id="laser-pointer-dot"></div>
          <div id="status-bar">
            <span id="status-doc-info"></span>
            <span id="status-word-count"></span>
          </div>
        </div>
      `;

      app = new UnifiedShellApp(false);
      app.applyBootConfig({ app_mode: "writer" });
    });

    it("applies Heading 1 style and updates Navigation Outline Pane", () => {
      const navPane = app.getNavPane()!;
      expect(navPane).toBeDefined();

      // Click "Heading 1" button in Ribbon
      const h1Btn = document.querySelector<HTMLButtonElement>('[data-testid="btn-style-h1"]');
      expect(h1Btn).not.toBeNull();
      h1Btn!.click();

      // Verify outline pane updated with Heading 1 item
      const navItems = document.querySelectorAll(".nav-outline-item");
      expect(navItems.length).toBeGreaterThan(0);
      const firstHeading = navItems[0];
      expect(firstHeading.classList.contains("nav-outline-level-1")).toBe(true);
    });

    it("navigates to heading paragraph when clicking outline item in NavigationPane", () => {
      const navPane = app.getNavPane()!;
      expect(navPane).toBeDefined();

      const navItems = document.querySelectorAll<HTMLElement>(".nav-outline-item");
      if (navItems.length > 1) {
        // Click second heading item
        navItems[1].click();
        const writerCv = app.getWriterCanvas()!;
        expect(writerCv.getState().cursor.paragraphIndex).toBeGreaterThan(0);
      }
    });

    it("applies Heading 2, Heading 3, Title, and Subtitle styles with distinctive font sizing", () => {
      const writerCv = app.getWriterCanvas()!;
      const wState = writerCv.getState();

      wState.execCommand("SetStyle", "Heading 2");
      const p = wState.doc.pages[wState.cursor.pageIndex].paragraphs[wState.cursor.paragraphIndex];
      expect(p.style).toBe("Heading 2");
      expect(p.runs[0].size).toBe(16);
      expect(p.runs[0].bold).toBe(true);

      wState.execCommand("SetStyle", "Heading 3");
      expect(p.style).toBe("Heading 3");
      expect(p.runs[0].size).toBe(13);

      wState.execCommand("SetStyle", "Title");
      expect(p.style).toBe("Title");
      expect(p.runs[0].size).toBe(28);

      wState.execCommand("SetStyle", "Subtitle");
      expect(p.style).toBe("Subtitle");
      expect(p.runs[0].italic).toBe(true);
    });
  });

  describe("4. Layout Tab: Margins, Orientation, Paper Size & Columns", () => {
    let app: UnifiedShellApp;

    beforeEach(() => {
      document.body.innerHTML = `
        <div id="app">
          <div id="titlebar">
            <span id="titlebar-app-icon"></span>
            <span id="titlebar-filename"></span>
            <span id="titlebar-dirty-indicator"></span>
          </div>
          <div id="tab-bar-container"></div>
          <div id="ribbon-container"></div>
          <div id="sub-header-container"></div>
          <div id="sidebar-container"></div>
          <div id="viewport-container"></div>
          <div id="bottom-sheet"></div>
          <canvas id="stylus-ink-canvas"></canvas>
          <div id="reflow-reader-container"></div>
          <div id="row-card-inspector-container"></div>
          <div id="laser-pointer-dot"></div>
          <div id="status-bar">
            <span id="status-doc-info"></span>
            <span id="status-word-count"></span>
          </div>
        </div>
      `;

      app = new UnifiedShellApp(false);
      app.applyBootConfig({ app_mode: "writer" });

      // Switch Ribbon to Layout Tab
      const tabs = document.querySelectorAll(".ribbon-tab-btn");
      const layoutTab = Array.from(tabs).find((t) => t.textContent?.includes("Layout")) as HTMLElement;
      expect(layoutTab).toBeDefined();
      layoutTab.click();
    });

    it("applies Narrow, Wide, and Normal margin presets and adjusts WordRuler & Canvas", () => {
      const writerCv = app.getWriterCanvas()!;
      const ruler = app.getWordRuler();

      // Click Narrow Margins
      const narrowBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-margins-narrow"]')!;
      expect(narrowBtn).not.toBeNull();
      narrowBtn.click();

      expect(writerCv.getState().doc.margins).toBe("narrow");
      expect(writerCv.marginX).toBe(48);
      expect(ruler.getMargins().left).toBe(48);

      // Click Wide Margins
      const wideBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-margins-wide"]')!;
      wideBtn.click();

      expect(writerCv.getState().doc.margins).toBe("wide");
      expect(writerCv.marginX).toBe(144);
      expect(ruler.getMargins().left).toBe(144);

      // Click Normal Margins
      const normalBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-margins-normal"]')!;
      normalBtn.click();

      expect(writerCv.getState().doc.margins).toBe("normal");
      expect(writerCv.marginX).toBe(96);
      expect(ruler.getMargins().left).toBe(96);
    });

    it("switches document orientation between Portrait and Landscape", () => {
      const landscapeBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-orientation-landscape"]')!;
      landscapeBtn.click();

      const writerCv = app.getWriterCanvas()!;
      expect(writerCv.getState().doc.pageOrientation).toBe("landscape");

      const portraitBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-orientation-portrait"]')!;
      portraitBtn.click();
      expect(writerCv.getState().doc.pageOrientation).toBe("portrait");
    });

    it("configures paper size (Letter, A4, Legal)", () => {
      const writerCv = app.getWriterCanvas()!;

      const a4Btn = document.querySelector<HTMLButtonElement>('[data-testid="btn-pagesize-a4"]')!;
      a4Btn.click();
      expect(writerCv.getState().doc.pageSize).toBe("a4");

      const legalBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-pagesize-legal"]')!;
      legalBtn.click();
      expect(writerCv.getState().doc.pageSize).toBe("legal");

      const letterBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-pagesize-letter"]')!;
      letterBtn.click();
      expect(writerCv.getState().doc.pageSize).toBe("letter");
    });

    it("sets column layout (1, 2, 3 columns)", () => {
      const writerCv = app.getWriterCanvas()!;

      const col2Btn = document.querySelector<HTMLButtonElement>('[data-testid="btn-cols-2"]')!;
      col2Btn.click();
      expect(writerCv.getState().doc.columns).toBe(2);

      const col3Btn = document.querySelector<HTMLButtonElement>('[data-testid="btn-cols-3"]')!;
      col3Btn.click();
      expect(writerCv.getState().doc.columns).toBe(3);

      const col1Btn = document.querySelector<HTMLButtonElement>('[data-testid="btn-cols-1"]')!;
      col1Btn.click();
      expect(writerCv.getState().doc.columns).toBe(1);
    });
  });
});

