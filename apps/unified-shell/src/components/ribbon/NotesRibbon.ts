import {
  AppRibbonComponent,
  RibbonActionEmitter,
  createRibbonButton,
  createRibbonGroup,
  createRibbonSelect,
} from "./types";

export class NotesRibbon implements AppRibbonComponent {
  public getTabs(): string[] {
    return ["Home", "Edit", "Text Tools", "View", "Markdown"];
  }

  public getKeytip(tab: string): string {
    switch (tab) {
      case "Home":
        return "H";
      case "Edit":
        return "E";
      case "Text Tools":
        return "T";
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
      case "Text Tools":
        this.renderTextToolsTab(body, emit);
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
    noteRow.appendChild(createRibbonButton("Rename Note", "✏️ Rename", () => emit("RenameActiveNote"), false, "notes-btn-rename"));
    noteRow.appendChild(createRibbonButton("Delete Note", "🗑️ Delete", () => emit("DeleteActiveNote"), false, "notes-btn-delete"));
    noteRow.appendChild(createRibbonButton("Print", "🖨️ Print", () => emit("PrintDocument"), false, "notes-btn-print"));
    noteGroup.appendChild(noteRow);
    body.appendChild(noteGroup);

    // 2. Formatting & Typography
    const fontGroup = createRibbonGroup("Typography");
    const fontRow1 = document.createElement("div");
    fontRow1.className = "ribbon-group-row";
    fontRow1.appendChild(
      createRibbonSelect(["JetBrains Mono", "Consolas", "Courier New", "Inter", "System UI", "Serif"], "JetBrains Mono", (val) =>
        emit("SetFontFamily", val)
      , "notes-select-font-family")
    );
    fontRow1.appendChild(
      createRibbonSelect([10, 11, 12, 14, 16, 18, 20, 24, 28, 32], 14, (val) =>
        emit("SetFontSize", parseFloat(val))
      , "notes-select-font-size")
    );
    fontRow1.appendChild(createRibbonButton("Word Wrap", "↩ Wrap", () => emit("ToggleWordWrap"), true, "notes-btn-word-wrap"));
    fontRow1.appendChild(createRibbonButton("Line Numbers", "# Numbers", () => emit("ToggleLineNumbers"), true, "notes-btn-line-numbers"));
    fontGroup.appendChild(fontRow1);
    body.appendChild(fontGroup);

    // 3. Zoom & Time/Date
    const navGroup = createRibbonGroup("Zoom & Time");
    const navRow = document.createElement("div");
    navRow.className = "ribbon-group-row";
    navRow.appendChild(createRibbonButton("Zoom In", "+ In", () => emit("ZoomIn"), false, "notes-btn-zoom-in"));
    navRow.appendChild(createRibbonButton("Zoom Out", "- Out", () => emit("ZoomOut"), false, "notes-btn-zoom-out"));
    navRow.appendChild(createRibbonButton("100% Size", "🔍 100%", () => emit("Zoom100"), false, "notes-btn-zoom-100"));
    navRow.appendChild(createRibbonButton("Time/Date (F5)", "⏱️ Time/Date", () => emit("InsertTimeDate"), false, "notes-btn-timedate"));
    navGroup.appendChild(navRow);
    body.appendChild(navGroup);

    // 4. Quick Utilities
    const utilGroup = createRibbonGroup("Utilities");
    const utilRow1 = document.createElement("div");
    utilRow1.className = "ribbon-group-row";
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

    // 2. Clipboard & Selection
    const clipGroup = createRibbonGroup("Clipboard");
    const clipRow = document.createElement("div");
    clipRow.className = "ribbon-group-row";
    clipRow.appendChild(createRibbonButton("Cut", "✂️ Cut", () => emit("Cut"), false, "notes-btn-cut"));
    clipRow.appendChild(createRibbonButton("Copy", "📋 Copy", () => emit("Copy"), false, "notes-btn-copy"));
    clipRow.appendChild(createRibbonButton("Paste", "📥 Paste", () => emit("Paste"), false, "notes-btn-paste"));
    clipRow.appendChild(createRibbonButton("Delete", "🗑️ Delete", () => emit("Delete"), false, "notes-btn-delete-sel"));
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

    // 4. Line Operations
    const lineGroup = createRibbonGroup("Lines");
    const lineRow = document.createElement("div");
    lineRow.className = "ribbon-group-row";
    lineRow.appendChild(createRibbonButton("Duplicate Line", "📋 Dup Line", () => emit("DuplicateLine"), false, "notes-btn-dup-line"));
    lineRow.appendChild(createRibbonButton("Delete Line", "🗑️ Del Line", () => emit("DeleteLine"), false, "notes-btn-del-line"));
    lineRow.appendChild(createRibbonButton("Move Line Up", "▲ Move Up", () => emit("MoveLineUp"), false, "notes-btn-move-line-up"));
    lineRow.appendChild(createRibbonButton("Move Line Down", "▼ Move Down", () => emit("MoveLineDown"), false, "notes-btn-move-line-down"));
    lineGroup.appendChild(lineRow);
    body.appendChild(lineGroup);
  }

  private renderTextToolsTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Case Conversion
    const caseGroup = createRibbonGroup("Case Conversion");
    const caseRow = document.createElement("div");
    caseRow.className = "ribbon-group-row";
    caseRow.appendChild(createRibbonButton("UPPERCASE", "AA Upper", () => emit("TransformText", "upper"), false, "notes-btn-upper"));
    caseRow.appendChild(createRibbonButton("lowercase", "aa Lower", () => emit("TransformText", "lower"), false, "notes-btn-lower"));
    caseRow.appendChild(createRibbonButton("Title Case", "Aa Title", () => emit("TransformText", "title"), false, "notes-btn-title-case"));
    caseRow.appendChild(createRibbonButton("Sentence case", "A. Sentence", () => emit("TransformText", "sentence"), false, "notes-btn-sentence-case"));
    caseRow.appendChild(createRibbonButton("Invert Case", "aA Invert", () => emit("TransformText", "invert"), false, "notes-btn-invert-case"));
    caseGroup.appendChild(caseRow);
    body.appendChild(caseGroup);

    // 2. Line Operations & Cleaning
    const cleanGroup = createRibbonGroup("Sort & Clean");
    const cleanRow = document.createElement("div");
    cleanRow.className = "ribbon-group-row";
    cleanRow.appendChild(createRibbonButton("Sort Lines A-Z", "⇅ Sort A-Z", () => emit("TransformText", "sort_az"), false, "notes-btn-sort-az"));
    cleanRow.appendChild(createRibbonButton("Sort Lines Z-A", "⇵ Sort Z-A", () => emit("TransformText", "sort_za"), false, "notes-btn-sort-za"));
    cleanRow.appendChild(createRibbonButton("Remove Duplicates", "🧹 De-dup", () => emit("TransformText", "remove_duplicates"), false, "notes-btn-remove-duplicates"));
    cleanRow.appendChild(createRibbonButton("Remove Empty Lines", "🗑️ Empty Lines", () => emit("TransformText", "remove_empty_lines"), false, "notes-btn-remove-empty-lines"));
    cleanRow.appendChild(createRibbonButton("Trim Whitespace", "✂ Trim Space", () => emit("TransformText", "trim_whitespace"), false, "notes-btn-trim-whitespace"));
    cleanGroup.appendChild(cleanRow);
    body.appendChild(cleanGroup);

    // 3. Data & Encoding Tools
    const codeGroup = createRibbonGroup("Format & Code");
    const codeRow = document.createElement("div");
    codeRow.className = "ribbon-group-row";
    codeRow.appendChild(createRibbonButton("Format JSON", "{} Format JSON", () => emit("TransformText", "format_json"), false, "notes-btn-format-json"));
    codeRow.appendChild(createRibbonButton("Minify JSON", "{..} Minify JSON", () => emit("TransformText", "minify_json"), false, "notes-btn-minify-json"));
    codeRow.appendChild(createRibbonButton("Base64 Encode", "🔐 B64 Enc", () => emit("TransformText", "base64_encode"), false, "notes-btn-b64-enc"));
    codeRow.appendChild(createRibbonButton("Base64 Decode", "🔓 B64 Dec", () => emit("TransformText", "base64_decode"), false, "notes-btn-b64-dec"));
    codeRow.appendChild(createRibbonButton("URL Encode", "% URL Enc", () => emit("TransformText", "url_encode"), false, "notes-btn-url-enc"));
    codeRow.appendChild(createRibbonButton("URL Decode", "🔤 URL Dec", () => emit("TransformText", "url_decode"), false, "notes-btn-url-dec"));
    codeRow.appendChild(createRibbonButton("Word Statistics", "📊 Stats", () => emit("ShowStatistics"), false, "notes-btn-stats"));
    codeGroup.appendChild(codeRow);
    body.appendChild(codeGroup);
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

    // 2. Display & Whitespace
    const dispGroup = createRibbonGroup("Display");
    const dispRow = document.createElement("div");
    dispRow.className = "ribbon-group-row";
    dispRow.appendChild(createRibbonButton("Line Numbers", "# Numbers", () => emit("ToggleLineNumbers"), true, "notes-btn-view-line-numbers"));
    dispRow.appendChild(createRibbonButton("Word Wrap", "↩ Wrap", () => emit("ToggleWordWrap"), true, "notes-btn-view-word-wrap"));
    dispRow.appendChild(createRibbonButton("Show Whitespace", "· Space", () => emit("ToggleShowWhitespace"), true, "notes-btn-view-whitespace"));
    dispGroup.appendChild(dispRow);
    body.appendChild(dispGroup);

    // 3. Encoding & EOL
    const encGroup = createRibbonGroup("Encoding & EOL");
    const encRow = document.createElement("div");
    encRow.className = "ribbon-group-row";
    encRow.appendChild(
      createRibbonSelect(["CRLF", "LF"], "CRLF", (val) =>
        emit("SetLineEnding", val)
      , "notes-select-line-ending")
    );
    encRow.appendChild(
      createRibbonSelect(["UTF-8", "UTF-16", "ASCII"], "UTF-8", (val) =>
        emit("SetEncoding", val)
      , "notes-select-encoding")
    );
    encGroup.appendChild(encRow);
    body.appendChild(encGroup);

    // 4. Zoom
    const zoomGroup = createRibbonGroup("Zoom");
    const zoomRow = document.createElement("div");
    zoomRow.className = "ribbon-group-row";
    zoomRow.appendChild(createRibbonButton("Zoom In", "+ In", () => emit("ZoomIn"), false, "notes-btn-zoom-in"));
    zoomRow.appendChild(createRibbonButton("Zoom Out", "- Out", () => emit("ZoomOut"), false, "notes-btn-zoom-out"));
    zoomRow.appendChild(createRibbonButton("100%", "100%", () => emit("Zoom100"), false, "notes-btn-zoom-100"));
    zoomGroup.appendChild(zoomRow);
    body.appendChild(zoomGroup);
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

    // 2. Formatting & Code
    const formatGroup = createRibbonGroup("Formatting & Code");
    const formatRow = document.createElement("div");
    formatRow.className = "ribbon-group-row";
    formatRow.appendChild(createRibbonButton("Bold", "𝗕 Bold", () => emit("InsertMarkdownSnippet", "bold"), false, "notes-btn-md-bold"));
    formatRow.appendChild(createRibbonButton("Italic", "𝘪 Italic", () => emit("InsertMarkdownSnippet", "italic"), false, "notes-btn-md-italic"));
    formatRow.appendChild(createRibbonButton("Strikethrough", "~ Strike", () => emit("InsertMarkdownSnippet", "strike"), false, "notes-btn-md-strike"));
    formatRow.appendChild(createRibbonButton("Inline Code", "` Code `", () => emit("InsertMarkdownSnippet", "inline_code"), false, "notes-btn-md-inline-code"));
    formatRow.appendChild(createRibbonButton("Code Block", "``` Code", () => emit("InsertMarkdownSnippet", "code"), false, "notes-btn-md-code"));
    formatGroup.appendChild(formatRow);
    body.appendChild(formatGroup);

    // 3. Lists & Insertions
    const listGroup = createRibbonGroup("Lists & Tables");
    const listRow = document.createElement("div");
    listRow.className = "ribbon-group-row";
    listRow.appendChild(createRibbonButton("Bullet List", "• Bullet", () => emit("InsertMarkdownSnippet", "bullet_list"), false, "notes-btn-md-bullet"));
    listRow.appendChild(createRibbonButton("Numbered List", "1. Numbered", () => emit("InsertMarkdownSnippet", "numbered_list"), false, "notes-btn-md-number"));
    listRow.appendChild(createRibbonButton("Checklist", "☑ Task", () => emit("InsertMarkdownSnippet", "checklist"), false, "notes-btn-md-check"));
    listRow.appendChild(createRibbonButton("Table", "▦ Table", () => emit("InsertMarkdownSnippet", "table"), false, "notes-btn-md-table"));
    listRow.appendChild(createRibbonButton("Link", "🔗 Link", () => emit("InsertMarkdownSnippet", "link"), false, "notes-btn-md-link"));
    listRow.appendChild(createRibbonButton("Image", "🖼️ Image", () => emit("InsertMarkdownSnippet", "image"), false, "notes-btn-md-image"));
    listGroup.appendChild(listRow);
    body.appendChild(listGroup);
  }
}
