import { describe, it, expect, beforeEach, vi } from "vitest";
import { WriterCanvas } from "../components/viewports/WriterCanvas";
import { WriterState } from "../state/writerState";
import { IPC } from "../ipc";

describe("Subphase 29.1.1: Word 100% Zoom Top-Visible Layout, Zero Scroll Flicker & Editable Headers/Footers", () => {
  let mockCtx: any;

  beforeEach(() => {
    localStorage.clear();
    IPC._resetMockState();

    mockCtx = {
      clearRect: vi.fn(),
      fillRect: vi.fn(),
      fillText: vi.fn(),
      strokeRect: vi.fn(),
      beginPath: vi.fn(),
      moveTo: vi.fn(),
      lineTo: vi.fn(),
      stroke: vi.fn(),
      fill: vi.fn(),
      measureText: vi.fn().mockImplementation((text: string) => ({ width: text.length * 8 })),
      save: vi.fn(),
      restore: vi.fn(),
      scale: vi.fn(),
      translate: vi.fn(),
      setTransform: vi.fn(),
      setLineDash: vi.fn(),
      roundRect: vi.fn(),
      rect: vi.fn(),
    };
    HTMLCanvasElement.prototype.getContext = vi.fn().mockReturnValue(mockCtx);

    document.body.innerHTML = `
      <div id="app" class="desktop-shell">
        <header class="title-bar">
          <div class="app-branding">
            <span class="app-name">AetherOffice</span>
            <div class="quick-access-toolbar">
              <button id="btn-quick-open">📂</button>
              <button id="btn-quick-save">💾</button>
              <button id="btn-quick-print">🖨️</button>
            </div>
          </div>
          <div class="title-actions">
            <button id="btn-theme-toggle" class="theme-toggle-btn">Light</button>
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
              <span id="status-doc-info">Page 1 of 1</span>
              <span id="status-word-count">0 words</span>
            </footer>
          </div>
        </div>
      </div>
    `;
  });

  // =========================================================================
  // 1. 100% Zoom Top-Visible Layout & Zero Vertical Clipping
  // =========================================================================
  it("1. Renders Page 1, Header, and first line of text fully visible at 100% zoom without negative vertical offset", () => {
    const container = document.getElementById("viewport-container")!;
    const state = new WriterState();
    state.setZoom(1.0); // 100% zoom
    const canvas = new WriterCanvas(container, state);

    expect(container.classList.contains("word-scroll-viewport")).toBe(true);
    expect(container.classList.contains("word-canvas-scroll")).toBe(true);
    expect(container.style.display).toBe("block");
    expect(container.style.overflowY).toBe("auto");

    const pageSheet = container.querySelector<HTMLElement>(".word-page-sheet");
    expect(pageSheet).not.toBeNull();
    expect(pageSheet?.style.transformOrigin).toBe("top center");
    expect(pageSheet?.style.margin).toBe("0px auto 32px");

    // Type text into page
    canvas.typeText("Antigravity GEC-v5.0 Certified Document");
    const bodyEl = container.querySelector<HTMLElement>('[data-testid="word-page-body"]');
    expect(bodyEl).not.toBeNull();
    expect(bodyEl?.style.fontSize).toBe("11pt");
    expect(bodyEl?.style.color).toBe("#0f172a");
    expect(bodyEl?.style.lineHeight).toBe("1.5");
    expect(bodyEl?.style.minHeight).toBe("860px");

    // Header Zone is rendered at top with dashed line
    const headerEl = container.querySelector<HTMLElement>('[data-testid="word-page-header"]');
    expect(headerEl).not.toBeNull();
    expect(headerEl?.style.borderBottom).toContain("dashed");
    expect(headerEl?.style.top).toBe("0px");
  });

  // =========================================================================
  // 2. Scroll Stability & Zero Flicker / Jump-to-Top
  // =========================================================================
  it("2. Preserves scroll position across re-renders and focuses with preventScroll: true", () => {
    const container = document.getElementById("viewport-container")!;
    const state = new WriterState();
    const canvas = new WriterCanvas(container, state);

    // Add 3 pages to make document scrollable
    canvas.insertNewPage();
    canvas.insertNewPage();
    expect(state.doc.pages.length).toBe(3);

    // Simulate user scrolling down 450 pixels
    container.scrollTop = 450;
    expect(container.scrollTop).toBe(450);

    // Trigger re-render (e.g. typing or formatting)
    canvas.typeText("Section 3 Content");

    // ScrollTop is preserved
    expect(container.scrollTop).toBe(450);

    // Focus on canvas verifies preventScroll: true (does not reset scrollTop to 0)
    const focusSpy = vi.spyOn(HTMLCanvasElement.prototype, "focus");
    canvas.render();
    expect(focusSpy).toHaveBeenCalledWith({ preventScroll: true });
    expect(container.scrollTop).toBe(450);
  });

  // =========================================================================
  // 3. Visible, Editable Headers & Footers
  // =========================================================================
  it("3. Displays visible dashed Header and Footer zones with direct inline editing", () => {
    const container = document.getElementById("viewport-container")!;
    const state = new WriterState();
    const canvas = new WriterCanvas(container, state);
    expect(canvas).toBeDefined();

    // 3.1 Header Zone & Input
    const headerInput = container.querySelector<HTMLElement>('[data-testid="word-header-input"]');
    expect(headerInput).not.toBeNull();
    expect(headerInput?.isContentEditable).toBe(true);

    // Edit Header
    headerInput!.textContent = "Confidential Project Roadmap 2026";
    headerInput!.dispatchEvent(new Event("blur"));
    expect(state.doc.headerText).toBe("Confidential Project Roadmap 2026");

    // 3.2 Footer Zone & Input
    const footerInput = container.querySelector<HTMLElement>('[data-testid="word-footer-input"]');
    expect(footerInput).not.toBeNull();
    expect(footerInput?.isContentEditable).toBe(true);

    // Edit Footer
    footerInput!.textContent = "AetherOffice Enterprise Edition";
    footerInput!.dispatchEvent(new Event("blur"));
    expect(state.doc.footerText).toBe("AetherOffice Enterprise Edition");

    // Page number display
    const pageNum = container.querySelector(".word-page-footer .page-number-display");
    expect(pageNum?.textContent).toContain("Page 1 of 1");
  });

  // =========================================================================
  // 4. '+ Page' / 'Blank Page' Appends New Page & Focuses Body
  // =========================================================================
  it("4. Appends a new blank page when clicking + Page button and focuses new page body", () => {
    const container = document.getElementById("viewport-container")!;
    const state = new WriterState();
    new WriterCanvas(container, state);

    expect(state.doc.pages.length).toBe(1);

    const addBtn = container.querySelector<HTMLButtonElement>('[data-testid="btn-add-page"]');
    expect(addBtn).not.toBeNull();

    // Click + Page
    addBtn?.click();

    // Page count increased from 1 to 2
    expect(state.doc.pages.length).toBe(2);
    expect(state.doc.pages[1].pageNumber).toBe(2);

    const pages = container.querySelectorAll(".word-page");
    expect(pages.length).toBe(2);

    // Page 2 has its own header and footer with Page 2 of 2
    const footer2 = pages[1].querySelector(".word-page-footer .page-number-display");
    expect(footer2?.textContent).toBe("Page 2 of 2");

    // Page 2 has its own contenteditable body ready for input
    const body2 = pages[1].querySelector<HTMLElement>('[data-testid="word-page-body"]');
    expect(body2).not.toBeNull();
    expect(body2?.getAttribute("data-page-index")).toBe("1");
  });
});

