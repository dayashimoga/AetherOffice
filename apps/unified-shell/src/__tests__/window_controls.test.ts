import { describe, it, expect, beforeEach, vi } from "vitest";
import { UnifiedShellApp } from "../main";
import { writerState } from "../state/writerState";
import { slidesState } from "../state/slidesState";

describe("Native Window Controls & Zero Fake Text (GEC-v3.0 Subphase 21.1.1)", () => {
  beforeEach(() => {
    document.body.innerHTML = `
      <div id="app">
        <header class="title-bar">
          <div class="window-controls">
            <button class="win-btn win-min" title="Minimize">−</button>
            <button class="win-btn win-max" title="Maximize">□</button>
            <button class="win-btn win-close" title="Close">×</button>
          </div>
        </header>
        <div id="tab-bar-container"></div>
        <div id="ribbon-container"></div>
        <div id="sub-header-container"></div>
        <div id="sidebar-container"></div>
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

  it("wires minimize, maximize, and close buttons on DOM init with window.close fallback", async () => {
    const app = new UnifiedShellApp(false);
    expect(app).toBeDefined();

    const minBtn = document.querySelector<HTMLButtonElement>(".win-min");
    const maxBtn = document.querySelector<HTMLButtonElement>(".win-max");
    const closeBtn = document.querySelector<HTMLButtonElement>(".win-close");

    expect(minBtn).not.toBeNull();
    expect(maxBtn).not.toBeNull();
    expect(closeBtn).not.toBeNull();

    // Verify close fallback
    const windowCloseSpy = vi.spyOn(window, "close").mockImplementation(() => {});
    closeBtn?.click();

    await new Promise((resolve) => setTimeout(resolve, 10));
    expect(windowCloseSpy).toHaveBeenCalled();
    windowCloseSpy.mockRestore();
  });

  it("wires minimize and toggleMaximize when running under Tauri runtime", async () => {
    const mockMinimize = vi.fn().mockResolvedValue(undefined);
    const mockToggleMaximize = vi.fn().mockResolvedValue(undefined);
    const mockClose = vi.fn().mockResolvedValue(undefined);

    (window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = {
      metadata: {},
    };

    // Mock Tauri window API module
    vi.doMock("@tauri-apps/api/window", () => ({
      getCurrentWindow: () => ({
        minimize: mockMinimize,
        toggleMaximize: mockToggleMaximize,
        close: mockClose,
      }),
    }));

    const app = new UnifiedShellApp(false);
    expect(app).toBeDefined();

    const minBtn = document.querySelector<HTMLButtonElement>(".win-min");
    const maxBtn = document.querySelector<HTMLButtonElement>(".win-max");
    const closeBtn = document.querySelector<HTMLButtonElement>(".win-close");

    minBtn?.click();
    for (let i = 0; i < 20; i++) {
      if (mockMinimize.mock.calls.length > 0) break;
      await new Promise((r) => setTimeout(r, 20));
    }

    maxBtn?.click();
    for (let i = 0; i < 20; i++) {
      if (mockToggleMaximize.mock.calls.length > 0) break;
      await new Promise((r) => setTimeout(r, 20));
    }

    closeBtn?.click();
    for (let i = 0; i < 20; i++) {
      if (mockClose.mock.calls.length > 0) break;
      await new Promise((r) => setTimeout(r, 20));
    }

    expect(mockMinimize).toHaveBeenCalled();
    expect(mockToggleMaximize).toHaveBeenCalled();
    expect(mockClose).toHaveBeenCalled();

    delete (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    vi.doUnmock("@tauri-apps/api/window");
  });

  it("verifies zero banned fake demo text exists in writerState or slidesState", () => {
    const bannedPatterns = [
      "Executive Technical Summary",
      "Performance SLAs & Benchmark Compliance",
      "Live Demo & Architecture Validation",
      "Universal Adaptive Form-Factor Parity",
    ];

    const writerJson = JSON.stringify(writerState.doc);
    for (const pat of bannedPatterns) {
      expect(writerJson.includes(pat)).toBe(false);
    }

    const slidesJson = JSON.stringify(slidesState.slides);
    for (const pat of bannedPatterns) {
      expect(slidesJson.includes(pat)).toBe(false);
    }
  });
});
