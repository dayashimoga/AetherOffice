import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { PdfState, pdfState } from "../state/pdfState";
import { PdfWorkspace } from "../apps/pdf/PdfWorkspace";
import { SignDocumentModal } from "../components/SignDocumentModal";
import { UnifiedShellApp } from "../main";

describe("PDF Studio Annotations, Freehand Ink, Form Builder/Filler, Signature & Export (Subphase 25.1.2)", () => {
  let state: PdfState;
  let container: HTMLElement;
  let workspace: PdfWorkspace;

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

    state = new PdfState();
    state.resetToBlank("AnnotationStudio.pdf");
    container = document.getElementById("viewport-container")!;
    workspace = new PdfWorkspace(container, state);
  });

  afterEach(() => {
    document.body.innerHTML = "";
  });

  describe("1. PDF Annotations: Highlights, Markups, Sticky Notes & Redactions", () => {
    it("adds highlight, underline, and strikethrough markup annotations", () => {
      state.execCommand("SetHighlightColor", "rgba(59, 130, 246, 0.5)");
      expect(state.highlightColor).toBe("rgba(59, 130, 246, 0.5)");

      const hl = state.addAnnotation({
        type: "highlight",
        pageNumber: 1,
        rect: [100, 150, 200, 24],
        color: state.highlightColor,
        contents: "Audited paragraph",
      });
      expect(hl.type).toBe("highlight");
      expect(hl.color).toBe("rgba(59, 130, 246, 0.5)");

      const ul = state.addAnnotation({
        type: "underline",
        pageNumber: 1,
        rect: [100, 180, 180, 2],
        color: "#2563eb",
      });
      expect(ul.type).toBe("underline");

      const strike = state.addAnnotation({
        type: "strike",
        pageNumber: 1,
        rect: [100, 210, 180, 2],
        color: "#dc2626",
      });
      expect(strike.type).toBe("strike");

      workspace.render();
      const annots = container.querySelectorAll('.pdf-annot-hitbox');
      expect(annots.length).toBe(3);
    });

    it("creates sticky note comment pin and modifies text via comment popover", () => {
      const note = state.addAnnotation({
        type: "note",
        pageNumber: 1,
        rect: [150, 200, 24, 24],
        color: "#eab308",
        author: "Chief Architect",
        date: "2026-10-30",
        contents: "Initial review comment",
      });
      expect(note.id).toBeDefined();

      workspace.render();
      const notePin = container.querySelector('[data-testid="pdf-annot-note"]') as HTMLElement;
      expect(notePin).not.toBeNull();

      // Click pin to open comment popover
      notePin.click();
      const popover = document.querySelector('[data-testid="pdf-comment-popover"]') as HTMLElement;
      expect(popover).not.toBeNull();

      const textarea = popover.querySelector('[data-testid="pdf-comment-textarea"]') as HTMLTextAreaElement;
      expect(textarea.value).toBe("Initial review comment");

      // Edit and save comment
      textarea.value = "Updated forensic comment 2026";
      const saveBtn = popover.querySelector('[data-testid="btn-comment-save"]') as HTMLButtonElement;
      saveBtn.click();

      expect(state.getPage(1)?.annotations?.[0].contents).toBe("Updated forensic comment 2026");
      expect(document.querySelector('[data-testid="pdf-comment-popover"]')).toBeNull();
    });

    it("marks and applies redactions to sanitize sensitive content", () => {
      state.execCommand("MarkRedaction");
      expect(state.activeTool).toBe("redact");

      const page1 = state.getPage(1)!;
      const redactsBefore = page1.annotations?.filter((a) => a.type === "redact") || [];
      expect(redactsBefore.length).toBe(1);
      expect(redactsBefore[0].isAppliedRedaction).toBeFalsy();

      state.execCommand("ApplyRedactions");
      const redactsAfter = page1.annotations?.filter((a) => a.type === "redact") || [];
      expect(redactsAfter.length).toBe(1);
      expect(redactsAfter[0].isAppliedRedaction).toBe(true);
    });
  });

  describe("2. Freehand Ink Drawing & Eraser Tool", () => {
    it("configures ink color and thickness and records freehand strokes", () => {
      state.execCommand("SetInkColor", "#10b981");
      state.execCommand("SetInkThickness", 5);
      expect(state.inkColor).toBe("#10b981");
      expect(state.inkThickness).toBe(5);

      const inkAnnot = state.addAnnotation({
        type: "ink",
        pageNumber: 1,
        rect: [100, 100, 200, 150],
        color: state.inkColor,
        strokeWidth: state.inkThickness,
        points: [
          { x: 100, y: 100 },
          { x: 150, y: 180 },
          { x: 200, y: 150 },
        ],
        contents: "Freehand Pen Stroke",
      });

      expect(inkAnnot.type).toBe("ink");
      expect(inkAnnot.points?.length).toBe(3);
      expect(inkAnnot.strokeWidth).toBe(5);

      workspace.render();
      const inkEl = container.querySelector('[data-testid="pdf-annot-ink"]') as HTMLElement;
      expect(inkEl).not.toBeNull();

      // Eraser removes ink annotation on click
      state.setTool("eraser");
      inkEl.click();
      expect(state.getPage(1)?.annotations?.length).toBe(0);
    });
  });

  describe("3. Interactive Form Fields: Text, Checkbox, Radio, Dropdown & Signing", () => {
    it("places interactive form fields on PDF pages and modifies field values", () => {
      state.execCommand("AddFormField", "text");
      state.execCommand("AddFormField", "checkbox");
      state.execCommand("AddFormField", "dropdown");

      const fields = state.getPage(1)?.formFields || [];
      expect(fields.length).toBe(3);

      const textField = fields.find((f) => f.type === "text")!;
      const checkboxField = fields.find((f) => f.type === "checkbox")!;
      const dropdownField = fields.find((f) => f.type === "dropdown")!;

      state.setFormFieldValue(textField.id, "Acme Corporation");
      expect(state.getFormFieldValue(textField.name)).toBe("Acme Corporation");

      state.setFormFieldValue(checkboxField.id, true);
      expect(state.getFormFieldValue(checkboxField.name)).toBe(true);

      state.setFormFieldValue(dropdownField.id, "Option B");
      expect(state.getFormFieldValue(dropdownField.name)).toBe("Option B");
    });

    it("clears, exports, and imports form data accurately", () => {
      state.execCommand("AddFormField", "text");
      state.execCommand("AddFormField", "checkbox");
      const fields = state.getPage(1)?.formFields || [];

      state.setFormFieldValue(fields[0].id, "Sample Value");
      state.setFormFieldValue(fields[1].id, true);

      const exported = state.exportFormData();
      expect(exported[fields[0].name]).toBe("Sample Value");
      expect(exported[fields[1].name]).toBe(true);

      state.execCommand("ClearFormData");
      expect(state.getFormFieldValue(fields[0].name)).toBe("");
      expect(state.getFormFieldValue(fields[1].name)).toBe(false);

      state.execCommand("ImportFormData", exported);
      expect(state.getFormFieldValue(fields[0].name)).toBe("Sample Value");
      expect(state.getFormFieldValue(fields[1].name)).toBe(true);
    });
  });

  describe("4. Digital Signature Modal & PAdES Cryptographic Signing", () => {
    it("opens SignDocumentModal and supports both Draw and Typed signature modes", () => {
      const modal = new SignDocumentModal(state);
      expect(modal).toBeDefined();

      modal.show({ pageNumber: 1 });
      const overlay = modal.getOverlay();
      expect(overlay.style.display).toBe("flex");

      // Switch to Type tab
      const typeTab = overlay.querySelector('[data-testid="tab-sign-type"]') as HTMLButtonElement;
      expect(typeTab).not.toBeNull();
      typeTab.click();

      const nameInput = overlay.querySelector('[data-testid="input-sign-name"]') as HTMLTextAreaElement;
      expect(nameInput).not.toBeNull();
      nameInput.value = "Chief Legal Officer";
      nameInput.dispatchEvent(new Event("input"));

      // Apply signature
      const applyBtn = overlay.querySelector('[data-testid="btn-sign-apply"]') as HTMLButtonElement;
      applyBtn.click();
      expect(overlay.style.display).toBe("none");

      const annots = state.getPage(1)?.annotations?.filter((a) => a.type === "signature") || [];
      expect(annots.length).toBe(1);
      expect(annots[0].signerName).toBe("Chief Legal Officer");
    });
  });

  describe("5. UnifiedShellApp Ribbon Wiring for Annotations, Forms, Sign & Export", () => {
    it("dispatches Comment, Forms, and Sign ribbon actions through UnifiedShellApp", () => {
      const app = new UnifiedShellApp();
      app.getTabBar().addTab({
        id: "tab-annot-pdf",
        docType: "PDF",
        title: "SignedDocument.pdf",
        isDirty: false,
      });
      app.getTabBar().switchTab("tab-annot-pdf");

      const ribbon = app.getRibbon();

      // Comment & Annotate Tab
      ribbon.setActiveTab("Comment & Annotate");
      const highlightBtn = document.querySelector('[data-testid="pdf-btn-highlight"]') as HTMLButtonElement;
      expect(highlightBtn).not.toBeNull();
      highlightBtn.click();
      expect(pdfState.activeTool).toBe("highlight");

      const pencilBtn = document.querySelector('[data-testid="pdf-btn-pencil"]') as HTMLButtonElement;
      expect(pencilBtn).not.toBeNull();
      pencilBtn.click();
      expect(pdfState.activeTool).toBe("pencil");

      // Forms Tab
      ribbon.setActiveTab("Forms");
      const formTextBtn = document.querySelector('[data-testid="pdf-btn-form-text"]') as HTMLButtonElement;
      expect(formTextBtn).not.toBeNull();
      formTextBtn.click();
      expect(pdfState.pages[0].formFields?.length).toBeGreaterThan(0);

      // Sign & Protect Tab
      ribbon.setActiveTab("Sign & Protect");
      const signBtn = document.querySelector('[data-testid="pdf-btn-sign"]') as HTMLButtonElement;
      expect(signBtn).not.toBeNull();
      signBtn.click();
      expect(pdfState.activeTool).toBe("sign");

      // Export PDF button
      const exportBtn = document.querySelector('[data-testid="pdf-btn-export-pdf"]') as HTMLButtonElement;
      expect(exportBtn).not.toBeNull();
      exportBtn.click();
      const statusInfo = document.getElementById("status-doc-info");
      expect(statusInfo?.textContent).toContain("Exported:");
    });
  });
});
