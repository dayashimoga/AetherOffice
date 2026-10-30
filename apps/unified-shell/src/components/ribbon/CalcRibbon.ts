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
    clipRow.appendChild(createRibbonButton("Cut", "✂️", () => emit("Cut"), false, "calc-btn-cut"));
    clipRow.appendChild(createRibbonButton("Copy", "📄", () => emit("Copy"), false, "calc-btn-copy"));
    clipRow.appendChild(createRibbonButton("Format Painter", "🖌️", () => emit("FormatPainter"), false, "calc-btn-format-painter"));
    clipGroup.appendChild(clipRow);
    body.appendChild(clipGroup);

    // 2. Font
    const fontGroup = createRibbonGroup("Font");
    const fontRow1 = document.createElement("div");
    fontRow1.className = "ribbon-group-row";
    fontRow1.appendChild(
      createRibbonSelect(["Aptos", "Calibri", "Arial", "Segoe UI", "Consolas"], "Aptos", (val) =>
        emit("SetFontFamily", val), "calc-select-font-family"
      )
    );
    fontRow1.appendChild(
      createRibbonSelect([9, 10, 11, 12, 14, 16, 18], 11, (val) =>
        emit("SetFontSize", parseFloat(val)), "calc-select-font-size"
      )
    );
    fontGroup.appendChild(fontRow1);

    const fontRow2 = document.createElement("div");
    fontRow2.className = "ribbon-group-row";
    fontRow2.appendChild(createRibbonButton("Bold", "B", () => emit("ToggleBold"), true, "calc-btn-bold"));
    fontRow2.appendChild(createRibbonButton("Italic", "I", () => emit("ToggleItalic"), true, "calc-btn-italic"));
    fontRow2.appendChild(createRibbonButton("Underline", "U", () => emit("ToggleUnderline"), true, "calc-btn-underline"));
    fontRow2.appendChild(createRibbonButton("Borders", "🔲 Borders", () => emit("SetBorders"), false, "calc-btn-borders"));
    fontRow2.appendChild(createRibbonButton("Fill Color", "🪣 Fill", () => emit("SetFillColor", "#dcfce7"), false, "calc-btn-fill-color"));
    fontRow2.appendChild(createRibbonButton("Font Color", "🎨 Color", () => emit("SetFontColor", "#15803d"), false, "calc-btn-font-color"));
    fontGroup.appendChild(fontRow2);
    body.appendChild(fontGroup);

    // 3. Alignment
    const alignGroup = createRibbonGroup("Alignment");
    const alignRow1 = document.createElement("div");
    alignRow1.className = "ribbon-group-row";
    alignRow1.appendChild(createRibbonButton("Top Align", "⬆ Top", () => emit("AlignTop"), false, "calc-btn-align-top"));
    alignRow1.appendChild(createRibbonButton("Middle Align", "⬌ Mid", () => emit("AlignMiddle"), false, "calc-btn-align-middle"));
    alignRow1.appendChild(createRibbonButton("Bottom Align", "⬇ Bot", () => emit("AlignBottom"), false, "calc-btn-align-bottom"));
    alignRow1.appendChild(createRibbonButton("Wrap Text", "↩ Wrap", () => emit("ToggleWrapText"), true, "calc-btn-wrap-text"));
    alignGroup.appendChild(alignRow1);

    const alignRow2 = document.createElement("div");
    alignRow2.className = "ribbon-group-row";
    alignRow2.appendChild(createRibbonButton("Left", "Left", () => emit("AlignLeft"), false, "calc-btn-align-left"));
    alignRow2.appendChild(createRibbonButton("Center", "Center", () => emit("AlignCenter"), false, "calc-btn-align-center"));
    alignRow2.appendChild(createRibbonButton("Right", "Right", () => emit("AlignRight"), false, "calc-btn-align-right"));
    alignRow2.appendChild(createRibbonButton("Merge & Center", "⤢ Merge", () => emit("MergeAndCenter"), true, "calc-btn-merge-center"));
    alignGroup.appendChild(alignRow2);
    body.appendChild(alignGroup);

    // 4. Number
    const numGroup = createRibbonGroup("Number");
    const numRow1 = document.createElement("div");
    numRow1.className = "ribbon-group-row";
    numRow1.appendChild(
      createRibbonSelect(
        ["General", "Number", "Currency", "Accounting", "Short Date", "Percentage", "Fraction"],
        "General",
        (val) => emit("SetNumberFormat", val),
        "calc-select-number-format"
      )
    );
    numGroup.appendChild(numRow1);

    const numRow2 = document.createElement("div");
    numRow2.className = "ribbon-group-row";
    numRow2.appendChild(createRibbonButton("Currency", "$", () => emit("SetNumberFormat", "Currency"), false, "calc-btn-num-currency"));
    numRow2.appendChild(createRibbonButton("Percent", "%", () => emit("SetNumberFormat", "Percentage"), false, "calc-btn-num-percent"));
    numRow2.appendChild(createRibbonButton("Comma", ",", () => emit("SetNumberFormat", "Comma"), false, "calc-btn-num-comma"));
    numRow2.appendChild(createRibbonButton("Inc Decimal", ".00 ➔", () => emit("IncreaseDecimal"), false, "calc-btn-inc-decimal"));
    numRow2.appendChild(createRibbonButton("Dec Decimal", ".0 ➔", () => emit("DecreaseDecimal"), false, "calc-btn-dec-decimal"));
    numGroup.appendChild(numRow2);
    body.appendChild(numGroup);

    // 5. Styles
    const styleGroup = createRibbonGroup("Styles");
    const styleRow1 = document.createElement("div");
    styleRow1.className = "ribbon-group-row";
    styleRow1.appendChild(createRibbonButton("Conditional Formatting", "📊 Cond Format", () => emit("ConditionalFormatting"), false, "calc-btn-cond-format"));
    styleRow1.appendChild(createRibbonButton("Highlight >", "> Max", () => emit("ConditionalFormatting", "greaterThan"), false, "calc-btn-cond-gt"));
    styleRow1.appendChild(createRibbonButton("Highlight <", "< Min", () => emit("ConditionalFormatting", "lessThan"), false, "calc-btn-cond-lt"));
    styleGroup.appendChild(styleRow1);

    const styleRow2 = document.createElement("div");
    styleRow2.className = "ribbon-group-row";
    styleRow2.appendChild(createRibbonButton("Top 10%", "🔝 10%", () => emit("ConditionalFormatting", "top10"), false, "calc-btn-cond-top10"));
    styleRow2.appendChild(createRibbonButton("Data Bars", "📶 Bars", () => emit("ConditionalFormatting", "dataBar"), false, "calc-btn-cond-databar"));
    styleRow2.appendChild(createRibbonButton("Color Scales", "🌈 Scales", () => emit("ConditionalFormatting", "colorScale"), false, "calc-btn-cond-colorscale"));
    styleRow2.appendChild(createRibbonButton("Format as Table", "▦ Table Style", () => emit("FormatAsTable"), false, "calc-btn-format-table"));
    styleRow2.appendChild(createRibbonButton("Cell Styles", "🎨 Cell Styles", () => emit("CellStyles"), false, "calc-btn-cell-styles"));
    styleGroup.appendChild(styleRow2);
    body.appendChild(styleGroup);

    // 6. Cells
    const cellGroup = createRibbonGroup("Cells");
    const cellRow1 = document.createElement("div");
    cellRow1.className = "ribbon-group-row";
    cellRow1.appendChild(createRibbonButton("Insert Cells", "➕ Insert", () => emit("InsertCells"), false, "calc-btn-insert-cells"));
    cellRow1.appendChild(createRibbonButton("Delete Cells", "➖ Delete", () => emit("DeleteCells"), false, "calc-btn-delete-cells"));
    cellRow1.appendChild(createRibbonButton("Insert Row", "➕ Row", () => emit("InsertRow"), false, "calc-btn-insert-row"));
    cellRow1.appendChild(createRibbonButton("Insert Col", "➕ Col", () => emit("InsertColumn"), false, "calc-btn-insert-col"));
    cellGroup.appendChild(cellRow1);

    const cellRow2 = document.createElement("div");
    cellRow2.className = "ribbon-group-row";
    cellRow2.appendChild(createRibbonButton("Delete Row", "➖ Row", () => emit("DeleteRow"), false, "calc-btn-delete-row"));
    cellRow2.appendChild(createRibbonButton("Delete Col", "➖ Col", () => emit("DeleteColumn"), false, "calc-btn-delete-col"));
    cellRow2.appendChild(createRibbonButton("AutoFit Col", "↔ Auto Col", () => emit("AutoFitColumn"), false, "calc-btn-autofit-col"));
    cellRow2.appendChild(createRibbonButton("AutoFit Row", "↕ Auto Row", () => emit("AutoFitRow"), false, "calc-btn-autofit-row"));
    cellRow2.appendChild(createRibbonButton("Format Cell", "📐 Format", () => emit("FormatCells"), false, "calc-btn-format-cells"));
    cellGroup.appendChild(cellRow2);
    body.appendChild(cellGroup);

    // 7. Editing
    const editGroup = createRibbonGroup("Editing");
    const editRow1 = document.createElement("div");
    editRow1.className = "ribbon-group-row";
    editRow1.appendChild(createRibbonButton("AutoSum", "Σ AutoSum", () => emit("AutoSum"), false, "calc-btn-autosum"));
    editRow1.appendChild(createRibbonButton("Fill Down", "⬇ Down", () => emit("FillDown"), false, "calc-btn-fill-down"));
    editRow1.appendChild(createRibbonButton("Fill Right", "➔ Right", () => emit("FillRight"), false, "calc-btn-fill-right"));
    editRow1.appendChild(createRibbonButton("Fill Series", "🔢 Series", () => emit("FillSeries"), false, "calc-btn-fill-series"));
    editGroup.appendChild(editRow1);

    const editRow2 = document.createElement("div");
    editRow2.className = "ribbon-group-row";
    editRow2.appendChild(createRibbonButton("Clear All", "🧹 Clear All", () => emit("ClearAll"), false, "calc-btn-clear-all"));
    editRow2.appendChild(createRibbonButton("Clear Formats", "🎨 Clear Formats", () => emit("ClearFormats"), false, "calc-btn-clear-formats"));
    editRow2.appendChild(createRibbonButton("Clear Contents", "📄 Clear Contents", () => emit("ClearContents"), false, "calc-btn-clear-contents"));
    editRow2.appendChild(createRibbonButton("Sort & Filter", "🌪️ Sort/Filter", () => emit("ToggleFilter"), false, "calc-btn-sort-filter"));
    editRow2.appendChild(createRibbonButton("Find & Select", "🔍 Find", () => emit("Find"), false, "calc-btn-find"));
    editGroup.appendChild(editRow2);
    body.appendChild(editGroup);

    // 8. Copilot (Subphase 20.1.3)
    const copilotGroup = createRibbonGroup("Copilot");
    copilotGroup.appendChild(
      createRibbonButton("Aether Copilot", "✨ Copilot", () => emit("ToggleCopilot"), false, "calc-btn-copilot")
    );
    body.appendChild(copilotGroup);
  }

  private renderInsertTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Tables Group
    const tblGroup = createRibbonGroup("Tables");
    const tblRow = document.createElement("div");
    tblRow.className = "ribbon-group-row";
    tblRow.appendChild(createRibbonButton("PivotTable", "📊 PivotTable", () => emit("InsertPivotTable"), false, "calc-btn-pivottable"));
    tblRow.appendChild(createRibbonButton("Pivot Summary", "📋 Pivot Summary", () => emit("PivotSummary"), false, "calc-btn-pivot-summary"));
    tblRow.appendChild(createRibbonButton("PivotChart", "📈 PivotChart", () => emit("InsertPivotChart"), false, "calc-btn-pivotchart"));
    tblRow.appendChild(createRibbonButton("Slicer", "🔪 Slicer", () => emit("InsertSlicer"), false, "calc-btn-slicer"));
    tblRow.appendChild(createRibbonButton("Table", "▦ Table", () => emit("InsertTable"), false, "calc-btn-insert-table"));
    tblGroup.appendChild(tblRow);
    body.appendChild(tblGroup);

    // 2. Illustrations
    const illGroup = createRibbonGroup("Illustrations");
    const illRow = document.createElement("div");
    illRow.className = "ribbon-group-row";
    illRow.appendChild(createRibbonButton("Pictures", "🖼️ Picture", () => emit("InsertPicture"), false, "calc-btn-insert-picture"));
    illRow.appendChild(createRibbonButton("Shapes", "⬛ Shapes", () => emit("InsertShape", "rect"), false, "calc-btn-insert-shapes"));
    illGroup.appendChild(illRow);
    body.appendChild(illGroup);

    // 3. Charts Group
    const chartGroup = createRibbonGroup("Charts");
    const chartRow1 = document.createElement("div");
    chartRow1.className = "ribbon-group-row";
    chartRow1.appendChild(createRibbonButton("Column", "📊 Column", () => emit("InsertChart", "column"), false, "calc-btn-chart-column"));
    chartRow1.appendChild(createRibbonButton("Bar", "📊 Bar", () => emit("InsertChart", "bar"), false, "calc-btn-chart-bar"));
    chartRow1.appendChild(createRibbonButton("Line", "📈 Line", () => emit("InsertChart", "line"), false, "calc-btn-chart-line"));
    chartGroup.appendChild(chartRow1);

    const chartRow2 = document.createElement("div");
    chartRow2.className = "ribbon-group-row";
    chartRow2.appendChild(createRibbonButton("Pie", "🥧 Pie", () => emit("InsertChart", "pie"), false, "calc-btn-chart-pie"));
    chartRow2.appendChild(createRibbonButton("Doughnut", "🍩 Doughnut", () => emit("InsertChart", "doughnut"), false, "calc-btn-chart-doughnut"));
    chartRow2.appendChild(createRibbonButton("Scatter", "⁘ Scatter", () => emit("InsertChart", "scatter"), false, "calc-btn-chart-scatter"));
    chartRow2.appendChild(createRibbonButton("Recommended", "🌟 Recommended", () => emit("InsertChart", "column"), false, "calc-btn-chart-recommended"));
    chartGroup.appendChild(chartRow2);
    body.appendChild(chartGroup);

    // 4. Sparklines
    const sparkGroup = createRibbonGroup("Sparklines");
    const sparkRow = document.createElement("div");
    sparkRow.className = "ribbon-group-row";
    sparkRow.appendChild(createRibbonButton("Line Sparkline", "📈 Line", () => emit("InsertSparkline", "line"), false, "calc-btn-sparkline-line"));
    sparkRow.appendChild(createRibbonButton("Column Sparkline", "📊 Col", () => emit("InsertSparkline", "column"), false, "calc-btn-sparkline-column"));
    sparkRow.appendChild(createRibbonButton("Win/Loss", "↕ Win/Loss", () => emit("InsertSparkline", "winloss"), false, "calc-btn-sparkline-winloss"));
    sparkGroup.appendChild(sparkRow);
    body.appendChild(sparkGroup);

    // 5. Links & Comments
    const linkGroup = createRibbonGroup("Links & Notes");
    linkGroup.appendChild(createRibbonButton("Link", "🔗 Link", () => emit("InsertLink"), false, "calc-btn-insert-link"));
    linkGroup.appendChild(createRibbonButton("Comment", "💬 Comment", () => emit("InsertComment"), false, "calc-btn-insert-comment"));
    linkGroup.appendChild(createRibbonButton("New Note", "📝 Note", () => emit("NewNote"), false, "calc-btn-insert-note"));
    body.appendChild(linkGroup);
  }

  private renderPageLayoutTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Themes Group
    const themeGroup = createRibbonGroup("Themes");
    const themeRow = document.createElement("div");
    themeRow.className = "ribbon-group-row";
    themeRow.appendChild(createRibbonButton("Office Theme", "🎨 Themes", () => emit("SetTheme", "Office"), false, "calc-btn-themes"));
    themeRow.appendChild(createRibbonButton("Theme Colors", "Colors", () => emit("SetThemeColors", "Office"), false, "calc-btn-theme-colors"));
    themeRow.appendChild(createRibbonButton("Theme Fonts", "Fonts", () => emit("SetThemeFonts", "Calibri"), false, "calc-btn-theme-fonts"));
    themeGroup.appendChild(themeRow);
    body.appendChild(themeGroup);

    // 2. Page Setup Group
    const setupGroup = createRibbonGroup("Page Setup");
    const setupRow1 = document.createElement("div");
    setupRow1.className = "ribbon-group-row";
    setupRow1.appendChild(createRibbonButton("Normal Margins", "📐 Normal", () => emit("SetMargins", "normal"), false, "calc-btn-margins-normal"));
    setupRow1.appendChild(createRibbonButton("Narrow Margins", "📐 Narrow", () => emit("SetMargins", "narrow"), false, "calc-btn-margins-narrow"));
    setupRow1.appendChild(createRibbonButton("Wide Margins", "📐 Wide", () => emit("SetMargins", "wide"), false, "calc-btn-margins-wide"));
    setupGroup.appendChild(setupRow1);

    const setupRow2 = document.createElement("div");
    setupRow2.className = "ribbon-group-row";
    setupRow2.appendChild(createRibbonButton("Portrait", "📄 Portrait", () => emit("SetOrientation", "portrait"), false, "calc-btn-orient-portrait"));
    setupRow2.appendChild(createRibbonButton("Landscape", "📃 Landscape", () => emit("SetOrientation", "landscape"), false, "calc-btn-orient-landscape"));
    setupRow2.appendChild(createRibbonButton("Set Print Area", "🖨️ Print Area", () => emit("SetPrintArea"), false, "calc-btn-set-print-area"));
    setupRow2.appendChild(createRibbonButton("Clear Print Area", "❌ Clear Area", () => emit("ClearPrintArea"), false, "calc-btn-clear-print-area"));
    setupRow2.appendChild(createRibbonButton("Scale to Fit", "↔ Scale", () => emit("ScaleToFit"), false, "calc-btn-scale-to-fit"));
    setupGroup.appendChild(setupRow2);
    body.appendChild(setupGroup);

    // 3. Sheet Options Group
    const optGroup = createRibbonGroup("Sheet Options");
    const optRow = document.createElement("div");
    optRow.className = "ribbon-group-row";
    optRow.appendChild(createRibbonButton("View Gridlines", "☑ View Grid", () => emit("ToggleGridlines"), true, "calc-btn-toggle-grid"));
    optRow.appendChild(createRibbonButton("Print Gridlines", "☑ Print Grid", () => emit("TogglePrintGrid"), true, "calc-btn-print-grid"));
    optRow.appendChild(createRibbonButton("View Headings", "☑ View Headings", () => emit("ToggleHeadings"), true, "calc-btn-toggle-headings"));
    optRow.appendChild(createRibbonButton("Print Headings", "☑ Print Headings", () => emit("TogglePrintHeadings"), true, "calc-btn-print-headings"));
    optGroup.appendChild(optRow);
    body.appendChild(optGroup);
  }

  private renderFormulasTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Function Library
    const fnGroup = createRibbonGroup("Function Library");
    const fnRow1 = document.createElement("div");
    fnRow1.className = "ribbon-group-row";
    fnRow1.appendChild(createRibbonButton("Insert Function", "ƒx Insert", () => emit("InsertFunction"), false, "calc-btn-insert-fn"));
    fnRow1.appendChild(createRibbonButton("AutoSum", "Σ SUM", () => emit("AutoSum"), false, "calc-btn-autosum"));
    fnRow1.appendChild(createRibbonButton("Average", "x̄ AVG", () => emit("AutoAverage"), false, "calc-btn-avg"));
    fnRow1.appendChild(createRibbonButton("Count", "123 Count", () => emit("AutoCount"), false, "calc-btn-count"));
    fnRow1.appendChild(createRibbonButton("Max", "⬆ Max", () => emit("AutoMax"), false, "calc-btn-max"));
    fnRow1.appendChild(createRibbonButton("Min", "⬇ Min", () => emit("AutoMin"), false, "calc-btn-min"));
    fnRow1.appendChild(createRibbonButton("Financial", "💰 Financial", () => emit("FormulaCategory", "financial"), false, "calc-btn-fn-financial"));
    fnRow1.appendChild(createRibbonButton("Logical", "⚖️ Logical", () => emit("FormulaCategory", "logical"), false, "calc-btn-fn-logical"));
    fnGroup.appendChild(fnRow1);

    const fnRow2 = document.createElement("div");
    fnRow2.className = "ribbon-group-row";
    fnRow2.appendChild(createRibbonButton("Text Functions", "🔤 Text", () => emit("FormulaCategory", "text"), false, "calc-btn-fn-text"));
    fnRow2.appendChild(createRibbonButton("Date & Time", "📅 Date", () => emit("FormulaCategory", "datetime"), false, "calc-btn-fn-date"));
    fnRow2.appendChild(createRibbonButton("Lookup & Ref", "🔍 Lookup", () => emit("FormulaCategory", "lookup"), false, "calc-btn-fn-lookup"));
    fnRow2.appendChild(createRibbonButton("Math & Trig", "∑ Math", () => emit("FormulaCategory", "math"), false, "calc-btn-fn-math"));
    fnGroup.appendChild(fnRow2);
    body.appendChild(fnGroup);

    // 2. Defined Names
    const nameGroup = createRibbonGroup("Defined Names");
    const nameRow = document.createElement("div");
    nameRow.className = "ribbon-group-row";
    nameRow.appendChild(createRibbonButton("Name Manager", "🏷️ Name Manager", () => emit("NameManager"), false, "calc-btn-name-manager"));
    nameRow.appendChild(createRibbonButton("Define Name", "➕ Define Name", () => emit("DefineName"), false, "calc-btn-define-name"));
    nameGroup.appendChild(nameRow);
    body.appendChild(nameGroup);

    // 3. Formula Auditing
    const auditGroup = createRibbonGroup("Formula Auditing");
    const auditRow = document.createElement("div");
    auditRow.className = "ribbon-group-row";
    auditRow.appendChild(createRibbonButton("Trace Precedents", "🔍 Trace Precedents", () => emit("TracePrecedents"), false, "calc-btn-trace-precedents"));
    auditRow.appendChild(createRibbonButton("Trace Dependents", "🔍 Trace Dependents", () => emit("TraceDependents"), false, "calc-btn-trace-dependents"));
    auditRow.appendChild(createRibbonButton("Remove Arrows", "❌ Remove Arrows", () => emit("RemoveArrows"), false, "calc-btn-remove-arrows"));
    auditRow.appendChild(createRibbonButton("Show Formulas", "🧮 Show Formulas", () => emit("ShowFormulas"), true, "calc-btn-show-formulas"));
    auditRow.appendChild(createRibbonButton("Evaluate", "⚡ Evaluate", () => emit("EvaluateFormula"), false, "calc-btn-evaluate"));
    auditGroup.appendChild(auditRow);
    body.appendChild(auditGroup);

    // 4. Calculation
    const calcGroup = createRibbonGroup("Calculation");
    calcGroup.appendChild(createRibbonButton("Calculate Now", "⚡ Calculate (F9)", () => emit("Recalculate"), false, "calc-btn-calculate-now"));
    calcGroup.appendChild(createRibbonButton("Calc Sheet", "Calc Sheet", () => emit("CalculateSheet"), false, "calc-btn-calc-sheet"));
    body.appendChild(calcGroup);
  }

  private renderDataTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Get & Transform Data
    const getGroup = createRibbonGroup("Get & Transform Data");
    const getRow = document.createElement("div");
    getRow.className = "ribbon-group-row";
    getRow.appendChild(createRibbonButton("From Text/CSV", "📄 From CSV", () => emit("ImportCsv"), false, "calc-btn-from-csv"));
    getRow.appendChild(createRibbonButton("From Web", "🌐 From Web", () => emit("ImportWeb"), false, "calc-btn-from-web"));
    getRow.appendChild(createRibbonButton("Refresh All", "🔄 Refresh All", () => emit("RefreshData"), false, "calc-btn-refresh-all"));
    getGroup.appendChild(getRow);
    body.appendChild(getGroup);

    // 2. Sort & Filter
    const sortGroup = createRibbonGroup("Sort & Filter");
    const sortRow = document.createElement("div");
    sortRow.className = "ribbon-group-row";
    sortRow.appendChild(createRibbonButton("Sort A to Z", "🔼 Sort A-Z", () => emit("SortAsc"), false, "calc-btn-sort-asc"));
    sortRow.appendChild(createRibbonButton("Sort Z to A", "🔽 Sort Z-A", () => emit("SortDesc"), false, "calc-btn-sort-desc"));
    sortRow.appendChild(createRibbonButton("Custom Sort", "↕ Custom Sort", () => emit("CustomSort"), false, "calc-btn-custom-sort"));
    sortRow.appendChild(createRibbonButton("Filter", "🌪️ Filter", () => emit("ToggleFilter"), true, "calc-btn-toggle-filter"));
    sortRow.appendChild(createRibbonButton("Clear Filter", "Clear", () => emit("ClearFilter"), false, "calc-btn-clear-filter"));
    sortGroup.appendChild(sortRow);
    body.appendChild(sortGroup);

    // 3. Data Tools
    const toolsGroup = createRibbonGroup("Data Tools");
    const toolsRow = document.createElement("div");
    toolsRow.className = "ribbon-group-row";
    toolsRow.appendChild(createRibbonButton("Text to Columns", "🔀 Text to Columns", () => emit("TextToColumns"), false, "calc-btn-text-to-columns"));
    toolsRow.appendChild(createRibbonButton("Flash Fill", "⚡ Flash Fill", () => emit("FlashFill"), false, "calc-btn-flash-fill"));
    toolsRow.appendChild(createRibbonButton("Remove Duplicates", "🗑️ Remove Dups", () => emit("RemoveDuplicates"), false, "calc-btn-remove-duplicates"));
    toolsRow.appendChild(createRibbonButton("Data Validation", "✓ Data Validation", () => emit("DataValidation"), false, "calc-btn-data-validation"));
    toolsGroup.appendChild(toolsRow);
    body.appendChild(toolsGroup);

    // 4. Outline & Solver
    const outGroup = createRibbonGroup("Outline & What-If");
    const outRow = document.createElement("div");
    outRow.className = "ribbon-group-row";
    outRow.appendChild(createRibbonButton("Group", "⊞ Group", () => emit("GroupRows"), false, "calc-btn-group"));
    outRow.appendChild(createRibbonButton("Ungroup", "⊟ Ungroup", () => emit("UngroupRows"), false, "calc-btn-ungroup"));
    outRow.appendChild(createRibbonButton("Subtotal", "∑ Subtotal", () => emit("Subtotal"), false, "calc-btn-subtotal"));
    outRow.appendChild(createRibbonButton("Goal Seek", "🎯 Goal Seek", () => emit("GoalSeek"), false, "calc-btn-goal-seek"));
    outGroup.appendChild(outRow);
    body.appendChild(outGroup);
  }

  private renderReviewTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Proofing
    const proofGroup = createRibbonGroup("Proofing");
    const proofRow = document.createElement("div");
    proofRow.className = "ribbon-group-row";
    proofRow.appendChild(createRibbonButton("Spelling", "✓ Spelling", () => emit("CheckSpelling"), false, "calc-btn-spelling"));
    proofRow.appendChild(createRibbonButton("Thesaurus", "📖 Thesaurus", () => emit("Thesaurus"), false, "calc-btn-thesaurus"));
    proofGroup.appendChild(proofRow);
    body.appendChild(proofGroup);

    // 2. Protect
    const protGroup = createRibbonGroup("Protect");
    const protRow = document.createElement("div");
    protRow.className = "ribbon-group-row";
    protRow.appendChild(createRibbonButton("Protect Sheet", "🔒 Protect Sheet", () => emit("ProtectSheet"), false, "calc-btn-protect-sheet"));
    protRow.appendChild(createRibbonButton("Protect Workbook", "🔒 Protect Workbook", () => emit("ProtectWorkbook"), false, "calc-btn-protect-workbook"));
    protRow.appendChild(createRibbonButton("Allow Edit Ranges", "🔓 Edit Ranges", () => emit("AllowEditRanges"), false, "calc-btn-allow-edit-ranges"));
    protGroup.appendChild(protRow);
    body.appendChild(protGroup);

    // 3. Comments & Notes
    const noteGroup = createRibbonGroup("Notes & Stats");
    noteGroup.appendChild(createRibbonButton("New Note", "💬 New Note", () => emit("NewNote"), false, "calc-btn-new-note"));
    noteGroup.appendChild(createRibbonButton("Show All Notes", "Show All Notes", () => emit("ShowAllNotes"), true, "calc-btn-show-all-notes"));
    noteGroup.appendChild(createRibbonButton("Workbook Statistics", "📊 Workbook Stats", () => emit("WorkbookStatistics"), false, "calc-btn-workbook-stats"));
    body.appendChild(noteGroup);

    // 4. Copilot (Subphase 20.1.3)
    const copilotReviewGroup = createRibbonGroup("Copilot");
    copilotReviewGroup.appendChild(
      createRibbonButton("Aether Copilot", "✨ Copilot", () => emit("ToggleCopilot"), false, "calc-btn-copilot-review")
    );
    body.appendChild(copilotReviewGroup);
  }

  private renderViewTab(body: HTMLElement, emit: RibbonActionEmitter) {
    // 1. Workbook Views
    const viewsGroup = createRibbonGroup("Workbook Views");
    const viewsRow = document.createElement("div");
    viewsRow.className = "ribbon-group-row";
    viewsRow.appendChild(createRibbonButton("Normal View", "▦ Normal", () => emit("ViewNormal"), false, "calc-btn-view-normal"));
    viewsRow.appendChild(createRibbonButton("Page Break Preview", "📑 Page Break", () => emit("ViewPageBreak"), false, "calc-btn-view-pagebreak"));
    viewsRow.appendChild(createRibbonButton("Page Layout", "📄 Page Layout", () => emit("ViewPageLayout"), false, "calc-btn-view-pagelayout"));
    viewsGroup.appendChild(viewsRow);
    body.appendChild(viewsGroup);

    // 2. Show Group
    const showGroup = createRibbonGroup("Show");
    const showRow = document.createElement("div");
    showRow.className = "ribbon-group-row";
    showRow.appendChild(createRibbonButton("Formula Bar", "☑ Formula Bar", () => emit("ToggleFormulaBar"), true, "calc-btn-toggle-formula-bar"));
    showRow.appendChild(createRibbonButton("Gridlines", "☑ Gridlines", () => emit("ToggleGridlines"), true, "calc-btn-toggle-gridlines"));
    showRow.appendChild(createRibbonButton("Headings", "☑ Headings", () => emit("ToggleHeadings"), true, "calc-btn-toggle-headings"));
    showGroup.appendChild(showRow);
    body.appendChild(showGroup);

    // 3. Zoom
    const zoomGroup = createRibbonGroup("Zoom");
    const zoomRow = document.createElement("div");
    zoomRow.className = "ribbon-group-row";
    zoomRow.appendChild(createRibbonButton("100%", "🔍 100%", () => emit("Zoom100"), false, "calc-btn-zoom-100"));
    zoomRow.appendChild(createRibbonButton("Zoom to Selection", "🎯 Selection", () => emit("ZoomToSelection"), false, "calc-btn-zoom-selection"));
    zoomRow.appendChild(createRibbonButton("Zoom In", "+ In", () => emit("ZoomIn"), false, "calc-btn-zoom-in"));
    zoomRow.appendChild(createRibbonButton("Zoom Out", "- Out", () => emit("ZoomOut"), false, "calc-btn-zoom-out"));
    zoomGroup.appendChild(zoomRow);
    body.appendChild(zoomGroup);

    // 4. Window
    const winGroup = createRibbonGroup("Window");
    const winRow = document.createElement("div");
    winRow.className = "ribbon-group-row";
    winRow.appendChild(createRibbonButton("Freeze Panes", "❄️ Freeze Panes", () => emit("FreezePanes"), false, "calc-btn-freeze-panes"));
    winRow.appendChild(createRibbonButton("Freeze Top Row", "Top Row", () => emit("FreezeTopRow"), false, "calc-btn-freeze-top-row"));
    winRow.appendChild(createRibbonButton("Freeze First Col", "First Col", () => emit("FreezeFirstColumn"), false, "calc-btn-freeze-first-col"));
    winRow.appendChild(createRibbonButton("Unfreeze Panes", "Unfreeze", () => emit("UnfreezePanes"), false, "calc-btn-unfreeze-panes"));
    winRow.appendChild(createRibbonButton("Split", "✂️ Split", () => emit("SplitWindow"), true, "calc-btn-split-window"));
    winGroup.appendChild(winRow);
    body.appendChild(winGroup);
  }
}

