/**
 * workspace_persistence.test.ts
 * GEC-v7.0 Stage 42 (Sub-Phase 42.2.3)
 *
 * Automated tests for Smart Workspace Persistence (.aether-workspace):
 * - Lossless schema serialization and deserialization
 * - Resilient error handling on corrupted / non-conforming inputs (zero crashes)
 * - Mixed-format tab group rehydration (.docx, .xlsx, .pptx, .pdf, .md)
 * - Dockable split-pane grid layout restoration (single, split-horizontal, split-vertical, split-quad)
 * - Active document, focused pane slot, and Project Explorer sidebar persistence
 * - Universal file open routing for .aether-workspace files
 * - Global shortcut accelerators for workspace persistence
 */

import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import {
  WorkspacePersistenceService,
  workspacePersistenceService,
  AetherWorkspaceFile,
} from "../services/WorkspacePersistenceService";
import { UnifiedShellApp } from "../main";
import { documentSessionRegistry } from "../state/documentSessionRegistry";

describe("Sub-Phase 42.2.3: Smart Workspace Persistence (.aether-workspace)", () => {
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
    HTMLCanvasElement.prototype.getContext = (() => mockCtx) as unknown as typeof HTMLCanvasElement.prototype.getContext;
    if (!HTMLCanvasElement.prototype.scrollIntoView) {
      HTMLCanvasElement.prototype.scrollIntoView = () => {};
    }

    document.body.innerHTML = `
      <div id="app">
        <header class="title-bar">
          <div class="title-actions">
            <button id="btn-project-explorer">Project Explorer</button>
          </div>
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

    documentSessionRegistry.reset();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("1. Schema Serialization & Defaults", () => {
    it("serializes valid workspace with correct format, version, and timestamps", () => {
      const service = new WorkspacePersistenceService();
      const ws = service.serialize({
        title: "Q4 Financial Review",
        tabs: [
          { id: "tab-1", docType: "Word", title: "Summary.docx", isDirty: false },
          { id: "tab-2", docType: "Excel", title: "Budget.xlsx", isDirty: true },
        ],
        activeTabId: "tab-2",
        activeDocType: "Excel",
        layoutMode: "split-horizontal",
      });

      expect(ws.format).toBe("aether-workspace");
      expect(ws.version).toBe(1);
      expect(ws.title).toBe("Q4 Financial Review");
      expect(ws.activeTabId).toBe("tab-2");
      expect(ws.activeDocType).toBe("Excel");
      expect(ws.tabs).toHaveLength(2);
      expect(ws.tabs[0].title).toBe("Summary.docx");
      expect(ws.tabs[1].isDirty).toBe(true);
      expect(ws.layout.mode).toBe("split-horizontal");
      expect(ws.layout.slots).toHaveLength(2);
      expect(ws.createdAt).toBeDefined();
      expect(ws.updatedAt).toBeDefined();
    });

    it("falls back to sensible defaults when given empty or minimal context", () => {
      const service = new WorkspacePersistenceService();
      const ws = service.serialize({
        tabs: [],
      });

      expect(ws.format).toBe("aether-workspace");
      expect(ws.title).toBe("Default Workspace");
      expect(ws.tabs.length).toBeGreaterThanOrEqual(1);
      expect(ws.tabs[0].docType).toBe("Word");
      expect(ws.layout.mode).toBe("single");
      expect(ws.layout.slots).toHaveLength(1);
      expect(ws.layout.focusedSlotId).toBe("pane-0");
    });
  });

  describe("2. Schema Deserialization & Robust Error Handling", () => {
    it("parses valid JSON string into typed AetherWorkspaceFile", () => {
      const service = new WorkspacePersistenceService();
      const validJson = JSON.stringify({
        format: "aether-workspace",
        version: 1,
        id: "ws-test-1",
        title: "Team Project",
        createdAt: "2026-11-01T04:00:00.000Z",
        updatedAt: "2026-11-01T04:00:00.000Z",
        activeTabId: "tab-notes",
        activeDocType: "Notes",
        tabs: [
          { id: "tab-notes", docType: "Notes", title: "ReadMe.md", isDirty: false },
          { id: "tab-pdf", docType: "PDF", title: "Specs.pdf", isDirty: false },
        ],
        layout: {
          mode: "split-vertical",
          focusedSlotId: "pane-1",
          slots: [
            { slotId: "pane-0", tabId: "tab-notes", docType: "Notes", title: "ReadMe.md" },
            { slotId: "pane-1", tabId: "tab-pdf", docType: "PDF", title: "Specs.pdf" },
          ],
        },
      });

      const res = service.deserialize(validJson);
      expect(res.success).toBe(true);
      expect(res.data?.id).toBe("ws-test-1");
      expect(res.data?.title).toBe("Team Project");
      expect(res.data?.tabs).toHaveLength(2);
      expect(res.data?.layout.mode).toBe("split-vertical");
      expect(res.data?.layout.focusedSlotId).toBe("pane-1");
    });

    it("gracefully rejects corrupt / invalid JSON without throwing errors", () => {
      const service = new WorkspacePersistenceService();
      const corruptJson = "{ format: 'aether-workspace', bad json...";

      const res = service.deserialize(corruptJson);
      expect(res.success).toBe(false);
      expect(res.error).toContain("Invalid JSON syntax");
    });

    it("rejects non-aether-workspace JSON files missing format marker", () => {
      const service = new WorkspacePersistenceService();
      const foreignJson = JSON.stringify({ name: "other-app-config", version: 2 });

      const res = service.deserialize(foreignJson);
      expect(res.success).toBe(false);
      expect(res.error).toContain("missing 'aether-workspace' format identifier");
    });

    it("normalizes malformed tab items and ensures at least one default tab", () => {
      const service = new WorkspacePersistenceService();
      const badTabsJson = JSON.stringify({
        format: "aether-workspace",
        version: 1,
        tabs: [null, "invalid-tab-string", { invalid: true }],
        layout: { mode: "unknown-layout-mode" },
      });

      const res = service.deserialize(badTabsJson);
      expect(res.success).toBe(true);
      expect(res.data?.tabs.length).toBeGreaterThanOrEqual(1);
      expect(res.data?.layout.mode).toBe("single");
    });
  });

  describe("3. Lossless Round-Trip Fidelity", () => {
    it("preserves all workspace properties through exportToJson -> deserialize round-trip", () => {
      const service = workspacePersistenceService;
      const original = service.serialize({
        id: "ws-lossless-007",
        title: "Lossless Round-Trip Suite",
        tabs: [
          { id: "tab-w", docType: "Word", title: "Brief.docx", isDirty: false, file_path: "C:/docs/Brief.docx" },
          { id: "tab-x", docType: "Excel", title: "Model.xlsx", isDirty: true, file_path: "C:/docs/Model.xlsx" },
          { id: "tab-p", docType: "PowerPoint", title: "Pitch.pptx", isDirty: false, file_path: "C:/docs/Pitch.pptx" },
        ],
        activeTabId: "tab-x",
        activeDocType: "Excel",
        layoutMode: "split-quad",
        focusedSlotId: "pane-2",
        isExplorerOpen: true,
        explorerActiveFilePath: "C:/docs/Model.xlsx",
        metadata: { client: "Acme Corp", version: "v2.5" },
      });

      const json = service.exportToJson(original);
      const res = service.deserialize(json);

      expect(res.success).toBe(true);
      const restored = res.data!;
      expect(restored.id).toBe(original.id);
      expect(restored.title).toBe(original.title);
      expect(restored.activeTabId).toBe("tab-x");
      expect(restored.activeDocType).toBe("Excel");
      expect(restored.tabs).toHaveLength(3);
      expect(restored.tabs[0].filePath).toBe("C:/docs/Brief.docx");
      expect(restored.tabs[1].isDirty).toBe(true);
      expect(restored.layout.mode).toBe("split-quad");
      expect(restored.layout.focusedSlotId).toBe("pane-2");
      expect(restored.explorer?.isOpen).toBe(true);
      expect(restored.explorer?.activeFilePath).toBe("C:/docs/Model.xlsx");
      expect(restored.metadata?.client).toBe("Acme Corp");
    });
  });

  describe("4. Mixed-Format Tab Rehydration via UnifiedShellApp", () => {
    it("rehydrates mixed-format tabs across Word, Excel, PowerPoint, PDF, and Notes", async () => {
      const app = new UnifiedShellApp({ autoBoot: false });
      const workspaceData: AetherWorkspaceFile = {
        format: "aether-workspace",
        version: 1,
        id: "ws-mixed-5",
        title: "All-in-One Office Workspace",
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        activeTabId: "tab-excel",
        activeDocType: "Excel",
        tabs: [
          { id: "tab-word", docType: "Word", title: "Report.docx", isDirty: false, filePath: "Report.docx" },
          { id: "tab-excel", docType: "Excel", title: "Data.xlsx", isDirty: true, filePath: "Data.xlsx" },
          { id: "tab-slides", docType: "PowerPoint", title: "Slides.pptx", isDirty: false, filePath: "Slides.pptx" },
          { id: "tab-pdf", docType: "PDF", title: "Manual.pdf", isDirty: false, filePath: "Manual.pdf" },
          { id: "tab-notes", docType: "Notes", title: "Notes.md", isDirty: false, filePath: "Notes.md" },
        ],
        layout: {
          mode: "single",
          focusedSlotId: "pane-0",
          slots: [{ slotId: "pane-0", tabId: "tab-excel", docType: "Excel", title: "Data.xlsx" }],
        },
      };

      const success = await app.loadWorkspace(workspaceData);
      expect(success).toBe(true);

      const tabs = app.getTabBar().getTabs();
      expect(tabs).toHaveLength(5);
      expect(tabs.map((t) => t.docType)).toEqual(["Word", "Excel", "PowerPoint", "PDF", "Notes"]);
      expect(tabs.map((t) => t.title)).toEqual(["Report.docx", "Data.xlsx", "Slides.pptx", "Manual.pdf", "Notes.md"]);

      // Active tab check
      const active = app.getTabBar().getActiveTab();
      expect(active?.id).toBe("tab-excel");
      expect(active?.title).toBe("Data.xlsx");

      // Verify sessions registered in documentSessionRegistry
      expect(documentSessionRegistry.listSessions()).toHaveLength(5);
      expect(documentSessionRegistry.getSession("tab-word")?.docType).toBe("Word");
      expect(documentSessionRegistry.getSession("tab-excel")?.docType).toBe("Excel");
      expect(documentSessionRegistry.getSession("tab-slides")?.docType).toBe("PowerPoint");
      expect(documentSessionRegistry.getSession("tab-pdf")?.docType).toBe("PDF");
      expect(documentSessionRegistry.getSession("tab-notes")?.docType).toBe("Notes");
    });
  });

  describe("5. Multi-Pane Split Layout Rehydration", () => {
    it("reconstructs dockable quad 2x2 split grid and assigns tabs to each quadrant", async () => {
      const app = new UnifiedShellApp({ autoBoot: false });
      const viewportEl = document.getElementById("viewport-container")!;
      app.getWorkspaceLayout().mountSplitGrid(viewportEl, []);

      const quadWorkspace: AetherWorkspaceFile = {
        format: "aether-workspace",
        version: 1,
        id: "ws-quad",
        title: "Quad Pane Analysis",
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        activeTabId: "tab-slot-2",
        activeDocType: "PowerPoint",
        tabs: [
          { id: "tab-slot-0", docType: "Word", title: "Requirements.docx" },
          { id: "tab-slot-1", docType: "Excel", title: "Calculations.xlsx" },
          { id: "tab-slot-2", docType: "PowerPoint", title: "Presentation.pptx" },
          { id: "tab-slot-3", docType: "PDF", title: "Standard.pdf" },
        ],
        layout: {
          mode: "split-quad",
          focusedSlotId: "pane-2",
          slots: [
            { slotId: "pane-0", tabId: "tab-slot-0", docType: "Word", title: "Requirements.docx" },
            { slotId: "pane-1", tabId: "tab-slot-1", docType: "Excel", title: "Calculations.xlsx" },
            { slotId: "pane-2", tabId: "tab-slot-2", docType: "PowerPoint", title: "Presentation.pptx" },
            { slotId: "pane-3", tabId: "tab-slot-3", docType: "PDF", title: "Standard.pdf" },
          ],
        },
      };

      const success = await app.loadWorkspace(quadWorkspace);
      expect(success).toBe(true);

      expect(app.getSplitGridLayoutMode()).toBe("split-quad");
      const grid = app.getSplitGrid();
      expect(grid).toBeDefined();
      expect(grid?.getLayoutMode()).toBe("split-quad");
      expect(grid?.getFocusedSlotId()).toBe("pane-2");

      const activeSlots = grid?.getActiveSlots();
      expect(activeSlots).toHaveLength(4);
      expect(activeSlots?.[0].tabId).toBe("tab-slot-0");
      expect(activeSlots?.[1].tabId).toBe("tab-slot-1");
      expect(activeSlots?.[2].tabId).toBe("tab-slot-2");
      expect(activeSlots?.[3].tabId).toBe("tab-slot-3");
    });
  });

  describe("6. Workspace Serialization & Disk Save via UnifiedShellApp", () => {
    it("serializes live state accurately and reports save status feedback", async () => {
      const app = new UnifiedShellApp({ autoBoot: false });
      const tabBar = app.getTabBar();
      tabBar.addTab({ id: "tab-live-1", docType: "Word", title: "Report.docx", isDirty: true, file_path: "Report.docx" });
      tabBar.addTab({ id: "tab-live-2", docType: "Excel", title: "Fin.xlsx", isDirty: false, file_path: "Fin.xlsx" });
      tabBar.switchTab("tab-live-1");

      const snapshot = app.serializeCurrentWorkspace("Active Corporate Review");
      expect(snapshot.format).toBe("aether-workspace");
      expect(snapshot.title).toBe("Active Corporate Review");
      expect(snapshot.tabs.length).toBeGreaterThanOrEqual(2);

      const saved = await app.saveWorkspace("workspaces/Corporate.aether-workspace");
      expect(saved).not.toBeNull();
      expect(saved?.title).toBe("Active Corporate Review");

      const statusDoc = document.getElementById("status-doc-info");
      expect(statusDoc?.textContent).toContain("Workspace Saved: Active Corporate Review");
    });
  });

  describe("7. Universal File Open Routing for .aether-workspace", () => {
    it("automatically rehydrates workspace when passed to handleOpenFile", async () => {
      const app = new UnifiedShellApp({ autoBoot: false });
      const wsFile: AetherWorkspaceFile = {
        format: "aether-workspace",
        version: 1,
        id: "ws-via-file-open",
        title: "Discovered Workspace",
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        activeTabId: "tab-opened-word",
        activeDocType: "Word",
        tabs: [
          { id: "tab-opened-word", docType: "Word", title: "Discovered.docx" },
          { id: "tab-opened-notes", docType: "Notes", title: "Scratchpad.md" },
        ],
        layout: {
          mode: "split-horizontal",
          focusedSlotId: "pane-0",
          slots: [
            { slotId: "pane-0", tabId: "tab-opened-word", docType: "Word", title: "Discovered.docx" },
            { slotId: "pane-1", tabId: "tab-opened-notes", docType: "Notes", title: "Scratchpad.md" },
          ],
        },
      };

      const jsonStr = JSON.stringify(wsFile);
      await app.handleOpenFile("my_project.aether-workspace", jsonStr);

      const tabs = app.getTabBar().getTabs();
      expect(tabs).toHaveLength(2);
      expect(tabs[0].title).toBe("Discovered.docx");
      expect(tabs[1].title).toBe("Scratchpad.md");
      expect(app.getSplitGridLayoutMode()).toBe("split-horizontal");
    });
  });

  describe("8. Global Keyboard Shortcut for Workspace", () => {
    it("emits SaveWorkspace action on Ctrl+Alt+S and triggers workspace save", async () => {
      const app = new UnifiedShellApp({ autoBoot: false });
      const saveSpy = vi.spyOn(app, "saveWorkspace");

      const shortcuts = app.getKeyboardShortcuts();
      shortcuts.handleKeyDown(new KeyboardEvent("keydown", { key: "s", ctrlKey: true, altKey: true }));

      expect(saveSpy).toHaveBeenCalled();
    });
  });
});
