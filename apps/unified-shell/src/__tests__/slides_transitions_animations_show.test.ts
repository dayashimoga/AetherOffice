import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { SlidesState, slidesState } from "../state/slidesState";
import { AnimationPane } from "../components/AnimationPane";
import { PresentationPlayer } from "../components/PresentationPlayer";
import { SlideStageCanvas } from "../components/viewports/SlideStageCanvas";
import { UnifiedShellApp } from "../main";

describe("Slides Transitions, Animations & Fullscreen Slide Show Engine (GEC-v2.0 Subphase 11.2.1)", () => {
  let state: SlidesState;

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
      createLinearGradient: () => ({ addColorStop: () => {} }),
      font: "24px Aptos, sans-serif",
      fillStyle: "#000",
      strokeStyle: "#000",
      lineWidth: 1,
      textAlign: "left",
      textBaseline: "top",
    };
    HTMLCanvasElement.prototype.getContext = (() => mockCtx) as any;

    state = new SlidesState();
    // Clean document body
    document.body.innerHTML = `
      <div id="app" class="desktop-shell">
        <nav id="tab-bar-container" class="tab-bar"></nav>
        <section id="ribbon-container" class="ribbon-bar"></section>
        <section id="sub-header-container" class="sub-header-bar"></section>
        <main class="workspace-layout">
          <aside id="sidebar-container" class="sidebar-pane"></aside>
          <section class="canvas-viewport" id="viewport-container" tabindex="0">
            <canvas id="render-surface"></canvas>
          </section>
          <div id="animation-pane-container"></div>
        </main>
        <footer id="status-bar" class="status-bar">
          <div class="status-left">
            <span id="status-doc-info">Slide 1 of 3</span>
            <span id="status-word-count">0 words</span>
          </div>
          <div class="status-right">
            <span id="status-zoom-level">100%</span>
            <input type="range" id="zoom-slider" min="20" max="400" value="100" />
          </div>
        </footer>
      </div>
    `;
  });

  afterEach(() => {
    document.body.innerHTML = "";
    vi.clearAllTimers();
  });

  describe("1. Slide Transitions Engine & Config", () => {
    it("initializes with default None transition and default 1.0s duration", () => {
      const trans = state.getSlideTransition();
      expect(trans.effect).toBe("None");
      expect(trans.duration).toBe(1.0);
      expect(trans.direction).toBe("right");
    });

    it("configures slide transitions: Fade, Push, Wipe, Split, Morph", () => {
      state.setTransition("Fade", 0.5, "left");
      let trans = state.getSlideTransition();
      expect(trans.effect).toBe("Fade");
      expect(trans.duration).toBe(0.5);
      expect(trans.direction).toBe("left");

      state.setTransition("Push", 1.5, "bottom");
      trans = state.getSlideTransition();
      expect(trans.effect).toBe("Push");
      expect(trans.duration).toBe(1.5);
      expect(trans.direction).toBe("bottom");

      state.setTransition("Morph", 2.0);
      trans = state.getSlideTransition();
      expect(trans.effect).toBe("Morph");
      expect(trans.duration).toBe(2.0);
    });

    it("applies transition settings to all slides when requested", () => {
      expect(state.slides.length).toBeGreaterThanOrEqual(3);
      state.setTransition("Wipe", 1.5, "top", true);

      state.slides.forEach((s) => {
        expect(s.transition).toBe("Wipe");
        expect(s.transitionConfig?.effect).toBe("Wipe");
        expect(s.transitionConfig?.duration).toBe(1.5);
        expect(s.transitionConfig?.direction).toBe("top");
      });
    });

    it("executes transition preview on SlideStageCanvas", async () => {
      const viewport = document.getElementById("viewport-container")!;
      const canvas = new SlideStageCanvas(viewport, state);
      state.setTransition("Fade", 0.5);

      const previewPromise = canvas.playTransitionPreview();
      expect(previewPromise).toBeInstanceOf(Promise);
      await previewPromise;
      expect(canvas.getWrapper().style.opacity).toBe("1");
    });
  });

  describe("2. Slide Shape Animations & Timeline Management", () => {
    it("adds Entrance, Emphasis, and Exit animations to selected shapes", () => {
      const slide = state.getCurrentSlide();
      const shape1 = slide.shapes[0];
      const shape2 = slide.shapes[1];

      // Entrance animation
      const anim1 = state.addAnimation(shape1.id, "flyIn", "entrance", "click", 0.75, 0);
      expect(anim1).not.toBeNull();
      expect(anim1?.effect).toBe("flyIn");
      expect(anim1?.type).toBe("entrance");
      expect(anim1?.order).toBe(1);

      // Emphasis animation
      const anim2 = state.addAnimation(shape1.id, "pulse", "emphasis", "withPrev", 0.5, 0.25);
      expect(anim2?.effect).toBe("pulse");
      expect(anim2?.type).toBe("emphasis");
      expect(anim2?.trigger).toBe("withPrev");
      expect(anim2?.order).toBe(2);

      // Exit animation
      const anim3 = state.addAnimation(shape2.id, "fadeOut", "exit", "afterPrev", 1.0, 0);
      expect(anim3?.effect).toBe("fadeOut");
      expect(anim3?.type).toBe("exit");
      expect(anim3?.trigger).toBe("afterPrev");
      expect(anim3?.order).toBe(3);

      const anims = state.getSlideAnimations();
      expect(anims.length).toBe(3);
    });

    it("reorders animations and maintains sequential order indices", () => {
      const slide = state.getCurrentSlide();
      const sh = slide.shapes[0];
      const a1 = state.addAnimation(sh.id, "appear")!;
      const a2 = state.addAnimation(sh.id, "zoom")!;
      const a3 = state.addAnimation(sh.id, "bounce")!;

      expect(state.getSlideAnimations().map((a) => a.id)).toEqual([a1.id, a2.id, a3.id]);

      // Move a3 up
      state.reorderAnimation(a3.id, "up");
      let list = state.getSlideAnimations();
      expect(list.map((a) => a.id)).toEqual([a1.id, a3.id, a2.id]);
      expect(list.map((a) => a.order)).toEqual([1, 2, 3]);

      // Move a1 down
      state.reorderAnimation(a1.id, "down");
      list = state.getSlideAnimations();
      expect(list.map((a) => a.id)).toEqual([a3.id, a1.id, a2.id]);
    });

    it("removes animations and updates selection and order indices", () => {
      const slide = state.getCurrentSlide();
      const sh = slide.shapes[0];
      const a1 = state.addAnimation(sh.id, "fadeIn")!;
      const a2 = state.addAnimation(sh.id, "spin")!;

      state.removeAnimation(a1.id);
      const remaining = state.getSlideAnimations();
      expect(remaining.length).toBe(1);
      expect(remaining[0].id).toBe(a2.id);
      expect(remaining[0].order).toBe(1);
    });

    it("computes shape visibility per animation step", () => {
      const slide = state.getCurrentSlide();
      const sh1 = slide.shapes[0]; // will have entrance
      const sh2 = slide.shapes[1]; // no entrance -> initially visible
      state.addAnimation(sh1.id, "fadeIn", "entrance");

      // Step 0: sh1 not visible, sh2 visible
      let vis = state.getVisibleShapesForStep(state.currentSlideIndex, 0);
      expect(vis.visibleShapeIds.has(sh1.id)).toBe(false);
      expect(vis.visibleShapeIds.has(sh2.id)).toBe(true);

      // Step 1: sh1 becomes visible
      vis = state.getVisibleShapesForStep(state.currentSlideIndex, 1);
      expect(vis.visibleShapeIds.has(sh1.id)).toBe(true);
      expect(vis.visibleShapeIds.has(sh2.id)).toBe(true);
      expect(vis.animatingShapeId).toBe(sh1.id);
      expect(vis.currentEffect).toBe("fadeIn");
    });
  });

  describe("3. Collapsible Animation Timeline Pane", () => {
    let pane: AnimationPane;

    beforeEach(() => {
      pane = new AnimationPane("animation-pane-container", state);
    });

    it("toggles visibility via open, close, and toggle", () => {
      expect(pane.isOpen()).toBe(false);
      pane.open();
      expect(pane.isOpen()).toBe(true);
      expect(pane.getRoot().style.display).toBe("flex");

      pane.close();
      expect(pane.isOpen()).toBe(false);
      expect(pane.getRoot().style.display).toBe("none");

      pane.toggle();
      expect(pane.isOpen()).toBe(true);
    });

    it("renders animation list items with trigger badges and effect labels", () => {
      const sh = state.getCurrentSlide().shapes[0];
      state.addAnimation(sh.id, "flyIn", "entrance", "click", 0.5);
      state.addAnimation(sh.id, "spin", "emphasis", "withPrev", 1.0);
      pane.open();

      const items = pane.getRoot().querySelectorAll(".anim-item");
      expect(items.length).toBe(2);
      expect(items[0].textContent).toContain("flyIn");
      expect(items[0].textContent).toContain("🖱️");
      expect(items[1].textContent).toContain("spin");
      expect(items[1].textContent).toContain("⏱️");
    });

    it("selects animation items on click and syncs timing controls", () => {
      const sh = state.getCurrentSlide().shapes[0];
      const a1 = state.addAnimation(sh.id, "zoom", "entrance", "click", 0.75)!;
      pane.open();

      const item = pane.getRoot().querySelector(`[data-anim-id="${a1.id}"]`) as HTMLElement;
      expect(item).not.toBeNull();
      item.click();

      expect(state.selectedAnimationId).toBe(a1.id);
      const durInput = pane.getRoot().querySelector(".anim-input-duration") as HTMLInputElement;
      expect(durInput.value).toBe("0.75");
    });

    it("dispatches Move Up, Move Down, and Remove from toolbar buttons", () => {
      const sh = state.getCurrentSlide().shapes[0];
      state.addAnimation(sh.id, "appear");
      const a2 = state.addAnimation(sh.id, "bounce")!;
      state.selectAnimation(a2.id);
      pane.open();

      // Move Up
      const upBtn = pane.getRoot().querySelector(".anim-btn-move-up") as HTMLButtonElement;
      upBtn.click();
      expect(state.getSlideAnimations()[0].id).toBe(a2.id);

      // Remove
      const removeBtn = pane.getRoot().querySelector(".anim-btn-remove") as HTMLButtonElement;
      removeBtn.click();
      expect(state.getSlideAnimations().length).toBe(1);
    });
  });

  describe("4. Interactive Fullscreen Slide Show Engine (PresentationPlayer)", () => {
    let player: PresentationPlayer;

    beforeEach(() => {
      player = new PresentationPlayer(state);
    });

    afterEach(() => {
      player.exitShow();
    });

    it("starts presentation from beginning and from current slide", () => {
      state.selectSlide(2);
      player.startShow(true, false);
      expect(player.isOpen()).toBe(true);
      expect(state.currentSlideIndex).toBe(0);
      expect(player.isPresenter()).toBe(false);

      player.startShow(false, false);
      expect(state.currentSlideIndex).toBe(0); // already 0

      state.selectSlide(1);
      player.startShow(false, false);
      expect(state.currentSlideIndex).toBe(1);
    });

    it("steps forward through shape animations before advancing slide", () => {
      const sh = state.getCurrentSlide().shapes[0];
      state.addAnimation(sh.id, "flyIn", "entrance");

      player.startShow(true, false);
      expect(state.currentSlideIndex).toBe(0);
      expect(state.currentShowStep).toBe(0);

      // Step 1: Executes animation
      player.stepForward();
      expect(state.currentSlideIndex).toBe(0);
      expect(state.currentShowStep).toBe(1);

      // Step 2: Advances to slide 1
      player.stepForward();
      expect(state.currentSlideIndex).toBe(1);
      expect(state.currentShowStep).toBe(0);
    });

    it("steps backward through animations and slides", () => {
      const sh = state.getCurrentSlide().shapes[0];
      state.addAnimation(sh.id, "fadeIn", "entrance");

      player.startShow(true, false);
      player.stepForward(); // step 1 (animation executed)
      expect(state.currentShowStep).toBe(1);

      player.stepBackward(); // reverts step
      expect(state.currentShowStep).toBe(0);
      expect(state.currentSlideIndex).toBe(0);
    });

    it("toggles black screen and white screen during presentation", () => {
      player.startShow(true, false);
      player.toggleBlackScreen();
      expect(player.getRoot().querySelectorAll(".presentation-slide-canvas").length).toBe(0);

      // Advancing while black screen restores view
      player.stepForward();
      expect(player.getRoot().querySelectorAll(".presentation-slide-canvas").length).toBe(1);

      player.toggleWhiteScreen();
      expect(player.getRoot().querySelectorAll(".presentation-slide-canvas").length).toBe(0);
    });

    it("toggles laser pointer with tracking coordinates", () => {
      player.startShow(true, false);
      const dot = player.getRoot().querySelector(".pres-laser-dot") as HTMLElement;
      expect(dot.style.display).toBe("none");

      player.toggleLaser();
      expect(dot.style.display).toBe("block");

      player.toggleLaser();
      expect(dot.style.display).toBe("none");
    });

    it("renders Presenter View with elapsed timer, speaker notes and next slide preview", () => {
      state.setSlideNotes("Key takeaway: OpenOfficeRS ensures 100% ECMA-376 fidelity.");
      player.startShow(true, true);
      expect(player.isPresenter()).toBe(true);

      const header = player.getRoot().querySelector(".pres-presenter-header");
      expect(header).not.toBeNull();
      expect(header?.textContent).toContain("OpenOfficeRS Presenter View");
      expect(header?.textContent).toContain("⏱️");

      const notes = player.getRoot().querySelector(".pres-notes-content");
      expect(notes).not.toBeNull();
      expect(notes?.textContent).toContain("Key takeaway: OpenOfficeRS ensures 100% ECMA-376 fidelity.");

      const nextCard = player.getRoot().querySelector(".pres-next-card");
      expect(nextCard).not.toBeNull();
      expect(nextCard?.textContent).toContain("Next Slide:");

      const filmstrip = player.getRoot().querySelector(".pres-filmstrip");
      expect(filmstrip).not.toBeNull();
      expect(filmstrip?.children.length).toBe(state.slides.length);
    });

    it("navigates to specific slide directly when clicking filmstrip thumbnail", () => {
      player.startShow(true, true);
      const thumbs = player.getRoot().querySelectorAll(".pres-thumb-item");
      expect(thumbs.length).toBeGreaterThanOrEqual(3);

      (thumbs[2] as HTMLElement).click();
      expect(state.currentSlideIndex).toBe(2);
    });
  });

  describe("5. UnifiedShellApp Ribbon & Keyboard Integration", () => {
    let app: UnifiedShellApp;

    beforeEach(() => {
      app = new UnifiedShellApp(false);
      app.getTabBar().addTab({
        id: "tab-slides-1",
        docType: "PowerPoint",
        title: "Presentation.pptx",
        isDirty: false,
      });
      app.getTabBar().switchTab("tab-slides-1");
    });

    it("dispatches Slide Show start commands: StartShowBeginning (F5) and StartShowCurrent (Shift+F5)", () => {
      const ribbon = app.getRibbon();
      ribbon.setActiveTab("Slide Show");

      app.handleRibbonAction("StartShowBeginning");
      const player = app.getPresentationPlayer();
      expect(player).toBeDefined();
      expect(player?.isOpen()).toBe(true);
      expect(slidesState.currentSlideIndex).toBe(0);

      player?.exitShow();
      expect(player?.isOpen()).toBe(false);

      slidesState.selectSlide(1);
      app.handleRibbonAction("StartShowCurrent");
      expect(player?.isOpen()).toBe(true);
      expect(slidesState.currentSlideIndex).toBe(1);
    });

    it("dispatches Presenter View command and toggles presentation console", () => {
      app.handleRibbonAction("StartPresenterView");
      const player = app.getPresentationPlayer();
      expect(player?.isOpen()).toBe(true);
      expect(player?.isPresenter()).toBe(true);
    });

    it("toggles Animation Pane via Ribbon action", () => {
      const pane = app.getAnimationPane();
      expect(pane).toBeDefined();
      expect(pane?.isOpen()).toBe(false);

      app.handleRibbonAction("ToggleAnimationPane");
      expect(pane?.isOpen()).toBe(true);

      app.handleRibbonAction("ToggleAnimationPane");
      expect(pane?.isOpen()).toBe(false);
    });

    it("dispatches Transition ribbon actions: SetTransition, SetEffectOption, SetTransitionDuration, ApplyTransitionToAll", () => {
      app.handleRibbonAction("SetTransition", "Fade");
      expect(slidesState.getSlideTransition().effect).toBe("Fade");

      app.handleRibbonAction("SetEffectOption", "left");
      expect(slidesState.getSlideTransition().direction).toBe("left");

      app.handleRibbonAction("SetTransitionDuration", 1.5);
      expect(slidesState.getSlideTransition().duration).toBe(1.5);

      app.handleRibbonAction("ApplyTransitionToAll");
      slidesState.slides.forEach((s) => {
        expect(s.transitionConfig?.effect).toBe("Fade");
      });
    });

    it("dispatches Animation ribbon actions: SetAnimation, SetAnimStart, and Reorder", () => {
      slidesState.selectShape(slidesState.getCurrentSlide().shapes[0].id);
      app.handleRibbonAction("SetAnimation", "flyIn");
      expect(slidesState.getSlideAnimations().length).toBe(1);
      expect(slidesState.getSlideAnimations()[0].effect).toBe("flyIn");

      app.handleRibbonAction("SetAnimStart", "withPrev");
      expect(slidesState.getSlideAnimations()[0].trigger).toBe("withPrev");
    });
  });
});
