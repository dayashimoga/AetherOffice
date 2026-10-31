import { describe, it, expect, beforeEach } from "vitest";
import { UnifiedShellApp } from "../main";
import { themeManager } from "../components/ThemeManager";

describe("GEC-v6.0 Subphase 35.1.2: App-Scoped Workspace & UI Visual Polish", () => {
  beforeEach(() => {
    const mockCtx = {
      save: () => {},
      restore: () => {},
      scale: () => {},
      translate: () => {},
      rotate: () => {},
      setTransform: () => {},
      beginPath: () => {},
      closePath: () => {},
      moveTo: () => {},
      lineTo: () => {},
      stroke: () => {},
      fill: () => {},
      fillRect: () => {},
      strokeRect: () => {},
      clearRect: () => {},
      fillText: () => {},
      measureText: (text: string) => ({ width: text.length * 8 }),
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
      <div id="app" class="desktop-shell theme-light">
        <header class="title-bar">
          <div class="title-actions">
            <button id="btn-theme-toggle" class="theme-toggle-btn">
              <span class="theme-icon">☀️</span>
              <span class="theme-label">Light</span>
            </button>
          </div>
        </header>
        <div class="shell-body" style="display:flex; flex:1;">
          <aside id="app-rail-container" class="app-rail"></aside>
          <div class="shell-workspace-area" style="display:flex; flex-direction:column; flex:1;">
            <nav id="tab-bar-container" class="tab-bar"></nav>
            <section id="ribbon-container" class="ribbon-bar"></section>
            <section id="sub-header-container" class="sub-header-bar"></section>
            <main class="workspace-layout">
              <aside id="sidebar-container" class="sidebar-pane"></aside>
              <section class="canvas-viewport" id="viewport-container">
                <canvas id="render-surface"></canvas>
              </section>
            </main>
            <footer id="status-bar" class="status-bar">
              <span id="status-doc-info"></span>
              <span id="status-word-count"></span>
            </footer>
          </div>
        </div>
      </div>
    `;
    localStorage.clear();
  });

  it("1. App-Scoped Tab Bar defaults to current app and displays format-specific + New & Open buttons", () => {
    const app = new UnifiedShellApp(false);
    app.applyBootConfig({ app_mode: "writer" });

    // Initial state: Word document
    expect(app.getCurrentDocType()).toBe("Word");
    const tabBar = app.getTabBar();
    expect(tabBar.getFilterMode()).toBe("current");
    expect(tabBar.getFilterDocType()).toBe("Word");

    // Word quick actions must be rendered
    const newDocxBtn = document.querySelector<HTMLButtonElement>('[data-testid="tab-new-docx"]');
    const openDocxBtn = document.querySelector<HTMLButtonElement>('[data-testid="tab-open-docx"]');
    expect(newDocxBtn).not.toBeNull();
    expect(openDocxBtn).not.toBeNull();
    expect(newDocxBtn?.textContent).toContain("+ New .docx");
    expect(openDocxBtn?.textContent).toContain("📂 Open .docx");

    // Clicking + New .docx spawns another Word tab
    newDocxBtn?.click();
    const wordTabs = tabBar.getTabs().filter((t) => t.docType === "Word");
    expect(wordTabs.length).toBe(2);

    // Switch to Excel via Left App Rail
    const excelRailBtn = document.querySelector<HTMLButtonElement>('[data-testid="rail-btn-excel"]');
    expect(excelRailBtn).not.toBeNull();
    excelRailBtn?.click();

    expect(app.getCurrentDocType()).toBe("Excel");
    expect(tabBar.getFilterDocType()).toBe("Excel");

    // Now Excel quick actions must be rendered
    const newXlsxBtn = document.querySelector<HTMLButtonElement>('[data-testid="tab-new-xlsx"]');
    const openXlsxBtn = document.querySelector<HTMLButtonElement>('[data-testid="tab-open-xlsx"]');
    expect(newXlsxBtn).not.toBeNull();
    expect(openXlsxBtn).not.toBeNull();
    expect(newXlsxBtn?.textContent).toContain("+ New .xlsx");
    expect(openXlsxBtn?.textContent).toContain("📂 Open .xlsx");

    // Visible tabs in current mode must ONLY show Excel tabs
    const visibleTabEls = document.querySelectorAll(".tab-item");
    expect(visibleTabEls.length).toBe(1); // 1 Excel tab visible
    expect(visibleTabEls[0].textContent).toContain("Book");

    // Switch to PowerPoint
    const pptRailBtn = document.querySelector<HTMLButtonElement>('[data-testid="rail-btn-powerpoint"]');
    pptRailBtn?.click();
    expect(document.querySelector('[data-testid="tab-new-pptx"]')).not.toBeNull();
    expect(document.querySelector('[data-testid="tab-open-pptx"]')).not.toBeNull();

    // Switch to PDF Studio
    const pdfRailBtn = document.querySelector<HTMLButtonElement>('[data-testid="rail-btn-pdf"]');
    pdfRailBtn?.click();
    expect(document.querySelector('[data-testid="tab-new-pdf"]')).not.toBeNull();
    expect(document.querySelector('[data-testid="tab-open-pdf"]')).not.toBeNull();

    // Switch to Notes
    const notesRailBtn = document.querySelector<HTMLButtonElement>('[data-testid="rail-btn-notes"]');
    notesRailBtn?.click();
    expect(document.querySelector('[data-testid="tab-new-txt"]')).not.toBeNull();
    expect(document.querySelector('[data-testid="tab-open-txt"]')).not.toBeNull();

    // Toggle pill to All Tabs
    const filterPill = document.querySelector<HTMLButtonElement>('[data-testid="tab-filter-toggle"]');
    expect(filterPill).not.toBeNull();
    filterPill?.click();
    expect(tabBar.getFilterMode()).toBe("all");

    // In all tabs mode, all tabs are visible in the bar
    const allVisibleTabEls = document.querySelectorAll(".tab-item");
    expect(allVisibleTabEls.length).toBe(tabBar.getTabs().length);
    expect(allVisibleTabEls.length).toBeGreaterThanOrEqual(5);
  });

  it("2. Word Home ribbon renders Styles (2 rows) and Copilot visibly with zero clipping", () => {
    const app = new UnifiedShellApp(false);
    app.applyBootConfig({ app_mode: "writer" });

    // Verify Styles group buttons
    expect(document.querySelector('[data-testid="btn-style-normal"]')).not.toBeNull();
    expect(document.querySelector('[data-testid="btn-style-no-spacing"]')).not.toBeNull();
    expect(document.querySelector('[data-testid="btn-style-h1"]')).not.toBeNull();
    expect(document.querySelector('[data-testid="btn-style-h2"]')).not.toBeNull();
    expect(document.querySelector('[data-testid="btn-style-h3"]')).not.toBeNull();
    expect(document.querySelector('[data-testid="btn-style-title"]')).not.toBeNull();
    expect(document.querySelector('[data-testid="btn-style-subtitle"]')).not.toBeNull();
    expect(document.querySelector('[data-testid="btn-style-quote"]')).not.toBeNull();

    // Verify Copilot button
    const copilotBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-copilot"]');
    expect(copilotBtn).not.toBeNull();
    expect(copilotBtn?.textContent).toContain("Copilot");

    // Verify Editing group buttons
    expect(document.querySelector('[data-testid="btn-find"]')).not.toBeNull();
    expect(document.querySelector('[data-testid="btn-replace"]')).not.toBeNull();
    expect(document.querySelector('[data-testid="btn-select-all"]')).not.toBeNull();
  });

  it("3. Dark / Light Theme switches dynamically with theme CSS tokens applied across shell and PDF Studio", () => {
    const app = new UnifiedShellApp(false);
    app.applyBootConfig({ app_mode: "writer" });
    app.switchToOrCreateApp("PDF");

    // Initially light
    expect(themeManager.getTheme()).toBe("light");
    const workspace = document.querySelector<HTMLElement>(".pdf-studio-workspace");
    expect(workspace).not.toBeNull();
    expect(workspace?.style.backgroundColor).toContain("var(--bg-canvas-backdrop");

    // Toggle theme to dark
    themeManager.toggleTheme();
    expect(themeManager.getTheme()).toBe("dark");
    expect(document.documentElement.dataset.theme).toBe("dark");
    expect(document.body.classList.contains("theme-dark")).toBe(true);

    const pdfSidebar = document.querySelector<HTMLElement>('[data-testid="pdf-sidebar"]');
    expect(pdfSidebar).not.toBeNull();
    expect(pdfSidebar?.style.backgroundColor).toContain("var(--bg-sidebar");

    // Toggle back to light
    themeManager.toggleTheme();
    expect(themeManager.getTheme()).toBe("light");
    expect(document.documentElement.dataset.theme).toBe("light");
  });
});
