import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { notesState } from "../state/notesState";
import { NotesWorkspace } from "../apps/notes/NotesWorkspace";
import { NotesRibbon } from "../components/ribbon/NotesRibbon";
import { fileCodec } from "../services/fileCodec";
import { IPC } from "../ipc";

describe("Subphase 40.1.1: Notepad / TXT File Open/Save Round-Trip & Complete 5-Tab Ribbon Functionality", () => {
  let workspace: NotesWorkspace;
  let ribbon: NotesRibbon;
  let container: HTMLElement;
  let ribbonBody: HTMLElement;

  beforeEach(() => {
    document.body.innerHTML = "";

    container = document.createElement("div");
    container.id = "notes-workspace-container";
    container.style.width = "1000px";
    container.style.height = "700px";
    document.body.appendChild(container);

    ribbonBody = document.createElement("div");
    ribbonBody.className = "ribbon-body";
    document.body.appendChild(ribbonBody);

    // Mock Tauri IPC
    (window as any).__TAURI_INTERNALS__ = {};
    vi.spyOn(IPC, "saveFileBytes").mockImplementation(async (filename: string) => {
      return `C:\\Users\\User\\Documents\\${filename}`;
    });

    notesState.resetToBlank("Draft_Notes.txt");
    workspace = new NotesWorkspace(container, notesState);
    ribbon = new NotesRibbon();
  });

  afterEach(() => {
    vi.restoreAllMocks();
    document.body.innerHTML = "";
  });

  // =========================================================================
  // 1. FILE OPEN & SAVE ROUND-TRIP (.txt, .md, .json, .log)
  // =========================================================================
  it("opens local .txt, .md, and .json files, populates active editor and scratchpads list", async () => {
    const sampleText = "# Enterprise Architecture\n- Microservices\n- Event Streams\n- Zero Trust";
    const opened = await fileCodec.openFile(sampleText, "architecture.md", "Notes");
    expect(opened).toBe(true);

    expect(notesState.title).toBe("architecture.md");
    expect(notesState.getContent()).toBe(sampleText);
    expect(notesState.notes.some((n) => n.title === "architecture.md")).toBe(true);

    workspace.render();
    const textarea = workspace.getTextarea();
    expect(textarea.value).toBe(sampleText);

    // Open second file (JSON)
    const jsonText = '{"status": "ok", "nodes": 12}';
    await fileCodec.openFile(jsonText, "config.json", "Notes");
    expect(notesState.title).toBe("config.json");
    expect(notesState.getContent()).toBe(jsonText);
    expect(notesState.notes.length).toBeGreaterThanOrEqual(2);
  });

  it("saves document as .txt and .md via fileCodec and displays saved toast banner", async () => {
    notesState.setContent("Mission Critical Production Notes 2026");

    const savedTxt = await fileCodec.saveFile("Production_Report.txt", "Notes");
    expect(savedTxt).toBe("C:\\Users\\User\\Documents\\Production_Report.txt");

    const toast = document.getElementById("save-toast-banner");
    expect(toast).not.toBeNull();
    expect(toast?.textContent).toContain("Production_Report.txt");

    // Save as Markdown
    const savedMd = await fileCodec.saveFile("Production_Report.md", "Notes");
    expect(savedMd).toBe("C:\\Users\\User\\Documents\\Production_Report.md");
    expect(toast?.textContent).toContain("Production_Report.md");
  });

  // =========================================================================
  // 2. HOME TAB COMPLETE SUITE
  // =========================================================================
  it("renders Home Tab and verifies all buttons: New, Save, Typography, Zoom, TimeDate & Utilities", () => {
    const firedActions: Array<{ action: string; payload?: unknown }> = [];
    ribbon.renderTab("Home", ribbonBody, (action, payload) => {
      firedActions.push({ action, payload });
      notesState.execCommand(action, payload);
    });

    const buttons = ribbonBody.querySelectorAll<HTMLButtonElement>("button");
    expect(buttons.length).toBeGreaterThanOrEqual(14);

    buttons.forEach((btn) => btn.click());

    const actions = firedActions.map((a) => a.action);
    expect(actions).toContain("NewNote");
    expect(actions).toContain("OpenNote");
    expect(actions).toContain("SaveDocument");
    expect(actions).toContain("SaveDocumentMd");
    expect(actions).toContain("RenameActiveNote");
    expect(actions).toContain("DeleteActiveNote");
    expect(actions).toContain("PrintDocument");
    expect(actions).toContain("ToggleWordWrap");
    expect(actions).toContain("ToggleLineNumbers");
    expect(actions).toContain("ZoomIn");
    expect(actions).toContain("ZoomOut");
    expect(actions).toContain("Zoom100");
    expect(actions).toContain("InsertTimeDate");
    expect(actions).toContain("ShowStatistics");

    // Select dropdowns
    const selects = ribbonBody.querySelectorAll<HTMLSelectElement>("select");
    expect(selects.length).toBeGreaterThanOrEqual(2);
    selects[0].value = "Consolas";
    selects[0].dispatchEvent(new Event("change"));
    expect(notesState.fontFamily).toBe("Consolas");

    selects[1].value = "18";
    selects[1].dispatchEvent(new Event("change"));
    expect(notesState.fontSize).toBe(18);
  });

  // =========================================================================
  // 3. EDIT TAB COMPLETE SUITE
  // =========================================================================
  it("renders Edit Tab and verifies Undo, Redo, Clipboard, Find & Replace, and Line Operations", () => {
    const firedActions: Array<{ action: string; payload?: unknown }> = [];
    ribbon.renderTab("Edit", ribbonBody, (action, payload) => {
      firedActions.push({ action, payload });
      notesState.execCommand(action, payload);
    });

    const buttons = ribbonBody.querySelectorAll<HTMLButtonElement>("button");
    expect(buttons.length).toBeGreaterThanOrEqual(12);

    buttons.forEach((btn) => btn.click());

    const actions = firedActions.map((a) => a.action);
    expect(actions).toContain("Undo");
    expect(actions).toContain("Redo");
    expect(actions).toContain("Cut");
    expect(actions).toContain("Copy");
    expect(actions).toContain("Paste");
    expect(actions).toContain("Delete");
    expect(actions).toContain("SelectAll");
    expect(actions).toContain("Find");
    expect(actions).toContain("Replace");
    expect(actions).toContain("GoToLine");
    expect(actions).toContain("DuplicateLine");
    expect(actions).toContain("DeleteLine");
    expect(actions).toContain("MoveLineUp");
    expect(actions).toContain("MoveLineDown");

    // Verify Find & Replace bar visibility
    expect(notesState.isFindBarVisible).toBe(true);
    expect(notesState.isReplaceBarVisible).toBe(true);

    // Verify Text Replacement
    notesState.setContent("Alpha Beta Gamma Beta");
    const replaced = notesState.replaceText("Beta", "Delta");
    expect(replaced).toBe(2);
    expect(notesState.getContent()).toBe("Alpha Delta Gamma Delta");
  });

  // =========================================================================
  // 4. TEXT TOOLS TAB COMPLETE SUITE
  // =========================================================================
  it("renders Text Tools Tab and executes Case, JSON, Base64, URL encoding, Line numbering & Frequency", () => {
    const firedActions: Array<{ action: string; payload?: unknown }> = [];
    ribbon.renderTab("Text Tools", ribbonBody, (action, payload) => {
      firedActions.push({ action, payload });
      notesState.execCommand(action, payload);
    });

    const buttons = ribbonBody.querySelectorAll<HTMLButtonElement>("button");
    expect(buttons.length).toBeGreaterThanOrEqual(14);

    // 1. JSON formatting
    notesState.setContent('{"name":"aether","port":8080}');
    notesState.execCommand("TransformText", "format_json");
    expect(notesState.getContent()).toContain('  "name": "aether"');

    // 2. JSON minification
    notesState.execCommand("TransformText", "minify_json");
    expect(notesState.getContent()).toBe('{"name":"aether","port":8080}');

    // 3. Base64 Encode & Decode
    notesState.setContent("HighSpeedCrypto2026");
    notesState.execCommand("TransformText", "base64_encode");
    const b64 = notesState.getContent();
    expect(b64).not.toBe("HighSpeedCrypto2026");

    notesState.execCommand("TransformText", "base64_decode");
    expect(notesState.getContent()).toBe("HighSpeedCrypto2026");

    // 4. URL Encode & Decode
    notesState.setContent("param=hello world & token=123");
    notesState.execCommand("TransformText", "url_encode");
    expect(notesState.getContent()).toContain("hello%20world");
    notesState.execCommand("TransformText", "url_decode");
    expect(notesState.getContent()).toBe("param=hello world & token=123");

    // 5. Line Operations (Number Lines, Reverse Lines)
    notesState.setContent("First\nSecond\nThird");
    notesState.execCommand("TransformText", "number_lines");
    expect(notesState.getContent()).toBe("1. First\n2. Second\n3. Third");

    notesState.execCommand("TransformText", "reverse_lines");
    expect(notesState.getContent().startsWith("3.")).toBe(true);

    // 6. Word Frequency Analysis
    notesState.setContent("engine cluster engine node server cluster engine");
    const freq = notesState.getWordFrequency();
    expect(freq[0].word).toBe("engine");
    expect(freq[0].count).toBe(3);
  });

  // =========================================================================
  // 5. VIEW & MARKDOWN TABS COMPLETE SUITE
  // =========================================================================
  it("renders View and Markdown Tabs, handles split mode, encoding and markdown snippets", () => {
    // 1. View Tab
    ribbon.renderTab("View", ribbonBody, (action, payload) => {
      notesState.execCommand(action, payload);
    });

    const viewButtons = ribbonBody.querySelectorAll<HTMLButtonElement>("button");
    viewButtons.forEach((btn) => btn.click());

    expect(notesState.viewMode).toBe("preview");
    notesState.setViewMode("split");
    expect(notesState.viewMode).toBe("split");

    const lineEndingSelect = ribbonBody.querySelector(".notes-select-line-ending") as HTMLSelectElement;
    if (lineEndingSelect) {
      lineEndingSelect.value = "LF";
      lineEndingSelect.dispatchEvent(new Event("change"));
      expect(notesState.lineEnding).toBe("LF");
    }

    // 2. Markdown Tab
    ribbon.renderTab("Markdown", ribbonBody, (action, payload) => {
      notesState.execCommand(action, payload);
    });

    const mdButtons = ribbonBody.querySelectorAll<HTMLButtonElement>("button");
    expect(mdButtons.length).toBeGreaterThanOrEqual(14);

    notesState.setContent("");
    notesState.execCommand("InsertMarkdownSnippet", "h1");
    notesState.execCommand("InsertMarkdownSnippet", "bold");
    notesState.execCommand("InsertMarkdownSnippet", "checklist");
    notesState.execCommand("InsertMarkdownSnippet", "table");

    const content = notesState.getContent();
    expect(content).toContain("# Heading 1");
    expect(content).toContain("**bold text**");
    expect(content).toContain("- [ ] Task item");
    expect(content).toContain("| Item | Description | Status |");

    // 3. Render in Split View and verify Markdown preview output
    workspace.render();
    const preview = container.querySelector(".notes-markdown-preview");
    expect(preview).not.toBeNull();
    expect(preview?.innerHTML).toContain("Heading 1");
  });
});
