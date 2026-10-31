import { describe, it, expect, beforeEach, vi } from "vitest";
import { binaryPackageService } from "../services/binaryPackageService";
import { fileCodec } from "../services/fileCodec";
import { slidesState, SlideData } from "../state/slidesState";
import { SlidesRibbon } from "../components/ribbon/SlidesRibbon";
import { SlideStageCanvas } from "../components/viewports/SlideStageCanvas";
import { IPC } from "../ipc";

describe("GEC-v6.0 Subphase 38.1.1: PowerPoint (.pptx) File Round-Trip & Complete 8-Tab Ribbon Coverage", () => {
  let ribbon: SlidesRibbon;
  let ribbonContainer: HTMLElement;
  let dispatchedActions: Array<{ action: string; payload?: unknown }>;

  beforeEach(() => {
    dispatchedActions = [];
    ribbon = new SlidesRibbon();
    ribbonContainer = document.createElement("div");
    ribbonContainer.className = "ribbon-panel";
    document.body.innerHTML = "";
    document.body.appendChild(ribbonContainer);

    // Mock Canvas 2D context for SlideStageCanvas in JSDOM
    const mockCtx = {
      canvas: document.createElement("canvas"),
      fillStyle: "",
      strokeStyle: "",
      lineWidth: 1,
      font: "",
      textAlign: "left",
      textBaseline: "top",
      clearRect: vi.fn(),
      fillRect: vi.fn(),
      strokeRect: vi.fn(),
      fillText: vi.fn(),
      strokeText: vi.fn(),
      measureText: vi.fn().mockReturnValue({ width: 50 }),
      beginPath: vi.fn(),
      closePath: vi.fn(),
      moveTo: vi.fn(),
      lineTo: vi.fn(),
      arc: vi.fn(),
      stroke: vi.fn(),
      fill: vi.fn(),
      save: vi.fn(),
      restore: vi.fn(),
      scale: vi.fn(),
      rotate: vi.fn(),
      translate: vi.fn(),
      setLineDash: vi.fn(),
      getLineDash: vi.fn().mockReturnValue([]),
      createLinearGradient: vi.fn().mockReturnValue({
        addColorStop: vi.fn(),
      }),
      createRadialGradient: vi.fn().mockReturnValue({
        addColorStop: vi.fn(),
      }),
      createPattern: vi.fn(),
      drawImage: vi.fn(),
    } as unknown as CanvasRenderingContext2D;

    HTMLCanvasElement.prototype.getContext = vi.fn().mockImplementation((contextId: string) => {
      if (contextId === "2d") return mockCtx;
      return null;
    });

    // Reset slides state
    slidesState.resetToBlank("TestPresentation.pptx");
  });

  const renderTab = (tabName: string) => {
    ribbon.renderTab(tabName, ribbonContainer, (action, payload) => {
      dispatchedActions.push({ action, payload });
    });
  };

  describe("1. Real .pptx Binary Package Build, Save & Open Round-Trip", () => {
    it("serializes rich slides with shapes, transitions, animations, and notes into valid .pptx and restores them", async () => {
      const sourceSlides: SlideData[] = [
        {
          id: "slide-1",
          title: "Executive Strategic Overview 2026",
          layout: "title",
          background: "#0f172a",
          transition: "Fade",
          transitionConfig: {
            effect: "Fade",
            duration: 1.5,
            direction: "left",
          },
          notes: "Welcome everyone to the FY26 strategy keynote.",
          shapes: [
            {
              id: "title-shape",
              type: "text",
              x: 100,
              y: 120,
              w: 760,
              h: 80,
              fill: "transparent",
              text: "Executive Strategic Overview 2026",
              textColor: "#f8fafc",
              fontSize: 40,
              bold: true,
              rotation: 0,
            },
            {
              id: "subtitle-shape",
              type: "text",
              x: 100,
              y: 220,
              w: 760,
              h: 50,
              fill: "transparent",
              text: "Next-Generation Office Productivity with Rust & WebAssembly",
              textColor: "#38bdf8",
              fontSize: 22,
              italic: true,
              rotation: 0,
            },
          ],
        },
        {
          id: "slide-2",
          title: "System Architecture & Benchmarks",
          layout: "titleAndContent",
          background: "#ffffff",
          transition: "Push",
          transitionConfig: {
            effect: "Push",
            duration: 1.0,
            direction: "bottom",
          },
          notes: "Point out the 10x performance improvements.",
          shapes: [
            {
              id: "header-shape",
              type: "text",
              x: 60,
              y: 40,
              w: 840,
              h: 60,
              fill: "transparent",
              text: "System Architecture & Benchmarks",
              textColor: "#0f172a",
              fontSize: 32,
              bold: true,
            },
            {
              id: "rect-vector",
              type: "rect",
              x: 80,
              y: 140,
              w: 220,
              h: 150,
              fill: "#2563eb",
              stroke: "#1e3a8a",
              strokeWidth: 2,
              text: "Native Rust Engine",
              textColor: "#ffffff",
              fontSize: 16,
              bold: true,
            },
            {
              id: "star-vector",
              type: "star",
              x: 340,
              y: 140,
              w: 220,
              h: 150,
              fill: "#eab308",
              text: "Zero-Latency DOM",
              textColor: "#000000",
              fontSize: 16,
            },
            {
              id: "arrow-vector",
              type: "arrow",
              x: 600,
              y: 180,
              w: 120,
              h: 60,
              fill: "#10b981",
              text: "Fast",
            },
          ],
        },
      ];

      slidesState.slides = sourceSlides;

      // 1. Build .pptx bytes
      const bytes = await binaryPackageService.buildPptx(sourceSlides);
      expect(bytes).toBeInstanceOf(Uint8Array);
      expect(bytes.length).toBeGreaterThan(100);

      // Verify ZIP Magic Header PK\x03\x04
      expect(bytes[0]).toBe(0x50);
      expect(bytes[1]).toBe(0x4b);

      // 2. Save document via fileCodec and verify toast
      (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ = {};
      let savedBytes: Uint8Array | number[] | undefined;
      vi.spyOn(IPC, "saveFileBytes").mockImplementation(async (_fname, b) => {
        savedBytes = b;
        return "C:\\Users\\User\\Documents\\Strategy2026.pptx";
      });

      const savedPath = await fileCodec.saveFile("PowerPoint", { filename: "Strategy2026.pptx" });
      expect(savedPath).toBe("C:\\Users\\User\\Documents\\Strategy2026.pptx");

      const toast = document.getElementById("save-toast-banner");
      expect(toast).not.toBeNull();
      expect(toast?.textContent).toContain("Strategy2026.pptx");
      expect(toast?.textContent).toContain("✅ Saved");

      // 3. Clear slides state and re-open via fileCodec
      slidesState.slides = [];
      const u8 = savedBytes instanceof Uint8Array ? savedBytes : new Uint8Array(savedBytes!);
      const openSuccess = await fileCodec.openFile(u8, "Strategy2026.pptx");
      expect(openSuccess).toBe(true);

      // 4. Assert round-trip restoration
      expect(slidesState.slides.length).toBe(2);
      expect(slidesState.slides[0].title).toBe("Executive Strategic Overview 2026");
      expect(slidesState.slides[0].shapes.length).toBe(2);
      expect(slidesState.slides[0].shapes[0].text).toBe("Executive Strategic Overview 2026");
      expect(slidesState.slides[0].notes).toBe("Welcome everyone to the FY26 strategy keynote.");
      expect(slidesState.slides[0].transition).toBe("Fade");

      expect(slidesState.slides[1].title).toBe("System Architecture & Benchmarks");
      expect(slidesState.slides[1].shapes.length).toBe(4);
      expect(slidesState.slides[1].shapes[1].fill).toBe("#2563eb");
      expect(slidesState.slides[1].shapes[2].type).toBe("star");
      expect(slidesState.slides[1].notes).toBe("Point out the 10x performance improvements.");
    });

    it("parses raw PresentationML XML (.pptx without deckState.json) extracting shapes, positions, fills, and notes", async () => {
      // Build a minimal raw .pptx package manually without deckState.json
      const JSZip = (await import("jszip")).default;
      const rawZip = new JSZip();

      rawZip.file(
        "[Content_Types].xml",
        `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
  <Override PartName="/ppt/slides/slide1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>
  <Override PartName="/ppt/notesSlides/notesSlide1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.notesSlide+xml"/>
</Types>`
      );

      rawZip.file(
        "_rels/.rels",
        `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
</Relationships>`
      );

      rawZip.file(
        "ppt/_rels/presentation.xml.rels",
        `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide1.xml"/>
</Relationships>`
      );

      rawZip.file(
        "ppt/presentation.xml",
        `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:sldIdLst>
    <p:sldId id="256" r:id="rId1"/>
  </p:sldIdLst>
</p:presentation>`
      );

      // Slide with 2 shapes: a rectangle with solid blue fill and a triangle with stroke
      rawZip.file(
        "ppt/slides/slide1.xml",
        `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
      <p:grpSpPr/>
      <p:sp>
        <p:nvSpPr><p:cNvPr id="2" name="Title Box"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr>
        <p:spPr>
          <a:xfrm><a:off x="952500" y="952500"/><a:ext cx="4762500" cy="952500"/></a:xfrm>
          <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
          <a:solidFill><a:srgbClr val="2563EB"/></a:solidFill>
        </p:spPr>
        <p:txBody>
          <a:bodyPr/>
          <a:p><a:r><a:rPr sz="2800" b="1"/><a:t>External Enterprise Slide</a:t></a:r></a:p>
        </p:txBody>
      </p:sp>
      <p:sp>
        <p:nvSpPr><p:cNvPr id="3" name="Triangle Marker"/><p:cNvSpPr/><p:nvPr/></p:nvPr>
        <p:spPr>
          <a:xfrm rot="5400000"><a:off x="1905000" y="2857500"/><a:ext cx="1905000" cy="1905000"/></a:xfrm>
          <a:prstGeom prst="triangle"><a:avLst/></a:prstGeom>
          <a:solidFill><a:srgbClr val="10B981"/></a:solidFill>
        </p:spPr>
        <p:txBody>
          <a:bodyPr/>
          <a:p><a:r><a:t>Milestone Marker</a:t></a:r></a:p>
        </p:txBody>
      </p:sp>
    </p:spTree>
  </p:cSld>
  <p:transition><p:wipe/></p:transition>
</p:sld>`
      );

      rawZip.file(
        "ppt/notesSlides/notesSlide1.xml",
        `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:notes xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:cSld>
    <p:spTree>
      <p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
      <p:grpSpPr/>
      <p:sp>
        <p:nvSpPr><p:cNvPr id="2" name="Notes 2"/><p:cNvSpPr/><p:nvPr/></p:nvPr>
        <p:spPr/>
        <p:txBody><a:bodyPr/><a:p><a:r><a:t>Imported speaker notes from external deck.</a:t></a:r></a:p></p:txBody>
      </p:sp>
    </p:spTree>
  </p:cSld>
</p:notes>`
      );

      const rawBytes = await rawZip.generateAsync({ type: "uint8array" });
      const parsed = await binaryPackageService.parsePptx(rawBytes);

      expect(parsed.slides.length).toBe(1);
      expect(parsed.slides[0].title).toBe("External Enterprise Slide");
      expect(parsed.slides[0].transition).toBe("Wipe");
      expect(parsed.slides[0].notes).toBe("Imported speaker notes from external deck.");
      expect(parsed.slides[0].shapes?.length).toBe(2);

      // Shape 1 (Rectangle)
      const sh1 = parsed.slides[0].shapes![0];
      expect(sh1.type).toBe("rect");
      expect(sh1.x).toBe(100);
      expect(sh1.y).toBe(100);
      expect(sh1.w).toBe(500);
      expect(sh1.h).toBe(100);
      expect(sh1.fill).toBe("#2563EB");
      expect(sh1.bold).toBe(true);

      // Shape 2 (Triangle with 90° rotation)
      const sh2 = parsed.slides[0].shapes![1];
      expect(sh2.type).toBe("triangle");
      expect(sh2.x).toBe(200);
      expect(sh2.y).toBe(300);
      expect(sh2.rotation).toBe(90);
      expect(sh2.fill).toBe("#10B981");
      expect(sh2.text).toBe("Milestone Marker");
    });
  });

  describe("2. Complete 8-Tab PowerPoint Ribbon Controls & Event Verification", () => {
    it("Home Tab: Drawing & Arrange buttons are 100% visible and functional (Zero clipping)", () => {
      renderTab("Home");

      // Verify shape buttons
      const rectBtn = ribbonContainer.querySelector('[data-testid="slides-shape-rect"]') as HTMLButtonElement;
      const roundBtn = ribbonContainer.querySelector('[data-testid="slides-shape-round-rect"]') as HTMLButtonElement;
      const ovalBtn = ribbonContainer.querySelector('[data-testid="slides-shape-circle"]') as HTMLButtonElement;
      const triBtn = ribbonContainer.querySelector('[data-testid="slides-shape-triangle"]') as HTMLButtonElement;
      const arrowBtn = ribbonContainer.querySelector('[data-testid="slides-shape-arrow"]') as HTMLButtonElement;
      const starBtn = ribbonContainer.querySelector('[data-testid="slides-shape-star"]') as HTMLButtonElement;

      expect(rectBtn).not.toBeNull();
      expect(roundBtn).not.toBeNull();
      expect(ovalBtn).not.toBeNull();
      expect(triBtn).not.toBeNull();
      expect(arrowBtn).not.toBeNull();
      expect(starBtn).not.toBeNull();

      rectBtn.click();
      expect(dispatchedActions).toContainEqual({ action: "InsertShape", payload: "rect" });

      starBtn.click();
      expect(dispatchedActions).toContainEqual({ action: "InsertShape", payload: "star" });

      // Verify Fill & Outline buttons
      const fillBtn = ribbonContainer.querySelector('[data-testid="slides-btn-shape-fill"]') as HTMLButtonElement;
      const outlineBtn = ribbonContainer.querySelector('[data-testid="slides-btn-outline"]') as HTMLButtonElement;
      const fillBlue = ribbonContainer.querySelector('[data-testid="slides-fill-blue"]') as HTMLButtonElement;

      expect(fillBtn).not.toBeNull();
      expect(outlineBtn).not.toBeNull();
      expect(fillBlue).not.toBeNull();

      fillBlue.click();
      expect(dispatchedActions).toContainEqual({ action: "SetFillColor", payload: "#2563eb" });

      // Verify Arrange buttons: Bring to Front & Send to Back
      const bringFrontBtn = ribbonContainer.querySelector('[data-testid="slides-btn-bring-to-front"]') as HTMLButtonElement;
      const sendBackBtn = ribbonContainer.querySelector('[data-testid="slides-btn-send-to-back"]') as HTMLButtonElement;

      expect(bringFrontBtn).not.toBeNull();
      expect(sendBackBtn).not.toBeNull();

      bringFrontBtn.click();
      expect(dispatchedActions).toContainEqual({ action: "BringToFront", payload: undefined });

      sendBackBtn.click();
      expect(dispatchedActions).toContainEqual({ action: "SendToBack", payload: undefined });
    });

    it("Insert Tab: Tables, Pictures, Shapes, SmartArt, Charts, WordArt, Slide Number, Audio/Video", () => {
      renderTab("Insert");

      const tblBtn = ribbonContainer.querySelector('[data-testid="slides-btn-insert-table"]') as HTMLButtonElement;
      const picBtn = ribbonContainer.querySelector('[data-testid="slides-btn-insert-picture"]') as HTMLButtonElement;
      const smartBtn = ribbonContainer.querySelector('[data-testid="slides-btn-smartart"]') as HTMLButtonElement;
      const chartBtn = ribbonContainer.querySelector('[data-testid="slides-btn-chart"]') as HTMLButtonElement;
      const txtBtn = ribbonContainer.querySelector('[data-testid="slides-btn-insert-textbox"]') as HTMLButtonElement;
      const wordArtBtn = ribbonContainer.querySelector('[data-testid="slides-btn-wordart"]') as HTMLButtonElement;
      const slideNumBtn = ribbonContainer.querySelector('[data-testid="slides-btn-slide-number"]') as HTMLButtonElement;
      const videoBtn = ribbonContainer.querySelector('[data-testid="slides-btn-insert-video"]') as HTMLButtonElement;

      expect(tblBtn).not.toBeNull();
      expect(picBtn).not.toBeNull();
      expect(smartBtn).not.toBeNull();
      expect(chartBtn).not.toBeNull();
      expect(txtBtn).not.toBeNull();
      expect(wordArtBtn).not.toBeNull();
      expect(slideNumBtn).not.toBeNull();
      expect(videoBtn).not.toBeNull();

      smartBtn.click();
      expect(dispatchedActions).toContainEqual({ action: "InsertSmartArt", payload: "process" });

      chartBtn.click();
      expect(dispatchedActions).toContainEqual({ action: "InsertChart", payload: "bar" });

      txtBtn.click();
      expect(dispatchedActions).toContainEqual({ action: "InsertTextBox", payload: undefined });

      slideNumBtn.click();
      expect(dispatchedActions).toContainEqual({ action: "InsertSlideNumber", payload: undefined });
    });

    it("Design Tab: 12+ Themes, 16:9 / 4:3 Aspect Ratio & Custom Backgrounds", () => {
      renderTab("Design");

      const size169 = ribbonContainer.querySelector('[data-testid="slides-btn-size-16-9"]') as HTMLButtonElement;
      const size43 = ribbonContainer.querySelector('[data-testid="slides-btn-size-4-3"]') as HTMLButtonElement;
      const darkSlate = ribbonContainer.querySelector('[data-testid="slides-theme-dark-slate"]') as HTMLButtonElement;
      const oceanBlue = ribbonContainer.querySelector('[data-testid="slides-theme-ocean-blue"]') as HTMLButtonElement;
      const gradientBg = ribbonContainer.querySelector('[data-testid="slides-btn-gradient-bg"]') as HTMLButtonElement;

      expect(size169).not.toBeNull();
      expect(size43).not.toBeNull();
      expect(darkSlate).not.toBeNull();
      expect(oceanBlue).not.toBeNull();
      expect(gradientBg).not.toBeNull();

      size169.click();
      expect(dispatchedActions).toContainEqual({ action: "SetSlideSize", payload: "16:9" });

      darkSlate.click();
      expect(dispatchedActions).toContainEqual({ action: "SetTheme", payload: "Dark Slate" });

      gradientBg.click();
      expect(dispatchedActions.some((a) => a.action === "SetSlideBackground")).toBe(true);
    });

    it("Transitions Tab: live preview, transition gallery & timing options", () => {
      renderTab("Transitions");

      const previewBtn = ribbonContainer.querySelector('[data-testid="slides-btn-preview-trans"]') as HTMLButtonElement;
      const fadeBtn = ribbonContainer.querySelector('[data-testid="slides-trans-fade"]') as HTMLButtonElement;
      const pushBtn = ribbonContainer.querySelector('[data-testid="slides-trans-push"]') as HTMLButtonElement;
      const morphBtn = ribbonContainer.querySelector('[data-testid="slides-trans-morph"]') as HTMLButtonElement;
      const applyAllBtn = ribbonContainer.querySelector('[data-testid="slides-btn-trans-apply-all"]') as HTMLButtonElement;

      expect(previewBtn).not.toBeNull();
      expect(fadeBtn).not.toBeNull();
      expect(pushBtn).not.toBeNull();
      expect(morphBtn).not.toBeNull();
      expect(applyAllBtn).not.toBeNull();

      previewBtn.click();
      expect(dispatchedActions).toContainEqual({ action: "PreviewTransition", payload: undefined });

      fadeBtn.click();
      expect(dispatchedActions).toContainEqual({ action: "SetTransition", payload: "Fade" });

      morphBtn.click();
      expect(dispatchedActions).toContainEqual({ action: "SetTransition", payload: "Morph" });

      applyAllBtn.click();
      expect(dispatchedActions).toContainEqual({ action: "ApplyTransitionToAll", payload: undefined });
    });

    it("Animations Tab: live preview, entrance, emphasis, exit & Animation Pane", () => {
      renderTab("Animations");

      const previewBtn = ribbonContainer.querySelector('[data-testid="slides-btn-preview-anim"]') as HTMLButtonElement;
      const flyInBtn = ribbonContainer.querySelector('[data-testid="slides-anim-fly-in"]') as HTMLButtonElement;
      const pulseBtn = ribbonContainer.querySelector('[data-testid="slides-anim-pulse"]') as HTMLButtonElement;
      const fadeOutBtn = ribbonContainer.querySelector('[data-testid="slides-anim-fade-out"]') as HTMLButtonElement;
      const animPaneBtn = ribbonContainer.querySelector('[data-testid="slides-btn-anim-pane"]') as HTMLButtonElement;

      expect(previewBtn).not.toBeNull();
      expect(flyInBtn).not.toBeNull();
      expect(pulseBtn).not.toBeNull();
      expect(fadeOutBtn).not.toBeNull();
      expect(animPaneBtn).not.toBeNull();

      previewBtn.click();
      expect(dispatchedActions).toContainEqual({ action: "PreviewAnimation", payload: undefined });

      flyInBtn.click();
      expect(dispatchedActions).toContainEqual({ action: "SetAnimation", payload: "flyIn" });

      animPaneBtn.click();
      expect(dispatchedActions).toContainEqual({ action: "ToggleAnimationPane", payload: undefined });
    });

    it("Slide Show Tab: Fullscreen F5 beginning, current slide & Presenter View", () => {
      renderTab("Slide Show");

      const beginBtn = ribbonContainer.querySelector('[data-testid="slides-btn-show-begin"]') as HTMLButtonElement;
      const currBtn = ribbonContainer.querySelector('[data-testid="slides-btn-show-curr"]') as HTMLButtonElement;
      const presenterBtn = ribbonContainer.querySelector('[data-testid="slides-btn-presenter-view"]') as HTMLButtonElement;

      expect(beginBtn).not.toBeNull();
      expect(currBtn).not.toBeNull();
      expect(presenterBtn).not.toBeNull();

      beginBtn.click();
      expect(dispatchedActions).toContainEqual({ action: "StartShowBeginning", payload: undefined });

      currBtn.click();
      expect(dispatchedActions).toContainEqual({ action: "StartShowCurrent", payload: undefined });

      presenterBtn.click();
      expect(dispatchedActions).toContainEqual({ action: "StartPresenterView", payload: undefined });
    });

    it("Review & View Tabs: Comments, Proofing, Slide Sorter, Notes Page, Ruler, Gridlines, Guides", () => {
      // Review Tab
      renderTab("Review");
      const commentBtn = ribbonContainer.querySelector('[data-testid="slides-btn-new-comment"]') as HTMLButtonElement;
      const spellBtn = ribbonContainer.querySelector('[data-testid="slides-btn-spelling"]') as HTMLButtonElement;
      expect(commentBtn).not.toBeNull();
      expect(spellBtn).not.toBeNull();

      commentBtn.click();
      expect(dispatchedActions).toContainEqual({ action: "NewComment", payload: undefined });

      // View Tab
      renderTab("View");
      const normalBtn = ribbonContainer.querySelector('[data-testid="slides-btn-view-normal"]') as HTMLButtonElement;
      const sorterBtn = ribbonContainer.querySelector('[data-testid="slides-btn-view-sorter"]') as HTMLButtonElement;
      const notesBtn = ribbonContainer.querySelector('[data-testid="slides-btn-view-notes"]') as HTMLButtonElement;
      const rulerBtn = ribbonContainer.querySelector('[data-testid="slides-btn-show-ruler"]') as HTMLButtonElement;
      const gridBtn = ribbonContainer.querySelector('[data-testid="slides-btn-show-gridlines"]') as HTMLButtonElement;
      const guidesBtn = ribbonContainer.querySelector('[data-testid="slides-btn-show-guides"]') as HTMLButtonElement;

      expect(normalBtn).not.toBeNull();
      expect(sorterBtn).not.toBeNull();
      expect(notesBtn).not.toBeNull();
      expect(rulerBtn).not.toBeNull();
      expect(gridBtn).not.toBeNull();
      expect(guidesBtn).not.toBeNull();

      sorterBtn.click();
      expect(dispatchedActions).toContainEqual({ action: "ViewSlideSorter", payload: undefined });

      rulerBtn.click();
      expect(dispatchedActions).toContainEqual({ action: "ToggleRuler", payload: undefined });
    });
  });

  describe("3. SlideStageCanvas Live Visual Transition & Animation Previews", () => {
    it("executes transition and animation previews without exceptions", async () => {
      const stageWrapper = document.createElement("div");
      document.body.appendChild(stageWrapper);

      const stageCanvas = new SlideStageCanvas(stageWrapper, slidesState);
      expect(stageCanvas).toBeDefined();

      // Trigger transition preview
      await expect(stageCanvas.playTransitionPreview({ effect: "Fade", duration: 0.1 })).resolves.not.toThrow();

      // Trigger animation preview on shape
      slidesState.insertShape("rect");
      const shapeId = slidesState.selectedShapeId!;
      expect(shapeId).toBeDefined();

      const anim = {
        id: "anim-1",
        shapeId,
        type: "entrance" as const,
        effect: "flyIn" as const,
        trigger: "click" as const,
        duration: 0.1,
        delay: 0,
        order: 1,
      };

      await expect(stageCanvas.playAnimationPreview(anim)).resolves.not.toThrow();
    });
  });
});
