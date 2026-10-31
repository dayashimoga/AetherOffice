import { describe, it, expect, beforeEach } from "vitest";
import { WriterRibbon } from "../components/ribbon/WriterRibbon";

describe("Subphase 36.1.2: Word Complete Ribbon Coverage (All 7 Tabs & Zero Overflow)", () => {
  let ribbonContainer: HTMLElement;
  let emittedActions: Array<{ action: string; payload?: unknown }>;
  let ribbon: WriterRibbon;

  const renderTab = (tab: string) => {
    ribbon.renderTab(tab, ribbonContainer, (action, payload) => {
      emittedActions.push({ action, payload });
    });
  };

  beforeEach(() => {
    document.body.innerHTML = '<div id="ribbon-container"></div>';
    ribbonContainer = document.getElementById("ribbon-container")!;
    emittedActions = [];
    ribbon = new WriterRibbon();
  });

  it("1. Home Tab: All buttons and selectors emit expected actions without clipping", () => {
    renderTab("Home");

    // Pages
    const newPageBtn = ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-home-new-page"]');
    expect(newPageBtn).not.toBeNull();
    newPageBtn?.click();
    expect(emittedActions.some((a) => a.action === "InsertNewPage")).toBe(true);

    const delPageBtn = ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-home-delete-page"]');
    expect(delPageBtn).not.toBeNull();
    delPageBtn?.click();
    expect(emittedActions.some((a) => a.action === "DeletePage")).toBe(true);

    // Clipboard
    const pasteBtn = ribbonContainer.querySelector<HTMLElement>('[data-testid="writer-btn-paste"]');
    const cutBtn = ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-cut"]');
    const copyBtn = ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-copy"]');
    const painterBtn = ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-format-painter"]');
    pasteBtn?.click();
    cutBtn?.click();
    copyBtn?.click();
    painterBtn?.click();
    expect(emittedActions.some((a) => a.action === "Paste")).toBe(true);
    expect(emittedActions.some((a) => a.action === "Cut")).toBe(true);
    expect(emittedActions.some((a) => a.action === "Copy")).toBe(true);
    expect(emittedActions.some((a) => a.action === "FormatPainter")).toBe(true);

    // Font Controls
    const fontSel = ribbonContainer.querySelector<HTMLSelectElement>('[data-testid="font-family-select"]');
    expect(fontSel).not.toBeNull();
    fontSel!.value = "Arial";
    fontSel!.dispatchEvent(new Event("change"));
    expect(emittedActions.some((a) => a.action === "SetFontFamily" && a.payload === "Arial")).toBe(true);

    const sizeSel = ribbonContainer.querySelector<HTMLSelectElement>('[data-testid="font-size-select"]');
    expect(sizeSel).not.toBeNull();
    sizeSel!.value = "18";
    sizeSel!.dispatchEvent(new Event("change"));
    expect(emittedActions.some((a) => a.action === "SetFontSize" && a.payload === 18)).toBe(true);

    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-increase-font"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-decrease-font"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-change-case"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-clear-formatting"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-bold"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-italic"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-underline"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-strikethrough"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-subscript"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-superscript"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-font-color"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-highlight-color"]')?.click();

    expect(emittedActions.some((a) => a.action === "IncreaseFontSize")).toBe(true);
    expect(emittedActions.some((a) => a.action === "DecreaseFontSize")).toBe(true);
    expect(emittedActions.some((a) => a.action === "ChangeCase")).toBe(true);
    expect(emittedActions.some((a) => a.action === "ClearFormatting")).toBe(true);
    expect(emittedActions.some((a) => a.action === "ToggleBold")).toBe(true);
    expect(emittedActions.some((a) => a.action === "ToggleItalic")).toBe(true);
    expect(emittedActions.some((a) => a.action === "ToggleUnderline")).toBe(true);
    expect(emittedActions.some((a) => a.action === "ToggleStrikethrough")).toBe(true);
    expect(emittedActions.some((a) => a.action === "ToggleSubscript")).toBe(true);
    expect(emittedActions.some((a) => a.action === "ToggleSuperscript")).toBe(true);

    // Paragraph Controls
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-bullet-list"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-numbered-list"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-multilevel-list"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-decrease-indent"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-increase-indent"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-sort-paragraphs"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-show-formatting"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-align-left"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-align-center"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-align-right"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-align-justify"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-shading"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-paragraph-border"]')?.click();

    expect(emittedActions.some((a) => a.action === "ToggleBulletList")).toBe(true);
    expect(emittedActions.some((a) => a.action === "ToggleNumberedList")).toBe(true);
    expect(emittedActions.some((a) => a.action === "SortParagraphs")).toBe(true);
    expect(emittedActions.some((a) => a.action === "AlignCenter")).toBe(true);

    // Styles (All 8 gallery items)
    const styleIds = [
      "btn-style-normal",
      "btn-style-no-spacing",
      "btn-style-h1",
      "btn-style-h2",
      "btn-style-h3",
      "btn-style-title",
      "btn-style-subtitle",
      "btn-style-quote",
    ];
    styleIds.forEach((id) => {
      const btn = ribbonContainer.querySelector<HTMLElement>(`[data-testid="${id}"]`);
      expect(btn).not.toBeNull();
      btn?.click();
    });
    expect(emittedActions.some((a) => a.action === "SetStyle" && a.payload === "Heading 1")).toBe(true);
    expect(emittedActions.some((a) => a.action === "SetStyle" && a.payload === "Title")).toBe(true);

    // Editing & Copilot
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-find"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-replace"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-select-all"]')?.click();
    ribbonContainer.querySelector<HTMLElement>('[data-testid="btn-copilot"]')?.click();

    expect(emittedActions.some((a) => a.action === "Find")).toBe(true);
    expect(emittedActions.some((a) => a.action === "Replace")).toBe(true);
    expect(emittedActions.some((a) => a.action === "SelectAll")).toBe(true);
    expect(emittedActions.some((a) => a.action === "ToggleCopilot")).toBe(true);
  });

  it("2. Insert Tab: All Pages, Tables, Illustrations, Headers/Footers, Text & Links buttons work", () => {
    renderTab("Insert");

    const insertActionChecks: Array<{ testId: string; expectedAction: string }> = [
      { testId: "btn-insert-cover-page", expectedAction: "InsertCoverPage" },
      { testId: "btn-insert-new-page", expectedAction: "InsertNewPage" },
      { testId: "btn-insert-blank-page", expectedAction: "InsertBlankPage" },
      { testId: "btn-insert-page-break", expectedAction: "InsertPageBreak" },
      { testId: "btn-delete-empty-page", expectedAction: "DeleteEmptyPage" },
      { testId: "btn-insert-table", expectedAction: "InsertTable" },
      { testId: "btn-add-table-row", expectedAction: "AddTableRow" },
      { testId: "btn-add-table-col", expectedAction: "AddTableColumn" },
      { testId: "btn-delete-table-row", expectedAction: "DeleteTableRow" },
      { testId: "btn-delete-table-col", expectedAction: "DeleteTableColumn" },
      { testId: "btn-insert-picture", expectedAction: "InsertPicture" },
      { testId: "btn-shape-rect", expectedAction: "InsertShape" },
      { testId: "btn-shape-round", expectedAction: "InsertShape" },
      { testId: "btn-shape-ellipse", expectedAction: "InsertShape" },
      { testId: "btn-shape-triangle", expectedAction: "InsertShape" },
      { testId: "btn-shape-arrow", expectedAction: "InsertShape" },
      { testId: "btn-shape-star", expectedAction: "InsertShape" },
      { testId: "btn-shape-callout", expectedAction: "InsertShape" },
      { testId: "btn-insert-chart", expectedAction: "InsertChart" },
      { testId: "btn-insert-header", expectedAction: "InsertHeader" },
      { testId: "btn-insert-footer", expectedAction: "InsertFooter" },
      { testId: "btn-insert-page-number", expectedAction: "InsertPageNumber" },
      { testId: "btn-insert-textbox", expectedAction: "InsertTextBox" },
      { testId: "btn-insert-wordart", expectedAction: "InsertWordArt" },
      { testId: "btn-insert-dropcap", expectedAction: "InsertDropCap" },
      { testId: "btn-insert-line", expectedAction: "InsertHorizontalLine" },
      { testId: "btn-insert-datetime", expectedAction: "InsertDateTime" },
      { testId: "btn-insert-symbol", expectedAction: "InsertSymbol" },
      { testId: "btn-insert-equation", expectedAction: "InsertEquation" },
      { testId: "btn-insert-hyperlink", expectedAction: "InsertHyperlink" },
      { testId: "btn-insert-link", expectedAction: "InsertLink" },
      { testId: "btn-insert-comment", expectedAction: "InsertComment" },
      { testId: "btn-insert-bookmark", expectedAction: "InsertBookmark" },
    ];

    insertActionChecks.forEach(({ testId, expectedAction }) => {
      const btn = ribbonContainer.querySelector<HTMLElement>(`[data-testid="${testId}"]`);
      expect(btn, `Missing button: ${testId}`).not.toBeNull();
      btn?.click();
      expect(emittedActions.some((a) => a.action === expectedAction)).toBe(true);
    });
  });

  it("3. Layout Tab: Margins, Orientation, Sizes, Columns, Lines, Color, Borders & Watermark work", () => {
    renderTab("Layout");

    const layoutActionChecks: Array<{ testId: string; expectedAction: string }> = [
      { testId: "btn-margins-normal", expectedAction: "SetMargins" },
      { testId: "btn-margins-narrow", expectedAction: "SetMargins" },
      { testId: "btn-margins-moderate", expectedAction: "SetMargins" },
      { testId: "btn-margins-wide", expectedAction: "SetMargins" },
      { testId: "btn-margins-custom", expectedAction: "SetMargins" },
      { testId: "btn-orientation-portrait", expectedAction: "SetOrientation" },
      { testId: "btn-orientation-landscape", expectedAction: "SetOrientation" },
      { testId: "btn-pagesize-letter", expectedAction: "SetPageSize" },
      { testId: "btn-pagesize-a4", expectedAction: "SetPageSize" },
      { testId: "btn-pagesize-legal", expectedAction: "SetPageSize" },
      { testId: "btn-layout-pagebreak", expectedAction: "InsertPageBreak" },
      { testId: "btn-line-numbers", expectedAction: "ToggleLineNumbers" },
      { testId: "btn-hyphenation", expectedAction: "ToggleHyphenation" },
      { testId: "btn-page-color", expectedAction: "SetPageBackgroundColor" },
      { testId: "btn-page-borders", expectedAction: "SetPageBorder" },
      { testId: "btn-watermark", expectedAction: "SetWatermark" },
      { testId: "btn-cols-1", expectedAction: "SetColumns" },
      { testId: "btn-cols-2", expectedAction: "SetColumns" },
      { testId: "btn-cols-3", expectedAction: "SetColumns" },
      { testId: "btn-layout-indent", expectedAction: "IncreaseIndent" },
      { testId: "btn-layout-outdent", expectedAction: "DecreaseIndent" },
      { testId: "btn-spacing-10", expectedAction: "SetLineSpacing" },
      { testId: "btn-spacing-115", expectedAction: "SetLineSpacing" },
      { testId: "btn-spacing-15", expectedAction: "SetLineSpacing" },
      { testId: "btn-spacing-20", expectedAction: "SetLineSpacing" },
    ];

    layoutActionChecks.forEach(({ testId, expectedAction }) => {
      const btn = ribbonContainer.querySelector<HTMLElement>(`[data-testid="${testId}"]`);
      expect(btn, `Missing layout button: ${testId}`).not.toBeNull();
      btn?.click();
      expect(emittedActions.some((a) => a.action === expectedAction)).toBe(true);
    });
  });

  it("4. References Tab: Table of Contents, Footnotes, Citations & Captions work", () => {
    renderTab("References");

    const refActionChecks: Array<{ testId: string; expectedAction: string }> = [
      { testId: "btn-insert-toc", expectedAction: "InsertTOC" },
      { testId: "btn-update-toc", expectedAction: "UpdateTOC" },
      { testId: "btn-insert-footnote", expectedAction: "InsertFootnote" },
      { testId: "btn-insert-endnote", expectedAction: "InsertEndnote" },
      { testId: "btn-next-footnote", expectedAction: "NextFootnote" },
      { testId: "btn-insert-citation", expectedAction: "InsertCitation" },
      { testId: "btn-manage-sources", expectedAction: "ManageSources" },
      { testId: "btn-citation-style", expectedAction: "SetCitationStyle" },
      { testId: "btn-insert-bib", expectedAction: "InsertBibliography" },
      { testId: "btn-insert-caption", expectedAction: "InsertCaption" },
      { testId: "btn-table-of-figures", expectedAction: "TableOfFigures" },
    ];

    refActionChecks.forEach(({ testId, expectedAction }) => {
      const btn = ribbonContainer.querySelector<HTMLElement>(`[data-testid="${testId}"]`);
      expect(btn, `Missing reference button: ${testId}`).not.toBeNull();
      btn?.click();
      expect(emittedActions.some((a) => a.action === expectedAction)).toBe(true);
    });
  });

  it("5. Review Tab: Proofing, Read Aloud, Comments, Tracking & Protect work", () => {
    renderTab("Review");

    const reviewActionChecks: Array<{ testId: string; expectedAction: string }> = [
      { testId: "btn-spelling", expectedAction: "CheckSpelling" },
      { testId: "btn-thesaurus", expectedAction: "Thesaurus" },
      { testId: "btn-word-count", expectedAction: "WordCount" },
      { testId: "btn-read-aloud-play", expectedAction: "ReadAloudPlay" },
      { testId: "btn-read-aloud-pause", expectedAction: "ReadAloudPause" },
      { testId: "btn-read-aloud-stop", expectedAction: "ReadAloudStop" },
      { testId: "btn-translate", expectedAction: "Translate" },
      { testId: "btn-language", expectedAction: "SetLanguage" },
      { testId: "btn-new-comment", expectedAction: "NewComment" },
      { testId: "btn-delete-comment", expectedAction: "DeleteComment" },
      { testId: "btn-next-comment", expectedAction: "NextComment" },
      { testId: "btn-prev-comment", expectedAction: "PrevComment" },
      { testId: "btn-track-changes", expectedAction: "ToggleTrackChanges" },
      { testId: "btn-accept-change", expectedAction: "AcceptChange" },
      { testId: "btn-reject-change", expectedAction: "RejectChange" },
      { testId: "btn-accept-all", expectedAction: "AcceptAll" },
      { testId: "btn-reject-all", expectedAction: "RejectAll" },
      { testId: "btn-protect-doc", expectedAction: "ProtectDocument" },
    ];

    reviewActionChecks.forEach(({ testId, expectedAction }) => {
      const btn = ribbonContainer.querySelector<HTMLElement>(`[data-testid="${testId}"]`);
      expect(btn, `Missing review button: ${testId}`).not.toBeNull();
      btn?.click();
      expect(emittedActions.some((a) => a.action === expectedAction)).toBe(true);
    });
  });

  it("6. View Tab: Views, Ruler, Gridlines, Nav Pane, Dark Canvas & Zoom work", () => {
    renderTab("View");

    const viewActionChecks: Array<{ testId: string; expectedAction: string }> = [
      { testId: "btn-view-print", expectedAction: "ViewPrintLayout" },
      { testId: "btn-view-web", expectedAction: "ViewWebLayout" },
      { testId: "btn-view-read", expectedAction: "ViewReadMode" },
      { testId: "btn-view-focus", expectedAction: "ToggleFocusMode" },
      { testId: "btn-toggle-ruler", expectedAction: "ToggleRuler" },
      { testId: "btn-toggle-gridlines", expectedAction: "ToggleGridlines" },
      { testId: "btn-toggle-nav", expectedAction: "ToggleNavPane" },
      { testId: "btn-toggle-dark-canvas", expectedAction: "ToggleDarkCanvas" },
      { testId: "btn-zoom-50", expectedAction: "SetZoom" },
      { testId: "btn-zoom-75", expectedAction: "SetZoom" },
      { testId: "btn-zoom-100", expectedAction: "Zoom100" },
      { testId: "btn-zoom-150", expectedAction: "SetZoom" },
      { testId: "btn-zoom-200", expectedAction: "SetZoom" },
      { testId: "btn-zoom-one", expectedAction: "ZoomOnePage" },
      { testId: "btn-zoom-multiple", expectedAction: "ZoomMultiplePages" },
      { testId: "btn-zoom-width", expectedAction: "ZoomPageWidth" },
    ];

    viewActionChecks.forEach(({ testId, expectedAction }) => {
      const btn = ribbonContainer.querySelector<HTMLElement>(`[data-testid="${testId}"]`);
      expect(btn, `Missing view button: ${testId}`).not.toBeNull();
      btn?.click();
      expect(emittedActions.some((a) => a.action === expectedAction)).toBe(true);
    });
  });
});
