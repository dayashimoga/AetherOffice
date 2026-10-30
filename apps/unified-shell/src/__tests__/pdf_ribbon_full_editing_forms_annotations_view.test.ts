import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { PdfState, pdfState } from "../state/pdfState";
import { PdfRibbon } from "../components/ribbon/PdfRibbon";
import { PdfWorkspace } from "../apps/pdf/PdfWorkspace";
import { UnifiedShellApp } from "../main";

describe("PDF Direct Text/Image Editing, Annotations, Redaction, Forms & Signatures (GEC-v4.0 Subphase 32.1.2)", () => {
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
      measureText: (txt: string) => ({ width: txt.length * 8 }),
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
    state.resetToBlank("EditableDocument.pdf");

    app = new UnifiedShellApp(false);
    app.getTabBar().addTab({
      id: "tab-pdf-full",
      docType: "PDF",
      title: "EditableDocument.pdf",
      isDirty: false,
    });
    app.getTabBar().switchTab("tab-pdf-full");
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
  // 1. HOME TAB (TOOLS, ZOOM, SEARCH, PRINT, SAVE)
  // =========================================================================
  describe("1. Home Tab", () => {
    beforeEach(() => {
      actions = [];
      ribbon.renderTab("Home", ribbonBody, (action, payload) => {
        actions.push({ action, payload });
        app.handleRibbonAction(action, payload);
      });
      state.createBlankPdf(3);
    });

    it("1.1 triggers tools: Open, Hand, Select, Snapshot", () => {
      const openBtn = ribbonBody.querySelector("[data-testid='pdf-btn-open-pdf']") as HTMLButtonElement;
      const handBtn = ribbonBody.querySelector("[data-testid='pdf-btn-hand']") as HTMLButtonElement;
      const selectBtn = ribbonBody.querySelector("[data-testid='pdf-btn-select']") as HTMLButtonElement;
      const snapshotBtn = ribbonBody.querySelector("[data-testid='pdf-btn-snapshot']") as HTMLButtonElement;

      openBtn.click();
      expect(actions).toContainEqual({ action: "OpenPdf", payload: undefined });

      handBtn.click();
      expect(actions).toContainEqual({ action: "SetPdfTool", payload: "hand" });
      expect(state.activeTool).toBe("hand");

      selectBtn.click();
      expect(actions).toContainEqual({ action: "SetPdfTool", payload: "select" });
      expect(state.activeTool).toBe("select");

      snapshotBtn.click();
      expect(actions).toContainEqual({ action: "SetPdfTool", payload: "snapshot" });
      expect(state.activeTool).toBe("snapshot");
    });

    it("1.2 triggers zoom and navigation: 100%, Fit Page, Fit Width, Prev, Next, Zoom In, Zoom Out", () => {
      const z100 = ribbonBody.querySelector("[data-testid='pdf-btn-zoom-100']") as HTMLButtonElement;
      const zFitPage = ribbonBody.querySelector("[data-testid='pdf-btn-fit-page']") as HTMLButtonElement;
      const zFitWidth = ribbonBody.querySelector("[data-testid='pdf-btn-fit-width']") as HTMLButtonElement;
      const prevBtn = ribbonBody.querySelector("[data-testid='pdf-btn-prev']") as HTMLButtonElement;
      const nextBtn = ribbonBody.querySelector("[data-testid='pdf-btn-next']") as HTMLButtonElement;
      const zIn = ribbonBody.querySelector("[data-testid='pdf-btn-zoom-in']") as HTMLButtonElement;
      const zOut = ribbonBody.querySelector("[data-testid='pdf-btn-zoom-out']") as HTMLButtonElement;

      zFitPage.click();
      expect(state.zoom).toBe(0.8);

      zFitWidth.click();
      expect(state.zoom).toBe(1.25);

      z100.click();
      expect(state.zoom).toBe(1.0);

      zIn.click();
      expect(state.zoom).toBe(1.25);

      zOut.click();
      expect(state.zoom).toBe(1.0);

      state.currentPage = 1;
      nextBtn.click();
      expect(state.currentPage).toBe(2);

      prevBtn.click();
      expect(state.currentPage).toBe(1);
    });

    it("1.3 triggers Search, Print, and Quick Save", () => {
      const searchBtn = ribbonBody.querySelector("[data-testid='pdf-btn-search']") as HTMLButtonElement;
      const printBtn = ribbonBody.querySelector("[data-testid='pdf-btn-print']") as HTMLButtonElement;
      const saveBtn = ribbonBody.querySelector("[data-testid='pdf-btn-save']") as HTMLButtonElement;

      searchBtn.click();
      expect(actions.some((a) => a.action === "Find")).toBe(true);

      printBtn.click();
      expect(actions.some((a) => a.action === "PrintDocument")).toBe(true);

      saveBtn.click();
      expect(actions.some((a) => a.action === "SaveDocument")).toBe(true);
    });
  });

  // =========================================================================
  // 2. EDIT PDF TAB (TEXT, IMAGES, WATERMARK, HEADER/FOOTER, BACKGROUND)
  // =========================================================================
  describe("2. Edit PDF Tab", () => {
    beforeEach(() => {
      actions = [];
      ribbon.renderTab("Edit PDF", ribbonBody, (action, payload) => {
        actions.push({ action, payload });
        app.handleRibbonAction(action, payload);
      });
      state.createBlankPdf(2);
    });

    it("2.1 formats text: Add Text, Edit Text, Bold, Italic, Color, Font Family, Font Size", () => {
      const addTextBtn = ribbonBody.querySelector("[data-testid='pdf-btn-add-text']") as HTMLButtonElement;
      const editTextBtn = ribbonBody.querySelector("[data-testid='pdf-btn-edit-text']") as HTMLButtonElement;
      const boldBtn = ribbonBody.querySelector("[data-testid='pdf-btn-bold']") as HTMLButtonElement;
      const italicBtn = ribbonBody.querySelector("[data-testid='pdf-btn-italic']") as HTMLButtonElement;
      const colorBtn = ribbonBody.querySelector("[data-testid='pdf-btn-color']") as HTMLButtonElement;

      addTextBtn.click();
      expect(actions).toContainEqual({ action: "AddPdfText", payload: undefined });
      expect(state.pages[0].textBlocks?.length).toBeGreaterThan(0);

      editTextBtn.click();
      expect(actions).toContainEqual({ action: "EditPdfText", payload: undefined });
      expect(state.activeTool).toBe("edit_text");

      boldBtn.click();
      expect(state.activeBold).toBe(true);

      italicBtn.click();
      expect(state.activeItalic).toBe(true);

      colorBtn.click();
      expect(state.activeFontColor).toBe("#000000");

      const fontSelect = ribbonBody.querySelector("[data-testid='pdf-select-font-family']") as HTMLSelectElement;
      fontSelect.value = "Arial";
      fontSelect.dispatchEvent(new Event("change"));
      expect(state.activeFontFamily).toBe("Arial");

      const sizeSelect = ribbonBody.querySelector("[data-testid='pdf-select-font-size']") as HTMLSelectElement;
      sizeSelect.value = "24";
      sizeSelect.dispatchEvent(new Event("change"));
      expect(state.activeFontSize).toBe(24);
    });

    it("2.2 manages images: Add Image, Replace Image, Crop Image, Rotate Image", () => {
      const addImgBtn = ribbonBody.querySelector("[data-testid='pdf-btn-add-image']") as HTMLButtonElement;
      const repImgBtn = ribbonBody.querySelector("[data-testid='pdf-btn-replace-image']") as HTMLButtonElement;
      const cropImgBtn = ribbonBody.querySelector("[data-testid='pdf-btn-crop-image']") as HTMLButtonElement;
      const rotImgBtn = ribbonBody.querySelector("[data-testid='pdf-btn-rotate-image']") as HTMLButtonElement;

      addImgBtn.click();
      expect(actions).toContainEqual({ action: "AddPdfImage", payload: undefined });
      expect(state.pages[0].imageBlocks?.length).toBeGreaterThan(0);

      repImgBtn.click();
      expect(actions).toContainEqual({ action: "ReplacePdfImage", payload: undefined });

      cropImgBtn.click();
      expect(actions).toContainEqual({ action: "CropPdfImage", payload: undefined });

      rotImgBtn.click();
      expect(actions).toContainEqual({ action: "RotatePdfImage", payload: undefined });
    });

    it("2.3 stamps Watermark, Header & Footer, Page Color, and Links", () => {
      const watermarkBtn = ribbonBody.querySelector("[data-testid='pdf-btn-add-watermark']") as HTMLButtonElement;
      const hfBtn = ribbonBody.querySelector("[data-testid='pdf-btn-header-footer']") as HTMLButtonElement;
      const bgBtn = ribbonBody.querySelector("[data-testid='pdf-btn-bg-color']") as HTMLButtonElement;
      const webLinkBtn = ribbonBody.querySelector("[data-testid='pdf-btn-web-link']") as HTMLButtonElement;
      const pageLinkBtn = ribbonBody.querySelector("[data-testid='pdf-btn-page-link']") as HTMLButtonElement;

      watermarkBtn.click();
      expect(actions).toContainEqual({ action: "AddWatermark", payload: "CONFIDENTIAL" });
      expect(state.pages[0].watermark?.text).toBe("CONFIDENTIAL");

      hfBtn.click();
      expect(actions).toContainEqual({ action: "AddHeaderFooter", payload: undefined });
      expect(state.pages[0].textBlocks?.some((t) => t.text.includes("Page 1"))).toBe(true);

      bgBtn.click();
      expect(actions).toContainEqual({ action: "SetPdfBackgroundColor", payload: "#f8fafc" });

      webLinkBtn.click();
      expect(actions).toContainEqual({ action: "AddWebLink", payload: undefined });

      pageLinkBtn.click();
      expect(actions).toContainEqual({ action: "AddPageJumpLink", payload: undefined });
    });
  });

  // =========================================================================
  // 3. FORMS TAB (FIELD CREATION, ALIGNMENT & DATA I/O)
  // =========================================================================
  describe("3. Forms Tab", () => {
    beforeEach(() => {
      actions = [];
      ribbon.renderTab("Forms", ribbonBody, (action, payload) => {
        actions.push({ action, payload });
        app.handleRibbonAction(action, payload);
      });
      state.createBlankPdf(1);
    });

    it("3.1 adds Text Field, Checkbox, Radio, Date, Dropdown, Listbox, Signature Field, and Button", () => {
      const fields = [
        { testId: "pdf-btn-form-text", type: "text" },
        { testId: "pdf-btn-form-checkbox", type: "checkbox" },
        { testId: "pdf-btn-form-radio", type: "radio" },
        { testId: "pdf-btn-form-date", type: "date" },
        { testId: "pdf-btn-form-dropdown", type: "dropdown" },
        { testId: "pdf-btn-form-listbox", type: "listbox" },
        { testId: "pdf-btn-form-sig", type: "signature" },
        { testId: "pdf-btn-form-btn", type: "button" },
      ];

      for (const f of fields) {
        const btn = ribbonBody.querySelector(`[data-testid="${f.testId}"]`) as HTMLButtonElement;
        expect(btn).not.toBeNull();
        btn.click();
        expect(actions).toContainEqual({ action: "AddFormField", payload: f.type });
      }

      expect(state.pages[0].formFields?.length).toBe(fields.length);
    });

    it("3.2 triggers Form Tools: Align, Distribute, Clear Form, Import Data, and Export Data", () => {
      const alignBtn = ribbonBody.querySelector("[data-testid='pdf-btn-form-align']") as HTMLButtonElement;
      const distBtn = ribbonBody.querySelector("[data-testid='pdf-btn-form-distribute']") as HTMLButtonElement;
      const clearBtn = ribbonBody.querySelector("[data-testid='pdf-btn-form-clear']") as HTMLButtonElement;
      const importBtn = ribbonBody.querySelector("[data-testid='pdf-btn-form-import']") as HTMLButtonElement;
      const exportBtn = ribbonBody.querySelector("[data-testid='pdf-btn-form-export']") as HTMLButtonElement;

      alignBtn.click();
      expect(actions).toContainEqual({ action: "AlignFormFields", payload: "left" });

      distBtn.click();
      expect(actions).toContainEqual({ action: "DistributeFormFields", payload: undefined });

      clearBtn.click();
      expect(actions).toContainEqual({ action: "ClearFormData", payload: undefined });

      importBtn.click();
      expect(actions).toContainEqual({ action: "ImportFormData", payload: undefined });

      exportBtn.click();
      expect(actions).toContainEqual({ action: "ExportFormData", payload: undefined });
    });
  });

  // =========================================================================
  // 4. COMMENT & ANNOTATE TAB (MARKUP, DRAWING, STAMPS & REDACTION)
  // =========================================================================
  describe("4. Comment & Annotate Tab", () => {
    beforeEach(() => {
      actions = [];
      ribbon.renderTab("Comment & Annotate", ribbonBody, (action, payload) => {
        actions.push({ action, payload });
        app.handleRibbonAction(action, payload);
      });
      state.createBlankPdf(1);
    });

    it("4.1 selects annotation tools: Highlight, Underline, Strike, Note, Pencil, Eraser, Callout", () => {
      const tools = [
        { testId: "pdf-btn-highlight", tool: "highlight" },
        { testId: "pdf-btn-underline", tool: "underline" },
        { testId: "pdf-btn-strike", tool: "strike" },
        { testId: "pdf-btn-note", tool: "note" },
        { testId: "pdf-btn-pencil", tool: "pencil" },
        { testId: "pdf-btn-eraser", tool: "eraser" },
        { testId: "pdf-btn-callout", tool: "callout" },
      ];

      for (const t of tools) {
        const btn = ribbonBody.querySelector(`[data-testid="${t.testId}"]`) as HTMLButtonElement;
        expect(btn).not.toBeNull();
        btn.click();
        expect(actions).toContainEqual({ action: "SetAnnotTool", payload: t.tool });
        expect(state.activeTool).toBe(t.tool);
      }
    });

    it("4.2 applies Stamps (Approved, Confidential, Draft, Reviewed, Final, Void)", () => {
      const stamps = [
        { testId: "pdf-btn-stamp-approved", name: "approved" },
        { testId: "pdf-btn-stamp-confidential", name: "confidential" },
        { testId: "pdf-btn-stamp-draft", name: "draft" },
        { testId: "pdf-btn-stamp-reviewed", name: "reviewed" },
        { testId: "pdf-btn-stamp-final", name: "final" },
        { testId: "pdf-btn-stamp-void", name: "void" },
      ];

      for (const s of stamps) {
        const btn = ribbonBody.querySelector(`[data-testid="${s.testId}"]`) as HTMLButtonElement;
        expect(btn).not.toBeNull();
        btn.click();
        expect(actions).toContainEqual({ action: "AddStamp", payload: s.name });
      }

      const stampAnnots = state.pages[0].annotations?.filter((a) => a.type === "stamp");
      expect(stampAnnots?.length).toBe(stamps.length);
    });

    it("4.3 inserts Drawing Shapes (Line, Arrow, Rectangle, Oval) and marks/applies Redactions", () => {
      const shapes = [
        { testId: "pdf-btn-shape-line", kind: "line" },
        { testId: "pdf-btn-shape-arrow", kind: "arrow" },
        { testId: "pdf-btn-shape-rect", kind: "rect" },
        { testId: "pdf-btn-shape-oval", kind: "ellipse" },
      ];

      for (const sh of shapes) {
        const btn = ribbonBody.querySelector(`[data-testid="${sh.testId}"]`) as HTMLButtonElement;
        expect(btn).not.toBeNull();
        btn.click();
        expect(actions).toContainEqual({ action: "InsertShape", payload: sh.kind });
      }

      const redactBtn = ribbonBody.querySelector("[data-testid='pdf-btn-redact']") as HTMLButtonElement;
      const applyRedactBtn = ribbonBody.querySelector("[data-testid='pdf-btn-apply-redact']") as HTMLButtonElement;

      redactBtn.click();
      expect(actions).toContainEqual({ action: "MarkRedaction", payload: undefined });
      expect(state.pages[0].annotations?.some((a) => a.type === "redact")).toBe(true);

      applyRedactBtn.click();
      expect(actions).toContainEqual({ action: "ApplyRedactions", payload: undefined });
    });
  });

  // =========================================================================
  // 5. SIGN & PROTECT TAB (SIGNING & EXPORT)
  // =========================================================================
  describe("5. Sign & Protect Tab", () => {
    beforeEach(() => {
      actions = [];
      ribbon.renderTab("Sign & Protect", ribbonBody, (action, payload) => {
        actions.push({ action, payload });
        app.handleRibbonAction(action, payload);
      });
      state.createBlankPdf(1);
    });

    it("5.1 triggers Add Signature, Add Initials, Sign Document, Export PDF, and Restrict Editing", () => {
      const addSigBtn = ribbonBody.querySelector("[data-testid='pdf-btn-add-signature']") as HTMLButtonElement;
      const addInitBtn = ribbonBody.querySelector("[data-testid='pdf-btn-add-initials']") as HTMLButtonElement;
      const signDocBtn = ribbonBody.querySelector("[data-testid='pdf-btn-sign']") as HTMLButtonElement;
      const exportPdfBtn = ribbonBody.querySelector("[data-testid='pdf-btn-export-pdf']") as HTMLButtonElement;
      const restrictBtn = ribbonBody.querySelector("[data-testid='pdf-btn-restrict-editing']") as HTMLButtonElement;

      addSigBtn.click();
      expect(actions).toContainEqual({ action: "AddSignature", payload: undefined });

      addInitBtn.click();
      expect(actions).toContainEqual({ action: "AddInitials", payload: undefined });
      expect(state.pages[0].annotations?.some((a) => a.signerName === "Initial")).toBe(true);

      signDocBtn.click();
      expect(actions).toContainEqual({ action: "SignPdf", payload: undefined });

      exportPdfBtn.click();
      expect(actions).toContainEqual({ action: "ExportPdf", payload: undefined });

      restrictBtn.click();
      expect(actions).toContainEqual({ action: "RestrictEditing", payload: undefined });
      expect(state.securityConfig.permissions?.canEdit).toBe(false);
    });
  });

  // =========================================================================
  // 6. VIEW TAB (LAYOUT, ROTATION & SIDEBAR PANES)
  // =========================================================================
  describe("6. View Tab", () => {
    beforeEach(() => {
      actions = [];
      ribbon.renderTab("View", ribbonBody, (action, payload) => {
        actions.push({ action, payload });
        app.handleRibbonAction(action, payload);
      });
      state.createBlankPdf(2);
    });

    it("6.1 toggles page display modes: Single, Continuous, Two Pages", () => {
      const singleBtn = ribbonBody.querySelector("[data-testid='pdf-btn-view-single']") as HTMLButtonElement;
      const contBtn = ribbonBody.querySelector("[data-testid='pdf-btn-view-continuous']") as HTMLButtonElement;
      const twoPageBtn = ribbonBody.querySelector("[data-testid='pdf-btn-view-twopage']") as HTMLButtonElement;

      singleBtn.click();
      expect(state.viewMode).toBe("single");

      twoPageBtn.click();
      expect(state.viewMode).toBe("twoPage");

      contBtn.click();
      expect(state.viewMode).toBe("continuous");
    });

    it("6.2 triggers Rotate View CW & CCW, Navigation Panes, and Full Screen", () => {
      const rotCW = ribbonBody.querySelector("[data-testid='pdf-btn-view-rot-cw']") as HTMLButtonElement;
      const rotCCW = ribbonBody.querySelector("[data-testid='pdf-btn-view-rot-ccw']") as HTMLButtonElement;
      const bmBtn = ribbonBody.querySelector("[data-testid='pdf-btn-view-bookmarks']") as HTMLButtonElement;
      const thumbBtn = ribbonBody.querySelector("[data-testid='pdf-btn-view-thumbnails']") as HTMLButtonElement;
      const attachBtn = ribbonBody.querySelector("[data-testid='pdf-btn-view-attachments']") as HTMLButtonElement;
      const fsBtn = ribbonBody.querySelector("[data-testid='pdf-btn-view-fullscreen']") as HTMLButtonElement;

      rotCW.click();
      expect(actions).toContainEqual({ action: "RotateView", payload: 90 });

      rotCCW.click();
      expect(actions).toContainEqual({ action: "RotateView", payload: -90 });

      bmBtn.click();
      expect(state.sidebarTab).toBe("bookmarks");

      thumbBtn.click();
      expect(state.sidebarTab).toBe("thumbnails");

      attachBtn.click();
      expect(state.sidebarTab).toBe("annotations");

      fsBtn.click();
      expect(actions).toContainEqual({ action: "ToggleFullScreen", payload: undefined });
    });
  });
});

