import { describe, it, expect, beforeEach, vi } from "vitest";
import { UnifiedShellApp } from "../main";
import { writerState } from "../state/writerState";
import { IPC } from "../ipc";

describe("Subphase 29.2.1: Fix Word References, Review & View Ribbons (100% Working Controls)", () => {
  let app: UnifiedShellApp;
  let mockSpeechSynthesis: {
    speak: ReturnType<typeof vi.fn>;
    pause: ReturnType<typeof vi.fn>;
    resume: ReturnType<typeof vi.fn>;
    cancel: ReturnType<typeof vi.fn>;
    speaking: boolean;
    paused: boolean;
  };

  beforeEach(() => {
    localStorage.clear();
    IPC._resetMockState();
    writerState.resetToBlank();

    mockSpeechSynthesis = {
      speak: vi.fn(),
      pause: vi.fn(),
      resume: vi.fn(),
      cancel: vi.fn(),
      speaking: false,
      paused: false,
    };
    (window as any).speechSynthesis = mockSpeechSynthesis;
    (window as any).SpeechSynthesisUtterance = function (this: any, text: string) {
      this.text = text;
    };

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

  describe("1. References Tab: TOC, Footnotes, Citations, Captions", () => {
    it("inserts and updates Table of Contents from headings", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("References");

      writerState.setStyle("Heading 1");
      writerState.insertText("Introduction to Architecture");
      writerState.insertParagraph();
      writerState.setStyle("Normal");
      writerState.insertText("Body text here.");
      writerState.insertParagraph();
      writerState.setStyle("Heading 2");
      writerState.insertText("Memory Virtualization");

      const insertTocBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-insert-toc"]')!;
      expect(insertTocBtn).not.toBeNull();
      insertTocBtn.click();

      const page0 = writerState.doc.pages[0];
      const tocPara = page0.paragraphs.find((p) => p.id.startsWith("toc-"));
      expect(tocPara).toBeDefined();
      const tocContent = tocPara!.runs.map((r) => r.text).join("");
      expect(tocContent).toContain("TABLE OF CONTENTS");
      expect(tocContent).toContain("Introduction to Architecture");
      expect(tocContent).toContain("Memory Virtualization");

      // Now add Heading 3 and click Update TOC
      writerState.insertParagraph();
      writerState.setStyle("Heading 3");
      writerState.insertText("Zero-Copy Allocation");

      const updateTocBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-update-toc"]')!;
      expect(updateTocBtn).not.toBeNull();
      updateTocBtn.click();

      const updatedToc = writerState.doc.pages[0].paragraphs.find((p) => p.id.startsWith("toc-"));
      expect(updatedToc).toBeDefined();
      const updatedContent = updatedToc!.runs.map((r) => r.text).join("");
      expect(updatedContent).toContain("Zero-Copy Allocation");
    });

    it("inserts footnotes, endnotes, and navigates footnotes", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("References");

      writerState.insertText("Standard compliance note");
      const fnBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-insert-footnote"]')!;
      expect(fnBtn).not.toBeNull();
      fnBtn.click();

      expect(writerState.footnotes.length).toBe(1);
      expect(writerState.footnotes[0].num).toBe(1);
      const fnOverlay = document.querySelector('[data-testid="word-page-footnotes"]');
      expect(fnOverlay).not.toBeNull();
      expect(fnOverlay?.textContent).toContain("[1]");

      const enBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-insert-endnote"]')!;
      expect(enBtn).not.toBeNull();
      enBtn.click();
      expect(writerState.footnotes.length).toBe(2);

      const nextFnBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-next-footnote"]')!;
      expect(nextFnBtn).not.toBeNull();
      nextFnBtn.click();
      expect(writerState.cursor.pageIndex).toBe(0);
    });

    it("manages citations, sources, citation styles, and bibliography", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("References");

      const citeStyleBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-citation-style"]')!;
      expect(citeStyleBtn).not.toBeNull();
      citeStyleBtn.click();
      expect(writerState.doc.citationStyle).toBe("APA");

      const insertCiteBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-insert-citation"]')!;
      expect(insertCiteBtn).not.toBeNull();
      insertCiteBtn.click();
      expect(writerState.citations.length).toBe(1);
      expect(writerState.citations[0].author).toBe("Smith, J.");

      const manageSourcesBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-manage-sources"]')!;
      expect(manageSourcesBtn).not.toBeNull();
      manageSourcesBtn.click();
      expect(writerState.citations.length).toBe(2);

      const bibBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-insert-bib"]')!;
      expect(bibBtn).not.toBeNull();
      bibBtn.click();

      const lastPara = writerState.doc.pages[0].paragraphs[writerState.doc.pages[0].paragraphs.length - 1];
      expect(lastPara.id.startsWith("bib-")).toBe(true);
      const bibText = lastPara.runs.map((r) => r.text).join("");
      expect(bibText).toContain("BIBLIOGRAPHY");
      expect(bibText).toContain("Smith, J.");
    });

    it("inserts captions and table of figures", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("References");

      const capBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-insert-caption"]')!;
      expect(capBtn).not.toBeNull();
      capBtn.click();

      const capPara = writerState.doc.pages[0].paragraphs.find((p) => p.id.startsWith("cap-"));
      expect(capPara).toBeDefined();
      expect(capPara?.runs[0].text).toContain("Figure 1");

      const tofBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-table-of-figures"]')!;
      expect(tofBtn).not.toBeNull();
      tofBtn.click();

      const tofPara = writerState.doc.pages[0].paragraphs.find((p) => p.id.startsWith("tof-"));
      expect(tofPara).toBeDefined();
      expect(tofPara?.runs[0].text).toContain("TABLE OF FIGURES");
    });
  });

  describe("2. Review Tab: Proofing, Speech, Comments, Track Changes & Protection", () => {
    it("handles proofing, thesaurus, language, and word count dialog", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Review");

      const spellingBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-spelling"]')!;
      expect(spellingBtn).not.toBeNull();
      spellingBtn.click();
      expect((writerState.doc as any).lastProofingAction).toBe("CheckSpelling");

      const thesaurusBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-thesaurus"]')!;
      expect(thesaurusBtn).not.toBeNull();
      thesaurusBtn.click();
      expect((writerState.doc as any).lastProofingAction).toBe("Thesaurus");

      const langBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-language"]')!;
      expect(langBtn).not.toBeNull();
      langBtn.click();
      expect((writerState.doc as any).lastProofingAction).toBe("SetLanguage");

      const translateBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-translate"]')!;
      expect(translateBtn).not.toBeNull();
      translateBtn.click();
      expect((writerState.doc as any).lastProofingAction).toBe("Translate");

      const wcBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-word-count"]')!;
      expect(wcBtn).not.toBeNull();
      wcBtn.click();
      expect(app.getWordCountDialog().isOpen()).toBe(true);
      app.getWordCountDialog().hide();
    });

    it("controls Read Aloud via Web Speech API (Play, Pause, Stop)", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Review");

      writerState.insertText("Audio document synthesis test.");

      const playBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-read-aloud-play"]')!;
      expect(playBtn).not.toBeNull();
      playBtn.click();
      expect(mockSpeechSynthesis.cancel).toHaveBeenCalled();
      expect(mockSpeechSynthesis.speak).toHaveBeenCalled();

      const pauseBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-read-aloud-pause"]')!;
      expect(pauseBtn).not.toBeNull();
      mockSpeechSynthesis.speaking = true;
      pauseBtn.click();
      expect(mockSpeechSynthesis.pause).toHaveBeenCalled();

      const stopBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-read-aloud-stop"]')!;
      expect(stopBtn).not.toBeNull();
      stopBtn.click();
      expect(mockSpeechSynthesis.cancel).toHaveBeenCalledTimes(2);
    });

    it("handles comments gutter rendering, navigation, and deletion", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Review");

      const newCommBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-new-comment"]')!;
      expect(newCommBtn).not.toBeNull();
      newCommBtn.click();

      expect(writerState.doc.comments?.length).toBe(1);
      const gutter = document.querySelector('[data-testid="word-comments-gutter"]');
      expect(gutter).not.toBeNull();
      const card = document.querySelector('[data-testid="word-comment-card"]');
      expect(card).not.toBeNull();

      const nextCommBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-next-comment"]')!;
      expect(nextCommBtn).not.toBeNull();
      nextCommBtn.click();
      expect((writerState as any).selectedCommentIndex).toBe(0);

      const prevCommBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-prev-comment"]')!;
      expect(prevCommBtn).not.toBeNull();
      prevCommBtn.click();
      expect((writerState as any).selectedCommentIndex).toBe(0);

      const delCommBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-delete-comment"]')!;
      expect(delCommBtn).not.toBeNull();
      delCommBtn.click();
      expect(writerState.doc.comments?.length).toBe(0);
    });

    it("toggles Track Changes and supports Accept & Reject changes", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Review");

      const trackBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-track-changes"]')!;
      expect(trackBtn).not.toBeNull();
      trackBtn.click();
      expect(writerState.doc.trackChanges).toBe(true);

      writerState.insertText("Tracked revision text");
      const currentPara = writerState.doc.pages[0].paragraphs[0];
      const insertedRun = currentPara.runs.find((r) => r.insertion);
      expect(insertedRun).toBeDefined();

      const acceptBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-accept-change"]')!;
      expect(acceptBtn).not.toBeNull();
      acceptBtn.click();
      expect(currentPara.runs.some((r) => r.insertion)).toBe(false);

      trackBtn.click();
      expect(writerState.doc.trackChanges).toBe(false);
    });

    it("locks document editing when Protect Document is activated", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Review");

      const protectBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-protect-doc"]')!;
      expect(protectBtn).not.toBeNull();
      protectBtn.click();

      expect(writerState.doc.readOnly).toBe(true);
      const bodyEl = document.querySelector<HTMLDivElement>('[data-testid="word-page-body"]')!;
      expect(bodyEl.contentEditable).toBe("false");

      // Click again to unlock
      protectBtn.click();
      expect(writerState.doc.readOnly).toBe(false);
      const unlockedBodyEl = document.querySelector<HTMLDivElement>('[data-testid="word-page-body"]')!;
      expect(unlockedBodyEl.contentEditable).toBe("true");
    });
  });

  describe("3. View Tab: Layout Modes, Focus, Gridlines, Dark Canvas & Zoom", () => {
    it("switches view modes and toggles Focus Mode", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("View");

      const webBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-view-web"]')!;
      expect(webBtn).not.toBeNull();
      webBtn.click();
      expect(writerState.doc.viewMode).toBe("web");

      const printBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-view-print"]')!;
      expect(printBtn).not.toBeNull();
      printBtn.click();
      expect(writerState.doc.viewMode).toBe("print");

      const focusBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-view-focus"]')!;
      expect(focusBtn).not.toBeNull();
      focusBtn.click();
      const ribbonBar = document.getElementById("ribbon-container")!;
      expect(ribbonBar.style.display).toBe("none");

      focusBtn.click();
      expect(ribbonBar.style.display).toBe("block");
    });

    it("toggles Ruler, Gridlines overlay, and Dark Page Canvas", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("View");

      const rulerBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-toggle-ruler"]')!;
      expect(rulerBtn).not.toBeNull();
      rulerBtn.click();
      const subHeader = document.getElementById("sub-header-container")!;
      expect(subHeader.style.display).toBe("none");

      const gridlinesBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-toggle-gridlines"]')!;
      expect(gridlinesBtn).not.toBeNull();
      gridlinesBtn.click();
      expect(writerState.doc.gridlines).toBe(true);
      const gridOverlay = document.querySelector('[data-testid="word-gridlines-overlay"]');
      expect(gridOverlay).not.toBeNull();

      const darkCanvasBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-toggle-dark-canvas"]')!;
      expect(darkCanvasBtn).not.toBeNull();
      darkCanvasBtn.click();
      expect(writerState.doc.darkCanvas).toBe(true);
      const pageEl = document.querySelector<HTMLDivElement>(".word-page")!;
      expect(["#1e293b", "rgb(30, 41, 59)"]).toContain(pageEl.style.backgroundColor);
    });

    it("applies Zoom levels (50%, 75%, 100%, 150%, 200%) and presets (One Page, Multi, Page Width)", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("View");

      const zoom50 = document.querySelector<HTMLButtonElement>('[data-testid="btn-zoom-50"]')!;
      zoom50.click();
      expect(writerState.zoom).toBe(0.5);
      expect(document.getElementById("status-zoom-level")?.textContent).toBe("50%");

      const zoom75 = document.querySelector<HTMLButtonElement>('[data-testid="btn-zoom-75"]')!;
      zoom75.click();
      expect(writerState.zoom).toBe(0.75);
      expect(document.getElementById("status-zoom-level")?.textContent).toBe("75%");

      const zoom150 = document.querySelector<HTMLButtonElement>('[data-testid="btn-zoom-150"]')!;
      zoom150.click();
      expect(writerState.zoom).toBe(1.5);
      expect(document.getElementById("status-zoom-level")?.textContent).toBe("150%");

      const zoom200 = document.querySelector<HTMLButtonElement>('[data-testid="btn-zoom-200"]')!;
      zoom200.click();
      expect(writerState.zoom).toBe(2.0);
      expect(document.getElementById("status-zoom-level")?.textContent).toBe("200%");

      const zoom100 = document.querySelector<HTMLButtonElement>('[data-testid="btn-zoom-100"]')!;
      zoom100.click();
      expect(writerState.zoom).toBe(1.0);
      expect(document.getElementById("status-zoom-level")?.textContent).toBe("100%");

      const zoomOne = document.querySelector<HTMLButtonElement>('[data-testid="btn-zoom-one"]')!;
      zoomOne.click();
      expect(writerState.zoom).toBe(0.85);

      const zoomMulti = document.querySelector<HTMLButtonElement>('[data-testid="btn-zoom-multiple"]')!;
      zoomMulti.click();
      expect(writerState.zoom).toBe(0.65);

      const zoomWidth = document.querySelector<HTMLButtonElement>('[data-testid="btn-zoom-width"]')!;
      zoomWidth.click();
      expect(writerState.zoom).toBe(1.25);
    });
  });
});
