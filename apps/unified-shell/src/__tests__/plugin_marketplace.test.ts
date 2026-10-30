import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { pluginHost } from "../plugins/pluginHost";
import { FunctionWizardDialog } from "../components/FunctionWizardDialog";
import { BackstageMenu } from "../components/BackstageMenu";
import { PluginManagerModal } from "../components/PluginManagerModal";
import { UnifiedShellApp } from "../main";

describe("Custom Calculation UDF Registry & Plugin Marketplace UI (GEC-v2.0 Subphase 16.1.3)", () => {
  beforeEach(() => {
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

  describe("1. Plugin UDF Registration & Function Wizard Integration", () => {
    it("registers dynamic calculation functions and exposes them in Function Wizard", () => {
      pluginHost.registerPlugin({
        id: "org.quant.blackscholes",
        name: "Quant Finance UDFs",
        version: "1.0.0",
        targetApps: ["Excel"],
        udfContributions: [
          {
            name: "BLACK_SCHOLES",
            description: "Calculates European option pricing using Black-Scholes formula",
            category: "User Defined",
            minArgs: 5,
            maxArgs: 5,
            parameters: [
              { name: "spot", type: "number" },
              { name: "strike", type: "number" },
              { name: "rate", type: "number" },
              { name: "vol", type: "number" },
              { name: "time", type: "number" },
            ],
          },
        ],
      });

      const wizard = new FunctionWizardDialog();
      wizard.show("User Defined");

      // Verify category select has User Defined
      const categorySelect = document.querySelector<HTMLSelectElement>(".fn-category-select");
      expect(categorySelect?.innerHTML).toContain("User Defined");

      // Function list should contain BLACK_SCHOLES
      const listContainer = document.querySelector(".fn-list-container");
      expect(listContainer?.textContent).toContain("BLACK_SCHOLES");

      // Details should display syntax and description
      const detailName = document.querySelector(".fn-detail-name");
      const detailSyntax = document.querySelector(".fn-detail-syntax");
      const detailDesc = document.querySelector(".fn-detail-desc");

      expect(detailName?.textContent).toBe("BLACK_SCHOLES");
      expect(detailSyntax?.textContent).toContain("spot, strike, rate, vol, time");
      expect(detailDesc?.textContent).toContain("European option pricing");

      wizard.hide();
    });
  });

  describe("2. Backstage File Menu 'Add-ins & Extensions' Entrypoint", () => {
    it("renders Plugins tab in Backstage and opens Plugin Manager modal", async () => {
      let openPluginsCalled = false;
      const backstage = new BackstageMenu("app");
      backstage.setCallbacks({
        onOpenPlugins: () => {
          openPluginsCalled = true;
        },
      });

      await backstage.show();

      // Verify Backstage has Add-ins & Extensions tab
      const pluginsTabBtn = document.querySelector<HTMLButtonElement>(
        '[data-testid="backstage-tab-plugins"]'
      );
      expect(pluginsTabBtn).not.toBeNull();
      expect(pluginsTabBtn?.textContent).toContain("Add-ins & Extensions");

      // Click tab to navigate to Plugins panel
      pluginsTabBtn?.click();

      const openManagerBtn = document.querySelector<HTMLButtonElement>(
        '[data-testid="btn-open-plugin-manager"]'
      );
      expect(openManagerBtn).not.toBeNull();

      openManagerBtn?.click();
      expect(openPluginsCalled).toBe(true);
      expect(backstage.isOpen()).toBe(false);
    });
  });

  describe("3. Plugin Manager Modal Installed List & Management Controls", () => {
    it("displays installed plugins with details, capability badges, and controls", () => {
      pluginHost.registerPlugin({
        id: "com.deepl.translator",
        name: "DeepL Document Translator",
        version: "2.1.0",
        author: "DeepL SE",
        icon: "🌐",
        description: "Translate paragraphs in real time.",
        sidebarContributions: [
          {
            id: "pane-deepl",
            title: "DeepL",
            icon: "🌐",
          },
        ],
      });

      const modal = new PluginManagerModal("app");
      modal.open("installed");

      const card = document.querySelector<HTMLElement>(
        '[data-testid="plugin-card-com.deepl.translator"]'
      );
      expect(card).not.toBeNull();
      expect(card?.textContent).toContain("DeepL Document Translator");
      expect(card?.textContent).toContain("v2.1.0");
      expect(card?.textContent).toContain("by DeepL SE");
      expect(card?.textContent).toContain("Custom Sidebar");

      modal.close();
    });

    it("toggles plugin enabled / disabled state", () => {
      pluginHost.registerPlugin({
        id: "plugin.math",
        name: "Math Tools",
        version: "1.0.0",
      });

      const modal = new PluginManagerModal("app");
      modal.open("installed");

      const toggle = document.querySelector<HTMLInputElement>(
        '[data-testid="plugin-toggle-plugin.math"]'
      );
      expect(toggle).not.toBeNull();
      expect(toggle?.checked).toBe(true);

      // Disable plugin
      toggle!.checked = false;
      toggle?.dispatchEvent(new Event("change"));

      expect(pluginHost.isPluginEnabled("plugin.math")).toBe(false);

      // Re-enable plugin
      const reQueryToggle = document.querySelector<HTMLInputElement>(
        '[data-testid="plugin-toggle-plugin.math"]'
      );
      reQueryToggle!.checked = true;
      reQueryToggle?.dispatchEvent(new Event("change"));

      expect(pluginHost.isPluginEnabled("plugin.math")).toBe(true);

      modal.close();
    });

    it("uninstalls a plugin from installed list", () => {
      pluginHost.registerPlugin({
        id: "plugin.temp",
        name: "Temporary Plugin",
        version: "1.0.0",
      });

      const modal = new PluginManagerModal("app");
      modal.open("installed");

      const uninstallBtn = document.querySelector<HTMLButtonElement>(
        '[data-testid="plugin-uninstall-plugin.temp"]'
      );
      expect(uninstallBtn).not.toBeNull();

      uninstallBtn?.click();

      expect(pluginHost.getPlugin("plugin.temp")).toBeUndefined();
      expect(
        document.querySelector('[data-testid="plugin-card-plugin.temp"]')
      ).toBeNull();

      modal.close();
    });
  });

  describe("4. Marketplace Catalog & In-App Installation", () => {
    it("switches to Marketplace tab and installs an extension", () => {
      const modal = new PluginManagerModal("app");
      modal.open("marketplace");

      const marketplaceTab = document.querySelector<HTMLButtonElement>(
        '[data-testid="tab-marketplace"]'
      );
      expect(marketplaceTab?.classList.contains("active")).toBe(true);

      // Verify marketplace card exists for Wolfram
      const wolframCard = document.querySelector<HTMLElement>(
        '[data-testid="plugin-card-org.wolfram.calc.engine"]'
      );
      expect(wolframCard).not.toBeNull();
      expect(wolframCard?.textContent).toContain("Wolfram Calculation UDFs");

      // Click install
      const installBtn = document.querySelector<HTMLButtonElement>(
        '[data-testid="plugin-install-org.wolfram.calc.engine"]'
      );
      expect(installBtn).not.toBeNull();
      installBtn?.click();

      // Plugin is now registered in pluginHost
      expect(pluginHost.getPlugin("org.wolfram.calc.engine")).toBeDefined();

      // Card now indicates "Installed"
      const updatedCard = document.querySelector<HTMLElement>(
        '[data-testid="plugin-card-org.wolfram.calc.engine"]'
      );
      expect(updatedCard?.textContent).toContain("Installed");

      modal.close();
    });
  });

  describe("5. Search Filtering & Keyboard Handling", () => {
    it("filters plugins in real time via search bar", () => {
      pluginHost.registerPlugin({
        id: "p1",
        name: "Alpha Translator",
        version: "1.0.0",
        description: "Translate words",
      });
      pluginHost.registerPlugin({
        id: "p2",
        name: "Beta Calculator",
        version: "1.0.0",
        description: "Perform math",
      });

      const modal = new PluginManagerModal("app");
      modal.open("installed");

      const searchInput = document.querySelector<HTMLInputElement>(
        '[data-testid="plugin-search-input"]'
      )!;

      // Filter by 'Calculator'
      searchInput.value = "Calculator";
      searchInput.dispatchEvent(new Event("input"));

      expect(document.querySelector('[data-testid="plugin-card-p1"]')).toBeNull();
      expect(document.querySelector('[data-testid="plugin-card-p2"]')).not.toBeNull();

      // Clear search
      searchInput.value = "";
      searchInput.dispatchEvent(new Event("input"));

      expect(document.querySelector('[data-testid="plugin-card-p1"]')).not.toBeNull();
      expect(document.querySelector('[data-testid="plugin-card-p2"]')).not.toBeNull();

      modal.close();
    });

    it("closes modal on close button click and on Escape key", () => {
      const modal = new PluginManagerModal("app");
      modal.open();
      expect(modal.isOpen()).toBe(true);

      const closeBtn = document.querySelector<HTMLButtonElement>(
        '[data-testid="plugin-manager-close"]'
      )!;
      closeBtn.click();
      expect(modal.isOpen()).toBe(false);

      modal.open();
      expect(modal.isOpen()).toBe(true);

      window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
      expect(modal.isOpen()).toBe(false);
    });
  });

  describe("6. UnifiedShellApp Integration", () => {
    it("opens Plugin Manager through UnifiedShellApp action dispatch", () => {
      const app = new UnifiedShellApp(false);
      const modal = app.getPluginManagerModal();
      expect(modal.isOpen()).toBe(false);

      pluginHost.dispatchAction("OpenPluginManager");
      // Or via ribbon action TogglePluginManager
      const ribbon = (app as unknown as { ribbon: { onActionCallback?: (action: string) => void } }).ribbon;
      if (ribbon.onActionCallback) {
        ribbon.onActionCallback("TogglePluginManager");
      }

      expect(modal.isOpen()).toBe(true);
      modal.close();
    });
  });
});
