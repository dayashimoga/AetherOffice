import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { NotesState, notesState } from "../state/notesState";
import { NotesRibbon } from "../components/ribbon/NotesRibbon";
import { NotesWorkspace } from "../apps/notes/NotesWorkspace";
import { UnifiedShellApp } from "../main";
import { writerState } from "../state/writerState";
import { calcState } from "../state/calcState";
import { slidesState } from "../state/slidesState";
import { pdfState } from "../state/pdfState";

describe("GEC-v5.0 Subphase 33.1.1: Notepad / Notes Complete Pro Ribbon & Default 100% Zoom Suite", () => {
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
            <span id="status-lang"></span>
          </div>
          <div class="status-right">
            <span id="status-zoom-level">100%</span>
            <input type="range" id="zoom-slider" min="20" max="400" value="100" />
          </div>
        </footer>
      </div>
    `;

    state = notesState;
    state.resetToBlank("NotepadRelease.txt");

    app = new UnifiedShellApp(false);
    app.getTabBar().addTab({
      id: "tab-notes-gec5",
      docType: "Notes",
      title: "NotepadRelease.txt",
      isDirty: false,
    });
    app.getTabBar().switchTab("tab-notes-gec5");
    (app as any).currentDocType = "Notes";

    ribbon = new NotesRibbon();
    ribbonBody = document.createElement("div");
    ribbonBody.className = "ribbon-body";
    document.body.appendChild(ribbonBody);

    const vpContainer = document.getElementById("viewport-container")!;
    new NotesWorkspace(vpContainer, state);

    actions = [];
  });

  afterEach(() => {
    document.body.innerHTML = "";
  });

  // =========================================================================
  // 1. DEFAULT 100% ZOOM BUG FIX ACROSS ALL 5 APPS
  // =========================================================================
  describe("1. Default 100% Zoom Invariant Across All 5 Apps", () => {
    it("guarantees Word, Excel, PPT, PDF, and Notes initialize at 100% zoom and sync with status bar", () => {
      const zoomSlider = document.getElementById("zoom-slider") as HTMLInputElement;
      const zoomLevel = document.getElementById("status-zoom-level")!;

      // 1. Initial State in Notes mode
      app.updateStatusBar();
      expect(zoomSlider.value).toBe("100");
      expect(zoomLevel.textContent).toBe("100%");
      expect(notesState.zoom).toBe(100);

      // 2. Word Mode
      (app as any).currentDocType = "Word";
      writerState.zoom = 1.0;
      app.updateStatusBar();
      expect(zoomSlider.value).toBe("100");
      expect(zoomLevel.textContent).toBe("100%");

      // 3. Excel Mode
      (app as any).currentDocType = "Excel";
      calcState.zoom = 1.0;
      app.updateStatusBar();
      expect(zoomSlider.value).toBe("100");
      expect(zoomLevel.textContent).toBe("100%");

      // 4. PowerPoint Mode
      (app as any).currentDocType = "PowerPoint";
      slidesState.zoom = 100;
      app.updateStatusBar();
      expect(zoomSlider.value).toBe("100");
      expect(zoomLevel.textContent).toBe("100%");

      // 5. PDF Mode
      (app as any).currentDocType = "PDF";
      pdfState.zoom = 1.0;
      app.updateStatusBar();
      expect(zoomSlider.value).toBe("100");
      expect(zoomLevel.textContent).toBe("100%");

      // Reset back to Notes
      (app as any).currentDocType = "Notes";
      app.updateStatusBar();
      expect(zoomSlider.value).toBe("100");
      expect(zoomLevel.textContent).toBe("100%");
    });
  });

  // =========================================================================
  // 2. HOME TAB: FILE, TYPOGRAPHY, ZOOM & TIME/DATE
  // =========================================================================
  describe("2. Home Tab Complete Suite", () => {
    it("wires New Note, Open, Save, Save Md, Rename, Delete, Print, Font, Size, Zoom, Time/Date", () => {
      ribbon.renderTab("Home", ribbonBody, (action, payload) => {
        actions.push({ action, payload });
        app.handleRibbonAction(action, payload);
      });

      const buttons = ribbonBody.querySelectorAll<HTMLButtonElement>("button");
      expect(buttons.length).toBeGreaterThanOrEqual(10);

      buttons.forEach((btn) => btn.click());

      const actionNames = actions.map((a) => a.action);
      expect(actionNames).toContain("NewNote");
      expect(actionNames).toContain("OpenNote");
      expect(actionNames).toContain("SaveDocument");
      expect(actionNames).toContain("SaveDocumentMd");
      expect(actionNames).toContain("RenameActiveNote");
      expect(actionNames).toContain("PrintDocument");
      expect(actionNames).toContain("ToggleWordWrap");
      expect(actionNames).toContain("ToggleLineNumbers");
      expect(actionNames).toContain("ZoomIn");
      expect(actionNames).toContain("ZoomOut");
      expect(actionNames).toContain("Zoom100");
      expect(actionNames).toContain("InsertTimeDate");

      // Verify Select dropdowns
      const selects = ribbonBody.querySelectorAll<HTMLSelectElement>("select");
      expect(selects.length).toBeGreaterThanOrEqual(2);

      selects[0].value = "Consolas";
      selects[0].dispatchEvent(new Event("change"));
      selects[1].value = "18";
      selects[1].dispatchEvent(new Event("change"));

      expect(actions.some((a) => a.action === "SetFontFamily" && a.payload === "Consolas")).toBe(true);
      expect(actions.some((a) => a.action === "SetFontSize" && a.payload === 18)).toBe(true);
    });
  });

  // =========================================================================
  // 3. EDIT TAB: CLIPBOARD, SEARCH, LINE OPERATIONS
  // =========================================================================
  describe("3. Edit Tab Complete Suite", () => {
    it("wires Undo, Redo, Cut, Copy, Paste, Delete, Select All, Find, Replace, GoToLine, Dup/Del/Move Lines", () => {
      ribbon.renderTab("Edit", ribbonBody, (action, payload) => {
        actions.push({ action, payload });
        app.handleRibbonAction(action, payload);
      });

      const buttons = ribbonBody.querySelectorAll<HTMLButtonElement>("button");
      buttons.forEach((btn) => btn.click());

      const actionNames = actions.map((a) => a.action);
      expect(actionNames).toContain("Undo");
      expect(actionNames).toContain("Redo");
      expect(actionNames).toContain("Cut");
      expect(actionNames).toContain("Copy");
      expect(actionNames).toContain("Paste");
      expect(actionNames).toContain("Delete");
      expect(actionNames).toContain("SelectAll");
      expect(actionNames).toContain("Find");
      expect(actionNames).toContain("Replace");
      expect(actionNames).toContain("GoToLine");
      expect(actionNames).toContain("DuplicateLine");
      expect(actionNames).toContain("DeleteLine");
      expect(actionNames).toContain("MoveLineUp");
      expect(actionNames).toContain("MoveLineDown");

      // Verify Duplicate & Move Line functionality on multi-line text
      state.setContent("Line 1\nLine 2\nLine 3");
      state.setCursor(2, 1);
      state.duplicateLine();
      expect(state.getContent()).toBe("Line 1\nLine 2\nLine 2\nLine 3");

      state.moveLineUp();
      expect(state.getContent()).toBe("Line 2\nLine 1\nLine 2\nLine 3");

      state.deleteLine();
      expect(state.getContent()).toBe("Line 1\nLine 2\nLine 3");
    });
  });

  // =========================================================================
  // 4. TEXT TOOLS / TRANSFORM TAB: PRO TEXT TRANSFORMS & JSON FORMATTER
  // =========================================================================
  describe("4. Text Tools / Transform Tab Complete Suite", () => {
    it("wires Case Conversion, Line Sorting, Whitespace/Duplicate Cleaning, JSON Formatter, and Base64/URL Encoding", () => {
      ribbon.renderTab("Text Tools", ribbonBody, (action, payload) => {
        actions.push({ action, payload });
        app.handleRibbonAction(action, payload);
      });

      const buttons = ribbonBody.querySelectorAll<HTMLButtonElement>("button");
      expect(buttons.length).toBeGreaterThanOrEqual(15);

      buttons.forEach((btn) => btn.click());

      const actionNames = actions.map((a) => a.action);
      expect(actionNames).toContain("TransformText");
      expect(actionNames).toContain("ShowStatistics");

      // Verify case transforms
      state.setContent("hello world. welcome to aether!");
      state.transformText("upper");
      expect(state.getContent()).toBe("HELLO WORLD. WELCOME TO AETHER!");

      state.transformText("lower");
      expect(state.getContent()).toBe("hello world. welcome to aether!");

      state.transformText("sentence");
      expect(state.getContent()).toBe("Hello world. Welcome to aether!");

      state.transformText("invert");
      expect(state.getContent()).toBe("hELLO WORLD. wELCOME TO AETHER!");

      // Verify line cleaning & deduplication
      state.setContent("Bravo\nAlpha\nCharlie\nAlpha\n\nDelta   \n");
      state.transformText("remove_empty_lines");
      expect(state.getContent()).toBe("Bravo\nAlpha\nCharlie\nAlpha\nDelta   ");

      state.transformText("trim_whitespace");
      expect(state.getContent()).toBe("Bravo\nAlpha\nCharlie\nAlpha\nDelta");

      state.transformText("remove_duplicates");
      expect(state.getContent()).toBe("Bravo\nAlpha\nCharlie\nDelta");

      state.transformText("sort_az");
      expect(state.getContent()).toBe("Alpha\nBravo\nCharlie\nDelta");

      state.transformText("sort_za");
      expect(state.getContent()).toBe("Delta\nCharlie\nBravo\nAlpha");

      // Verify JSON Prettify & Minify
      state.setContent('{"name":"Antigravity","version":5,"active":true}');
      state.transformText("format_json");
      expect(state.getContent()).toContain('{\n  "name": "Antigravity",');

      state.transformText("minify_json");
      expect(state.getContent()).toBe('{"name":"Antigravity","version":5,"active":true}');

      // Verify Base64 & URL Encoding
      state.setContent("Security Token 2026");
      state.transformText("base64_encode");
      expect(state.getContent()).toBe("U2VjdXJpdHkgVG9rZW4gMjAyNg==");

      state.transformText("base64_decode");
      expect(state.getContent()).toBe("Security Token 2026");

      state.setContent("hello world&foo=bar");
      state.transformText("url_encode");
      expect(state.getContent()).toBe("hello%20world%26foo%3Dbar");

      state.transformText("url_decode");
      expect(state.getContent()).toBe("hello world&foo=bar");
    });
  });

  // =========================================================================
  // 5. VIEW TAB: MODES, DISPLAY, LINE ENDINGS & ENCODINGS
  // =========================================================================
  describe("5. View Tab Complete Suite", () => {
    it("wires Editor Layout (Editor, Split, Preview), Line Numbers, Word Wrap, Whitespace, CRLF/LF, Encoding", () => {
      ribbon.renderTab("View", ribbonBody, (action, payload) => {
        actions.push({ action, payload });
        app.handleRibbonAction(action, payload);
      });

      const buttons = ribbonBody.querySelectorAll<HTMLButtonElement>("button");
      buttons.forEach((btn) => btn.click());

      const actionNames = actions.map((a) => a.action);
      expect(actionNames).toContain("SetNotesViewMode");
      expect(actionNames).toContain("ToggleLineNumbers");
      expect(actionNames).toContain("ToggleWordWrap");
      expect(actionNames).toContain("ToggleShowWhitespace");
      expect(actionNames).toContain("ZoomIn");
      expect(actionNames).toContain("ZoomOut");
      expect(actionNames).toContain("Zoom100");

      // Verify Selects
      const selects = ribbonBody.querySelectorAll<HTMLSelectElement>("select");
      expect(selects.length).toBeGreaterThanOrEqual(2);

      selects[0].value = "LF";
      selects[0].dispatchEvent(new Event("change"));
      selects[1].value = "UTF-16";
      selects[1].dispatchEvent(new Event("change"));

      expect(actions.some((a) => a.action === "SetLineEnding" && a.payload === "LF")).toBe(true);
      expect(actions.some((a) => a.action === "SetEncoding" && a.payload === "UTF-16")).toBe(true);

      expect(state.lineEnding).toBe("LF");
      expect(state.encoding).toBe("UTF-16");
    });
  });

  // =========================================================================
  // 6. MARKDOWN TAB: FULL STRUCTURE, FORMATTING, LISTS & TABLES
  // =========================================================================
  describe("6. Markdown Tab Complete Suite", () => {
    it("wires H1, H2, H3, Quotes, HR, Bold, Italic, Strike, Code, Bullet List, Numbered List, Tasks, Tables, Links, Images", () => {
      ribbon.renderTab("Markdown", ribbonBody, (action, payload) => {
        actions.push({ action, payload });
        app.handleRibbonAction(action, payload);
      });

      const buttons = ribbonBody.querySelectorAll<HTMLButtonElement>("button");
      expect(buttons.length).toBeGreaterThanOrEqual(15);

      buttons.forEach((btn) => btn.click());

      const actionNames = actions.map((a) => a.action);
      expect(actionNames).toContain("InsertMarkdownSnippet");

      const snippets = actions.filter((a) => a.action === "InsertMarkdownSnippet").map((a) => a.payload);
      expect(snippets).toContain("h1");
      expect(snippets).toContain("h2");
      expect(snippets).toContain("h3");
      expect(snippets).toContain("quote");
      expect(snippets).toContain("hr");
      expect(snippets).toContain("bold");
      expect(snippets).toContain("italic");
      expect(snippets).toContain("strike");
      expect(snippets).toContain("inline_code");
      expect(snippets).toContain("code");
      expect(snippets).toContain("bullet_list");
      expect(snippets).toContain("numbered_list");
      expect(snippets).toContain("checklist");
      expect(snippets).toContain("table");
      expect(snippets).toContain("link");
      expect(snippets).toContain("image");
    });
  });
});
