import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { SlidesState, slidesState } from "../state/slidesState";
import { SlidesRibbon } from "../components/ribbon/SlidesRibbon";
import { SlideStageCanvas } from "../components/viewports/SlideStageCanvas";
import { UnifiedShellApp } from "../main";

describe("PowerPoint Animations, Slide Show, Review & View Ribbons (100% Working) (GEC-v4.0 Subphase 31.1.2)", () => {
  let state: SlidesState;
  let ribbon: SlidesRibbon;
  let ribbonBody: HTMLElement;
  let app: UnifiedShellApp;
  let stageCanvas: SlideStageCanvas;
  let actions: { action: string; payload?: unknown }[];

  beforeEach(() => {
    // Setup DOM environment
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
      createLinearGradient: () => ({ addColorStop: () => {} }),
      createRadialGradient: () => ({ addColorStop: () => {} }),
      measureText: (txt: string) => ({ width: txt.length * 8 }),
      setLineDash: () => {},
      font: "24px Aptos, sans-serif",
      fillStyle: "#000",
      strokeStyle: "#000",
      lineWidth: 1,
      textAlign: "left",
      textBaseline: "top",
      shadowColor: "",
      shadowBlur: 0,
      shadowOffsetX: 0,
      shadowOffsetY: 0,
    };

    HTMLCanvasElement.prototype.getContext = ((type: string) => {
      if (type === "2d") return mockCtx;
      return null;
    }) as any;

    state = slidesState;
    state.resetToBlank("Presentation.pptx");

    app = new UnifiedShellApp(false);
    app.getTabBar().addTab({
      id: "tab-slides-1",
      docType: "PowerPoint",
      title: "Deck.pptx",
      isDirty: false,
    });
    app.getTabBar().switchTab("tab-slides-1");
    (app as any).currentDocType = "PowerPoint";

    ribbon = new SlidesRibbon();
    ribbonBody = document.createElement("div");
    ribbonBody.className = "ribbon-body";
    document.body.appendChild(ribbonBody);

    const viewport = document.getElementById("viewport-container")!;
    stageCanvas = new SlideStageCanvas(viewport, state);

    actions = [];
  });

  afterEach(() => {
    document.body.innerHTML = "";
  });

  // =========================================================================
  // 1. ANIMATIONS TAB
  // =========================================================================
  describe("Animations Tab (Entrance, Emphasis, Exit, Motion, Timing)", () => {
    beforeEach(() => {
      actions = [];
      ribbon.renderTab("Animations", ribbonBody, (action, payload) => {
        actions.push({ action, payload });
        app.handleRibbonAction(action, payload);
      });
      state.insertShape("rect");
      expect(state.selectedShapeId).not.toBeNull();
    });

    it("1.1 triggers Preview Animation on current slide", () => {
      const btn = ribbonBody.querySelector("[data-testid='slides-btn-preview-anim']") as HTMLButtonElement;
      expect(btn).not.toBeNull();
      btn.click();
      expect(actions).toContainEqual({ action: "PreviewAnimation", payload: undefined });
      expect(state.isPreviewingAnimation).toBe(true);
    });

    it("1.2 applies Entrance animation effects (Appear, Fade In, Fly In, Fly Up, Float In, Split, Wipe, Shape, Wheel, Random Bars, Grow & Turn, Bounce, Zoom, Swivel, Pop)", () => {
      const testCases = [
        { testId: "slides-anim-appear", expected: "appear" },
        { testId: "slides-anim-fade-in", expected: "fadeIn" },
        { testId: "slides-anim-fly-in", expected: "flyIn" },
        { testId: "slides-anim-fly-up", expected: "flyIn" },
        { testId: "slides-anim-float-in", expected: "floatIn" },
        { testId: "slides-anim-split", expected: "split" },
        { testId: "slides-anim-wipe", expected: "wipe" },
        { testId: "slides-anim-shape", expected: "shape" },
        { testId: "slides-anim-wheel", expected: "wheel" },
        { testId: "slides-anim-random-bars", expected: "randomBars" },
        { testId: "slides-anim-grow-turn", expected: "growTurn" },
        { testId: "slides-anim-bounce", expected: "bounce" },
        { testId: "slides-anim-zoom", expected: "zoom" },
        { testId: "slides-anim-swivel", expected: "swivel" },
        { testId: "slides-anim-pop", expected: "zoom" },
      ];

      for (const tc of testCases) {
        const btn = ribbonBody.querySelector(`[data-testid="${tc.testId}"]`) as HTMLButtonElement;
        expect(btn).not.toBeNull();
        btn.click();
        const anim = state.getSlideAnimations().pop();
        expect(anim).toBeDefined();
        expect(anim!.effect).toBe(tc.expected);
      }
    });

    it("1.3 applies Emphasis animation effects (Pulse, Color Pulse, Teeter, Spin, Grow/Shrink)", () => {
      const testCases = [
        { testId: "slides-anim-pulse", expected: "pulse" },
        { testId: "slides-anim-color-pulse", expected: "colorPulse" },
        { testId: "slides-anim-teeter", expected: "teeter" },
        { testId: "slides-anim-spin", expected: "spin" },
        { testId: "slides-anim-grow-shrink", expected: "growShrink" },
      ];

      for (const tc of testCases) {
        const btn = ribbonBody.querySelector(`[data-testid="${tc.testId}"]`) as HTMLButtonElement;
        expect(btn).not.toBeNull();
        btn.click();
        const anim = state.getSlideAnimations().pop();
        expect(anim).toBeDefined();
        expect(anim!.effect).toBe(tc.expected);
      }
    });

    it("1.4 applies Exit animation effects (None, Disappear, Fade Out, Fly Out, Float Out, Shrink & Turn, Zoom Out)", () => {
      const testCases = [
        { testId: "slides-anim-disappear", expected: "disappear" },
        { testId: "slides-anim-fade-out", expected: "fadeOut" },
        { testId: "slides-anim-fly-out", expected: "flyOut" },
        { testId: "slides-anim-float-out", expected: "floatOut" },
        { testId: "slides-anim-shrink-turn", expected: "shrinkTurn" },
        { testId: "slides-anim-zoom-out", expected: "zoomOut" },
      ];

      for (const tc of testCases) {
        const btn = ribbonBody.querySelector(`[data-testid="${tc.testId}"]`) as HTMLButtonElement;
        expect(btn).not.toBeNull();
        btn.click();
        const anim = state.getSlideAnimations().pop();
        expect(anim).toBeDefined();
        expect(anim!.effect).toBe(tc.expected);
      }

      // None removes animations
      const noneBtn = ribbonBody.querySelector("[data-testid='slides-anim-none']") as HTMLButtonElement;
      expect(noneBtn).not.toBeNull();
      noneBtn.click();
      expect(state.getShapeAnimations(state.selectedShapeId!).length).toBe(0);
    });

    it("1.5 toggles Animation Pane and sets Motion Path", () => {
      const motionBtn = ribbonBody.querySelector("[data-testid='slides-anim-motion-path']") as HTMLButtonElement;
      expect(motionBtn).not.toBeNull();
      motionBtn.click();
      expect(actions).toContainEqual({ action: "AddMotionPath", payload: undefined });
      const anim = state.getSlideAnimations().pop();
      expect(anim).toBeDefined();
      expect(anim!.effect).toBe("motionPath");

      const paneBtn = ribbonBody.querySelector("[data-testid='slides-btn-anim-pane']") as HTMLButtonElement;
      expect(paneBtn).not.toBeNull();
      const initialPaneState = state.showAnimationPane;
      paneBtn.click();
      expect(actions).toContainEqual({ action: "ToggleAnimationPane", payload: undefined });
      expect(state.showAnimationPane).toBe(!initialPaneState);

      const addAnimBtn = ribbonBody.querySelector("[data-testid='slides-btn-add-anim']") as HTMLButtonElement;
      expect(addAnimBtn).not.toBeNull();
      addAnimBtn.click();
      expect(actions).toContainEqual({ action: "AddAnimation", payload: undefined });

      const triggerBtn = ribbonBody.querySelector("[data-testid='slides-btn-trigger']") as HTMLButtonElement;
      expect(triggerBtn).not.toBeNull();
      triggerBtn.click();
      expect(actions).toContainEqual({ action: "SetAnimationTrigger", payload: undefined });

      const effectOptionsBtn = ribbonBody.querySelector("[data-testid='slides-btn-effect-options']") as HTMLButtonElement;
      expect(effectOptionsBtn).not.toBeNull();
      effectOptionsBtn.click();
      expect(actions).toContainEqual({ action: "EffectOptions", payload: "property" });
    });

    it("1.6 sets animation timing (Start trigger, Duration, Delay, Reorder)", () => {
      // Add an animation to adjust timing on
      state.addAnimation(state.selectedShapeId!, "fadeIn");
      expect(state.selectedAnimationId).not.toBeNull();

      const clickBtn = ribbonBody.querySelector("[data-testid='slides-btn-start-click']") as HTMLButtonElement;
      const withPrevBtn = ribbonBody.querySelector("[data-testid='slides-btn-start-with-prev']") as HTMLButtonElement;
      const afterPrevBtn = ribbonBody.querySelector("[data-testid='slides-btn-start-after-prev']") as HTMLButtonElement;

      withPrevBtn.click();
      expect(actions).toContainEqual({ action: "SetAnimStart", payload: "withPrev" });
      expect(state.getSelectedAnimation()?.trigger).toBe("withPrev");

      afterPrevBtn.click();
      expect(actions).toContainEqual({ action: "SetAnimStart", payload: "afterPrev" });
      expect(state.getSelectedAnimation()?.trigger).toBe("afterPrev");

      clickBtn.click();
      expect(actions).toContainEqual({ action: "SetAnimStart", payload: "click" });
      expect(state.getSelectedAnimation()?.trigger).toBe("click");

      const dur05 = ribbonBody.querySelector("[data-testid='slides-btn-duration-anim-05']") as HTMLButtonElement;
      const dur10 = ribbonBody.querySelector("[data-testid='slides-btn-duration-anim-10']") as HTMLButtonElement;
      const dur20 = ribbonBody.querySelector("[data-testid='slides-btn-duration-anim-20']") as HTMLButtonElement;
      dur05.click();
      expect(state.getSelectedAnimation()?.duration).toBe(0.5);
      dur10.click();
      expect(state.getSelectedAnimation()?.duration).toBe(1.0);
      dur20.click();
      expect(state.getSelectedAnimation()?.duration).toBe(2.0);

      const delay0 = ribbonBody.querySelector("[data-testid='slides-btn-delay-0']") as HTMLButtonElement;
      const delay25 = ribbonBody.querySelector("[data-testid='slides-btn-delay-25']") as HTMLButtonElement;
      const delay50 = ribbonBody.querySelector("[data-testid='slides-btn-delay-50']") as HTMLButtonElement;
      delay25.click();
      expect(state.getSelectedAnimation()?.delay).toBe(0.25);
      delay50.click();
      expect(state.getSelectedAnimation()?.delay).toBe(0.5);
      delay0.click();
      expect(state.getSelectedAnimation()?.delay).toBe(0);

      const earlierBtn = ribbonBody.querySelector("[data-testid='slides-btn-reorder-earlier']") as HTMLButtonElement;
      const laterBtn = ribbonBody.querySelector("[data-testid='slides-btn-reorder-later']") as HTMLButtonElement;
      expect(earlierBtn).not.toBeNull();
      expect(laterBtn).not.toBeNull();
      earlierBtn.click();
      laterBtn.click();
      expect(actions.some((a) => a.action === "ReorderAnimationEarlier")).toBe(true);
      expect(actions.some((a) => a.action === "ReorderAnimationLater")).toBe(true);
    });
  });

  // =========================================================================
  // 2. SLIDE SHOW TAB
  // =========================================================================
  describe("Slide Show Tab (From Beginning, Current, Hide, Rehearse, Loop, Presenter)", () => {
    beforeEach(() => {
      actions = [];
      ribbon.renderTab("Slide Show", ribbonBody, (action, payload) => {
        actions.push({ action, payload });
        app.handleRibbonAction(action, payload);
      });
      state.addSlide();
      state.addSlide();
      expect(state.slides.length).toBe(3);
    });

    it("2.1 starts Slide Show From Beginning (F5), skipping hidden slides", () => {
      state.currentSlideIndex = 0;
      state.toggleHideSlide();
      expect(state.slides[0].isHidden).toBe(true);

      const beginBtn = ribbonBody.querySelector("[data-testid='slides-btn-show-begin']") as HTMLButtonElement;
      expect(beginBtn).not.toBeNull();
      beginBtn.click();

      expect(actions).toContainEqual({ action: "StartShowBeginning", payload: undefined });
      expect(state.isShowActive).toBe(true);
      // Because slide 0 is hidden, first visible slide is slide 1
      expect(state.currentSlideIndex).toBe(1);

      state.exitShow();
      expect(state.isShowActive).toBe(false);
    });

    it("2.2 starts Slide Show From Current Slide (Shift+F5)", () => {
      state.currentSlideIndex = 2;
      const currBtn = ribbonBody.querySelector("[data-testid='slides-btn-show-curr']") as HTMLButtonElement;
      expect(currBtn).not.toBeNull();
      currBtn.click();

      expect(actions).toContainEqual({ action: "StartShowCurrent", payload: undefined });
      expect(state.isShowActive).toBe(true);
      expect(state.currentSlideIndex).toBe(2);

      state.exitShow();
    });

    it("2.3 toggles Hide Slide button", () => {
      state.currentSlideIndex = 1;
      expect(state.slides[1].isHidden).toBeFalsy();

      const hideBtn = ribbonBody.querySelector("[data-testid='slides-btn-hide-slide']") as HTMLButtonElement;
      expect(hideBtn).not.toBeNull();
      hideBtn.click();
      expect(actions).toContainEqual({ action: "HideSlide", payload: undefined });
      expect(state.slides[1].isHidden).toBe(true);

      hideBtn.click();
      expect(state.slides[1].isHidden).toBe(false);
    });

    it("2.4 records timings with Rehearse Timings and toggles Loop Continuously", () => {
      const rehearseBtn = ribbonBody.querySelector("[data-testid='slides-btn-rehearse']") as HTMLButtonElement;
      expect(rehearseBtn).not.toBeNull();
      rehearseBtn.click();
      expect(actions).toContainEqual({ action: "RehearseTimings", payload: undefined });
      expect(state.isRehearsingTimings).toBe(true);

      // Advance slide and then finish rehearsal
      state.nextSlide();
      rehearseBtn.click();
      expect(state.isRehearsingTimings).toBe(false);
      expect(state.slideTimings.size).toBeGreaterThan(0);

      // Loop continuously toggle
      const loopBtn = ribbonBody.querySelector("[data-testid='slides-btn-loop-continuous']") as HTMLButtonElement;
      expect(loopBtn).not.toBeNull();
      loopBtn.click();
      expect(actions).toContainEqual({ action: "SetLoopContinuously", payload: undefined });
      expect(state.loopContinuously).toBe(true);

      // When loopContinuously is true, advancing past the end wraps back to 0
      state.startShow(false, false);
      state.currentSlideIndex = state.slides.length - 1;
      state.nextSlide();
      expect(state.currentSlideIndex).toBe(0);
      state.exitShow();
    });

    it("2.5 handles Set Up Show, Record Show and Presenter View buttons", () => {
      const setupBtn = ribbonBody.querySelector("[data-testid='slides-btn-setup-show']") as HTMLButtonElement;
      expect(setupBtn).not.toBeNull();
      setupBtn.click();
      expect(actions).toContainEqual({ action: "SetUpShow", payload: undefined });

      const customBtn = ribbonBody.querySelector("[data-testid='slides-btn-custom-show']") as HTMLButtonElement;
      expect(customBtn).not.toBeNull();
      customBtn.click();
      expect(actions).toContainEqual({ action: "CustomSlideShow", payload: undefined });

      const kioskBtn = ribbonBody.querySelector("[data-testid='slides-btn-kiosk-mode']") as HTMLButtonElement;
      expect(kioskBtn).not.toBeNull();
      kioskBtn.click();
      expect(actions).toContainEqual({ action: "SetKioskMode", payload: undefined });
      expect(state.isKioskMode).toBe(true);

      const recordBtn = ribbonBody.querySelector("[data-testid='slides-btn-record-show']") as HTMLButtonElement;
      expect(recordBtn).not.toBeNull();
      recordBtn.click();
      expect(actions).toContainEqual({ action: "RecordShow", payload: undefined });

      const onlineBtn = ribbonBody.querySelector("[data-testid='slides-btn-present-online']") as HTMLButtonElement;
      expect(onlineBtn).not.toBeNull();
      onlineBtn.click();
      expect(actions).toContainEqual({ action: "PresentOnline", payload: undefined });

      const presenterBtn = ribbonBody.querySelector("[data-testid='slides-btn-presenter-view']") as HTMLButtonElement;
      expect(presenterBtn).not.toBeNull();
      presenterBtn.click();
      expect(actions).toContainEqual({ action: "StartPresenterView", payload: undefined });

      const studioBtn = ribbonBody.querySelector("[data-testid='slides-btn-presenter-studio']") as HTMLButtonElement;
      expect(studioBtn).not.toBeNull();
      studioBtn.click();
      expect(actions).toContainEqual({ action: "StartDualScreenPresenter", payload: undefined });

      const togglePresenterBtn = ribbonBody.querySelector("[data-testid='slides-btn-toggle-presenter']") as HTMLButtonElement;
      expect(togglePresenterBtn).not.toBeNull();
      togglePresenterBtn.click();
      expect(actions).toContainEqual({ action: "TogglePresenterView", payload: undefined });
      state.exitShow();
    });
  });

  // =========================================================================
  // 3. REVIEW TAB
  // =========================================================================
  describe("Review Tab (Proofing, Comments, Compare, Copilot)", () => {
    beforeEach(() => {
      actions = [];
      ribbon.renderTab("Review", ribbonBody, (action, payload) => {
        actions.push({ action, payload });
        app.handleRibbonAction(action, payload);
      });
    });

    it("3.1 executes Spelling and Thesaurus commands", () => {
      const spellBtn = ribbonBody.querySelector("[data-testid='slides-btn-spelling']") as HTMLButtonElement;
      expect(spellBtn).not.toBeNull();
      spellBtn.click();
      expect(actions).toContainEqual({ action: "CheckSpelling", payload: undefined });
      expect(state.isSpellingChecked).toBe(true);

      const thesaurusBtn = ribbonBody.querySelector("[data-testid='slides-btn-thesaurus']") as HTMLButtonElement;
      expect(thesaurusBtn).not.toBeNull();
      thesaurusBtn.click();
      expect(actions).toContainEqual({ action: "Thesaurus", payload: undefined });
      expect(state.thesaurusQuery).toBeDefined();
    });

    it("3.2 adds, shows/hides, and deletes slide comments", () => {
      const newCommentBtn = ribbonBody.querySelector("[data-testid='slides-btn-new-comment']") as HTMLButtonElement;
      expect(newCommentBtn).not.toBeNull();
      newCommentBtn.click();
      expect(actions).toContainEqual({ action: "NewComment", payload: undefined });

      const comments = state.getComments();
      expect(comments.length).toBeGreaterThan(0);
      const commentId = comments[0].id;

      const showCommentsBtn = ribbonBody.querySelector("[data-testid='slides-btn-show-comments']") as HTMLButtonElement;
      expect(showCommentsBtn).not.toBeNull();
      const currentShow = state.showCommentsPane;
      showCommentsBtn.click();
      expect(actions).toContainEqual({ action: "ShowComments", payload: undefined });
      expect(state.showCommentsPane).toBe(!currentShow);

      const prevBtn = ribbonBody.querySelector("[data-testid='slides-btn-prev-comment']") as HTMLButtonElement;
      const nextBtn = ribbonBody.querySelector("[data-testid='slides-btn-next-comment']") as HTMLButtonElement;
      expect(prevBtn).not.toBeNull();
      expect(nextBtn).not.toBeNull();
      nextBtn.click();
      expect(actions).toContainEqual({ action: "NextComment", payload: undefined });
      prevBtn.click();
      expect(actions).toContainEqual({ action: "PrevComment", payload: undefined });

      const deleteCommentBtn = ribbonBody.querySelector("[data-testid='slides-btn-delete-comment']") as HTMLButtonElement;
      expect(deleteCommentBtn).not.toBeNull();
      deleteCommentBtn.click();
      expect(actions).toContainEqual({ action: "DeleteComment", payload: undefined });
      expect(state.getComments().find((c) => c.id === commentId)).toBeUndefined();
    });

    it("3.3 executes Compare Presentations and Copilot buttons", () => {
      const compareBtn = ribbonBody.querySelector("[data-testid='slides-btn-compare']") as HTMLButtonElement;
      expect(compareBtn).not.toBeNull();
      compareBtn.click();
      expect(actions).toContainEqual({ action: "Compare", payload: undefined });

      const copilotBtn = ribbonBody.querySelector("[data-testid='slides-btn-copilot-review']") as HTMLButtonElement;
      expect(copilotBtn).not.toBeNull();
      copilotBtn.click();
      expect(actions).toContainEqual({ action: "ToggleCopilot", payload: undefined });
    });
  });

  // =========================================================================
  // 4. VIEW TAB
  // =========================================================================
  describe("View Tab (Presentation Views, Master Views, Show Toggles, Color, Zoom)", () => {
    beforeEach(() => {
      actions = [];
      ribbon.renderTab("View", ribbonBody, (action, payload) => {
        actions.push({ action, payload });
        app.handleRibbonAction(action, payload);
      });
    });

    it("4.1 switches Presentation Views (Normal, Outline, Slide Sorter, Notes Page, Reading)", () => {
      const views = [
        { testId: "slides-btn-view-normal", action: "ViewNormal", expected: "normal" },
        { testId: "slides-btn-view-outline", action: "ViewOutline", expected: "outline" },
        { testId: "slides-btn-view-sorter", action: "ViewSlideSorter", expected: "sorter" },
        { testId: "slides-btn-view-notes", action: "ViewNotesPage", expected: "notes" },
        { testId: "slides-btn-view-reading", action: "ViewReadingView", expected: "reading" },
      ];

      for (const v of views) {
        const btn = ribbonBody.querySelector(`[data-testid="${v.testId}"]`) as HTMLButtonElement;
        expect(btn).not.toBeNull();
        btn.click();
        expect(actions).toContainEqual({ action: v.action, payload: undefined });
        expect(state.currentView).toBe(v.expected);
      }
    });

    it("4.2 switches Master Views (Slide Master, Handout Master, Notes Master)", () => {
      const masters = [
        { testId: "slides-btn-master-slide", action: "ViewSlideMaster", expected: "slideMaster" },
        { testId: "slides-btn-master-handout", action: "ViewHandoutMaster", expected: "handoutMaster" },
        { testId: "slides-btn-master-notes", action: "ViewNotesMaster", expected: "notesMaster" },
      ];

      for (const m of masters) {
        const btn = ribbonBody.querySelector(`[data-testid="${m.testId}"]`) as HTMLButtonElement;
        expect(btn).not.toBeNull();
        btn.click();
        expect(actions).toContainEqual({ action: m.action, payload: undefined });
        expect(state.masterViewMode).toBe(m.expected);
      }
    });

    it("4.3 toggles Show elements (Ruler, Gridlines, Guides, Speaker Notes Drawer)", () => {
      const rulerBtn = ribbonBody.querySelector("[data-testid='slides-btn-show-ruler']") as HTMLButtonElement;
      expect(rulerBtn).not.toBeNull();
      const initialRuler = state.showRuler;
      rulerBtn.click();
      expect(actions).toContainEqual({ action: "ToggleRuler", payload: undefined });
      expect(state.showRuler).toBe(!initialRuler);

      const gridBtn = ribbonBody.querySelector("[data-testid='slides-btn-show-gridlines']") as HTMLButtonElement;
      expect(gridBtn).not.toBeNull();
      const initialGrid = state.showGridlines;
      gridBtn.click();
      expect(actions).toContainEqual({ action: "ToggleGridlines", payload: undefined });
      expect(state.showGridlines).toBe(!initialGrid);

      const guidesBtn = ribbonBody.querySelector("[data-testid='slides-btn-show-guides']") as HTMLButtonElement;
      expect(guidesBtn).not.toBeNull();
      const initialGuides = state.showGuides;
      guidesBtn.click();
      expect(actions).toContainEqual({ action: "ToggleGuides", payload: undefined });
      expect(state.showGuides).toBe(!initialGuides);

      const notesBtn = ribbonBody.querySelector("[data-testid='slides-btn-show-notes']") as HTMLButtonElement;
      expect(notesBtn).not.toBeNull();
      const initialNotes = state.showSpeakerNotesDrawer;
      notesBtn.click();
      expect(actions).toContainEqual({ action: "ToggleNotes", payload: undefined });
      expect(state.showSpeakerNotesDrawer).toBe(!initialNotes);
    });

    it("4.4 switches Color Modes (Color, Grayscale, Black & White)", () => {
      const modes = [
        { testId: "slides-btn-color-mode-grayscale", expected: "grayscale" },
        { testId: "slides-btn-color-mode-bw", expected: "bw" },
        { testId: "slides-btn-color-mode-color", expected: "color" },
      ];

      for (const m of modes) {
        const btn = ribbonBody.querySelector(`[data-testid="${m.testId}"]`) as HTMLButtonElement;
        expect(btn).not.toBeNull();
        btn.click();
        expect(actions).toContainEqual({ action: "SetColorMode", payload: m.expected });
        expect(state.colorMode).toBe(m.expected);
      }
    });

    it("4.5 adjusts Zoom controls (Fit Window, 50%, 100%, 200%)", () => {
      const z50 = ribbonBody.querySelector("[data-testid='slides-btn-zoom-50']") as HTMLButtonElement;
      expect(z50).not.toBeNull();
      z50.click();
      expect(actions).toContainEqual({ action: "SetZoom", payload: 50 });
      expect(state.zoomLevel).toBe(50);

      const z200 = ribbonBody.querySelector("[data-testid='slides-btn-zoom-200']") as HTMLButtonElement;
      expect(z200).not.toBeNull();
      z200.click();
      expect(actions).toContainEqual({ action: "SetZoom", payload: 200 });
      expect(state.zoomLevel).toBe(200);

      const z100 = ribbonBody.querySelector("[data-testid='slides-btn-zoom-100']") as HTMLButtonElement;
      expect(z100).not.toBeNull();
      z100.click();
      expect(actions).toContainEqual({ action: "Zoom100", payload: undefined });
      expect(state.zoomLevel).toBe(100);

      const zFit = ribbonBody.querySelector("[data-testid='slides-btn-zoom-fit']") as HTMLButtonElement;
      expect(zFit).not.toBeNull();
      zFit.click();
      expect(actions).toContainEqual({ action: "ZoomFit", payload: undefined });
      expect(state.zoomLevel).toBe(100);
    });
  });

  // =========================================================================
  // 5. CANVAS & STAGE RENDERING VERIFICATIONS
  // =========================================================================
  describe("SlideStageCanvas Render Elements (Gridlines, Guides, Comments, Grayscale)", () => {
    it("5.1 renders without error when gridlines, guides, and comments are active", () => {
      state.showGridlines = true;
      state.showGuides = true;
      state.colorMode = "grayscale";
      state.addComment("Review note for slide 1");

      expect(() => {
        stageCanvas.render();
      }).not.toThrow();

      state.colorMode = "bw";
      expect(() => {
        stageCanvas.render();
      }).not.toThrow();
    });
  });

  // =========================================================================
  // 6. COMPREHENSIVE GEC-v5.0 RIBBON DOM AUDIT
  // =========================================================================
  describe("6. Comprehensive GEC-v5.0 DOM Audit for Animations, Slide Show, Review, and View tabs", () => {
    it("renders all four tabs and verifies 100% of buttons are present, labeled, and interactive", () => {
      const tabs = ["Animations", "Slide Show", "Review", "View"];
      for (const tab of tabs) {
        ribbonBody.innerHTML = "";
        ribbon.renderTab(tab, ribbonBody, (action, payload) => {
          actions.push({ action, payload });
        });

        const groups = ribbonBody.querySelectorAll(".ribbon-group");
        expect(groups.length).toBeGreaterThan(0);

        const buttons = ribbonBody.querySelectorAll("button");
        expect(buttons.length).toBeGreaterThan(0);

        buttons.forEach((btn) => {
          expect(btn.textContent?.trim().length).toBeGreaterThan(0);
          expect(() => btn.click()).not.toThrow();
        });
      }
    }, 15000);
  });
});
