import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import {
  WorkspaceLayoutManager,
  PaneController,
  FocusRouter,
} from "../controllers";
import { TabItem } from "../components/TabBar";
import { UnifiedShellApp } from "../main";

describe("Sub-Phase 42.1.2: Modular Workspace Controllers", () => {
  let layoutContainer: HTMLElement;
  let viewportContainer: HTMLElement;

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
        <button id="btn-split-view"></button>
      </div>
    `;
    layoutContainer = document.querySelector(".workspace-layout") as HTMLElement;
    viewportContainer = document.getElementById("viewport-container") as HTMLElement;
  });

  afterEach(() => {
    document.body.innerHTML = "";
    vi.restoreAllMocks();
  });

  describe("WorkspaceLayoutManager", () => {
    it("manages split-view toggling and layout classes cleanly", () => {
      const onToggleSplit = vi.fn();
      const layoutMgr = new WorkspaceLayoutManager({ onToggleSplit });

      expect(layoutMgr.isSplitView()).toBe(false);

      const tabs: TabItem[] = [
        { id: "tab-1", title: "Doc1.docx", docType: "Word", isDirty: false },
        { id: "tab-2", title: "Sheet1.xlsx", docType: "Excel", isDirty: false },
      ];

      // Toggle split on
      const active1 = layoutMgr.toggleSplitView(tabs, tabs[0]);
      expect(active1).toBe(true);
      expect(layoutMgr.isSplitView()).toBe(true);
      expect(onToggleSplit).toHaveBeenCalledWith(true);
      expect(layoutContainer.classList.contains("split-view-active")).toBe(true);
      expect(document.getElementById("workspace-split-pane")).not.toBeNull();

      // Toggle split off
      const active2 = layoutMgr.toggleSplitView(tabs, tabs[0]);
      expect(active2).toBe(false);
      expect(layoutMgr.isSplitView()).toBe(false);
      expect(onToggleSplit).toHaveBeenCalledWith(false);
      expect(layoutContainer.classList.contains("split-view-active")).toBe(false);
      expect(document.getElementById("workspace-split-pane")).toBeNull();

      layoutMgr.destroy();
    });

    it("renders split pane selector and updates preview content on selection", () => {
      const onSelectSecondaryTab = vi.fn();
      const layoutMgr = new WorkspaceLayoutManager({ onSelectSecondaryTab });

      const tabs: TabItem[] = [
        { id: "tab-1", title: "Word1.docx", docType: "Word", isDirty: false },
        { id: "tab-2", title: "Budget.xlsx", docType: "Excel", isDirty: false },
        { id: "tab-3", title: "Deck.pptx", docType: "PowerPoint", isDirty: false },
      ];

      layoutMgr.toggleSplitView(tabs, tabs[0]);

      const select = document.querySelector(".split-pane-select") as HTMLSelectElement;
      expect(select).not.toBeNull();
      expect(select.children.length).toBe(2); // tab-2 and tab-3

      // Change selected tab to Deck.pptx
      select.value = "tab-3";
      select.dispatchEvent(new Event("change"));

      expect(layoutMgr.getSplitRightTabId()).toBe("tab-3");
      expect(onSelectSecondaryTab).toHaveBeenCalledWith("tab-3");

      const preview = document.querySelector(".split-doc-preview");
      expect(preview).not.toBeNull();
      expect(preview?.textContent).toContain("Deck.pptx");

      layoutMgr.destroy();
    });

    it("handles orientation switching (horizontal vs vertical)", () => {
      const layoutMgr = new WorkspaceLayoutManager();
      const tabs: TabItem[] = [
        { id: "tab-1", title: "Word1.docx", docType: "Word", isDirty: false },
        { id: "tab-2", title: "Budget.xlsx", docType: "Excel", isDirty: false },
      ];

      layoutMgr.toggleSplitView(tabs, tabs[0]);
      const pane = document.getElementById("workspace-split-pane");
      expect(pane).not.toBeNull();

      layoutMgr.setSplitOrientation("vertical");
      expect(layoutMgr.getSplitOrientation()).toBe("vertical");
      expect(pane?.classList.contains("split-vertical")).toBe(true);

      layoutMgr.setSplitOrientation("horizontal");
      expect(layoutMgr.getSplitOrientation()).toBe("horizontal");
      expect(pane?.classList.contains("split-vertical")).toBe(false);

      layoutMgr.destroy();
    });

    it("controls subheader and sidebar visibility states", () => {
      const layoutMgr = new WorkspaceLayoutManager();

      layoutMgr.setSubHeaderVisibility(false);
      expect(document.getElementById("sub-header-container")?.style.display).toBe("none");

      layoutMgr.setSubHeaderVisibility(true);
      expect(document.getElementById("sub-header-container")?.style.display).toBe("flex");

      layoutMgr.setSidebarVisibility(false);
      expect(document.getElementById("sidebar-container")?.style.display).toBe("none");

      layoutMgr.setSidebarVisibility(true);
      expect(document.getElementById("sidebar-container")?.style.display).toBe("flex");

      layoutMgr.setShowNavPane(false);
      expect(layoutMgr.getShowNavPane()).toBe(false);

      layoutMgr.setAppFilter("Excel");
      expect(layoutMgr.getAppFilter()).toBe("Excel");

      layoutMgr.setHubMode(true);
      expect(document.getElementById("app")?.classList.contains("shell-hub-mode")).toBe(true);
      layoutMgr.setHubMode(false);
      expect(document.getElementById("app")?.classList.contains("shell-hub-mode")).toBe(false);

      layoutMgr.destroy();
    });
  });

  describe("PaneController", () => {
    it("mounts Word viewport and binds status and modification callbacks", () => {
      const paneCtrl = new PaneController("test-pane");
      const onStatusChange = vi.fn();
      const onDirty = vi.fn();

      const tab: TabItem = {
        id: "tab-word-1",
        title: "Test.docx",
        docType: "Word",
        isDirty: false,
      };

      paneCtrl.mountViewport(viewportContainer, tab, { onStatusChange, onDirty });

      expect(paneCtrl.getCurrentDocType()).toBe("Word");
      expect(paneCtrl.getActiveTabId()).toBe("tab-word-1");
      expect(paneCtrl.getWriterCanvas()).toBeDefined();

      paneCtrl.destroy();
      expect(paneCtrl.getWriterCanvas()).toBeUndefined();
    });

    it("mounts Excel viewport and instantiates CalcGridCanvas and SheetTabBar", () => {
      const paneCtrl = new PaneController("test-pane");
      const tab: TabItem = {
        id: "tab-excel-1",
        title: "Model.xlsx",
        docType: "Excel",
        isDirty: false,
      };

      paneCtrl.mountViewport(viewportContainer, tab);

      expect(paneCtrl.getCurrentDocType()).toBe("Excel");
      expect(paneCtrl.getCalcGridCanvas()).toBeDefined();
      expect(paneCtrl.getSheetTabBar()).toBeDefined();

      paneCtrl.destroy();
      expect(paneCtrl.getCalcGridCanvas()).toBeUndefined();
    });

    it("mounts PowerPoint, PDF, and Notes viewports seamlessly", () => {
      const paneCtrl = new PaneController("test-pane");

      // PowerPoint
      const pptTab: TabItem = { id: "tab-ppt-1", title: "Pres.pptx", docType: "PowerPoint", isDirty: false };
      paneCtrl.mountViewport(viewportContainer, pptTab);
      expect(paneCtrl.getSlideStageCanvas()).toBeDefined();
      expect(paneCtrl.getSpeakerNotesPane()).toBeDefined();
      expect(paneCtrl.getAnimationPane()).toBeDefined();
      expect(paneCtrl.getOrCreatePresentationPlayer()).toBeDefined();

      // PDF
      const pdfTab: TabItem = { id: "tab-pdf-1", title: "Doc.pdf", docType: "PDF", isDirty: false };
      paneCtrl.mountViewport(viewportContainer, pdfTab);
      expect(paneCtrl.getPdfWorkspace()).toBeDefined();
      expect(paneCtrl.getPdfViewerCanvas()).toBeDefined();
      expect(paneCtrl.getSlideStageCanvas()).toBeUndefined(); // teardown previous

      // Notes
      const notesTab: TabItem = { id: "tab-notes-1", title: "Readme.txt", docType: "Notes", isDirty: false };
      paneCtrl.mountViewport(viewportContainer, notesTab);
      expect(paneCtrl.getNotesWorkspace()).toBeDefined();
      expect(paneCtrl.getPdfWorkspace()).toBeUndefined(); // teardown previous

      paneCtrl.destroy();
    });

    it("sets zoom across document types without errors", () => {
      const paneCtrl = new PaneController("test-pane");

      const wordTab: TabItem = { id: "tab-w", title: "Doc.docx", docType: "Word", isDirty: false };
      paneCtrl.mountViewport(viewportContainer, wordTab);
      expect(() => paneCtrl.setZoom(150)).not.toThrow();

      const excelTab: TabItem = { id: "tab-x", title: "Grid.xlsx", docType: "Excel", isDirty: false };
      paneCtrl.mountViewport(viewportContainer, excelTab);
      expect(() => paneCtrl.setZoom(75)).not.toThrow();

      const pptTab: TabItem = { id: "tab-p", title: "Deck.pptx", docType: "PowerPoint", isDirty: false };
      paneCtrl.mountViewport(viewportContainer, pptTab);
      expect(() => paneCtrl.setZoom(120)).not.toThrow();

      paneCtrl.destroy();
    });
  });

  describe("FocusRouter", () => {
    it("tracks focused pane, session, and docType with event notifications", () => {
      const router = new FocusRouter("Word");
      const focusListener = vi.fn();
      const unsub = router.onFocusChange(focusListener);

      expect(router.getFocusedPaneId()).toBe("primary-pane");
      expect(router.getFocusedDocType()).toBe("Word");

      router.setFocus("secondary-pane", "sess-excel-9", "Excel");
      expect(router.getFocusedPaneId()).toBe("secondary-pane");
      expect(router.getFocusedSessionId()).toBe("sess-excel-9");
      expect(router.getFocusedDocType()).toBe("Excel");
      expect(focusListener).toHaveBeenCalledWith("secondary-pane", "sess-excel-9", "Excel");

      unsub();
      router.setFocus("primary-pane", "sess-word-1", "Word");
      expect(focusListener).toHaveBeenCalledTimes(1); // unsubscribed

      router.destroy();
    });

    it("routes keyboard shortcuts to appropriate delegate methods", () => {
      const router = new FocusRouter("Word");
      const delegate = {
        getActiveTabId: vi.fn(() => "tab-1"),
        getCurrentDocType: vi.fn(() => "Word" as const),
        onNextTab: vi.fn(),
        onPrevTab: vi.fn(),
        onSave: vi.fn(),
        onSaveAs: vi.fn(),
        onOpen: vi.fn(),
        onPrint: vi.fn(),
        onNew: vi.fn(),
        onClose: vi.fn(),
        onFind: vi.fn(),
        onReplace: vi.fn(),
        onRenderCurrent: vi.fn(),
      };

      router.handleShortcutAction("NextTab", delegate);
      expect(delegate.onNextTab).toHaveBeenCalledTimes(1);

      router.handleShortcutAction("PrevTab", delegate);
      expect(delegate.onPrevTab).toHaveBeenCalledTimes(1);

      router.handleShortcutAction("Save", delegate);
      expect(delegate.onSave).toHaveBeenCalledTimes(1);

      router.handleShortcutAction("SaveAs", delegate);
      expect(delegate.onSaveAs).toHaveBeenCalledTimes(1);

      router.handleShortcutAction("Open", delegate);
      expect(delegate.onOpen).toHaveBeenCalledTimes(1);

      router.handleShortcutAction("Print", delegate);
      expect(delegate.onPrint).toHaveBeenCalledTimes(1);

      router.handleShortcutAction("New", delegate);
      expect(delegate.onNew).toHaveBeenCalledTimes(1);

      router.handleShortcutAction("Close", delegate);
      expect(delegate.onClose).toHaveBeenCalledWith("tab-1");

      router.handleShortcutAction("Find", delegate);
      expect(delegate.onFind).toHaveBeenCalledTimes(1);

      router.handleShortcutAction("Replace", delegate);
      expect(delegate.onReplace).toHaveBeenCalledTimes(1);

      router.handleShortcutAction("Undo", delegate);
      expect(delegate.onRenderCurrent).toHaveBeenCalled();

      router.handleShortcutAction("Redo", delegate);
      expect(delegate.onRenderCurrent).toHaveBeenCalled();

      router.destroy();
    });

    it("resolves Alt keytips accurately based on document type", () => {
      const router = new FocusRouter();

      // Backstage
      expect(router.resolveAltKeytip("F", "Word")).toEqual({ type: "backstage" });

      // Common Home / Insert
      expect(router.resolveAltKeytip("H", "Word")).toEqual({ type: "tab", tabId: "Home" });
      expect(router.resolveAltKeytip("N", "Excel")).toEqual({ type: "tab", tabId: "Insert" });

      // Format-specific "P"
      expect(router.resolveAltKeytip("P", "Word")).toEqual({ type: "tab", tabId: "Layout" });
      expect(router.resolveAltKeytip("P", "Excel")).toEqual({ type: "tab", tabId: "Page Layout" });
      expect(router.resolveAltKeytip("P", "PDF")).toEqual({ type: "tab", tabId: "Page Tools" });

      // Format-specific "S"
      expect(router.resolveAltKeytip("S", "PowerPoint")).toEqual({ type: "tab", tabId: "Slide Show" });
      expect(router.resolveAltKeytip("S", "Word")).toEqual({ type: "tab", tabId: "References" });
      expect(router.resolveAltKeytip("S", "PDF")).toEqual({ type: "tab", tabId: "Sign & Protect" });

      // Formulas & Data
      expect(router.resolveAltKeytip("M", "Excel")).toEqual({ type: "tab", tabId: "Formulas" });
      expect(router.resolveAltKeytip("A", "Excel")).toEqual({ type: "tab", tabId: "Data" });

      // Invalid keytip
      expect(router.resolveAltKeytip("Z", "Word")).toBeNull();

      router.destroy();
    });

    it("maps Copilot and Collab presence modes accurately", () => {
      const router = new FocusRouter();

      expect(router.getCopilotMode("Word")).toBe("writer");
      expect(router.getCopilotMode("Excel")).toBe("calc");
      expect(router.getCopilotMode("PowerPoint")).toBe("slides");
      expect(router.getCopilotMode("PDF")).toBe("writer");
      expect(router.getCopilotMode("Notes")).toBe("writer");

      expect(router.getCollabMode("Excel")).toBe("calc");
      expect(router.getCollabMode("PowerPoint")).toBe("slides");

      router.destroy();
    });
  });

  describe("UnifiedShellApp Integration with Modular Controllers", () => {
    it("initializes and delegates workspace operations cleanly to controllers", () => {
      const app = new UnifiedShellApp(false);

      expect(app.getWorkspaceLayout()).toBeInstanceOf(WorkspaceLayoutManager);
      expect(app.getPaneController()).toBeInstanceOf(PaneController);
      expect(app.getFocusRouter()).toBeInstanceOf(FocusRouter);

      // Verify split view delegation
      expect(app.isSplitView()).toBe(false);
      expect(app.isSplitViewOpen()).toBe(false);

      app.toggleSplitView();
      expect(app.isSplitView()).toBe(true);
      expect(app.getWorkspaceLayout().isSplitView()).toBe(true);

      app.toggleSplitView();
      expect(app.isSplitView()).toBe(false);

      // Verify canvas delegation
      expect(app.getWriterCanvas()).toBe(app.getPaneController().getWriterCanvas());
      expect(app.getCalcGridCanvas()).toBe(app.getPaneController().getCalcGridCanvas());
      expect(app.getSlideStageCanvas()).toBe(app.getPaneController().getSlideStageCanvas());
      expect(app.getPdfWorkspace()).toBe(app.getPaneController().getPdfWorkspace());
      expect(app.getNotesWorkspace()).toBe(app.getPaneController().getNotesWorkspace());
    });
  });
});
