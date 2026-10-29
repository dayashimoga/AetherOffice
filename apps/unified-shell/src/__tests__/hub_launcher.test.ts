import { describe, it, expect, beforeEach, vi } from "vitest";
import { HubLauncher, DEFAULT_TEMPLATES, INITIAL_RECENT_FILES } from "../apps/hub/HubLauncher";
import { UnifiedShellApp } from "../main";
import { IPC } from "../ipc";

describe("AetherHub Main Suite Launcher & Process Spawner (GEC-v2.0 Subphase 8.1.2)", () => {
  let container: HTMLDivElement;

  beforeEach(() => {
    localStorage.clear();
    IPC._resetMockState();
    document.body.innerHTML = `
      <div id="app" class="desktop-shell">
        <header class="title-bar">
          <div class="app-branding">
            <span class="app-logo">⚡</span>
            <span class="app-name">AetherOffice</span>
          </div>
        </header>
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
    container = document.getElementById("viewport-container") as HTMLDivElement;
  });

  it("renders 4 prominent App Launch Cards for Writer, Calc, Slides, and PDF Studio", async () => {
    const hub = new HubLauncher(container);
    await hub.render();

    expect(container.querySelector('[data-testid="hub-launcher"]')).not.toBeNull();

    // 4 App Cards
    expect(container.querySelector('[data-testid="app-card-writer"]')).not.toBeNull();
    expect(container.querySelector('[data-testid="app-card-calc"]')).not.toBeNull();
    expect(container.querySelector('[data-testid="app-card-slides"]')).not.toBeNull();
    expect(container.querySelector('[data-testid="app-card-pdf"]')).not.toBeNull();

    // Verify format badges and descriptions
    const writerText = container.querySelector('[data-testid="app-card-writer"]')?.textContent;
    expect(writerText).toContain("AetherWriter");
    expect(writerText).toContain(".docx");

    const calcText = container.querySelector('[data-testid="app-card-calc"]')?.textContent;
    expect(calcText).toContain("AetherCalc");
    expect(calcText).toContain(".xlsx");

    const slidesText = container.querySelector('[data-testid="app-card-slides"]')?.textContent;
    expect(slidesText).toContain("AetherSlides");
    expect(slidesText).toContain(".pptx");

    const pdfText = container.querySelector('[data-testid="app-card-pdf"]')?.textContent;
    expect(pdfText).toContain("AetherPDF");
    expect(pdfText).toContain(".pdf");
  });

  it("launches standalone child processes upon clicking App Launch Cards", async () => {
    const hub = new HubLauncher(container);
    const launchSpy = vi.fn();
    hub.setCallbacks({ onLaunchApp: launchSpy });
    await hub.render();

    const writerBtn = container.querySelector<HTMLButtonElement>('[data-launch-app="writer"]');
    expect(writerBtn).not.toBeNull();
    writerBtn?.click();

    // Wait for async dispatch
    await new Promise((r) => setTimeout(r, 10));
    expect(launchSpy).toHaveBeenCalledWith("writer", undefined);

    const calcBtn = container.querySelector<HTMLButtonElement>('[data-launch-app="calc"]');
    calcBtn?.click();
    await new Promise((r) => setTimeout(r, 10));
    expect(launchSpy).toHaveBeenCalledWith("calc", undefined);
  });

  it("renders Template Gallery and launches apps with pre-structured template files", async () => {
    const hub = new HubLauncher(container);
    const templateSpy = vi.fn();
    hub.setCallbacks({ onSelectTemplate: templateSpy });
    await hub.render();

    expect(DEFAULT_TEMPLATES.length).toBe(4);
    const tplCards = container.querySelectorAll("[data-template-id]");
    expect(tplCards.length).toBe(4);

    // Click Budget sheet template
    const budgetCard = container.querySelector<HTMLElement>('[data-template-id="tpl-calc-budget"]');
    expect(budgetCard).not.toBeNull();
    budgetCard?.click();

    await new Promise((r) => setTimeout(r, 10));
    expect(templateSpy).toHaveBeenCalled();
    const calledTpl = templateSpy.mock.calls[0][0];
    expect(calledTpl.id).toBe("tpl-calc-budget");
    expect(calledTpl.appMode).toBe("calc");
    expect(calledTpl.defaultFilename).toBe("Annual_Budget.xlsx");
  });

  it("persists, displays, and manages Recent Documents list in localStorage", async () => {
    const hub = new HubLauncher(container);
    await hub.render();

    // Initial default files rendered
    const table = container.querySelector('[data-testid="recent-files-table"]');
    expect(table).not.toBeNull();
    const rows = container.querySelectorAll("[data-recent-index]");
    expect(rows.length).toBe(INITIAL_RECENT_FILES.length);

    // Remove one file
    const removeBtn = container.querySelector<HTMLButtonElement>('[data-remove-recent="0"]');
    removeBtn?.click();
    await new Promise((r) => setTimeout(r, 10));

    const rowsAfter = container.querySelectorAll("[data-recent-index]");
    expect(rowsAfter.length).toBe(INITIAL_RECENT_FILES.length - 1);

    // Clear all recent files
    const clearBtn = container.querySelector<HTMLButtonElement>("#btn-clear-recent");
    clearBtn?.click();
    await new Promise((r) => setTimeout(r, 10));

    expect(container.textContent).toContain("No recent documents found");
  });

  it("tracks and manages active running standalone instances from the Hub", async () => {
    // Pre-launch an instance via IPC
    const inst = await IPC.launchStandaloneApp("calc", "Budget2026.xlsx");
    expect(inst.process_id).toBeGreaterThan(0);

    const hub = new HubLauncher(container);
    await hub.render();

    const instTable = container.querySelector('[data-testid="running-instances-table"]');
    expect(instTable).not.toBeNull();
    expect(instTable?.textContent).toContain(inst.instance_id);
    expect(instTable?.textContent).toContain("AetherCalc");

    // Close instance
    const closeBtn = container.querySelector<HTMLButtonElement>(`[data-close-instance="${inst.instance_id}"]`);
    expect(closeBtn).not.toBeNull();
    closeBtn?.click();

    await new Promise((r) => setTimeout(r, 20));
    const instTableAfter = container.querySelector('[data-testid="running-instances-table"]');
    expect(instTableAfter).toBeNull();
    expect(container.textContent).toContain("No external child instances currently running");
  });

  it("boots UnifiedShellApp into AetherHub mode with full UI layout adaptation", async () => {
    const app = new UnifiedShellApp(false);
    app.applyBootConfig({ app_mode: "hub" });

    const appEl = document.getElementById("app");
    expect(appEl?.classList.contains("shell-hub-mode")).toBe(true);

    const hubLauncher = app.getHubLauncher();
    expect(hubLauncher).toBeDefined();

    const docInfo = document.getElementById("status-doc-info");
    expect(docInfo?.textContent).toBe("AetherOffice Suite Hub");

    const wordCount = document.getElementById("status-word-count");
    expect(wordCount?.textContent).toContain("4 Apps Available");
  });
});
