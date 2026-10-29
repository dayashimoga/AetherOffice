import { describe, it, expect, beforeEach } from "vitest";
import { SlidesState, slidesState } from "../state/slidesState";
import { SlideStageCanvas } from "../components/viewports/SlideStageCanvas";
import { SpeakerNotesPane } from "../components/SpeakerNotesPane";
import { FormatBackgroundDialog } from "../components/FormatBackgroundDialog";
import { UnifiedShellApp } from "../main";

describe("Slides Home, Insert & Design Ribbons: Shapes, Images, Tables, Themes & Backgrounds (GEC-v2.0 Subphase 11.1.2)", () => {
  let state: SlidesState;
  let canvasComponent: SlideStageCanvas;
  let notesPane: SpeakerNotesPane;
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
        <div id="reflow-reader-container"></div>
        <div id="row-card-inspector-container"></div>
        <div id="laser-pointer-dot"></div>
      </div>
    `;

    state = new SlidesState();
    viewportContainer = document.getElementById("viewport-container")!;
    canvasComponent = new SlideStageCanvas(viewportContainer, state);
    notesPane = new SpeakerNotesPane(viewportContainer, state);
  });

  describe("1. Shape, Image & Table Insertions", () => {
    it("inserts all preset shapes: rect, roundRect, ellipse, triangle, arrow, star, callout, diamond, heart", () => {
      const shapes = [
        "rect",
        "roundRect",
        "ellipse",
        "triangle",
        "arrow",
        "star",
        "callout",
        "diamond",
        "heart",
      ] as const;

      shapes.forEach((type) => {
        const sh = state.addShape(type, 150, 150, 120, 80);
        expect(sh.type).toBe(type);
        expect(state.selectedShapeId).toBe(sh.id);
      });
      canvasComponent.render();
    });

    it("inserts Image shape with URL and alt metadata and renders image card", () => {
      const img = state.insertImage("https://example.com/slide-chart.png", 100, 100, 300, 200);
      expect(img.type).toBe("image");
      expect(img.imageUrl).toBe("https://example.com/slide-chart.png");
      expect(img.imageAlt).toBe("Presentation Graphic");
      expect(state.selectedShapeId).toBe(img.id);

      canvasComponent.render();
    });

    it("inserts Table shape with headers and row-major cell matrix", () => {
      const tbl = state.insertTable(3, 4, 80, 80, 600, 200);
      expect(tbl.type).toBe("table");
      expect(tbl.tableHeaders).toHaveLength(4);
      expect(tbl.tableData).toHaveLength(3);
      expect(tbl.tableData![0]).toHaveLength(4);
      expect(tbl.tableData![0][0]).toBe("R1C1");

      canvasComponent.render();
    });

    it("updates shape fill color and outline weight via commands", () => {
      const sh = state.addShape("rect", 100, 100, 200, 100);
      expect(state.setShapeFill("#10b981")).toBe(true);
      expect(sh.fill).toBe("#10b981");

      expect(state.setShapeStroke("#c43e1c", 4)).toBe(true);
      expect(sh.stroke).toBe("#c43e1c");
      expect(sh.strokeWidth).toBe(4);

      expect(state.setShapeStrokeWeight(6)).toBe(true);
      expect(sh.strokeWidth).toBe(6);
    });
  });

  describe("2. Arrange & Align Commands", () => {
    it("brings shape to front and sends shape to back in Z-order", () => {
      const slide = state.slides[0];
      const firstShapeId = slide.shapes[0].id;

      // Select first shape and bring to front
      state.selectShape(firstShapeId);
      expect(state.bringToFront()).toBe(true);
      expect(slide.shapes[slide.shapes.length - 1].id).toBe(firstShapeId);

      // Send it back to the first position
      expect(state.sendToBack()).toBe(true);
      expect(slide.shapes[0].id).toBe(firstShapeId);
    });

    it("aligns selected shape horizontally: left, center, right", () => {
      const sh = state.addShape("rect", 200, 100, 160, 80);

      state.alignSelectedShape("left");
      expect(sh.x).toBe(80);

      state.alignSelectedShape("center");
      expect(sh.x).toBe(Math.round((960 - 160) / 2)); // 400

      state.alignSelectedShape("right");
      expect(sh.x).toBe(960 - 80 - 160); // 720
    });

    it("aligns selected shape vertically: top, middle, bottom", () => {
      const sh = state.addShape("rect", 200, 100, 160, 80);

      state.alignSelectedShape("top");
      expect(sh.y).toBe(50);

      state.alignSelectedShape("middle");
      expect(sh.y).toBe(Math.round((540 - 80) / 2)); // 230

      state.alignSelectedShape("bottom");
      expect(sh.y).toBe(540 - 50 - 80); // 410
    });
  });

  describe("3. Design Ribbon: 8 Built-in Themes", () => {
    const themeNames = [
      "OfficeDefault",
      "DarkMode",
      "VibrantCoral",
      "HighContrast",
      "EmeraldForest",
      "OceanBreeze",
      "SunsetGlow",
      "SlateMinimal",
    ] as const;

    themeNames.forEach((theme) => {
      it(`applies built-in theme '${theme}' across all presentation slides`, () => {
        state.setTheme(theme);
        expect(state.currentTheme).toBe(theme);

        const slide = state.slides[0];
        expect(slide.background).toBeDefined();

        const textShape = slide.shapes.find((s) => s.type === "text")!;
        expect(textShape.textColor).toBeDefined();
        expect(textShape.fontFamily).toBeDefined();

        canvasComponent.render();
      });
    });
  });

  describe("4. Slide Sizing & Format Background", () => {
    it("switches between 16:9 widescreen and 4:3 standard dimensions", () => {
      expect(state.slideAspectRatio).toBe("16:9");
      expect(state.slideWidth).toBe(960);
      expect(state.slideHeight).toBe(540);

      state.setSlideSize("4:3");
      expect(state.slideAspectRatio).toBe("4:3");
      expect(state.slideWidth).toBe(720);
      expect(state.slideHeight).toBe(540);

      canvasComponent.render();
      expect(canvasComponent.getCanvas().style.width).toBe("720px");

      state.setSlideSize("16:9");
      expect(state.slideWidth).toBe(960);
      canvasComponent.render();
      expect(canvasComponent.getCanvas().style.width).toBe("960px");
    });

    it("applies background solid color or gradient to current slide or all slides", () => {
      state.setSlideBackground("#f0fdf4", false);
      expect(state.slides[0].background).toBe("#f0fdf4");
      expect(state.slides[1].background).toBeUndefined();

      state.setSlideBackground("linear-gradient(135deg, #1e293b, #0f172a)", true);
      expect(state.slides[0].background).toContain("linear-gradient");
      expect(state.slides[1].background).toContain("linear-gradient");

      canvasComponent.render();
    });

    it("opens FormatBackgroundDialog with solid and gradient swatches and executes callbacks", () => {
      const dialog = new FormatBackgroundDialog();
      let appliedBg = "";
      let appliedAll = false;

      dialog.open((bg, applyToAll) => {
        appliedBg = bg;
        appliedAll = applyToAll;
      });

      expect(dialog.isOpen()).toBe(true);
      expect(dialog.getSolidSwatches().length).toBeGreaterThan(0);
      expect(dialog.getGradientSwatches().length).toBeGreaterThan(0);

      // Click second solid swatch (Light Slate #f8fafc)
      const slateSwatch = dialog.getSolidSwatches()[1];
      slateSwatch.click();

      const applyBtn = document.querySelector(".btn-apply-bg") as HTMLButtonElement;
      applyBtn.click();

      expect(dialog.isOpen()).toBe(false);
      expect(appliedBg).toBe("#f8fafc");
      expect(appliedAll).toBe(false);
    });
  });

  describe("5. Speaker Notes Pane", () => {
    it("renders SpeakerNotesPane below stage canvas and reflects current slide notes", () => {
      expect(notesPane.getRoot().className).toBe("speaker-notes-pane");
      expect(notesPane.getTextarea()).toBeDefined();

      state.setSlideNotes("Key speaker talking point: sub-millisecond Piece-Tree.", 0);
      notesPane.refresh();
      expect(notesPane.getTextarea().value).toBe("Key speaker talking point: sub-millisecond Piece-Tree.");
    });

    it("updates slide notes on textarea input", () => {
      const textarea = notesPane.getTextarea();
      textarea.value = "Audience Q&A: Discuss clean-room reverse engineering.";
      textarea.dispatchEvent(new Event("input"));

      expect(state.getSlideNotes(0)).toBe("Audience Q&A: Discuss clean-room reverse engineering.");
    });

    it("toggles collapse and expand on header click", () => {
      expect(notesPane.isCollapsed()).toBe(false);

      const header = notesPane.getRoot().querySelector(".speaker-notes-header") as HTMLElement;
      header.click();
      expect(notesPane.isCollapsed()).toBe(true);

      header.click();
      expect(notesPane.isCollapsed()).toBe(false);
    });

    it("refreshes speaker notes when switching between slides", () => {
      state.setSlideNotes("Slide 1 Notes", 0);
      state.setSlideNotes("Slide 2 Notes", 1);

      state.selectSlide(0);
      notesPane.refresh();
      expect(notesPane.getTextarea().value).toBe("Slide 1 Notes");

      state.selectSlide(1);
      notesPane.refresh();
      expect(notesPane.getTextarea().value).toBe("Slide 2 Notes");
    });
  });

  describe("6. UnifiedShellApp Ribbon Wiring for Slides Home, Insert & Design", () => {
    let app: UnifiedShellApp;

    beforeEach(async () => {
      app = new UnifiedShellApp(false);
      app.getTabBar().addTab({
        id: "tab-slides-1",
        docType: "PowerPoint",
        title: "Deck.pptx",
        isDirty: false,
      });
      app.getTabBar().switchTab("tab-slides-1");
    });

    it("handles Home & Insert ribbon actions: InsertShape, InsertPicture, InsertTable, AlignCenter", () => {
      const initialCount = slidesState.slides[slidesState.currentSlideIndex].shapes.length;

      app.handleRibbonAction("InsertShape", "heart");
      expect(slidesState.slides[slidesState.currentSlideIndex].shapes.length).toBe(initialCount + 1);
      expect(slidesState.getSelectedShape()?.type).toBe("heart");

      app.handleRibbonAction("InsertPicture");
      expect(slidesState.getSelectedShape()?.type).toBe("image");

      app.handleRibbonAction("InsertTable", { rows: 2, cols: 2 });
      expect(slidesState.getSelectedShape()?.type).toBe("table");

      app.handleRibbonAction("AlignCenter");
      const sh = slidesState.getSelectedShape()!;
      expect(sh.x).toBe(Math.round((slidesState.slideWidth - sh.w) / 2));
    });

    it("handles Design ribbon actions: SetTheme, SetSlideSize, FormatBackground", () => {
      app.handleRibbonAction("SetTheme", "EmeraldForest");
      expect(slidesState.currentTheme).toBe("EmeraldForest");

      app.handleRibbonAction("SetSlideSize", "4:3");
      expect(slidesState.slideAspectRatio).toBe("4:3");
      expect(slidesState.slideWidth).toBe(720);

      const bgDialog = app.getFormatBackgroundDialog();
      expect(bgDialog.isOpen()).toBe(false);

      app.handleRibbonAction("FormatBackground");
      expect(bgDialog.isOpen()).toBe(true);

      const applyAllBtn = document.querySelector(".btn-apply-all-bg") as HTMLButtonElement;
      applyAllBtn.click();
      expect(bgDialog.isOpen()).toBe(false);
    });
  });
});

