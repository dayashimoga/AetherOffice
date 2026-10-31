import {
  AppRibbonComponent,
  RibbonActionEmitter,
  createRibbonButton,
  createRibbonGroup,
  createRibbonSelect,
} from "./types";

export class WriterRibbon implements AppRibbonComponent {
  public getTabs(): string[] {
    return ["Home", "Insert", "Layout", "References", "Review", "View"];
  }

  public getKeytip(tab: string): string {
    switch (tab) {
      case "Home":
        return "H";
      case "Insert":
        return "N";
      case "Layout":
        return "P";
      case "References":
        return "S";
      case "Review":
        return "R";
      case "View":
        return "W";
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
      case "Insert":
        this.renderInsertTab(body, emit);
        break;
      case "Layout":
        this.renderLayoutTab(body, emit);
        break;
      case "References":
        this.renderReferencesTab(body, emit);
        break;
      case "Review":
        this.renderReviewTab(body, emit);
        break;
      case "View":
        this.renderViewTab(body, emit);
        break;
      default:
        this.renderHomeTab(body, emit);
    }
  }

  private renderHomeTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 0. Quick Pages Group
    const quickPagesGroup = createRibbonGroup("Pages");
    quickPagesGroup.appendChild(
      createRibbonButton("+ New Page", "➕ Page", () => emit("InsertNewPage"), false, "btn-home-new-page")
    );
    body.appendChild(quickPagesGroup);

    // 1. Clipboard Group
    const clipGroup = createRibbonGroup("Clipboard");
    clipGroup.appendChild(
      createRibbonButton("Paste", "📋 Paste", () => emit("Paste"), false, "writer-btn-paste")
    );
    const clipRow = document.createElement("div");
    clipRow.className = "ribbon-group-row";
    clipRow.appendChild(createRibbonButton("Cut", "✂️ Cut", () => emit("Cut"), false, "btn-cut"));
    clipRow.appendChild(createRibbonButton("Copy", "📄 Copy", () => emit("Copy"), false, "btn-copy"));
    clipRow.appendChild(createRibbonButton("Format Painter", "🖌️ Painter", () => emit("FormatPainter"), false, "btn-format-painter"));
    clipGroup.appendChild(clipRow);
    body.appendChild(clipGroup);

    // 2. Font Group
    const fontGroup = createRibbonGroup("Font");
    const fontRow1 = document.createElement("div");
    fontRow1.className = "ribbon-group-row";
    fontRow1.appendChild(
      createRibbonSelect(
        ["Calibri", "Arial", "Times New Roman", "Inter", "Georgia", "Courier New", "Consolas", "Aptos", "Verdana"],
        "Calibri",
        (val) => emit("SetFontFamily", val),
        "font-family-select"
      )
    );
    fontRow1.appendChild(
      createRibbonSelect(
        [8, 9, 10, 11, 12, 14, 16, 18, 20, 24, 28, 36, 48, 72],
        11,
        (val) => emit("SetFontSize", parseFloat(val)),
        "font-size-select"
      )
    );
    fontRow1.appendChild(createRibbonButton("Increase Font", "A+", () => emit("IncreaseFontSize"), false, "btn-increase-font"));
    fontRow1.appendChild(createRibbonButton("Decrease Font", "A-", () => emit("DecreaseFontSize"), false, "btn-decrease-font"));
    fontRow1.appendChild(createRibbonButton("Change Case", "Aa", () => emit("ChangeCase", "uppercase"), false, "btn-change-case"));
    fontRow1.appendChild(createRibbonButton("Clear Formatting", "🧹", () => emit("ClearFormatting"), false, "btn-clear-formatting"));
    fontGroup.appendChild(fontRow1);

    const fontRow2 = document.createElement("div");
    fontRow2.className = "ribbon-group-row";
    fontRow2.appendChild(createRibbonButton("Bold", "B", () => emit("ToggleBold"), true, "btn-bold"));
    fontRow2.appendChild(createRibbonButton("Italic", "I", () => emit("ToggleItalic"), true, "btn-italic"));
    fontRow2.appendChild(createRibbonButton("Underline", "U", () => emit("ToggleUnderline"), true, "btn-underline"));
    fontRow2.appendChild(createRibbonButton("Strikethrough", "S", () => emit("ToggleStrikethrough"), true, "btn-strikethrough"));
    fontRow2.appendChild(createRibbonButton("Subscript", "X₂", () => emit("ToggleSubscript"), true, "btn-subscript"));
    fontRow2.appendChild(createRibbonButton("Superscript", "X²", () => emit("ToggleSuperscript"), true, "btn-superscript"));
    fontRow2.appendChild(createRibbonButton("Font Color", "🎨", () => emit("SetFontColor", "#2563eb"), false, "btn-font-color"));
    fontRow2.appendChild(createRibbonButton("Highlight", "🖍️", () => emit("SetHighlightColor", "#fef08a"), false, "btn-highlight-color"));
    fontGroup.appendChild(fontRow2);
    body.appendChild(fontGroup);

    // 3. Paragraph Group
    const paraGroup = createRibbonGroup("Paragraph");
    const paraRow1 = document.createElement("div");
    paraRow1.className = "ribbon-group-row";
    paraRow1.appendChild(createRibbonButton("Bullets", "•≡", () => emit("ToggleBulletList"), false, "btn-bullet-list"));
    paraRow1.appendChild(createRibbonButton("Numbering", "1≡", () => emit("ToggleNumberedList"), false, "btn-numbered-list"));
    paraRow1.appendChild(createRibbonButton("Multilevel", "1.1≡", () => emit("ToggleMultilevelList"), false, "btn-multilevel-list"));
    paraRow1.appendChild(createRibbonButton("DecIndent", "⇤", () => emit("DecreaseIndent"), false, "btn-decrease-indent"));
    paraRow1.appendChild(createRibbonButton("IncIndent", "⇥", () => emit("IncreaseIndent"), false, "btn-increase-indent"));
    paraRow1.appendChild(createRibbonButton("Sort", "AZ↓", () => emit("SortParagraphs"), false, "btn-sort-paragraphs"));
    paraRow1.appendChild(createRibbonButton("Formatting Marks", "¶", () => emit("ToggleFormattingMarks"), false, "btn-show-formatting"));
    paraGroup.appendChild(paraRow1);

    const paraRow2 = document.createElement("div");
    paraRow2.className = "ribbon-group-row";
    paraRow2.appendChild(createRibbonButton("Align Left", "Left", () => emit("AlignLeft"), false, "btn-align-left"));
    paraRow2.appendChild(createRibbonButton("Center", "Center", () => emit("AlignCenter"), false, "btn-align-center"));
    paraRow2.appendChild(createRibbonButton("Align Right", "Right", () => emit("AlignRight"), false, "btn-align-right"));
    paraRow2.appendChild(createRibbonButton("Justify", "Justify", () => emit("AlignJustify"), false, "btn-align-justify"));
    paraRow2.appendChild(
      createRibbonSelect(
        ["1.0", "1.15", "1.5", "2.0"],
        "1.15",
        (val) => emit("SetLineSpacing", parseFloat(val)),
        "line-spacing-select"
      )
    );
    paraRow2.appendChild(createRibbonButton("Shading", "🪣", () => emit("SetShading", "#f3f4f6"), false, "btn-shading"));
    paraRow2.appendChild(createRibbonButton("Borders", "⊞", () => emit("SetParagraphBorder", "1px solid #94a3b8"), false, "btn-paragraph-border"));
    paraGroup.appendChild(paraRow2);
    body.appendChild(paraGroup);

    // 4. Styles Group
    const stylesGroup = createRibbonGroup("Styles");
    const stylesRow = document.createElement("div");
    stylesRow.className = "ribbon-group-row";
    stylesRow.appendChild(createRibbonButton("Normal", "Normal", () => emit("SetStyle", "Normal"), false, "btn-style-normal"));
    stylesRow.appendChild(createRibbonButton("No Spacing", "No Spacing", () => emit("SetStyle", "No Spacing"), false, "btn-style-no-spacing"));
    stylesRow.appendChild(createRibbonButton("Heading 1", "Heading 1", () => emit("SetStyle", "Heading 1"), false, "btn-style-h1"));
    stylesRow.appendChild(createRibbonButton("Heading 2", "Heading 2", () => emit("SetStyle", "Heading 2"), false, "btn-style-h2"));
    stylesRow.appendChild(createRibbonButton("Heading 3", "Heading 3", () => emit("SetStyle", "Heading 3"), false, "btn-style-h3"));
    stylesRow.appendChild(createRibbonButton("Title", "Title", () => emit("SetStyle", "Title"), false, "btn-style-title"));
    stylesRow.appendChild(createRibbonButton("Subtitle", "Subtitle", () => emit("SetStyle", "Subtitle"), false, "btn-style-subtitle"));
    stylesRow.appendChild(createRibbonButton("Quote", "❝ Quote", () => emit("SetStyle", "Quote"), false, "btn-style-quote"));
    stylesGroup.appendChild(stylesRow);
    body.appendChild(stylesGroup);

    // 5. Editing Group
    const editGroup = createRibbonGroup("Editing");
    editGroup.appendChild(createRibbonButton("Find", "🔍 Find", () => emit("Find"), false, "btn-find"));
    editGroup.appendChild(createRibbonButton("Replace", "🔄 Replace", () => emit("Replace"), false, "btn-replace"));
    editGroup.appendChild(createRibbonButton("Select All", "Select All", () => emit("SelectAll"), false, "btn-select-all"));
    body.appendChild(editGroup);

    // 6. Copilot Group (Subphase 20.1.3)
    const copilotGroup = createRibbonGroup("Copilot");
    copilotGroup.appendChild(
      createRibbonButton("Aether Copilot", "✨ Copilot", () => emit("ToggleCopilot"), false, "btn-copilot")
    );
    body.appendChild(copilotGroup);
  }

  private renderInsertTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Pages Group
    const pagesGroup = createRibbonGroup("Pages");
    const pagesRow = document.createElement("div");
    pagesRow.className = "ribbon-group-row";
    pagesRow.appendChild(createRibbonButton("Cover Page", "📄 Cover", () => emit("InsertCoverPage"), false, "btn-insert-cover-page"));
    pagesRow.appendChild(createRibbonButton("+ New Page", "➕ Page", () => emit("InsertNewPage"), false, "btn-insert-new-page"));
    pagesRow.appendChild(createRibbonButton("Blank Page", "📃 Blank", () => emit("InsertBlankPage"), false, "btn-insert-blank-page"));
    pagesRow.appendChild(createRibbonButton("Page Break", "✂️ Break", () => emit("InsertPageBreak"), false, "btn-insert-page-break"));
    pagesRow.appendChild(createRibbonButton("Delete Page", "🗑️ Page", () => emit("DeleteEmptyPage"), false, "btn-delete-empty-page"));
    pagesGroup.appendChild(pagesRow);
    body.appendChild(pagesGroup);

    // 2. Tables Group
    const tableGroup = createRibbonGroup("Tables");
    const tableRow = document.createElement("div");
    tableRow.className = "ribbon-group-row";
    tableRow.appendChild(
      createRibbonButton("Insert Table", "📊 Table", () => emit("InsertTable", { rows: 3, cols: 3 }), false, "btn-insert-table")
    );
    tableRow.appendChild(createRibbonButton("Add Row", "+ Row", () => emit("AddTableRow"), false, "btn-add-table-row"));
    tableRow.appendChild(createRibbonButton("Add Col", "+ Col", () => emit("AddTableColumn"), false, "btn-add-table-col"));
    tableRow.appendChild(createRibbonButton("Del Row", "- Row", () => emit("DeleteTableRow"), false, "btn-delete-table-row"));
    tableRow.appendChild(createRibbonButton("Del Col", "- Col", () => emit("DeleteTableColumn"), false, "btn-delete-table-col"));
    tableGroup.appendChild(tableRow);
    body.appendChild(tableGroup);

    // 3. Illustrations & Shapes Group
    const illustGroup = createRibbonGroup("Illustrations");
    const illustRow1 = document.createElement("div");
    illustRow1.className = "ribbon-group-row";
    illustRow1.appendChild(createRibbonButton("Picture", "🖼️ Picture", () => emit("InsertPicture"), false, "btn-insert-picture"));
    illustRow1.appendChild(createRibbonButton("Rectangle", "⬛ Rect", () => emit("InsertShape", "rect"), false, "btn-shape-rect"));
    illustRow1.appendChild(createRibbonButton("Rounded", "▢ Round", () => emit("InsertShape", "roundRect"), false, "btn-shape-round"));
    illustRow1.appendChild(createRibbonButton("Ellipse", "⚪ Ellipse", () => emit("InsertShape", "ellipse"), false, "btn-shape-ellipse"));
    illustGroup.appendChild(illustRow1);

    const illustRow2 = document.createElement("div");
    illustRow2.className = "ribbon-group-row";
    illustRow2.appendChild(createRibbonButton("Triangle", "▲ Tri", () => emit("InsertShape", "triangle"), false, "btn-shape-triangle"));
    illustRow2.appendChild(createRibbonButton("Arrow", "➔ Arrow", () => emit("InsertShape", "arrow"), false, "btn-shape-arrow"));
    illustRow2.appendChild(createRibbonButton("Star", "★ Star", () => emit("InsertShape", "star"), false, "btn-shape-star"));
    illustRow2.appendChild(createRibbonButton("Callout", "💬 Callout", () => emit("InsertShape", "callout"), false, "btn-shape-callout"));
    illustRow2.appendChild(createRibbonButton("Chart", "📈 Chart", () => emit("InsertChart"), false, "btn-insert-chart"));
    illustGroup.appendChild(illustRow2);
    body.appendChild(illustGroup);

    // 4. Header & Footer Group
    const hfGroup = createRibbonGroup("Header & Footer");
    const hfRow = document.createElement("div");
    hfRow.className = "ribbon-group-row";
    hfRow.appendChild(createRibbonButton("Header", "Header", () => emit("InsertHeader"), false, "btn-insert-header"));
    hfRow.appendChild(createRibbonButton("Footer", "Footer", () => emit("InsertFooter"), false, "btn-insert-footer"));
    hfRow.appendChild(createRibbonButton("Page Number", "# Page No", () => emit("InsertPageNumber"), false, "btn-insert-page-number"));
    hfGroup.appendChild(hfRow);
    body.appendChild(hfGroup);

    // 5. Text Group
    const textGroup = createRibbonGroup("Text");
    const textRow = document.createElement("div");
    textRow.className = "ribbon-group-row";
    textRow.appendChild(createRibbonButton("Text Box", "🔤 Text Box", () => emit("InsertTextBox"), false, "btn-insert-textbox"));
    textRow.appendChild(createRibbonButton("WordArt", "🎨 WordArt", () => emit("InsertWordArt"), false, "btn-insert-wordart"));
    textRow.appendChild(createRibbonButton("Drop Cap", "🔠 Drop Cap", () => emit("InsertDropCap"), false, "btn-insert-dropcap"));
    textRow.appendChild(createRibbonButton("Line", "━ Line", () => emit("InsertHorizontalLine"), false, "btn-insert-line"));
    textRow.appendChild(createRibbonButton("Date/Time", "🕒 Date", () => emit("InsertDateTime"), false, "btn-insert-datetime"));
    textRow.appendChild(createRibbonButton("Symbol", "Ω Symbol", () => emit("InsertSymbol", "©"), false, "btn-insert-symbol"));
    textRow.appendChild(createRibbonButton("Equation", "∑ Equation", () => emit("InsertEquation", "∑(x) = ∫ 2π r dr"), false, "btn-insert-equation"));
    textGroup.appendChild(textRow);
    body.appendChild(textGroup);

    // 6. Links & Comments Group
    const linksGroup = createRibbonGroup("Links & Comments");
    const linksRow = document.createElement("div");
    linksRow.className = "ribbon-group-row";
    linksRow.appendChild(createRibbonButton("Hyperlink", "🔗 Link", () => emit("InsertHyperlink"), false, "btn-insert-hyperlink"));
    linksRow.appendChild(createRibbonButton("Link", "🔗 Link", () => emit("InsertLink"), false, "btn-insert-link"));
    linksRow.appendChild(createRibbonButton("Comment", "💬 Comment", () => emit("InsertComment"), false, "btn-insert-comment"));
    linksRow.appendChild(createRibbonButton("Bookmark", "🔖 Bookmark", () => emit("InsertBookmark"), false, "btn-insert-bookmark"));
    linksGroup.appendChild(linksRow);
    body.appendChild(linksGroup);
  }

  private renderLayoutTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Page Setup Group
    const pageGroup = createRibbonGroup("Page Setup");
    const pageRow1 = document.createElement("div");
    pageRow1.className = "ribbon-group-row";
    pageRow1.appendChild(createRibbonButton("Normal Margins", "📐 Normal", () => emit("SetMargins", "normal"), false, "btn-margins-normal"));
    pageRow1.appendChild(createRibbonButton("Narrow Margins", "📐 Narrow", () => emit("SetMargins", "narrow"), false, "btn-margins-narrow"));
    pageRow1.appendChild(createRibbonButton("Moderate Margins", "📐 Moderate", () => emit("SetMargins", "moderate"), false, "btn-margins-moderate"));
    pageRow1.appendChild(createRibbonButton("Wide Margins", "📐 Wide", () => emit("SetMargins", "wide"), false, "btn-margins-wide"));
    pageRow1.appendChild(createRibbonButton("Custom Margins", "📐 Custom", () => emit("SetMargins", "custom"), false, "btn-margins-custom"));
    pageGroup.appendChild(pageRow1);

    const pageRow2 = document.createElement("div");
    pageRow2.className = "ribbon-group-row";
    pageRow2.appendChild(createRibbonButton("Portrait", "📄 Portrait", () => emit("SetOrientation", "portrait"), false, "btn-orientation-portrait"));
    pageRow2.appendChild(createRibbonButton("Landscape", "📃 Landscape", () => emit("SetOrientation", "landscape"), false, "btn-orientation-landscape"));
    pageRow2.appendChild(createRibbonButton("Letter", "Letter", () => emit("SetPageSize", "letter"), false, "btn-pagesize-letter"));
    pageRow2.appendChild(createRibbonButton("A4", "A4", () => emit("SetPageSize", "a4"), false, "btn-pagesize-a4"));
    pageRow2.appendChild(createRibbonButton("Legal", "Legal", () => emit("SetPageSize", "legal"), false, "btn-pagesize-legal"));
    pageRow2.appendChild(createRibbonButton("Break", "✂️ Break", () => emit("InsertPageBreak"), false, "btn-layout-pagebreak"));
    pageGroup.appendChild(pageRow2);

    const pageRow3 = document.createElement("div");
    pageRow3.className = "ribbon-group-row";
    pageRow3.appendChild(createRibbonButton("Line Numbers", "123 Lines", () => emit("ToggleLineNumbers"), false, "btn-line-numbers"));
    pageRow3.appendChild(createRibbonButton("Hyphenation", "Hyphen", () => emit("ToggleHyphenation"), false, "btn-hyphenation"));
    pageRow3.appendChild(createRibbonButton("Page Color", "🎨 Color", () => emit("SetPageBackgroundColor", "#f8fafc"), false, "btn-page-color"));
    pageRow3.appendChild(createRibbonButton("Page Borders", "⊞ Borders", () => emit("SetPageBorder", "2px solid #2563eb"), false, "btn-page-borders"));
    pageRow3.appendChild(createRibbonButton("Watermark", "🌊 Watermark", () => emit("SetWatermark", "CONFIDENTIAL"), false, "btn-watermark"));
    pageGroup.appendChild(pageRow3);
    body.appendChild(pageGroup);

    // 2. Columns Group
    const colsGroup = createRibbonGroup("Columns");
    const colsRow = document.createElement("div");
    colsRow.className = "ribbon-group-row";
    colsRow.appendChild(createRibbonButton("One Column", "1 Col", () => emit("SetColumns", 1), false, "btn-cols-1"));
    colsRow.appendChild(createRibbonButton("Two Columns", "2 Col", () => emit("SetColumns", 2), false, "btn-cols-2"));
    colsRow.appendChild(createRibbonButton("Three Columns", "3 Col", () => emit("SetColumns", 3), false, "btn-cols-3"));
    colsGroup.appendChild(colsRow);
    body.appendChild(colsGroup);

    // 3. Paragraph Spacing Group
    const paraGroup = createRibbonGroup("Paragraph Spacing");
    const paraRow = document.createElement("div");
    paraRow.className = "ribbon-group-row";
    paraRow.appendChild(createRibbonButton("Indent", "⇥ Indent", () => emit("IncreaseIndent"), false, "btn-layout-indent"));
    paraRow.appendChild(createRibbonButton("Outdent", "⇤ Outdent", () => emit("DecreaseIndent"), false, "btn-layout-outdent"));
    paraRow.appendChild(createRibbonButton("1.0 Spacing", "1.0", () => emit("SetLineSpacing", 1.0), false, "btn-spacing-10"));
    paraRow.appendChild(createRibbonButton("1.15 Spacing", "1.15", () => emit("SetLineSpacing", 1.15), false, "btn-spacing-115"));
    paraRow.appendChild(createRibbonButton("1.5 Spacing", "1.5", () => emit("SetLineSpacing", 1.5), false, "btn-spacing-15"));
    paraRow.appendChild(createRibbonButton("2.0 Spacing", "2.0", () => emit("SetLineSpacing", 2.0), false, "btn-spacing-20"));
    paraGroup.appendChild(paraRow);
    body.appendChild(paraGroup);
  }

  private renderReferencesTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Table of Contents Group
    const tocGroup = createRibbonGroup("Table of Contents");
    const tocRow = document.createElement("div");
    tocRow.className = "ribbon-group-row";
    tocRow.appendChild(createRibbonButton("Insert TOC", "📑 Insert TOC", () => emit("InsertTOC"), false, "btn-insert-toc"));
    tocRow.appendChild(createRibbonButton("Update Table", "🔄 Update TOC", () => emit("UpdateTOC"), false, "btn-update-toc"));
    tocGroup.appendChild(tocRow);
    body.appendChild(tocGroup);

    // 2. Footnotes Group
    const footGroup = createRibbonGroup("Footnotes");
    const footRow = document.createElement("div");
    footRow.className = "ribbon-group-row";
    footRow.appendChild(createRibbonButton("Insert Footnote", "¹ Footnote", () => emit("InsertFootnote"), false, "btn-insert-footnote"));
    footRow.appendChild(createRibbonButton("Insert Endnote", "ᵢ Endnote", () => emit("InsertEndnote"), false, "btn-insert-endnote"));
    footRow.appendChild(createRibbonButton("Next Footnote", "⏭ Next", () => emit("NextFootnote"), false, "btn-next-footnote"));
    footGroup.appendChild(footRow);
    body.appendChild(footGroup);

    // 3. Citations & Bibliography Group
    const citeGroup = createRibbonGroup("Citations & Bibliography");
    const citeRow = document.createElement("div");
    citeRow.className = "ribbon-group-row";
    citeRow.appendChild(createRibbonButton("Insert Citation", "📖 Citation", () => emit("InsertCitation"), false, "btn-insert-citation"));
    citeRow.appendChild(createRibbonButton("Manage Sources", "📚 Sources", () => emit("ManageSources"), false, "btn-manage-sources"));
    citeRow.appendChild(createRibbonButton("Style: APA", "Style APA", () => emit("SetCitationStyle", "APA"), false, "btn-citation-style"));
    citeRow.appendChild(createRibbonButton("Bibliography", "📚 Bibliography", () => emit("InsertBibliography"), false, "btn-insert-bib"));
    citeGroup.appendChild(citeRow);
    body.appendChild(citeGroup);

    // 4. Captions Group
    const capGroup = createRibbonGroup("Captions");
    const capRow = document.createElement("div");
    capRow.className = "ribbon-group-row";
    capRow.appendChild(createRibbonButton("Insert Caption", "🖼️ Caption", () => emit("InsertCaption"), false, "btn-insert-caption"));
    capRow.appendChild(createRibbonButton("Table of Figures", "📋 Figures", () => emit("TableOfFigures"), false, "btn-table-of-figures"));
    capGroup.appendChild(capRow);
    body.appendChild(capGroup);
  }

  private renderReviewTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Proofing Group
    const proofGroup = createRibbonGroup("Proofing");
    const proofRow = document.createElement("div");
    proofRow.className = "ribbon-group-row";
    proofRow.appendChild(createRibbonButton("Spelling & Grammar", "✓ Spelling", () => emit("CheckSpelling"), false, "btn-spelling"));
    proofRow.appendChild(createRibbonButton("Thesaurus", "📖 Thesaurus", () => emit("Thesaurus"), false, "btn-thesaurus"));
    proofRow.appendChild(createRibbonButton("Word Count", "🔢 Word Count", () => emit("WordCount"), false, "btn-word-count"));
    proofGroup.appendChild(proofRow);
    body.appendChild(proofGroup);

    // 2. Speech Group (Read Aloud)
    const speechGroup = createRibbonGroup("Read Aloud");
    const speechRow = document.createElement("div");
    speechRow.className = "ribbon-group-row";
    speechRow.appendChild(createRibbonButton("Play", "▶ Read", () => emit("ReadAloudPlay"), false, "btn-read-aloud-play"));
    speechRow.appendChild(createRibbonButton("Pause", "⏸ Pause", () => emit("ReadAloudPause"), false, "btn-read-aloud-pause"));
    speechRow.appendChild(createRibbonButton("Stop", "⏹ Stop", () => emit("ReadAloudStop"), false, "btn-read-aloud-stop"));
    speechGroup.appendChild(speechRow);
    body.appendChild(speechGroup);

    // 3. Language Group
    const langGroup = createRibbonGroup("Language");
    const langRow = document.createElement("div");
    langRow.className = "ribbon-group-row";
    langRow.appendChild(createRibbonButton("Translate", "🌐 Translate", () => emit("Translate"), false, "btn-translate"));
    langRow.appendChild(createRibbonButton("Proofing Language", "Language", () => emit("SetLanguage"), false, "btn-language"));
    langGroup.appendChild(langRow);
    body.appendChild(langGroup);

    // 4. Comments Group
    const commGroup = createRibbonGroup("Comments");
    const commRow = document.createElement("div");
    commRow.className = "ribbon-group-row";
    commRow.appendChild(createRibbonButton("New Comment", "💬 Comment", () => emit("NewComment"), false, "btn-new-comment"));
    commRow.appendChild(createRibbonButton("Delete Comment", "🗑️ Delete", () => emit("DeleteComment"), false, "btn-delete-comment"));
    commRow.appendChild(createRibbonButton("Next Comment", "⏭ Next", () => emit("NextComment"), false, "btn-next-comment"));
    commRow.appendChild(createRibbonButton("Prev Comment", "⏮ Prev", () => emit("PrevComment"), false, "btn-prev-comment"));
    commGroup.appendChild(commRow);
    body.appendChild(commGroup);

    // 5. Tracking Group
    const trackGroup = createRibbonGroup("Tracking");
    const trackRow = document.createElement("div");
    trackRow.className = "ribbon-group-row";
    trackRow.appendChild(
      createRibbonButton("Track Changes", "📝 Track", () => emit("ToggleTrackChanges"), true, "btn-track-changes")
    );
    trackRow.appendChild(createRibbonButton("Accept Change", "✓ Accept", () => emit("AcceptChange"), false, "btn-accept-change"));
    trackRow.appendChild(createRibbonButton("Reject Change", "✕ Reject", () => emit("RejectChange"), false, "btn-reject-change"));
    trackRow.appendChild(createRibbonButton("Accept All", "✓✓ All", () => emit("AcceptAll"), false, "btn-accept-all"));
    trackRow.appendChild(createRibbonButton("Reject All", "✕✕ All", () => emit("RejectAll"), false, "btn-reject-all"));
    trackGroup.appendChild(trackRow);
    body.appendChild(trackGroup);

    // 6. Protect Group
    const protectGroup = createRibbonGroup("Protect");
    protectGroup.appendChild(
      createRibbonButton("Protect Document", "🔒 Protect", () => emit("ProtectDocument"), true, "btn-protect-doc")
    );
    body.appendChild(protectGroup);
  }

  private renderViewTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Views Group
    const viewsGroup = createRibbonGroup("Views");
    const viewsRow = document.createElement("div");
    viewsRow.className = "ribbon-group-row";
    viewsRow.appendChild(createRibbonButton("Print Layout", "📄 Print Layout", () => emit("ViewPrintLayout"), false, "btn-view-print"));
    viewsRow.appendChild(createRibbonButton("Web Layout", "🌐 Web Layout", () => emit("ViewWebLayout"), false, "btn-view-web"));
    viewsRow.appendChild(createRibbonButton("Read Mode", "📖 Read Mode", () => emit("ViewReadMode"), false, "btn-view-read"));
    viewsRow.appendChild(createRibbonButton("Focus Mode", "🎯 Focus", () => emit("ToggleFocusMode"), false, "btn-view-focus"));
    viewsGroup.appendChild(viewsRow);
    body.appendChild(viewsGroup);

    // 2. Show Group
    const showGroup = createRibbonGroup("Show");
    const showRow = document.createElement("div");
    showRow.className = "ribbon-group-row";
    showRow.appendChild(createRibbonButton("Ruler", "📏 Ruler", () => emit("ToggleRuler"), true, "btn-toggle-ruler"));
    showRow.appendChild(createRibbonButton("Gridlines", "🔲 Gridlines", () => emit("ToggleGridlines"), true, "btn-toggle-gridlines"));
    showRow.appendChild(createRibbonButton("Navigation Pane", "📑 Nav Pane", () => emit("ToggleNavPane"), true, "btn-toggle-nav"));
    showRow.appendChild(createRibbonButton("Dark Canvas", "🌙 Dark Canvas", () => emit("ToggleDarkCanvas"), true, "btn-toggle-dark-canvas"));
    showGroup.appendChild(showRow);
    body.appendChild(showGroup);

    // 3. Zoom Group
    const zoomGroup = createRibbonGroup("Zoom");
    const zoomRow1 = document.createElement("div");
    zoomRow1.className = "ribbon-group-row";
    zoomRow1.appendChild(createRibbonButton("50%", "🔍 50%", () => emit("SetZoom", 0.5), false, "btn-zoom-50"));
    zoomRow1.appendChild(createRibbonButton("75%", "🔍 75%", () => emit("SetZoom", 0.75), false, "btn-zoom-75"));
    zoomRow1.appendChild(createRibbonButton("100%", "🔍 100%", () => emit("Zoom100"), false, "btn-zoom-100"));
    zoomRow1.appendChild(createRibbonButton("150%", "🔍 150%", () => emit("SetZoom", 1.5), false, "btn-zoom-150"));
    zoomRow1.appendChild(createRibbonButton("200%", "🔍 200%", () => emit("SetZoom", 2.0), false, "btn-zoom-200"));
    zoomGroup.appendChild(zoomRow1);

    const zoomRow2 = document.createElement("div");
    zoomRow2.className = "ribbon-group-row";
    zoomRow2.appendChild(createRibbonButton("One Page", "📄 One Page", () => emit("ZoomOnePage"), false, "btn-zoom-one"));
    zoomRow2.appendChild(createRibbonButton("Multiple Pages", "📄📄 Multi Pages", () => emit("ZoomMultiplePages"), false, "btn-zoom-multiple"));
    zoomRow2.appendChild(createRibbonButton("Page Width", "↔️ Page Width", () => emit("ZoomPageWidth"), false, "btn-zoom-width"));
    zoomGroup.appendChild(zoomRow2);
    body.appendChild(zoomGroup);
  }
}

