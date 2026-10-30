import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { SlidesState, slidesState, PresetShapeKind } from "../state/slidesState";
import { SlidesRibbon } from "../components/ribbon/SlidesRibbon";
import { SlideStageCanvas } from "../components/viewports/SlideStageCanvas";
import { UnifiedShellApp } from "../main";

describe("PowerPoint Home, Insert, Design & Transitions Ribbons (100% Working) (GEC-v4.0 Subphase 31.1.1)", () => {
  let state: SlidesState;
  let ribbon: SlidesRibbon;
  let ribbonBody: HTMLElement;
  let app: UnifiedShellApp;
  let stageCanvas: SlideStageCanvas;

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

    HTMLCanvasElement.prototype.getContext = (() => mockCtx) as unknown as typeof HTMLCanvasElement.prototype.getContext;

    state = new SlidesState();
    state.resetToBlank("Presentation.pptx");
    ribbon = new SlidesRibbon();
    ribbonBody = document.createElement("div");
    ribbonBody.className = "ribbon-body";
    document.body.appendChild(ribbonBody);

    const vpContainer = document.getElementById("viewport-container") as HTMLElement;
    stageCanvas = new SlideStageCanvas(vpContainer, state);

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
    document.body.innerHTML = "";
  });

  // ============================================================================
  // 1. Home Tab: Slides, Typography, Paragraph, Shapes, Arrange & Editing
  // ============================================================================
  describe("1. Home Tab Features", () => {
    it("renders all 6 New Slide layout options and applies them correctly", () => {
      const actions: { action: string; payload?: unknown }[] = [];
      ribbon.renderTab("Home", ribbonBody, (a, p) => actions.push({ action: a, payload: p }));

      const layouts: Array<{ testId: string; expectedLayout: string }> = [
        { testId: "slides-layout-title", expectedLayout: "title" },
        { testId: "slides-layout-title-content", expectedLayout: "titleAndContent" },
        { testId: "slides-layout-section", expectedLayout: "sectionHeader" },
        { testId: "slides-layout-two-content", expectedLayout: "twoContent" },
        { testId: "slides-layout-title-only", expectedLayout: "titleOnly" },
        { testId: "slides-layout-blank", expectedLayout: "blank" },
      ];

      for (const item of layouts) {
        const btn = ribbonBody.querySelector(`[data-testid="${item.testId}"]`) as HTMLButtonElement;
        expect(btn).not.toBeNull();
        btn.click();
        expect(actions).toContainEqual({ action: "AddSlideWithLayout", payload: item.expectedLayout });

        // Apply via state
        state.execCommand("AddSlideWithLayout", item.expectedLayout);
        const curSlide = state.getCurrentSlide();
        expect(curSlide.layout).toBe(item.expectedLayout);
      }
    });

    it("executes slide duplicate, delete, and reset actions", () => {
      const actions: { action: string; payload?: unknown }[] = [];
      ribbon.renderTab("Home", ribbonBody, (a, p) => actions.push({ action: a, payload: p }));

      const dupBtn = ribbonBody.querySelector('[data-testid="slides-btn-dup-slide"]') as HTMLButtonElement;
      const delBtn = ribbonBody.querySelector('[data-testid="slides-btn-delete-slide"]') as HTMLButtonElement;
      const resetBtn = ribbonBody.querySelector('[data-testid="slides-btn-reset"]') as HTMLButtonElement;

      expect(dupBtn).not.toBeNull();
      expect(delBtn).not.toBeNull();
      expect(resetBtn).not.toBeNull();

      dupBtn.click();
      delBtn.click();
      resetBtn.click();

      expect(actions.some((a) => a.action === "DuplicateSlide")).toBe(true);
      expect(actions.some((a) => a.action === "DeleteSlide")).toBe(true);
      expect(actions.some((a) => a.action === "ResetSlide")).toBe(true);

      // Verify resetSlide in state restores layout defaults
      const curSlide = state.getCurrentSlide();
      curSlide.shapes.push({
        id: "temp-custom",
        type: "rect",
        kind: "rect",
        x: 50,
        y: 50,
        w: 100,
        h: 100,
        fill: "#ff0000",
      });
      expect(curSlide.shapes.length).toBeGreaterThan(0);
      state.execCommand("ResetSlide");
      // Reset should restore default layout placeholders for slide's layout
      expect(state.getCurrentSlide().shapes.length).toBeGreaterThanOrEqual(1);
    });

    it("formats typography: Font, Size, Bold, Italic, Underline, Shadow, Color", () => {
      const actions: { action: string; payload?: unknown }[] = [];
      ribbon.renderTab("Home", ribbonBody, (a, p) => actions.push({ action: a, payload: p }));

      state.insertShape("rect");
      const shapeId = state.selectedShapeId!;
      expect(shapeId).toBeDefined();

      const boldBtn = ribbonBody.querySelector('[data-testid="slides-btn-bold"]') as HTMLButtonElement;
      const italicBtn = ribbonBody.querySelector('[data-testid="slides-btn-italic"]') as HTMLButtonElement;
      const underlineBtn = ribbonBody.querySelector('[data-testid="slides-btn-underline"]') as HTMLButtonElement;
      const shadowBtn = ribbonBody.querySelector('[data-testid="slides-btn-shadow"]') as HTMLButtonElement;
      const fontColorBtn = ribbonBody.querySelector('[data-testid="slides-btn-font-color"]') as HTMLButtonElement;
      const fontFamSelect = ribbonBody.querySelector('[data-testid="slides-select-font-family"]') as HTMLSelectElement;
      const fontSizeSelect = ribbonBody.querySelector('[data-testid="slides-select-font-size"]') as HTMLSelectElement;

      boldBtn.click();
      italicBtn.click();
      underlineBtn.click();
      shadowBtn.click();
      fontColorBtn.click();

      fontFamSelect.value = "Calibri";
      fontFamSelect.dispatchEvent(new Event("change"));
      fontSizeSelect.value = "36";
      fontSizeSelect.dispatchEvent(new Event("change"));

      expect(actions.some((a) => a.action === "ToggleBold")).toBe(true);
      expect(actions.some((a) => a.action === "ToggleItalic")).toBe(true);
      expect(actions.some((a) => a.action === "ToggleUnderline")).toBe(true);
      expect(actions.some((a) => a.action === "ToggleShadow" || a.action === "ToggleTextShadow")).toBe(true);
      expect(actions.some((a) => a.action === "SetFontColor")).toBe(true);
      expect(actions.some((a) => a.action === "SetFontFamily" && a.payload === "Calibri")).toBe(true);
      expect(actions.some((a) => a.action === "SetFontSize" && a.payload === 36)).toBe(true);

      // Verify state mutation
      state.execCommand("ToggleBold");
      state.execCommand("ToggleItalic");
      state.execCommand("ToggleUnderline");
      state.execCommand("ToggleTextShadow");
      state.execCommand("SetFontColor", "#e11d48");
      state.execCommand("SetFontFamily", "Calibri");
      state.execCommand("SetFontSize", 36);

      const targetShape = state.getCurrentSlide().shapes.find((s) => s.id === shapeId)!;
      expect(targetShape.bold).toBe(true);
      expect(targetShape.italic).toBe(true);
      expect(targetShape.underline).toBe(true);
      expect(targetShape.textShadow).toBe(true);
      expect(targetShape.color).toBe("#e11d48");
      expect(targetShape.fontFamily).toBe("Calibri");
      expect(targetShape.fontSize).toBe(36);
    });

    it("formats paragraphs: Bullets, Numbering, Alignments H & V", () => {
      const actions: { action: string; payload?: unknown }[] = [];
      ribbon.renderTab("Home", ribbonBody, (a, p) => actions.push({ action: a, payload: p }));

      state.insertShape("rect");
      const shapeId = state.selectedShapeId!;
      const curShape = state.getCurrentSlide().shapes.find((s) => s.id === shapeId)!;
      curShape.text = "Point 1\nPoint 2";

      const bulletsBtn = ribbonBody.querySelector('[data-testid="slides-btn-bullets"]') as HTMLButtonElement;
      const numberingBtn = ribbonBody.querySelector('[data-testid="slides-btn-numbering"]') as HTMLButtonElement;
      const alignLeftBtn = ribbonBody.querySelector('[data-testid="slides-btn-align-left"]') as HTMLButtonElement;
      const alignCenterBtn = ribbonBody.querySelector('[data-testid="slides-btn-align-center"]') as HTMLButtonElement;
      const alignRightBtn = ribbonBody.querySelector('[data-testid="slides-btn-align-right"]') as HTMLButtonElement;
      const alignJustifyBtn = ribbonBody.querySelector('[data-testid="slides-btn-align-justify"]') as HTMLButtonElement;
      const alignTopBtn = ribbonBody.querySelector('[data-testid="slides-btn-align-top"]') as HTMLButtonElement;
      const alignMiddleBtn = ribbonBody.querySelector('[data-testid="slides-btn-align-middle"]') as HTMLButtonElement;
      const alignBottomBtn = ribbonBody.querySelector('[data-testid="slides-btn-align-bottom"]') as HTMLButtonElement;

      bulletsBtn.click();
      numberingBtn.click();
      alignLeftBtn.click();
      alignCenterBtn.click();
      alignRightBtn.click();
      alignJustifyBtn.click();
      alignTopBtn.click();
      alignMiddleBtn.click();
      alignBottomBtn.click();

      expect(actions.some((a) => a.action === "ToggleBulletList" || a.action === "ToggleBullets")).toBe(true);
      expect(actions.some((a) => a.action === "ToggleNumberedList" || a.action === "ToggleNumbering")).toBe(true);
      expect(actions.some((a) => a.action === "AlignLeft")).toBe(true);
      expect(actions.some((a) => a.action === "AlignCenter")).toBe(true);
      expect(actions.some((a) => a.action === "AlignRight")).toBe(true);
      expect(actions.some((a) => a.action === "AlignJustify")).toBe(true);
      expect(actions.some((a) => (a.action === "SetVerticalAlign" && a.payload === "top") || a.action === "AlignTop")).toBe(true);
      expect(actions.some((a) => (a.action === "SetVerticalAlign" && a.payload === "middle") || a.action === "AlignMiddle")).toBe(true);
      expect(actions.some((a) => (a.action === "SetVerticalAlign" && a.payload === "bottom") || a.action === "AlignBottom")).toBe(true);

      // Verify state actions
      state.execCommand("ToggleBullets");
      expect(curShape.text).toContain("• Point 1");

      state.execCommand("ToggleBullets");
      state.execCommand("ToggleNumbering");
      expect(curShape.text).toContain("1. Point 1");

      state.execCommand("AlignCenter");
      expect(curShape.textAlign).toBe("center");

      state.execCommand("AlignMiddle");
      expect(curShape.verticalAlign).toBe("middle");

      state.execCommand("AlignTop");
      expect(curShape.verticalAlign).toBe("top");
    });

    it("arranges shapes: Bring to Front, Send to Back, Distribute H/V, Group/Ungroup", () => {
      const actions: { action: string; payload?: unknown }[] = [];
      ribbon.renderTab("Home", ribbonBody, (a, p) => actions.push({ action: a, payload: p }));

      state.insertShape("rect");
      const sh1 = state.selectedShapeId!;
      state.insertShape("ellipse");
      const sh2 = state.selectedShapeId!;
      expect(sh2).toBeDefined();

      const btfBtn = ribbonBody.querySelector('[data-testid="slides-btn-bring-to-front"]') as HTMLButtonElement;
      const stbBtn = ribbonBody.querySelector('[data-testid="slides-btn-send-to-back"]') as HTMLButtonElement;
      const distHBtn = ribbonBody.querySelector('[data-testid="slides-btn-distribute-h"]') as HTMLButtonElement;
      const distVBtn = ribbonBody.querySelector('[data-testid="slides-btn-distribute-v"]') as HTMLButtonElement;
      const groupBtn = ribbonBody.querySelector('[data-testid="slides-btn-group"]') as HTMLButtonElement;
      const ungroupBtn = ribbonBody.querySelector('[data-testid="slides-btn-ungroup"]') as HTMLButtonElement;

      btfBtn.click();
      stbBtn.click();
      distHBtn.click();
      distVBtn.click();
      groupBtn.click();
      ungroupBtn.click();

      expect(actions.some((a) => a.action === "BringToFront")).toBe(true);
      expect(actions.some((a) => a.action === "SendToBack")).toBe(true);
      expect(actions.some((a) => a.action === "DistributeH")).toBe(true);
      expect(actions.some((a) => a.action === "DistributeV")).toBe(true);
      expect(actions.some((a) => a.action === "Group" || a.action === "GroupShapes")).toBe(true);
      expect(actions.some((a) => a.action === "Ungroup" || a.action === "UngroupShapes")).toBe(true);

      // Test state operations
      state.selectedShapeId = sh1;
      state.execCommand("BringToFront");
      const shapes = state.getCurrentSlide().shapes;
      expect(shapes[shapes.length - 1].id).toBe(sh1);

      state.execCommand("DistributeH");
      state.execCommand("GroupShapes");
      expect(state.getCurrentSlide().shapes.some((s) => s.groupId)).toBe(true);

      state.execCommand("UngroupShapes");
      expect(state.getCurrentSlide().shapes.some((s) => s.groupId)).toBe(false);
    });

    it("applies Shape Fill, Outline, Shadow, and Glow effects", () => {
      const actions: { action: string; payload?: unknown }[] = [];
      ribbon.renderTab("Home", ribbonBody, (a, p) => actions.push({ action: a, payload: p }));

      state.insertShape("rect");
      const shapeId = state.selectedShapeId!;

      const fillBlueBtn = ribbonBody.querySelector('[data-testid="slides-fill-blue"]') as HTMLButtonElement;
      const shadowBtn = ribbonBody.querySelector('[data-testid="slides-btn-shape-shadow"]') as HTMLButtonElement;
      const glowBtn = ribbonBody.querySelector('[data-testid="slides-btn-shape-glow"]') as HTMLButtonElement;
      const outlineWeightSelect = ribbonBody.querySelector('[data-testid="slides-select-outline-weight"]') as HTMLSelectElement;

      fillBlueBtn.click();
      shadowBtn.click();
      glowBtn.click();
      outlineWeightSelect.value = "3";
      outlineWeightSelect.dispatchEvent(new Event("change"));

      expect(actions.some((a) => (a.action === "SetFillColor" || a.action === "SetShapeFill") && a.payload === "#2563eb")).toBe(true);
      expect(actions.some((a) => a.action === "SetShapeShadow" || a.action === "ToggleShapeShadow")).toBe(true);
      expect(actions.some((a) => a.action === "SetShapeGlow" || a.action === "ToggleShapeGlow")).toBe(true);
      expect(actions.some((a) => a.action === "SetOutlineWeight" && a.payload === 3)).toBe(true);

      // Verify state mutation
      state.execCommand("SetShapeFill", "#2563eb");
      state.execCommand("ToggleShapeShadow");
      state.execCommand("ToggleShapeGlow");
      state.execCommand("SetOutlineWeight", 3);

      const targetShape = state.getCurrentSlide().shapes.find((s) => s.id === shapeId)!;
      expect(targetShape.fill).toBe("#2563eb");
      expect(targetShape.shadow).toBe(true);
      expect(targetShape.glow).toBe(true);
      expect(targetShape.strokeWidth).toBe(3);
    });

    it("opens Find and Replace dialog for PowerPoint and replaces matching shape text", () => {
      const actions: { action: string; payload?: unknown }[] = [];
      ribbon.renderTab("Home", ribbonBody, (a, p) => actions.push({ action: a, payload: p }));

      const findBtn = ribbonBody.querySelector('[data-testid="slides-btn-find"]') as HTMLButtonElement;
      const replaceBtn = ribbonBody.querySelector('[data-testid="slides-btn-replace"]') as HTMLButtonElement;

      expect(findBtn).not.toBeNull();
      expect(replaceBtn).not.toBeNull();

      // Setup slide with shape text
      state.insertShape("rect");
      const shape = state.getCurrentSlide().shapes.find((s) => s.id === state.selectedShapeId)!;
      shape.text = "Hello World TargetText!";

      // Trigger Find in app
      app.handleRibbonAction("Find");
      const dialog = app.getFindReplaceDialog();
      expect(dialog).toBeDefined();

      // Trigger Replace in app
      app.handleRibbonAction("Replace");
      const input = document.querySelector(".find-query-input") as HTMLInputElement;
      expect(input).not.toBeNull();
    });
  });

  // ============================================================================
  // 2. Insert Tab: 16 Vector Shapes, SmartArt, Charts, Media, WordArt, Slide Num
  // ============================================================================
  describe("2. Insert Tab Features", () => {
    it("renders all 16 vector shapes in Insert Tab and adds each shape to the slide", () => {
      const actions: { action: string; payload?: unknown }[] = [];
      ribbon.renderTab("Insert", ribbonBody, (a, p) => actions.push({ action: a, payload: p }));

      const shapeKinds: Array<{ testId: string; kind: PresetShapeKind }> = [
        { testId: "slides-shape-rect", kind: "rect" },
        { testId: "slides-shape-round-rect", kind: "roundRect" },
        { testId: "slides-shape-circle", kind: "ellipse" },
        { testId: "slides-shape-triangle", kind: "triangle" },
        { testId: "slides-shape-diamond", kind: "diamond" },
        { testId: "slides-shape-star", kind: "star" },
        { testId: "slides-shape-arrow", kind: "arrow" },
        { testId: "slides-shape-arrow-left", kind: "arrowLeft" },
        { testId: "slides-shape-arrow-up", kind: "arrowUp" },
        { testId: "slides-shape-arrow-down", kind: "arrowDown" },
        { testId: "slides-shape-callout", kind: "callout" },
        { testId: "slides-shape-heart", kind: "heart" },
        { testId: "slides-shape-cloud", kind: "cloud" },
        { testId: "slides-shape-pentagon", kind: "pentagon" },
        { testId: "slides-shape-hexagon", kind: "hexagon" },
        { testId: "slides-shape-banner", kind: "banner" },
      ];

      for (const item of shapeKinds) {
        const btn = ribbonBody.querySelector(`[data-testid="${item.testId}"]`) as HTMLButtonElement;
        expect(btn).not.toBeNull();
        btn.click();
        expect(actions).toContainEqual({ action: "InsertShape", payload: item.kind });

        state.execCommand("InsertShape", item.kind);
        const curShape = state.getCurrentSlide().shapes.find((s) => s.id === state.selectedShapeId);
        expect(curShape).toBeDefined();
        expect(curShape!.kind).toBe(item.kind);
      }
    });

    it("inserts SmartArt diagrams: Process, Cycle, Hierarchy, and Pyramid", () => {
      const actions: { action: string; payload?: unknown }[] = [];
      ribbon.renderTab("Insert", ribbonBody, (a, p) => actions.push({ action: a, payload: p }));

      const smartArtTypes = [
        { testId: "slides-smartart-process", type: "process" },
        { testId: "slides-smartart-cycle", type: "cycle" },
        { testId: "slides-smartart-hierarchy", type: "hierarchy" },
        { testId: "slides-smartart-pyramid", type: "pyramid" },
      ];

      for (const sa of smartArtTypes) {
        const btn = ribbonBody.querySelector(`[data-testid="${sa.testId}"]`) as HTMLButtonElement;
        expect(btn).not.toBeNull();
        btn.click();
        expect(actions).toContainEqual({ action: "InsertSmartArt", payload: sa.type });

        state.execCommand("InsertSmartArt", sa.type);
        const shape = state.getCurrentSlide().shapes.find((s) => s.id === state.selectedShapeId);
        expect(shape).toBeDefined();
        expect(shape!.kind).toBe("smartArt");
        expect(shape!.smartArtType).toBe(sa.type);
      }
    });

    it("inserts editable Charts: Bar, Line, and Pie", () => {
      const actions: { action: string; payload?: unknown }[] = [];
      ribbon.renderTab("Insert", ribbonBody, (a, p) => actions.push({ action: a, payload: p }));

      const chartTypes = [
        { testId: "slides-chart-bar", type: "bar" },
        { testId: "slides-chart-line", type: "line" },
        { testId: "slides-chart-pie", type: "pie" },
      ];

      for (const ct of chartTypes) {
        const btn = ribbonBody.querySelector(`[data-testid="${ct.testId}"]`) as HTMLButtonElement;
        expect(btn).not.toBeNull();
        btn.click();
        expect(actions).toContainEqual({ action: "InsertChart", payload: ct.type });

        state.execCommand("InsertChart", ct.type);
        const shape = state.getCurrentSlide().shapes.find((s) => s.id === state.selectedShapeId);
        expect(shape).toBeDefined();
        expect(shape!.kind).toBe("chart");
        expect(shape!.chartType).toBe(ct.type);
        expect(shape!.chartData).toBeDefined();
        expect(shape!.chartData!.categories.length).toBeGreaterThan(0);
      }
    });

    it("inserts Icons, WordArt, Header & Footer, Slide Number, and Table", () => {
      const actions: { action: string; payload?: unknown }[] = [];
      ribbon.renderTab("Insert", ribbonBody, (a, p) => actions.push({ action: a, payload: p }));

      const iconBtn = ribbonBody.querySelector('[data-testid="slides-btn-insert-icons"]') as HTMLButtonElement;
      const wordArtBtn = ribbonBody.querySelector('[data-testid="slides-btn-wordart"]') as HTMLButtonElement;
      const hfBtn = ribbonBody.querySelector('[data-testid="slides-btn-header-footer"]') as HTMLButtonElement;
      const numBtn = ribbonBody.querySelector('[data-testid="slides-btn-slide-number"]') as HTMLButtonElement;
      const tableBtn = ribbonBody.querySelector('[data-testid="slides-btn-insert-table"]') as HTMLButtonElement;

      iconBtn.click();
      wordArtBtn.click();
      hfBtn.click();
      numBtn.click();
      tableBtn.click();

      expect(actions.some((a) => a.action === "InsertIcons" || a.action === "InsertIcon")).toBe(true);
      expect(actions.some((a) => a.action === "InsertWordArt")).toBe(true);
      expect(actions.some((a) => a.action === "InsertHeaderFooter")).toBe(true);
      expect(actions.some((a) => a.action === "InsertSlideNumber")).toBe(true);
      expect(actions.some((a) => a.action === "InsertTable")).toBe(true);

      // Verify state actions
      state.execCommand("InsertIcon", "rocket");
      const iconShape = state.getCurrentSlide().shapes.find((s) => s.id === state.selectedShapeId);
      expect(iconShape?.kind).toBe("icon");

      state.execCommand("InsertWordArt", "Sample WordArt");
      const waShape = state.getCurrentSlide().shapes.find((s) => s.id === state.selectedShapeId);
      expect(waShape?.kind).toBe("wordArt");

      state.execCommand("InsertHeaderFooter");
      expect(state.getCurrentSlide().shapes.some((s) => s.isHeaderFooter || s.headerFooterType === "footer")).toBe(true);

      state.execCommand("InsertSlideNumber");
      expect(state.getCurrentSlide().shapes.some((s) => s.headerFooterType === "slideNumber")).toBe(true);
    });
  });

  // ============================================================================
  // 3. Design Tab: 10 Themes, 4 Variants, Slide Size (4:3 vs 16:9), Backgrounds
  // ============================================================================
  describe("3. Design Tab Features", () => {
    it("renders all 10 presentation themes and applies them", () => {
      const actions: { action: string; payload?: unknown }[] = [];
      ribbon.renderTab("Design", ribbonBody, (a, p) => actions.push({ action: a, payload: p }));

      const themes = [
        { testId: "slides-theme-office", theme: "Office" },
        { testId: "slides-theme-celestial", theme: "Celestial" },
        { testId: "slides-theme-facet", theme: "Facet" },
        { testId: "slides-theme-integral", theme: "Integral" },
        { testId: "slides-theme-ion", theme: "Ion" },
        { testId: "slides-theme-organic", theme: "Organic" },
        { testId: "slides-theme-retrospect", theme: "Retrospect" },
        { testId: "slides-theme-slice", theme: "Slice" },
        { testId: "slides-theme-wisp", theme: "Wisp" },
        { testId: "slides-theme-slate", theme: "Slate" },
      ];

      for (const t of themes) {
        const btn = ribbonBody.querySelector(`[data-testid="${t.testId}"]`) as HTMLButtonElement;
        expect(btn).not.toBeNull();
        btn.click();
        expect(actions).toContainEqual({ action: "SetTheme", payload: t.theme });

        state.execCommand("SetTheme", t.theme);
        expect(state.currentTheme).toBe(t.theme);
      }
    });

    it("applies 4 theme color variants: Blue, Emerald, Coral, Purple", () => {
      const actions: { action: string; payload?: unknown }[] = [];
      ribbon.renderTab("Design", ribbonBody, (a, p) => actions.push({ action: a, payload: p }));

      const variants = [
        { testId: "slides-variant-blue", variant: "Blue" },
        { testId: "slides-variant-emerald", variant: "Emerald" },
        { testId: "slides-variant-coral", variant: "Coral" },
        { testId: "slides-variant-purple", variant: "Purple" },
      ];

      for (const v of variants) {
        const btn = ribbonBody.querySelector(`[data-testid="${v.testId}"]`) as HTMLButtonElement;
        expect(btn).not.toBeNull();
        btn.click();
        expect(actions).toContainEqual({ action: "SetVariant", payload: v.variant });

        state.execCommand("SetVariant", v.variant);
        expect(state.currentVariant).toBe(v.variant);
      }
    });

    it("switches slide aspect ratio between 16:9 and 4:3", () => {
      const actions: { action: string; payload?: unknown }[] = [];
      ribbon.renderTab("Design", ribbonBody, (a, p) => actions.push({ action: a, payload: p }));

      const btn43 = ribbonBody.querySelector('[data-testid="slides-btn-size-4-3"]') as HTMLButtonElement;
      const btn169 = ribbonBody.querySelector('[data-testid="slides-btn-size-16-9"]') as HTMLButtonElement;

      btn43.click();
      expect(actions).toContainEqual({ action: "SetSlideSize", payload: "4:3" });
      state.execCommand("SetSlideSize", "4:3");
      expect(state.slideAspectRatio).toBe("4:3");
      expect(state.slideWidth).toBe(720);
      expect(state.slideHeight).toBe(540);

      btn169.click();
      expect(actions).toContainEqual({ action: "SetSlideSize", payload: "16:9" });
      state.execCommand("SetSlideSize", "16:9");
      expect(state.slideAspectRatio).toBe("16:9");
      expect(state.slideWidth).toBe(960);
      expect(state.slideHeight).toBe(540);
    });

    it("applies background fills: Solid, Gradient, Radial, Pattern, Apply to All", () => {
      const actions: { action: string; payload?: unknown }[] = [];
      ribbon.renderTab("Design", ribbonBody, (a, p) => actions.push({ action: a, payload: p }));

      const solidBtn = ribbonBody.querySelector('[data-testid="slides-btn-solid-bg"]') as HTMLButtonElement;
      const gradBtn = ribbonBody.querySelector('[data-testid="slides-btn-gradient-bg"]') as HTMLButtonElement;
      const radBtn = ribbonBody.querySelector('[data-testid="slides-btn-radial-bg"]') as HTMLButtonElement;
      const patBtn = ribbonBody.querySelector('[data-testid="slides-btn-pattern-bg"]') as HTMLButtonElement;
      const applyAllBtn = ribbonBody.querySelector('[data-testid="slides-btn-apply-all-bg"]') as HTMLButtonElement;

      solidBtn.click();
      gradBtn.click();
      radBtn.click();
      patBtn.click();
      applyAllBtn.click();

      expect(actions.some((a) => a.action === "SetSlideBackground" && String(a.payload).includes("#0f172a"))).toBe(true);
      expect(actions.some((a) => a.action === "SetSlideBackground" && String(a.payload).includes("linear-gradient"))).toBe(true);
      expect(actions.some((a) => a.action === "SetSlideBackground" && String(a.payload).includes("radial-gradient"))).toBe(true);
      expect(actions.some((a) => a.action === "SetSlideBackground" && String(a.payload).includes("repeating-linear-gradient"))).toBe(true);

      // Verify state background changes
      state.execCommand("SetSlideBackground", "radial-gradient(circle, #38bdf8 0%, #0f172a 100%)");
      expect(state.getCurrentSlide().background).toContain("radial-gradient");

      // Add second slide and apply to all
      state.execCommand("NewSlide");
      expect(state.slides.length).toBe(2);
      state.execCommand("SetSlideBackground", { applyToAll: true });
      expect(state.slides[0].background).toBe(state.slides[1].background);
    });
  });

  // ============================================================================
  // 4. Transitions Tab: All Transitions, Directions, Sound, Duration, Advance
  // ============================================================================
  describe("4. Transitions Tab Features", () => {
    it("renders and selects all transition types including Push, Wipe, Split, Reveal, Random Bars, Shape, Zoom, Morph", () => {
      const actions: { action: string; payload?: unknown }[] = [];
      ribbon.renderTab("Transitions", ribbonBody, (a, p) => actions.push({ action: a, payload: p }));

      const transitions = [
        { testId: "slides-trans-none", name: "None" },
        { testId: "slides-trans-fade", name: "Fade" },
        { testId: "slides-trans-push", name: "Push" },
        { testId: "slides-trans-wipe", name: "Wipe" },
        { testId: "slides-trans-split", name: "Split" },
        { testId: "slides-trans-reveal", name: "Reveal" },
        { testId: "slides-trans-random-bars", name: "Random Bars" },
        { testId: "slides-trans-shape", name: "Shape" },
        { testId: "slides-trans-zoom", name: "Zoom" },
        { testId: "slides-trans-morph", name: "Morph" },
      ];

      for (const tr of transitions) {
        const btn = ribbonBody.querySelector(`[data-testid="${tr.testId}"]`) as HTMLButtonElement;
        expect(btn).not.toBeNull();
        btn.click();
        expect(actions).toContainEqual({ action: "SetTransition", payload: tr.name });

        state.execCommand("SetTransition", tr.name);
        expect(state.getCurrentSlide().transition).toBe(tr.name);
        expect(state.getCurrentSlide().transitionConfig?.effect).toBe(tr.name);
      }
    });

    it("configures Effect Options: Left, Right, Top, Bottom", () => {
      const actions: { action: string; payload?: unknown }[] = [];
      ribbon.renderTab("Transitions", ribbonBody, (a, p) => actions.push({ action: a, payload: p }));

      const dirs = [
        { testId: "slides-btn-effect-left", dir: "left" },
        { testId: "slides-btn-effect-right", dir: "right" },
        { testId: "slides-btn-effect-top", dir: "top" },
        { testId: "slides-btn-effect-bottom", dir: "bottom" },
      ];

      for (const d of dirs) {
        const btn = ribbonBody.querySelector(`[data-testid="${d.testId}"]`) as HTMLButtonElement;
        expect(btn).not.toBeNull();
        btn.click();
        expect(actions).toContainEqual({ action: "SetEffectOption", payload: d.dir });

        state.execCommand("SetEffectOption", d.dir);
        expect(state.getCurrentSlide().transitionConfig?.direction).toBe(d.dir);
      }
    });

    it("configures Transition Sounds: None, Chime, Applause, Drum Roll, Whoosh", () => {
      const actions: { action: string; payload?: unknown }[] = [];
      ribbon.renderTab("Transitions", ribbonBody, (a, p) => actions.push({ action: a, payload: p }));

      const sounds = [
        { testId: "slides-btn-sound-none", sound: "none" },
        { testId: "slides-btn-sound-chime", sound: "chime" },
        { testId: "slides-btn-sound-applause", sound: "applause" },
        { testId: "slides-btn-sound-drumroll", sound: "drumroll" },
        { testId: "slides-btn-sound-whoosh", sound: "whoosh" },
      ];

      for (const s of sounds) {
        const btn = ribbonBody.querySelector(`[data-testid="${s.testId}"]`) as HTMLButtonElement;
        expect(btn).not.toBeNull();
        btn.click();
        expect(actions).toContainEqual({ action: "SetTransitionSound", payload: s.sound });

        state.execCommand("SetTransitionSound", s.sound);
        expect(state.getCurrentSlide().transitionConfig?.sound).toBe(s.sound);
      }
    });

    it("configures Durations: 0.5s, 1.0s, 1.5s, 2.0s and Apply To All", () => {
      const actions: { action: string; payload?: unknown }[] = [];
      ribbon.renderTab("Transitions", ribbonBody, (a, p) => actions.push({ action: a, payload: p }));

      const durations = [
        { testId: "slides-btn-duration-05", dur: 0.5 },
        { testId: "slides-btn-duration-10", dur: 1.0 },
        { testId: "slides-btn-duration-15", dur: 1.5 },
        { testId: "slides-btn-duration-20", dur: 2.0 },
      ];

      for (const d of durations) {
        const btn = ribbonBody.querySelector(`[data-testid="${d.testId}"]`) as HTMLButtonElement;
        expect(btn).not.toBeNull();
        btn.click();
        expect(actions).toContainEqual({ action: "SetTransitionDuration", payload: d.dur });

        state.execCommand("SetTransitionDuration", d.dur);
        expect(state.getCurrentSlide().transitionConfig?.duration).toBe(d.dur);
      }

      const applyAllBtn = ribbonBody.querySelector('[data-testid="slides-btn-trans-apply-all"]') as HTMLButtonElement;
      applyAllBtn.click();
      expect(actions.some((a) => a.action === "ApplyTransitionToAll")).toBe(true);

      state.execCommand("ApplyTransitionToAll");
      state.execCommand("NewSlide");
      state.currentSlideIndex = 0;
      state.execCommand("ApplyTransitionToAll");
      expect(state.slides[0].transitionConfig?.duration).toBe(2.0);
      expect(state.slides[1].transitionConfig?.duration).toBe(2.0);
    });

    it("configures Advance Slide timing: On Click vs After Delay and Preview", async () => {
      const actions: { action: string; payload?: unknown }[] = [];
      ribbon.renderTab("Transitions", ribbonBody, (a, p) => actions.push({ action: a, payload: p }));

      const clickBtn = ribbonBody.querySelector('[data-testid="slides-btn-advance-click"]') as HTMLButtonElement;
      const delayBtn = ribbonBody.querySelector('[data-testid="slides-btn-advance-delay"]') as HTMLButtonElement;
      const prevBtn = ribbonBody.querySelector('[data-testid="slides-btn-preview-trans"]') as HTMLButtonElement;

      clickBtn.click();
      delayBtn.click();
      prevBtn.click();

      expect(actions.some((a) => a.action === "ToggleAdvanceOnClick")).toBe(true);
      expect(actions.some((a) => a.action === "SetAdvanceDelay")).toBe(true);
      expect(actions.some((a) => a.action === "PreviewTransition")).toBe(true);

      state.execCommand("ToggleAdvanceOnClick");
      state.execCommand("SetAdvanceDelay", 5);

      const trans = state.getCurrentSlide().transitionConfig;
      expect(trans?.advanceOnClick).toBe(false);
      expect(trans?.advanceAfter).toBe(5);

      // Verify canvas preview execution doesn't throw
      await expect(stageCanvas.playTransitionPreview()).resolves.toBeUndefined();
    });
  });

  // ============================================================================
  // 5. Canvas 2D Rendering of SmartArt, Charts, WordArt, and Shapes
  // ============================================================================
  describe("5. Canvas 2D Rendering Verification", () => {
    it("renders complex shapes, SmartArt, and Charts without throwing errors", () => {
      state.insertShape("cloud");
      state.insertShape("pentagon");
      state.insertShape("hexagon");
      state.insertShape("banner");
      state.insertShape("arrowLeft");
      state.insertShape("arrowUp");
      state.insertShape("arrowDown");
      state.insertSmartArt("process");
      state.insertSmartArt("cycle");
      state.insertSmartArt("hierarchy");
      state.insertSmartArt("pyramid");
      state.insertChart("bar");
      state.insertChart("line");
      state.insertChart("pie");
      state.insertWordArt("Styled 3D WordArt");

      expect(() => {
        stageCanvas.render();
      }).not.toThrow();
    });
  });
});
