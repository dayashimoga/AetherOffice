import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { PdfState, pdfState } from "../state/pdfState";
import { PdfRibbon } from "../components/ribbon/PdfRibbon";
import { PdfWorkspace } from "../apps/pdf/PdfWorkspace";
import { UnifiedShellApp } from "../main";

describe("GEC-v5.0 Subphase 32.1.2: PDF Studio Complete Ribbon Suite & In-Place Editing", () => {
  let state: PdfState;
  let ribbon: PdfRibbon;
  let ribbonBody: HTMLElement;
  let app: UnifiedShellApp;
  let actions: { action: string; payload?: unknown }[];

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
      measureText: (txt: string) => ({ width: (txt || "").length * 8 }),
      setLineDash: () => {},
      font: "14px Segoe UI, sans-serif",
      fillStyle: "#000",
      strokeStyle: "#000",
      lineWidth: 1,
      textAlign: "left",
      textBaseline: "top",
    };
    HTMLCanvasElement.prototype.getContext = (() => mockCtx) as any;
    if (!HTMLCanvasElement.prototype.scrollIntoView) {
      HTMLCanvasElement.prototype.scrollIntoView = () => {};
    }

    document.body.innerHTML = `
      <div id="app" class="desktop-shell">
        <header id="tab-bar-container" class="tab-bar"></header>
        <header id="ribbon-container" class="ribbon-bar"></header>
        <div id="sub-header-container" class="sub-header-bar"></div>
        <div class="main-content" style="display:flex; height:600px;">
          <aside id="sidebar-container" class="sidebar-pane"></aside>
          <main id="viewport-container" style="flex:1;"></main>
        </div>
        <footer id="status-bar" class="status-bar">
          <div class="status-left">
            <span id="status-doc-info"></span>
            <span id="status-word-count"></span>
          </div>
        </footer>
      </div>
    `;

    state = pdfState;
    state.resetToBlank("ProductionAudit.pdf");

    app = new UnifiedShellApp(false);
    app.getTabBar().addTab({
      id: "tab-pdf-gec5",
      docType: "PDF",
      title: "ProductionAudit.pdf",
      isDirty: false,
    });
    app.getTabBar().switchTab("tab-pdf-gec5");
    (app as any).currentDocType = "PDF";

    ribbon = new PdfRibbon();
    ribbonBody = document.createElement("div");
    ribbonBody.className = "ribbon-body";
    document.body.appendChild(ribbonBody);

    const vpContainer = document.getElementById("viewport-container")!;
    new PdfWorkspace(vpContainer, state);

    actions = [];
  });

  afterEach(() => {
    document.body.innerHTML = "";
  });

  // =========================================================================
  // 1. HOME TAB: COMPLETE TOOLS, ZOOM, PAGE NAVIGATION & SEARCH HIGHLIGHTS
  // =========================================================================
  describe("1. Home Tab Complete Suite", () => {
    it("wires Open, Blank PDF, Hand Tool, Select Tool, Snapshot, Zoom, Prev/Next/First/Last Page & Search", () => {
      ribbon.renderTab("Home", ribbonBody, (action, payload) => {
        actions.push({ action, payload });
        app.handleRibbonAction(action, payload);
      });

      const buttons = ribbonBody.querySelectorAll<HTMLButtonElement>("button");
      expect(buttons.length).toBeGreaterThanOrEqual(13);

      // Verify all buttons have click handlers and fire valid actions
      buttons.forEach((btn) => {
        btn.click();
      });

      const actionNames = actions.map((a) => a.action);
      expect(actionNames).toContain("OpenPdf");
      expect(actionNames).toContain("CreateBlankPdf");
      expect(actionNames).toContain("SetPdfTool");
      expect(actionNames).toContain("Zoom100");
      expect(actionNames).toContain("ZoomFitPage");
      expect(actionNames).toContain("ZoomFitWidth");
      expect(actionNames).toContain("ZoomIn");
      expect(actionNames).toContain("ZoomOut");
      expect(actionNames).toContain("FirstPage");
      expect(actionNames).toContain("PrevPage");
      expect(actionNames).toContain("NextPage");
      expect(actionNames).toContain("LastPage");
      expect(actionNames).toContain("Find");
      expect(actionNames).toContain("PrintDocument");
      expect(actionNames).toContain("SaveDocument");

      // Verify Search Text updates state search matches
      state.addTextBlock(1, 100, 100, "ISO 32000-2 Standard Document Specification");
      state.searchText("ISO");
      expect(state.searchQuery).toBe("iso");
      expect(state.searchMatches.length).toBeGreaterThan(0);
      expect(state.searchMatches[0].text).toContain("ISO 32000-2");
    });
  });

  // =========================================================================
  // 2. EDIT PDF TAB: IN-PLACE TEXT, IMAGES, CROP, WATERMARK, HEADER/FOOTER & ALIGN
  // =========================================================================
  describe("2. Edit PDF Tab Complete Suite", () => {
    it("wires in-place text editing, font styling, alignment, delete element, images and crop page", () => {
      ribbon.renderTab("Edit PDF", ribbonBody, (action, payload) => {
        actions.push({ action, payload });
        app.handleRibbonAction(action, payload);
      });

      const buttons = ribbonBody.querySelectorAll<HTMLButtonElement>("button");
      expect(buttons.length).toBeGreaterThanOrEqual(12);

      buttons.forEach((btn) => btn.click());

      const actionNames = actions.map((a) => a.action);
      expect(actionNames).toContain("AddPdfText");
      expect(actionNames).toContain("EditPdfText");
      expect(actionNames).toContain("TogglePdfBold");
      expect(actionNames).toContain("TogglePdfItalic");
      expect(actionNames).toContain("SetPdfAlign");
      expect(actionNames).toContain("DeletePdfElement");
      expect(actionNames).toContain("AddPdfImage");
      expect(actionNames).toContain("ReplacePdfImage");
      expect(actionNames).toContain("CropPdfImage");
      expect(actionNames).toContain("RotatePdfImage");
      expect(actionNames).toContain("CropPage");
      expect(actionNames).toContain("AddWatermark");
      expect(actionNames).toContain("AddHeaderFooter");
      expect(actionNames).toContain("SetPdfBackgroundColor");
      expect(actionNames).toContain("AddWebLink");
      expect(actionNames).toContain("AddPageJumpLink");

      // Verify Select dropdowns
      const selects = ribbonBody.querySelectorAll<HTMLSelectElement>("select");
      expect(selects.length).toBeGreaterThanOrEqual(2);
      selects[0].value = "Courier";
      selects[0].dispatchEvent(new Event("change"));
      selects[1].value = "18";
      selects[1].dispatchEvent(new Event("change"));

      expect(actions.some((a) => a.action === "SetPdfFontFamily" && a.payload === "Courier")).toBe(true);
      expect(actions.some((a) => a.action === "SetPdfFontSize" && a.payload === 18)).toBe(true);
    });
  });

  // =========================================================================
  // 3. FORMS TAB: ACROFORMS BUILDER, HIGHLIGHT FIELDS, CLEAR & EXPORT
  // =========================================================================
  describe("3. Forms Tab Complete Suite", () => {
    it("wires Form Fields, Highlight Fields toggle, Clear, and Import/Export", () => {
      ribbon.renderTab("Forms", ribbonBody, (action, payload) => {
        actions.push({ action, payload });
        app.handleRibbonAction(action, payload);
      });

      const buttons = ribbonBody.querySelectorAll<HTMLButtonElement>("button");
      buttons.forEach((btn) => btn.click());

      const actionNames = actions.map((a) => a.action);
      expect(actionNames).toContain("AddFormField");
      expect(actionNames).toContain("HighlightFormFields");
      expect(actionNames).toContain("AlignFormFields");
      expect(actionNames).toContain("DistributeFormFields");
      expect(actionNames).toContain("ClearFormData");
      expect(actionNames).toContain("ImportFormData");
      expect(actionNames).toContain("ExportFormData");

      // Verify field highlighting is toggled
      expect(state.highlightFields).toBe(true);
    });
  });

  // =========================================================================
  // 4. COMMENT & ANNOTATE TAB: INK PEN, THICKNESS, STAMPS, SHAPES & REDACTION
  // =========================================================================
  describe("4. Comment & Annotate Tab Complete Suite", () => {
    it("wires annotations, ink controls, stamps including Signed and Urgent, shapes and redaction", () => {
      ribbon.renderTab("Comment & Annotate", ribbonBody, (action, payload) => {
        actions.push({ action, payload });
        app.handleRibbonAction(action, payload);
      });

      const buttons = ribbonBody.querySelectorAll<HTMLButtonElement>("button");
      buttons.forEach((btn) => btn.click());

      const actionNames = actions.map((a) => a.action);
      expect(actionNames).toContain("SetAnnotTool");
      expect(actionNames).toContain("AddStamp");
      expect(actionNames).toContain("InsertShape");
      expect(actionNames).toContain("MarkRedaction");
      expect(actionNames).toContain("ApplyRedactions");

      // Verify stamps include Signed and Urgent
      const stampsApplied = actions.filter((a) => a.action === "AddStamp").map((a) => a.payload);
      expect(stampsApplied).toContain("signed");
      expect(stampsApplied).toContain("urgent");
      expect(stampsApplied).toContain("approved");
      expect(stampsApplied).toContain("confidential");

      // Verify ink selects
      const selects = ribbonBody.querySelectorAll<HTMLSelectElement>("select");
      expect(selects.length).toBeGreaterThanOrEqual(2);
      selects[0].value = "#15803d";
      selects[0].dispatchEvent(new Event("change"));
      selects[1].value = "5";
      selects[1].dispatchEvent(new Event("change"));

      expect(actions.some((a) => a.action === "SetInkColor" && a.payload === "#15803d")).toBe(true);
      expect(actions.some((a) => a.action === "SetInkThickness" && a.payload === 5)).toBe(true);
    });
  });

  // =========================================================================
  // 5. PAGE TOOLS TAB: RESIZE SIZES, INSERT/DELETE/DUPLICATE & SPLIT/MERGE
  // =========================================================================
  describe("5. Page Tools Tab Complete Suite", () => {
    it("wires Page Tools operations, multi-page manipulation, and page sizing", () => {
      ribbon.renderTab("Page Tools", ribbonBody, (action, payload) => {
        actions.push({ action, payload });
        app.handleRibbonAction(action, payload);
      });

      const buttons = ribbonBody.querySelectorAll<HTMLButtonElement>("button");
      buttons.forEach((btn) => btn.click());

      const actionNames = actions.map((a) => a.action);
      expect(actionNames).toContain("RotatePageCW");
      expect(actionNames).toContain("RotatePageCCW");
      expect(actionNames).toContain("DeletePage");
      expect(actionNames).toContain("InsertBlankPage");
      expect(actionNames).toContain("DuplicatePage");
      expect(actionNames).toContain("MovePageUp");
      expect(actionNames).toContain("MovePageDown");
      expect(actionNames).toContain("ExtractPage");
      expect(actionNames).toContain("SplitPdf");
      expect(actionNames).toContain("MergePdf");
      expect(actionNames).toContain("SetPageSize");
      expect(actionNames).toContain("ResizePageModal");
    });
  });

  // =========================================================================
  // 6. SIGN & PROTECT TAB: SIGNATURE STUDIO, PASSWORD ENCRYPT/DECRYPT & REDACTION
  // =========================================================================
  describe("6. Sign & Protect Tab Complete Suite", () => {
    it("wires Signature Studio, Password Protect, Unlock Screen, Decrypt and Redaction", () => {
      ribbon.renderTab("Sign & Protect", ribbonBody, (action, payload) => {
        actions.push({ action, payload });
        app.handleRibbonAction(action, payload);
      });

      const buttons = ribbonBody.querySelectorAll<HTMLButtonElement>("button");
      buttons.forEach((btn) => btn.click());

      const actionNames = actions.map((a) => a.action);
      expect(actionNames).toContain("AddSignature");
      expect(actionNames).toContain("AddInitials");
      expect(actionNames).toContain("SignPdf");
      expect(actionNames).toContain("ExportPdf");
      expect(actionNames).toContain("EncryptPassword");
      expect(actionNames).toContain("DecryptPassword");
      expect(actionNames).toContain("CompressPdf");
      expect(actionNames).toContain("RestrictEditing");
      expect(actionNames).toContain("MarkRedaction");
      expect(actionNames).toContain("ApplyRedactions");

      // Verify Password Protect activates Lock Screen in PdfWorkspace
      state.encryptWithPassword("ExecutivePass2026");
      expect(state.isEncrypted).toBe(true);

      const vp = document.getElementById("viewport-container")!;
      const workspace = new PdfWorkspace(vp, state);
      workspace.render();

      const lockScreen = vp.querySelector('[data-testid="pdf-lock-screen"]');
      expect(lockScreen).not.toBeNull();
      expect(lockScreen?.textContent).toContain("Document is Password Protected");

      // Unlock via password input
      const input = vp.querySelector<HTMLInputElement>('[data-testid="pdf-unlock-input"]');
      const unlockBtn = vp.querySelector<HTMLButtonElement>('[data-testid="pdf-unlock-btn"]');
      expect(input).not.toBeNull();
      expect(unlockBtn).not.toBeNull();

      if (input && unlockBtn) {
        input.value = "ExecutivePass2026";
        unlockBtn.click();
      }

      expect(state.isEncrypted).toBe(false);
      expect(vp.querySelector('[data-testid="pdf-lock-screen"]')).toBeNull();
    });
  });

  // =========================================================================
  // 7. VIEW TAB: PAGE DISPLAY, ROTATION, NAVIGATION PANES & FULL SCREEN
  // =========================================================================
  describe("7. View Tab Complete Suite", () => {
    it("wires Single, Continuous, Two-Page, Rotate View and Panes", () => {
      ribbon.renderTab("View", ribbonBody, (action, payload) => {
        actions.push({ action, payload });
        app.handleRibbonAction(action, payload);
      });

      const buttons = ribbonBody.querySelectorAll<HTMLButtonElement>("button");
      buttons.forEach((btn) => btn.click());

      const actionNames = actions.map((a) => a.action);
      expect(actionNames).toContain("SetPageLayout");
      expect(actionNames).toContain("RotateView");
      expect(actionNames).toContain("ToggleBookmarks");
      expect(actionNames).toContain("ToggleThumbnails");
      expect(actionNames).toContain("ToggleAttachments");
      expect(actionNames).toContain("ToggleFullScreen");
    });
  });
});
