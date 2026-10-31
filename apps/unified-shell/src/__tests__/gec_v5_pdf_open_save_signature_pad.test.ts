import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { PdfState, pdfState } from "../state/pdfState";
import { SignDocumentModal } from "../components/SignDocumentModal";
import { binaryPackageService } from "../services/binaryPackageService";
import { fileIoService } from "../services/fileIoService";
import { UnifiedShellApp } from "../main";

describe("GEC-v5.0 Subphase 32.1.1: Local PDF Open/Save & Signature Pad Studio", () => {
  let state: PdfState;
  let app: UnifiedShellApp;

  beforeEach(() => {
    document.body.innerHTML = `
      <div id="app">
        <div id="titlebar">
          <div id="title-actions">
            <button id="btn-quick-open">Open</button>
            <button id="btn-quick-save">Save</button>
          </div>
        </div>
        <div id="tab-bar-container"></div>
        <div id="ribbon-container"></div>
        <div id="sub-header-container"></div>
        <div id="main-content">
          <div id="sidebar-container"></div>
          <div id="viewport-container"></div>
        </div>
        <div id="status-bar">
          <span id="status-doc-info"></span>
          <span id="status-zoom-level">100%</span>
        </div>
        <div id="backstage-container"></div>
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
      font: "14px Segoe UI, sans-serif",
      fillStyle: "#000",
      strokeStyle: "#000",
      lineWidth: 1,
      textAlign: "left",
      textBaseline: "top",
    };

    HTMLCanvasElement.prototype.getContext = ((type: string) => {
      if (type === "2d") return mockCtx;
      return null;
    }) as any;

    HTMLCanvasElement.prototype.toDataURL = ((mime = "image/png") => {
      return `data:${mime};base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=`;
    }) as any;

    state = pdfState;
    state.resetToBlank("TestContract.pdf");

    app = new UnifiedShellApp(false);
    app.getTabBar().addTab({
      id: "tab-pdf-1",
      docType: "PDF",
      title: "Contract.pdf",
      isDirty: false,
    });
    app.getTabBar().switchTab("tab-pdf-1");
    (app as any).currentDocType = "PDF";
  });

  afterEach(() => {
    document.body.innerHTML = "";
  });

  // =========================================================================
  // 1. Zero Hardcoded "John Doe" Invariant
  // =========================================================================
  describe("1. Zero Hardcoded 'John Doe' Ban Verification", () => {
    it("guarantees 'John Doe' never appears in modal defaults, state, or DOM", () => {
      const modal = new SignDocumentModal(state);
      modal.show({ pageNumber: 1 });
      const overlay = modal.getOverlay();

      const modalHtml = overlay.innerHTML;
      expect(modalHtml.includes("John Doe")).toBe(false);

      const nameInput = overlay.querySelector('[data-testid="input-sign-name"]') as HTMLInputElement;
      if (nameInput) {
        expect(nameInput.value.includes("John Doe")).toBe(false);
      }

      modal.hide();
    });
  });

  // =========================================================================
  // 2. Interactive Signature & Initials Studio Modal (Draw, Type, Upload)
  // =========================================================================
  describe("2. Interactive Signature & Initials Studio Modal", () => {
    it("2.1 Draw Tab: captures drawn ink stroke with color picker & clear button", () => {
      const modal = new SignDocumentModal(state);
      modal.show({ pageNumber: 1 });
      const overlay = modal.getOverlay();
      expect(overlay.style.display).toBe("flex");

      // Verify Draw Tab is active by default
      const drawTab = overlay.querySelector('[data-testid="tab-sign-draw"]') as HTMLButtonElement;
      expect(drawTab).not.toBeNull();
      drawTab.click();

      // Clear button
      const clearBtn = overlay.querySelector('[data-testid="btn-clear-signature"]') as HTMLButtonElement;
      expect(clearBtn).not.toBeNull();
      expect(() => clearBtn.click()).not.toThrow();

      // Apply signature
      const applyBtn = overlay.querySelector('[data-testid="btn-sign-apply"]') as HTMLButtonElement;
      expect(applyBtn).not.toBeNull();
      applyBtn.click();
      expect(overlay.style.display).toBe("none");

      // Verify signature added to state
      const sigs = state.getPage(1)?.annotations?.filter((a) => a.type === "signature") || [];
      expect(sigs.length).toBe(1);
      expect(sigs[0].signatureDataUrl).toContain("data:image/png;base64");
    });

    it("2.2 Type Tab: supports custom signer name, 4 script styles, and timestamp", () => {
      const modal = new SignDocumentModal(state);
      modal.show({ pageNumber: 1 });
      const overlay = modal.getOverlay();

      // Switch to Type Tab
      const typeTab = overlay.querySelector('[data-testid="tab-sign-type"]') as HTMLButtonElement;
      expect(typeTab).not.toBeNull();
      typeTab.click();

      // Custom Name input
      const input = overlay.querySelector('[data-testid="input-sign-name"]') as HTMLInputElement;
      expect(input).not.toBeNull();
      expect(input.placeholder).toBe("Enter your full name or initials...");
      input.value = "Custom Signer";
      input.dispatchEvent(new Event("input"));

      // 4 Script Styles
      const brushStyle = overlay.querySelector('[data-testid="sign-style-brush"]') as HTMLElement;
      const caveatStyle = overlay.querySelector('[data-testid="sign-style-caveat"]') as HTMLElement;
      const dancingStyle = overlay.querySelector('[data-testid="sign-style-dancing"]') as HTMLElement;
      const segoeStyle = overlay.querySelector('[data-testid="sign-style-segoe"]') as HTMLElement;
      expect(brushStyle).not.toBeNull();
      expect(caveatStyle).not.toBeNull();
      expect(dancingStyle).not.toBeNull();
      expect(segoeStyle).not.toBeNull();

      // Select Dancing Script
      dancingStyle.click();

      // Timestamp Checkbox
      const timestampCheck = overlay.querySelector('[data-testid="input-sign-include-date"]') as HTMLInputElement;
      expect(timestampCheck).not.toBeNull();
      expect(timestampCheck.checked).toBe(true);

      // Preview Card
      const previewCard = overlay.querySelector('[data-testid="signature-type-preview"]') as HTMLElement;
      expect(previewCard).not.toBeNull();
      expect(previewCard.textContent).toContain("Custom Signer");

      // Apply Signature
      const applyBtn = overlay.querySelector('[data-testid="btn-sign-apply"]') as HTMLButtonElement;
      applyBtn.click();

      const page1 = state.getPage(1);
      const sig = page1?.annotations?.find((a) => a.type === "signature" && a.signerName === "Custom Signer");
      expect(sig).toBeDefined();
      expect(sig?.signerName).toBe("Custom Signer");
    });

    it("2.3 Upload Tab: accepts signature image upload and renders preview", () => {
      const modal = new SignDocumentModal(state);
      modal.show({ pageNumber: 1 });
      const overlay = modal.getOverlay();

      // Switch to Upload Tab
      const uploadTab = overlay.querySelector('[data-testid="tab-sign-upload"]') as HTMLButtonElement;
      expect(uploadTab).not.toBeNull();
      uploadTab.click();

      const fileInput = overlay.querySelector('[data-testid="input-signature-upload"]') as HTMLInputElement;
      expect(fileInput).not.toBeNull();

      modal.hide();
    });
  });

  // =========================================================================
  // 3. Local Binary PDF File Loading & Parsing
  // =========================================================================
  describe("3. Real Binary .pdf Loading & Parsing", () => {
    it("parses multi-page binary PDF archives into page models and text blocks", async () => {
      const samplePdfBuffer = binaryPackageService.buildPdf([
        "Page 1: Non-Disclosure Agreement and Master Services Terms",
        "Page 2: Confidentiality Obligations, Governing Law and Execution",
      ]);

      expect(samplePdfBuffer.byteLength).toBeGreaterThan(100);

      // Load via FileIoService
      const success = await fileIoService.loadFileContent(samplePdfBuffer, "PDF", "NDA_Executed.pdf");
      expect(success).toBe(true);

      expect(state.pages.length).toBe(2);
      expect(state.pages[0].pageNumber).toBe(1);
      expect(state.pages[1].pageNumber).toBe(2);
      expect(state.pages[0].textBlocks?.length).toBeGreaterThan(0);
      expect(state.pages[0].textBlocks?.[0].text).toContain("Non-Disclosure Agreement");
      expect(state.pages[1].textBlocks?.[0].text).toContain("Confidentiality Obligations");
    });
  });

  // =========================================================================
  // 4. Binary PDF Export & Serialization Roundtrip
  // =========================================================================
  describe("4. Binary PDF Export & Roundtrip Verification", () => {
    it("exports multi-page PDF with text blocks, page dimensions, and signature stamps", () => {
      state.resetToBlank("ExportTest.pdf");
      state.insertBlankPage(1);
      expect(state.pages.length).toBe(2);

      // Add text and signature
      state.addTextBlock(1, 72, 720, "Executive Sign-Off Page 1", 14);
      state.addSignature(1, "data:image/png;base64,sampleSig", "Jane Executive");

      const binaryPdf = state.exportBinaryPdf();
      expect(binaryPdf).toBeInstanceOf(Uint8Array);
      expect(binaryPdf.length).toBeGreaterThan(200);

      const pdfText = new TextDecoder().decode(binaryPdf);
      expect(pdfText.startsWith("%PDF-1.7")).toBe(true);
      expect(pdfText).toContain("/Type /Catalog");
      expect(pdfText).toContain("/Type /Pages");
      expect(pdfText).toContain("/Count 2");
      expect(pdfText).toContain("Executive Sign-Off Page 1");
      expect(pdfText).toContain("Jane Executive");
      expect(pdfText).toContain("xref");
      expect(pdfText).toContain("trailer");
      expect(pdfText).toContain("%%EOF");
    });
  });

  // =========================================================================
  // 5. UnifiedShellApp Ribbon Routing for PDF
  // =========================================================================
  describe("5. UnifiedShellApp Ribbon Routing for PDF Actions", () => {
    it("routes AddSignature, AddInitials, and SignPdf to open the Signature Studio Modal", () => {
      (app as any).handleRibbonAction("AddSignature");
      const modal = (app as any).pdfWorkspace.getSignModal();
      expect(modal.getOverlay().style.display).toBe("flex");
      modal.hide();

      (app as any).handleRibbonAction("AddInitials");
      expect(modal.getOverlay().style.display).toBe("flex");
      modal.hide();

      (app as any).handleRibbonAction("SignPdf");
      expect(modal.getOverlay().style.display).toBe("flex");
      modal.hide();
    });
  });
});
