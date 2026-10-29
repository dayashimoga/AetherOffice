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

    if (this.activeTab === "Home") {
      this.renderHomeTab(body);
    } else {
      const placeholder = document.createElement("div");
      placeholder.style.color = "#8a8a8a";
      placeholder.style.padding = "10px";
      placeholder.textContent = `${this.activeTab} commands for ${this.docType}`;
      body.appendChild(placeholder);
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
    fontGroup.appendChild(fontRow2);
    body.appendChild(fontGroup);

    // Paragraph / Alignment Group
    const alignGroup = this.createGroup("Alignment");
    const alignRow = document.createElement("div");
    alignRow.className = "ribbon-group-row";
    alignRow.appendChild(this.createButton("Left", "Left", () => this.emit("AlignLeft")));
    alignRow.appendChild(this.createButton("Center", "Center", () => this.emit("AlignCenter")));
    alignRow.appendChild(this.createButton("Right", "Right", () => this.emit("AlignRight")));
    alignRow.appendChild(this.createButton("Justify", "Justify", () => this.emit("AlignJustify")));
    alignGroup.appendChild(alignRow);
    body.appendChild(alignGroup);
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
