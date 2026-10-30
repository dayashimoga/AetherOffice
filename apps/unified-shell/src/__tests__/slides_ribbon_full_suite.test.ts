import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { SlidesState, slidesState } from "../state/slidesState";
import { SlideStageCanvas } from "../components/viewports/SlideStageCanvas";
import { SpeakerNotesPane } from "../components/SpeakerNotesPane";
import { PresentationPlayer } from "../components/PresentationPlayer";
import { SlidesRibbon } from "../components/ribbon/SlidesRibbon";
import { UnifiedShellApp } from "../main";

describe("Working PowerPoint Ribbon, Curated Themes, Transitions, Animations & Presentation Player (GEC-v3.0 Subphase 24.1.2)", () => {
  let state: SlidesState;
  let ribbon: SlidesRibbon;
  let canvasComponent: SlideStageCanvas;
  let notesPane: SpeakerNotesPane;
  let player: PresentationPlayer;
  let app: UnifiedShellApp;
  let viewportContainer: HTMLElement;
  let ribbonBody: HTMLElement;

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
      quadraticCurveTo: () => {},
      bezierCurveTo: () => {},
      createLinearGradient: () => ({
        addColorStop: () => {},
      }),
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
          <div id="viewport-container"></div>
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
        <div id="navigation-outline-container"></div>
        <div id="animation-pane-container"></div>
        <div id="calc-formula-bar-container"></div>
      </div>
    `;

    viewportContainer = document.getElementById("viewport-container")!;
    ribbonBody = document.createElement("div");
    ribbonBody.className = "ribbon-body";
    document.body.appendChild(ribbonBody);

    state = new SlidesState();
    state.resetToBlank("SlideShowTest.pptx");
    ribbon = new SlidesRibbon();
    canvasComponent = new SlideStageCanvas(viewportContainer, state);
    expect(canvasComponent).toBeDefined();
    notesPane = new SpeakerNotesPane(viewportContainer, state);
    player = new PresentationPlayer(state);

    app = new UnifiedShellApp(false);
    app.getTabBar().addTab({
      id: "tab-slides-1",
      docType: "PowerPoint",
      title: "Deck.pptx",
      isDirty: false,
    });
    app.getTabBar().switchTab("tab-slides-1");
    (app as any).currentDocType = "PowerPoint";
    slidesState.resetToBlank("Deck.pptx");
  });

  afterEach(() => {
    player.exitShow();
    document.body.innerHTML = "";
  });

  // ==========================================================================
  // 1. Home & Insert Tabs: Shapes, Text Boxes, Tables, Images & Formatting
  // ==========================================================================
  describe("1. Home & Insert Tabs: Working Tools & Shape Manipulations", () => {
    it("renders Home Tab and inserts all 9 preset shape kinds plus text box", () => {
      const actions: { action: string; payload?: unknown }[] = [];
      ribbon.renderTab("Home", ribbonBody, (a, p) => actions.push({ action: a, payload: p }));

      const shapeKinds = [
        "rect",
        "roundRect",
        "ellipse",
        "triangle",
        "arrow",
        "star",
        "callout",
        "diamond",
        "heart",
      ];

      shapeKinds.forEach((kind) => {
        state.execCommand("InsertShape", kind);
        const sel = state.getSelectedShape();
        expect(sel).toBeDefined();
        expect(sel?.type).toBe(kind);
      });

      // Insert Text Box
      state.execCommand("InsertTextBox");
      const textShape = state.getSelectedShape();
      expect(textShape?.type).toBe("text");
    });

    it("formats selected shape: Font Family, Size, Bold, Italic, Underline & Color", () => {
      state.addShape("text");
      const sel = state.getSelectedShape();
      expect(sel).toBeDefined();

      state.execCommand("SetFontFamily", "Georgia");
      state.execCommand("SetFontSize", 36);
      state.execCommand("ToggleBold");
      state.execCommand("ToggleItalic");
      state.execCommand("ToggleUnderline");
      state.execCommand("SetFontColor", "#e11d48");

      expect(sel?.fontFamily).toBe("Georgia");
      expect(sel?.fontSize).toBe(36);
      expect(sel?.bold).toBe(true);
      expect(sel?.italic).toBe(true);
      expect(sel?.underline).toBe(true);
      expect(sel?.textColor).toBe("#e11d48");
    });

    it("applies text and shape alignment (left, center, right, justify)", () => {
      state.addShape("rect");
      const sel = state.getSelectedShape();
      expect(sel).toBeDefined();

      state.execCommand("AlignLeft");
      expect(sel?.textAlign).toBe("left");
      expect(sel?.x).toBe(80);

      state.execCommand("AlignCenter");
      expect(sel?.textAlign).toBe("center");
      expect(sel?.x).toBe(Math.round((state.slideWidth - sel!.w) / 2));

      state.execCommand("AlignRight");
      expect(sel?.textAlign).toBe("right");
      expect(sel?.x).toBe(state.slideWidth - 80 - sel!.w);

      state.execCommand("AlignJustify");
      expect(sel?.textAlign).toBe("justify");
    });

    it("modifies shape fill color, outline stroke & outline weight", () => {
      state.addShape("roundRect");
      const sel = state.getSelectedShape();
      expect(sel).toBeDefined();

      state.execCommand("SetFillColor", "#10b981");
      expect(sel?.fill).toBe("#10b981");

      state.execCommand("SetShapeOutline", "#059669");
      expect(sel?.stroke).toBe("#059669");

      state.execCommand("SetOutlineWeight", 4);
      expect(sel?.strokeWidth).toBe(4);
    });

    it("arranges shapes: BringToFront, SendToBack, Duplicate & Delete", () => {
      const sh1 = state.addShape("rect");
      const sh2 = state.addShape("ellipse");
      expect(sh2).toBeDefined();
      const slide = state.getCurrentSlide();
      expect(slide.shapes.length).toBeGreaterThanOrEqual(2);

      state.selectShape(sh1.id);
      state.execCommand("BringToFront");
      expect(slide.shapes[slide.shapes.length - 1].id).toBe(sh1.id);

      state.execCommand("SendToBack");
      expect(slide.shapes[0].id).toBe(sh1.id);

      const countBefore = slide.shapes.length;
      state.execCommand("DuplicateShape");
      expect(slide.shapes.length).toBe(countBefore + 1);

      state.execCommand("DeleteShape");
      expect(slide.shapes.length).toBe(countBefore);
    });

    it("renders Insert Tab and inserts tables, pictures, audio and video", () => {
      const actions: { action: string; payload?: unknown }[] = [];
      ribbon.renderTab("Insert", ribbonBody, (a, p) => actions.push({ action: a, payload: p }));

      // Insert Table
      state.execCommand("InsertTable", { rows: 4, cols: 4 });
      const tblShape = state.getSelectedShape();
      expect(tblShape?.type).toBe("table");
      expect(tblShape?.tableHeaders?.length).toBe(4);
      expect(tblShape?.tableData?.length).toBe(4);

      // Insert Picture with custom dimensions
      state.execCommand("InsertPicture", {
        url: "branding_logo.png",
        width: 320,
        height: 200,
      });
      const imgShape = state.getSelectedShape();
      expect(imgShape?.type).toBe("image");
      expect(imgShape?.imageUrl).toBe("branding_logo.png");
      expect(imgShape?.w).toBe(320);

      // Insert Media
      state.execCommand("InsertVideo", "demo_clip.mp4");
      const vidShape = state.getSelectedShape();
      expect(vidShape?.mediaType).toBe("video");
      expect(vidShape?.mediaUrl).toBe("demo_clip.mp4");

      state.execCommand("InsertAudio", "background_tune.mp3");
      const audShape = state.getSelectedShape();
      expect(audShape?.mediaType).toBe("audio");
      expect(audShape?.mediaUrl).toBe("background_tune.mp3");
    });
  });

  // ==========================================================================
  // 2. Design Tab: 8 Curated Slide Themes & Backgrounds
  // ==========================================================================
  describe("2. Design Tab: 8 Curated Slide Themes & Custom Backgrounds", () => {
    const curatedThemes = [
      { name: "Dark Slate", expectedTheme: "DarkSlate", expectedBg: "#0f172a" },
      { name: "Ocean Blue", expectedTheme: "OceanBlue", expectedBg: "#f0f9ff" },
      { name: "Emerald", expectedTheme: "Emerald", expectedBg: "#f0fdf4" },
      { name: "Sunset", expectedTheme: "Sunset", expectedBg: "#fdf2f8" },
      { name: "Minimal White", expectedTheme: "MinimalWhite", expectedBg: "#ffffff" },
      { name: "Executive Navy", expectedTheme: "ExecutiveNavy", expectedBg: "#0a1128" },
      { name: "Warm Sand", expectedTheme: "WarmSand", expectedBg: "#fef3c7" },
      { name: "Cyber Neon", expectedTheme: "CyberNeon", expectedBg: "#000000" },
    ];

    curatedThemes.forEach(({ name, expectedTheme, expectedBg }) => {
      it(`applies curated theme '${name}' correctly across all slides`, () => {
        state.execCommand("SetTheme", name);
        expect(state.currentTheme).toBe(expectedTheme);
        const slide = state.getCurrentSlide();
        expect(slide.background).toBe(expectedBg);
      });
    });

    it("renders Design Tab buttons for all 8 curated themes and background options", () => {
      const actions: { action: string; payload?: unknown }[] = [];
      ribbon.renderTab("Design", ribbonBody, (a, p) => actions.push({ action: a, payload: p }));

      const themeButtons = ribbonBody.querySelectorAll(".ribbon-group:first-child .ribbon-btn");
      expect(themeButtons.length).toBe(8);

      // Click Dark Slate theme button
      const darkSlateBtn = ribbonBody.querySelector('[data-testid="slides-theme-dark-slate"]') as HTMLButtonElement;
      expect(darkSlateBtn).not.toBeNull();
      darkSlateBtn.click();
      expect(actions).toContainEqual({ action: "SetTheme", payload: "Dark Slate" });

      // Click Solid Fill button
      const solidFillBtn = ribbonBody.querySelector('[data-testid="slides-btn-solid-bg"]') as HTMLButtonElement;
      expect(solidFillBtn).not.toBeNull();
      solidFillBtn.click();
      expect(actions).toContainEqual({ action: "SetSlideBackground", payload: "#0f172a" });

      // Click Gradient Fill button
      const gradientBtn = ribbonBody.querySelector('[data-testid="slides-btn-gradient-bg"]') as HTMLButtonElement;
      expect(gradientBtn).not.toBeNull();
      gradientBtn.click();
      expect(actions.some((a) => a.action === "SetSlideBackground" && String(a.payload).includes("gradient"))).toBe(true);
    });

    it("toggles slide aspect ratio between 16:9 (960x540) and 4:3 (720x540)", () => {
      state.execCommand("SetSlideSize", "4:3");
      expect(state.slideAspectRatio).toBe("4:3");
      expect(state.slideWidth).toBe(720);
      expect(state.slideHeight).toBe(540);

      state.execCommand("SetSlideSize", "16:9");
      expect(state.slideAspectRatio).toBe("16:9");
      expect(state.slideWidth).toBe(960);
      expect(state.slideHeight).toBe(540);
    });
  });

  // ==========================================================================
  // 3. Transitions & Animations Tabs: Live Selector & Preview
  // ==========================================================================
  describe("3. Transitions & Animations Tabs: Live Selector & Preview", () => {
    it("configures slide transitions: None, Fade, Slide Left, Zoom, Flip", () => {
      const transitions = ["None", "Fade", "Slide Left", "Zoom", "Flip"];
      transitions.forEach((trans) => {
        state.execCommand("SetTransition", trans);
        const curSlide = state.getCurrentSlide();
        expect(curSlide.transition).toBeDefined();
      });

      // Direction and Timing
      state.execCommand("SetEffectOption", "left");
      expect(state.getCurrentSlide().transitionConfig?.direction).toBe("left");

      state.execCommand("SetTransitionDuration", 1.5);
      expect(state.getCurrentSlide().transitionConfig?.duration).toBe(1.5);
    });

    it("renders Transitions Tab and triggers PreviewTransition", () => {
      const actions: { action: string; payload?: unknown }[] = [];
      ribbon.renderTab("Transitions", ribbonBody, (a, p) => actions.push({ action: a, payload: p }));

      const prevBtn = ribbonBody.querySelector('[data-testid="slides-btn-preview-trans"]') as HTMLButtonElement;
      expect(prevBtn).not.toBeNull();
      prevBtn.click();
      expect(actions).toContainEqual({ action: "PreviewTransition", payload: undefined });

      // Check transition selector buttons exist
      const slideLeftBtn = ribbonBody.querySelector('[data-testid="slides-trans-slide-left"]') as HTMLButtonElement;
      expect(slideLeftBtn).not.toBeNull();
      slideLeftBtn.click();
      expect(actions).toContainEqual({ action: "SetTransition", payload: "Slide Left" });
    });

    it("configures object animations: None, Fade In, Fly Up, Pop, Spin", () => {
      const shape = state.addShape("star");
      state.selectShape(shape.id);

      const animEffects = ["fadeIn", "flyUp", "pop", "spin"];
      animEffects.forEach((eff) => {
        state.execCommand("SetAnimation", eff);
        const anims = state.getSlideAnimations();
        expect(anims.length).toBeGreaterThanOrEqual(1);
      });

      // Removing animation with 'none'
      state.execCommand("SetAnimation", "none");
      const clearedAnims = state.getSlideAnimations().filter((a) => a.shapeId === shape.id);
      expect(clearedAnims.length).toBe(0);
    });

    it("renders Animations Tab and triggers PreviewAnimation", () => {
      const actions: { action: string; payload?: unknown }[] = [];
      ribbon.renderTab("Animations", ribbonBody, (a, p) => actions.push({ action: a, payload: p }));

      const prevAnimBtn = ribbonBody.querySelector('[data-testid="slides-btn-preview-anim"]') as HTMLButtonElement;
      expect(prevAnimBtn).not.toBeNull();
      prevAnimBtn.click();
      expect(actions).toContainEqual({ action: "PreviewAnimation", payload: undefined });

      const flyUpBtn = ribbonBody.querySelector('[data-testid="slides-anim-fly-up"]') as HTMLButtonElement;
      expect(flyUpBtn).not.toBeNull();
      flyUpBtn.click();
      expect(actions).toContainEqual({ action: "SetAnimation", payload: "flyUp" });
    });
  });

  // ==========================================================================
  // 4. Slide Show Tab & Fullscreen Presentation Player
  // ==========================================================================
  describe("4. Slide Show Tab & Fullscreen Presentation Player (F5 / Shift+F5)", () => {
    beforeEach(() => {
      // Setup 3 slides
      while (state.slides.length < 3) {
        state.addSlide(`Slide ${state.slides.length + 1}`);
      }
      state.selectSlide(1);
    });

    it("starts presentation show from beginning (F5) and from current slide (Shift+F5)", () => {
      player.startShow(true, false);
      expect(player.isOpen()).toBe(true);
      expect(player.isPresenter()).toBe(false);
      expect(state.currentSlideIndex).toBe(0);

      player.exitShow();
      expect(player.isOpen()).toBe(false);

      state.selectSlide(2);
      player.startShow(false, false);
      expect(player.isOpen()).toBe(true);
      expect(state.currentSlideIndex).toBe(2);
    });

    it("navigates forward and backward via player buttons and updates slide counter", () => {
      player.startShow(true, false);
      expect(state.currentSlideIndex).toBe(0);

      const nextBtn = player.getRoot().querySelector('[data-testid="pres-btn-next"]') as HTMLButtonElement;
      expect(nextBtn).not.toBeNull();
      nextBtn.click();
      expect(state.currentSlideIndex).toBe(1);

      const counter = player.getRoot().querySelector(".pres-slide-counter");
      expect(counter?.textContent).toContain("Slide 2 of 3");

      const prevBtn = player.getRoot().querySelector('[data-testid="pres-btn-prev"]') as HTMLButtonElement;
      expect(prevBtn).not.toBeNull();
      prevBtn.click();
      expect(state.currentSlideIndex).toBe(0);
    });

    it("toggles laser pointer dot and screen blackout/whiteout controls", () => {
      player.startShow(true, false);
      const laserBtn = player.getRoot().querySelector('[data-testid="pres-btn-laser"]') as HTMLButtonElement;
      expect(laserBtn).not.toBeNull();
      laserBtn.click();

      const laserDot = player.getRoot().querySelector(".pres-laser-dot") as HTMLElement;
      expect(laserDot.style.display).toBe("block");

      // Blackout toggle
      const blackBtn = player.getRoot().querySelector('[data-testid="pres-btn-black"]') as HTMLButtonElement;
      blackBtn.click();
      // Forward steps restore screen
      player.stepForward();

      // Whiteout toggle
      const whiteBtn = player.getRoot().querySelector('[data-testid="pres-btn-white"]') as HTMLButtonElement;
      whiteBtn.click();
      player.stepBackward();
    });

    it("handles keyboard events: ArrowRight, Space, ArrowLeft, and Escape to exit", () => {
      player.startShow(true, false);
      expect(state.currentSlideIndex).toBe(0);

      // Space advances
      window.dispatchEvent(new KeyboardEvent("keydown", { key: " " }));
      expect(state.currentSlideIndex).toBe(1);

      // ArrowRight advances
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowRight" }));
      expect(state.currentSlideIndex).toBe(2);

      // ArrowLeft steps backward
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowLeft" }));
      expect(state.currentSlideIndex).toBe(1);

      // Escape exits presentation
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
      expect(player.isOpen()).toBe(false);
    });
  });

  // ==========================================================================
  // 5. Speaker Notes Bar: Collapse, Input Sync & State Persistence
  // ==========================================================================
  describe("5. Speaker Notes Bar: Collapsible Panel & Live Synchronization", () => {
    it("renders collapsible Speaker Notes bar with header toggle", () => {
      const root = notesPane.getRoot();
      expect(root).not.toBeNull();
      expect(notesPane.isCollapsed()).toBe(false);

      const header = root.querySelector('[data-testid="speaker-notes-header"]') as HTMLElement;
      expect(header).not.toBeNull();

      // Click to collapse
      header.click();
      expect(notesPane.isCollapsed()).toBe(true);

      // Click to expand
      header.click();
      expect(notesPane.isCollapsed()).toBe(false);
    });

    it("synchronizes typed speaker notes directly into slidesState", () => {
      let isDirty = false;
      notesPane.setOnDirty(() => {
        isDirty = true;
      });

      const textarea = notesPane.getTextarea();
      textarea.value = "Slide presenter talking point: outline system architecture components.";
      textarea.dispatchEvent(new Event("input"));

      expect(isDirty).toBe(true);
      expect(state.getSlideNotes()).toBe("Slide presenter talking point: outline system architecture components.");

      // Switching slides refreshes textarea
      state.addSlide("Second Slide");
      state.selectSlide(1);
      notesPane.refresh();
      expect(textarea.value).toBe("");
    });
  });

  // ==========================================================================
  // 6. UnifiedShellApp PowerPoint Mode Ribbon Integration
  // ==========================================================================
  describe("6. UnifiedShellApp Ribbon Action Dispatching in PowerPoint Mode", () => {
    it("dispatches StartShowBeginning and StartShowCurrent through handleRibbonAction", () => {
      app.handleRibbonAction("StartShowBeginning");
      const activePlayer = (app as any).presentationPlayer as PresentationPlayer;
      expect(activePlayer).toBeDefined();
      expect(activePlayer.isOpen()).toBe(true);
      activePlayer.exitShow();

      app.handleRibbonAction("StartShowCurrent");
      expect(activePlayer.isOpen()).toBe(true);
      activePlayer.exitShow();
    });

    it("dispatches SetTheme and changes PowerPoint presentation theme via app action", () => {
      app.handleRibbonAction("SetTheme", "Cyber Neon");
      expect(slidesState.currentTheme).toBe("CyberNeon");
      expect(slidesState.getCurrentSlide().background).toBe("#000000");

      app.handleRibbonAction("SetTheme", "Ocean Blue");
      expect(slidesState.currentTheme).toBe("OceanBlue");
    });
  });
});
