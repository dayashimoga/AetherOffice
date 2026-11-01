/**
 * SplitPaneGrid.ts
 * GEC-v7.0 Stage 42 (Sub-Phase 42.2.2)
 *
 * Dockable 2-Way / 4-Way Multi-Document Split-Pane Grid:
 * - Supports single, 2-way horizontal (side-by-side), 2-way vertical, and 4-way quad (2x2) layouts
 * - Allows concurrent rendering and editing of .docx, .xlsx, .pptx, and .pdf documents
 * - Focus-driven pane switching with contextual ribbon and command routing
 */

import { DocumentType } from "../ipc";
import { TabItem } from "./TabBar";

export type SplitGridLayoutMode = "single" | "split-horizontal" | "split-vertical" | "split-quad";

export interface PaneSlot {
  slotId: string;
  tabId: string | null;
  docType: DocumentType | null;
  title: string;
  containerEl: HTMLElement;
  viewportEl: HTMLElement;
}

export interface SplitPaneGridCallbacks {
  onFocusSlot?: (slotId: string, tabId: string | null, docType: DocumentType | null) => void;
  onSelectSlotTab?: (slotId: string, tabId: string) => void;
  onCloseSlot?: (slotId: string) => void;
}

export class SplitPaneGrid {
  private container: HTMLElement;
  private layoutMode: SplitGridLayoutMode = "single";
  private slots: Map<string, PaneSlot> = new Map();
  private focusedSlotId = "pane-0";
  private tabs: TabItem[] = [];
  private callbacks: SplitPaneGridCallbacks;

  constructor(
    containerIdOrElement: string | HTMLElement,
    callbacks: SplitPaneGridCallbacks = {}
  ) {
    if (typeof containerIdOrElement === "string") {
      const el = document.getElementById(containerIdOrElement);
      if (!el) throw new Error(`SplitPaneGrid container #${containerIdOrElement} not found`);
      this.container = el;
    } else {
      this.container = containerIdOrElement;
    }
    this.callbacks = callbacks;
    this.initDefaultSlots();
    this.render();
  }

  private initDefaultSlots(): void {
    // Up to 4 slots for quad split
    for (let i = 0; i < 4; i++) {
      const slotId = `pane-${i}`;
      const slotContainer = document.createElement("div");
      slotContainer.className = "split-pane-slot";
      slotContainer.dataset.slotId = slotId;
      slotContainer.dataset.testid = `split-pane-slot-${i}`;

      const viewportEl = document.createElement("div");
      viewportEl.className = "split-pane-viewport";
      viewportEl.id = `split-viewport-${i}`;
      viewportEl.dataset.testid = `split-viewport-${i}`;

      this.slots.set(slotId, {
        slotId,
        tabId: null,
        docType: null,
        title: `Pane ${i + 1}`,
        containerEl: slotContainer,
        viewportEl,
      });
    }
  }

  public getLayoutMode(): SplitGridLayoutMode {
    return this.layoutMode;
  }

  public setLayoutMode(mode: SplitGridLayoutMode): void {
    this.layoutMode = mode;
    this.render();
  }

  public getFocusedSlotId(): string {
    return this.focusedSlotId;
  }

  public setFocusedSlot(slotId: string): void {
    if (!this.slots.has(slotId)) return;
    this.focusedSlotId = slotId;

    for (const [id, slot] of this.slots.entries()) {
      if (id === slotId) {
        slot.containerEl.classList.add("pane-focused");
      } else {
        slot.containerEl.classList.remove("pane-focused");
      }
    }

    const slot = this.slots.get(slotId);
    if (slot) {
      this.callbacks.onFocusSlot?.(slotId, slot.tabId, slot.docType);
    }
  }

  public updateTabs(tabs: TabItem[]): void {
    this.tabs = [...tabs];
    // If pane-0 has no tab, assign first tab
    const slot0 = this.slots.get("pane-0");
    if (slot0 && !slot0.tabId && tabs.length > 0) {
      this.setSlotTab("pane-0", tabs[0].id);
    }
    this.render();
  }

  public setSlotTab(slotId: string, tabId: string): void {
    const slot = this.slots.get(slotId);
    if (!slot) return;

    const targetTab = this.tabs.find((t) => t.id === tabId);
    if (targetTab) {
      slot.tabId = targetTab.id;
      slot.docType = targetTab.docType;
      slot.title = targetTab.title;
    } else {
      slot.tabId = null;
      slot.docType = null;
      slot.title = `Empty Slot`;
    }

    if (slotId === this.focusedSlotId) {
      this.callbacks.onFocusSlot?.(slotId, slot.tabId, slot.docType);
    }
    this.render();
  }

  public getSlot(slotId: string): PaneSlot | undefined {
    return this.slots.get(slotId);
  }

  public getSlotViewport(slotId: string): HTMLElement | undefined {
    return this.slots.get(slotId)?.viewportEl;
  }

  public getActiveSlots(): PaneSlot[] {
    const count =
      this.layoutMode === "single"
        ? 1
        : this.layoutMode === "split-quad"
        ? 4
        : 2;

    const result: PaneSlot[] = [];
    for (let i = 0; i < count; i++) {
      const slot = this.slots.get(`pane-${i}`);
      if (slot) result.push(slot);
    }
    return result;
  }

  public render(): void {
    this.container.innerHTML = "";
    this.container.className = `split-pane-grid-container layout-${this.layoutMode}`;
    this.container.dataset.testid = "split-pane-grid";

    // Set CSS grid styles according to layout mode
    this.container.style.display = "grid";
    this.container.style.width = "100%";
    this.container.style.height = "100%";
    this.container.style.gap = "2px";
    this.container.style.backgroundColor = "var(--border-color)";
    this.container.style.boxSizing = "border-box";

    if (this.layoutMode === "single") {
      this.container.style.gridTemplateColumns = "1fr";
      this.container.style.gridTemplateRows = "1fr";
    } else if (this.layoutMode === "split-horizontal") {
      this.container.style.gridTemplateColumns = "1fr 1fr";
      this.container.style.gridTemplateRows = "1fr";
    } else if (this.layoutMode === "split-vertical") {
      this.container.style.gridTemplateColumns = "1fr";
      this.container.style.gridTemplateRows = "1fr 1fr";
    } else if (this.layoutMode === "split-quad") {
      this.container.style.gridTemplateColumns = "1fr 1fr";
      this.container.style.gridTemplateRows = "1fr 1fr";
    }

    const activeSlots = this.getActiveSlots();

    activeSlots.forEach((slot, index) => {
      const slotEl = slot.containerEl;
      slotEl.innerHTML = "";
      slotEl.style.display = "flex";
      slotEl.style.flexDirection = "column";
      slotEl.style.backgroundColor = "var(--bg-canvas)";
      slotEl.style.overflow = "hidden";
      slotEl.style.position = "relative";

      if (slot.slotId === this.focusedSlotId) {
        slotEl.classList.add("pane-focused");
        slotEl.style.outline = "2px solid var(--accent-primary)";
      } else {
        slotEl.classList.remove("pane-focused");
        slotEl.style.outline = "none";
      }

      slotEl.onclick = () => {
        this.setFocusedSlot(slot.slotId);
      };

      // Header bar for each pane slot (only when in multi-pane mode)
      if (this.layoutMode !== "single") {
        const header = document.createElement("div");
        header.className = "pane-slot-header";
        header.style.display = "flex";
        header.style.alignItems = "center";
        header.style.justifyContent = "space-between";
        header.style.padding = "4px 8px";
        header.style.backgroundColor = "var(--bg-surface)";
        header.style.borderBottom = "1px solid var(--border-color)";
        header.style.fontSize = "11px";

        // Tab Selector dropdown
        const select = document.createElement("select");
        select.className = "pane-slot-tab-select";
        select.dataset.testid = `pane-slot-select-${index}`;
        select.style.fontSize = "11px";
        select.style.padding = "2px 4px";
        select.style.borderRadius = "3px";
        select.style.border = "1px solid var(--border-color)";
        select.style.backgroundColor = "var(--bg-canvas)";
        select.style.color = "var(--text-primary)";

        if (this.tabs.length === 0) {
          const opt = document.createElement("option");
          opt.textContent = "No document";
          select.appendChild(opt);
        } else {
          this.tabs.forEach((tab) => {
            const opt = document.createElement("option");
            opt.value = tab.id;
            opt.textContent = `[${tab.docType}] ${tab.title}`;
            if (tab.id === slot.tabId) opt.selected = true;
            select.appendChild(opt);
          });
        }

        select.onchange = (e) => {
          e.stopPropagation();
          this.setSlotTab(slot.slotId, select.value);
          this.callbacks.onSelectSlotTab?.(slot.slotId, select.value);
        };

        const titleSpan = document.createElement("span");
        titleSpan.style.fontWeight = "600";
        titleSpan.style.color = "var(--text-primary)";
        titleSpan.textContent = `Pane ${index + 1}:`;

        const leftGroup = document.createElement("div");
        leftGroup.style.display = "flex";
        leftGroup.style.alignItems = "center";
        leftGroup.style.gap = "6px";
        leftGroup.appendChild(titleSpan);
        leftGroup.appendChild(select);

        const closeBtn = document.createElement("button");
        closeBtn.className = "pane-slot-close-btn";
        closeBtn.textContent = "✕";
        closeBtn.title = "Close Split Pane";
        closeBtn.style.background = "none";
        closeBtn.style.border = "none";
        closeBtn.style.cursor = "pointer";
        closeBtn.style.color = "var(--text-muted)";
        closeBtn.style.fontSize = "11px";
        closeBtn.onclick = (e) => {
          e.stopPropagation();
          this.setLayoutMode("single");
          this.callbacks.onCloseSlot?.(slot.slotId);
        };

        header.appendChild(leftGroup);
        header.appendChild(closeBtn);
        slotEl.appendChild(header);
      }

      // Slot Viewport
      const viewport = slot.viewportEl;
      viewport.style.flex = "1";
      viewport.style.width = "100%";
      viewport.style.height = "100%";
      viewport.style.position = "relative";
      viewport.style.overflow = "hidden";

      // If slot has no content yet, render placeholder card
      if (!viewport.hasChildNodes() || viewport.children.length === 0) {
        if (slot.tabId) {
          viewport.innerHTML = `
            <div class="split-pane-preview-card" style="padding: 24px; text-align: center; color: var(--text-secondary);">
              <div style="font-size: 24px; margin-bottom: 8px;">📄</div>
              <div style="font-weight: 600; font-size: 14px; color: var(--text-primary); margin-bottom: 4px;">
                [${slot.docType || "Doc"}] ${slot.title}
              </div>
              <div style="font-size: 12px; color: var(--text-muted);">
                Split view synchronized. Click to focus and edit.
              </div>
            </div>
          `;
        } else {
          viewport.innerHTML = `
            <div style="padding: 24px; text-align: center; color: var(--text-muted); font-size: 12px;">
              Select a document to display in this split pane.
            </div>
          `;
        }
      }

      slotEl.appendChild(viewport);
      this.container.appendChild(slotEl);
    });
  }

  public destroy(): void {
    this.container.innerHTML = "";
    this.slots.clear();
  }
}

