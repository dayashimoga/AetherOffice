import {
  AppRibbonComponent,
  RibbonActionEmitter,
  createRibbonButton,
  createRibbonGroup,
  createRibbonSelect,
} from "./types";

export class NotesRibbon implements AppRibbonComponent {
  public getTabs(): string[] {
    return ["Home", "Edit", "View", "Markdown"];
  }

  public getKeytip(tab: string): string {
    switch (tab) {
      case "Home":
        return "H";
      case "Edit":
        return "E";
      case "View":
        return "W";
      case "Markdown":
        return "M";
      default:
        return "H";
    }
  }

  public renderTab(tab: string, body: HTMLElement, emit: RibbonActionEmitter): void {
    body.innerHTML = "";
    switch (tab) {
      case "Home":
        this.renderHomeTab(body, emit);
        break;
      case "Edit":
        this.renderEditTab(body, emit);
        break;
      case "View":
        this.renderViewTab(body, emit);
        break;
      case "Markdown":
        this.renderMarkdownTab(body, emit);
        break;
      default:
        this.renderHomeTab(body, emit);
    }
  }

  private renderHomeTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Notes Management
    const noteGroup = createRibbonGroup("Notes");
    const noteRow = document.createElement("div");
    noteRow.className = "ribbon-group-row";
    noteRow.appendChild(createRibbonButton("+ New Note", "📄 New", () => emit("NewNote"), false, "notes-btn-new"));
    noteRow.appendChild(createRibbonButton("Open File", "📂 Open", () => emit("OpenNote"), false, "notes-btn-open"));
    noteRow.appendChild(createRibbonButton("Save (.txt)", "💾 Save .txt", () => emit("SaveDocument"), false, "notes-btn-save-txt"));
    noteRow.appendChild(createRibbonButton("Save Note", "💾 Save", () => emit("SaveDocument"), false, "notes-btn-save"));
    noteRow.appendChild(createRibbonButton("Save (.md)", "📝 Save .md", () => emit("SaveDocumentMd"), false, "notes-btn-save-md"));
    noteRow.appendChild(createRibbonButton("Print", "🖨️ Print", () => emit("PrintDocument"), false, "notes-btn-print"));
    noteGroup.appendChild(noteRow);
    body.appendChild(noteGroup);

    // 2. Formatting & Typography
    const fontGroup = createRibbonGroup("Typography");
    const fontRow1 = document.createElement("div");
    fontRow1.className = "ribbon-group-row";
    fontRow1.appendChild(
      createRibbonSelect(["JetBrains Mono", "Consolas", "Inter", "Serif"], "JetBrains Mono", (val) =>
        emit("SetFontFamily", val)
      , "notes-select-font-family")
    );
    fontRow1.appendChild(
      createRibbonSelect([11, 12, 14, 16, 18, 20, 24, 28], 14, (val) =>
        emit("SetFontSize", parseFloat(val))
      , "notes-select-font-size")
    );
    fontRow1.appendChild(createRibbonButton("Word Wrap", "↩ Wrap", () => emit("ToggleWordWrap"), true, "notes-btn-word-wrap"));
    fontRow1.appendChild(createRibbonButton("Line Numbers", "# Numbers", () => emit("ToggleLineNumbers"), true, "notes-btn-line-numbers"));
    fontGroup.appendChild(fontRow1);
    body.appendChild(fontGroup);

    // 3. Text Utilities / Power Tools
    const utilGroup = createRibbonGroup("Utilities");
    const utilRow1 = document.createElement("div");
    utilRow1.className = "ribbon-group-row";
    utilRow1.appendChild(createRibbonButton("Time/Date (F5)", "⏱️ Time/Date", () => emit("InsertTimeDate"), false, "notes-btn-timedate"));
    utilRow1.appendChild(createRibbonButton("Uppercase", "AA Upper", () => emit("TransformText", "upper"), false, "notes-btn-upper"));
    utilRow1.appendChild(createRibbonButton("Lowercase", "aa Lower", () => emit("TransformText", "lower"), false, "notes-btn-lower"));
    utilRow1.appendChild(createRibbonButton("Title Case", "Aa Title", () => emit("TransformText", "title"), false, "notes-btn-title-case"));
    utilRow1.appendChild(createRibbonButton("Sort Lines A-Z", "⇅ Sort A-Z", () => emit("TransformText", "sort_az"), false, "notes-btn-sort-az"));
    utilRow1.appendChild(createRibbonButton("Sort Lines", "⇅ Sort", () => emit("TransformText", "sort_lines"), false, "notes-btn-sort"));
    utilRow1.appendChild(createRibbonButton("Sort Lines Z-A", "⇵ Sort Z-A", () => emit("TransformText", "sort_za"), false, "notes-btn-sort-za"));
    utilRow1.appendChild(createRibbonButton("Remove Duplicates", "🧹 De-dup", () => emit("TransformText", "remove_duplicates"), false, "notes-btn-remove-duplicates"));
    utilRow1.appendChild(createRibbonButton("Trim Whitespace", "✂ Trim", () => emit("TransformText", "trim_whitespace"), false, "notes-btn-trim-whitespace"));
    utilRow1.appendChild(createRibbonButton("Statistics", "📊 Stats", () => emit("ShowStatistics"), false, "notes-btn-stats"));
    utilGroup.appendChild(utilRow1);
    body.appendChild(utilGroup);
  }

  private renderEditTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. History
    const histGroup = createRibbonGroup("History");
    const histRow = document.createElement("div");
    histRow.className = "ribbon-group-row";
    histRow.appendChild(createRibbonButton("Undo", "↶ Undo", () => emit("Undo"), false, "notes-btn-undo"));
    histRow.appendChild(createRibbonButton("Redo", "↷ Redo", () => emit("Redo"), false, "notes-btn-redo"));
    histGroup.appendChild(histRow);
    body.appendChild(histGroup);

    // 2. Clipboard
    const clipGroup = createRibbonGroup("Clipboard");
    const clipRow = document.createElement("div");
    clipRow.className = "ribbon-group-row";
    clipRow.appendChild(createRibbonButton("Cut", "✂️ Cut", () => emit("Cut"), false, "notes-btn-cut"));
    clipRow.appendChild(createRibbonButton("Copy", "📋 Copy", () => emit("Copy"), false, "notes-btn-copy"));
    clipRow.appendChild(createRibbonButton("Paste", "📥 Paste", () => emit("Paste"), false, "notes-btn-paste"));
    clipRow.appendChild(createRibbonButton("Select All", "🔲 Select All", () => emit("SelectAll"), false, "notes-btn-select-all"));
    clipGroup.appendChild(clipRow);
    body.appendChild(clipGroup);

    // 3. Search & Replace
    const searchGroup = createRibbonGroup("Find & Replace");
    const searchRow = document.createElement("div");
    searchRow.className = "ribbon-group-row";
    searchRow.appendChild(createRibbonButton("Find Text", "🔍 Find", () => emit("Find"), false, "notes-btn-find"));
    searchRow.appendChild(createRibbonButton("Replace Text", "🔄 Replace", () => emit("Replace"), false, "notes-btn-replace"));
    searchRow.appendChild(createRibbonButton("Go To Line", "➡️ Go To Line", () => emit("GoToLine"), false, "notes-btn-goto-line"));
    searchGroup.appendChild(searchRow);
    body.appendChild(searchGroup);
  }

  private renderViewTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Modes
    const modeGroup = createRibbonGroup("Editor Layout");
    const modeRow = document.createElement("div");
    modeRow.className = "ribbon-group-row";
    modeRow.appendChild(createRibbonButton("Classic Editor", "📝 Editor", () => emit("SetNotesViewMode", "editor"), false, "notes-btn-view-editor"));
    modeRow.appendChild(createRibbonButton("Split Preview", "◫ Split", () => emit("SetNotesViewMode", "split"), false, "notes-btn-view-split"));
    modeRow.appendChild(createRibbonButton("Markdown Preview", "👁️ Preview", () => emit("SetNotesViewMode", "preview"), false, "notes-btn-view-preview"));
    modeGroup.appendChild(modeRow);
    body.appendChild(modeGroup);

    // 2. Display
    const dispGroup = createRibbonGroup("Display");
    const dispRow = document.createElement("div");
    dispRow.className = "ribbon-group-row";
    dispRow.appendChild(createRibbonButton("Line Numbers", "# Numbers", () => emit("ToggleLineNumbers"), true, "notes-btn-view-line-numbers"));
    dispRow.appendChild(createRibbonButton("Word Wrap", "↩ Wrap", () => emit("ToggleWordWrap"), true, "notes-btn-view-word-wrap"));
    dispRow.appendChild(createRibbonButton("Zoom In", "+ In", () => emit("ZoomIn"), false, "notes-btn-zoom-in"));
    dispRow.appendChild(createRibbonButton("Zoom Out", "- Out", () => emit("ZoomOut"), false, "notes-btn-zoom-out"));
    dispRow.appendChild(createRibbonButton("100%", "100%", () => emit("Zoom100"), false, "notes-btn-zoom-100"));
    dispGroup.appendChild(dispRow);
    body.appendChild(dispGroup);
  }

  private renderMarkdownTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Structure
    const structGroup = createRibbonGroup("Structure");
    const structRow = document.createElement("div");
    structRow.className = "ribbon-group-row";
    structRow.appendChild(createRibbonButton("H1", "H1", () => emit("InsertMarkdownSnippet", "h1"), false, "notes-btn-h1"));
    structRow.appendChild(createRibbonButton("H2", "H2", () => emit("InsertMarkdownSnippet", "h2"), false, "notes-btn-h2"));
    structRow.appendChild(createRibbonButton("H3", "H3", () => emit("InsertMarkdownSnippet", "h3"), false, "notes-btn-h3"));
    structRow.appendChild(createRibbonButton("Blockquote", "> Quote", () => emit("InsertMarkdownSnippet", "quote"), false, "notes-btn-quote"));
    structRow.appendChild(createRibbonButton("Horizontal Rule", "--- HR", () => emit("InsertMarkdownSnippet", "hr"), false, "notes-btn-hr"));
    structGroup.appendChild(structRow);
    body.appendChild(structGroup);

    // 2. Inline & Code
    const inlineGroup = createRibbonGroup("Formatting & Code");
    const inlineRow = document.createElement("div");
    inlineRow.className = "ribbon-group-row";
    inlineRow.appendChild(createRibbonButton("Bold", "𝗕 Bold", () => emit("InsertMarkdownSnippet", "bold"), false, "notes-btn-md-bold"));
    inlineRow.appendChild(createRibbonButton("Italic", "𝘪 Italic", () => emit("InsertMarkdownSnippet", "italic"), false, "notes-btn-md-italic"));
    inlineRow.appendChild(createRibbonButton("Inline Code", "` Code `", () => emit("InsertMarkdownSnippet", "inline_code"), false, "notes-btn-md-inline-code"));
    inlineRow.appendChild(createRibbonButton("Code Block", "``` Code", () => emit("InsertMarkdownSnippet", "code"), false, "notes-btn-md-code"));
    inlineRow.appendChild(createRibbonButton("Checklist", "☑ Task", () => emit("InsertMarkdownSnippet", "checklist"), false, "notes-btn-md-check"));
    inlineRow.appendChild(createRibbonButton("Link", "🔗 Link", () => emit("InsertMarkdownSnippet", "link"), false, "notes-btn-md-link"));
    inlineRow.appendChild(createRibbonButton("Table", "▦ Table", () => emit("InsertMarkdownSnippet", "table"), false, "notes-btn-md-table"));
    inlineGroup.appendChild(inlineRow);
    body.appendChild(inlineGroup);
  }
}

