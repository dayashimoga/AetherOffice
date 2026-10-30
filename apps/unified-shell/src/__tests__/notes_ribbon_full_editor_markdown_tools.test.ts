import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { NotesState, notesState } from "../state/notesState";
import { NotesRibbon } from "../components/ribbon/NotesRibbon";
import { NotesWorkspace } from "../apps/notes/NotesWorkspace";
import { UnifiedShellApp } from "../main";

describe("Notepad / Notes Complete Home, Edit, View, Markdown Ribbons & Text Power Tools (GEC-v4.0 Subphase 33.1.1)", () => {
  let state: NotesState;
  let ribbon: NotesRibbon;
  let ribbonBody: HTMLElement;
  let app: UnifiedShellApp;
  let actions: { action: string; payload?: unknown }[];

  beforeEach(() => {
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

    state = notesState;
    state.resetToBlank("TestNotes.txt");

    app = new UnifiedShellApp(false);
    app.getTabBar().addTab({
      id: "tab-notes-test",
      docType: "Notes",
      title: "TestNotes.txt",
      isDirty: false,
    });
    app.getTabBar().switchTab("tab-notes-test");
    (app as any).currentDocType = "Notes";

    ribbon = new NotesRibbon();
    ribbonBody = document.createElement("div");
    ribbonBody.className = "ribbon-body";
    document.body.appendChild(ribbonBody);

    actions = [];
  });

  afterEach(() => {
    document.body.innerHTML = "";
  });

  const renderAndTrack = (tab: string) => {
    ribbon.renderTab(tab, ribbonBody, (action, payload) => {
      actions.push({ action, payload });
      app.handleRibbonAction(action, payload);
    });
  };

  // =========================================================================
  // 1. HOME TAB (NOTES MANAGEMENT, TYPOGRAPHY & POWER TOOLS)
  // =========================================================================
  describe("1. Home Tab", () => {
    beforeEach(() => {
      renderAndTrack("Home");
    });

    it("1.1 triggers Notes File operations: New, Open, Save (.txt), Save (.md), Print", () => {
      const initialCount = state.notes.length;

      // New Note
      const newBtn = ribbonBody.querySelector("[data-testid='notes-btn-new']") as HTMLButtonElement;
      expect(newBtn).not.toBeNull();
      newBtn.click();
      expect(actions).toContainEqual({ action: "NewNote", payload: undefined });
      expect(state.notes.length).toBe(initialCount + 1);

      // Open Note
      const openBtn = ribbonBody.querySelector("[data-testid='notes-btn-open']") as HTMLButtonElement;
      expect(openBtn).not.toBeNull();
      openBtn.click();
      expect(actions).toContainEqual({ action: "OpenNote", payload: undefined });

      // Save Note (.txt)
      const saveTxtBtn = ribbonBody.querySelector("[data-testid='notes-btn-save-txt']") as HTMLButtonElement;
      expect(saveTxtBtn).not.toBeNull();
      saveTxtBtn.click();
      expect(actions).toContainEqual({ action: "SaveDocument", payload: undefined });

      // Save Note (.md)
      const saveMdBtn = ribbonBody.querySelector("[data-testid='notes-btn-save-md']") as HTMLButtonElement;
      expect(saveMdBtn).not.toBeNull();
      saveMdBtn.click();
      expect(actions).toContainEqual({ action: "SaveDocumentMd", payload: undefined });
      expect(state.title.endsWith(".md")).toBe(true);

      // Print
      const printBtn = ribbonBody.querySelector("[data-testid='notes-btn-print']") as HTMLButtonElement;
      expect(printBtn).not.toBeNull();
      printBtn.click();
      expect(actions).toContainEqual({ action: "PrintDocument", payload: undefined });
    });

    it("1.2 manages Typography: Font Family, Font Size, Word Wrap, Line Numbers", () => {
      // Font Family
      const fontSel = ribbonBody.querySelector("[data-testid='notes-select-font-family']") as HTMLSelectElement;
      expect(fontSel).not.toBeNull();
      fontSel.value = "Consolas";
      fontSel.dispatchEvent(new Event("change"));
      expect(actions).toContainEqual({ action: "SetFontFamily", payload: "Consolas" });
      expect(state.fontFamily).toBe("Consolas");

      // Font Size
      const sizeSel = ribbonBody.querySelector("[data-testid='notes-select-font-size']") as HTMLSelectElement;
      expect(sizeSel).not.toBeNull();
      sizeSel.value = "18";
      sizeSel.dispatchEvent(new Event("change"));
      expect(actions).toContainEqual({ action: "SetFontSize", payload: 18 });
      expect(state.fontSize).toBe(18);

      // Word Wrap
      const wrapBtn = ribbonBody.querySelector("[data-testid='notes-btn-word-wrap']") as HTMLButtonElement;
      expect(wrapBtn).not.toBeNull();
      const wrapBefore = state.wordWrap;
      wrapBtn.click();
      expect(actions).toContainEqual({ action: "ToggleWordWrap", payload: undefined });
      expect(state.wordWrap).toBe(!wrapBefore);

      // Line Numbers
      const lineNumBtn = ribbonBody.querySelector("[data-testid='notes-btn-line-numbers']") as HTMLButtonElement;
      expect(lineNumBtn).not.toBeNull();
      const linesBefore = state.showLineNumbers;
      lineNumBtn.click();
      expect(actions).toContainEqual({ action: "ToggleLineNumbers", payload: undefined });
      expect(state.showLineNumbers).toBe(!linesBefore);
    });

    it("1.3 executes Text Power Tools: Time/Date, Upper, Lower, Title, Sort A-Z, Sort Z-A, De-dup, Trim, Stats", () => {
      // Time/Date F5
      const timeBtn = ribbonBody.querySelector("[data-testid='notes-btn-timedate']") as HTMLButtonElement;
      expect(timeBtn).not.toBeNull();
      timeBtn.click();
      expect(actions).toContainEqual({ action: "InsertTimeDate", payload: undefined });
      expect(state.content.length).toBeGreaterThan(0);

      // Setup multiline content for transformations
      state.setContent("banana  \napple  \nbanana  \ncherry  ");

      // Uppercase
      const upperBtn = ribbonBody.querySelector("[data-testid='notes-btn-upper']") as HTMLButtonElement;
      expect(upperBtn).not.toBeNull();
      upperBtn.click();
      expect(actions).toContainEqual({ action: "TransformText", payload: "upper" });
      expect(state.content).toContain("BANANA");

      // Lowercase
      const lowerBtn = ribbonBody.querySelector("[data-testid='notes-btn-lower']") as HTMLButtonElement;
      expect(lowerBtn).not.toBeNull();
      lowerBtn.click();
      expect(actions).toContainEqual({ action: "TransformText", payload: "lower" });
      expect(state.content).toContain("banana");

      // Title Case
      const titleBtn = ribbonBody.querySelector("[data-testid='notes-btn-title-case']") as HTMLButtonElement;
      expect(titleBtn).not.toBeNull();
      titleBtn.click();
      expect(actions).toContainEqual({ action: "TransformText", payload: "title" });
      expect(state.content).toContain("Banana");

      // Sort Lines A-Z
      const sortAzBtn = ribbonBody.querySelector("[data-testid='notes-btn-sort-az']") as HTMLButtonElement;
      expect(sortAzBtn).not.toBeNull();
      sortAzBtn.click();
      expect(actions).toContainEqual({ action: "TransformText", payload: "sort_az" });
      expect(state.content.startsWith("Apple")).toBe(true);

      // Sort Lines Z-A
      const sortZaBtn = ribbonBody.querySelector("[data-testid='notes-btn-sort-za']") as HTMLButtonElement;
      expect(sortZaBtn).not.toBeNull();
      sortZaBtn.click();
      expect(actions).toContainEqual({ action: "TransformText", payload: "sort_za" });
      expect(state.content.startsWith("Cherry")).toBe(true);

      // Remove Duplicate Lines
      const dedupBtn = ribbonBody.querySelector("[data-testid='notes-btn-remove-duplicates']") as HTMLButtonElement;
      expect(dedupBtn).not.toBeNull();
      dedupBtn.click();
      expect(actions).toContainEqual({ action: "TransformText", payload: "remove_duplicates" });
      expect(state.content.split("\n").length).toBe(3);

      // Trim Trailing Whitespace
      const trimBtn = ribbonBody.querySelector("[data-testid='notes-btn-trim-whitespace']") as HTMLButtonElement;
      expect(trimBtn).not.toBeNull();
      trimBtn.click();
      expect(actions).toContainEqual({ action: "TransformText", payload: "trim_whitespace" });
      expect(state.content.endsWith(" ")).toBe(false);

      // Statistics
      const statsBtn = ribbonBody.querySelector("[data-testid='notes-btn-stats']") as HTMLButtonElement;
      expect(statsBtn).not.toBeNull();
      statsBtn.click();
      expect(actions).toContainEqual({ action: "ShowStatistics", payload: undefined });
    });
  });

  // =========================================================================
  // 2. EDIT TAB (HISTORY, CLIPBOARD & SEARCH/REPLACE)
  // =========================================================================
  describe("2. Edit Tab", () => {
    beforeEach(() => {
      renderAndTrack("Edit");
    });

    it("2.1 triggers Undo, Redo, Cut, Copy, Paste, Select All", () => {
      state.setContent("Initial text");
      state.setContent("Updated text");

      // Undo
      const undoBtn = ribbonBody.querySelector("[data-testid='notes-btn-undo']") as HTMLButtonElement;
      expect(undoBtn).not.toBeNull();
      undoBtn.click();
      expect(actions).toContainEqual({ action: "Undo", payload: undefined });
      expect(state.content).toBe("Initial text");

      // Redo
      const redoBtn = ribbonBody.querySelector("[data-testid='notes-btn-redo']") as HTMLButtonElement;
      expect(redoBtn).not.toBeNull();
      redoBtn.click();
      expect(actions).toContainEqual({ action: "Redo", payload: undefined });
      expect(state.content).toBe("Updated text");

      // Clipboard buttons
      const cutBtn = ribbonBody.querySelector("[data-testid='notes-btn-cut']") as HTMLButtonElement;
      const copyBtn = ribbonBody.querySelector("[data-testid='notes-btn-copy']") as HTMLButtonElement;
      const pasteBtn = ribbonBody.querySelector("[data-testid='notes-btn-paste']") as HTMLButtonElement;
      const selectAllBtn = ribbonBody.querySelector("[data-testid='notes-btn-select-all']") as HTMLButtonElement;

      expect(cutBtn).not.toBeNull();
      expect(copyBtn).not.toBeNull();
      expect(pasteBtn).not.toBeNull();
      expect(selectAllBtn).not.toBeNull();

      cutBtn.click();
      expect(actions).toContainEqual({ action: "Cut", payload: undefined });

      copyBtn.click();
      expect(actions).toContainEqual({ action: "Copy", payload: undefined });

      pasteBtn.click();
      expect(actions).toContainEqual({ action: "Paste", payload: undefined });

      selectAllBtn.click();
      expect(actions).toContainEqual({ action: "SelectAll", payload: undefined });
    });

    it("2.2 triggers Find, Replace, and Go To Line", () => {
      // Find
      const findBtn = ribbonBody.querySelector("[data-testid='notes-btn-find']") as HTMLButtonElement;
      expect(findBtn).not.toBeNull();
      findBtn.click();
      expect(actions).toContainEqual({ action: "Find", payload: undefined });
      expect(state.isFindBarVisible).toBe(true);

      // Replace
      const replaceBtn = ribbonBody.querySelector("[data-testid='notes-btn-replace']") as HTMLButtonElement;
      expect(replaceBtn).not.toBeNull();
      replaceBtn.click();
      expect(actions).toContainEqual({ action: "Replace", payload: undefined });
      expect(state.isReplaceBarVisible).toBe(true);

      // Go To Line
      const gotoBtn = ribbonBody.querySelector("[data-testid='notes-btn-goto-line']") as HTMLButtonElement;
      expect(gotoBtn).not.toBeNull();
      gotoBtn.click();
      expect(actions).toContainEqual({ action: "GoToLine", payload: undefined });
    });
  });

  // =========================================================================
  // 3. VIEW TAB (LAYOUT MODES, DISPLAY TOGGLES & ZOOM)
  // =========================================================================
  describe("3. View Tab", () => {
    beforeEach(() => {
      renderAndTrack("View");
    });

    it("3.1 toggles Editor Layout: Classic, Split, Preview", () => {
      const editorBtn = ribbonBody.querySelector("[data-testid='notes-btn-view-editor']") as HTMLButtonElement;
      const splitBtn = ribbonBody.querySelector("[data-testid='notes-btn-view-split']") as HTMLButtonElement;
      const previewBtn = ribbonBody.querySelector("[data-testid='notes-btn-view-preview']") as HTMLButtonElement;

      expect(editorBtn).not.toBeNull();
      expect(splitBtn).not.toBeNull();
      expect(previewBtn).not.toBeNull();

      editorBtn.click();
      expect(actions).toContainEqual({ action: "SetNotesViewMode", payload: "editor" });
      expect(state.viewMode).toBe("editor");

      splitBtn.click();
      expect(actions).toContainEqual({ action: "SetNotesViewMode", payload: "split" });
      expect(state.viewMode).toBe("split");

      previewBtn.click();
      expect(actions).toContainEqual({ action: "SetNotesViewMode", payload: "preview" });
      expect(state.viewMode).toBe("preview");
    });

    it("3.2 controls Zoom (In, Out, 100%) and View Display toggles", () => {
      const zoomInBtn = ribbonBody.querySelector("[data-testid='notes-btn-zoom-in']") as HTMLButtonElement;
      const zoomOutBtn = ribbonBody.querySelector("[data-testid='notes-btn-zoom-out']") as HTMLButtonElement;
      const zoom100Btn = ribbonBody.querySelector("[data-testid='notes-btn-zoom-100']") as HTMLButtonElement;

      expect(zoomInBtn).not.toBeNull();
      expect(zoomOutBtn).not.toBeNull();
      expect(zoom100Btn).not.toBeNull();

      zoomInBtn.click();
      expect(actions).toContainEqual({ action: "ZoomIn", payload: undefined });
      expect(state.zoom).toBe(110);

      zoomOutBtn.click();
      expect(actions).toContainEqual({ action: "ZoomOut", payload: undefined });
      expect(state.zoom).toBe(100);

      zoomInBtn.click();
      zoomInBtn.click();
      expect(state.zoom).toBe(120);

      zoom100Btn.click();
      expect(actions).toContainEqual({ action: "Zoom100", payload: undefined });
      expect(state.zoom).toBe(100);
    });
  });

  // =========================================================================
  // 4. MARKDOWN TAB (STRUCTURE & INLINE FORMATTING)
  // =========================================================================
  describe("4. Markdown Tab", () => {
    beforeEach(() => {
      renderAndTrack("Markdown");
    });

    it("4.1 inserts Structure snippets: H1, H2, H3, Blockquote, Horizontal Rule", () => {
      state.setContent("");

      const snippets = [
        { testId: "notes-btn-h1", kind: "h1", expected: "# Heading 1" },
        { testId: "notes-btn-h2", kind: "h2", expected: "## Heading 2" },
        { testId: "notes-btn-h3", kind: "h3", expected: "### Heading 3" },
        { testId: "notes-btn-quote", kind: "quote", expected: "> Important note" },
        { testId: "notes-btn-hr", kind: "hr", expected: "---" },
      ];

      for (const sn of snippets) {
        const btn = ribbonBody.querySelector(`[data-testid="${sn.testId}"]`) as HTMLButtonElement;
        expect(btn).not.toBeNull();
        btn.click();
        expect(actions).toContainEqual({ action: "InsertMarkdownSnippet", payload: sn.kind });
        expect(state.content).toContain(sn.expected);
      }
    });

    it("4.2 inserts Inline & Code snippets: Bold, Italic, Inline Code, Code Block, Checklist, Link, Table", () => {
      state.setContent("");

      const formatting = [
        { testId: "notes-btn-md-bold", kind: "bold", expected: "**bold text**" },
        { testId: "notes-btn-md-italic", kind: "italic", expected: "*italic text*" },
        { testId: "notes-btn-md-inline-code", kind: "inline_code", expected: "`code`" },
        { testId: "notes-btn-md-code", kind: "code", expected: "```ts" },
        { testId: "notes-btn-md-check", kind: "checklist", expected: "- [ ] Task item" },
        { testId: "notes-btn-md-link", kind: "link", expected: "[Link](https://example.com)" },
        { testId: "notes-btn-md-table", kind: "table", expected: "| Item | Description | Status |" },
      ];

      for (const fm of formatting) {
        const btn = ribbonBody.querySelector(`[data-testid="${fm.testId}"]`) as HTMLButtonElement;
        expect(btn).not.toBeNull();
        btn.click();
        expect(actions).toContainEqual({ action: "InsertMarkdownSnippet", payload: fm.kind });
        expect(state.content).toContain(fm.expected);
      }
    });
  });

  // =========================================================================
  // 5. WORKSPACE INTERACTIVITY: DOUBLE-CLICK RENAME & LIVE COUNTERS
  // =========================================================================
  describe("5. Workspace Interactivity & Status Bar", () => {
    it("5.1 supports double-click to inline rename scratchpad note", () => {
      const vp = document.getElementById("viewport-container")!;
      const ws = new NotesWorkspace(vp, state);
      ws.render();

      const card = vp.querySelector("[data-testid='note-card']") as HTMLElement;
      expect(card).not.toBeNull();

      const nameSpan = card.querySelector("span[title='Double-click to rename']") as HTMLElement;
      expect(nameSpan).not.toBeNull();

      // Trigger double-click
      nameSpan.dispatchEvent(new MouseEvent("dblclick", { bubbles: true }));

      const renameInput = card.querySelector("[data-testid='input-rename-scratchpad']") as HTMLInputElement;
      expect(renameInput).not.toBeNull();
      expect(renameInput.value).toBe("TestNotes.txt");

      // Change title and blur
      renameInput.value = "RenamedQuickNote.md";
      renameInput.dispatchEvent(new Event("blur"));

      expect(state.title).toBe("RenamedQuickNote.md");
    });

    it("5.2 updates status bar metrics live on editor changes", () => {
      const vp = document.getElementById("viewport-container")!;
      const ws = new NotesWorkspace(vp, state);
      ws.setCallbacks({
        onStatusChange: () => app.updateStatusBar(),
      });
      ws.render();

      const textarea = ws.getTextarea();
      textarea.value = "Hello world\nSecond line of text";
      textarea.selectionStart = textarea.selectionEnd = 15;
      textarea.dispatchEvent(new Event("input"));
      textarea.dispatchEvent(new Event("keyup"));

      const stats = state.getStats();
      expect(stats.lines).toBe(2);
      expect(stats.words).toBe(6);

      const docInfo = document.getElementById("status-doc-info");
      const wordCount = document.getElementById("status-word-count");

      expect(docInfo?.textContent).toContain("Ln 2, Col 4");
      expect(wordCount?.textContent).toContain("2 lines | 6 words");
    });
  });
});
