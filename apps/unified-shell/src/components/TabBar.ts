import { DocumentType } from "../ipc";

export interface TabItem {
  id: string;
  docType: DocumentType;
  title: string;
  isDirty: boolean;
}

export class TabBar {
  private container: HTMLElement;
  private tabs: TabItem[] = [];
  private activeTabId: string | null = null;
  private onTabChangeCallback?: (tab: TabItem) => void;
  private onTabCloseCallback?: (id: string) => void;
  private onNewTabCallback?: () => void;

  constructor(containerId: string) {
    const el = document.getElementById(containerId);
    if (!el) throw new Error(`TabBar container #${containerId} not found`);
    this.container = el;
    this.render();
  }

  public setCallbacks(callbacks: {
    onTabChange?: (tab: TabItem) => void;
    onTabClose?: (id: string) => void;
    onNewTab?: () => void;
  }) {
    this.onTabChangeCallback = callbacks.onTabChange;
    this.onTabCloseCallback = callbacks.onTabClose;
    this.onNewTabCallback = callbacks.onNewTab;
  }

  public addTab(tab: TabItem) {
    this.tabs.push(tab);
    this.activeTabId = tab.id;
    this.render();
    if (this.onTabChangeCallback) this.onTabChangeCallback(tab);
  }

  public switchTab(id: string) {
    const target = this.tabs.find((t) => t.id === id);
    if (target && this.activeTabId !== id) {
      this.activeTabId = id;
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

  private render() {
    this.container.innerHTML = "";

    this.tabs.forEach((tab) => {
      const tabEl = document.createElement("div");
      tabEl.className = `tab-item ${tab.id === this.activeTabId ? "active" : ""}`;
      tabEl.onclick = () => this.switchTab(tab.id);

      const typeClass =
        tab.docType === "Word"
          ? "tab-type-word"
          : tab.docType === "Excel"
          ? "tab-type-excel"
          : "tab-type-ppt";

      const iconChar = tab.docType === "Word" ? "W" : tab.docType === "Excel" ? "X" : "P";

      const icon = document.createElement("span");
      icon.className = `tab-type-icon ${typeClass}`;
      icon.textContent = iconChar;

      const label = document.createElement("span");
      label.className = "tab-title";
      label.textContent = tab.isDirty ? `${tab.title} *` : tab.title;

      const closeBtn = document.createElement("button");
      closeBtn.className = "tab-close-btn";
      closeBtn.textContent = "×";
      closeBtn.onclick = (e) => {
        e.stopPropagation();
        this.closeTab(tab.id);
      };

      tabEl.appendChild(icon);
      tabEl.appendChild(label);
      tabEl.appendChild(closeBtn);
      this.container.appendChild(tabEl);
    });

    const addBtn = document.createElement("button");
    addBtn.className = "tab-add-btn";
    addBtn.textContent = "+";
    addBtn.title = "New Document";
    addBtn.onclick = () => {
      if (this.onNewTabCallback) this.onNewTabCallback();
    };
    this.container.appendChild(addBtn);
  }
}

