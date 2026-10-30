import { describe, it, expect, beforeEach, vi } from "vitest";
import { UnifiedShellApp } from "../main";
import { themeManager } from "../components/ThemeManager";
import { Ribbon } from "../components/Ribbon";
import { IPC } from "../ipc";

describe("Subphase 28.1.2: Zero Ghost Sidebars, Alt-Badges & Full-App Theme Cascade", () => {
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

    document.body.className = "";
    document.documentElement.className = "";

    document.body.innerHTML = `
      <div id="app" class="desktop-shell">
        <header class="title-bar">
          <div class="app-branding">
            <span class="app-logo">📄</span>
            <span class="app-name">OpenOfficeRS</span>
          </div>
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
        <div class="shell-body">
          <div id="app-rail-container"></div>
          <div class="shell-workspace-area">
            <div id="tab-bar-container"></div>
            <div id="ribbon-container"></div>
            <div id="sub-header-container"></div>
            <main class="workspace-layout">
              <aside id="sidebar-container" class="sidebar-pane"></aside>
              <section id="viewport-container" class="canvas-viewport"></section>
            </main>
            <footer id="status-bar" class="status-bar">
              <div class="status-left">
                <span id="status-doc-info">Page 1 of 1</span>
                <span class="status-divider">|</span>
                <span id="status-word-count">0 words</span>
                <span class="status-divider">|</span>
                <span id="status-lang">English (US)</span>
              </div>
              <div class="status-right">
                <span id="status-zoom-level">100%</span>
                <input type="range" id="zoom-slider" min="20" max="400" value="100" />
              </div>
            </footer>
          </div>
        </div>
      </div>
    `;
  });

  describe("1. Zero Ghost Sidebars (GEC-v4.0 Article II.1)", () => {
    it("renders Word Navigation sidebar ONLY in Word and hides when showNavPane is false", () => {
      const app = new UnifiedShellApp(false);
      app.switchToOrCreateApp("Word");

      const sidebar = document.getElementById("sidebar-container")!;
      expect(sidebar.style.display).toBe("flex");
      expect(sidebar.innerHTML).toContain("Navigation");

      // Toggle off nav pane
      app.setShowNavPane(false);
      expect(sidebar.style.display).toBe("none");
      expect(sidebar.innerHTML).toBe("");

      // Toggle back on
      app.setShowNavPane(true);
      expect(sidebar.style.display).toBe("flex");
      expect(sidebar.innerHTML).toContain("Navigation");
    });

    it("NEVER bleeds Word Navigation sidebar into PDF, Notes, or Excel", () => {
      const app = new UnifiedShellApp(false);
      const sidebar = document.getElementById("sidebar-container")!;

      // Switch to PDF
      app.switchToOrCreateApp("PDF");
      expect(sidebar.style.display).toBe("none");
      expect(sidebar.innerHTML).toBe("");
      expect(sidebar.innerHTML).not.toContain("Navigation");

      // Switch to Notes
      app.switchToOrCreateApp("Notes");
      expect(sidebar.style.display).toBe("none");
      expect(sidebar.innerHTML).toBe("");
      expect(sidebar.innerHTML).not.toContain("Navigation");

      // Switch to Excel
      app.switchToOrCreateApp("Excel");
      expect(sidebar.innerHTML).not.toContain("Navigation");
      expect(sidebar.innerHTML).toContain("Workbook Sheets");
    });
  });

  describe("2. Alt-Keytips Visibility (GEC-v4.0 Article II.2)", () => {
    it("hides keytip badges by default and toggles class on body when Alt is triggered", () => {
      const ribbon = new Ribbon("ribbon-container");

      expect(document.body.classList.contains("alt-keytips-active")).toBe(false);

      // Activate keytips
      ribbon.toggleKeytips(true);
      expect(document.body.classList.contains("alt-keytips-active")).toBe(true);

      const header = document.querySelector(".ribbon-tabs-header");
      expect(header?.classList.contains("keytips-active")).toBe(true);

      // Deactivate keytips
      ribbon.toggleKeytips(false);
      expect(document.body.classList.contains("alt-keytips-active")).toBe(false);
      const newHeader = document.querySelector(".ribbon-tabs-header");
      expect(newHeader?.classList.contains("keytips-active")).toBe(false);
    });
  });

  describe("3. Full-App Dark / Light Theme Cascade (GEC-v4.0 Article II.3)", () => {
    it("cascades theme classes and data attributes across html, body, and #app", () => {
      themeManager.init("#btn-theme-toggle");

      themeManager.setTheme("dark");
      expect(document.documentElement.dataset.theme).toBe("dark");
      expect(document.documentElement.classList.contains("theme-dark")).toBe(true);
      expect(document.body.classList.contains("theme-dark")).toBe(true);
      expect(document.getElementById("app")?.classList.contains("theme-dark")).toBe(true);

      themeManager.setTheme("light");
      expect(document.documentElement.dataset.theme).toBe("light");
      expect(document.documentElement.classList.contains("theme-light")).toBe(true);
      expect(document.body.classList.contains("theme-light")).toBe(true);
      expect(document.getElementById("app")?.classList.contains("theme-light")).toBe(true);
    });
  });

  describe("4. Dynamic Per-App Status Bar (GEC-v4.0 Article II.4)", () => {
    it("displays live metrics per app type with zero hardcoded 22 words", () => {
      const app = new UnifiedShellApp(false);
      const docInfo = document.getElementById("status-doc-info")!;
      const wordInfo = document.getElementById("status-word-count")!;
      const langInfo = document.getElementById("status-lang")!;

      // Word
      app.switchToOrCreateApp("Word");
      expect(docInfo.textContent).toContain("Page 1 of");
      expect(wordInfo.textContent).toContain("words");
      expect(wordInfo.textContent).not.toContain("22 words");
      expect(langInfo.textContent).toBe("English (US)");

      // Excel
      app.switchToOrCreateApp("Excel");
      expect(docInfo.textContent).toContain("Sheet 1 of");
      expect(wordInfo.textContent).toBe("Selected: A1");
      expect(wordInfo.textContent).not.toContain("22 words");

      // PowerPoint
      app.switchToOrCreateApp("PowerPoint");
      expect(docInfo.textContent).toContain("Slide 1 of");
      expect(wordInfo.textContent).toContain("shapes");
      expect(wordInfo.textContent).not.toContain("22 words");

      // PDF
      app.switchToOrCreateApp("PDF");
      expect(docInfo.textContent).toContain("Page 1 of");
      expect(wordInfo.textContent).toContain("in");
      expect(wordInfo.textContent).not.toContain("22 words");
      expect(langInfo.textContent).toContain("Standard");

      // Notes
      app.switchToOrCreateApp("Notes");
      expect(docInfo.textContent).toContain("Ln 1, Col 1");
      expect(wordInfo.textContent).toContain("lines");
      expect(wordInfo.textContent).not.toContain("22 words");
      expect(langInfo.textContent).toBe("UTF-8");
    });
  });
});
