import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { WriterCanvas } from "../components/viewports/WriterCanvas";
import { WriterState } from "../state/writerState";
import { UnifiedShellApp } from "../main";
import { FindReplaceDialog } from "../components/FindReplaceDialog";
import { WordCountDialog } from "../components/WordCountDialog";

describe("Writer Insert, Review & View Ribbons: Tables, Images, Find/Replace & Word Count (GEC-v2.0 Subphase 9.2.2)", () => {
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
      arc: () => {},
      ellipse: () => {},
      roundRect: () => {},
      closePath: () => {},
      fill: () => {},
      stroke: () => {},
      drawImage: () => {},
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
        <input type="range" id="zoom-slider" min="50" max="200" value="100" />
        <span id="status-zoom-level">100%</span>
      </div>
      <div id="bottom-sheet"></div>
      <canvas id="stylus-ink-canvas"></canvas>
      <div id="reflow-reader-container"></div>
      <div id="row-card-inspector-container"></div>
      <div id="laser-pointer-dot"></div>
    `;

    container = document.getElementById("viewport-container")!;
    state = new WriterState();
    state.doc = {
      id: "doc-adv-test",
      title: "AdvancedWriterTest.docx",
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
                  text: "The quick brown fox jumps over the lazy dog.",
                  font: "Calibri",
                  size: 11,
                  color: "#242424",
                },
              ],
            },
            {
              id: "p2",
              style: "Normal",
              alignment: "left",
              runs: [
                {
                  text: "AetherOffice delivers zero-loss roundtrip fidelity for enterprise productivity.",
                  font: "Calibri",
                  size: 11,
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
    document.body.innerHTML = "";
  });

  describe("1. Structured Table Manipulation (Insert, Add/Delete Row & Col)", () => {
    it("inserts a 3x3 table paragraph into the document structure", () => {
      const initialParas = state.doc.pages[0].paragraphs.length;
      state.insertTable(3, 3);
      expect(state.doc.pages[0].paragraphs.length).toBe(initialParas + 1);

      const tablePara = state.doc.pages[0].paragraphs[state.cursor.paragraphIndex];
      expect(tablePara.table).toBeDefined();
      expect(tablePara.table?.colCount).toBe(3);
      expect(tablePara.table?.rows.length).toBe(3);
      expect(tablePara.table?.rows[0].cells[0].text).toBe("Header 1");
      expect(tablePara.table?.rows[1].cells[1].text).toBe("Data 1,2");
    });

    it("adds and deletes table rows with column consistency", () => {
      state.insertTable(2, 4);
      const tablePara = state.doc.pages[0].paragraphs[state.cursor.paragraphIndex];
      expect(tablePara.table?.rows.length).toBe(2);

      state.addTableRow();
      expect(tablePara.table?.rows.length).toBe(3);
      expect(tablePara.table?.rows[2].cells.length).toBe(4);

      state.deleteTableRow();
      expect(tablePara.table?.rows.length).toBe(2);

      // Does not delete beyond 1 row
      state.deleteTableRow();
      state.deleteTableRow();
      expect(tablePara.table?.rows.length).toBe(1);
    });

    it("adds and deletes table columns dynamically updating all rows", () => {
      state.insertTable(3, 2);
      const tablePara = state.doc.pages[0].paragraphs[state.cursor.paragraphIndex];
      expect(tablePara.table?.colCount).toBe(2);

      state.addTableColumn();
      expect(tablePara.table?.colCount).toBe(3);
      expect(tablePara.table?.rows[0].cells.length).toBe(3);
      expect(tablePara.table?.rows[1].cells.length).toBe(3);

      state.deleteTableColumn();
      expect(tablePara.table?.colCount).toBe(2);
      expect(tablePara.table?.rows[0].cells.length).toBe(2);

      // Does not delete beyond 1 column
      state.deleteTableColumn();
      state.deleteTableColumn();
      expect(tablePara.table?.colCount).toBe(1);
    });

    it("renders table grid without crashing during canvas redraw", () => {
      state.insertTable(2, 2);
      expect(() => canvas.render()).not.toThrow();
    });
  });

  describe("2. Picture, Shape & Hyperlink Insertion", () => {
    it("inserts an embedded picture with dimensions and caption", () => {
      const initialParas = state.doc.pages[0].paragraphs.length;
      state.insertPicture("data:image/svg+xml;base64,mock", 400, 250, "Architecture Diagram");
      expect(state.doc.pages[0].paragraphs.length).toBe(initialParas + 1);

      const imgPara = state.doc.pages[0].paragraphs[state.cursor.paragraphIndex];
      expect(imgPara.image).toBeDefined();
      expect(imgPara.image?.width).toBe(400);
      expect(imgPara.image?.height).toBe(250);
      expect(imgPara.image?.caption).toBe("Architecture Diagram");
      expect(() => canvas.render()).not.toThrow();
    });

    it("inserts preset vector shapes (rect, ellipse, triangle, arrow, star)", () => {
      state.insertShape("ellipse");
      const shapePara = state.doc.pages[0].paragraphs[state.cursor.paragraphIndex];
      expect(shapePara.shape).toBeDefined();
      expect(shapePara.shape?.shapeType).toBe("ellipse");
      expect(shapePara.shape?.color).toBe("#2563eb");
      expect(() => canvas.render()).not.toThrow();

      state.insertShape("rect");
      expect(state.doc.pages[0].paragraphs[state.cursor.paragraphIndex].shape?.shapeType).toBe("rect");
    });

    it("inserts a styled hyperlink with blue color and underline", () => {
      state.insertHyperlink("OpenOfficeRS Repository", "https://github.com/rustoffice");
      const currentPara = state.doc.pages[state.cursor.pageIndex].paragraphs[state.cursor.paragraphIndex];
      const linkRun = currentPara.runs.find((r) => r.text.includes("OpenOfficeRS Repository"));
      expect(linkRun).toBeDefined();
      expect(linkRun?.color).toBe("#2563eb");
      expect(linkRun?.underline).toBe(true);
      expect(linkRun?.linkUrl).toBe("https://github.com/rustoffice");
    });
  });

  describe("3. Comments Management", () => {
    it("creates, links to paragraph, and deletes comments", () => {
      const comment = state.insertComment("Please revise this section for accuracy", "Lead Architect");
      expect(comment.id).toBeDefined();
      expect(comment.text).toBe("Please revise this section for accuracy");
      expect(comment.author).toBe("Lead Architect");
      expect(state.doc.comments?.length).toBe(1);

      const para = state.doc.pages[state.cursor.pageIndex].paragraphs[state.cursor.paragraphIndex];
      expect(para.commentId).toBe(comment.id);

      state.deleteComment(comment.id);
      expect(state.doc.comments?.length).toBe(0);
    });
  });

  describe("4. Track Changes Mode & Review Workflow", () => {
    it("toggles track changes state", () => {
      expect(state.doc.trackChanges).toBeFalsy();
      const enabled = state.toggleTrackChanges();
      expect(enabled).toBe(true);
      expect(state.doc.trackChanges).toBe(true);

      const disabled = state.toggleTrackChanges();
      expect(disabled).toBe(false);
      expect(state.doc.trackChanges).toBe(false);
    });

    it("records text additions as blue underlined insertion runs when track changes is active", () => {
      state.toggleTrackChanges();
      state.cursor = { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 4 }; // after "The "
      state.insertText("super ");

      const para = state.doc.pages[0].paragraphs[0];
      const insertedRun = para.runs.find((r) => r.text === "super ");
      expect(insertedRun).toBeDefined();
      expect(insertedRun?.insertion).toBe(true);
      expect(insertedRun?.color).toBe("#2563eb");
      expect(insertedRun?.underline).toBe(true);
    });

    it("records backspace deletions as red strikethrough runs when track changes is active", () => {
      state.toggleTrackChanges();
      // Position cursor at end of first word "The"
      state.cursor = { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 3 };
      state.deleteBackward();

      const para = state.doc.pages[0].paragraphs[0];
      const deletedRun = para.runs.find((r) => r.deletion);
      expect(deletedRun).toBeDefined();
      expect(deletedRun?.text).toBe("e");
      expect(deletedRun?.color).toBe("#dc2626");
      expect(deletedRun?.strikethrough).toBe(true);
    });

    it("accepts all changes, committing insertions and removing deletions", () => {
      state.toggleTrackChanges();
      state.cursor = { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 0 };
      state.insertText("Fresh ");
      state.cursor = { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 3 };
      state.deleteBackward();

      state.acceptChanges();
      const para = state.doc.pages[0].paragraphs[0];
      // All deletion runs removed
      expect(para.runs.some((r) => r.deletion)).toBe(false);
      // Insertions normalized
      const freshRun = para.runs.find((r) => r.text === "Fresh ");
      expect(freshRun?.insertion).toBe(false);
      expect(freshRun?.color).toBe("#242424");
      expect(freshRun?.underline).toBe(false);
    });

    it("rejects all changes, dropping insertions and restoring deletions", () => {
      state.toggleTrackChanges();
      state.cursor = { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 0 };
      state.insertText("Unwanted ");
      state.cursor = { pageIndex: 0, paragraphIndex: 0, runIndex: 0, offset: 2 };
      state.deleteBackward();

      state.rejectChanges();
      const para = state.doc.pages[0].paragraphs[0];
      // Insertions dropped
      expect(para.runs.some((r) => r.text === "Unwanted ")).toBe(false);
      // Deletions restored to plain text
      expect(para.runs.some((r) => r.deletion)).toBe(false);
      expect(para.runs.some((r) => r.strikethrough)).toBe(false);
    });
  });

  describe("5. Detailed Word Count Statistics & Dialog", () => {
    it("computes accurate word, character, paragraph, and line counts", () => {
      const stats = state.getDetailedWordCount();
      expect(stats.pages).toBe(1);
      expect(stats.paragraphs).toBe(2);
      expect(stats.words).toBeGreaterThan(15);
      expect(stats.charactersNoSpaces).toBeGreaterThan(80);
      expect(stats.charactersWithSpaces).toBeGreaterThan(stats.charactersNoSpaces);
      expect(stats.lines).toBeGreaterThanOrEqual(2);
    });

    it("renders and toggles WordCountDialog correctly", () => {
      const dialog = new WordCountDialog();
      expect(dialog.isOpen()).toBe(false);

      const stats = state.getDetailedWordCount();
      dialog.show(stats);
      expect(dialog.isOpen()).toBe(true);

      const wordsEl = document.querySelector(".wc-words");
      expect(wordsEl?.textContent).toBe(String(stats.words));

      const pagesEl = document.querySelector(".wc-pages");
      expect(pagesEl?.textContent).toBe(String(stats.pages));

      const closeBtn = document.querySelector<HTMLButtonElement>(".btn-close-wc");
      closeBtn?.click();
      expect(dialog.isOpen()).toBe(false);
    });
  });

  describe("6. Find & Replace Dialog Functionality", () => {
    it("finds all query matches across runs and paragraphs", () => {
      const matches = state.findMatches("the", false); // case insensitive
      expect(matches.length).toBeGreaterThanOrEqual(2); // "The" and "the" in p1

      const caseMatches = state.findMatches("The", true); // case sensitive
      expect(caseMatches.length).toBe(1);
    });

    it("replaces a single match accurately", () => {
      const matches = state.findMatches("brown", false);
      expect(matches.length).toBe(1);
      state.replaceMatch(matches[0], "red");

      const afterMatches = state.findMatches("red", false);
      expect(afterMatches.length).toBe(1);
      const oldMatches = state.findMatches("brown", false);
      expect(oldMatches.length).toBe(0);
    });

    it("replaces all matches in document and returns count", () => {
      const count = state.replaceAllMatches("the", "THAT", false);
      expect(count).toBeGreaterThanOrEqual(2);

      const remainingThe = state.findMatches("the", false);
      expect(remainingThe.length).toBe(0);
      const replacedThat = state.findMatches("THAT", true);
      expect(replacedThat.length).toBe(count);
    });

    it("FindReplaceDialog interactive workflow: search, navigate, replace, and replace all", () => {
      const dialog = new FindReplaceDialog();
      let selectedMatchText = "";
      dialog.setCallbacks({
        onFind: (q, c) => state.findMatches(q, c),
        onSelectMatch: (m) => {
          selectedMatchText = m.text;
        },
        onReplace: (m, r) => state.replaceMatch(m, r),
        onReplaceAll: (q, r, c) => state.replaceAllMatches(q, r, c),
        onClose: () => {},
      });

      // 1. Show in Find mode
      dialog.show("find", "quick");
      expect(dialog.isOpen()).toBe(true);
      expect(dialog.getMatchesCount()).toBe(1);
      expect(selectedMatchText).toBe("quick");

      // 2. Next / Prev navigation
      const countDisplay = document.querySelector(".match-count-display");
      expect(countDisplay?.textContent).toContain("1 of 1");

      // 3. Switch to Replace mode
      const replaceTab = document.querySelector<HTMLButtonElement>(".replace-tab-btn");
      replaceTab?.click();
      const replaceInput = document.querySelector<HTMLInputElement>(".replace-text-input");
      if (replaceInput) replaceInput.value = "swift";

      const replaceBtn = document.querySelector<HTMLButtonElement>(".btn-replace");
      replaceBtn?.click();

      // "quick" replaced by "swift"
      expect(state.findMatches("swift", false).length).toBe(1);
      expect(state.findMatches("quick", false).length).toBe(0);

      // 4. Replace All
      const findInput = document.querySelector<HTMLInputElement>(".find-query-input");
      if (findInput) findInput.value = "fox";
      if (replaceInput) replaceInput.value = "wolf";

      const replaceAllBtn = document.querySelector<HTMLButtonElement>(".btn-replace-all");
      replaceAllBtn?.click();

      expect(state.findMatches("wolf", false).length).toBe(1);
      expect(state.findMatches("fox", false).length).toBe(0);

      dialog.hide();
      expect(dialog.isOpen()).toBe(false);
    });
  });

  describe("7. View Modes, Ruler & Navigation Toggles", () => {
    it("toggles ViewMode between print, web layout and read mode", () => {
      state.execCommand("viewWebLayout");
      expect(state.doc.viewMode).toBe("web");
      expect(() => canvas.render()).not.toThrow();

      state.execCommand("viewPrintLayout");
      expect(state.doc.viewMode).toBe("print");
      expect(() => canvas.render()).not.toThrow();

      state.execCommand("viewReadMode");
      expect(state.doc.viewMode).toBe("read");
    });

    it("applies zoom presets (100%, One Page, Multi Page, Page Width)", () => {
      state.execCommand("zoom100");
      expect(state.zoom).toBe(1.0);

      state.execCommand("zoomOnePage");
      expect(state.zoom).toBe(0.85);

      state.execCommand("zoomMultiplePages");
      expect(state.zoom).toBe(0.65);

      state.execCommand("zoomPageWidth");
      expect(state.zoom).toBe(1.25);
    });
  });

  describe("8. UnifiedShellApp Ribbon Integration & Status Bar Handlers", () => {
    let app: UnifiedShellApp;

    beforeEach(() => {
      document.body.innerHTML = `
        <div id="app">
          <div id="titlebar">
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
            <input type="range" id="zoom-slider" min="50" max="200" value="100" />
            <span id="status-zoom-level">100%</span>
          </div>
        </div>
      `;
      app = new UnifiedShellApp(false);
      app.applyBootConfig({ app_mode: "writer" });
    });

    it("opens Find and Replace dialogs via Ribbon and shortcuts", () => {
      const ribbon = document.querySelector(".ribbon") as HTMLElement;
      expect(ribbon).toBeDefined();

      app.showFindReplace("find");
      const findDialog = app.getFindReplaceDialog();
      expect(findDialog.isOpen()).toBe(true);

      findDialog.hide();
      expect(findDialog.isOpen()).toBe(false);

      app.showFindReplace("replace");
      expect(findDialog.isOpen()).toBe(true);
      findDialog.hide();
    });

    it("opens WordCountDialog via Ribbon action or status bar click", () => {
      const wordCountSpan = document.getElementById("status-word-count");
      expect(wordCountSpan).toBeDefined();

      wordCountSpan?.click();
      const wcDialog = app.getWordCountDialog();
      expect(wcDialog.isOpen()).toBe(true);

      wcDialog.hide();
      expect(wcDialog.isOpen()).toBe(false);
    });

    it("wires status bar Zoom slider to dynamically change writer zoom level", () => {
      const zoomSlider = document.getElementById("zoom-slider") as HTMLInputElement;
      const zoomLabel = document.getElementById("status-zoom-level");
      expect(zoomSlider).toBeDefined();

      zoomSlider.value = "150";
      zoomSlider.dispatchEvent(new Event("input"));

      expect(zoomLabel?.textContent).toBe("150%");
    });

    it("dispatches Insert Ribbon commands: Table, Picture, Shape, Link, Comment", () => {
      const activeTab = app.getTabBar().getActiveTab()!;
      expect(activeTab).toBeDefined();

      // Trigger Insert Table
      (app as any).handleRibbonAction("InsertTable", { rows: 2, cols: 2 });
      expect(activeTab.isDirty).toBe(true);

      // Trigger Insert Picture
      (app as any).handleRibbonAction("InsertPicture", { width: 200, height: 100, caption: "Test Pic" });
      expect(activeTab.isDirty).toBe(true);

      // Trigger Insert Shape
      (app as any).handleRibbonAction("InsertShape", "triangle");
      expect(activeTab.isDirty).toBe(true);

      // Trigger Insert Link
      (app as any).handleRibbonAction("InsertLink", { text: "Docs", url: "https://aetheroffice.org" });
      expect(activeTab.isDirty).toBe(true);

      // Trigger Insert Comment
      (app as any).handleRibbonAction("InsertComment", "Check formatting");
      expect(activeTab.isDirty).toBe(true);
    });

    it("dispatches Review Ribbon commands: Track Changes, Accept, Reject", () => {
      (app as any).handleRibbonAction("ToggleTrackChanges");
      (app as any).handleRibbonAction("AcceptChange");
      (app as any).handleRibbonAction("RejectChange");
    });

    it("dispatches View Ribbon toggles: Ruler, Navigation Pane, Web Layout, Read Mode", () => {
      const subHeader = document.getElementById("sub-header-container")!;
      subHeader.style.display = "flex";
      (app as any).handleRibbonAction("ToggleRuler");
      expect(subHeader.style.display).toBe("none");

      const sidebar = document.getElementById("sidebar-container")!;
      sidebar.style.display = "block";
      (app as any).handleRibbonAction("ToggleNavPane");
      expect(sidebar.style.display).toBe("none");

      (app as any).handleRibbonAction("ViewWebLayout");
      (app as any).handleRibbonAction("ViewPrintLayout");
      (app as any).handleRibbonAction("ViewReadMode");
    });
  });
});
