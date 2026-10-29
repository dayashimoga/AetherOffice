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
    // 1. Clipboard Group
    const clipGroup = createRibbonGroup("Clipboard");
    clipGroup.appendChild(
      createRibbonButton("Paste", "📋", () => emit("Paste"), false, "writer-btn-paste")
    );
    const clipRow = document.createElement("div");
    clipRow.className = "ribbon-group-row";
    clipRow.appendChild(createRibbonButton("Cut", "✂️", () => emit("Cut")));
    clipRow.appendChild(createRibbonButton("Copy", "📄", () => emit("Copy")));
    clipRow.appendChild(createRibbonButton("Format Painter", "🖌️", () => emit("FormatPainter")));
    clipGroup.appendChild(clipRow);
    body.appendChild(clipGroup);

    // 2. Font Group
    const fontGroup = createRibbonGroup("Font");
    const fontRow1 = document.createElement("div");
    fontRow1.className = "ribbon-group-row";
    fontRow1.appendChild(
      createRibbonSelect(
        ["Calibri", "Arial", "Times New Roman", "Inter", "Consolas", "Georgia", "Aptos"],
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
    fontRow2.appendChild(createRibbonButton("Increase Font", "A+", () => emit("IncreaseFontSize"), false, "btn-increase-font"));
    fontRow2.appendChild(createRibbonButton("Decrease Font", "A-", () => emit("DecreaseFontSize"), false, "btn-decrease-font"));
    fontRow2.appendChild(createRibbonButton("Clear Formatting", "🧹", () => emit("ClearFormatting"), false, "btn-clear-formatting"));
    fontGroup.appendChild(fontRow2);
    body.appendChild(fontGroup);

    // 3. Paragraph Group
    const paraGroup = createRibbonGroup("Paragraph");
    const paraRow1 = document.createElement("div");
    paraRow1.className = "ribbon-group-row";
    paraRow1.appendChild(createRibbonButton("Bullets", "•≡", () => emit("ToggleBulletList"), false, "btn-bullet-list"));
    paraRow1.appendChild(createRibbonButton("Numbering", "1≡", () => emit("ToggleNumberedList"), false, "btn-numbered-list"));
    paraRow1.appendChild(createRibbonButton("DecIndent", "⇤", () => emit("DecreaseIndent"), false, "btn-decrease-indent"));
    paraRow1.appendChild(createRibbonButton("IncIndent", "⇥", () => emit("IncreaseIndent"), false, "btn-increase-indent"));
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
    paraGroup.appendChild(paraRow2);
    body.appendChild(paraGroup);

    // 4. Styles Group
    const stylesGroup = createRibbonGroup("Styles");
    const stylesRow = document.createElement("div");
    stylesRow.className = "ribbon-group-row";
    stylesRow.appendChild(createRibbonButton("Normal", "Normal", () => emit("SetStyle", "Normal"), false, "btn-style-normal"));
    stylesRow.appendChild(createRibbonButton("Heading 1", "Heading 1", () => emit("SetStyle", "Heading 1"), false, "btn-style-h1"));
    stylesRow.appendChild(createRibbonButton("Heading 2", "Heading 2", () => emit("SetStyle", "Heading 2"), false, "btn-style-h2"));
    stylesRow.appendChild(createRibbonButton("Heading 3", "Heading 3", () => emit("SetStyle", "Heading 3"), false, "btn-style-h3"));
    stylesRow.appendChild(createRibbonButton("Title", "Title", () => emit("SetStyle", "Title"), false, "btn-style-title"));
    stylesRow.appendChild(createRibbonButton("Subtitle", "Subtitle", () => emit("SetStyle", "Subtitle"), false, "btn-style-subtitle"));
    stylesGroup.appendChild(stylesRow);
    body.appendChild(stylesGroup);

    // 5. Editing Group
    const editGroup = createRibbonGroup("Editing");
    editGroup.appendChild(createRibbonButton("Find", "🔍 Find", () => emit("Find")));
    editGroup.appendChild(createRibbonButton("Replace", "🔄 Replace", () => emit("Replace")));
    editGroup.appendChild(createRibbonButton("Select All", "Select All", () => emit("SelectAll")));
    body.appendChild(editGroup);
  }

  private renderInsertTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Pages Group
    const pagesGroup = createRibbonGroup("Pages");
    const pagesRow = document.createElement("div");
    pagesRow.className = "ribbon-group-row";
    pagesRow.appendChild(createRibbonButton("Cover Page", "📄 Cover", () => emit("InsertCoverPage")));
    pagesRow.appendChild(createRibbonButton("Blank Page", "📃 Blank", () => emit("InsertBlankPage")));
    pagesRow.appendChild(createRibbonButton("Page Break", "✂️ Break", () => emit("InsertPageBreak")));
    pagesGroup.appendChild(pagesRow);
    body.appendChild(pagesGroup);

    // 2. Tables Group
    const tableGroup = createRibbonGroup("Tables");
    tableGroup.appendChild(
      createRibbonButton("Insert Table", "📊 Table", () => emit("InsertTable", { rows: 3, cols: 3 }))
    );
    body.appendChild(tableGroup);

    // 3. Illustrations & Shapes Group
    const illustGroup = createRibbonGroup("Illustrations");
    const illustRow1 = document.createElement("div");
    illustRow1.className = "ribbon-group-row";
    illustRow1.appendChild(createRibbonButton("Picture", "🖼️ Picture", () => emit("InsertPicture")));
    illustRow1.appendChild(createRibbonButton("Rectangle", "⬛ Rect", () => emit("InsertShape", "rect")));
    illustRow1.appendChild(createRibbonButton("Rounded", "▢ Round", () => emit("InsertShape", "roundRect")));
    illustRow1.appendChild(createRibbonButton("Ellipse", "⚪ Ellipse", () => emit("InsertShape", "ellipse")));
    illustGroup.appendChild(illustRow1);

    const illustRow2 = document.createElement("div");
    illustRow2.className = "ribbon-group-row";
    illustRow2.appendChild(createRibbonButton("Triangle", "▲ Tri", () => emit("InsertShape", "triangle")));
    illustRow2.appendChild(createRibbonButton("Arrow", "➔ Arrow", () => emit("InsertShape", "arrow")));
    illustRow2.appendChild(createRibbonButton("Star", "★ Star", () => emit("InsertShape", "star")));
    illustRow2.appendChild(createRibbonButton("Chart", "📈 Chart", () => emit("InsertChart")));
    illustGroup.appendChild(illustRow2);
    body.appendChild(illustGroup);

    // 4. Header & Footer Group
    const hfGroup = createRibbonGroup("Header & Footer");
    const hfRow = document.createElement("div");
    hfRow.className = "ribbon-group-row";
    hfRow.appendChild(createRibbonButton("Header", "Header", () => emit("InsertHeader")));
    hfRow.appendChild(createRibbonButton("Footer", "Footer", () => emit("InsertFooter")));
    hfRow.appendChild(createRibbonButton("Page Number", "# Page No", () => emit("InsertPageNumber")));
    hfGroup.appendChild(hfRow);
    body.appendChild(hfGroup);

    // 5. Text Group
    const textGroup = createRibbonGroup("Text");
    const textRow = document.createElement("div");
    textRow.className = "ribbon-group-row";
    textRow.appendChild(createRibbonButton("Text Box", "🔤 Text Box", () => emit("InsertTextBox")));
    textRow.appendChild(createRibbonButton("WordArt", "🎨 WordArt", () => emit("InsertWordArt")));
    textRow.appendChild(createRibbonButton("Drop Cap", "🔠 Drop Cap", () => emit("InsertDropCap")));
    textGroup.appendChild(textRow);
    body.appendChild(textGroup);

    // 6. Links Group
    const linksGroup = createRibbonGroup("Links");
    linksGroup.appendChild(createRibbonButton("Link", "🔗 Link", () => emit("InsertLink")));
    linksGroup.appendChild(createRibbonButton("Bookmark", "🔖 Bookmark", () => emit("InsertBookmark")));
    body.appendChild(linksGroup);
  }

  private renderLayoutTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Page Setup Group
    const pageGroup = createRibbonGroup("Page Setup");
    const pageRow1 = document.createElement("div");
    pageRow1.className = "ribbon-group-row";
    pageRow1.appendChild(createRibbonButton("Normal Margins", "📐 Normal", () => emit("SetMargins", "normal"), false, "btn-margins-normal"));
    pageRow1.appendChild(createRibbonButton("Narrow Margins", "📐 Narrow", () => emit("SetMargins", "narrow"), false, "btn-margins-narrow"));
    pageRow1.appendChild(createRibbonButton("Wide Margins", "📐 Wide", () => emit("SetMargins", "wide"), false, "btn-margins-wide"));
    pageGroup.appendChild(pageRow1);

    const pageRow2 = document.createElement("div");
    pageRow2.className = "ribbon-group-row";
    pageRow2.appendChild(createRibbonButton("Portrait", "📄 Portrait", () => emit("SetOrientation", "portrait"), false, "btn-orientation-portrait"));
    pageRow2.appendChild(createRibbonButton("Landscape", "📃 Landscape", () => emit("SetOrientation", "landscape"), false, "btn-orientation-landscape"));
    pageRow2.appendChild(createRibbonButton("Letter", "Letter", () => emit("SetPageSize", "letter"), false, "btn-pagesize-letter"));
    pageRow2.appendChild(createRibbonButton("A4", "A4", () => emit("SetPageSize", "a4"), false, "btn-pagesize-a4"));
    pageRow2.appendChild(createRibbonButton("Legal", "Legal", () => emit("SetPageSize", "legal"), false, "btn-pagesize-legal"));
    pageGroup.appendChild(pageRow2);
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
    tocRow.appendChild(createRibbonButton("Insert TOC", "📑 Insert TOC", () => emit("InsertTOC")));
    tocRow.appendChild(createRibbonButton("Update Table", "🔄 Update TOC", () => emit("UpdateTOC")));
    tocGroup.appendChild(tocRow);
    body.appendChild(tocGroup);

    // 2. Footnotes Group
    const footGroup = createRibbonGroup("Footnotes");
    const footRow = document.createElement("div");
    footRow.className = "ribbon-group-row";
    footRow.appendChild(createRibbonButton("Insert Footnote", "¹ Footnote", () => emit("InsertFootnote")));
    footRow.appendChild(createRibbonButton("Insert Endnote", "ᵢ Endnote", () => emit("InsertEndnote")));
    footRow.appendChild(createRibbonButton("Next Footnote", "⏭ Next", () => emit("NextFootnote")));
    footGroup.appendChild(footRow);
    body.appendChild(footGroup);

    // 3. Citations & Bibliography Group
    const citeGroup = createRibbonGroup("Citations & Bibliography");
    const citeRow = document.createElement("div");
    citeRow.className = "ribbon-group-row";
    citeRow.appendChild(createRibbonButton("Insert Citation", "📖 Citation", () => emit("InsertCitation")));
    citeRow.appendChild(createRibbonButton("Manage Sources", "📚 Sources", () => emit("ManageSources")));
    citeRow.appendChild(createRibbonButton("Style: APA", "Style APA", () => emit("SetCitationStyle", "APA")));
    citeGroup.appendChild(citeRow);
    body.appendChild(citeGroup);

    // 4. Captions Group
    const capGroup = createRibbonGroup("Captions");
    capGroup.appendChild(createRibbonButton("Insert Caption", "🖼️ Caption", () => emit("InsertCaption")));
    capGroup.appendChild(createRibbonButton("Table of Figures", "📋 Figures", () => emit("TableOfFigures")));
    body.appendChild(capGroup);
  }

  private renderReviewTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Proofing Group
    const proofGroup = createRibbonGroup("Proofing");
    const proofRow = document.createElement("div");
    proofRow.className = "ribbon-group-row";
    proofRow.appendChild(createRibbonButton("Spelling & Grammar", "✓ Spelling", () => emit("CheckSpelling")));
    proofRow.appendChild(createRibbonButton("Thesaurus", "📖 Thesaurus", () => emit("Thesaurus")));
    proofRow.appendChild(createRibbonButton("Word Count", "🔢 Word Count", () => emit("WordCount")));
    proofGroup.appendChild(proofRow);
    body.appendChild(proofGroup);

    // 2. Language Group
    const langGroup = createRibbonGroup("Language");
    langGroup.appendChild(createRibbonButton("Translate", "🌐 Translate", () => emit("Translate")));
    langGroup.appendChild(createRibbonButton("Proofing Language", "Language", () => emit("SetLanguage")));
    body.appendChild(langGroup);

    // 3. Comments Group
    const commGroup = createRibbonGroup("Comments");
    const commRow = document.createElement("div");
    commRow.className = "ribbon-group-row";
    commRow.appendChild(createRibbonButton("New Comment", "💬 New Comment", () => emit("NewComment")));
    commRow.appendChild(createRibbonButton("Delete Comment", "🗑️ Delete", () => emit("DeleteComment")));
    commGroup.appendChild(commRow);
    body.appendChild(commGroup);

    // 4. Tracking Group
    const trackGroup = createRibbonGroup("Tracking");
    const trackRow = document.createElement("div");
    trackRow.className = "ribbon-group-row";
    trackRow.appendChild(
      createRibbonButton("Track Changes", "📝 Track", () => emit("ToggleTrackChanges"), true)
    );
    trackRow.appendChild(createRibbonButton("Accept Change", "✓ Accept", () => emit("AcceptChange")));
    trackRow.appendChild(createRibbonButton("Reject Change", "✕ Reject", () => emit("RejectChange")));
    trackGroup.appendChild(trackRow);
    body.appendChild(trackGroup);
  }

  private renderViewTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Views Group
    const viewsGroup = createRibbonGroup("Views");
    const viewsRow = document.createElement("div");
    viewsRow.className = "ribbon-group-row";
    viewsRow.appendChild(createRibbonButton("Print Layout", "📄 Print Layout", () => emit("ViewPrintLayout")));
    viewsRow.appendChild(createRibbonButton("Web Layout", "🌐 Web Layout", () => emit("ViewWebLayout")));
    viewsRow.appendChild(createRibbonButton("Read Mode", "📖 Read Mode", () => emit("ViewReadMode")));
    viewsGroup.appendChild(viewsRow);
    body.appendChild(viewsGroup);

    // 2. Show Group
    const showGroup = createRibbonGroup("Show");
    const showRow = document.createElement("div");
    showRow.className = "ribbon-group-row";
    showRow.appendChild(createRibbonButton("Ruler", "📏 Ruler", () => emit("ToggleRuler"), true));
    showRow.appendChild(createRibbonButton("Gridlines", "🔲 Gridlines", () => emit("ToggleGridlines"), true));
    showRow.appendChild(createRibbonButton("Navigation Pane", "📑 Nav Pane", () => emit("ToggleNavPane"), true));
    showGroup.appendChild(showRow);
    body.appendChild(showGroup);

    // 3. Zoom Group
    const zoomGroup = createRibbonGroup("Zoom");
    const zoomRow = document.createElement("div");
    zoomRow.className = "ribbon-group-row";
    zoomRow.appendChild(createRibbonButton("100%", "🔍 100%", () => emit("Zoom100")));
    zoomRow.appendChild(createRibbonButton("One Page", "📄 One Page", () => emit("ZoomOnePage")));
    zoomRow.appendChild(createRibbonButton("Multiple Pages", "📄📄 Multi Pages", () => emit("ZoomMultiplePages")));
    zoomRow.appendChild(createRibbonButton("Page Width", "↔️ Page Width", () => emit("ZoomPageWidth")));
    zoomGroup.appendChild(zoomRow);
    body.appendChild(zoomGroup);
  }
}

