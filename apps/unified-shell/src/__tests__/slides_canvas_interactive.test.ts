import { describe, it, expect, beforeEach } from "vitest";
import { SlidesState, slidesState } from "../state/slidesState";
import { SlideStageCanvas } from "../components/viewports/SlideStageCanvas";
import { SlideLayoutDialog } from "../components/SlideLayoutDialog";
import { UnifiedShellApp } from "../main";

describe("Interactive Slide Canvas: Drag, 8-Point Resize, Rotate & In-Place Text Editing (GEC-v2.0 Subphase 11.1.1)", () => {
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
    state.slides = [
      {
        id: "s1",
        title: "Title Slide",
        shapes: [
          {
            id: "sh1",
            type: "text",
            x: 100,
            y: 120,
            w: 760,
            h: 70,
            fill: "transparent",
            text: "OpenOfficeRS Presentation",
            textColor: "#1e293b",
            fontSize: 38,
            rotation: 0,
          },
          {
            id: "sh2",
            type: "rect",
            x: 100,
            y: 200,
            w: 360,
            h: 6,
            fill: "#c43e1c",
            rotation: 0,
          },
          {
            id: "sh3",
            type: "roundRect",
            x: 100,
            y: 240,
            w: 220,
            h: 120,
            fill: "#2563eb",
            stroke: "#1d4ed8",
            strokeWidth: 2,
            rotation: 0,
          },
          {
            id: "sh4",
            type: "ellipse",
            x: 360,
            y: 240,
            w: 140,
            h: 120,
            fill: "#10b981",
            stroke: "#059669",
            strokeWidth: 2,
            rotation: 0,
          },
          {
            id: "sh5",
            type: "star",
            x: 540,
            y: 240,
            w: 130,
            h: 120,
            fill: "#f59e0b",
            stroke: "#d97706",
            strokeWidth: 2,
            rotation: 0,
          },
        ],
      },
      {
        id: "s2",
        title: "Problem Statement",
        shapes: [
          {
            id: "sh2_1",
            type: "text",
            x: 80,
            y: 80,
            w: 800,
            h: 60,
            fill: "transparent",
            text: "Market Need & Clean-Room Architecture",
            textColor: "#1e293b",
            fontSize: 32,
          },
        ],
      },
      {
        id: "s3",
        title: "Solution & Demo",
        shapes: [],
      },
    ];
    viewportContainer = document.getElementById("viewport-container")!;
    canvasComponent = new SlideStageCanvas(viewportContainer, state);
  });

  describe("1. Selection & Transform Gizmos", () => {
    it("selects a shape when clicked inside its bounding box", () => {
      // Shape 1 is title text at x: 100, y: 120, w: 760, h: 70
      canvasComponent.simulateMouseDown(150, 150);
      expect(state.selectedShapeId).toBe("sh1");
    });

    it("deselects shape when clicking on empty canvas area", () => {
      canvasComponent.simulateMouseDown(150, 150);
      expect(state.selectedShapeId).toBe("sh1");

      // Click outside all shapes
      canvasComponent.simulateMouseDown(20, 20);
      expect(state.selectedShapeId).toBeNull();
    });

    it("calculates 8 transform handles and top rotation handle positions accurately", () => {
      const sh = state.slides[0].shapes[0]; // x: 100, y: 120, w: 760, h: 70
      const handles = canvasComponent.getHandlePositions(sh);

      expect(handles).toHaveLength(8);
      expect(handles.find((h) => h.handle === "tl")).toEqual({ handle: "tl", x: 100, y: 120 });
      expect(handles.find((h) => h.handle === "tc")).toEqual({ handle: "tc", x: 480, y: 120 });
      expect(handles.find((h) => h.handle === "tr")).toEqual({ handle: "tr", x: 860, y: 120 });
      expect(handles.find((h) => h.handle === "mr")).toEqual({ handle: "mr", x: 860, y: 155 });
      expect(handles.find((h) => h.handle === "br")).toEqual({ handle: "br", x: 860, y: 190 });
      expect(handles.find((h) => h.handle === "bc")).toEqual({ handle: "bc", x: 480, y: 190 });
      expect(handles.find((h) => h.handle === "bl")).toEqual({ handle: "bl", x: 100, y: 190 });
      expect(handles.find((h) => h.handle === "ml")).toEqual({ handle: "ml", x: 100, y: 155 });
    });

    it("renders stage wrapper and canvas properly in DOM", () => {
      const wrapper = canvasComponent.getWrapper();
      expect(wrapper).not.toBeNull();
      expect(wrapper.className).toBe("slide-stage-wrapper");
      expect(wrapper.style.width).toBe("960px");
      expect(wrapper.style.height).toBe("540px");

      const canvas = canvasComponent.getCanvas();
      expect(canvas.className).toBe("slide-stage-canvas");
    });
  });

  describe("2. Live Dragging & Smart Snapping Guides", () => {
    it("moves shape coordinates on drag and commits snapshot on mouseup", () => {
      // Select sh3 (roundRect at 100, 240, w: 220, h: 120)
      canvasComponent.simulateMouseDown(120, 260);
      expect(state.selectedShapeId).toBe("sh3");

      // Drag by +40px X, +30px Y
      canvasComponent.simulateMouseMove(160, 290);
      const sh = state.getSelectedShape()!;
      expect(sh.x).toBe(140);
      expect(sh.y).toBe(270);

      // Release mouse
      canvasComponent.simulateMouseUp();
      expect(canvasComponent.getActiveGuides()).toHaveLength(0);

      // Undo should revert
      state.undo();
      const reverted = state.slides[0].shapes.find((s) => s.id === "sh3")!;
      expect(reverted.x).toBe(100);
      expect(reverted.y).toBe(240);
    });

    it("activates vertical smart alignment guide when shape center aligns with slide center (480)", () => {
      // sh3 has w: 220. Center is at x + 110.
      // Slide center is 480. So when x = 370, center is 480.
      canvasComponent.simulateMouseDown(120, 260);
      // Move mouse close to x: 372 (within 6px threshold of 370)
      canvasComponent.simulateMouseMove(120 + (372 - 100), 260);

      const guides = canvasComponent.getActiveGuides();
      const vGuide = guides.find((g) => g.orientation === "vertical" && g.pos === 480);
      expect(vGuide).toBeDefined();

      const sh = state.getSelectedShape()!;
      // Snapped to exactly 370 so center is 480
      expect(sh.x + sh.w / 2).toBe(480);
    });

    it("activates horizontal smart alignment guide when shape center aligns with slide center (270)", () => {
      // sh3 has h: 120. Center is at y + 60.
      // Slide center is 270. So when y = 210, center is 270.
      canvasComponent.simulateMouseDown(120, 260);
      // Move close to y = 212 (within 6px threshold of 210)
      canvasComponent.simulateMouseMove(120, 260 + (212 - 240));

      const guides = canvasComponent.getActiveGuides();
      const hGuide = guides.find((g) => g.orientation === "horizontal" && g.pos === 270);
      expect(hGuide).toBeDefined();

      const sh = state.getSelectedShape()!;
      expect(sh.y + sh.h / 2).toBe(270);
    });
  });

  describe("3. 8-Point Resize Handles & Rotation", () => {
    it("resizes shape width and height by dragging bottom-right handle", () => {
      // Select sh3 at x: 100, y: 240, w: 220, h: 120 => br at x: 320, y: 360
      canvasComponent.simulateMouseDown(120, 260);
      expect(state.selectedShapeId).toBe("sh3");

      // Click on br handle at (320, 360)
      canvasComponent.simulateMouseDown(320, 360);
      // Drag br handle by +50px X, +40px Y
      canvasComponent.simulateMouseMove(370, 400);

      const sh = state.getSelectedShape()!;
      expect(sh.w).toBe(270);
      expect(sh.h).toBe(160);
      canvasComponent.simulateMouseUp();
    });

    it("preserves aspect ratio during resize when shift key is held", () => {
      // sh3 init: w: 220, h: 120 (aspect ratio: 1.833)
      canvasComponent.simulateMouseDown(120, 260);
      canvasComponent.simulateMouseDown(320, 360);
      // Drag br with shift key
      canvasComponent.simulateMouseMove(420, 380, true);

      const sh = state.getSelectedShape()!;
      const expectedScale = Math.max(320 / 220, 140 / 120);
      expect(sh.w).toBe(Math.round(220 * expectedScale));
      expect(sh.h).toBe(Math.round(120 * expectedScale));
      canvasComponent.simulateMouseUp();
    });

    it("resizes from top-left handle adjusting position and dimensions", () => {
      // sh3 at x: 100, y: 240, w: 220, h: 120 => tl handle at (100, 240)
      canvasComponent.simulateMouseDown(120, 260);
      canvasComponent.simulateMouseDown(100, 240);
      // Drag tl to the right (+20) and down (+15)
      canvasComponent.simulateMouseMove(120, 255);

      const sh = state.getSelectedShape()!;
      expect(sh.x).toBe(120);
      expect(sh.y).toBe(255);
      expect(sh.w).toBe(200);
      expect(sh.h).toBe(105);
      canvasComponent.simulateMouseUp();
    });

    it("rotates shape when dragging top rotation handle", () => {
      // sh3 cx: 210, cy: 300, rot handle at (210, 216)
      canvasComponent.simulateMouseDown(120, 260);
      // Mouse down on rotation handle
      canvasComponent.simulateMouseDown(210, 216);

      // Drag 90 degrees to the right (cx + 90, cy) => approx (300, 300)
      canvasComponent.simulateMouseMove(300, 300);

      const sh = state.getSelectedShape()!;
      expect(sh.rotation).toBeGreaterThan(0);
      canvasComponent.simulateMouseUp();
    });

    it("snaps rotation to 15-degree increments when shift key is held", () => {
      canvasComponent.simulateMouseDown(120, 260);
      canvasComponent.simulateMouseDown(210, 216);
      canvasComponent.simulateMouseMove(300, 300, true);

      const sh = state.getSelectedShape()!;
      expect(sh.rotation! % 15).toBe(0);
      canvasComponent.simulateMouseUp();
    });
  });

  describe("4. In-Place Rich-Text Editing", () => {
    it("activates inline textarea overlay on double-clicking a text box", () => {
      // Double click sh1 (title text box)
      canvasComponent.simulateDblClick(150, 150);

      const editor = canvasComponent.getInlineEditor();
      expect(editor).not.toBeNull();
      expect(editor?.className).toBe("slide-inline-text-editor");
      expect(editor?.value).toBe("OpenOfficeRS Presentation");
      expect(editor?.style.left).toBe("100px");
      expect(editor?.style.top).toBe("120px");
      expect(editor?.style.width).toBe("760px");
      expect(editor?.style.height).toBe("70px");
    });

    it("updates shape text in real time while typing in inline editor", () => {
      canvasComponent.simulateDblClick(150, 150);
      const editor = canvasComponent.getInlineEditor()!;

      editor.value = "New Modern Title";
      editor.dispatchEvent(new Event("input"));

      const sh = state.getSelectedShape()!;
      expect(sh.text).toBe("New Modern Title");
    });

    it("commits text and removes editor from DOM on blur", () => {
      canvasComponent.simulateDblClick(150, 150);
      const editor = canvasComponent.getInlineEditor()!;

      editor.value = "Finalized Title";
      editor.dispatchEvent(new Event("input"));
      editor.dispatchEvent(new Event("blur"));

      expect(canvasComponent.getInlineEditor()).toBeNull();
      const sh = state.getSelectedShape()!;
      expect(sh.text).toBe("Finalized Title");

      // Verify committed to undo stack
      expect(state.canUndo()).toBe(true);
    });

    it("commits text and removes editor on Escape key", () => {
      canvasComponent.simulateDblClick(150, 150);
      const editor = canvasComponent.getInlineEditor()!;

      editor.value = "Escaped Title";
      editor.dispatchEvent(new Event("input"));
      editor.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));

      expect(canvasComponent.getInlineEditor()).toBeNull();
      const sh = state.getSelectedShape()!;
      expect(sh.text).toBe("Escaped Title");
    });

    it("allows adding and editing text on non-text shapes (e.g. roundRect)", () => {
      // Double click sh3 (roundRect)
      canvasComponent.simulateDblClick(120, 260);

      const editor = canvasComponent.getInlineEditor();
      expect(editor).not.toBeNull();
      editor!.value = "Card Description";
      editor!.dispatchEvent(new Event("input"));
      editor!.dispatchEvent(new Event("blur"));

      const sh = state.slides[0].shapes.find((s) => s.id === "sh3")!;
      expect(sh.text).toBe("Card Description");
    });
  });

  describe("5. Slide Layouts & Slide Management", () => {
    it("adds slide with 'title' layout having title and subtitle placeholders", () => {
      const newSlide = state.addSlideWithLayout("title", "Project Kickoff");
      expect(newSlide.shapes).toHaveLength(2);
      expect(newSlide.shapes[0].text).toBe("Project Kickoff");
      expect(newSlide.shapes[0].fontSize).toBe(40);
      expect(newSlide.shapes[1].text).toBe("Click to add subtitle");
      expect(newSlide.shapes[1].fontSize).toBe(22);
    });

    it("adds slide with 'titleAndContent' layout having title and bulleted body", () => {
      const newSlide = state.addSlideWithLayout("titleAndContent", "Agenda");
      expect(newSlide.shapes).toHaveLength(2);
      expect(newSlide.shapes[0].text).toBe("Agenda");
      expect(newSlide.shapes[1].text).toContain("• First bullet point");
    });

    it("adds slide with 'twoContent' layout having title and two column boxes", () => {
      const newSlide = state.addSlideWithLayout("twoContent", "Comparison");
      expect(newSlide.shapes).toHaveLength(3);
      expect(newSlide.shapes[0].text).toBe("Comparison");
      expect(newSlide.shapes[1].x).toBe(80);
      expect(newSlide.shapes[2].x).toBe(500);
    });

    it("adds slide with 'blank' layout having zero shapes", () => {
      const newSlide = state.addSlideWithLayout("blank", "Empty Canvas");
      expect(newSlide.shapes).toHaveLength(0);
    });

    it("applies layout to existing slide while preserving slide title", () => {
      state.selectSlide(1); // Slide 2: "Problem Statement"
      const applied = state.applyLayout("twoContent");
      expect(applied).toBe(true);

      const slide = state.slides[1];
      expect(slide.shapes).toHaveLength(3);
      expect(slide.shapes[0].text).toContain("Market Need & Clean-Room Architecture");
    });

    it("reorders slides with moveSlideUp and moveSlideDown", () => {
      expect(state.slides[0].title).toBe("Title Slide");
      expect(state.slides[1].title).toBe("Problem Statement");

      // Move Slide 2 up to position 0
      state.selectSlide(1);
      const movedUp = state.moveSlideUp();
      expect(movedUp).toBe(true);
      expect(state.slides[0].title).toBe("Problem Statement");
      expect(state.slides[1].title).toBe("Title Slide");

      // Move it down back to position 1
      const movedDown = state.moveSlideDown();
      expect(movedDown).toBe(true);
      expect(state.slides[0].title).toBe("Title Slide");
      expect(state.slides[1].title).toBe("Problem Statement");
    });

    it("opens SlideLayoutDialog with 4 visual cards and triggers callback", () => {
      const dialog = new SlideLayoutDialog();
      let chosenLayout: string | null = null;

      dialog.open((layout) => {
        chosenLayout = layout;
      });

      expect(dialog.isOpen()).toBe(true);
      const cards = dialog.getLayoutCards();
      expect(cards.length).toBeGreaterThanOrEqual(4);

      // Click "Two Content" card
      const twoContentCard = cards.find((c) => c.getAttribute("data-layout") === "twoContent");
      expect(twoContentCard).toBeDefined();
      twoContentCard?.click();

      expect(chosenLayout).toBe("twoContent");
      expect(dialog.isOpen()).toBe(false);
    });
  });

  describe("6. UnifiedShellApp Ribbon & Keyboard Integration for Slides", () => {
    let app: UnifiedShellApp;

    beforeEach(async () => {
      app = new UnifiedShellApp(false);
      app.getTabBar().addTab({
        id: "tab-slides-1",
        docType: "PowerPoint",
        title: "Presentation1.pptx",
        isDirty: false,
      });
      app.getTabBar().switchTab("tab-slides-1");
    });

    it("handles PowerPoint ribbon commands: NewSlide, DuplicateSlide, MoveSlideUp, MoveSlideDown, DeleteSlide", () => {
      const initialCount = slidesState.slides.length;

      app.handleRibbonAction("NewSlide");
      expect(slidesState.slides.length).toBe(initialCount + 1);

      app.handleRibbonAction("DuplicateSlide");
      expect(slidesState.slides.length).toBe(initialCount + 2);

      app.handleRibbonAction("MoveSlideUp");
      expect(slidesState.currentSlideIndex).toBe(slidesState.slides.length - 2);

      app.handleRibbonAction("MoveSlideDown");
      expect(slidesState.currentSlideIndex).toBe(slidesState.slides.length - 1);

      app.handleRibbonAction("DeleteSlide");
      expect(slidesState.slides.length).toBe(initialCount + 1);
    });

    it("opens SlideLayoutDialog via SlideLayout ribbon action and applies selection", () => {
      const dialog = app.getSlideLayoutDialog();
      expect(dialog.isOpen()).toBe(false);

      app.handleRibbonAction("SlideLayout");
      expect(dialog.isOpen()).toBe(true);

      const cards = dialog.getLayoutCards();
      const blankCard = cards.find((c) => c.getAttribute("data-layout") === "blank");
      blankCard?.click();

      expect(dialog.isOpen()).toBe(false);
      const currentSlide = slidesState.slides[slidesState.currentSlideIndex];
      expect(currentSlide.shapes).toHaveLength(0);
    });

    it("formats selected shape text via ribbon: bold, italic, underline, font size, font family", () => {
      // Add a text box
      slidesState.addShape("text", 100, 100, 200, 50);
      const sh = slidesState.getSelectedShape()!;

      app.handleRibbonAction("ToggleBold");
      expect(sh.bold).toBe(true);

      app.handleRibbonAction("ToggleItalic");
      expect(sh.italic).toBe(true);

      app.handleRibbonAction("ToggleUnderline");
      expect(sh.underline).toBe(true);

      app.handleRibbonAction("SetFontSize", 36);
      expect(sh.fontSize).toBe(36);

      app.handleRibbonAction("SetFontFamily", "Segoe UI");
      expect(sh.fontFamily).toBe("Segoe UI");

      app.handleRibbonAction("SetFontColor", "#2563eb");
      expect(sh.textColor).toBe("#2563eb");
    });

    it("supports SlideSorter thumbnail list and Move Up/Down actions", () => {
      const sorter = app.getSlideSorter();
      expect(sorter).toBeDefined();

      const upBtns = document.querySelectorAll(".slide-thumb-actions button[title='Move Slide Up']");
      expect(upBtns.length).toBeGreaterThan(0);

      const downBtns = document.querySelectorAll(".slide-thumb-actions button[title='Move Slide Down']");
      expect(downBtns.length).toBeGreaterThan(0);
    });
  });
});
