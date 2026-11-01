/**
 * WorkspaceLayoutManager.ts
 * GEC-v7.0 Stage 42 (Sub-Phase 42.1.2)
 *
 * Modular workspace layout controller responsible for:
 * - Multi-pane split docking (2-way side-by-side & vertical splits)
 * - Sidebar, Sub-header, and App Rail layout coordination
 * - Viewport layout state tracking and DOM updates
 */

import { DocumentType } from "../ipc";
import { TabItem } from "../components/TabBar";
import { SplitPaneGrid, SplitGridLayoutMode } from "../components/SplitPaneGrid";

export type SplitOrientation = "horizontal" | "vertical";

export interface WorkspaceLayoutOptions {
  onToggleSplit?: (active: boolean) => void;
  onSelectSecondaryTab?: (tabId: string) => void;
  onFocusPaneSlot?: (slotId: string, tabId: string | null, docType: DocumentType | null) => void;
}

export class WorkspaceLayoutManager {
  private isSplitViewActive = false;
  private splitRightTabId: string | null = null;
  private splitPaneEl: HTMLElement | null = null;
  private splitOrientation: SplitOrientation = "horizontal";
  private gridLayoutMode: SplitGridLayoutMode = "single";
  private splitGrid?: SplitPaneGrid;
  private showNavPane = true;
  private appFilter: DocumentType | null = null;
  private options: WorkspaceLayoutOptions;

  constructor(options: WorkspaceLayoutOptions = {}) {
    this.options = options;
  }

  public isSplitView(): boolean {
    return this.isSplitViewActive;
  }

  public isSplitViewActiveMode(): boolean {
    return this.isSplitViewActive;
  }

  public isSplitViewOpen(): boolean {
    return this.isSplitViewActive;
  }

  public getSplitRightTabId(): string | null {
    return this.splitRightTabId;
  }

  public setSplitRightTabId(id: string | null): void {
    this.splitRightTabId = id;
  }

  public getSplitOrientation(): SplitOrientation {
    return this.splitOrientation;
  }

  public setSplitOrientation(orientation: SplitOrientation): void {
    this.splitOrientation = orientation;
    if (this.splitPaneEl) {
      if (orientation === "vertical") {
        this.splitPaneEl.classList.add("split-vertical");
      } else {
        this.splitPaneEl.classList.remove("split-vertical");
      }
    }
  }

  public getShowNavPane(): boolean {
    return this.showNavPane;
  }

  public setShowNavPane(show: boolean): void {
    this.showNavPane = show;
  }

  public getAppFilter(): DocumentType | null {
    return this.appFilter;
  }

  public setAppFilter(filter: DocumentType | null): void {
    this.appFilter = filter;
  }

  public getSplitGrid(): SplitPaneGrid | undefined {
    return this.splitGrid;
  }

  public getSplitGridLayoutMode(): SplitGridLayoutMode {
    return this.gridLayoutMode;
  }

  public setSplitGridLayoutMode(mode: SplitGridLayoutMode, tabs: TabItem[] = []): void {
    this.gridLayoutMode = mode;
    this.isSplitViewActive = mode !== "single";

    if (mode === "single") {
      this.splitOrientation = "horizontal";
      if (this.splitGrid) {
        this.splitGrid.setLayoutMode("single");
      }
    } else if (mode === "split-horizontal") {
      this.splitOrientation = "horizontal";
      if (this.splitGrid) {
        this.splitGrid.setLayoutMode("split-horizontal");
      }
    } else if (mode === "split-vertical") {
      this.splitOrientation = "vertical";
      if (this.splitGrid) {
        this.splitGrid.setLayoutMode("split-vertical");
      }
    } else if (mode === "split-quad") {
      if (this.splitGrid) {
        this.splitGrid.setLayoutMode("split-quad");
      }
    }

    if (this.splitGrid && tabs.length > 0) {
      this.splitGrid.updateTabs(tabs);
    }
  }

  public mountSplitGrid(container: HTMLElement, tabs: TabItem[] = []): SplitPaneGrid {
    this.splitGrid = new SplitPaneGrid(container, {
      onFocusSlot: (slotId, tabId, docType) => {
        this.options.onFocusPaneSlot?.(slotId, tabId, docType);
      },
      onSelectSlotTab: (slotId, tabId) => {
        if (slotId !== "pane-0") {
          this.splitRightTabId = tabId;
        }
        this.options.onSelectSecondaryTab?.(tabId);
      },
      onCloseSlot: () => {
        this.setSplitGridLayoutMode("single");
      },
    });

    this.splitGrid.setLayoutMode(this.gridLayoutMode);
    this.splitGrid.updateTabs(tabs);
    return this.splitGrid;
  }

  public toggleSplitView(
    tabs: TabItem[] = [],
    activeTab: TabItem | null = null,
    onSecondaryChange?: (tabId: string) => void
  ): boolean {
    this.isSplitViewActive = !this.isSplitViewActive;

    const splitBtn = document.getElementById("btn-split-view");
    if (splitBtn) {
      if (this.isSplitViewActive) {
        splitBtn.classList.add("active");
      } else {
        splitBtn.classList.remove("active");
      }
    }

    const layout = document.querySelector(".workspace-layout");
    if (!layout) {
      this.options.onToggleSplit?.(this.isSplitViewActive);
      return this.isSplitViewActive;
    }

    if (this.isSplitViewActive) {
      layout.classList.add("split-view-active");
      this.renderSplitPane(layout as HTMLElement, tabs, activeTab, onSecondaryChange);
    } else {
      layout.classList.remove("split-view-active");
      if (this.splitPaneEl) {
        this.splitPaneEl.remove();
        this.splitPaneEl = null;
      }
    }

    this.options.onToggleSplit?.(this.isSplitViewActive);
    return this.isSplitViewActive;
  }

  public renderSplitPane(
    layout: HTMLElement,
    tabs: TabItem[] = [],
    activeTab: TabItem | null = null,
    onSecondaryChange?: (tabId: string) => void
  ): HTMLElement {
    if (!this.splitPaneEl) {
      this.splitPaneEl = document.createElement("div");
      this.splitPaneEl.id = "workspace-split-pane";
      this.splitPaneEl.className = "workspace-split-pane";
      this.splitPaneEl.dataset.testid = "workspace-split-pane";
      layout.appendChild(this.splitPaneEl);
    }

    this.splitPaneEl.innerHTML = "";
    const header = document.createElement("div");
    header.className = "workspace-split-header";
    header.innerHTML = `<span>◫ Split View (Right Pane)</span>`;

    const select = document.createElement("select");
    select.className = "split-pane-select";
    select.dataset.testid = "split-pane-select";

    const otherTabs = tabs.filter((t) => t.id !== activeTab?.id);

    if (otherTabs.length === 0) {
      const opt = document.createElement("option");
      opt.textContent = "No secondary tab (Open another file)";
      select.appendChild(opt);
    } else {
      otherTabs.forEach((t) => {
        const opt = document.createElement("option");
        opt.value = t.id;
        opt.textContent = `[${t.docType}] ${t.title}`;
        if (t.id === this.splitRightTabId) opt.selected = true;
        select.appendChild(opt);
      });
      if (!this.splitRightTabId && otherTabs[0]) {
        this.splitRightTabId = otherTabs[0].id;
      }
    }

    select.onchange = () => {
      this.splitRightTabId = select.value;
      const targetTab = tabs.find((t) => t.id === this.splitRightTabId);
      this.renderSplitContent(targetTab);
      if (onSecondaryChange) {
        onSecondaryChange(select.value);
      }
      this.options.onSelectSecondaryTab?.(select.value);
    };

    const closeBtn = document.createElement("button");
    closeBtn.className = "split-pane-close";
    closeBtn.textContent = "✕";
    closeBtn.title = "Close Split View";
    closeBtn.onclick = () => {
      this.toggleSplitView(tabs, activeTab, onSecondaryChange);
    };

    header.appendChild(select);
    header.appendChild(closeBtn);
    this.splitPaneEl.appendChild(header);

    const content = document.createElement("div");
    content.className = "workspace-split-content";
    content.id = "workspace-split-content";
    this.splitPaneEl.appendChild(content);

    const targetTab = tabs.find((t) => t.id === this.splitRightTabId);
    this.renderSplitContent(targetTab);

    return this.splitPaneEl;
  }

  public renderSplitContent(targetTab?: TabItem | null): void {
    const content = document.getElementById("workspace-split-content");
    if (!content) return;
    content.innerHTML = "";

    if (!targetTab) {
      content.innerHTML = `<div style="padding: 24px; color: var(--text-muted); text-align: center;">Open or create another document to view side-by-side.</div>`;
      return;
    }

    const previewCard = document.createElement("div");
    previewCard.className = "split-doc-preview";
    previewCard.style.padding = "24px";
    previewCard.style.backgroundColor = "var(--bg-surface)";
    previewCard.style.borderRadius = "8px";
    previewCard.style.border = "1px solid var(--border-color)";
    previewCard.style.width = "90%";
    previewCard.style.boxShadow = "var(--shadow-md)";

    previewCard.innerHTML = `
      <h3 style="margin-top:0; color: var(--text-primary); font-size: 16px; border-bottom: 1px solid var(--border-color); padding-bottom: 8px;">
        📄 [${targetTab.docType}] ${targetTab.title}
      </h3>
      <p style="color: var(--text-secondary); font-size: 13px; line-height: 1.6;">
        Side-by-Side Split View Active. Editing enabled across synchronized tabs.
      </p>
    `;
    content.appendChild(previewCard);
  }

  public setSubHeaderVisibility(visible: boolean): void {
    const subHeader = document.getElementById("sub-header-container");
    if (subHeader) {
      subHeader.style.display = visible ? "flex" : "none";
    }
  }

  public setSidebarVisibility(visible: boolean): void {
    const sidebar = document.getElementById("sidebar-container");
    if (sidebar) {
      sidebar.style.display = visible ? "flex" : "none";
      if (!visible) {
        sidebar.innerHTML = "";
      }
    }
  }

  public setHubMode(isHub: boolean): void {
    const appContainer = document.getElementById("app");
    if (appContainer) {
      if (isHub) {
        appContainer.classList.add("shell-hub-mode");
      } else {
        appContainer.classList.remove("shell-hub-mode");
      }
    }
  }

  public destroy(): void {
    if (this.splitPaneEl) {
      this.splitPaneEl.remove();
      this.splitPaneEl = null;
    }
    this.isSplitViewActive = false;
    this.splitRightTabId = null;
  }
}

