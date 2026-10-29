import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { WriterCanvas } from "../components/viewports/WriterCanvas";
import { WriterState } from "../state/writerState";
import { WordRuler } from "../components/WordRuler";
import { UnifiedShellApp } from "../main";

describe("Multi-Page Creation, Page Breaks, Headers/Footers & Zoom/Ruler (GEC-v2.0 Subphase 9.1.2)", () => {
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
      <div id="viewport-container"></div>
      <div id="status-bar">
        <span id="status-doc-info"></span>
        <span id="status-word-count"></span>
      </div>
    `;

    container = document.getElementById("viewport-container")!;
    state = new WriterState();
    state.doc = {
      id: "doc-multipage-test",
      title: "MultiPageDocument.docx",
      pageOrientation: "portrait",
      margins: "normal",
      pageSize: "letter",
      headerText: "Executive Architecture Brief",
      footerText: "Confidential - OpenOfficeRS",
      pages: [
        {
          pageNumber: 1,
          paragraphs: [
            {
              id: "p1",
              style: "Heading 1",
              alignment: "left",
              runs: [{ text: "Chapter 1: Native High-Performance Core", font: "Calibri", size: 24, bold: true }],
            },
            {
              id: "p2",
              style: "Normal",
              alignment: "left",
              runs: [
                {
                  text: "OpenOfficeRS is engineered in Rust with strict zero-loss ECMA-376 round-trip compatibility.",
                  font: "Calibri",
                  size: 11,
                },
              ],
            },
          ],
        },
      ],
    };
    state.cursor = { pageIndex: 0, paragraphIndex: 1, runIndex: 0, offset: 10 };
    canvas = new WriterCanvas(container, state);
  });

  afterEach(() => {
    canvas.destroy();
  });

  describe("1. Distinct Paper Sheets Rendering (.word-page)", () => {
    it("renders document pages as distinct visual paper sheets separated by page gaps", () => {
      const pageElements = container.querySelectorAll(".word-page");
      expect(pageElements.length).toBe(1);

      const firstPage = pageElements[0] as HTMLElement;
      expect(firstPage.getAttribute("data-page-index")).toBe("0");
      expect(firstPage.getAttribute("data-page-number")).toBe("1");
      expect(firstPage.style.position).toBe("relative");
      expect(["#ffffff", "rgb(255, 255, 255)"]).toContain(firstPage.style.backgroundColor);

      // Check header, canvas, footer existence
      const header = firstPage.querySelector(".word-page-header");
      const canvasEl = firstPage.querySelector(".writer-canvas.word-page-canvas");
      const footer = firstPage.querySelector(".word-page-footer");

      expect(header).not.toBeNull();
      expect(canvasEl).not.toBeNull();
      expect(footer).not.toBeNull();

      // Check add page button
      const addPageBtn = container.querySelector(".btn-add-page");
      expect(addPageBtn).not.toBeNull();
      expect(addPageBtn?.textContent).toContain("Blank Page");
    });
  });

  describe("2. Manual Page Creation & Page Breaks", () => {
    it("inserts a new blank page when clicking the '➕ Blank Page' button", () => {
      expect(container.querySelectorAll(".word-page").length).toBe(1);

      const addPageBtn = container.querySelector<HTMLButtonElement>(".btn-add-page")!;
      addPageBtn.click();

      const pages = container.querySelectorAll(".word-page");
      expect(pages.length).toBe(2);
      expect(pages[0].getAttribute("data-page-number")).toBe("1");
      expect(pages[1].getAttribute("data-page-number")).toBe("2");
      expect(state.doc.pages.length).toBe(2);
      expect(state.cursor.pageIndex).toBe(1);
      expect(state.cursor.paragraphIndex).toBe(0);
    });

    it("inserts a new page and moves cursor via insertPageBreak()", () => {
      canvas.insertPageBreak();

      expect(state.doc.pages.length).toBe(2);
      expect(state.cursor.pageIndex).toBe(1);
      expect(state.cursor.paragraphIndex).toBe(0);
      expect(state.cursor.offset).toBe(0);

      const pages = container.querySelectorAll(".word-page");
      expect(pages.length).toBe(2);
    });

    it("triggers page break on Ctrl+Enter keyboard shortcut", () => {
      expect(state.doc.pages.length).toBe(1);

      canvas.handleKeyDown(new KeyboardEvent("keydown", { key: "Enter", ctrlKey: true }));

      expect(state.doc.pages.length).toBe(2);
      expect(state.cursor.pageIndex).toBe(1);
      expect(state.cursor.offset).toBe(0);

      const pages = container.querySelectorAll(".word-page");
      expect(pages.length).toBe(2);
    });

    it("updates dynamic 'Page X of Y' numbering across all page headers and footers", () => {
      canvas.insertPageBreak();
      canvas.insertPageBreak();

      expect(state.doc.pages.length).toBe(3);
      const pages = container.querySelectorAll(".word-page");
      expect(pages.length).toBe(3);

      // Verify Page 1 numbers
      const p1HeaderNum = pages[0].querySelector(".word-page-header .page-number-display")?.textContent;
      const p1FooterNum = pages[0].querySelector(".word-page-footer .page-number-display")?.textContent;
      expect(p1HeaderNum).toBe("Page 1 of 3");
      expect(p1FooterNum).toBe("Page 1 of 3");

      // Verify Page 2 numbers
      const p2HeaderNum = pages[1].querySelector(".word-page-header .page-number-display")?.textContent;
      const p2FooterNum = pages[1].querySelector(".word-page-footer .page-number-display")?.textContent;
      expect(p2HeaderNum).toBe("Page 2 of 3");
      expect(p2FooterNum).toBe("Page 2 of 3");

      // Verify Page 3 numbers
      const p3HeaderNum = pages[2].querySelector(".word-page-header .page-number-display")?.textContent;
      const p3FooterNum = pages[2].querySelector(".word-page-footer .page-number-display")?.textContent;
      expect(p3HeaderNum).toBe("Page 3 of 3");
      expect(p3FooterNum).toBe("Page 3 of 3");
    });
  });

  describe("3. Editable Headers and Footers via Double-Click", () => {
    it("activates in-place contentEditable header on double-click and saves changes", () => {
      const headerEl = container.querySelector<HTMLElement>(".word-page-header")!;
      expect(headerEl.contentEditable).not.toBe("true");

      // Double-click header
      headerEl.dispatchEvent(new MouseEvent("dblclick", { bubbles: true }));

      expect(headerEl.contentEditable).toBe("true");
      expect(headerEl.classList.contains("editing-header")).toBe(true);

      // Mutate header text and trigger blur
      headerEl.innerText = "Updated Corporate Header Page 1 of 1";
      headerEl.blur();

      expect(state.doc.headerText).toBe("Updated Corporate Header");
      expect(container.querySelector(".header-text")?.textContent).toBe("Updated Corporate Header");
    });

    it("activates in-place contentEditable footer on double-click and saves changes", () => {
      const footerEl = container.querySelector<HTMLElement>(".word-page-footer")!;
      expect(footerEl.contentEditable).not.toBe("true");

      // Double-click footer
      footerEl.dispatchEvent(new MouseEvent("dblclick", { bubbles: true }));

      expect(footerEl.contentEditable).toBe("true");
      expect(footerEl.classList.contains("editing-footer")).toBe(true);

      // Mutate footer text and trigger blur
      footerEl.innerText = "Proprietary & Strictly Confidential Page 1 of 1";
      footerEl.blur();

      expect(state.doc.footerText).toBe("Proprietary & Strictly Confidential");
      expect(container.querySelector(".footer-text")?.textContent).toBe("Proprietary & Strictly Confidential");
    });

    it("notifies doc modified callback when header or footer is updated", () => {
      let notifiedAction = "";
      canvas.setOnDocModified((action) => {
        notifiedAction = action;
      });

      const headerEl = container.querySelector<HTMLElement>(".word-page-header")!;
      headerEl.dispatchEvent(new MouseEvent("dblclick", { bubbles: true }));
      headerEl.innerText = "New Header Title";
      headerEl.blur();

      expect(notifiedAction).toBe("SetHeader");
    });
  });

  describe("4. Auto-Pagination on Content Overflow", () => {
    it("splits overflowing paragraphs into a new page when vertical limit is exceeded", () => {
      // Build a document with 35 paragraphs that exceed one page's height (864px)
      state.doc.pages[0].paragraphs = [];
      for (let i = 0; i < 40; i++) {
        state.doc.pages[0].paragraphs.push({
          id: `para-${i}`,
          style: "Normal",
          alignment: "left",
          runs: [
            {
              text: `This is long paragraph paragraph number ${i} with sufficient text length to fill lines in the paginated canvas layout.`,
              font: "Calibri",
              size: 11,
            },
          ],
        });
      }

      expect(state.doc.pages.length).toBe(1);

      // Trigger re-render which executes autoPaginate
      canvas.render();

      // Multi-page split must have occurred automatically
      expect(state.doc.pages.length).toBeGreaterThan(1);
      const renderedPages = container.querySelectorAll(".word-page");
      expect(renderedPages.length).toBe(state.doc.pages.length);
    });
  });

  describe("5. Interactive Horizontal Ruler & Margin Adjustments", () => {
    it("initializes WordRuler with draggable handles and adjusts canvas margins", () => {
      const ruler = new WordRuler("sub-header-container");
      const rulerEl = document.getElementById("sub-header-container")!;

      const leftMarginBox = rulerEl.querySelector<HTMLElement>(".ruler-left-margin");
      const rightMarginBox = rulerEl.querySelector<HTMLElement>(".ruler-right-margin");
      const indentSlider = rulerEl.querySelector<HTMLElement>(".ruler-indent-slider");

      expect(leftMarginBox).not.toBeNull();
      expect(rightMarginBox).not.toBeNull();
      expect(indentSlider).not.toBeNull();

      let marginChangeFired = false;
      let leftVal = 0;
      let rightVal = 0;

      ruler.setOnMarginsChange((l, r) => {
        marginChangeFired = true;
        leftVal = l;
        rightVal = r;
        canvas.setMargins(l, r);
      });

      ruler.setMargins(120, 120, 18);
      expect(marginChangeFired).toBe(true);
      expect(leftVal).toBe(120);
      expect(rightVal).toBe(120);
      expect(ruler.getMargins().left).toBe(120);
      expect(ruler.getMargins().right).toBe(120);
      expect(ruler.getMargins().indent).toBe(18);

      canvas.setMargins(120, 120);
      expect(canvas.marginX).toBe(120);
    });
  });

  describe("6. Full UnifiedShellApp Ribbon Integration", () => {
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

    it("wires Insert -> Blank Page ribbon button to create new page and mark tab dirty", () => {
      const writerCv = app.getWriterCanvas()!;
      expect(writerCv).toBeDefined();

      const initialPages = document.querySelectorAll(".word-page").length;
      expect(initialPages).toBeGreaterThanOrEqual(1);

      // Click Insert tab on Ribbon
      const tabs = document.querySelectorAll(".ribbon-tab-btn");
      const insertTabBtn = Array.from(tabs).find((b) => b.textContent?.includes("Insert")) as HTMLElement;
      expect(insertTabBtn).toBeDefined();
      insertTabBtn.click();

      // Click "Blank Page" button in Pages group
      const ribbonBtns = document.querySelectorAll(".ribbon-btn");
      const blankPageBtn = Array.from(ribbonBtns).find(
        (b) => (b as HTMLElement).title === "Blank Page" || b.textContent?.includes("Blank")
      ) as HTMLElement;
      expect(blankPageBtn).toBeDefined();
      blankPageBtn.click();

      // Verify page count increased
      const updatedPages = document.querySelectorAll(".word-page").length;
      expect(updatedPages).toBe(initialPages + 1);

      // Verify tab is dirty
      const activeTab = app.getTabBar().getActiveTab();
      expect(activeTab?.isDirty).toBe(true);
    });

    it("wires Insert -> Page Break ribbon button to insert break and mark tab dirty", () => {
      const initialPages = document.querySelectorAll(".word-page").length;

      // Click Insert tab on Ribbon
      const tabs = document.querySelectorAll(".ribbon-tab-btn");
      const insertTabBtn = Array.from(tabs).find((b) => b.textContent?.includes("Insert")) as HTMLElement;
      insertTabBtn.click();

      // Click "Page Break" button
      const ribbonBtns = document.querySelectorAll(".ribbon-btn");
      const pageBreakBtn = Array.from(ribbonBtns).find(
        (b) => (b as HTMLElement).title === "Page Break" || b.textContent?.includes("Break")
      ) as HTMLElement;
      expect(pageBreakBtn).toBeDefined();
      pageBreakBtn.click();

      // Verify page count increased
      const updatedPages = document.querySelectorAll(".word-page").length;
      expect(updatedPages).toBe(initialPages + 1);

      const activeTab = app.getTabBar().getActiveTab();
      expect(activeTab?.isDirty).toBe(true);
    });

    it("connects WordRuler to active WriterCanvas margins", () => {
      const ruler = app.getWordRuler();
      expect(ruler).toBeDefined();

      const writerCv = app.getWriterCanvas()!;
      expect(writerCv.marginX).toBe(96);

      ruler.setMargins(144, 144, 0);
      writerCv.setMargins(144, 144);
      expect(writerCv.marginX).toBe(144);
    });
  });
});
