import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { NotesState, notesState } from "../state/notesState";
import { NotesWorkspace } from "../apps/notes/NotesWorkspace";
import { UnifiedShellApp } from "../main";

describe("Ultra-Fast Notepad & Markdown Notes Editor (.txt / .md) with Live Preview (Subphase 26.1.1)", () => {
  let state: NotesState;
  let container: HTMLElement;
  let workspace: NotesWorkspace;

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

    state = new NotesState();
    state.resetToBlank("Untitled 1.txt");
    container = document.getElementById("viewport-container")!;
    workspace = new NotesWorkspace(container, state);
  });

  afterEach(() => {
    document.body.innerHTML = "";
  });

  describe("1. Classic Notepad Mode: Blank Text Editor, Line Numbers & Typography", () => {
    it("starts with a 100% blank note under GEC-v3.0 Article II", () => {
      expect(state.content).toBe("");
      expect(state.notes.length).toBe(1);
      expect(state.title).toBe("Untitled 1.txt");

      const textarea = workspace.getTextarea();
      expect(textarea.value).toBe("");

      const stats = state.getStats();
      expect(stats.lines).toBe(1);
      expect(stats.words).toBe(0);
      expect(stats.chars).toBe(0);
    });

    it("renders line numbers gutter synchronized with textarea lines", () => {
      const textarea = workspace.getTextarea();
      textarea.value = "Line 1\nLine 2\nLine 3\nLine 4";
      textarea.dispatchEvent(new Event("input"));

      const gutter = container.querySelector('[data-testid="notes-line-gutter"]') as HTMLElement;
      expect(gutter).not.toBeNull();
      expect(gutter.textContent).toBe("1\n2\n3\n4");

      const stats = state.getStats();
      expect(stats.lines).toBe(4);
      expect(stats.words).toBe(8);
      expect(stats.chars).toBe(27);
    });

    it("toggles line numbers and word wrap modes", () => {
      expect(state.showLineNumbers).toBe(true);
      expect(state.wordWrap).toBe(true);

      state.execCommand("ToggleLineNumbers");
      expect(state.showLineNumbers).toBe(false);

      state.execCommand("ToggleWordWrap");
      expect(state.wordWrap).toBe(false);

      workspace.render();
      const gutter = container.querySelector('[data-testid="notes-line-gutter"]') as HTMLElement;
      expect(gutter.style.display).toBe("none");

      const textarea = workspace.getTextarea();
      expect(textarea.style.whiteSpace).toBe("pre");
    });

    it("toggles monospace and sans typography with custom font size", () => {
      expect(state.fontFamily).toBe("monospace");
      state.execCommand("SetFontFamily", "sans");
      expect(state.fontFamily).toBe("sans");

      state.execCommand("SetFontSize", 18);
      expect(state.fontSize).toBe(18);

      workspace.render();
      const textarea = workspace.getTextarea();
      expect(textarea.style.fontSize).toBe("18px");
      expect(textarea.style.fontFamily).toContain("sans-serif");
    });

    it("inserts F5 Time and Date stamp into note", () => {
      state.setContent("Meeting Log:\n");
      const inserted = state.insertTimeDate();
      expect(inserted).toBeDefined();
      expect(state.content).toContain("Meeting Log:");
      expect(state.content).toContain(inserted);
    });

    it("performs case transformation, line sorting, and regex find/replace", () => {
      state.setContent("zebra\napple\nmango");
      state.transformText("sort_lines");
      expect(state.content).toBe("apple\nmango\nzebra");

      state.transformText("upper");
      expect(state.content).toBe("APPLE\nMANGO\nZEBRA");

      const replacedCount = state.replaceText("APPLE", "PEAR");
      expect(replacedCount).toBe(1);
      expect(state.content).toContain("PEAR");
    });
  });

  describe("2. Rich Markdown Notes Mode with Live Preview & Interactive Checklists", () => {
    it("renders headings, formatting, code blocks, and tables in Markdown preview", () => {
      state.setViewMode("split");
      const mdContent = `
# Project Specs
## Overview
This is **bold text** and *italic text* with ~~deleted text~~.
\`\`\`ts
const x = 42;
\`\`\`
| Feature | Status |
|---|---|
| Engine | Complete |
`;
      state.setContent(mdContent.trim());
      workspace.render();

      const preview = container.querySelector('[data-testid="notes-markdown-preview"]') as HTMLElement;
      expect(preview).not.toBeNull();
      expect(preview.innerHTML).toContain("<h1");
      expect(preview.innerHTML).toContain("Project Specs");
      expect(preview.innerHTML).toContain("<strong>bold text</strong>");
      expect(preview.innerHTML).toContain("<em>italic text</em>");
      expect(preview.innerHTML).toContain("<del>deleted text</del>");
      expect(preview.innerHTML).toContain("<pre");
      expect(preview.innerHTML).toContain("const x = 42;");
      expect(preview.innerHTML).toContain("<table");
      expect(preview.innerHTML).toContain("Feature");
    });

    it("renders interactive task checkboxes and toggles items back to editor", () => {
      state.setViewMode("split");
      state.setContent("- [ ] Build architecture\n- [ ] Run test suite");
      workspace.render();

      const preview = container.querySelector('[data-testid="notes-markdown-preview"]') as HTMLElement;
      const cbs = preview.querySelectorAll<HTMLInputElement>('input[type="checkbox"]');
      expect(cbs.length).toBe(2);
      expect(cbs[0].checked).toBe(false);

      // Click first checkbox in preview
      cbs[0].click();

      expect(state.content).toContain("- [x] Build architecture");
      expect(state.content).toContain("- [ ] Run test suite");
    });
  });

  describe("3. Notes Notebook Sidebar & Multiple Scratchpads Management", () => {
    it("creates, switches, renames, and deletes multiple scratchpad notes", () => {
      expect(state.notes.length).toBe(1);

      const note2 = state.addNote("Architecture.md", "# Architecture 2026");
      expect(state.notes.length).toBe(2);
      expect(state.activeNoteId).toBe(note2.id);
      expect(state.content).toBe("# Architecture 2026");

      workspace.render();
      const cards = container.querySelectorAll('[data-testid="note-card"]');
      expect(cards.length).toBe(2);

      // Switch back to note 1
      state.selectNote(state.notes[0].id);
      expect(state.activeNoteId).toBe(state.notes[0].id);

      // Rename note
      state.renameNote(note2.id, "SystemDesign.md");
      expect(note2.title).toBe("SystemDesign.md");

      // Delete note 2
      const deleted = state.deleteNote(note2.id);
      expect(deleted).toBe(true);
      expect(state.notes.length).toBe(1);
    });
  });

  describe("4. UnifiedShellApp Ribbon Wiring & Integration for Notes", () => {
    it("mounts NotesWorkspace and handles Notes ribbon actions through UnifiedShellApp", () => {
      const app = new UnifiedShellApp();
      expect(app).toBeDefined();

      app.getTabBar().addTab({
        id: "tab-notes-test",
        docType: "Notes",
        title: "Scratchpad.txt",
        isDirty: false,
      });
      app.getTabBar().switchTab("tab-notes-test");

      const ws = app.getNotesWorkspace();
      expect(ws).toBeDefined();

      const ribbon = app.getRibbon();
      expect(ribbon.getVisibleTabs()).toContain("Home");
      expect(ribbon.getVisibleTabs()).toContain("Edit");
      expect(ribbon.getVisibleTabs()).toContain("View");
      expect(ribbon.getVisibleTabs()).toContain("Markdown");

      // Home Tab: + New Note
      ribbon.setActiveTab("Home");
      const initialNotesCount = notesState.notes.length;
      const newBtn = document.querySelector('[data-testid="notes-btn-new"]') as HTMLButtonElement | null;
      expect(newBtn).not.toBeNull();
      newBtn?.click();
      expect(notesState.notes.length).toBe(initialNotesCount + 1);

      // Home Tab: Time/Date
      const timeBtn = document.querySelector('[data-testid="notes-btn-timedate"]') as HTMLButtonElement | null;
      expect(timeBtn).not.toBeNull();
      timeBtn?.click();
      expect(notesState.content.length).toBeGreaterThan(0);

      // View Tab: Split Preview
      ribbon.setActiveTab("View");
      const splitBtn = document.querySelector('[data-testid="notes-btn-view-split"]') as HTMLButtonElement | null;
      expect(splitBtn).not.toBeNull();
      splitBtn?.click();
      expect(notesState.viewMode).toBe("split");

      // Markdown Tab: Insert Code Block
      ribbon.setActiveTab("Markdown");
      const codeBtn = document.querySelector('[data-testid="notes-btn-md-code"]') as HTMLButtonElement | null;
      expect(codeBtn).not.toBeNull();
      codeBtn?.click();
      expect(notesState.content).toContain("```");

      // Save Note
      ribbon.setActiveTab("Home");
      const saveBtn = document.querySelector('[data-testid="notes-btn-save"]') as HTMLButtonElement | null;
      expect(saveBtn).not.toBeNull();
      saveBtn?.click();

      const docInfo = document.getElementById("status-doc-info");
      expect(docInfo?.textContent).toContain("Saved:");
    });
  });
});
