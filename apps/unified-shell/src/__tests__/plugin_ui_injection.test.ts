import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { pluginHost, PluginManifest } from "../plugins/pluginHost";
import { Ribbon } from "../components/Ribbon";
import { UnifiedShellApp } from "../main";

describe("Dynamic UI Ribbon & Sidebar Injection API (GEC-v2.0 Subphase 16.1.2)", () => {
  beforeEach(() => {
    // Reset DOM
    document.body.innerHTML = `
      <div id="app" class="desktop-shell">
        <header class="title-bar">
          <div class="window-controls">
            <button class="win-btn win-min">−</button>
            <button class="win-btn win-max">□</button>
            <button class="win-btn win-close">×</button>
          </div>
        </header>
        <nav id="tab-bar-container" class="tab-bar"></nav>
        <section id="ribbon-container" class="ribbon-bar"></section>
        <section id="sub-header-container" class="sub-header-bar"></section>
        <main class="workspace-layout">
          <aside id="sidebar-container" class="sidebar-pane"></aside>
          <section class="canvas-viewport" id="viewport-container">
            <canvas id="render-surface"></canvas>
          </section>
        </main>
        <div id="bottom-sheet" class="bottom-sheet"></div>
        <footer id="status-bar" class="status-bar">
          <div class="status-left">
            <span id="status-doc-info">Page 1 of 1</span>
            <span id="status-word-count">0 words</span>
          </div>
        </footer>
      </div>
    `;

    pluginHost.clear();
  });

  afterEach(() => {
    pluginHost.clear();
  });

  describe("1. Plugin Manifest Parser & Schema Validation", () => {
    it("parses valid JSON string manifest and normalizes defaults", () => {
      const json = JSON.stringify({
        id: "com.example.translator",
        name: "DeepL Translator",
        version: "1.2.0",
        description: "Translate document runs in real-time",
        author: "DeepL GmbH",
      });

      const manifest = pluginHost.parseManifest(json);
      expect(manifest.id).toBe("com.example.translator");
      expect(manifest.name).toBe("DeepL Translator");
      expect(manifest.version).toBe("1.2.0");
      expect(manifest.targetApps).toEqual(["All"]);
      expect(manifest.enabled).toBe(true);
    });

    it("throws descriptive error when required fields are missing", () => {
      expect(() => pluginHost.parseManifest("{}")).toThrow(
        "Invalid plugin manifest: missing required 'id'"
      );

      expect(() =>
        pluginHost.parseManifest({ id: "test-id" } as unknown as PluginManifest)
      ).toThrow("Invalid plugin manifest: missing required 'name'");

      expect(() =>
        pluginHost.parseManifest({ id: "test-id", name: "Test" } as unknown as PluginManifest)
      ).toThrow("Invalid plugin manifest: missing required 'version'");
    });
  });

  describe("2. Plugin Lifecycle & Enable/Disable State", () => {
    it("handles registration, toggle enable/disable, and unregistration", () => {
      const lifecycleEvents: string[] = [];
      pluginHost.onLifecycle((event, id) => {
        lifecycleEvents.push(`${event}:${id}`);
      });

      pluginHost.registerPlugin({
        id: "plugin.grammar",
        name: "Grammar Checker",
        version: "2.0.0",
      });

      expect(pluginHost.getAllPlugins().length).toBe(1);
      expect(pluginHost.isPluginEnabled("plugin.grammar")).toBe(true);
      expect(lifecycleEvents).toContain("plugin-registered:plugin.grammar");

      // Disable
      pluginHost.disablePlugin("plugin.grammar");
      expect(pluginHost.isPluginEnabled("plugin.grammar")).toBe(false);
      expect(pluginHost.getActivePlugins().length).toBe(0);
      expect(lifecycleEvents).toContain("plugin-disabled:plugin.grammar");

      // Enable
      pluginHost.enablePlugin("plugin.grammar");
      expect(pluginHost.isPluginEnabled("plugin.grammar")).toBe(true);
      expect(pluginHost.getActivePlugins().length).toBe(1);
      expect(lifecycleEvents).toContain("plugin-enabled:plugin.grammar");

      // Unregister
      pluginHost.unregisterPlugin("plugin.grammar");
      expect(pluginHost.getAllPlugins().length).toBe(0);
      expect(lifecycleEvents).toContain("plugin-unregistered:plugin.grammar");
    });
  });

  describe("3. Dynamic Custom Ribbon Tab Injection", () => {
    it("injects custom tab into Ribbon alongside standard tabs and supports keytips", () => {
      const ribbon = new Ribbon("ribbon-container");
      const initialTabs = ribbon.getVisibleTabs();

      // Register plugin contributing an 'AI Assistant' tab
      pluginHost.registerPlugin({
        id: "org.aether.ai",
        name: "AI Assistant",
        version: "1.0.0",
        ribbonContributions: {
          tabs: [
            {
              id: "tab-ai-assistant",
              label: "AI Assistant",
              keytip: "A",
              groups: [
                {
                  id: "grp-generation",
                  title: "Generation",
                  buttons: [
                    {
                      id: "btn-draft",
                      label: "Draft",
                      icon: "✨",
                      action: "ai:draft-text",
                      tooltip: "Generate draft content with AI",
                      testId: "plugin-btn-draft",
                    },
                  ],
                },
              ],
            },
          ],
        },
      });

      const updatedTabs = ribbon.getVisibleTabs();
      expect(updatedTabs).toEqual([...initialTabs, "AI Assistant"]);

      // Verify DOM rendering of injected tab header
      const aiTabBtn = document.querySelector<HTMLButtonElement>(
        '[data-testid="ribbon-tab-ai-assistant"]'
      );
      expect(aiTabBtn).not.toBeNull();
      expect(aiTabBtn?.textContent).toContain("AI Assistant");

      // Verify Keytips
      ribbon.toggleKeytips(true);
      const updatedAiTabBtn = document.querySelector<HTMLButtonElement>(
        '[data-testid="ribbon-tab-ai-assistant"]'
      );
      const keytipBadge = updatedAiTabBtn?.querySelector(".ribbon-keytip-badge");
      expect(keytipBadge?.textContent).toBe("A");
    });

    it("activates custom tab and renders custom groups and buttons with dispatch wiring", () => {
      let dispatchedAction = "";
      const ribbon = new Ribbon("ribbon-container");

      pluginHost.registerPlugin({
        id: "org.aether.translate",
        name: "Translation Suite",
        version: "1.0.0",
        ribbonContributions: {
          tabs: [
            {
              id: "tab-translate",
              label: "Translate",
              groups: [
                {
                  id: "grp-languages",
                  title: "Languages",
                  buttons: [
                    {
                      id: "btn-spanish",
                      label: "Spanish",
                      icon: "🇪🇸",
                      action: "translate:es",
                      testId: "plugin-btn-spanish",
                    },
                    {
                      id: "btn-french",
                      label: "French",
                      icon: "🇫🇷",
                      action: "translate:fr",
                      testId: "plugin-btn-french",
                    },
                  ],
                },
              ],
            },
          ],
        },
      });

      ribbon.setOnAction((act) => {
        dispatchedAction = act;
      });

      // Switch to custom tab
      ribbon.setActiveTab("Translate");
      expect(ribbon.getActiveTab()).toBe("Translate");

      const body = document.getElementById("ribbon-tools-body");
      expect(body?.textContent).toContain("Languages");

      const spanishBtn = body?.querySelector<HTMLButtonElement>(
        '[data-testid="plugin-btn-spanish"]'
      );
      expect(spanishBtn).not.toBeNull();
      expect(spanishBtn?.textContent).toBe("🇪🇸");

      spanishBtn?.click();
      expect(dispatchedAction).toBe("translate:es");
    });
  });

  describe("4. Dynamic Ribbon Group Injection into Standard Tabs", () => {
    it("appends custom plugin groups to standard Home tab", () => {
      let triggeredAction = "";
      const ribbon = new Ribbon("ribbon-container");
      ribbon.setOnAction((act) => {
        triggeredAction = act;
      });

      pluginHost.registerPlugin({
        id: "com.productivity.stamps",
        name: "Watermark Stamps",
        version: "1.0.0",
        ribbonContributions: {
          injectedGroups: [
            {
              targetTab: "Home",
              groups: [
                {
                  id: "grp-stamps",
                  title: "Security Stamps",
                  buttons: [
                    {
                      id: "btn-confidential",
                      label: "Confidential",
                      icon: "🔒",
                      action: "stamp:confidential",
                      testId: "plugin-btn-confidential",
                    },
                  ],
                },
              ],
            },
          ],
        },
      });

      // Ribbon is currently on Home tab
      const body = document.getElementById("ribbon-tools-body");
      expect(body?.textContent).toContain("Security Stamps");

      const stampBtn = body?.querySelector<HTMLButtonElement>(
        '[data-testid="plugin-btn-confidential"]'
      );
      expect(stampBtn).not.toBeNull();
      expect(stampBtn?.textContent).toBe("🔒");

      stampBtn?.click();
      expect(triggeredAction).toBe("stamp:confidential");
    });
  });

  describe("5. Target App Filtering", () => {
    it("respects targetApps filter (e.g. Excel only vs Word only)", () => {
      const ribbon = new Ribbon("ribbon-container");

      pluginHost.registerPlugin({
        id: "calc-only-plugin",
        name: "Financial Modeler",
        version: "1.0.0",
        targetApps: ["Excel"],
        ribbonContributions: {
          tabs: [
            {
              id: "tab-financial",
              label: "Finance",
              groups: [],
            },
          ],
        },
      });

      // Default docType is Word -> Finance tab must NOT appear
      ribbon.setDocumentType("Word");
      expect(ribbon.getVisibleTabs()).not.toContain("Finance");

      // Switch to Excel -> Finance tab MUST appear
      ribbon.setDocumentType("Excel");
      expect(ribbon.getVisibleTabs()).toContain("Finance");

      // Switch to PowerPoint -> Finance tab must NOT appear
      ribbon.setDocumentType("PowerPoint");
      expect(ribbon.getVisibleTabs()).not.toContain("Finance");
    });
  });

  describe("6. Custom Sandboxed Sidebar Pane Registration & Rendering", () => {
    it("renders custom plugin sidebar in #sidebar-container with sandboxed iframe", () => {
      const app = new UnifiedShellApp(false);

      pluginHost.registerPlugin({
        id: "plugin.translator.sidebar",
        name: "Translator Sidebar",
        version: "1.0.0",
        sidebarContributions: [
          {
            id: "pane-translator",
            title: "Live Translator",
            icon: "🌐",
            width: 320,
            entrypointHtml: "<h1>Translator UI</h1><p>Ready to translate</p>",
            sandboxOptions: "allow-scripts allow-forms",
          },
        ],
      });

      const sidebarContainer = document.getElementById("sidebar-container")!;
      expect(sidebarContainer.classList.contains("collapsed")).toBe(false);

      // Toggle open sidebar
      app.togglePluginSidebar("pane-translator");

      expect(sidebarContainer.style.display).toBe("flex");
      expect(sidebarContainer.style.width).toBe("320px");
      expect(sidebarContainer.textContent).toContain("Live Translator");
      expect(sidebarContainer.textContent).toContain("🌐");

      // Verify sandboxed iframe
      const iframe = sidebarContainer.querySelector<HTMLIFrameElement>(
        '[data-testid="plugin-sidebar-iframe"]'
      );
      expect(iframe).not.toBeNull();
      expect(iframe?.getAttribute("sandbox")).toBe("allow-scripts allow-forms");
      expect(iframe?.srcdoc).toContain("<h1>Translator UI</h1>");

      // Verify close button closes sidebar
      const closeBtn = sidebarContainer.querySelector<HTMLButtonElement>(
        '[data-testid="plugin-sidebar-close"]'
      );
      expect(closeBtn).not.toBeNull();
      closeBtn?.click();

      expect(sidebarContainer.style.display).toBe("none");
      expect(app.getPluginSidebar().isVisible()).toBe(false);
    });

    it("toggles sidebar via Ribbon action command", () => {
      const app = new UnifiedShellApp(false);

      pluginHost.registerPlugin({
        id: "plugin.notes",
        name: "Sticky Notes",
        version: "1.0.0",
        sidebarContributions: [
          {
            id: "pane-notes",
            title: "Quick Notes",
            icon: "📝",
            entrypointHtml: "<p>My Notes</p>",
          },
        ],
      });

      const sidebarContainer = document.getElementById("sidebar-container")!;

      // Dispatch TogglePluginSidebar:pane-notes
      pluginHost.dispatchAction("TogglePluginSidebar:pane-notes");
      app.togglePluginSidebar("pane-notes");

      expect(app.getPluginSidebar().isVisible()).toBe(true);
      expect(sidebarContainer.textContent).toContain("Quick Notes");

      // Toggle again closes
      app.togglePluginSidebar("pane-notes");
      expect(app.getPluginSidebar().isVisible()).toBe(false);
    });
  });

  describe("7. Bidirectional PostMessage Bridge", () => {
    it("handles postMessage events from plugin iframe and dispatches to pluginHost", () => {
      const app = new UnifiedShellApp(false);
      let receivedAction = "";
      let receivedPayload: unknown = null;

      pluginHost.onAction((act, payload) => {
        receivedAction = act;
        receivedPayload = payload;
      });

      pluginHost.registerPlugin({
        id: "plugin.bridge.test",
        name: "Bridge Test Plugin",
        version: "1.0.0",
        sidebarContributions: [
          {
            id: "pane-bridge",
            title: "Bridge",
            icon: "⚡",
            entrypointHtml: "<p>Bridge</p>",
          },
        ],
      });

      app.togglePluginSidebar("pane-bridge");
      const iframe = document.querySelector<HTMLIFrameElement>(
        '[data-testid="plugin-sidebar-iframe"]'
      )!;

      // Simulate postMessage from iframe
      const event = new MessageEvent("message", {
        data: {
          type: "insert-glossary-term",
          payload: { term: "AetherOffice", definition: "Office productivity suite" },
        },
        source: iframe.contentWindow,
      });

      window.dispatchEvent(event);

      expect(receivedAction).toBe("sidebar:insert-glossary-term");
      expect(receivedPayload).toEqual({
        term: "AetherOffice",
        definition: "Office productivity suite",
      });
    });
  });
});
