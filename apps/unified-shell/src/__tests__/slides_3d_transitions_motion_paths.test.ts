import { describe, it, expect, beforeEach } from "vitest";
import { SlidesState, slidesState } from "../state/slidesState";
import { SlideTransitionEngine } from "../components/SlideTransitionEngine";
import { SlideStageCanvas } from "../components/viewports/SlideStageCanvas";
import { PresentationPlayer } from "../components/PresentationPlayer";
import { UnifiedShellApp } from "../main";

describe("3D Shader Transitions & Keyframe Motion Paths (Subphase 19.1.2)", () => {
  let state: SlidesState;
  let canvasComponent: SlideStageCanvas;
  let container: HTMLElement;

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
      </div>
    `;

    state = new SlidesState();
    container = document.getElementById("viewport-container")!;
    canvasComponent = new SlideStageCanvas(container, state);
    SlideTransitionEngine.setHardwareAcceleration(true);
  });

  it("1. configures 3D slide transitions: Cube, Flip, Vortex, Honeycomb, Origami, Ripple", () => {
    state.setTransition("Cube", 1.5, "right");
    let trans = state.getSlideTransition();
    expect(trans.effect).toBe("Cube");
    expect(trans.duration).toBe(1.5);
    expect(trans.direction).toBe("right");

    state.setTransition("Flip", 1.2, "left");
    trans = state.getSlideTransition();
    expect(trans.effect).toBe("Flip");
    expect(trans.duration).toBe(1.2);
    expect(trans.direction).toBe("left");

    state.setTransition("Vortex", 2.0);
    expect(state.getSlideTransition().effect).toBe("Vortex");

    state.setTransition("Honeycomb", 1.0);
    expect(state.getSlideTransition().effect).toBe("Honeycomb");

    state.setTransition("Origami", 1.5);
    expect(state.getSlideTransition().effect).toBe("Origami");

    state.setTransition("Ripple", 1.0);
    expect(state.getSlideTransition().effect).toBe("Ripple");
  });

  it("2. adds motion path animation to slide shape and configures properties", () => {
    const shape = state.addShape("rect", 100, 100, 120, 80);
    const anim = state.addMotionPathAnimation(
      shape.id,
      "M 0 0 C 100 -50 200 150 300 0",
      2.0,
      "bounce",
      true
    );

    expect(anim).not.toBeNull();
    expect(anim?.effect).toBe("motionPath");
    expect(anim?.motionPath).toBe("M 0 0 C 100 -50 200 150 300 0");
    expect(anim?.duration).toBe(2.0);
    expect(anim?.easing).toBe("bounce");
    expect(anim?.rotateToPath).toBe(true);

    const slideAnims = state.getSlideAnimations();
    expect(slideAnims).toContainEqual(anim);
  });

  it("3. handles AddMotionPath action via execCommand", () => {
    const shape = state.addShape("ellipse", 200, 150, 100, 100);
    state.selectShape(shape.id);

    const handled = state.execCommand("AddMotionPath", "M 0 0 L 150 150");
    expect(handled).toBe(true);

    const anims = state.getSlideAnimations();
    const motionAnim = anims.find((a) => a.shapeId === shape.id && a.effect === "motionPath");
    expect(motionAnim).toBeDefined();
    expect(motionAnim?.motionPath).toBe("M 0 0 L 150 150");
  });

  it("4. evaluates velocity easing curves: linear, ease-in-out, bounce, spring", () => {
    expect(SlideTransitionEngine.evaluateEasing("linear", 0.0)).toBe(0.0);
    expect(SlideTransitionEngine.evaluateEasing("linear", 0.5)).toBe(0.5);
    expect(SlideTransitionEngine.evaluateEasing("linear", 1.0)).toBe(1.0);

    const easeMid = SlideTransitionEngine.evaluateEasing("ease-in-out", 0.5);
    expect(easeMid).toBeCloseTo(0.5, 2);

    expect(SlideTransitionEngine.evaluateEasing("bounce", 0.0)).toBe(0.0);
    expect(SlideTransitionEngine.evaluateEasing("bounce", 1.0)).toBeCloseTo(1.0, 4);

    expect(SlideTransitionEngine.evaluateEasing("spring", 0.0)).toBe(0.0);
    const springEnd = SlideTransitionEngine.evaluateEasing("spring", 1.0);
    expect(springEnd).toBeGreaterThan(0.9);
  });

  it("5. compiles SVG Bezier motion paths into smooth trajectory points", () => {
    // 1. Straight Line
    const lineTrajectory = SlideTransitionEngine.compileTrajectory("M 0 0 L 200 100", "linear", 5, true);
    expect(lineTrajectory.length).toBe(5);
    expect(lineTrajectory[0].x).toBe(0);
    expect(lineTrajectory[0].y).toBe(0);
    expect(lineTrajectory[4].x).toBe(200);
    expect(lineTrajectory[4].y).toBe(100);
    expect(lineTrajectory[2].angle).toBeCloseTo((Math.atan2(100, 200) * 180) / Math.PI, 1);

    // 2. Quadratic Curve
    const quadTrajectory = SlideTransitionEngine.compileTrajectory("M 0 0 Q 50 100 100 0", "ease-in-out", 11);
    expect(quadTrajectory.length).toBe(11);
    expect(quadTrajectory[0].x).toBe(0);
    expect(quadTrajectory[10].x).toBe(100);

    // 3. Cubic Curve
    const cubicTrajectory = SlideTransitionEngine.compileTrajectory("M 0 0 C 50 150 150 -50 200 100", "spring", 20);
    expect(cubicTrajectory.length).toBe(20);
    expect(cubicTrajectory[0].x).toBe(0);
    expect(cubicTrajectory[19].x).toBeCloseTo(200, 0);
  });

  it("6. executes hardware-accelerated 3D transitions on DOM element", async () => {
    const dummyEl = document.createElement("div");
    document.body.appendChild(dummyEl);

    let midpointCalled = false;
    await SlideTransitionEngine.executeTransition(
      dummyEl,
      { effect: "Cube", duration: 0.1, direction: "right" },
      () => {
        midpointCalled = true;
      }
    );

    expect(midpointCalled).toBe(true);
    expect(dummyEl.style.transition).toBe("none");
    expect(dummyEl.style.transform).toBe("none");
    expect(dummyEl.style.opacity).toBe("1");
    dummyEl.remove();
  });

  it("7. gracefully falls back to 2D cross-fade when hardware acceleration is disabled", async () => {
    SlideTransitionEngine.setHardwareAcceleration(false);
    expect(SlideTransitionEngine.getHardwareAcceleration()).toBe(false);

    const dummyEl = document.createElement("div");
    document.body.appendChild(dummyEl);

    let midpointCalled = false;
    await SlideTransitionEngine.executeTransition(
      dummyEl,
      { effect: "Vortex", duration: 0.1 },
      () => {
        midpointCalled = true;
      }
    );

    expect(midpointCalled).toBe(true);
    expect(dummyEl.style.opacity).toBe("1");
    dummyEl.remove();

    // Restore
    SlideTransitionEngine.setHardwareAcceleration(true);
  });

  it("8. previews 3D slide transition on SlideStageCanvas and animates motion path", async () => {
    state.setTransition("Flip", 0.1, "left");
    await canvasComponent.playTransitionPreview();

    const shape = state.addShape("rect", 80, 80, 100, 60);
    const motionAnim = state.addMotionPathAnimation(shape.id, "M 0 0 L 50 50", 0.05);
    expect(motionAnim).not.toBeNull();
    if (motionAnim) {
      await canvasComponent.playAnimationPreview(motionAnim);
    }
  });

  it("9. executes 3D transitions in PresentationPlayer fullscreen mode", () => {
    state.setTransition("Origami", 0.1);
    const player = new PresentationPlayer(state);
    player.startShow(true, false);

    expect(player.isOpen()).toBe(true);
    expect(player.isPresenter()).toBe(false);
    player.exitShow();
  });

  it("10. integrates 3D transitions and motion path with UnifiedShellApp ribbon actions", () => {
    const app = new UnifiedShellApp(false);
    const tabBar = app.getTabBar();
    tabBar.addTab({
      id: "tab-ppt-3d",
      title: "Deck3D.pptx",
      docType: "PowerPoint",
      isDirty: false,
    });

    app.handleRibbonAction("SetTransition", "Vortex");
    expect(slidesState.getSlideTransition().effect).toBe("Vortex");

    app.handleRibbonAction("InsertShape", "rect");
    app.handleRibbonAction("AddMotionPath", "M 0 0 C 50 100 150 100 200 0");

    const activeSlide = slidesState.getCurrentSlide();
    const anim = activeSlide?.animations?.find((a) => a.effect === "motionPath");
    expect(anim).toBeDefined();
    expect(anim?.motionPath).toBe("M 0 0 C 50 100 150 100 200 0");
  });
});
