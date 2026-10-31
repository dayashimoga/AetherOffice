import { describe, it, expect, beforeEach } from "vitest";
import { PdfState } from "../state/pdfState";
import { PdfWorkspace } from "../apps/pdf/PdfWorkspace";
import { SignDocumentModal } from "../components/SignDocumentModal";
import { UnifiedShellApp } from "../main";

describe("PDF In-Place Text/Image Editor, Annotations, Redaction & AcroForms Filler (GEC-v2.0 Subphase 12.1.2)", () => {
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
      <div id="app">
        <header id="tab-bar-container"></header>
        <header id="ribbon-container"></header>
        <div id="sub-header-container"></div>
        <div class="main-content" style="display:flex; height:600px;">
          <aside id="sidebar-container" class="sidebar-pane"></aside>
          <main id="viewport-container" style="flex:1;"></main>
        </div>
        <footer id="status-bar-container">
          <span id="status-doc-info"></span>
        </footer>
      </div>
    `;

    state = new PdfState();
    container = document.getElementById("viewport-container")!;
    workspace = new PdfWorkspace(container, state);
  });

  // =========================================================================
  // 1. In-Place Text Editing, New Text Blocks, Images & Watermarks
  // =========================================================================
  describe("1. In-Place Text Editing, Images & Watermarks", () => {
    it("modifies existing text blocks and records changes in history snapshot", () => {
      const page1 = state.getPage(1)!;
      const initialText = page1.textBlocks![0].text;
      const tbId = page1.textBlocks![0].id;

      const updated = state.updateTextBlock(1, tbId, "Updated Standard Title 2026", { bold: false });
      expect(updated).toBe(true);
      expect(page1.textBlocks![0].text).toBe("Updated Standard Title 2026");
      expect(page1.textBlocks![0].bold).toBe(false);

      // Undo reverts text block modification
      expect(state.canUndo()).toBe(true);
      state.undo();
      const reverted = state.getPage(1)!;
      expect(reverted.textBlocks![0].text).toBe(initialText);
    });

    it("adds a new text block to a page and allows deletion", () => {
      const page2 = state.getPage(2)!;
      const initialCount = page2.textBlocks?.length || 0;

      const newBlock = state.addTextBlock(2, 120, 240, "Confidential Appendix", 16, "#dc2626");
      expect(newBlock.id).toBeDefined();
      expect(newBlock.text).toBe("Confidential Appendix");
      expect(page2.textBlocks?.length).toBe(initialCount + 1);

      // Delete the created text block
      const deleted = state.deleteTextBlock(2, newBlock.id);
      expect(deleted).toBe(true);
      expect(page2.textBlocks?.length).toBe(initialCount);
    });

    it("renders editable bounding box overlays in edit_text mode and handles inline editing", () => {
      state.setTool("edit_text");
      workspace.render();

      const overlay = workspace.getViewport().querySelector(".pdf-page-overlay");
      expect(overlay).not.toBeNull();

      const editableBlocks = overlay!.querySelectorAll(".pdf-text-block-editable");
      expect(editableBlocks.length).toBeGreaterThanOrEqual(3);

      // Click on first text block to launch inline editor
      const firstBlockEl = editableBlocks[0] as HTMLElement;
      firstBlockEl.click();

      const inlineEditor = overlay!.querySelector(".pdf-inline-text-editor") as HTMLInputElement;
      expect(inlineEditor).not.toBeNull();
      expect(inlineEditor.value).toContain("ISO 32000-2 Certified PDF Document");

      // Type new text and trigger blur
      inlineEditor.value = "AetherOffice Next-Gen PDF Editor";
      inlineEditor.dispatchEvent(new Event("blur"));

      const page1 = state.getPage(1)!;
      expect(page1.textBlocks![0].text).toBe("AetherOffice Next-Gen PDF Editor");
    });

    it("adds, renders, and deletes image graphic blocks", () => {
      const page1 = state.getPage(1)!;
      const img = state.addImageBlock(1, {
        x: 150,
        y: 400,
        width: 300,
        height: 200,
        title: "Architecture Diagram",
      });

      expect(img.id).toBeDefined();
      expect(page1.imageBlocks?.length).toBe(1);

      workspace.render();
      const canvas = workspace.getViewport().querySelector("canvas")!;
      expect(canvas).not.toBeNull();

      // Delete image block
      const del = state.deleteImageBlock(1, img.id);
      expect(del).toBe(true);
      expect(page1.imageBlocks?.length).toBe(0);
    });

    it("configures document-wide or page watermarks with custom opacity and rotation", () => {
      state.setWatermark({ text: "DRAFT SPECIFICATION", opacity: 0.15, rotation: -30 });

      state.pages.forEach((p) => {
        expect(p.watermark).toBeDefined();
        expect(p.watermark?.text).toBe("DRAFT SPECIFICATION");
        expect(p.watermark?.opacity).toBe(0.15);
      });

      // Remove watermark
      state.removeWatermark(true);
      state.pages.forEach((p) => {
        expect(p.watermark).toBeUndefined();
      });
    });
  });

  // =========================================================================
  // 2. Annotations: Highlight, Sticky Note Popover, Ink & Eraser
  // =========================================================================
  describe("2. Annotations: Highlight, Sticky Note Popover, Ink & Eraser", () => {
    it("adds highlight, underline, and strike annotations with correct geometry", () => {
      const hl = state.addAnnotation({
        type: "highlight",
        pageNumber: 2,
        rect: [80, 100, 240, 20],
        color: "rgba(255, 255, 0, 0.4)",
        contents: "Key security note",
      });
      expect(hl.id).toBeDefined();

      const ul = state.addAnnotation({
        type: "underline",
        pageNumber: 2,
        rect: [80, 140, 200, 2],
        color: "#0078d4",
      });
      expect(ul.id).toBeDefined();

      const strike = state.addAnnotation({
        type: "strike",
        pageNumber: 2,
        rect: [80, 180, 200, 2],
        color: "#dc2626",
      });
      expect(strike.id).toBeDefined();

      const page2Annots = state.getPage(2)!.annotations!;
      expect(page2Annots.length).toBe(3);
    });

    it("displays comment popover on sticky note click, updates contents, and saves", () => {
      workspace.render();
      const noteAnn = state.getPage(1)!.annotations!.find((a) => a.type === "note")!;

      // Find note hitbox on overlay and trigger click
      const overlay = workspace.getViewport().querySelector(".pdf-page-overlay")!;
      const noteEl = overlay.querySelector(`[data-annot-id="${noteAnn.id}"]`) as HTMLElement;
      expect(noteEl).not.toBeNull();
      noteEl.click();

      // Comment popover should be open in DOM
      const popover = document.querySelector(".pdf-comment-popover");
      expect(popover).not.toBeNull();
      expect(popover!.textContent).toContain(noteAnn.author);

      const textarea = popover!.querySelector(".pdf-comment-textarea") as HTMLTextAreaElement;
      expect(textarea.value).toBe(noteAnn.contents);

      // Edit and click save
      textarea.value = "Updated compliance review verified.";
      const saveBtn = popover!.querySelector(".btn-comment-save") as HTMLButtonElement;
      saveBtn.click();

      // Popover closes and annotation is updated
      expect(document.querySelector(".pdf-comment-popover")).toBeNull();
      const updatedNote = state.getAnnotation(noteAnn.id);
      expect(updatedNote?.contents).toBe("Updated compliance review verified.");
    });

    it("records freehand ink drawing paths with points and color", () => {
      state.setTool("pencil");
      state.inkColor = "#16a34a";
      state.inkThickness = 4;
      workspace.render();

      const overlay = workspace.getViewport().querySelector(".pdf-page-overlay") as HTMLElement;

      // Simulate mouse drag for ink stroke
      overlay.dispatchEvent(new MouseEvent("mousedown", { clientX: 100, clientY: 100 }));
      overlay.dispatchEvent(new MouseEvent("mousemove", { clientX: 110, clientY: 105 }));
      overlay.dispatchEvent(new MouseEvent("mousemove", { clientX: 120, clientY: 115 }));
      overlay.dispatchEvent(new MouseEvent("mouseup"));

      const p1Annots = state.getPage(1)!.annotations!;
      const inkAnn = p1Annots.find((a) => a.type === "ink");
      expect(inkAnn).toBeDefined();
      expect(inkAnn?.color).toBe("#16a34a");
      expect(inkAnn?.strokeWidth).toBe(4);
      expect(inkAnn?.points?.length).toBeGreaterThan(1);
    });

    it("erases annotations when clicked with eraser tool", () => {
      const page1 = state.getPage(1)!;
      const initialCount = page1.annotations!.length;
      const targetAnnot = page1.annotations![0];

      state.setTool("eraser");
      workspace.render();

      const overlay = workspace.getViewport().querySelector(".pdf-page-overlay")!;
      const annotHitbox = overlay.querySelector(`[data-annot-id="${targetAnnot.id}"]`) as HTMLElement;
      expect(annotHitbox).not.toBeNull();

      annotHitbox.click();
      expect(page1.annotations!.length).toBe(initialCount - 1);
      expect(state.getAnnotation(targetAnnot.id)).toBeUndefined();
    });
  });

  // =========================================================================
  // 3. Permanent Redaction Tool
  // =========================================================================
  describe("3. Permanent Redaction Tool", () => {
    it("marks redaction and expunges underlying text blocks upon applying redactions", () => {
      const page1 = state.getPage(1)!;
      expect(page1.textBlocks!.length).toBe(3);

      // Mark a redaction rectangle over the first text block (y: 80)
      const redactAnn = state.markRedaction(1, [70, 60, 400, 35]);
      expect(redactAnn.type).toBe("redact");
      expect(redactAnn.isAppliedRedaction).toBeFalsy();

      // Apply redactions
      const removedCount = state.applyRedactions(1);
      expect(removedCount).toBeGreaterThanOrEqual(1);

      // The intersecting text block has been permanently expunged
      const remainingTexts = page1.textBlocks!.map((tb) => tb.text);
      expect(remainingTexts).not.toContain("ISO 32000-2 Certified PDF Document");

      // The redaction annotation is permanently marked applied
      expect(redactAnn.isAppliedRedaction).toBe(true);
    });
  });

  // =========================================================================
  // 4. Interactive AcroForms Builder & Form Filler
  // =========================================================================
  describe("4. Interactive AcroForms Builder & Form Filler", () => {
    it("adds interactive form fields: text, checkbox, radio, dropdown, and signature", () => {
      const p3 = state.getPage(3)!;
      const textField = state.addFormField(3, { type: "text", name: "Company" });
      const chkField = state.addFormField(3, { type: "checkbox", name: "IsVendor" });
      const rad1 = state.addFormField(3, { type: "radio", name: "PlanA", group: "plans" });
      const rad2 = state.addFormField(3, { type: "radio", name: "PlanB", group: "plans" });
      const dropField = state.addFormField(3, {
        type: "dropdown",
        name: "Tier",
        options: ["Gold", "Silver", "Bronze"],
      });
      const sigField = state.addFormField(3, { type: "signature", name: "SignBox" });

      expect(p3.formFields?.length).toBe(6);
      expect(textField.type).toBe("text");
      expect(chkField.type).toBe("checkbox");
      expect(rad1.group).toBe("plans");
      expect(rad2.group).toBe("plans");
      expect(dropField.options).toContain("Silver");
      expect(sigField.type).toBe("signature");
    });

    it("interactively toggles checkboxes and radio buttons in the workspace overlay", () => {
      workspace.render();
      const p1 = state.getPage(1)!;
      const chk = p1.formFields!.find((f) => f.type === "checkbox")!;
      expect(chk.value).toBe(true);

      const overlay = workspace.getViewport().querySelector(".pdf-page-overlay")!;
      const chkEl = overlay.querySelector(`[data-field-id="${chk.id}"]`) as HTMLElement;
      expect(chkEl).not.toBeNull();

      // Click to toggle
      chkEl.click();
      expect(chk.value).toBe(false);

      chkEl.click();
      expect(chk.value).toBe(true);
    });

    it("clears all form fields, exports data dictionary, and imports JSON data", () => {
      // Export current form data
      const exported = state.exportFormData();
      expect(exported["Full_Name"]).toBe("Jane Citizen");
      expect(exported["Agreed"]).toBe(true);

      // Clear form data
      state.clearFormData();
      const cleared = state.exportFormData();
      expect(cleared["Full_Name"]).toBe("");
      expect(cleared["Agreed"]).toBe(false);

      // Import new form data
      const importedCount = state.importFormData({
        Full_Name: "Alice Wonderland",
        Agreed: true,
      });
      expect(importedCount).toBe(2);

      const restored = state.exportFormData();
      expect(restored["Full_Name"]).toBe("Alice Wonderland");
      expect(restored["Agreed"]).toBe(true);
    });
  });

  // =========================================================================
  // 5. Digital Signature Modal & PAdES Cryptographic Signing
  // =========================================================================
  describe("5. Digital Signature Modal & Signing", () => {
    it("launches SignDocumentModal, switches tabs, and previews typed cursive signature", () => {
      const modal = new SignDocumentModal(state);
      modal.show({ pageNumber: 1 });

      const overlay = modal.getOverlay();
      expect(overlay.style.display).toBe("flex");

      // Switch to Type tab
      const typeTabBtn = overlay.querySelector('[data-tab="type"]') as HTMLButtonElement;
      expect(typeTabBtn).not.toBeNull();
      typeTabBtn.click();

      const nameInput = overlay.querySelector(".signature-name-input") as HTMLInputElement;
      expect(nameInput).not.toBeNull();
      nameInput.value = "Dr. Robert Smith";
      nameInput.dispatchEvent(new Event("input"));

      const preview = overlay.querySelector(".signature-type-preview")!;
      expect(preview.textContent).toBe("Dr. Robert Smith");

      modal.hide();
      expect(overlay.style.display).toBe("none");
    });

    it("applies drawn signature stamp onto the document page and updates signature field", () => {
      const page1 = state.getPage(1)!;
      state.addFormField(1, { type: "signature", name: "ExecutiveSign" });

      const modal = new SignDocumentModal(state);
      let signedCallbackFired = false;
      modal.show({
        pageNumber: 1,
        targetFieldId: "ExecutiveSign",
        onSigned: () => {
          signedCallbackFired = true;
        },
      });

      const applyBtn = modal.getOverlay().querySelector(".btn-sign-apply") as HTMLButtonElement;
      applyBtn.click();

      expect(signedCallbackFired).toBe(true);
      const sigAnnot = page1.annotations!.find((a) => a.type === "signature");
      expect(sigAnnot).toBeDefined();
      expect(sigAnnot?.signerName).toBe("Authorized Signer");

      // Signature form field is also filled
      const sigField = page1.formFields!.find((f) => f.name === "ExecutiveSign");
      expect(sigField?.value).toBe("Authorized Signer");
    });
  });

  // =========================================================================
  // 6. Binary ISO 32000-2 PDF Export
  // =========================================================================
  describe("6. Binary ISO 32000-2 PDF Serialization", () => {
    it("serializes document state into standard ISO 32000 compliant binary PDF bytes", () => {
      const bytes = state.exportBinaryPdf();
      expect(bytes).toBeInstanceOf(Uint8Array);
      expect(bytes.length).toBeGreaterThan(100);

      const decoder = new TextDecoder("latin1");
      const pdfText = decoder.decode(bytes);

      // ISO 32000 Header verification
      expect(pdfText.startsWith("%PDF-1.7")).toBe(true);

      // Core Object Dictionaries
      expect(pdfText).toContain("/Type /Catalog");
      expect(pdfText).toContain("/Type /Pages");
      expect(pdfText).toContain("/Type /Page");
      expect(pdfText).toContain("/Contents");

      // Trailer and Cross-Reference Table
      expect(pdfText).toContain("xref");
      expect(pdfText).toContain("trailer");
      expect(pdfText).toContain("startxref");
      expect(pdfText).toContain("%%EOF");
    });
  });

  // =========================================================================
  // 7. UnifiedShellApp Ribbon & Keyboard Integration
  // =========================================================================
  describe("7. UnifiedShellApp Ribbon Wiring for PDF Editing & Signing", () => {
    let app: UnifiedShellApp;

    beforeEach(() => {
      app = new UnifiedShellApp(false);

      // Open a PDF tab
      const tabBar = app.getTabBar();
      tabBar.addTab({
        id: "tab-pdf-edit",
        docType: "PDF",
        title: "StandardContract.pdf",
        isDirty: false,
      });
      tabBar.switchTab("tab-pdf-edit");
    });

    it("dispatches Edit PDF ribbon actions: AddPdfText, EditPdfText, AddPdfImage, AddWatermark", () => {
      app.handleRibbonAction("AddWatermark", "RESTRICTED");
      const appState = app.getPdfWorkspace()?.getState() || state;
      expect(appState.pages[0].watermark?.text).toBe("RESTRICTED");

      app.handleRibbonAction("AddPdfText");
      expect(appState.activeTool).toBe("edit_text");

      app.handleRibbonAction("AddPdfImage");
      expect(appState.getPage(appState.currentPage)!.imageBlocks?.length).toBeGreaterThanOrEqual(1);

      expect(app.getTabBar().getTabs().find((t) => t.id === "tab-pdf-edit")?.isDirty).toBe(true);
    });

    it("dispatches Comment & Annotate ribbon actions: SetAnnotTool and AddStamp", () => {
      const appState = app.getPdfWorkspace()?.getState() || state;

      app.handleRibbonAction("SetAnnotTool", "highlight");
      expect(appState.activeTool).toBe("highlight");

      app.handleRibbonAction("SetAnnotTool", "pencil");
      expect(appState.activeTool).toBe("pencil");

      app.handleRibbonAction("AddStamp", "confidential");
      const stamp = appState.getPage(appState.currentPage)!.annotations!.find((a) => a.type === "stamp");
      expect(stamp).toBeDefined();
      expect(stamp?.contents).toBe("CONFIDENTIAL");
    });

    it("dispatches Forms ribbon actions: AddFormField, ClearFormData, ExportFormData", () => {
      const appState = app.getPdfWorkspace()?.getState() || state;

      app.handleRibbonAction("AddFormField", "checkbox");
      const chk = appState.getPage(appState.currentPage)!.formFields!.find((f) => f.type === "checkbox");
      expect(chk).toBeDefined();

      app.handleRibbonAction("ClearFormData");
      expect(chk?.value).toBe(false);
    });

    it("dispatches Sign and Redaction ribbon actions", () => {
      const appState = app.getPdfWorkspace()?.getState() || state;

      app.handleRibbonAction("MarkRedaction");
      expect(appState.activeTool).toBe("redact");

      app.handleRibbonAction("ApplyRedactions");
      // Applied without errors
      expect(appState.pages.length).toBeGreaterThan(0);

      // Sign Document triggers sign modal
      app.handleRibbonAction("SignPdf");
      const signOverlay = app.getPdfWorkspace()?.getSignModal().getOverlay();
      expect(signOverlay).toBeDefined();
      expect(signOverlay?.style.display).toBe("flex");
    });
  });
});
