import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { TabBar, TabItem } from "../components/TabBar";
import { ProjectExplorerSidebar } from "../components/ProjectExplorerSidebar";
import { UnifiedShellApp } from "../main";

describe("Sub-Phase 42.2.1: Unified Mixed-Format Tab Bar & Project Explorer Sidebar", () => {
  beforeEach(() => {
    const mockCtx = {
      fillRect: () => { },
      strokeRect: () => { },
      clearRect: () => { },
      fillText: () => { },
      strokeText: () => { },
      beginPath: () => { },
      closePath: () => { },
      moveTo: () => { },
      lineTo: () => { },
      stroke: () => { },
      fill: () => { },
      save: () => { },
      restore: () => { },
      clip: () => { },
      rect: () => { },
      scale: () => { },
      setTransform: () => { },
      translate: () => { },
      rotate: () => { },
      arc: () => { },
      roundRect: () => { },
      ellipse: () => { },
      measureText: (txt: string) => ({ width: txt.length * 8 } as TextMetrics),
      setLineDash: () => { },
      font: "14px Segoe UI, sans-serif",
      fillStyle: "#000",
      strokeStyle: "#000",
      lineWidth: 1,
      textAlign: "left",
      textBaseline: "top",
    };
    HTMLCanvasElement.prototype.getContext = (() => mockCtx) as any;
    if (!HTMLCanvasElement.prototype.scrollIntoView) {
      HTMLCanvasElement.prototype.scrollIntoView = () => { };
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
          <div id="viewport-container"></div>
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
  });

  afterEach(() => {
    document.body.innerHTML = "";
    vi.restoreAllMocks();
  });

  describe("Unified Mixed-Format TabBar", () => {
    it("renders mixed-format tabs side-by-side with format badges", () => {
      const tabBar = new TabBar("tab-bar-container");
      tabBar.setFilterMode("all");

      const tabs: TabItem[] = [
        { id: "tab-1", title: "Proposal.docx", docType: "Word", isDirty: false },
        { id: "tab-2", title: "Financials.xlsx", docType: "Excel", isDirty: true },
        { id: "tab-3", title: "Roadmap.pptx", docType: "PowerPoint", isDirty: false },
        { id: "tab-4", title: "Contract.pdf", docType: "PDF", isDirty: false },
        { id: "tab-5", title: "Notes.md", docType: "Notes", isDirty: false },
      ];

      tabs.forEach((t) => tabBar.addTab(t));

      const tabEls = document.querySelectorAll<HTMLElement>('[data-testid="tab-item"]');
      expect(tabEls.length).toBe(5);

      const badges = Array.from(document.querySelectorAll<HTMLElement>(".tab-item .tab-type-icon")).map(
        (el) => el.textContent
      );
      expect(badges).toEqual(["W", "X", "P", "PDF", "N"]);

      expect(tabBar.getActiveDocType()).toBe("Notes"); // last added was active
      expect(tabBar.getVisibleTabs().length).toBe(5);
    });

    it("toggles between 'All Tabs' and 'Current App Tabs' filter modes", () => {
      const tabBar = new TabBar("tab-bar-container");

      tabBar.addTab({ id: "t1", title: "Word1.docx", docType: "Word", isDirty: false });
      tabBar.addTab({ id: "t2", title: "Word2.docx", docType: "Word", isDirty: false });
      tabBar.addTab({ id: "t3", title: "Sheet1.xlsx", docType: "Excel", isDirty: false });

      // In "all" mode, all 3 tabs are visible
      tabBar.setFilterMode("all");
      expect(tabBar.getVisibleTabs().length).toBe(3);
      expect(document.querySelectorAll('[data-testid="tab-item"]').length).toBe(3);

      // In "current" mode with filterDocType="Word", only the 2 Word tabs are visible
      tabBar.setFilterDocType("Word");
      tabBar.setFilterMode("current");
      expect(tabBar.getVisibleTabs().length).toBe(2);
      expect(document.querySelectorAll('[data-testid="tab-item"]').length).toBe(2);

      // Toggle filter mode button
      const toggleBtn = document.querySelector<HTMLButtonElement>('[data-testid="tab-filter-toggle"]');
      expect(toggleBtn).not.toBeNull();
      toggleBtn?.click();

      expect(tabBar.getFilterMode()).toBe("all");
      expect(tabBar.getVisibleTabs().length).toBe(3);
    });

    it("reorders tabs dynamically", () => {
      const tabBar = new TabBar("tab-bar-container");
      tabBar.setFilterMode("all");

      tabBar.addTab({ id: "t1", title: "Doc1", docType: "Word", isDirty: false });
      tabBar.addTab({ id: "t2", title: "Doc2", docType: "Excel", isDirty: false });
      tabBar.addTab({ id: "t3", title: "Doc3", docType: "PDF", isDirty: false });

      expect(tabBar.getTabs().map((t) => t.id)).toEqual(["t1", "t2", "t3"]);

      tabBar.reorderTabs(0, 2);
      expect(tabBar.getTabs().map((t) => t.id)).toEqual(["t2", "t3", "t1"]);
    });
  });

  describe("ProjectExplorerSidebar", () => {
    it("renders default workspace folder tree with format badges", () => {
      const explorer = new ProjectExplorerSidebar("sidebar-container");

      const folderRows = document.querySelectorAll('[data-testid="project-folder-item"]');
      expect(folderRows.length).toBeGreaterThanOrEqual(3); // Root, Executive, Financial, Legal

      const fileRows = document.querySelectorAll('[data-testid="project-file-item"]');
      expect(fileRows.length).toBeGreaterThanOrEqual(6); // Q4_Proposal, Pitch_Deck, Budget_Model, Sales_Forecast, Enterprise_Contract, Meeting_Notes

      const wordFile = document.querySelector('[data-file-path="/workspace/Executive Briefings/Q4_Proposal.docx"]');
      expect(wordFile).not.toBeNull();
      expect(wordFile?.querySelector(".project-badge-word")?.textContent).toBe("W");

      const excelFile = document.querySelector('[data-file-path="/workspace/Financial Models/Budget_Model_2026.xlsx"]');
      expect(excelFile).not.toBeNull();
      expect(excelFile?.querySelector(".project-badge-excel")?.textContent).toBe("X");

      explorer.destroy();
    });

    it("filters files matching search queries and auto-expands parent folders", () => {
      const explorer = new ProjectExplorerSidebar("sidebar-container");

      const filterInput = document.querySelector<HTMLInputElement>('[data-testid="project-explorer-filter-input"]');
      expect(filterInput).not.toBeNull();

      explorer.setSearchQuery("budget");

      const fileRows = document.querySelectorAll('[data-testid="project-file-item"]');
      expect(fileRows.length).toBe(1);
      expect(fileRows[0].textContent).toContain("Budget_Model_2026.xlsx");

      explorer.setSearchQuery("");
      expect(document.querySelectorAll('[data-testid="project-file-item"]').length).toBeGreaterThanOrEqual(6);

      explorer.destroy();
    });

    it("triggers onOpenFile callback when a project file is clicked", () => {
      const onOpenFile = vi.fn();
      const explorer = new ProjectExplorerSidebar("sidebar-container", { onOpenFile });

      const fileEl = document.querySelector<HTMLElement>('[data-file-path="/workspace/Legal & Compliance/Enterprise_Contract.pdf"]');
      expect(fileEl).not.toBeNull();
      fileEl?.click();

      expect(onOpenFile).toHaveBeenCalledTimes(1);
      expect(onOpenFile).toHaveBeenCalledWith(
        expect.objectContaining({
          name: "Enterprise_Contract.pdf",
          docType: "PDF",
          path: "/workspace/Legal & Compliance/Enterprise_Contract.pdf",
        })
      );
      expect(explorer.getActiveFilePath()).toBe("/workspace/Legal & Compliance/Enterprise_Contract.pdf");

      explorer.destroy();
    });

    it("supports expanding and collapsing all folders", () => {
      const explorer = new ProjectExplorerSidebar("sidebar-container");

      explorer.collapseAll();
      const folder = explorer.getRootFolder();
      expect(folder.isExpanded).toBe(false);

      explorer.expandAll();
      expect(folder.isExpanded).toBe(true);

      explorer.destroy();
    });
  });

  describe("UnifiedShellApp Integration with Project Explorer", () => {
    it("renders Project Explorer toggle button in title bar and mounts sidebar on click", () => {
      const app = new UnifiedShellApp(false);

      const explorerBtn = document.getElementById("btn-project-explorer");
      expect(explorerBtn).not.toBeNull();

      expect(app.isProjectExplorerActive()).toBe(false);

      // Toggle Project Explorer open
      app.toggleProjectExplorer();
      expect(app.isProjectExplorerActive()).toBe(true);
      expect(app.getProjectExplorer()).toBeDefined();

      const explorerTree = document.getElementById("project-explorer-tree");
      expect(explorerTree).not.toBeNull();

      // Toggle Project Explorer closed
      app.toggleProjectExplorer();
      expect(app.isProjectExplorerActive()).toBe(false);
    });

    it("syncs active document file path with Project Explorer", async () => {
      const app = new UnifiedShellApp(false);
      app.toggleProjectExplorer();

      await app.handleOpenFile("/workspace/Financial Models/Budget_Model_2026.xlsx");

      expect(app.getProjectExplorer()?.getActiveFilePath()).toBe(
        "/workspace/Financial Models/Budget_Model_2026.xlsx"
      );
    });
  });
});