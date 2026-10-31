import { DocumentType } from "../ipc";

export interface TabItem {
  id: string;
  docType: DocumentType;
  title: string;
  isDirty: boolean;
  file_path?: string;
}

export class TabBar {
  private container: HTMLElement;
  private tabs: TabItem[] = [];
  private activeTabId: string | null = null;
  private filterDocType: DocumentType | null = null;
  private filterMode: "current" | "all" = "all";
  private onTabChangeCallback?: (tab: TabItem) => void;
  private onTabCloseCallback?: (id: string) => void;
  private onNewTabCallback?: (docType?: DocumentType) => void;
  private onOpenTabCallback?: (docType?: DocumentType, file?: File) => void;
  private onTabPopoutCallback?: (tab: TabItem) => void;
  private outsideClickListener?: (e: MouseEvent) => void;

  constructor(containerId: string) {
    const el = document.getElementById(containerId);
    if (!el) throw new Error(`TabBar container #${containerId} not found`);
    this.container = el;
    this.render();
  }

  public setCallbacks(callbacks: {
    onTabChange?: (tab: TabItem) => void;
    onTabClose?: (id: string) => void;
    onNewTab?: (docType?: DocumentType) => void;
    onOpenTab?: (docType?: DocumentType, file?: File) => void;
    onTabPopout?: (tab: TabItem) => void;
  }) {
    this.onTabChangeCallback = callbacks.onTabChange;
    this.onTabCloseCallback = callbacks.onTabClose;
    this.onNewTabCallback = callbacks.onNewTab;
    this.onOpenTabCallback = callbacks.onOpenTab;
    this.onTabPopoutCallback = callbacks.onTabPopout;
  }

  public setFilterDocType(docType: DocumentType | null) {
    this.filterDocType = docType;
    if (docType) {
      this.filterMode = "current";
    } else {
      this.filterMode = "all";
    }
    this.render();
  }

  public getFilterDocType(): DocumentType | null {
    return this.filterDocType;
  }

  public setFilterMode(mode: "current" | "all") {
    this.filterMode = mode;
    this.render();
  }

  public getFilterMode(): "current" | "all" {
    return this.filterMode;
  }

  public addTab(tab: TabItem) {
    this.tabs.push(tab);
    this.activeTabId = tab.id;
    if (this.filterMode === "current") {
      this.filterDocType = tab.docType;
    }
    this.render();
    if (this.onTabChangeCallback) this.onTabChangeCallback(tab);
  }

  public switchTab(id: string) {
    const target = this.tabs.find((t) => t.id === id);
    if (target && this.activeTabId !== id) {
      this.activeTabId = id;
      if (this.filterMode === "current") {
        this.filterDocType = target.docType;
      }
      this.render();
      if (this.onTabChangeCallback) this.onTabChangeCallback(target);
    }
  }

  public closeTab(id: string) {
    const index = this.tabs.findIndex((t) => t.id === id);
    if (index === -1) return;

    this.tabs.splice(index, 1);
    if (this.activeTabId === id) {
      if (this.tabs.length > 0) {
        const nextActive = this.tabs[Math.min(index, this.tabs.length - 1)];
        this.activeTabId = nextActive.id;
        if (this.filterMode === "current") {
          this.filterDocType = nextActive.docType;
        }
        if (this.onTabChangeCallback) this.onTabChangeCallback(nextActive);
      } else {
        this.activeTabId = null;
      }
    }
    this.render();
    if (this.onTabCloseCallback) this.onTabCloseCallback(id);
  }

  public setDirty(id: string, isDirty: boolean) {
    const target = this.tabs.find((t) => t.id === id);
    if (target && target.isDirty !== isDirty) {
      target.isDirty = isDirty;
      this.render();
    }
  }

  public updateTab(id: string, updates: Partial<TabItem>) {
    const target = this.tabs.find((t) => t.id === id);
    if (target) {
      Object.assign(target, updates);
      this.render();
    }
  }

  public nextTab() {
    if (this.tabs.length <= 1) return;
    const currIdx = this.tabs.findIndex((t) => t.id === this.activeTabId);
    if (currIdx !== -1) {
      const nextIdx = (currIdx + 1) % this.tabs.length;
      this.switchTab(this.tabs[nextIdx].id);
    }
  }

  public prevTab() {
    if (this.tabs.length <= 1) return;
    const currIdx = this.tabs.findIndex((t) => t.id === this.activeTabId);
    if (currIdx !== -1) {
      const prevIdx = currIdx === 0 ? this.tabs.length - 1 : currIdx - 1;
      this.switchTab(this.tabs[prevIdx].id);
    }
  }

  public getActiveTab(): TabItem | undefined {
    return this.tabs.find((t) => t.id === this.activeTabId);
  }

  public getTabs(): TabItem[] {
    return [...this.tabs];
  }

  public render() {
    this.container.innerHTML = "";

    // 1. Tab Filter Mode Toggle (Pill: App Tabs | All Tabs (N))
    const filterToggleBtn = document.createElement("button");
    filterToggleBtn.className = `tab-filter-toggle ${this.filterMode === "all" ? "active" : ""}`;
    filterToggleBtn.dataset.testid = "tab-filter-toggle";
    filterToggleBtn.title = "Toggle Tab Bar view: Current App Tabs vs All Open Tabs";
    filterToggleBtn.innerHTML = `
      <span class="pill-app ${this.filterMode === "current" ? "pill-active" : ""}">App Tabs</span>
      <span class="pill-divider">|</span>
      <span class="pill-all ${this.filterMode === "all" ? "pill-active" : ""}">All Tabs (${this.tabs.length})</span>
    `;
    filterToggleBtn.onclick = (e) => {
      e.stopPropagation();
      this.filterMode = this.filterMode === "all" ? "current" : "all";
      this.render();
    };
    this.container.appendChild(filterToggleBtn);

    // 2. Visible Tabs
    const visibleTabs =
      this.filterMode === "current" && this.filterDocType
        ? this.tabs.filter((t) => t.docType === this.filterDocType)
        : this.tabs;

    visibleTabs.forEach((tab) => {
      const tabEl = document.createElement("div");
      tabEl.className = `tab-item ${tab.id === this.activeTabId ? "active" : ""}`;
      tabEl.onclick = () => this.switchTab(tab.id);

      const typeClass =
        tab.docType === "Word"
          ? "tab-type-word"
          : tab.docType === "Excel"
          ? "tab-type-excel"
          : tab.docType === "PowerPoint"
          ? "tab-type-ppt"
          : tab.docType === "PDF"
          ? "tab-type-pdf"
          : "tab-type-notes";

      const iconChar =
        tab.docType === "Word"
          ? "W"
          : tab.docType === "Excel"
          ? "X"
          : tab.docType === "PowerPoint"
          ? "P"
          : tab.docType === "PDF"
          ? "PDF"
          : "N";

      const icon = document.createElement("span");
      icon.className = `tab-type-icon ${typeClass}`;
      icon.textContent = iconChar;

      const label = document.createElement("span");
      label.className = "tab-title";
      label.textContent = tab.isDirty ? `${tab.title} *` : tab.title;

      const popoutBtn = document.createElement("button");
      popoutBtn.className = "tab-popout-btn";
      popoutBtn.dataset.testid = "tab-popout-btn";
      popoutBtn.title = "Open in New Window ↗";
      popoutBtn.textContent = "↗";
      popoutBtn.onclick = (e) => {
        e.stopPropagation();
        if (this.onTabPopoutCallback) {
          this.onTabPopoutCallback(tab);
        }
      };

      const closeBtn = document.createElement("button");
      closeBtn.className = "tab-close-btn";
      closeBtn.textContent = "×";
      closeBtn.onclick = (e) => {
        e.stopPropagation();
        this.closeTab(tab.id);
      };

      tabEl.appendChild(icon);
      tabEl.appendChild(label);
      tabEl.appendChild(popoutBtn);
      tabEl.appendChild(closeBtn);
      this.container.appendChild(tabEl);
    });

    // 3. Quick + New and 📂 Open actions right next to tabs
    const currentType = this.filterDocType || "Word";
    const extInfo: Record<DocumentType, { ext: string; accept: string }> = {
      Word: { ext: ".docx", accept: ".docx" },
      Excel: { ext: ".xlsx", accept: ".xlsx,.xls,.csv" },
      PowerPoint: { ext: ".pptx", accept: ".pptx,.ppt" },
      PDF: { ext: ".pdf", accept: ".pdf" },
      Notes: { ext: ".txt", accept: ".txt,.md,.json,.log" },
    };
    const info = extInfo[currentType] || { ext: ".docx", accept: ".docx" };
    const cleanExt = info.ext.replace(".", "");

    // Quick New button: + New .<ext>
    const quickNewBtn = document.createElement("button");
    quickNewBtn.className = "tab-quick-btn tab-quick-new";
    quickNewBtn.dataset.testid = `tab-new-${cleanExt}`;
    quickNewBtn.title = `Create New ${currentType} (${info.ext})`;
    quickNewBtn.innerHTML = `+ New ${info.ext}`;
    quickNewBtn.onclick = (e) => {
      e.stopPropagation();
      if (this.onNewTabCallback) {
        this.onNewTabCallback(currentType);
      }
    };
    this.container.appendChild(quickNewBtn);

    // Quick Open button: 📂 Open .<ext>
    const quickOpenBtn = document.createElement("button");
    quickOpenBtn.className = "tab-quick-btn tab-quick-open";
    quickOpenBtn.dataset.testid = `tab-open-${cleanExt}`;
    quickOpenBtn.title = `Open ${currentType} from Disk (${info.ext})`;
    quickOpenBtn.innerHTML = `📂 Open ${info.ext}`;

    const quickFileInput = document.createElement("input");
    quickFileInput.type = "file";
    quickFileInput.accept = info.accept;
    quickFileInput.style.display = "none";
    quickFileInput.dataset.testid = `tab-file-input-${cleanExt}`;
    quickFileInput.onchange = () => {
      const file = quickFileInput.files?.[0];
      if (file && this.onOpenTabCallback) {
        this.onOpenTabCallback(currentType, file);
      }
    };
    quickOpenBtn.onclick = (e) => {
      e.stopPropagation();
      quickFileInput.click();
    };
    this.container.appendChild(quickOpenBtn);
    this.container.appendChild(quickFileInput);

    // 4. Add Tab Container with New Document Dropdown Menu (+ icon)
    const addContainer = document.createElement("div");
    addContainer.className = "tab-add-container";
    addContainer.style.position = "relative";
    addContainer.style.display = "inline-flex";
    addContainer.style.alignItems = "center";

    const addBtn = document.createElement("button");
    addBtn.className = "tab-add-btn";
    addBtn.dataset.testid = "tab-add-btn";
    addBtn.textContent = "+";
    addBtn.title = "New Document (Click for all formats)";

    const dropdown = document.createElement("div");
    dropdown.className = "tab-new-dropdown";
    dropdown.dataset.testid = "tab-new-dropdown";
    dropdown.style.display = "none";

    const options: { docType: DocumentType; label: string; icon: string; cls: string }[] = [
      { docType: "Word", label: "Blank Word Document (.docx)", icon: "W", cls: "tab-type-word" },
      { docType: "Excel", label: "Blank Excel Workbook (.xlsx)", icon: "X", cls: "tab-type-excel" },
      { docType: "PowerPoint", label: "Blank PowerPoint Presentation (.pptx)", icon: "P", cls: "tab-type-ppt" },
      { docType: "PDF", label: "Open / Blank PDF (.pdf)", icon: "PDF", cls: "tab-type-pdf" },
      { docType: "Notes", label: "Blank Note / Markdown (.txt/.md)", icon: "N", cls: "tab-type-notes" },
    ];

    options.forEach((opt) => {
      const item = document.createElement("button");
      item.className = "tab-dropdown-item";
      item.dataset.newType = opt.docType;
      item.dataset.testid = `dropdown-new-${opt.docType.toLowerCase()}`;
      item.innerHTML = `<span class="tab-type-icon ${opt.cls}">${opt.icon}</span> <span>${opt.label}</span>`;
      item.onclick = (e) => {
        e.stopPropagation();
        dropdown.style.display = "none";
        if (this.onNewTabCallback) this.onNewTabCallback(opt.docType);
      };
      dropdown.appendChild(item);
    });

    addBtn.onclick = (e) => {
      e.stopPropagation();
      const isVisible = dropdown.style.display === "block";
      dropdown.style.display = isVisible ? "none" : "block";
    };

    if (this.outsideClickListener) {
      document.removeEventListener("click", this.outsideClickListener);
    }
    this.outsideClickListener = () => {
      dropdown.style.display = "none";
    };
    document.addEventListener("click", this.outsideClickListener);

    addContainer.appendChild(addBtn);
    addContainer.appendChild(dropdown);
    this.container.appendChild(addContainer);
  }
}
