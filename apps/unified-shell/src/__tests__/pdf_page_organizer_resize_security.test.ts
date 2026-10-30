import { describe, it, expect, beforeEach, afterEach } from "vitest";
import { PdfState, pdfState, PdfPageModel } from "../state/pdfState";
import { PdfRibbon } from "../components/ribbon/PdfRibbon";
import { PdfWorkspace } from "../apps/pdf/PdfWorkspace";
import { UnifiedShellApp } from "../main";

describe("PDF Open/Save, Multi-Page Organizer, Page Resize & Password Encrypt/Decrypt (GEC-v4.0 Subphase 32.1.1)", () => {
  let state: PdfState;
  let ribbon: PdfRibbon;
  let ribbonBody: HTMLElement;
  let workspace: PdfWorkspace;
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
    state.resetToBlank("TestDocument.pdf");

    app = new UnifiedShellApp(false);
    app.getTabBar().addTab({
      id: "tab-pdf-1",
      docType: "PDF",
      title: "Contract.pdf",
      isDirty: false,
    });
    app.getTabBar().switchTab("tab-pdf-1");
    (app as any).currentDocType = "PDF";

    ribbon = new PdfRibbon();
    ribbonBody = document.createElement("div");
    ribbonBody.className = "ribbon-body";
    document.body.appendChild(ribbonBody);

    const vpContainer = document.getElementById("viewport-container")!;
    workspace = new PdfWorkspace(vpContainer, state);

    actions = [];
  });

  afterEach(() => {
    document.body.innerHTML = "";
  });

  // =========================================================================
  // 1. OPEN & CREATE BLANK PDF
  // =========================================================================
  describe("1. Open & Create Blank PDF", () => {
    it("1.1 resets and creates a completely blank PDF document", () => {
      state.createBlankPdf(2);
      expect(state.pageCount).toBe(2);
      expect(state.currentPage).toBe(1);
      expect(state.pages[0].textBlocks?.length).toBe(0);
      expect(state.pages[1].textBlocks?.length).toBe(0);
    });

    it("1.2 loads existing PDF document pages into state and renders", () => {
      const incomingPages: PdfPageModel[] = [
        {
          id: "p-src-1",
          pageNumber: 1,
          width: 816,
          height: 1056,
          rotation: 0,
          title: "Page 1 - Executive Report",
          textBlocks: [{ id: "t1", x: 100, y: 100, text: "Quarterly Review" }],
        },
        {
          id: "p-src-2",
          pageNumber: 2,
          width: 816,
          height: 1056,
          rotation: 0,
          title: "Page 2 - Financial Summary",
          textBlocks: [{ id: "t2", x: 100, y: 100, text: "Balance Sheet" }],
        },
      ];

      state.loadPdfDocument(incomingPages);
      expect(state.pageCount).toBe(2);
      expect(state.pages[0].title).toBe("Page 1 - Executive Report");
      expect(state.pages[1].title).toBe("Page 2 - Financial Summary");
      expect(() => workspace.render()).not.toThrow();
    });
  });

  // =========================================================================
  // 2. PAGE TOOLS TAB (PAGE MANAGEMENT & REORGANIZATION)
  // =========================================================================
  describe("2. Page Tools Tab (Insert, Delete, Duplicate, Rotate, Move, Split, Merge)", () => {
    beforeEach(() => {
      actions = [];
      ribbon.renderTab("Page Tools", ribbonBody, (action, payload) => {
        actions.push({ action, payload });
        app.handleRibbonAction(action, payload);
      });
      // Start with 2 pages
      state.createBlankPdf(2);
      expect(state.pageCount).toBe(2);
    });

    it("2.1 inserts a new blank page via ribbon button", () => {
      const addPageBtn = ribbonBody.querySelector("[data-testid='pdf-btn-add-page']") as HTMLButtonElement;
      expect(addPageBtn).not.toBeNull();
      addPageBtn.click();

      expect(actions).toContainEqual({ action: "InsertBlankPage", payload: undefined });
      expect(state.pageCount).toBe(3);
      expect(state.currentPage).toBe(2); // Inserted after page 1
    });

    it("2.2 deletes the selected page and prevents deleting the last page", () => {
      const deletePageBtn = ribbonBody.querySelector("[data-testid='pdf-btn-delete-page']") as HTMLButtonElement;
      expect(deletePageBtn).not.toBeNull();

      deletePageBtn.click();
      expect(actions).toContainEqual({ action: "DeletePage", payload: undefined });
      expect(state.pageCount).toBe(1);

      // Attempting to delete when only 1 page remains must fail gracefully
      const deleteResult = state.deletePage(1);
      expect(deleteResult).toBe(false);
      expect(state.pageCount).toBe(1);
    });

    it("2.3 duplicates selected page and copies its text and form fields", () => {
      state.pages[0].textBlocks = [{ id: "tb-copy", x: 50, y: 50, text: "Confidential Stamp" }];
      state.pages[0].formFields = [{ id: "ff-copy", name: "Sig", type: "signature", value: "", rect: [100, 100, 150, 40] }];

      const dupBtn = ribbonBody.querySelector("[data-testid='pdf-btn-duplicate-page']") as HTMLButtonElement;
      expect(dupBtn).not.toBeNull();
      dupBtn.click();

      expect(actions).toContainEqual({ action: "DuplicatePage", payload: undefined });
      expect(state.pageCount).toBe(3);
      expect(state.pages[1].textBlocks?.length).toBe(1);
      expect(state.pages[1].formFields?.length).toBe(1);
      expect(state.pages[1].title).toContain("(Copy)");
    });

    it("2.4 rotates page clockwise and counter-clockwise", () => {
      const rotCW = ribbonBody.querySelector("[data-testid='pdf-btn-rotate-cw']") as HTMLButtonElement;
      const rotCCW = ribbonBody.querySelector("[data-testid='pdf-btn-rotate-ccw']") as HTMLButtonElement;
      expect(rotCW).not.toBeNull();
      expect(rotCCW).not.toBeNull();

      rotCW.click();
      expect(actions).toContainEqual({ action: "RotatePageCW", payload: undefined });
      expect(state.pages[0].rotation).toBe(90);

      rotCW.click();
      expect(state.pages[0].rotation).toBe(180);

      rotCCW.click();
      expect(actions).toContainEqual({ action: "RotatePageCCW", payload: undefined });
      expect(state.pages[0].rotation).toBe(90);
    });

    it("2.5 moves pages up and down", () => {
      state.pages[0].title = "First";
      state.pages[1].title = "Second";

      const moveDownBtn = ribbonBody.querySelector("[data-testid='pdf-btn-move-page-down']") as HTMLButtonElement;
      expect(moveDownBtn).not.toBeNull();
      moveDownBtn.click();

      expect(actions).toContainEqual({ action: "MovePageDown", payload: undefined });
      expect(state.pages[0].title).toBe("Second");
      expect(state.pages[1].title).toBe("First");

      const moveUpBtn = ribbonBody.querySelector("[data-testid='pdf-btn-move-page-up']") as HTMLButtonElement;
      expect(moveUpBtn).not.toBeNull();
      moveUpBtn.click();

      expect(actions).toContainEqual({ action: "MovePageUp", payload: undefined });
      expect(state.pages[0].title).toBe("First");
      expect(state.pages[1].title).toBe("Second");
    });

    it("2.6 splits PDF page range and merges incoming pages", () => {
      const splitBtn = ribbonBody.querySelector("[data-testid='pdf-btn-split-pdf']") as HTMLButtonElement;
      expect(splitBtn).not.toBeNull();
      splitBtn.click();
      expect(actions).toContainEqual({ action: "SplitPdf", payload: undefined });

      const extracted = state.splitPdf(1, 2);
      expect(extracted.length).toBe(2);

      const mergeBtn = ribbonBody.querySelector("[data-testid='pdf-btn-merge-pdf']") as HTMLButtonElement;
      expect(mergeBtn).not.toBeNull();
      mergeBtn.click();
      expect(actions).toContainEqual({ action: "MergePdf", payload: undefined });

      state.mergePdf(extracted);
      expect(state.pageCount).toBe(4);
    });
  });

  // =========================================================================
  // 3. PAGE RESIZE & STANDARD SIZES
  // =========================================================================
  describe("3. Page Resize (A4, Letter, Legal, A3, A5 & Custom Dimensions)", () => {
    beforeEach(() => {
      actions = [];
      ribbon.renderTab("Page Tools", ribbonBody, (action, payload) => {
        actions.push({ action, payload });
        app.handleRibbonAction(action, payload);
      });
      state.createBlankPdf(2);
      // Place text block and form field at known coords to test content scaling
      state.pages[0].width = 816;
      state.pages[0].height = 1056;
      state.pages[0].textBlocks = [{ id: "t1", x: 100, y: 200, text: "Sample", fontSize: 16 }];
      state.pages[0].formFields = [{ id: "f1", name: "User", type: "text", value: "", rect: [100, 200, 200, 40] }];
    });

    it("3.1 resizes current page to A4 (794 × 1123 px)", () => {
      const a4Btn = ribbonBody.querySelector("[data-testid='pdf-btn-page-size-a4']") as HTMLButtonElement;
      expect(a4Btn).not.toBeNull();
      a4Btn.click();

      expect(actions).toContainEqual({ action: "SetPageSize", payload: "A4" });
      expect(state.pages[0].width).toBe(794);
      expect(state.pages[0].height).toBe(1123);
      // Content coordinates are scaled proportionally
      expect(state.pages[0].textBlocks![0].x).toBe(Math.round(100 * (794 / 816)));
      expect(state.pages[0].textBlocks![0].y).toBe(Math.round(200 * (1123 / 1056)));
    });

    it("3.2 resizes current page to Letter (816 × 1056 px)", () => {
      const letterBtn = ribbonBody.querySelector("[data-testid='pdf-btn-page-size-letter']") as HTMLButtonElement;
      expect(letterBtn).not.toBeNull();
      letterBtn.click();

      expect(actions).toContainEqual({ action: "SetPageSize", payload: "Letter" });
      expect(state.pages[0].width).toBe(816);
      expect(state.pages[0].height).toBe(1056);
    });

    it("3.3 resizes current page to Legal (816 × 1344 px)", () => {
      const legalBtn = ribbonBody.querySelector("[data-testid='pdf-btn-page-size-legal']") as HTMLButtonElement;
      expect(legalBtn).not.toBeNull();
      legalBtn.click();

      expect(actions).toContainEqual({ action: "SetPageSize", payload: "Legal" });
      expect(state.pages[0].width).toBe(816);
      expect(state.pages[0].height).toBe(1344);
    });

    it("3.4 resizes current page to A3 and A5", () => {
      const a3Btn = ribbonBody.querySelector("[data-testid='pdf-btn-page-size-a3']") as HTMLButtonElement;
      expect(a3Btn).not.toBeNull();
      a3Btn.click();
      expect(state.pages[0].width).toBe(1123);
      expect(state.pages[0].height).toBe(1587);

      const a5Btn = ribbonBody.querySelector("[data-testid='pdf-btn-page-size-a5']") as HTMLButtonElement;
      expect(a5Btn).not.toBeNull();
      a5Btn.click();
      expect(state.pages[0].width).toBe(559);
      expect(state.pages[0].height).toBe(794);
    });

    it("3.5 applies custom resize dimensions across all pages", () => {
      const resizeBtn = ribbonBody.querySelector("[data-testid='pdf-btn-resize-page']") as HTMLButtonElement;
      expect(resizeBtn).not.toBeNull();
      resizeBtn.click();
      expect(actions).toContainEqual({ action: "ResizePageModal", payload: undefined });

      // Apply custom dimensions to all pages
      const success = state.resizePage(1, 1200, 1600, true, true);
      expect(success).toBe(true);
      expect(state.pages[0].width).toBe(1200);
      expect(state.pages[0].height).toBe(1600);
      expect(state.pages[1].width).toBe(1200);
      expect(state.pages[1].height).toBe(1600);
    });
  });

  // =========================================================================
  // 4. SIGN & PROTECT TAB (PASSWORD ENCRYPTION, DECRYPTION & COMPRESSION)
  // =========================================================================
  describe("4. Sign & Protect Tab (Password Encrypt, Decrypt & Compress)", () => {
    beforeEach(() => {
      actions = [];
      ribbon.renderTab("Sign & Protect", ribbonBody, (action, payload) => {
        actions.push({ action, payload });
        app.handleRibbonAction(action, payload);
      });
      state.createBlankPdf(1);
    });

    it("4.1 encrypts document with password and updates security status", () => {
      const encryptBtn = ribbonBody.querySelector("[data-testid='pdf-btn-encrypt']") as HTMLButtonElement;
      expect(encryptBtn).not.toBeNull();
      encryptBtn.click();

      expect(actions).toContainEqual({ action: "EncryptPassword", payload: undefined });
      expect(state.isEncrypted).toBe(true);
      expect(state.getSecurityStatus()).toBe("🔒 Password Protected");
      expect(state.securityConfig.isEncrypted).toBe(true);
      expect(state.securityConfig.permissions?.canPrint).toBe(true);
    });

    it("4.2 decrypts password-protected PDF and removes encryption restrictions", () => {
      state.encryptWithPassword("CorporateSecret", "MasterSecret", { canPrint: false, canEdit: false });
      expect(state.isEncrypted).toBe(true);

      // Incorrect password fails decryption
      const wrongAttempt = state.decryptPassword("WrongPwd");
      expect(wrongAttempt).toBe(false);
      expect(state.isEncrypted).toBe(true);

      // Correct password unlocks document
      const decryptBtn = ribbonBody.querySelector("[data-testid='pdf-btn-decrypt']") as HTMLButtonElement;
      expect(decryptBtn).not.toBeNull();
      decryptBtn.click();

      expect(actions).toContainEqual({ action: "DecryptPassword", payload: undefined });
      state.decryptPassword("CorporateSecret");
      expect(state.isEncrypted).toBe(false);
      expect(state.getSecurityStatus()).toBe("🔓 Standard");
    });

    it("4.3 compresses PDF size and returns size reduction statistics", () => {
      const compressBtn = ribbonBody.querySelector("[data-testid='pdf-btn-compress']") as HTMLButtonElement;
      expect(compressBtn).not.toBeNull();
      compressBtn.click();

      expect(actions).toContainEqual({ action: "CompressPdf", payload: undefined });
      const stats = state.compressPdf(0.6);
      expect(stats.originalSizeBytes).toBeGreaterThan(0);
      expect(stats.compressedSizeBytes).toBeLessThan(stats.originalSizeBytes);
      expect(stats.savingsPercent).toBeGreaterThan(0);
    });
  });
});
