import {
  AppRibbonComponent,
  RibbonActionEmitter,
  createRibbonButton,
  createRibbonGroup,
  createRibbonSelect,
} from "./types";
import { DeveloperRibbon } from "./DeveloperRibbon";

export class CalcRibbon implements AppRibbonComponent {
  private developerRibbon = new DeveloperRibbon();
  private showDeveloper = false;

  public setDeveloperTabVisible(visible: boolean): void {
    this.showDeveloper = visible;
  }

  public getTabs(): string[] {
    const tabs = ["Home", "Insert", "Page Layout", "Formulas", "Data", "Review", "View"];
    if (this.showDeveloper) {
      tabs.push("Developer");
    }
    return tabs;
  }

  public getKeytip(tab: string): string {
    switch (tab) {
      case "Home":
        return "H";
      case "Insert":
        return "N";
      case "Page Layout":
        return "P";
      case "Formulas":
        return "M";
      case "Data":
        return "A";
      case "Review":
        return "R";
      case "View":
        return "W";
      case "Developer":
        return "L";
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
      case "Page Layout":
        this.renderPageLayoutTab(body, emit);
        break;
      case "Formulas":
        this.renderFormulasTab(body, emit);
        break;
      case "Data":
        this.renderDataTab(body, emit);
        break;
      case "Review":
        this.renderReviewTab(body, emit);
        break;
      case "View":
        this.renderViewTab(body, emit);
        break;
      case "Developer":
        this.developerRibbon.renderDeveloperTab(body, emit);
        break;
      default:
        this.renderHomeTab(body, emit);
    }
  }

  private renderHomeTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Clipboard
    const clipGroup = createRibbonGroup("Clipboard");
    clipGroup.appendChild(createRibbonButton("Paste", "📋", () => emit("Paste"), false, "calc-btn-paste"));
    const clipRow = document.createElement("div");
    clipRow.className = "ribbon-group-row";
    clipRow.appendChild(createRibbonButton("Cut", "✂️", () => emit("Cut")));
    clipRow.appendChild(createRibbonButton("Copy", "📄", () => emit("Copy")));
    clipRow.appendChild(createRibbonButton("Format Painter", "🖌️", () => emit("FormatPainter")));
    clipGroup.appendChild(clipRow);
    body.appendChild(clipGroup);

    // 2. Font
    const fontGroup = createRibbonGroup("Font");
    const fontRow1 = document.createElement("div");
    fontRow1.className = "ribbon-group-row";
    fontRow1.appendChild(
      createRibbonSelect(["Aptos", "Calibri", "Arial", "Segoe UI", "Consolas"], "Aptos", (val) =>
        emit("SetFontFamily", val)
      )
    );
    fontRow1.appendChild(
      createRibbonSelect([9, 10, 11, 12, 14, 16, 18], 11, (val) =>
        emit("SetFontSize", parseFloat(val))
      )
    );
    fontGroup.appendChild(fontRow1);

    const fontRow2 = document.createElement("div");
    fontRow2.className = "ribbon-group-row";
    fontRow2.appendChild(createRibbonButton("Bold", "B", () => emit("ToggleBold"), true));
    fontRow2.appendChild(createRibbonButton("Italic", "I", () => emit("ToggleItalic"), true));
    fontRow2.appendChild(createRibbonButton("Underline", "U", () => emit("ToggleUnderline"), true));
    fontRow2.appendChild(createRibbonButton("Borders", "🔲 Borders", () => emit("SetBorders")));
    fontRow2.appendChild(createRibbonButton("Fill Color", "🪣 Fill", () => emit("SetFillColor", "#dcfce7")));
    fontRow2.appendChild(createRibbonButton("Font Color", "🎨 Color", () => emit("SetFontColor", "#15803d")));
    fontGroup.appendChild(fontRow2);
    body.appendChild(fontGroup);

    // 3. Alignment
    const alignGroup = createRibbonGroup("Alignment");
    const alignRow1 = document.createElement("div");
    alignRow1.className = "ribbon-group-row";
    alignRow1.appendChild(createRibbonButton("Top Align", "⬆ Top", () => emit("AlignTop")));
    alignRow1.appendChild(createRibbonButton("Middle Align", "⬌ Mid", () => emit("AlignMiddle")));
    alignRow1.appendChild(createRibbonButton("Bottom Align", "⬇ Bot", () => emit("AlignBottom")));
    alignRow1.appendChild(createRibbonButton("Wrap Text", "↩ Wrap", () => emit("ToggleWrapText"), true));
    alignGroup.appendChild(alignRow1);

    const alignRow2 = document.createElement("div");
    alignRow2.className = "ribbon-group-row";
    alignRow2.appendChild(createRibbonButton("Left", "Left", () => emit("AlignLeft")));
    alignRow2.appendChild(createRibbonButton("Center", "Center", () => emit("AlignCenter")));
    alignRow2.appendChild(createRibbonButton("Right", "Right", () => emit("AlignRight")));
    alignRow2.appendChild(createRibbonButton("Merge & Center", "⤢ Merge", () => emit("MergeAndCenter"), true));
    alignGroup.appendChild(alignRow2);
    body.appendChild(alignGroup);

    // 4. Number
    const numGroup = createRibbonGroup("Number");
    const numRow1 = document.createElement("div");
    numRow1.className = "ribbon-group-row";
    numRow1.appendChild(
      createRibbonSelect(
        ["General", "Number", "Currency", "Accounting", "Short Date", "Percentage"],
        "General",
        (val) => emit("SetNumberFormat", val)
      )
    );
    numGroup.appendChild(numRow1);

    const numRow2 = document.createElement("div");
    numRow2.className = "ribbon-group-row";
    numRow2.appendChild(createRibbonButton("Currency", "$", () => emit("SetNumberFormat", "Currency")));
    numRow2.appendChild(createRibbonButton("Percent", "%", () => emit("SetNumberFormat", "Percentage")));
    numRow2.appendChild(createRibbonButton("Comma", ",", () => emit("SetNumberFormat", "Comma")));
    numRow2.appendChild(createRibbonButton("Inc Decimal", ".00 ➔", () => emit("IncreaseDecimal")));
    numRow2.appendChild(createRibbonButton("Dec Decimal", ".0 ➔", () => emit("DecreaseDecimal")));
    numGroup.appendChild(numRow2);
    body.appendChild(numGroup);

    // 5. Styles
    const styleGroup = createRibbonGroup("Styles");
    const styleRow = document.createElement("div");
    styleRow.className = "ribbon-group-row";
    styleRow.appendChild(createRibbonButton("Conditional Formatting", "📊 Cond Format", () => emit("ConditionalFormatting")));
    styleRow.appendChild(createRibbonButton("Format as Table", "▦ Table Style", () => emit("FormatAsTable")));
    styleRow.appendChild(createRibbonButton("Cell Styles", "🎨 Cell Styles", () => emit("CellStyles")));
    styleGroup.appendChild(styleRow);
    body.appendChild(styleGroup);

    // 6. Cells
    const cellGroup = createRibbonGroup("Cells");
    const cellRow = document.createElement("div");
    cellRow.className = "ribbon-group-row";
    cellRow.appendChild(createRibbonButton("Insert Cells", "➕ Insert", () => emit("InsertCells")));
    cellRow.appendChild(createRibbonButton("Delete Cells", "➖ Delete", () => emit("DeleteCells")));
    cellRow.appendChild(createRibbonButton("Format Cell", "📐 Format", () => emit("FormatCells")));
    cellGroup.appendChild(cellRow);
    body.appendChild(cellGroup);

    // 7. Editing
    const editGroup = createRibbonGroup("Editing");
    const editRow = document.createElement("div");
    editRow.className = "ribbon-group-row";
    editRow.appendChild(createRibbonButton("AutoSum", "Σ AutoSum", () => emit("AutoSum")));
    editRow.appendChild(createRibbonButton("Fill", "⬇ Fill", () => emit("FillDown")));
    editRow.appendChild(createRibbonButton("Clear", "🧹 Clear", () => emit("ClearCells")));
    editRow.appendChild(createRibbonButton("Sort & Filter", "🌪️ Sort/Filter", () => emit("SortFilter")));
    editRow.appendChild(createRibbonButton("Find & Select", "🔍 Find", () => emit("Find")));
    editGroup.appendChild(editRow);
    body.appendChild(editGroup);
  }

  private renderInsertTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Tables Group
    const tblGroup = createRibbonGroup("Tables");
    const tblRow = document.createElement("div");
    tblRow.className = "ribbon-group-row";
    tblRow.appendChild(createRibbonButton("PivotTable", "📊 PivotTable", () => emit("InsertPivotTable")));
    tblRow.appendChild(createRibbonButton("PivotChart", "📈 PivotChart", () => emit("InsertPivotChart")));
    tblRow.appendChild(createRibbonButton("Slicer", "🔪 Slicer", () => emit("InsertSlicer")));
    tblRow.appendChild(createRibbonButton("Table", "▦ Table", () => emit("InsertTable")));
    tblGroup.appendChild(tblRow);
    body.appendChild(tblGroup);

    // 2. Illustrations
    const illGroup = createRibbonGroup("Illustrations");
    const illRow = document.createElement("div");
    illRow.className = "ribbon-group-row";
    illRow.appendChild(createRibbonButton("Pictures", "🖼️ Picture", () => emit("InsertPicture")));
    illRow.appendChild(createRibbonButton("Shapes", "⬛ Shapes", () => emit("InsertShape", "rect")));
    illGroup.appendChild(illRow);
    body.appendChild(illGroup);

    // 3. Charts Group
    const chartGroup = createRibbonGroup("Charts");
    const chartRow1 = document.createElement("div");
    chartRow1.className = "ribbon-group-row";
    chartRow1.appendChild(createRibbonButton("Column", "📊 Column", () => emit("InsertChart", "column")));
    chartRow1.appendChild(createRibbonButton("Bar", "📊 Bar", () => emit("InsertChart", "bar")));
    chartRow1.appendChild(createRibbonButton("Line", "📈 Line", () => emit("InsertChart", "line")));
    chartGroup.appendChild(chartRow1);

    const chartRow2 = document.createElement("div");
    chartRow2.className = "ribbon-group-row";
    chartRow2.appendChild(createRibbonButton("Pie", "🥧 Pie", () => emit("InsertChart", "pie")));
    chartRow2.appendChild(createRibbonButton("Scatter", "⁘ Scatter", () => emit("InsertChart", "scatter")));
    chartRow2.appendChild(createRibbonButton("Recommended", "🌟 Recommended", () => emit("InsertChart", "recommended")));
    chartGroup.appendChild(chartRow2);
    body.appendChild(chartGroup);

    // 4. Sparklines
    const sparkGroup = createRibbonGroup("Sparklines");
    const sparkRow = document.createElement("div");
    sparkRow.className = "ribbon-group-row";
    sparkRow.appendChild(createRibbonButton("Line Sparkline", "📈 Line", () => emit("InsertSparkline", "line")));
    sparkRow.appendChild(createRibbonButton("Column Sparkline", "📊 Col", () => emit("InsertSparkline", "column")));
    sparkRow.appendChild(createRibbonButton("Win/Loss", "↕ Win/Loss", () => emit("InsertSparkline", "winloss")));
    sparkGroup.appendChild(sparkRow);
    body.appendChild(sparkGroup);

    // 5. Links & Comments
    const linkGroup = createRibbonGroup("Links & Notes");
    linkGroup.appendChild(createRibbonButton("Link", "🔗 Link", () => emit("InsertLink")));
    linkGroup.appendChild(createRibbonButton("New Note", "📝 Note", () => emit("NewNote")));
    body.appendChild(linkGroup);
  }

  private renderPageLayoutTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Themes Group
    const themeGroup = createRibbonGroup("Themes");
    const themeRow = document.createElement("div");
    themeRow.className = "ribbon-group-row";
    themeRow.appendChild(createRibbonButton("Office Theme", "🎨 Themes", () => emit("SetTheme", "Office")));
    themeRow.appendChild(createRibbonButton("Theme Colors", "Colors", () => emit("SetThemeColors")));
    themeRow.appendChild(createRibbonButton("Theme Fonts", "Fonts", () => emit("SetThemeFonts")));
    themeGroup.appendChild(themeRow);
    body.appendChild(themeGroup);

    // 2. Page Setup Group
    const setupGroup = createRibbonGroup("Page Setup");
    const setupRow1 = document.createElement("div");
    setupRow1.className = "ribbon-group-row";
    setupRow1.appendChild(createRibbonButton("Normal Margins", "📐 Normal", () => emit("SetMargins", "normal")));
    setupRow1.appendChild(createRibbonButton("Narrow Margins", "📐 Narrow", () => emit("SetMargins", "narrow")));
    setupRow1.appendChild(createRibbonButton("Wide Margins", "📐 Wide", () => emit("SetMargins", "wide")));
    setupGroup.appendChild(setupRow1);

    const setupRow2 = document.createElement("div");
    setupRow2.className = "ribbon-group-row";
    setupRow2.appendChild(createRibbonButton("Portrait", "📄 Portrait", () => emit("SetOrientation", "portrait")));
    setupRow2.appendChild(createRibbonButton("Landscape", "📃 Landscape", () => emit("SetOrientation", "landscape")));
    setupRow2.appendChild(createRibbonButton("Set Print Area", "🖨️ Print Area", () => emit("SetPrintArea")));
    setupGroup.appendChild(setupRow2);
    body.appendChild(setupGroup);

    // 3. Sheet Options Group
    const optGroup = createRibbonGroup("Sheet Options");
    const optRow = document.createElement("div");
    optRow.className = "ribbon-group-row";
    optRow.appendChild(createRibbonButton("View Gridlines", "☑ View Grid", () => emit("ToggleGridlines"), true));
    optRow.appendChild(createRibbonButton("Print Gridlines", "☑ Print Grid", () => emit("TogglePrintGrid"), true));
    optRow.appendChild(createRibbonButton("View Headings", "☑ View Headings", () => emit("ToggleHeadings"), true));
    optRow.appendChild(createRibbonButton("Print Headings", "☑ Print Headings", () => emit("TogglePrintHeadings"), true));
    optGroup.appendChild(optRow);
    body.appendChild(optGroup);
  }

  private renderFormulasTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Function Library
    const fnGroup = createRibbonGroup("Function Library");
    const fnRow1 = document.createElement("div");
    fnRow1.className = "ribbon-group-row";
    fnRow1.appendChild(createRibbonButton("Insert Function", "ƒx Insert", () => emit("InsertFunction")));
    fnRow1.appendChild(createRibbonButton("AutoSum", "Σ SUM", () => emit("AutoSum")));
    fnRow1.appendChild(createRibbonButton("Financial", "💰 Financial", () => emit("FormulaCategory", "financial")));
    fnRow1.appendChild(createRibbonButton("Logical", "⚖️ Logical", () => emit("FormulaCategory", "logical")));
    fnGroup.appendChild(fnRow1);

    const fnRow2 = document.createElement("div");
    fnRow2.className = "ribbon-group-row";
    fnRow2.appendChild(createRibbonButton("Text Functions", "🔤 Text", () => emit("FormulaCategory", "text")));
    fnRow2.appendChild(createRibbonButton("Date & Time", "📅 Date", () => emit("FormulaCategory", "datetime")));
    fnRow2.appendChild(createRibbonButton("Lookup & Ref", "🔍 Lookup", () => emit("FormulaCategory", "lookup")));
    fnRow2.appendChild(createRibbonButton("Math & Trig", "∑ Math", () => emit("FormulaCategory", "math")));
    fnGroup.appendChild(fnRow2);
    body.appendChild(fnGroup);

    // 2. Defined Names
    const nameGroup = createRibbonGroup("Defined Names");
    const nameRow = document.createElement("div");
    nameRow.className = "ribbon-group-row";
    nameRow.appendChild(createRibbonButton("Name Manager", "🏷️ Name Manager", () => emit("NameManager")));
    nameRow.appendChild(createRibbonButton("Define Name", "➕ Define Name", () => emit("DefineName")));
    nameGroup.appendChild(nameRow);
    body.appendChild(nameGroup);

    // 3. Formula Auditing
    const auditGroup = createRibbonGroup("Formula Auditing");
    const auditRow = document.createElement("div");
    auditRow.className = "ribbon-group-row";
    auditRow.appendChild(createRibbonButton("Trace Precedents", "🔍 Trace Precedents", () => emit("TracePrecedents")));
    auditRow.appendChild(createRibbonButton("Trace Dependents", "🔍 Trace Dependents", () => emit("TraceDependents")));
    auditRow.appendChild(createRibbonButton("Remove Arrows", "❌ Remove Arrows", () => emit("RemoveArrows")));
    auditRow.appendChild(createRibbonButton("Show Formulas", "🧮 Show Formulas", () => emit("ShowFormulas"), true));
    auditRow.appendChild(createRibbonButton("Evaluate", "⚡ Evaluate", () => emit("EvaluateFormula")));
    auditGroup.appendChild(auditRow);
    body.appendChild(auditGroup);

    // 4. Calculation
    const calcGroup = createRibbonGroup("Calculation");
    calcGroup.appendChild(createRibbonButton("Calculate Now", "⚡ Calculate (F9)", () => emit("Recalculate")));
    calcGroup.appendChild(createRibbonButton("Calc Sheet", "Calc Sheet", () => emit("CalculateSheet")));
    body.appendChild(calcGroup);
  }

  private renderDataTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Get & Transform Data
    const getGroup = createRibbonGroup("Get & Transform Data");
    const getRow = document.createElement("div");
    getRow.className = "ribbon-group-row";
    getRow.appendChild(createRibbonButton("From Text/CSV", "📄 From CSV", () => emit("ImportCsv")));
    getRow.appendChild(createRibbonButton("From Web", "🌐 From Web", () => emit("ImportWeb")));
    getRow.appendChild(createRibbonButton("Refresh All", "🔄 Refresh All", () => emit("RefreshData")));
    getGroup.appendChild(getRow);
    body.appendChild(getGroup);

    // 2. Sort & Filter
    const sortGroup = createRibbonGroup("Sort & Filter");
    const sortRow = document.createElement("div");
    sortRow.className = "ribbon-group-row";
    sortRow.appendChild(createRibbonButton("Sort A to Z", "🔼 Sort A-Z", () => emit("SortAsc")));
    sortRow.appendChild(createRibbonButton("Sort Z to A", "🔽 Sort Z-A", () => emit("SortDesc")));
    sortRow.appendChild(createRibbonButton("Filter", "🌪️ Filter", () => emit("ToggleFilter"), true));
    sortRow.appendChild(createRibbonButton("Clear Filter", "Clear", () => emit("ClearFilter")));
    sortGroup.appendChild(sortRow);
    body.appendChild(sortGroup);

    // 3. Data Tools
    const toolsGroup = createRibbonGroup("Data Tools");
    const toolsRow = document.createElement("div");
    toolsRow.className = "ribbon-group-row";
    toolsRow.appendChild(createRibbonButton("Text to Columns", "🔀 Text to Columns", () => emit("TextToColumns")));
    toolsRow.appendChild(createRibbonButton("Flash Fill", "⚡ Flash Fill", () => emit("FlashFill")));
    toolsRow.appendChild(createRibbonButton("Remove Duplicates", "🗑️ Remove Dups", () => emit("RemoveDuplicates")));
    toolsRow.appendChild(createRibbonButton("Data Validation", "✓ Data Validation", () => emit("DataValidation")));
    toolsGroup.appendChild(toolsRow);
    body.appendChild(toolsGroup);

    // 4. Outline
    const outGroup = createRibbonGroup("Outline");
    const outRow = document.createElement("div");
    outRow.className = "ribbon-group-row";
    outRow.appendChild(createRibbonButton("Group", "⊞ Group", () => emit("GroupRows")));
    outRow.appendChild(createRibbonButton("Ungroup", "⊟ Ungroup", () => emit("UngroupRows")));
    outRow.appendChild(createRibbonButton("Subtotal", "∑ Subtotal", () => emit("Subtotal")));
    outGroup.appendChild(outRow);
    body.appendChild(outGroup);
  }

  private renderReviewTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Proofing
    const proofGroup = createRibbonGroup("Proofing");
    const proofRow = document.createElement("div");
    proofRow.className = "ribbon-group-row";
    proofRow.appendChild(createRibbonButton("Spelling", "✓ Spelling", () => emit("CheckSpelling")));
    proofRow.appendChild(createRibbonButton("Thesaurus", "📖 Thesaurus", () => emit("Thesaurus")));
    proofGroup.appendChild(proofRow);
    body.appendChild(proofGroup);

    // 2. Protect
    const protGroup = createRibbonGroup("Protect");
    const protRow = document.createElement("div");
    protRow.className = "ribbon-group-row";
    protRow.appendChild(createRibbonButton("Protect Sheet", "🔒 Protect Sheet", () => emit("ProtectSheet")));
    protRow.appendChild(createRibbonButton("Protect Workbook", "🔒 Protect Workbook", () => emit("ProtectWorkbook")));
    protRow.appendChild(createRibbonButton("Allow Edit Ranges", "🔓 Edit Ranges", () => emit("AllowEditRanges")));
    protGroup.appendChild(protRow);
    body.appendChild(protGroup);

    // 3. Comments & Notes
    const noteGroup = createRibbonGroup("Notes");
    noteGroup.appendChild(createRibbonButton("New Note", "💬 New Note", () => emit("NewNote")));
    noteGroup.appendChild(createRibbonButton("Show All Notes", "Show All Notes", () => emit("ShowAllNotes"), true));
    body.appendChild(noteGroup);
  }

  private renderViewTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Workbook Views
    const viewsGroup = createRibbonGroup("Workbook Views");
    const viewsRow = document.createElement("div");
    viewsRow.className = "ribbon-group-row";
    viewsRow.appendChild(createRibbonButton("Normal View", "▦ Normal", () => emit("ViewNormal")));
    viewsRow.appendChild(createRibbonButton("Page Break Preview", "📑 Page Break", () => emit("ViewPageBreak")));
    viewsRow.appendChild(createRibbonButton("Page Layout", "📄 Page Layout", () => emit("ViewPageLayout")));
    viewsGroup.appendChild(viewsRow);
    body.appendChild(viewsGroup);

    // 2. Show Group
    const showGroup = createRibbonGroup("Show");
    const showRow = document.createElement("div");
    showRow.className = "ribbon-group-row";
    showRow.appendChild(createRibbonButton("Formula Bar", "☑ Formula Bar", () => emit("ToggleFormulaBar"), true));
    showRow.appendChild(createRibbonButton("Gridlines", "☑ Gridlines", () => emit("ToggleGridlines"), true));
    showRow.appendChild(createRibbonButton("Headings", "☑ Headings", () => emit("ToggleHeadings"), true));
    showGroup.appendChild(showRow);
    body.appendChild(showGroup);

    // 3. Zoom
    const zoomGroup = createRibbonGroup("Zoom");
    const zoomRow = document.createElement("div");
    zoomRow.className = "ribbon-group-row";
    zoomRow.appendChild(createRibbonButton("100%", "🔍 100%", () => emit("Zoom100")));
    zoomRow.appendChild(createRibbonButton("Zoom to Selection", "🎯 Selection", () => emit("ZoomToSelection")));
    zoomRow.appendChild(createRibbonButton("Zoom In", "+ In", () => emit("ZoomIn")));
    zoomRow.appendChild(createRibbonButton("Zoom Out", "- Out", () => emit("ZoomOut")));
    zoomGroup.appendChild(zoomRow);
    body.appendChild(zoomGroup);

    // 4. Window
    const winGroup = createRibbonGroup("Window");
    const winRow = document.createElement("div");
    winRow.className = "ribbon-group-row";
    winRow.appendChild(createRibbonButton("Freeze Panes", "❄️ Freeze Panes", () => emit("FreezePanes")));
    winRow.appendChild(createRibbonButton("Freeze Top Row", "Top Row", () => emit("FreezeTopRow")));
    winRow.appendChild(createRibbonButton("Freeze First Col", "First Col", () => emit("FreezeFirstColumn")));
    winRow.appendChild(createRibbonButton("Split", "✂️ Split", () => emit("SplitWindow"), true));
    winGroup.appendChild(winRow);
    body.appendChild(winGroup);
  }
}

