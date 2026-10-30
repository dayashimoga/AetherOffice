import { describe, it, expect, beforeEach, vi } from "vitest";
import { UnifiedShellApp } from "../main";
import { TabBar } from "../components/TabBar";
import { OpenDocumentsDrawer } from "../components/OpenDocumentsDrawer";
import { IPC } from "../ipc";

describe("Subphase 28.1.1: Multi-Window Per Document Type & Global Session Hub", () => {
  beforeEach(() => {
    localStorage.clear();
    IPC._resetMockState();

    HTMLCanvasElement.prototype.getContext = (() => ({
      clearRect: vi.fn(),
      fillRect: vi.fn(),
      fillText: vi.fn(),
      strokeRect: vi.fn(),
      measureText: () => ({ width: 10 }),
      beginPath: vi.fn(),
      moveTo: vi.fn(),
      lineTo: vi.fn(),
      stroke: vi.fn(),
      fill: vi.fn(),
      save: vi.fn(),
      restore: vi.fn(),
      scale: vi.fn(),
      setTransform: vi.fn(),
      setLineDash: vi.fn(),
    })) as unknown as typeof HTMLCanvasElement.prototype.getContext;

    document.body.innerHTML = `
      <div id="app" class="desktop-shell">
        <header class="title-bar">
          <div class="app-branding">
            <span class="app-logo">📄</span>
            <span class="app-name">AetherOffice</span>
          </div>
          <div class="title-actions">
            <button id="btn-theme-toggle" class="theme-toggle-btn">
              <span class="theme-icon">☀️</span>
              <span class="theme-label">Light</span>
            </button>
          </div>
          <div class="window-controls">
            <button id="btn-minimize" class="win-btn minimize">−</button>
            <button id="btn-maximize" class="win-btn maximize">□</button>
            <button id="btn-close" class="win-btn close">×</button>
          </div>
        </header>
        <div class="shell-body">
          <div id="app-rail-container"></div>
          <div class="shell-main-area">
            <div id="tab-bar-container"></div>
            <div id="ribbon-container"></div>
            <div id="sub-header-container"></div>
            <div id="viewport-container" class="shell-workspace-area"></div>
            <div id="sidebar-container" class="sidebar-pane"></div>
            <div id="copilot-sidebar-container"></div>
            <div id="bottom-sheet"></div>
            <canvas id="stylus-ink-canvas"></canvas>
            <div id="reflow-reader-container"></div>
            <div id="row-card-inspector-container"></div>
            <div id="laser-pointer-dot"></div>
          </div>
        </div>
        <footer class="status-bar">
          <div class="status-left">
            <span id="status-doc-info">Ready</span>
            <span id="status-word-count">0 words</span>
          </div>
          <div class="status-right">
            <input type="range" id="zoom-slider" min="50" max="200" value="100" />
            <span id="status-zoom-level">100%</span>
          </div>
        </footer>
      </div>
    `;
  });

  it("1. TabBar supports per-format filtering and Popout 'Open in New Window ↗' buttons", () => {
    const tabBar = new TabBar("tab-bar-container");
    const onPopout = vi.fn();
    tabBar.setCallbacks({ onTabPopout: onPopout });

    tabBar.addTab({ id: "t-w1", docType: "Word", title: "Doc1.docx", isDirty: false });
    tabBar.addTab({ id: "t-x1", docType: "Excel", title: "Book1.xlsx", isDirty: false });
    tabBar.addTab({ id: "t-p1", docType: "PowerPoint", title: "Pres1.pptx", isDirty: false });
    tabBar.addTab({ id: "t-pdf1", docType: "PDF", title: "Manual.pdf", isDirty: false });
    tabBar.addTab({ id: "t-n1", docType: "Notes", title: "Notes.txt", isDirty: false });

    // Without filter: all 5 tabs rendered
    let tabElements = document.querySelectorAll(".tab-item");
    expect(tabElements.length).toBe(5);

    // Each tab has an 'Open in New Window ↗' button
    const popoutButtons = document.querySelectorAll(".tab-popout-btn");
    expect(popoutButtons.length).toBe(5);

    // Clicking popout calls callback
    (popoutButtons[0] as HTMLButtonElement).click();
    expect(onPopout).toHaveBeenCalledWith(expect.objectContaining({ id: "t-w1", docType: "Word" }));

    // Apply filter for Excel only
    tabBar.setFilterDocType("Excel");
    tabElements = document.querySelectorAll(".tab-item");
    expect(tabElements.length).toBe(1);
    expect(tabElements[0].textContent).toContain("Book1.xlsx");

    // Switch filter to Word
    tabBar.setFilterDocType("Word");
    tabElements = document.querySelectorAll(".tab-item");
    expect(tabElements.length).toBe(1);
    expect(tabElements[0].textContent).toContain("Doc1.docx");

    // Clear filter
    tabBar.setFilterDocType(null);
    tabElements = document.querySelectorAll(".tab-item");
    expect(tabElements.length).toBe(5);
  });

  it("2. OpenDocumentsDrawer tracks all active documents and persists to localStorage", () => {
    const drawer = new OpenDocumentsDrawer("app");
    expect(drawer.getIsOpen()).toBe(false);

    // Register active documents across formats
    drawer.registerDocument({
      id: "doc-word-1",
      docType: "Word",
      title: "Quarterly Report.docx",
      isDirty: false,
      lastAutoSavedMs: Date.now(),
    }, "Document body snapshot");

    drawer.registerDocument({
      id: "doc-calc-1",
      docType: "Excel",
      title: "Financials.xlsx",
      isDirty: true,
      lastAutoSavedMs: Date.now(),
    });

    drawer.registerDocument({
      id: "doc-notes-1",
      docType: "Notes",
      title: "MeetingNotes.md",
      isDirty: false,
      lastAutoSavedMs: Date.now(),
    });

    expect(drawer.getEntries().length).toBe(3);

    // Verify localStorage persistence
    const saved = localStorage.getItem("aether_open_documents_registry");
    expect(saved).not.toBeNull();
    expect(saved).toContain("Quarterly Report.docx");
    expect(saved).toContain("Financials.xlsx");

    // Auto-save snapshot
    const snap = localStorage.getItem("aether_autosave_doc-word-1");
    expect(snap).toBe("Document body snapshot");

    // Open drawer
    drawer.open();
    expect(drawer.getIsOpen()).toBe(true);

    const cards = document.querySelectorAll(".drawer-doc-card");
    expect(cards.length).toBe(3);

    // Check badges
    expect(document.querySelector(".badge-word")).not.toBeNull();
    expect(document.querySelector(".badge-excel")).not.toBeNull();
    expect(document.querySelector(".badge-notes")).not.toBeNull();

    // Toggle dirty state
    drawer.setDirty("doc-word-1", true);
    expect(drawer.getEntries().find((d) => d.id === "doc-word-1")?.isDirty).toBe(true);

    // Unregister document
    drawer.unregisterDocument("doc-notes-1");
    expect(drawer.getEntries().length).toBe(2);
  });

  it("3. UnifiedShellApp integrates OpenDocumentsDrawer, All Open Files button, and IPC window spawning", async () => {
    const spawnSpy = vi.spyOn(IPC, "spawnAppWindow").mockResolvedValue("win-word-test");

    const app = new UnifiedShellApp(false);
    app.applyBootConfig({ app_mode: "writer" });

    // Verify All Open Files button exists in header
    const allFilesBtn = document.getElementById("btn-all-open-files");
    expect(allFilesBtn).not.toBeNull();

    // Drawer toggles on click
    const drawer = app.getOpenDocumentsDrawer();
    expect(drawer.getIsOpen()).toBe(false);
    allFilesBtn?.click();
    expect(drawer.getIsOpen()).toBe(true);
    allFilesBtn?.click();
    expect(drawer.getIsOpen()).toBe(false);

    // Creating new tabs registers them in the drawer
    app.handleNewTab("Excel");
    app.handleNewTab("PowerPoint");
    expect(drawer.getEntries().length).toBeGreaterThanOrEqual(3);

    // Popout tab invokes IPC.spawnAppWindow
    const activeTab = app.getTabBar().getActiveTab();
    expect(activeTab).toBeDefined();
    if (activeTab) {
      await app.handlePopoutTab(activeTab);
      expect(spawnSpy).toHaveBeenCalledWith("ppt", undefined);
    }
  });
});

