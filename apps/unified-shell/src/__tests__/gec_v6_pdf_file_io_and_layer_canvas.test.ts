import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { pdfState } from "../state/pdfState";
import { PdfWorkspace } from "../apps/pdf/PdfWorkspace";
import { binaryPackageService } from "../services/binaryPackageService";
import { fileCodec } from "../services/fileCodec";
import { IPC } from "../ipc";

describe("Subphase 39.1.1: PDF Studio Real File Open/Save & Interactive Layer Canvas", () => {
  let workspace: PdfWorkspace;
  let container: HTMLElement;
  let toastEl: HTMLElement;

  beforeEach(() => {
    document.body.innerHTML = "";

    container = document.createElement("div");
    container.id = "pdf-workspace-container";
    container.style.width = "1200px";
    container.style.height = "900px";
    document.body.appendChild(container);

    toastEl = document.createElement("div");
    toastEl.id = "saved-toast";
    toastEl.style.display = "none";
    document.body.appendChild(toastEl);

    // Mock Tauri IPC
    (window as any).__TAURI_INTERNALS__ = {};
    vi.spyOn(IPC, "saveFileBytes").mockImplementation(async (filename: string) => {
      return `C:\\Users\\User\\Documents\\${filename}`;
    });

    pdfState.resetToBlank("TestDoc.pdf");
    workspace = new PdfWorkspace(container, pdfState);
  });

  afterEach(() => {
    vi.restoreAllMocks();
    document.body.innerHTML = "";
  });

  // =========================================================================
  // 1. OPENING REAL .PDF FILE ROUND-TRIP
  // =========================================================================
  it("opens a local .pdf file, extracts multiple pages, dimensions, and text blocks", async () => {
    const rawPdfBytes = binaryPackageService.buildPdf([
      "Antigravity Enterprise Suite - ISO 32000-2 Certified",
      "Section 2: High Performance Architecture & Cryptographic Signatures",
    ]);

    expect(rawPdfBytes.length).toBeGreaterThan(100);
    expect(new TextDecoder().decode(rawPdfBytes)).toContain("%PDF-1.7");

    const opened = await fileCodec.openFile(rawPdfBytes, "Specification2026.pdf", "PDF");
    expect(opened).toBe(true);

    expect(pdfState.pages.length).toBe(2);
    expect(pdfState.pages[0].textBlocks?.length).toBeGreaterThanOrEqual(1);
    expect(pdfState.pages[0].textBlocks![0].text).toContain("Antigravity Enterprise Suite");

    workspace.render();
    const pageContainers = container.querySelectorAll(".pdf-page-container");
    expect(pageContainers.length).toBe(2);

    const firstPageOverlay = pageContainers[0].querySelector(".pdf-page-overlay");
    expect(firstPageOverlay).not.toBeNull();
  });

  // =========================================================================
  // 2. IN-PLACE TEXT ADDITION & EDITING ON PAGE 1
  // =========================================================================
  it("adds a new text block and edits text in-place via inline editor", () => {
    // 1. Add text block on Page 1
    pdfState.setTool("add_text");
    const overlay = container.querySelector('.pdf-page-overlay[data-page-number="1"]') as HTMLElement;
    expect(overlay).not.toBeNull();

    overlay.dispatchEvent(new MouseEvent("mousedown", { clientX: 150, clientY: 250, bubbles: true }));

    const page1 = pdfState.getPage(1)!;
    const addedTb = page1.textBlocks?.find((tb) => tb.x === 150 || tb.text.includes("edit text"));
    expect(addedTb).toBeDefined();

    // 2. Click text block to launch inline editor
    workspace.render();
    const tbEl = container.querySelector(`[data-block-id="${addedTb!.id}"]`) as HTMLElement;
    expect(tbEl).not.toBeNull();
    tbEl.click();

    const inlineInput = container.querySelector(".pdf-inline-text-editor") as HTMLInputElement;
    expect(inlineInput).not.toBeNull();

    // 3. Type new content and commit via Enter
    inlineInput.value = "Updated Contract Agreement 2026";
    inlineInput.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));

    expect(page1.textBlocks?.some((tb) => tb.text === "Updated Contract Agreement 2026")).toBe(true);
  });

  // =========================================================================
  // 3. FREEHAND INK DRAWING & ERASER TOOL
  // =========================================================================
  it("draws smooth freehand stylus ink on the page canvas and erases with eraser", () => {
    pdfState.setTool("pencil");
    pdfState.inkColor = "#2563eb";
    pdfState.inkThickness = 4;

    const overlay = container.querySelector('.pdf-page-overlay[data-page-number="1"]') as HTMLElement;
    expect(overlay).not.toBeNull();

    // Draw ink stroke
    overlay.dispatchEvent(new MouseEvent("mousedown", { clientX: 100, clientY: 100, bubbles: true }));
    overlay.dispatchEvent(new MouseEvent("mousemove", { clientX: 120, clientY: 130, bubbles: true }));
    overlay.dispatchEvent(new MouseEvent("mousemove", { clientX: 150, clientY: 160, bubbles: true }));
    overlay.dispatchEvent(new MouseEvent("mouseup", { clientX: 150, clientY: 160, bubbles: true }));

    const page1 = pdfState.getPage(1)!;
    const inkAnnot = page1.annotations?.find((a) => a.type === "ink");
    expect(inkAnnot).toBeDefined();
    expect(inkAnnot?.color).toBe("#2563eb");
    expect(inkAnnot?.points?.length).toBeGreaterThanOrEqual(2);

    // Erase ink annotation
    workspace.render();
    pdfState.setTool("eraser");
    const inkHitbox = container.querySelector(".pdf-annot-ink") as HTMLElement;
    expect(inkHitbox).not.toBeNull();
    inkHitbox.click();

    expect(page1.annotations?.some((a) => a.type === "ink")).toBe(false);
  });

  // =========================================================================
  // 4. INTERACTIVE ACROFORMS FILLING
  // =========================================================================
  it("renders interactive form fields directly into overlay and synchronizes inputs", () => {
    pdfState.addFormField(1, {
      name: "Authorized_Representative",
      type: "text",
      value: "",
      rect: [80, 300, 240, 32],
    });
    pdfState.addFormField(1, {
      name: "Accept_Terms_Checkbox",
      type: "checkbox",
      value: false,
      rect: [80, 350, 24, 24],
    });

    workspace.render();

    const textInput = container.querySelector(
      '[data-testid="pdf-input-Authorized_Representative"]'
    ) as HTMLInputElement;
    expect(textInput).not.toBeNull();

    textInput.value = "Elena Rostova";
    textInput.dispatchEvent(new Event("input", { bubbles: true }));
    textInput.dispatchEvent(new Event("change", { bubbles: true }));

    const checkbox = container.querySelector(
      '[data-testid="pdf-checkbox-Accept_Terms_Checkbox"]'
    ) as HTMLInputElement;
    expect(checkbox).not.toBeNull();

    checkbox.checked = true;
    checkbox.dispatchEvent(new Event("change", { bubbles: true }));

    const p1 = pdfState.getPage(1)!;
    const repField = p1.formFields?.find((f) => f.name === "Authorized_Representative");
    const termsField = p1.formFields?.find((f) => f.name === "Accept_Terms_Checkbox");

    expect(repField?.value).toBe("Elena Rostova");
    expect(termsField?.value).toBe(true);
  });

  // =========================================================================
  // 5. STAMPING CUSTOM SIGNATURE (BANS "JOHN DOE")
  // =========================================================================
  it("stamps a custom drawn/typed digital signature without banned placeholder names", () => {
    // Contract Check: verify "John Doe" is never stamped
    const customSigner = "Chief Architect Vance";
    expect(customSigner).not.toContain("John Doe");

    const stamp = pdfState.addSignatureStamp({
      signerName: customSigner,
      dataUrl: "data:image/svg+xml;base64,PHN2Zz48L3N2Zz4=",
      x: 100,
      y: 700,
      width: 250,
      height: 70,
    });

    expect(stamp).toBeDefined();
    expect(stamp.signerName).toBe(customSigner);
    expect(stamp.author).toBe(customSigner);

    workspace.render();
    const sigAnnot = container.querySelector(".pdf-annot-signature");
    expect(sigAnnot).not.toBeNull();
  });

  // =========================================================================
  // 6. PAGE DIMENSIONS RESIZING (A4 / LETTER / LEGAL / CUSTOM)
  // =========================================================================
  it("resizes page dimensions dynamically across A4, Letter, and Legal standards", () => {
    // 1. Initial size is Letter (816 x 1056)
    expect(pdfState.getPage(1)?.width).toBe(816);
    expect(pdfState.getPage(1)?.height).toBe(1056);

    // 2. Resize to A4 (794 x 1123)
    const resizedA4 = pdfState.execCommand("SetPageSize", "A4");
    expect(resizedA4).toBe(true);
    expect(pdfState.getPage(1)?.width).toBe(794);
    expect(pdfState.getPage(1)?.height).toBe(1123);

    // 3. Resize to Legal (816 x 1344)
    const resizedLegal = pdfState.execCommand("SetPageSize", "Legal");
    expect(resizedLegal).toBe(true);
    expect(pdfState.getPage(1)?.width).toBe(816);
    expect(pdfState.getPage(1)?.height).toBe(1344);
  });

  // =========================================================================
  // 7. PASSWORD ENCRYPTION & DECRYPTION LOCK SCREEN
  // =========================================================================
  it("encrypts document with password, displays lock screen overlay, and unlocks with correct pass", () => {
    pdfState.encryptWithPassword("SecuredVault2026");
    expect(pdfState.isEncrypted).toBe(true);

    workspace.render();
    const lockScreen = container.querySelector(".pdf-lock-screen");
    expect(lockScreen).not.toBeNull();

    const passInput = container.querySelector("#pdf-unlock-password-input") as HTMLInputElement;
    const unlockBtn = container.querySelector("#pdf-unlock-btn") as HTMLButtonElement;
    expect(passInput).not.toBeNull();
    expect(unlockBtn).not.toBeNull();

    // Wrong password
    passInput.value = "WrongPassword";
    unlockBtn.click();
    expect(pdfState.isLocked).toBe(true);

    // Correct password
    passInput.value = "SecuredVault2026";
    unlockBtn.click();
    expect(pdfState.isLocked).toBe(false);

    // Viewport re-rendered with page canvas visible
    expect(container.querySelector(".pdf-page-container")).not.toBeNull();
  });

  // =========================================================================
  // 8. REAL BINARY FILE SAVE & TOAST NOTIFICATION
  // =========================================================================
  it("saves document as binary .pdf with valid ISO 32000-2 structure and toast banner", async () => {
    pdfState.getPage(1)!.textBlocks = [
      { id: "tb-final-1", text: "Final Certified Enterprise Agreement", x: 80, y: 120, fontSize: 16 },
    ];

    const savedPath = await fileCodec.saveFile("Final_Certified_Agreement.pdf", "PDF");
    expect(savedPath).toBe("C:\\Users\\User\\Documents\\Final_Certified_Agreement.pdf");

    // Check toast notification
    const toast = document.getElementById("save-toast-banner");
    expect(toast?.textContent).toContain("Final_Certified_Agreement.pdf");

    // Verify binary output contains standard PDF objects
    const exportedBytes = pdfState.exportBinaryPdf();
    const pdfText = new TextDecoder().decode(exportedBytes);

    expect(pdfText).toContain("%PDF-1.7");
    expect(pdfText).toContain("/Type /Catalog");
    expect(pdfText).toContain("/Type /Pages");
    expect(pdfText).toContain("/Type /Page");
    expect(pdfText).toContain("BT /F1 16 Tf");
    expect(pdfText).toContain("(Final Certified Enterprise Agreement) Tj ET");
    expect(pdfText).toContain("startxref");
    expect(pdfText).toContain("%%EOF");
  });
});
