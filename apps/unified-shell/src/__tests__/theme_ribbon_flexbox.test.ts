import { describe, it, expect, beforeEach, vi } from "vitest";
import { ThemeManager, themeManager } from "../components/ThemeManager";
import { Ribbon } from "../components/Ribbon";
import { UnifiedShellApp } from "../main";

describe("Theme Engine & Non-Overflowing Flexbox Ribbon (GEC-v3.0 Subphase 21.2.1)", () => {
  beforeEach(() => {
    // Mock 2D canvas context for viewport components if initialized
    HTMLCanvasElement.prototype.getContext = vi.fn().mockReturnValue({
      clearRect: vi.fn(),
      fillRect: vi.fn(),
      fillText: vi.fn(),
      strokeRect: vi.fn(),
      beginPath: vi.fn(),
      moveTo: vi.fn(),
      lineTo: vi.fn(),
      stroke: vi.fn(),
      fill: vi.fn(),
      measureText: vi.fn().mockReturnValue({ width: 50 }),
      save: vi.fn(),
      restore: vi.fn(),
      scale: vi.fn(),
      translate: vi.fn(),
    });

    localStorage.clear();
    document.documentElement.removeAttribute("data-theme");

    document.body.innerHTML = `
      <div id="app">
        <header class="title-bar">
          <div class="title-actions">
            <button id="btn-theme-toggle" class="theme-toggle-btn">
              <span class="theme-icon">☀️</span>
              <span class="theme-label">Light</span>
            </button>
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
        <div id="sidebar-container"></div>
        <div id="viewport-container"></div>
        <div id="bottom-sheet"></div>
        <canvas id="stylus-ink-canvas"></canvas>
        <div id="reflow-reader-container"></div>
        <div id="row-card-inspector-container"></div>
        <div id="laser-pointer-dot"></div>
        <span id="status-doc-info"></span>
        <span id="status-word-count"></span>
        <span id="status-zoom-level"></span>
        <input type="range" id="zoom-slider" />
      </div>
    `;
  });

  describe("ThemeManager", () => {
    it("initializes with default light theme and sets dataset.theme", () => {
      const tm = new ThemeManager("#btn-theme-toggle");
      expect(tm.getTheme()).toBe("light");
      expect(document.documentElement.dataset.theme).toBe("light");
    });

    it("toggles between light and dark themes on click", () => {
      const tm = new ThemeManager("#btn-theme-toggle");
      const btn = document.querySelector<HTMLButtonElement>("#btn-theme-toggle");
      expect(btn).not.toBeNull();

      btn?.click();
      expect(tm.getTheme()).toBe("dark");
      expect(document.documentElement.dataset.theme).toBe("dark");
      expect(localStorage.getItem("aether_theme")).toBe("dark");
      expect(btn?.textContent).toContain("Dark");

      btn?.click();
      expect(tm.getTheme()).toBe("light");
      expect(document.documentElement.dataset.theme).toBe("light");
      expect(localStorage.getItem("aether_theme")).toBe("light");
      expect(btn?.textContent).toContain("Light");
    });

    it("restores saved theme preference from localStorage", () => {
      localStorage.setItem("aether_theme", "dark");
      const tm = new ThemeManager("#btn-theme-toggle");
      expect(tm.getTheme()).toBe("dark");
      expect(document.documentElement.dataset.theme).toBe("dark");
    });
  });

  describe("Non-Overflowing Flexbox Ribbon", () => {
    it("renders ribbon tabs and switches active tab cleanly", () => {
      const ribbon = new Ribbon("ribbon-container");
      const homeTab = document.querySelector<HTMLButtonElement>('[data-testid="ribbon-tab-home"]');
      const insertTab = document.querySelector<HTMLButtonElement>('[data-testid="ribbon-tab-insert"]');

      expect(homeTab).not.toBeNull();
      expect(insertTab).not.toBeNull();
      expect(homeTab?.classList.contains("active")).toBe(true);

      insertTab?.click();
      expect(ribbon.getActiveTab()).toBe("Insert");
      const activeInsert = document.querySelector<HTMLButtonElement>('[data-testid="ribbon-tab-insert"]');
      expect(activeInsert?.classList.contains("active")).toBe(true);
    });

    it("highlights File tab only when Backstage is open", () => {
      const ribbon = new Ribbon("ribbon-container");
      const fileTab = document.querySelector<HTMLButtonElement>('[data-testid="ribbon-tab-file"]');
      expect(fileTab).not.toBeNull();
      expect(fileTab?.classList.contains("backstage-active")).toBe(false);

      ribbon.setBackstageActive(true);
      const activeFile = document.querySelector<HTMLButtonElement>('[data-testid="ribbon-tab-file"]');
      expect(activeFile?.classList.contains("backstage-active")).toBe(true);

      ribbon.setBackstageActive(false);
      const normalFile = document.querySelector<HTMLButtonElement>('[data-testid="ribbon-tab-file"]');
      expect(normalFile?.classList.contains("backstage-active")).toBe(false);
    });

    it("dynamically adapts tabs and accent colors across Word, Excel, and PowerPoint", () => {
      const ribbon = new Ribbon("ribbon-container");

      ribbon.setDocumentType("Word");
      expect(ribbon.getDocAccentColor()).toBe("#2b579a");

      ribbon.setDocumentType("Excel");
      expect(ribbon.getDocAccentColor()).toBe("#107c41");
      expect(ribbon.getVisibleTabs()).toContain("Formulas");
      expect(ribbon.getVisibleTabs()).toContain("Data");

      ribbon.setDocumentType("PowerPoint");
      expect(ribbon.getDocAccentColor()).toBe("#c43e1c");
      expect(ribbon.getVisibleTabs()).toContain("Transitions");
      expect(ribbon.getVisibleTabs()).toContain("Animations");
    });
  });

  describe("UnifiedShellApp Theme & Ribbon Integration", () => {
    it("boots unified shell with themeManager initialized", () => {
      const app = new UnifiedShellApp(false);
      expect(app).toBeDefined();

      const btn = document.querySelector<HTMLButtonElement>("#btn-theme-toggle");
      expect(btn).not.toBeNull();

      btn?.click();
      expect(themeManager.getTheme()).toBe("dark");
      expect(document.documentElement.dataset.theme).toBe("dark");
    });
  });
});

