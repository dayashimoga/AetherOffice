import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { SlidesState } from "../state/slidesState";
import { PresenterConsoleWindow } from "../components/PresenterConsoleWindow";
import { UnifiedShellApp } from "../main";

describe("Presenter Studio Dual-Screen Mode & Audience Annotation Laser (Subphase 19.1.3)", () => {
  let state: SlidesState;
  let presenterConsole: PresenterConsoleWindow;
  let audienceContainer: HTMLElement;

  beforeEach(() => {
    const mockCtx = {
      fillRect: () => {},
      strokeRect: () => {},
      fillText: () => {},
      clearRect: () => {},
      beginPath: () => {},
      closePath: () => {},
      moveTo: () => {},
      lineTo: () => {},
      stroke: () => {},
      fill: () => {},
      save: () => {},
      restore: () => {},
      clip: () => {},
      scale: () => {},
      setTransform: () => {},
      translate: () => {},
      rotate: () => {},
      arc: () => {},
      roundRect: () => {},
      ellipse: () => {},
      measureText: (txt: string) => ({ width: txt.length * 8 }),
      setLineDash: () => {},
      font: "24px Aptos, sans-serif",
      fillStyle: "#000",
      strokeStyle: "#000",
      lineWidth: 1,
      textAlign: "left",
      textBaseline: "top",
    };
    HTMLCanvasElement.prototype.getContext = (() => mockCtx) as any;

    document.body.innerHTML = `
      <div id="app">
        <div id="titlebar"></div>
        <div id="tab-bar-container"></div>
        <div id="ribbon-container"></div>
        <div id="sub-header-container"></div>
        <div id="main-content">
          <div id="sidebar-container"></div>
          <div id="viewport-container" style="position: relative; width: 1000px; height: 600px;"></div>
        </div>
        <div id="status-bar">
          <span id="status-doc-info"></span>
          <span id="status-calc-stats"></span>
          <span id="status-word-count"></span>
          <input id="zoom-slider" type="range" value="100" />
          <span id="status-zoom-level">100%</span>
        </div>
        <div id="backstage-container"></div>
        <div id="ink-layer-container"></div>
        <div id="reflow-reader-container"></div>
        <div id="row-card-inspector-container"></div>
        <div id="laser-pointer-dot"></div>
        <div id="animation-pane-container"></div>
        <div id="audience-screen-container" style="position: relative; width: 1920px; height: 1080px;"></div>
      </div>
    `;

    state = new SlidesState();
    while (state.slides.length < 3) {
      state.addSlide();
    }
    state.selectSlide(0);
    state.selectSlide(0);
    state.slides[0].notes = "Welcome to the presentation!\nKey point 1: High performance Rust.";
    state.slides[1].notes = "Architecture breakdown and benchmarks.";

    audienceContainer = document.getElementById("audience-screen-container")!;
    presenterConsole = new PresenterConsoleWindow("app", state, audienceContainer);
  });

  afterEach(() => {
    presenterConsole.close();
  });

  it("1. initializes PresenterConsoleWindow with dual-screen layout and UI panels", () => {
    expect(presenterConsole.isOpen()).toBe(false);
    presenterConsole.open();
    expect(presenterConsole.isOpen()).toBe(true);

    const root = document.querySelector(".presenter-console-window") as HTMLElement;
    expect(root).not.toBeNull();
    expect(root.style.display).toBe("flex");

    expect(root.querySelector(".presenter-header")).not.toBeNull();
    expect(root.querySelector(".presenter-timer-display")).not.toBeNull();
    expect(root.querySelector(".presenter-current-canvas")).not.toBeNull();
    expect(root.querySelector(".presenter-next-canvas")).not.toBeNull();
    expect(root.querySelector(".presenter-notes-content")).not.toBeNull();
    expect(root.querySelector(".presenter-filmstrip")).not.toBeNull();
    expect(root.querySelector(".presenter-annotation-toolbar")).not.toBeNull();
  });

  it("2. displays formatted speaker notes and supports teleprompter font resizing", () => {
    presenterConsole.open();
    const notesContent = document.querySelector(".presenter-notes-content") as HTMLElement;
    expect(notesContent.innerHTML).toContain("Welcome to the presentation!");
    expect(notesContent.innerHTML).toContain("Key point 1");

    const fontBefore = parseFloat(notesContent.style.fontSize);
    const incBtn = Array.from(document.querySelectorAll("button")).find(
      (b) => b.textContent === "A+"
    );
    expect(incBtn).toBeDefined();
    incBtn?.click();

    const fontAfter = parseFloat(notesContent.style.fontSize);
    expect(fontAfter).toBeGreaterThan(fontBefore);
  });

  it("3. executes slide navigation between current and next slides", () => {
    presenterConsole.open();
    expect(state.currentSlideIndex).toBe(0);

    presenterConsole.nextSlide();
    expect(state.currentSlideIndex).toBe(1);

    presenterConsole.prevSlide();
    expect(state.currentSlideIndex).toBe(0);

    presenterConsole.jumpToSlide(2);
    expect(state.currentSlideIndex).toBe(2);
  });

  it("4. switches annotation tools: arrow, laser, pen, and eraser", () => {
    presenterConsole.open();
    expect(presenterConsole.getActiveTool()).toBe("arrow");

    presenterConsole.setTool("laser");
    expect(presenterConsole.getActiveTool()).toBe("laser");

    presenterConsole.setTool("pen");
    expect(presenterConsole.getActiveTool()).toBe("pen");

    presenterConsole.setTool("eraser");
    expect(presenterConsole.getActiveTool()).toBe("eraser");
  });

  it("5. toggles black screen and white screen modes", () => {
    presenterConsole.open();
    expect(presenterConsole.isBlackScreenActive()).toBe(false);
    expect(presenterConsole.isWhiteScreenActive()).toBe(false);

    presenterConsole.toggleBlackScreen();
    expect(presenterConsole.isBlackScreenActive()).toBe(true);
    expect(presenterConsole.isWhiteScreenActive()).toBe(false);

    presenterConsole.toggleWhiteScreen();
    expect(presenterConsole.isBlackScreenActive()).toBe(false);
    expect(presenterConsole.isWhiteScreenActive()).toBe(true);

    presenterConsole.toggleWhiteScreen();
    expect(presenterConsole.isWhiteScreenActive()).toBe(false);
  });

  it("6. tracks presentation timer and updates timer state", () => {
    presenterConsole.open();
    const initialSecs = presenterConsole.getTimerSeconds();
    expect(initialSecs).toBe(0);

    const timerDisp = document.querySelector(".presenter-timer-display");
    expect(timerDisp?.textContent).toContain("00:00:00");
  });

  it("7. synchronizes laser pointer dot position with audience display", () => {
    presenterConsole.open();
    presenterConsole.setTool("laser");

    const audienceDot = audienceContainer.querySelector(".audience-laser-dot") as HTMLElement;
    expect(audienceDot).not.toBeNull();
  });

  it("8. dispatches keyboard shortcuts: B, W, P, L, E, and Escape", () => {
    presenterConsole.open();

    // 'B' for black screen
    window.dispatchEvent(new KeyboardEvent("keydown", { key: "b" }));
    expect(presenterConsole.isBlackScreenActive()).toBe(true);

    // 'W' for white screen
    window.dispatchEvent(new KeyboardEvent("keydown", { key: "w" }));
    expect(presenterConsole.isWhiteScreenActive()).toBe(true);

    // 'P' for pen
    window.dispatchEvent(new KeyboardEvent("keydown", { key: "p" }));
    expect(presenterConsole.getActiveTool()).toBe("pen");

    // 'L' for laser
    window.dispatchEvent(new KeyboardEvent("keydown", { key: "l" }));
    expect(presenterConsole.getActiveTool()).toBe("laser");

    // 'ArrowRight' advances slide
    window.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowRight" }));
    expect(state.currentSlideIndex).toBe(1);

    // 'Escape' closes console
    window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    expect(presenterConsole.isOpen()).toBe(false);
  });

  it("9. integrates PresenterConsoleWindow with UnifiedShellApp in PowerPoint mode", () => {
    const app = new UnifiedShellApp(false);
    const tabBar = app.getTabBar();
    tabBar.addTab({
      id: "tab-ppt-studio",
      title: "Conference.pptx",
      docType: "PowerPoint",
      isDirty: false,
    });

    const consoleInstance = app.getPresenterConsole();
    expect(consoleInstance).toBeDefined();

    // Ribbon action launches dual-screen studio
    app.handleRibbonAction("StartDualScreenPresenter");
    expect(consoleInstance?.isOpen()).toBe(true);

    consoleInstance?.close();
    expect(consoleInstance?.isOpen()).toBe(false);
  });
});
