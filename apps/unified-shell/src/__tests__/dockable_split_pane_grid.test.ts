import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { SplitPaneGrid } from "../components/SplitPaneGrid";
import { WorkspaceLayoutManager } from "../controllers";
import { TabItem } from "../components/TabBar";
import { UnifiedShellApp } from "../main";

describe("Sub-Phase 42.2.2: Dockable 2-Way / 4-Way Split-Pane Editor Grid", () => {
  let gridContainer: HTMLElement;

  beforeEach(() => {
    const mockCtx = {
      fillRect: () => {},
      strokeRect: () => {},
      clearRect: () => {},
      fillText: () => {},
      strokeText: () => {},
      beginPath: () => {},
      closePath: () => {},
      moveTo: () => {},
      lineTo: () => {},
      stroke: () => {},
      fill: () => {},
      save: () => {},
      restore: () => {},
      clip: () => {},
      rect: () => {},
      scale: () => {},
      setTransform: () => {},
      translate: () => {},
      rotate: () => {},
      arc: () => {},
      roundRect: () => {},
      ellipse: () => {},
      measureText: (txt: string) => ({ width: txt.length * 8 } as TextMetrics),
      setLineDash: () => {},
      font: "14px Segoe UI, sans-serif",
      fillStyle: "#000",
      strokeStyle: "#000",
      lineWidth: 1,
      textAlign: "left",
      textBaseline: "top",
    };
    HTMLCanvasElement.prototype.getContext = (() => mockCtx) as any;
    if (!HTMLCanvasElement.prototype.scrollIntoView) {
      HTMLCanvasElement.prototype.scrollIntoView = () => {};
    }

    document.body.innerHTML = `
      <div id="app">
        <header class="title-bar">
          <div class="title-actions"></div>
          <div class="window-controls">
            <button class="win-btn win-min">−</button>
            <button class="win-btn win-max">□</button>
            <button class="win-btn win-close">×</button>
          </div>
        </header>
        <aside id="app-rail-container" class="app-rail"></aside>
        <div id="tab-bar-container"></div>
        <div id="ribbon-container"></div>
        <div id="sub-header-container"></div>
        <div class="workspace-layout">
          <div id="sidebar-container"></div>
          <div id="viewport-container">
            <div id="split-grid-container"></div>
          </div>
        </div>
        <div id="copilot-sidebar-container"></div>
        <div id="bottom-sheet"></div>
        <canvas id="stylus-ink-canvas"></canvas>
        <div id="reflow-reader-container"></div>
        <div id="row-card-inspector-container"></div>
        <div id="laser-pointer-dot"></div>
        <div id="animation-pane-container"></div>
        <div id="collab-presence-bar"></div>
        <div id="status-bar">
          <span id="status-word-count"></span>
          <span id="status-doc-info"></span>
          <input type="range" id="zoom-slider" />
          <span id="status-zoom-level"></span>
        </div>
      </div>
    `;
    gridContainer = document.getElementById("split-grid-container") as HTMLElement;
  });

  afterEach(() => {
    document.body.innerHTML = "";
    vi.restoreAllMocks();
  });

  describe("SplitPaneGrid Core Engine", () => {
    it("initializes with single pane layout and updates to horizontal split", () => {
      const grid = new SplitPaneGrid(gridContainer);

      expect(grid.getLayoutMode()).toBe("single");
      expect(grid.getActiveSlots().length).toBe(1);

      const slot0 = document.querySelector('[data-testid="split-pane-slot-0"]');
      expect(slot0).not.toBeNull();
      expect(document.querySelector('[data-testid="split-pane-slot-1"]')).toBeNull();

      // Switch to horizontal 2-way split
      grid.setLayoutMode("split-horizontal");
      expect(grid.getLayoutMode()).toBe("split-horizontal");
      expect(grid.getActiveSlots().length).toBe(2);

      expect(document.querySelector('[data-testid="split-pane-slot-0"]')).not.toBeNull();
      expect(document.querySelector('[data-testid="split-pane-slot-1"]')).not.toBeNull();

      grid.destroy();
    });

    it("supports vertical 2-way split and quad 4-way split layouts", () => {
      const grid = new SplitPaneGrid(gridContainer);

      // Vertical split
      grid.setLayoutMode("split-vertical");
      expect(grid.getLayoutMode()).toBe("split-vertical");
      expect(grid.getActiveSlots().length).toBe(2);

      // Quad 4-way split (2x2)
      grid.setLayoutMode("split-quad");
      expect(grid.getLayoutMode()).toBe("split-quad");
      expect(grid.getActiveSlots().length).toBe(4);

      for (let i = 0; i < 4; i++) {
        expect(document.querySelector(`[data-testid="split-pane-slot-${i}"]`)).not.toBeNull();
      }

      grid.destroy();
    });

    it("assigns tabs across slots and tracks focus switching with callbacks", () => {
      const onFocusSlot = vi.fn();
      const onSelectSlotTab = vi.fn();
      const grid = new SplitPaneGrid(gridContainer, { onFocusSlot, onSelectSlotTab });

      const tabs: TabItem[] = [
        { id: "tab-w", title: "Document.docx", docType: "Word", isDirty: false },
        { id: "tab-x", title: "Spreadsheet.xlsx", docType: "Excel", isDirty: false },
        { id: "tab-p", title: "Slides.pptx", docType: "PowerPoint", isDirty: false },
        { id: "tab-pdf", title: "Manual.pdf", docType: "PDF", isDirty: false },
      ];

      grid.updateTabs(tabs);
      grid.setLayoutMode("split-quad");

      grid.setSlotTab("pane-0", "tab-w");
      grid.setSlotTab("pane-1", "tab-x");
      grid.setSlotTab("pane-2", "tab-p");
      grid.setSlotTab("pane-3", "tab-pdf");

      expect(grid.getSlot("pane-0")?.docType).toBe("Word");
      expect(grid.getSlot("pane-1")?.docType).toBe("Excel");
      expect(grid.getSlot("pane-2")?.docType).toBe("PowerPoint");
      expect(grid.getSlot("pane-3")?.docType).toBe("PDF");

      // Focus pane-1 (Excel)
      grid.setFocusedSlot("pane-1");
      expect(grid.getFocusedSlotId()).toBe("pane-1");
      expect(onFocusSlot).toHaveBeenCalledWith("pane-1", "tab-x", "Excel");

      const slot1El = document.querySelector<HTMLElement>('[data-testid="split-pane-slot-1"]');
      expect(slot1El?.classList.contains("pane-focused")).toBe(true);

      // Focus pane-3 (PDF)
      grid.setFocusedSlot("pane-3");
      expect(grid.getFocusedSlotId()).toBe("pane-3");
      expect(onFocusSlot).toHaveBeenCalledWith("pane-3", "tab-pdf", "PDF");

      grid.destroy();
    });

    it("closes a split pane and resets to single layout mode", () => {
      const onCloseSlot = vi.fn();
      const grid = new SplitPaneGrid(gridContainer, { onCloseSlot });

      grid.setLayoutMode("split-horizontal");
      expect(grid.getActiveSlots().length).toBe(2);

      const closeBtn = document.querySelector<HTMLButtonElement>(".pane-slot-close-btn");
      expect(closeBtn).not.toBeNull();
      closeBtn?.click();

      expect(grid.getLayoutMode()).toBe("single");
      expect(grid.getActiveSlots().length).toBe(1);
      expect(onCloseSlot).toHaveBeenCalled();

      grid.destroy();
    });
  });

  describe("WorkspaceLayoutManager Split Grid Integration", () => {
    it("mounts and switches split grid through WorkspaceLayoutManager", () => {
      const onFocusPaneSlot = vi.fn();
      const layoutMgr = new WorkspaceLayoutManager({ onFocusPaneSlot });

      const tabs: TabItem[] = [
        { id: "t-1", title: "Proposal.docx", docType: "Word", isDirty: false },
        { id: "t-2", title: "Fin.xlsx", docType: "Excel", isDirty: false },
      ];

      const splitGrid = layoutMgr.mountSplitGrid(gridContainer, tabs);
      expect(splitGrid).toBeDefined();

      layoutMgr.setSplitGridLayoutMode("split-horizontal", tabs);
      expect(layoutMgr.getSplitGridLayoutMode()).toBe("split-horizontal");
      expect(layoutMgr.isSplitView()).toBe(true);

      layoutMgr.setSplitGridLayoutMode("split-quad", tabs);
      expect(layoutMgr.getSplitGridLayoutMode()).toBe("split-quad");
      expect(splitGrid.getActiveSlots().length).toBe(4);

      layoutMgr.setSplitGridLayoutMode("single");
      expect(layoutMgr.getSplitGridLayoutMode()).toBe("single");
      expect(layoutMgr.isSplitView()).toBe(false);

      layoutMgr.destroy();
    });
  });

  describe("UnifiedShellApp Multi-Pane Grid Integration", () => {
    it("supports programmatic multi-pane layout mode switching on UnifiedShellApp", () => {
      const app = new UnifiedShellApp(false);

      expect(app.getSplitGridLayoutMode()).toBe("single");

      app.setSplitGridLayoutMode("split-horizontal");
      expect(app.getSplitGridLayoutMode()).toBe("split-horizontal");
      expect(app.isSplitView()).toBe(true);

      app.setSplitGridLayoutMode("split-quad");
      expect(app.getSplitGridLayoutMode()).toBe("split-quad");

      app.setSplitGridLayoutMode("single");
      expect(app.getSplitGridLayoutMode()).toBe("single");
      expect(app.isSplitView()).toBe(false);
    });
  });
});

