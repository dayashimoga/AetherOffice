import { DocumentType } from "../ipc";

export type RibbonTabId =
  | "File"
  | "Home"
  | "Insert"
  | "Layout"
  | "Formulas"
  | "Data"
  | "Transitions"
  | "Animations"
  | "SlideShow"
  | "Review"
  | "View";

export class Ribbon {
  private container: HTMLElement;
  private activeTab: RibbonTabId = "Home";
  private docType: DocumentType = "Word";
  private keytipsActive = false;
  private onActionCallback?: (action: string, payload?: unknown) => void;

  constructor(containerId: string) {
    const el = document.getElementById(containerId);
    if (!el) throw new Error(`Ribbon container #${containerId} not found`);
    this.container = el;
    this.render();
  }

  public setDocumentType(docType: DocumentType) {
    this.docType = docType;
    if (!this.getVisibleTabs().includes(this.activeTab)) {
      this.activeTab = "Home";
    }
    this.render();
  }

  public setOnAction(callback: (action: string, payload?: unknown) => void) {
    this.onActionCallback = callback;
  }

  public setActiveTab(tab: RibbonTabId) {
    if (this.getVisibleTabs().includes(tab)) {
      this.activeTab = tab;
      this.render();
    }
  }

  public toggleKeytips(active?: boolean): boolean {
    this.keytipsActive = active !== undefined ? active : !this.keytipsActive;
    this.render();
    return this.keytipsActive;
  }

  public getVisibleTabs(): RibbonTabId[] {
    switch (this.docType) {
      case "Word":
        return ["File", "Home", "Insert", "Layout", "Review", "View"];
      case "Excel":
        return ["File", "Home", "Insert", "Layout", "Formulas", "Data", "Review", "View"];
      case "PowerPoint":
        return ["File", "Home", "Insert", "Layout", "Transitions", "Animations", "SlideShow", "Review", "View"];
      case "PDF":
        return ["File", "Home", "Insert", "Review", "View"];
    }
  }

  private getKeytipCode(tab: RibbonTabId): string {
    switch (tab) {
      case "File": return "F";
      case "Home": return "H";
      case "Insert": return "N";
      case "Layout": return "P";
      case "Formulas": return "M";
      case "Data": return "A";
      case "Transitions": return "T";
      case "Animations": return "K";
      case "SlideShow": return "S";
      case "Review": return "R";
      case "View": return "W";
    }
  }

  private render() {
    this.container.innerHTML = "";

    // 1. Ribbon Tabs Header
    const header = document.createElement("div");
    header.className = "ribbon-tabs-header";

    this.getVisibleTabs().forEach((tab) => {
      const tabBtn = document.createElement("button");
      tabBtn.className = `ribbon-tab-btn ${tab === this.activeTab ? "active" : ""}`;
      tabBtn.textContent = tab === "SlideShow" ? "Slide Show" : tab;
      tabBtn.onclick = () => this.setActiveTab(tab);

      if (this.keytipsActive) {
        const badge = document.createElement("span");
        badge.className = "ribbon-keytip-badge";
        badge.textContent = this.getKeytipCode(tab);
        tabBtn.appendChild(badge);
      }

      header.appendChild(tabBtn);
    });

    this.container.appendChild(header);

    // 2. Ribbon Tools Body
    const body = document.createElement("div");
    body.className = "ribbon-tools-body";

    switch (this.activeTab) {
      case "Home":
        this.renderHomeTab(body);
        break;
      case "Insert":
        this.renderInsertTab(body);
        break;
      case "Layout":
        this.renderLayoutTab(body);
        break;
      case "Formulas":
        this.renderFormulasTab(body);
        break;
      case "Data":
        this.renderDataTab(body);
        break;
      case "Transitions":
        this.renderTransitionsTab(body);
        break;
      case "Animations":
        this.renderAnimationsTab(body);
        break;
      case "SlideShow":
        this.renderSlideShowTab(body);
        break;
      case "Review":
        this.renderReviewTab(body);
        break;
      case "View":
        this.renderViewTab(body);
        break;
      default:
        this.renderHomeTab(body);
    }

    this.container.appendChild(body);
  }

  private renderHomeTab(body: HTMLElement) {
    // Clipboard Group
    const clipGroup = this.createGroup("Clipboard");
    clipGroup.appendChild(this.createButton("Paste", "📋", () => this.emit("Paste")));
    const rowClip = document.createElement("div");
    rowClip.className = "ribbon-group-row";
    rowClip.appendChild(this.createButton("Cut", "✂️", () => this.emit("Cut")));
    rowClip.appendChild(this.createButton("Copy", "📄", () => this.emit("Copy")));
    clipGroup.appendChild(rowClip);
    body.appendChild(clipGroup);

    // Font Group
    const fontGroup = this.createGroup("Font");
    const fontRow1 = document.createElement("div");
    fontRow1.className = "ribbon-group-row";

    const fontSelect = document.createElement("select");
    fontSelect.className = "ribbon-select";
    ["Calibri", "Aptos", "Arial", "Times New Roman", "Consolas"].forEach((f) => {
      const opt = document.createElement("option");
      opt.value = f;
      opt.textContent = f;
      fontSelect.appendChild(opt);
    });
    fontSelect.onchange = () => this.emit("SetFontFamily", fontSelect.value);

    const sizeSelect = document.createElement("select");
    sizeSelect.className = "ribbon-select";
    [9, 10, 11, 12, 14, 16, 18, 24, 32].forEach((sz) => {
      const opt = document.createElement("option");
      opt.value = String(sz);
      opt.textContent = String(sz);
      if (sz === 11) opt.selected = true;
      sizeSelect.appendChild(opt);
    });
    sizeSelect.onchange = () => this.emit("SetFontSize", parseFloat(sizeSelect.value));

    fontRow1.appendChild(fontSelect);
    fontRow1.appendChild(sizeSelect);
    fontGroup.appendChild(fontRow1);

    const fontRow2 = document.createElement("div");
    fontRow2.className = "ribbon-group-row";
    fontRow2.appendChild(this.createButton("B", "B", () => this.emit("ToggleBold"), true));
    fontRow2.appendChild(this.createButton("I", "I", () => this.emit("ToggleItalic"), true));
    fontRow2.appendChild(this.createButton("U", "U", () => this.emit("ToggleUnderline"), true));
    fontRow2.appendChild(this.createButton("S", "S", () => this.emit("ToggleStrikethrough"), true));
    fontRow2.appendChild(this.createButton("A+", "A+", () => this.emit("IncreaseFontSize")));
    fontRow2.appendChild(this.createButton("A-", "A-", () => this.emit("DecreaseFontSize")));
    fontRow2.appendChild(this.createButton("Clear", "🧹", () => this.emit("ClearFormatting")));
    fontGroup.appendChild(fontRow2);
    body.appendChild(fontGroup);

    // Paragraph / Alignment Group
    const alignGroup = this.createGroup("Paragraph");
    const paraRow1 = document.createElement("div");
    paraRow1.className = "ribbon-group-row";
    paraRow1.appendChild(this.createButton("Bullets", "•≡", () => this.emit("ToggleBulletList")));
    paraRow1.appendChild(this.createButton("Numbering", "1≡", () => this.emit("ToggleNumberedList")));
    paraRow1.appendChild(this.createButton("DecIndent", "⇤", () => this.emit("DecreaseIndent")));
    paraRow1.appendChild(this.createButton("IncIndent", "⇥", () => this.emit("IncreaseIndent")));
    alignGroup.appendChild(paraRow1);

    const paraRow2 = document.createElement("div");
    paraRow2.className = "ribbon-group-row";
    paraRow2.appendChild(this.createButton("Left", "Left", () => this.emit("AlignLeft")));
    paraRow2.appendChild(this.createButton("Center", "Center", () => this.emit("AlignCenter")));
    paraRow2.appendChild(this.createButton("Right", "Right", () => this.emit("AlignRight")));
    paraRow2.appendChild(this.createButton("Justify", "Justify", () => this.emit("AlignJustify")));
    alignGroup.appendChild(paraRow2);
    body.appendChild(alignGroup);

    // Styles Group
    const stylesGroup = this.createGroup("Styles");
    const stylesRow = document.createElement("div");
    stylesRow.className = "ribbon-group-row";
    stylesRow.appendChild(this.createButton("Normal", "Normal", () => this.emit("SetStyle", "Normal")));
    stylesRow.appendChild(this.createButton("H1", "Heading 1", () => this.emit("SetStyle", "Heading 1")));
    stylesRow.appendChild(this.createButton("H2", "Heading 2", () => this.emit("SetStyle", "Heading 2")));
    stylesRow.appendChild(this.createButton("Title", "Title", () => this.emit("SetStyle", "Title")));
    stylesGroup.appendChild(stylesRow);
    body.appendChild(stylesGroup);
  }

  private renderInsertTab(body: HTMLElement) {
    const tableGroup = this.createGroup("Tables");
    tableGroup.appendChild(this.createButton("Insert Table", "📊 Table", () => this.emit("InsertTable", { rows: 3, cols: 3 })));
    body.appendChild(tableGroup);

    const illustGroup = this.createGroup("Illustrations");
    const illustRow = document.createElement("div");
    illustRow.className = "ribbon-group-row";
    illustRow.appendChild(this.createButton("Picture", "🖼️ Picture", () => this.emit("InsertPicture")));
    illustRow.appendChild(this.createButton("Shapes", "🔷 Shapes", () => this.emit("InsertShape", "rect")));
    illustGroup.appendChild(illustRow);
    body.appendChild(illustGroup);

    const textGroup = this.createGroup("Text");
    textGroup.appendChild(this.createButton("Text Box", "🔤 Text Box", () => this.emit("InsertTextBox")));
    body.appendChild(textGroup);
  }

  private renderLayoutTab(body: HTMLElement) {
    const pageGroup = this.createGroup("Page Setup");
    const pageRow1 = document.createElement("div");
    pageRow1.className = "ribbon-group-row";
    pageRow1.appendChild(this.createButton("Normal", "📐 Normal", () => this.emit("SetMargins", "normal")));
    pageRow1.appendChild(this.createButton("Narrow", "📐 Narrow", () => this.emit("SetMargins", "narrow")));
    pageRow1.appendChild(this.createButton("Wide", "📐 Wide", () => this.emit("SetMargins", "wide")));
    pageGroup.appendChild(pageRow1);

    const pageRow2 = document.createElement("div");
    pageRow2.className = "ribbon-group-row";
    pageRow2.appendChild(this.createButton("Portrait", "📄 Portrait", () => this.emit("SetOrientation", "portrait")));
    pageRow2.appendChild(this.createButton("Landscape", "📃 Landscape", () => this.emit("SetOrientation", "landscape")));
    pageRow2.appendChild(this.createButton("Letter", "Letter", () => this.emit("SetPageSize", "letter")));
    pageRow2.appendChild(this.createButton("A4", "A4", () => this.emit("SetPageSize", "a4")));
    pageGroup.appendChild(pageRow2);
    body.appendChild(pageGroup);

    const paraGroup = this.createGroup("Paragraph");
    const paraRow = document.createElement("div");
    paraRow.className = "ribbon-group-row";
    paraRow.appendChild(this.createButton("Indent", "⇥ Indent", () => this.emit("IncreaseIndent")));
    paraRow.appendChild(this.createButton("Outdent", "⇤ Outdent", () => this.emit("DecreaseIndent")));
    paraRow.appendChild(this.createButton("1.0", "1.0", () => this.emit("SetLineSpacing", 1.0)));
    paraRow.appendChild(this.createButton("1.5", "1.5", () => this.emit("SetLineSpacing", 1.5)));
    paraGroup.appendChild(paraRow);
    body.appendChild(paraGroup);
  }

  private renderFormulasTab(body: HTMLElement) {
    const fnGroup = this.createGroup("Function Library");
    const fnRow1 = document.createElement("div");
    fnRow1.className = "ribbon-group-row";
    fnRow1.appendChild(this.createButton("Insert Function", "ƒx Insert", () => this.emit("InsertFunction")));
    fnRow1.appendChild(this.createButton("AutoSum", "Σ AutoSum", () => this.emit("AutoSum")));
    fnRow1.appendChild(this.createButton("Financial", "💰 Financial", () => this.emit("FormulaCategory", "financial")));
    fnGroup.appendChild(fnRow1);
    body.appendChild(fnGroup);

    const auditGroup = this.createGroup("Formula Auditing");
    const auditRow = document.createElement("div");
    auditRow.className = "ribbon-group-row";
    auditRow.appendChild(this.createButton("Trace Precedents", "🔍 Trace Precedents", () => this.emit("TracePrecedents")));
    auditRow.appendChild(this.createButton("Evaluate Formula", "🧮 Evaluate", () => this.emit("EvaluateFormula")));
    auditGroup.appendChild(auditRow);
    body.appendChild(auditGroup);

    const calcGroup = this.createGroup("Calculation");
    calcGroup.appendChild(this.createButton("Calculate Now", "⚡ Calculate Now (F9)", () => this.emit("Recalculate")));
    body.appendChild(calcGroup);
  }

  private renderDataTab(body: HTMLElement) {
    const sortGroup = this.createGroup("Sort & Filter");
    const sortRow = document.createElement("div");
    sortRow.className = "ribbon-group-row";
    sortRow.appendChild(this.createButton("Sort A-Z", "🔼 Sort A-Z", () => this.emit("SortAsc")));
    sortRow.appendChild(this.createButton("Sort Z-A", "🔽 Sort Z-A", () => this.emit("SortDesc")));
    sortRow.appendChild(this.createButton("Filter", "🌪️ Filter", () => this.emit("ToggleFilter")));
    sortGroup.appendChild(sortRow);
    body.appendChild(sortGroup);

    const toolGroup = this.createGroup("Data Tools");
    toolGroup.appendChild(this.createButton("Flash Fill", "⚡ Flash Fill", () => this.emit("FlashFill")));
    body.appendChild(toolGroup);
  }

  private renderTransitionsTab(body: HTMLElement) {
    const transGroup = this.createGroup("Transition to This Slide");
    const transRow = document.createElement("div");
    transRow.className = "ribbon-group-row";
    ["None", "Morph", "Fade", "Push", "Wipe"].forEach((t) => {
      transRow.appendChild(this.createButton(t, t, () => this.emit("SetTransition", t)));
    });
    transGroup.appendChild(transRow);
    body.appendChild(transGroup);

    const timingGroup = this.createGroup("Timing");
    timingGroup.appendChild(this.createButton("Apply To All", "🔄 Apply To All", () => this.emit("ApplyTransitionToAll")));
    body.appendChild(timingGroup);
  }

  private renderAnimationsTab(body: HTMLElement) {
    const animGroup = this.createGroup("Animation");
    const animRow = document.createElement("div");
    animRow.className = "ribbon-group-row";
    ["Appear", "Fade", "Fly In", "Float In", "Split"].forEach((a) => {
      animRow.appendChild(this.createButton(a, a, () => this.emit("SetAnimation", a)));
    });
    animGroup.appendChild(animRow);
    body.appendChild(animGroup);

    const advGroup = this.createGroup("Advanced Animation");
    advGroup.appendChild(this.createButton("Animation Pane", "🎞️ Animation Pane", () => this.emit("ToggleAnimationPane")));
    body.appendChild(advGroup);
  }

  private renderSlideShowTab(body: HTMLElement) {
    const showGroup = this.createGroup("Start Slide Show");
    const showRow = document.createElement("div");
    showRow.className = "ribbon-group-row";
    showRow.appendChild(this.createButton("From Beginning", "▶️ From Beginning (F5)", () => this.emit("StartShowBeginning")));
    showRow.appendChild(this.createButton("From Current", "⏩ From Current Slide", () => this.emit("StartShowCurrent")));
    showGroup.appendChild(showRow);
    body.appendChild(showGroup);

    const setupGroup = this.createGroup("Set Up");
    const setupRow = document.createElement("div");
    setupRow.className = "ribbon-group-row";
    setupRow.appendChild(this.createButton("Hide Slide", "👁️‍🗨️ Hide Slide", () => this.emit("HideSlide")));
    setupRow.appendChild(this.createButton("Rehearse Timings", "⏱️ Rehearse", () => this.emit("RehearseTimings")));
    setupGroup.appendChild(setupRow);
    body.appendChild(setupGroup);
  }

  private renderReviewTab(body: HTMLElement) {
    const proofGroup = this.createGroup("Proofing");
    const proofRow = document.createElement("div");
    proofRow.className = "ribbon-group-row";
    proofRow.appendChild(this.createButton("Spelling", "✓ Spelling & Grammar", () => this.emit("CheckSpelling")));
    proofRow.appendChild(this.createButton("Word Count", "🔢 Word Count", () => this.emit("WordCount")));
    proofGroup.appendChild(proofRow);
    body.appendChild(proofGroup);

    const commentGroup = this.createGroup("Comments");
    commentGroup.appendChild(this.createButton("New Comment", "💬 New Comment", () => this.emit("NewComment")));
    body.appendChild(commentGroup);
  }

  private renderViewTab(body: HTMLElement) {
    const viewGroup = this.createGroup("Views");
    const viewRow = document.createElement("div");
    viewRow.className = "ribbon-group-row";
    viewRow.appendChild(this.createButton("Normal", "📄 Normal", () => this.emit("ViewNormal")));
    viewRow.appendChild(this.createButton("Outline", "📑 Outline", () => this.emit("ViewOutline")));
    viewGroup.appendChild(viewRow);
    body.appendChild(viewGroup);

    const showGroup = this.createGroup("Show");
    const showRow = document.createElement("div");
    showRow.className = "ribbon-group-row";
    showRow.appendChild(this.createButton("Ruler", "📏 Ruler", () => this.emit("ToggleRuler"), true));
    showRow.appendChild(this.createButton("Gridlines", "🔲 Gridlines", () => this.emit("ToggleGridlines"), true));
    showGroup.appendChild(showRow);
    body.appendChild(showGroup);

    const zoomGroup = this.createGroup("Zoom");
    const zoomRow = document.createElement("div");
    zoomRow.className = "ribbon-group-row";
    zoomRow.appendChild(this.createButton("100%", "🔍 100%", () => this.emit("Zoom100")));
    zoomRow.appendChild(this.createButton("Fit Window", "↔️ Fit Window", () => this.emit("ZoomFit")));
    zoomGroup.appendChild(zoomRow);
    body.appendChild(zoomGroup);
  }

  private createGroup(title: string): HTMLElement {
    const group = document.createElement("div");
    group.className = "ribbon-group";
    const titleEl = document.createElement("span");
    titleEl.className = "ribbon-group-title";
    titleEl.textContent = title;
    group.appendChild(titleEl);
    return group;
  }

  private createButton(label: string, icon: string, onClick: () => void, isToggle = false): HTMLElement {
    const btn = document.createElement("button");
    btn.className = "ribbon-btn";
    btn.title = label;
    btn.textContent = icon;
    btn.onclick = () => {
      if (isToggle) btn.classList.toggle("active");
      onClick();
    };
    return btn;
  }

  private emit(action: string, payload?: unknown) {
    if (this.onActionCallback) {
      this.onActionCallback(action, payload);
    }
  }
}
