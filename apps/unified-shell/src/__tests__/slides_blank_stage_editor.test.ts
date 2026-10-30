import { describe, it, expect, beforeEach } from "vitest";
import { SlidesState, slidesState } from "../state/slidesState";
import { SlideStageCanvas } from "../components/viewports/SlideStageCanvas";
import { SlideSorter } from "../components/SlideSorter";
import { UnifiedShellApp } from "../main";

describe("Blank Editable Slides, New Slide Layouts, Drag/Resize/Rotate & Inline Editing (GEC-v3.0 Subphase 24.1.1)", () => {
  let state: SlidesState;
  let canvasComponent: SlideStageCanvas;
  let viewportContainer: HTMLElement;

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
        <div id="reflow-reader-container"></div>
        <div id="row-card-inspector-container"></div>
        <div id="laser-pointer-dot"></div>
      </div>
    `;

    state = new SlidesState();
    viewportContainer = document.getElementById("viewport-container")!;
    canvasComponent = new SlideStageCanvas(viewportContainer, state);
  });

  describe("1. Clean Blank Presentation Initialization (GEC-v3.0 Zero Fake Text)", () => {
    it("starts with exactly 1 clean blank Title Slide and zero prepopulated demo text", () => {
      expect(state.slides).toHaveLength(1);
      const firstSlide = state.slides[0];
      expect(firstSlide.title).toBe("Title Slide");
      expect(firstSlide.layout).toBe("title");
      expect(firstSlide.shapes).toHaveLength(2);

      const titleShape = firstSlide.shapes[0];
      expect(titleShape.type).toBe("text");
      expect(titleShape.text).toBe("Click to add title");
      expect(titleShape.fontSize).toBe(40);
      expect(titleShape.bold).toBe(true);

      const subShape = firstSlide.shapes[1];
      expect(subShape.type).toBe("text");
      expect(subShape.text).toBe("Click to add subtitle");
      expect(subShape.fontSize).toBe(22);
    });

    it("resets state to clean blank presentation via resetToBlank", () => {
      // Mutate state with extra slides and shapes
      state.addSlide("Extra Slide 1");
      state.addSlide("Extra Slide 2");
      state.addShape("star", 150, 150, 100, 100);
      expect(state.slides.length).toBe(3);

      state.resetToBlank("NewDeck.pptx");
      expect(state.slides).toHaveLength(1);
      expect(state.currentSlideIndex).toBe(0);
      expect(state.slides[0].shapes).toHaveLength(2);
      expect(state.slides[0].shapes[0].text).toBe("Click to add title");
      expect(state.slides[0].shapes[1].text).toBe("Click to add subtitle");
      expect(state.selectedShapeId).toBeNull();
    });
  });

  describe("2. Inline Text Editing & Typing on 16:9 Canvas", () => {
    it("activates inline textarea editor on clicking text placeholder box", () => {
      // Click on title placeholder at (150, 180) - within sh1 (100, 150, 760, 80)
      canvasComponent.simulateClick(150, 180);

      const editor = canvasComponent.getInlineEditor();
      expect(editor).not.toBeNull();
      expect(editor?.className).toBe("slide-inline-text-editor");
      expect(editor?.value).toBe("Click to add title");
      expect(state.selectedShapeId).toBe("sh1");
    });

    it("updates placeholder text in real time while typing in inline editor", () => {
      canvasComponent.simulateClick(150, 180);
      const editor = canvasComponent.getInlineEditor()!;

      editor.value = "Strategic Quarterly Roadmap";
      editor.dispatchEvent(new Event("input"));

      const shape = state.getSelectedShape()!;
      expect(shape.text).toBe("Strategic Quarterly Roadmap");
    });

    it("commits typed text to undo history on blur and cleans up editor", () => {
      canvasComponent.simulateClick(150, 180);
      const editor = canvasComponent.getInlineEditor()!;

      editor.value = "Executive Keynote 2026";
      editor.dispatchEvent(new Event("input"));
      editor.dispatchEvent(new Event("blur"));

      expect(canvasComponent.getInlineEditor()).toBeNull();
      expect(state.slides[0].shapes[0].text).toBe("Executive Keynote 2026");
      expect(state.canUndo()).toBe(true);

      // Undo reverts text back
      state.undo();
      expect(state.slides[0].shapes[0].text).toBe("Click to add title");
    });

    it("commits typed text and closes editor on Escape key", () => {
      canvasComponent.simulateClick(150, 180);
      const editor = canvasComponent.getInlineEditor()!;

      editor.value = "Product Vision";
      editor.dispatchEvent(new Event("input"));
      editor.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));

      expect(canvasComponent.getInlineEditor()).toBeNull();
      expect(state.slides[0].shapes[0].text).toBe("Product Vision");
    });

    it("activates inline editing directly from canvas keystrokes on selected shape", () => {
      // Select title shape via mousedown without drag
      canvasComponent.simulateMouseDown(150, 180);
      canvasComponent.simulateMouseUp();
      expect(state.selectedShapeId).toBe("sh1");

      // Press a character key on canvas
      const canvas = canvasComponent.getCanvas();
      canvas.dispatchEvent(new KeyboardEvent("keydown", { key: "W" }));

      const editor = canvasComponent.getInlineEditor();
      expect(editor).not.toBeNull();
      expect(editor?.value).toBe("W");
      expect(state.getSelectedShape()?.text).toBe("W");
    });
  });

  describe("3. Interactive Drag, 8-Point Resize, Rotate & Delete", () => {
    it("displays 8 resize handles and top rotation handle on shape selection", () => {
      canvasComponent.simulateMouseDown(150, 180);
      expect(state.selectedShapeId).toBe("sh1");

      const sh = state.getSelectedShape()!;
      const handles = canvasComponent.getHandlePositions(sh);
      expect(handles).toHaveLength(8);

      const handlesNames = handles.map((h) => h.handle);
      expect(handlesNames).toEqual(["tl", "tc", "tr", "mr", "br", "bc", "bl", "ml"]);

      // Verify rotation handle stalk coordinates
      const rotCx = sh.x + sh.w / 2;
      const rotCy = sh.y - 24;
      expect(rotCx).toBe(100 + 760 / 2);
      expect(rotCy).toBe(150 - 24);
    });

    it("moves shape coordinates on drag and commits snapshot", () => {
      // Add a rectangle shape at (200, 200, 160, 100)
      const rect = state.addShape("rect", 200, 200, 160, 100);
      state.selectShape(rect.id);

      // Start drag at (220, 220)
      canvasComponent.simulateMouseDown(220, 220);
      // Move by +50 X, +30 Y
      canvasComponent.simulateMouseMove(270, 250);

      expect(rect.x).toBe(250);
      expect(rect.y).toBe(230);

      canvasComponent.simulateMouseUp();
      expect(rect.x).toBe(250);
      expect(rect.y).toBe(230);

      // Undo reverts position
      state.undo();
      expect(state.slides[0].shapes.find((s) => s.id === rect.id)?.x).toBe(200);
    });

    it("resizes shape dimensions from bottom-right (br) handle", () => {
      const rect = state.addShape("rect", 100, 100, 200, 100);
      state.selectShape(rect.id);

      // Bottom-right handle is at (300, 200)
      canvasComponent.simulateMouseDown(300, 200);
      // Drag handle by +40 X, +20 Y
      canvasComponent.simulateMouseMove(340, 220);

      expect(rect.w).toBe(240);
      expect(rect.h).toBe(120);
      canvasComponent.simulateMouseUp();
    });

    it("resizes shape from top-left (tl) handle adjusting both origin and size", () => {
      const rect = state.addShape("rect", 100, 100, 200, 100);
      state.selectShape(rect.id);

      // Top-left handle is at (100, 100)
      canvasComponent.simulateMouseDown(100, 100);
      // Move right +20, down +10
      canvasComponent.simulateMouseMove(120, 110);

      expect(rect.x).toBe(120);
      expect(rect.y).toBe(110);
      expect(rect.w).toBe(180);
      expect(rect.h).toBe(90);
      canvasComponent.simulateMouseUp();
    });

    it("rotates shape by dragging top rotation handle", () => {
      const rect = state.addShape("roundRect", 200, 200, 100, 100);
      state.selectShape(rect.id);

      // Center is (250, 250). Rotation handle is at (250, 176)
      canvasComponent.simulateMouseDown(250, 176);
      // Drag 90 degrees right to approx (324, 250)
      canvasComponent.simulateMouseMove(324, 250);

      expect(rect.rotation).toBeGreaterThan(0);
      canvasComponent.simulateMouseUp();
    });

    it("removes selected shape when pressing Delete or Backspace key", () => {
      const star = state.addShape("star", 300, 200, 120, 120);
      state.selectShape(star.id);
      expect(state.slides[0].shapes).toHaveLength(3);

      const canvas = canvasComponent.getCanvas();
      canvas.dispatchEvent(new KeyboardEvent("keydown", { key: "Delete" }));

      expect(state.slides[0].shapes).toHaveLength(2);
      expect(state.getSelectedShape()).toBeNull();

      // Undo restores shape
      state.undo();
      expect(state.slides[0].shapes).toHaveLength(3);
    });
  });

  describe("4. Slide Thumbnail Rail (+ New Slide with Layout Choices, Duplicate, Move, Delete)", () => {
    let sorter: SlideSorter;
    let sidebarContainer: HTMLElement;

    beforeEach(() => {
      sidebarContainer = document.getElementById("sidebar-container")!;
      sorter = new SlideSorter("sidebar-container");
    });

    it("renders + New Slide button and slide count badge in sidebar rail", () => {
      sorter.setSlides(
        state.slides.map((s, idx) => ({
          id: idx + 1,
          slideNumber: idx + 1,
          title: s.title,
          shapeCount: s.shapes.length,
        }))
      );

      const countBadge = sidebarContainer.querySelector(".slide-count-badge");
      expect(countBadge?.textContent).toBe("Slides (1)");

      const newSlideBtn = sidebarContainer.querySelector('[data-testid="btn-new-slide"]');
      expect(newSlideBtn).not.toBeNull();
      expect(newSlideBtn?.textContent).toContain("+ New Slide");
    });

    it("toggles slide layout menu with 4 standard layouts on dropdown click", () => {
      const dropdownBtn = sidebarContainer.querySelector<HTMLButtonElement>(
        '[data-testid="btn-layout-dropdown"]'
      );
      expect(dropdownBtn).not.toBeNull();
      dropdownBtn?.click();

      const layoutMenu = sidebarContainer.querySelector('[data-testid="slide-sorter-layout-menu"]');
      expect(layoutMenu).not.toBeNull();

      const items = sidebarContainer.querySelectorAll(".slide-layout-menu-item");
      expect(items).toHaveLength(4);
      expect(items[0].getAttribute("data-layout")).toBe("title");
      expect(items[1].getAttribute("data-layout")).toBe("titleAndContent");
      expect(items[2].getAttribute("data-layout")).toBe("twoContent");
      expect(items[3].getAttribute("data-layout")).toBe("blank");
    });

    it("adds slide with 'titleAndContent' layout having title and bulleted body", () => {
      let addedLayout: string | null = null;
      sorter.setCallbacks({
        onAddSlideWithLayout: (layout) => {
          addedLayout = layout;
          state.addSlideWithLayout(layout, "Key Findings");
        },
      });

      const dropdownBtn = sidebarContainer.querySelector<HTMLButtonElement>(
        '[data-testid="btn-layout-dropdown"]'
      );
      dropdownBtn?.click();

      const tcBtn = sidebarContainer.querySelector<HTMLButtonElement>(
        '[data-testid="slide-layout-item-titleAndContent"]'
      );
      tcBtn?.click();

      expect(addedLayout).toBe("titleAndContent");
      expect(state.slides).toHaveLength(2);
      const newSlide = state.slides[1];
      expect(newSlide.shapes).toHaveLength(2);
      expect(newSlide.shapes[0].text).toBe("Key Findings");
      expect(newSlide.shapes[1].text).toContain("• First bullet point");
    });

    it("adds slide with 'twoContent' layout having title and two column placeholders", () => {
      const slide = state.addSlideWithLayout("twoContent", "Market Comparison");
      expect(slide.shapes).toHaveLength(3);
      expect(slide.shapes[0].text).toBe("Market Comparison");
      expect(slide.shapes[1].x).toBe(80);
      expect(slide.shapes[2].x).toBe(500);
    });

    it("adds slide with 'blank' layout having zero predefined shapes", () => {
      const slide = state.addSlideWithLayout("blank", "Custom Infographic");
      expect(slide.shapes).toHaveLength(0);
    });

    it("executes slide thumbnail actions: Duplicate, Move Up, Move Down, and Delete", () => {
      state.addSlide("Slide 2");
      state.addSlide("Slide 3");
      expect(state.slides).toHaveLength(3);

      sorter.setSlides(
        state.slides.map((s, idx) => ({
          id: idx + 1,
          slideNumber: idx + 1,
          title: s.title,
          shapeCount: s.shapes.length,
        }))
      );

      sorter.setCallbacks({
        onDuplicateSlide: (idx) => {
          state.duplicateSlide(idx);
        },
        onDeleteSlide: (idx) => {
          state.deleteSlide(idx);
        },
        onMoveUp: (idx) => {
          state.moveSlideUp(idx);
        },
        onMoveDown: (idx) => {
          state.moveSlideDown(idx);
        },
      });

      // Duplicate slide 1
      const dupBtn = sidebarContainer.querySelector<HTMLButtonElement>('[data-testid="btn-duplicate-0"]');
      dupBtn?.click();
      expect(state.slides).toHaveLength(4);
      expect(state.slides[1].title).toContain("(Copy)");

      // Move Slide 2 up to position 0
      state.selectSlide(1);
      state.moveSlideUp();
      expect(state.slides[0].title).toContain("(Copy)");

      // Move Slide 0 down to position 1
      state.moveSlideDown();
      expect(state.slides[1].title).toContain("(Copy)");

      // Delete slide
      state.deleteSlide(1);
      expect(state.slides).toHaveLength(3);
    });
  });

  describe("5. UnifiedShellApp PowerPoint Integration & Status Bar", () => {
    let app: UnifiedShellApp;

    beforeEach(() => {
      app = new UnifiedShellApp(false);
      slidesState.resetToBlank("Presentation1.pptx");
      app.getTabBar().addTab({
        id: "tab-slides-e2e",
        docType: "PowerPoint",
        title: "Presentation1.pptx",
        isDirty: false,
      });
      app.getTabBar().switchTab("tab-slides-e2e");
    });

    it("displays clean blank Title Slide and dynamic status bar 'Slide 1 of 1'", () => {
      const docInfo = document.getElementById("status-doc-info");
      expect(docInfo?.textContent).toBe("Slide 1 of 1");
      expect(slidesState.slides).toHaveLength(1);
      expect(slidesState.slides[0].shapes).toHaveLength(2);
      expect(slidesState.slides[0].shapes[0].text).toBe("Click to add title");
    });

    it("resets PowerPoint to blank when opening a new PowerPoint tab via handleNewTab", () => {
      // Add extra slide to current presentation
      slidesState.addSlide("Old Slide");
      expect(slidesState.slides).toHaveLength(2);

      // Open new PowerPoint tab
      app.handleNewTab("PowerPoint");
      expect(slidesState.slides).toHaveLength(1);
      expect(slidesState.slides[0].shapes[0].text).toBe("Click to add title");
    });

    it("wires SlideSorter sidebar callbacks in UnifiedShellApp for adding slides with layouts", () => {
      const sorter = app.getSlideSorter();
      expect(sorter).toBeDefined();

      const newSlideBtn = document.querySelector<HTMLButtonElement>('[data-testid="btn-new-slide"]');
      newSlideBtn?.click();

      expect(slidesState.slides).toHaveLength(2);
      const docInfo = document.getElementById("status-doc-info");
      expect(docInfo?.textContent).toBe("Slide 2 of 2");
    });
  });
});
