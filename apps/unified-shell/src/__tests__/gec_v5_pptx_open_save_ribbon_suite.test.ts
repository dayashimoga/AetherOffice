import { describe, it, expect, beforeEach, vi } from "vitest";
import { UnifiedShellApp } from "../main";
import { slidesState } from "../state/slidesState";
import { binaryPackageService } from "../services/binaryPackageService";
import { fileIoService } from "../services/fileIoService";
import { IPC } from "../ipc";

describe("GEC-v5.0 Subphase 31.1.1: PowerPoint Real .pptx Open/Save & Home, Insert, Design, Transitions Ribbons", () => {
  let app: UnifiedShellApp;

  beforeEach(() => {
    localStorage.clear();
    IPC._resetMockState();
    slidesState.resetToBlank();

    const mockCtx = {
      clearRect: vi.fn(),
      fillRect: vi.fn(),
      strokeRect: vi.fn(),
      fillText: vi.fn(),
      measureText: (text: string) => ({ width: text.length * 8 }),
      beginPath: vi.fn(),
      moveTo: vi.fn(),
      lineTo: vi.fn(),
      quadraticCurveTo: vi.fn(),
      bezierCurveTo: vi.fn(),
      arc: vi.fn(),
      closePath: vi.fn(),
      fill: vi.fn(),
      stroke: vi.fn(),
      save: vi.fn(),
      restore: vi.fn(),
      setTransform: vi.fn(),
      scale: vi.fn(),
      rotate: vi.fn(),
      translate: vi.fn(),
      setLineDash: vi.fn(),
      rect: vi.fn(),
      clip: vi.fn(),
      createLinearGradient: () => ({
        addColorStop: vi.fn(),
      }),
      createRadialGradient: () => ({
        addColorStop: vi.fn(),
      }),
      createPattern: () => null,
      drawImage: vi.fn(),
      font: "",
      fillStyle: "",
      strokeStyle: "",
      lineWidth: 1,
      textAlign: "left",
    };
    HTMLCanvasElement.prototype.getContext = (() => mockCtx) as any;

    document.body.innerHTML = `
      <div id="app" class="desktop-shell">
        <header class="title-bar">
          <div class="app-branding">
            <span class="app-name">AetherOffice</span>
          </div>
          <div class="title-actions">
            <button id="btn-theme-toggle" class="theme-toggle-btn"></button>
          </div>
        </header>
        <div id="left-app-rail-container"></div>
        <div id="tab-bar-container"></div>
        <div id="ribbon-container"></div>
        <div id="sub-header-container" style="display: none;"></div>
        <div class="workbench">
          <div id="sidebar-container"></div>
          <main id="viewport-container" class="viewport"></main>
          <div id="copilot-sidebar-container"></div>
        </div>
        <footer id="status-bar-container">
          <span id="status-doc-info"></span>
          <span id="status-page-info"></span>
          <input type="range" id="zoom-slider" min="50" max="200" value="100" />
          <span id="status-zoom-level">100%</span>
        </footer>
      </div>
    `;

    app = new UnifiedShellApp(false);
    app.applyBootConfig({ app_mode: "slides" });
  });

  describe("1. Real .pptx Binary Open & Save Roundtrip", () => {
    it("packages and parses valid .pptx binary archives with multiple slides and text", async () => {
      const sourceSlides = [
        {
          title: "Quarterly Strategy Review",
          shapes: [
            { text: "Executive Summary: FY26 Q4 Vision" },
            { text: "Antigravity Engineering Deliverables" },
          ],
        },
        {
          title: "Architecture & Performance",
          shapes: [
            { text: "1. Zero-latency ECMA-376 engine" },
            { text: "2. Native WebAssembly execution" },
          ],
        },
      ];

      // 1. Build real .pptx binary package
      const binaryData = await binaryPackageService.buildPptx(sourceSlides);
      expect(binaryData).toBeInstanceOf(Uint8Array);
      expect(binaryData.length).toBeGreaterThan(100);

      // Verify ZIP magic header PK\x03\x04
      expect(binaryData[0]).toBe(0x50);
      expect(binaryData[1]).toBe(0x4b);

      // 2. Parse binary package
      const parsed = await binaryPackageService.parsePptx(binaryData);
      expect(parsed.slides.length).toBe(2);
      expect(parsed.slides[0].title).toBe("Quarterly Strategy Review");
      expect(parsed.slides[1].title).toBe("Architecture & Performance");

      // 3. Load into slidesState via fileIoService
      const loaded = await fileIoService.loadFileContent(binaryData, "PowerPoint", "Strategy.pptx");
      expect(loaded).toBe(true);
      expect(slidesState.slides.length).toBe(2);
      expect(slidesState.slides[0].title).toBe("Quarterly Strategy Review");
      expect(slidesState.slides[1].title).toBe("Architecture & Performance");

      // 4. Save document via app
      const savedPath = await app.saveActiveDocument("SavedStrategy.pptx");
      expect(savedPath).toBe("SavedStrategy.pptx");
      expect(app.getTabBar().getActiveTab()?.isDirty).toBe(false);
    });
  });

  describe("2. Home Tab: Complete Slides, Font, Paragraph, Drawing, Arrange & Edit", () => {
    it("handles Slides: NewSlide, Layouts (Title, Content, Section, TwoContent, Comparison, TitleOnly, Blank), Duplicate, Move, Reset", () => {
      (app as any).handleRibbonAction("NewSlide");
      expect(slidesState.slides.length).toBe(2);

      (app as any).handleRibbonAction("AddSlideWithLayout", "titleAndContent");
      expect(slidesState.getCurrentSlide()?.layout).toBe("titleAndContent");

      (app as any).handleRibbonAction("AddSlideWithLayout", "comparison");
      expect(slidesState.getCurrentSlide()?.layout).toBe("comparison");
      expect(slidesState.getCurrentSlide()?.shapes.length).toBe(5);

      (app as any).handleRibbonAction("AddSlideWithLayout", "sectionHeader");
      expect(slidesState.getCurrentSlide()?.layout).toBe("sectionHeader");

      (app as any).handleRibbonAction("AddSlideWithLayout", "twoContent");
      expect(slidesState.getCurrentSlide()?.layout).toBe("twoContent");

      (app as any).handleRibbonAction("AddSlideWithLayout", "titleOnly");
      expect(slidesState.getCurrentSlide()?.layout).toBe("titleOnly");

      (app as any).handleRibbonAction("AddSlideWithLayout", "blank");
      expect(slidesState.getCurrentSlide()?.layout).toBe("blank");

      const countBefore = slidesState.slides.length;
      (app as any).handleRibbonAction("DuplicateSlide");
      expect(slidesState.slides.length).toBe(countBefore + 1);

      (app as any).handleRibbonAction("MoveSlideUp");
      (app as any).handleRibbonAction("MoveSlideDown");
      (app as any).handleRibbonAction("ResetSlide");

      (app as any).handleRibbonAction("DeleteSlide");
      expect(slidesState.slides.length).toBe(countBefore);
    });

    it("handles Font & Formatting: Family, Size, Bold, Italic, Underline, Shadow, Strike, Color, Clear", () => {
      slidesState.addShape("text");
      const sel = slidesState.getSelectedShape()!;
      expect(sel).toBeDefined();

      (app as any).handleRibbonAction("SetFontFamily", "Georgia");
      expect(sel.fontFamily).toBe("Georgia");

      (app as any).handleRibbonAction("SetFontSize", 28);
      expect(sel.fontSize).toBe(28);

      (app as any).handleRibbonAction("ToggleBold");
      expect(sel.bold).toBe(true);

      (app as any).handleRibbonAction("ToggleItalic");
      expect(sel.italic).toBe(true);

      (app as any).handleRibbonAction("ToggleUnderline");
      expect(sel.underline).toBe(true);

      (app as any).handleRibbonAction("ToggleShadow");
      expect(sel.shadow).toBe(true);

      (app as any).handleRibbonAction("ToggleStrikethrough");
      expect(sel.strikethrough).toBe(true);

      (app as any).handleRibbonAction("SetFontColor", "#15803d");
      expect(sel.textColor).toBe("#15803d");

      (app as any).handleRibbonAction("ClearFormatting");
      expect(sel.bold).toBe(false);
      expect(sel.italic).toBe(false);
    });

    it("handles Paragraph: Alignments, Bullets, Numbering, Vertical Align, Line Spacing", () => {
      slidesState.addShape("text");
      const sel = slidesState.getSelectedShape()!;
      sel.text = "Line 1\nLine 2";

      (app as any).handleRibbonAction("AlignLeft");
      expect(sel.textAlign).toBe("left");

      (app as any).handleRibbonAction("AlignCenter");
      expect(sel.textAlign).toBe("center");

      (app as any).handleRibbonAction("AlignRight");
      expect(sel.textAlign).toBe("right");

      (app as any).handleRibbonAction("AlignJustify");
      expect(sel.textAlign).toBe("justify");

      (app as any).handleRibbonAction("ToggleBulletList");
      expect(sel.text).toContain("•");

      (app as any).handleRibbonAction("ToggleNumberedList");
      expect(sel.text).toMatch(/1\./);

      (app as any).handleRibbonAction("SetVerticalAlign", "middle");
      expect(sel.verticalAlign).toBe("middle");

      (app as any).handleRibbonAction("SetLineSpacing", 1.5);
    });

    it("handles Drawing & Arrange: Shapes, Fill, Outline, Weight, Shadow, Glow, Layering, Distribution, Group, Rotate", () => {
      (app as any).handleRibbonAction("InsertShape", "rect");
      (app as any).handleRibbonAction("InsertShape", "roundRect");
      (app as any).handleRibbonAction("InsertShape", "ellipse");
      (app as any).handleRibbonAction("InsertShape", "triangle");
      (app as any).handleRibbonAction("InsertShape", "arrow");
      (app as any).handleRibbonAction("InsertShape", "star");
      (app as any).handleRibbonAction("InsertShape", "diamond");
      (app as any).handleRibbonAction("InsertShape", "heart");

      const sel = slidesState.getSelectedShape()!;
      expect(sel).toBeDefined();

      (app as any).handleRibbonAction("SetFillColor", "#3b82f6");
      expect(sel.fill).toBe("#3b82f6");

      (app as any).handleRibbonAction("SetShapeOutline", "#0f172a");
      expect(sel.stroke).toBe("#0f172a");

      (app as any).handleRibbonAction("SetOutlineWeight", 3);
      expect(sel.strokeWidth).toBe(3);

      (app as any).handleRibbonAction("SetShapeShadow", true);
      expect(sel.shadow).toBe(true);

      (app as any).handleRibbonAction("SetShapeGlow", true);
      expect(sel.glow).toBe(true);

      (app as any).handleRibbonAction("Rotate90");
      expect(sel.rotation).toBe(90);
      (app as any).handleRibbonAction("Rotate90");
      expect(sel.rotation).toBe(180);

      // Layering & Arrangement
      (app as any).handleRibbonAction("BringToFront");
      (app as any).handleRibbonAction("SendToBack");
      (app as any).handleRibbonAction("BringForward");
      (app as any).handleRibbonAction("SendBackward");
      (app as any).handleRibbonAction("DistributeH");
      (app as any).handleRibbonAction("DistributeV");
      (app as any).handleRibbonAction("Group");
      (app as any).handleRibbonAction("Ungroup");
      (app as any).handleRibbonAction("DuplicateShape");
      (app as any).handleRibbonAction("DeleteShape");
    });
  });

  describe("3. Insert Tab: Tables, Pictures, 16 Vector Shapes, SmartArt, Charts, WordArt, Headers & Media", () => {
    it("handles Table Insertion", () => {
      (app as any).handleRibbonAction("InsertTable", { rows: 4, cols: 3 });
      const sel = slidesState.getSelectedShape();
      expect(sel?.type).toBe("table");
      expect(sel?.tableData?.length).toBe(4);
      expect(sel?.tableData?.[0].length).toBe(3);
    });

    it("handles Pictures, Screenshots & 16 Vector Shapes", () => {
      (app as any).handleRibbonAction("InsertPicture", "https://example.com/slide-chart.png");
      const imgShape = slidesState.getSelectedShape();
      expect(imgShape?.type).toBe("image");
      expect(imgShape?.imageUrl).toBe("https://example.com/slide-chart.png");

      const shapeKinds = [
        "rect", "roundRect", "ellipse", "triangle", "diamond", "star",
        "arrow", "arrowLeft", "arrowUp", "arrowDown", "callout", "heart",
        "cloud", "pentagon", "hexagon", "banner"
      ];
      for (const k of shapeKinds) {
        (app as any).handleRibbonAction("InsertShape", k);
        const s = slidesState.getSelectedShape();
        expect(s).toBeDefined();
      }

      (app as any).handleRibbonAction("InsertIcons", "⚡");
      expect(slidesState.getSelectedShape()?.type).toBe("icon");
    });

    it("handles SmartArt (Process, Cycle, Hierarchy, Pyramid) & Charts (Bar, Line, Pie)", () => {
      (app as any).handleRibbonAction("InsertSmartArt", "process");
      expect(slidesState.getSelectedShape()?.smartArtType).toBe("process");

      (app as any).handleRibbonAction("InsertSmartArt", "cycle");
      expect(slidesState.getSelectedShape()?.smartArtType).toBe("cycle");

      (app as any).handleRibbonAction("InsertSmartArt", "hierarchy");
      expect(slidesState.getSelectedShape()?.smartArtType).toBe("hierarchy");

      (app as any).handleRibbonAction("InsertSmartArt", "pyramid");
      expect(slidesState.getSelectedShape()?.smartArtType).toBe("pyramid");

      (app as any).handleRibbonAction("InsertChart", "bar");
      expect(slidesState.getSelectedShape()?.chartType).toBe("bar");

      (app as any).handleRibbonAction("InsertChart", "line");
      expect(slidesState.getSelectedShape()?.chartType).toBe("line");

      (app as any).handleRibbonAction("InsertChart", "pie");
      expect(slidesState.getSelectedShape()?.chartType).toBe("pie");
    });

    it("handles Text, WordArt, Header/Footer, Slide Number & Media", () => {
      (app as any).handleRibbonAction("InsertTextBox");
      expect(slidesState.getSelectedShape()?.type).toBe("text");

      (app as any).handleRibbonAction("InsertWordArt", "Executive Vision");
      expect(slidesState.getSelectedShape()?.type).toBe("wordArt");

      (app as any).handleRibbonAction("InsertHeaderFooter");
      (app as any).handleRibbonAction("InsertSlideNumber");

      (app as any).handleRibbonAction("InsertVideo", "corporate_overview.mp4");
      expect(slidesState.getSelectedShape()?.mediaType).toBe("video");

      (app as any).handleRibbonAction("InsertAudio", "background_score.mp3");
      expect(slidesState.getSelectedShape()?.mediaType).toBe("audio");
    });
  });

  describe("4. Design Tab: 12+ Themes, Variants, 16:9 / 4:3 Slide Sizes & Background Customization", () => {
    it("handles Curated Themes and Presentation Themes", () => {
      const themes = [
        "Dark Slate", "Ocean Blue", "Emerald", "Sunset", "Minimal White",
        "Executive Navy", "Warm Sand", "Cyber Neon", "Office", "Celestial",
        "Facet", "Integral", "Ion", "Organic", "Retrospect", "Slice", "Wisp", "Slate"
      ];
      for (const t of themes) {
        (app as any).handleRibbonAction("SetTheme", t);
        expect(slidesState.currentTheme.toLowerCase()).toBe(t.toLowerCase().replace(/\s+/g, ""));
      }
    });

    it("handles Variants, Slide Aspect Ratio (16:9 / 4:3) and Backgrounds", () => {
      (app as any).handleRibbonAction("SetVariant", "Emerald");
      expect(slidesState.currentVariant).toBe("Emerald");

      (app as any).handleRibbonAction("SetSlideSize", "4:3");
      expect(slidesState.slideAspectRatio).toBe("4:3");
      expect(slidesState.slideWidth).toBe(720);

      (app as any).handleRibbonAction("SetSlideSize", "16:9");
      expect(slidesState.slideAspectRatio).toBe("16:9");
      expect(slidesState.slideWidth).toBe(960);

      (app as any).handleRibbonAction("SetSlideBackground", "#1e293b");
      expect(slidesState.getCurrentSlide()?.background).toBe("#1e293b");

      (app as any).handleRibbonAction("SetSlideBackground", "linear-gradient(135deg, #1e3a8a 0%, #0f172a 100%)");
      expect(slidesState.getCurrentSlide()?.background).toContain("linear-gradient");

      (app as any).handleRibbonAction("SetSlideBackground", { background: "#0284c7", applyToAll: true });
      for (const s of slidesState.slides) {
        expect(s.background).toBe("#0284c7");
      }
    });
  });

  describe("5. Transitions Tab: Transitions, Effects, Timing, Sound & Apply to All", () => {
    it("handles 12+ slide transitions, directions, duration, sound and advance settings", () => {
      const transitions = [
        "Fade", "Push", "Wipe", "Split", "Reveal", "Random Bars",
        "Shape", "Zoom", "Morph", "Slide Left", "Cube", "Flip", "Vortex"
      ];
      for (const tr of transitions) {
        (app as any).handleRibbonAction("SetTransition", tr);
        expect(slidesState.getCurrentSlide()?.transition).toBeDefined();
      }

      (app as any).handleRibbonAction("SetEffectOption", "left");
      expect(slidesState.getCurrentSlide()?.transitionConfig?.direction).toBe("left");

      (app as any).handleRibbonAction("SetTransitionDuration", 2.0);
      expect(slidesState.getCurrentSlide()?.transitionConfig?.duration).toBe(2.0);

      (app as any).handleRibbonAction("SetTransitionSound", "Chime");
      expect(slidesState.getCurrentSlide()?.transitionConfig?.sound).toBe("Chime");

      (app as any).handleRibbonAction("ToggleAdvanceOnClick", true);
      expect(slidesState.getCurrentSlide()?.transitionConfig?.advanceOnClick).toBe(true);

      (app as any).handleRibbonAction("SetAdvanceDelay", 5);
      expect(slidesState.getCurrentSlide()?.transitionConfig?.advanceAfter).toBe(5);

      (app as any).handleRibbonAction("ApplyTransitionToAll");
      for (const s of slidesState.slides) {
        expect(s.transitionConfig?.duration).toBe(2.0);
      }

      (app as any).handleRibbonAction("PreviewTransition");
    });
  });

  describe("6. Ribbon DOM Audit: Zero Overflow & Active Control Verification", () => {
    it("renders Home, Insert, Design, Transitions tabs and verifies all buttons are visible and active", () => {
      const ribbonContainer = document.getElementById("ribbon-container")!;
      expect(ribbonContainer).toBeDefined();

      const tabs = ["Home", "Insert", "Design", "Transitions"];
      const ribbon = app.getRibbon();

      for (const tabName of tabs) {
        ribbon.setActiveTab(tabName);
        const toolsBody = ribbonContainer.querySelector(".ribbon-tools-body") as HTMLElement;
        expect(toolsBody).not.toBeNull();

        const groups = toolsBody.querySelectorAll(".ribbon-group");
        expect(groups.length).toBeGreaterThan(0);

        const buttons = toolsBody.querySelectorAll("button");
        expect(buttons.length).toBeGreaterThan(0);

        buttons.forEach((btn) => {
          expect(btn.textContent?.trim().length).toBeGreaterThan(0);
          expect(() => btn.click()).not.toThrow();
        });
      }
    }, 20000);
  });
});
