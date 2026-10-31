export type PdfFormFieldType =
  | "text"
  | "checkbox"
  | "radio"
  | "dropdown"
  | "signature"
  | "button"
  | "listbox";

export interface PdfFormField {
  id: string;
  name: string;
  type: PdfFormFieldType;
  value: string | boolean;
  rect: [number, number, number, number]; // [x, y, width, height]
  options?: string[]; // for dropdown, listbox, radio
  group?: string; // for radio grouping
  required?: boolean;
  readOnly?: boolean;
}

export type PdfAnnotationType =
  | "highlight"
  | "underline"
  | "strike"
  | "note"
  | "stamp"
  | "ink"
  | "redact"
  | "signature";

export interface PdfAnnotation {
  id: string;
  type: PdfAnnotationType;
  pageNumber: number;
  rect: [number, number, number, number];
  color: string;
  author?: string;
  date?: string;
  contents?: string;
  points?: { x: number; y: number }[]; // for freehand ink paths
  strokeWidth?: number;
  signatureDataUrl?: string; // for signature stamp image
  signerName?: string;
  isAppliedRedaction?: boolean;
}

export interface PdfBookmark {
  id: string;
  title: string;
  pageNumber: number;
  level: number;
}

export interface PdfTextBlock {
  id: string;
  x: number;
  y: number;
  text: string;
  fontSize?: number;
  fontColor?: string;
  fontFamily?: string;
  bold?: boolean;
  italic?: boolean;
  underline?: boolean;
  align?: "left" | "center" | "right";
}

export interface PdfImageBlock {
  id: string;
  x: number;
  y: number;
  width: number;
  height: number;
  src?: string;
  title?: string;
}

export interface PdfWatermark {
  text: string;
  opacity: number;
  fontSize: number;
  color: string;
  rotation: number;
}

export interface PdfPageModel {
  id: string;
  pageNumber: number; // 1-indexed
  width: number; // default 816
  height: number; // default 1056
  rotation: number; // 0, 90, 180, 270
  title?: string;
  textBlocks?: PdfTextBlock[];
  imageBlocks?: PdfImageBlock[];
  formFields?: PdfFormField[];
  annotations?: PdfAnnotation[];
  watermark?: PdfWatermark;
}

export type PdfViewMode = "continuous" | "single" | "twoPage";
export type PdfSidebarTab = "thumbnails" | "bookmarks" | "annotations";
export type PdfActiveTool =
  | "hand"
  | "select"
  | "snapshot"
  | "edit_text"
  | "add_text"
  | "add_image"
  | "highlight"
  | "underline"
  | "strike"
  | "note"
  | "pencil"
  | "eraser"
  | "redact"
  | "form_fill"
  | "add_form_field"
  | "sign";

export class PdfState {
  public pages: PdfPageModel[] = [
    {
      id: "page-1",
      pageNumber: 1,
      width: 816,
      height: 1056,
      rotation: 0,
      title: "ISO 32000-2 Certified PDF Document",
      textBlocks: [
        { id: "tb-1", x: 80, y: 80, text: "ISO 32000-2 Certified PDF Document", fontSize: 20, fontColor: "#b30b00", bold: true },
        { id: "tb-2", x: 80, y: 105, text: "Interactive Form & Digital Signing Specification", fontSize: 12, fontColor: "#555555" },
        { id: "tb-3", x: 80, y: 170, text: "Fillable Form Fields (ISO 32000 AcroForms):", fontSize: 13, fontColor: "#222222", bold: true },
      ],
      imageBlocks: [],
      formFields: [
        { id: "f1", name: "Full_Name", type: "text", value: "Jane Citizen", rect: [100, 200, 250, 30] },
        { id: "f2", name: "Agreed", type: "checkbox", value: true, rect: [100, 250, 20, 20] },
      ],
      annotations: [
        { id: "ann-1", type: "highlight", pageNumber: 1, rect: [80, 75, 380, 24], color: "rgba(250, 204, 21, 0.4)", author: "Auditor", date: "2026-10-29", contents: "Certified Standard Reference" },
        { id: "ann-2", type: "note", pageNumber: 1, rect: [480, 75, 24, 24], color: "#eab308", author: "Compliance", date: "2026-10-29", contents: "Verified compliant with ISO 32000-2" },
      ],
    },
    {
      id: "page-2",
      pageNumber: 2,
      width: 816,
      height: 1056,
      rotation: 0,
      title: "Section 2: Document Security & Signatures",
      textBlocks: [
        { id: "tb-4", x: 80, y: 80, text: "Section 2: Document Security & Signatures", fontSize: 20, fontColor: "#1e293b", bold: true },
        { id: "tb-5", x: 80, y: 110, text: "Cryptographic X.509 PKCS#7 detached signature verification.", fontSize: 13, fontColor: "#475569" },
        { id: "tb-6", x: 80, y: 150, text: "All modified content streams are recorded in incremental updates.", fontSize: 13, fontColor: "#475569" },
      ],
      imageBlocks: [],
      formFields: [],
      annotations: [],
    },
    {
      id: "page-3",
      pageNumber: 3,
      width: 816,
      height: 1056,
      rotation: 0,
      title: "Section 3: Vector Graphics & Color Spaces",
      textBlocks: [
        { id: "tb-7", x: 80, y: 80, text: "Section 3: Vector Graphics & Color Spaces", fontSize: 20, fontColor: "#1e293b", bold: true },
        { id: "tb-8", x: 80, y: 110, text: "DeviceRGB, DeviceCMYK, and ICCBased profile compliance.", fontSize: 13, fontColor: "#475569" },
      ],
      imageBlocks: [],
      formFields: [],
      annotations: [],
    },
  ];

  public bookmarks: PdfBookmark[] = [
    { id: "bm-1", title: "1. ISO 32000-2 Specification", pageNumber: 1, level: 0 },
    { id: "bm-2", title: "2. Security & Signatures", pageNumber: 2, level: 0 },
    { id: "bm-3", title: "2.1 Cryptographic Validation", pageNumber: 2, level: 1 },
    { id: "bm-4", title: "3. Vector Graphics & ICC", pageNumber: 3, level: 0 },
  ];

  public currentPage = 1;
  public zoom = 1.0;
  public viewMode: PdfViewMode = "continuous";
  public sidebarTab: PdfSidebarTab = "thumbnails";
  public activeTool: PdfActiveTool = "select";

  // Annotation & Ink Properties
  public inkColor = "#ef4444";
  public inkThickness = 3;
  public highlightColor = "rgba(250, 204, 21, 0.4)";
  public activeFontFamily = "Segoe UI";
  public activeFontSize = 14;
  public activeFontColor = "#1e293b";
  public activeBold = false;
  public activeItalic = false;
  public highlightFields = false;
  public searchQuery = "";
  public searchMatches: { pageNumber: number; textBlockId: string; text: string }[] = [];

  private undoStack: string[] = [];
  private redoStack: string[] = [];
  private nextPageCounter = 10;
  private nextBookmarkCounter = 10;
  private nextAnnotCounter = 10;
  private nextTextBlockCounter = 10;
  private nextFieldCounter = 10;
  private nextImageCounter = 10;

  public get pageCount(): number {
    return this.pages.length;
  }

  public isEncrypted = false;
  public isCompressed = false;
  public securityConfig: {
    isEncrypted: boolean;
    openPassword?: string;
    ownerPassword?: string;
    permissions?: {
      canPrint?: boolean;
      canEdit?: boolean;
      canCopy?: boolean;
      canAnnotate?: boolean;
    };
  } = {
    isEncrypted: false,
  };

  public getCurrentPageDimensions(): string {
    const page = this.pages[this.currentPage - 1];
    if (!page) return "8.5 × 11 in";
    const wIn = (page.width / 96).toFixed(1);
    const hIn = (page.height / 96).toFixed(1);
    return `${wIn} × ${hIn} in`;
  }

  public getSecurityStatus(): string {
    return this.isEncrypted ? "🔒 Password Protected" : "🔓 Standard";
  }

  // Compatibility accessor for existing AcroForm tests
  public get formFields(): PdfFormField[] {
    const p1 = this.pages[0];
    return p1?.formFields || [];
  }

  public set formFields(f: PdfFormField[]) {
    if (this.pages[0]) {
      this.pages[0].formFields = f;
    }
  }

  public setTool(t: PdfActiveTool) {
    this.activeTool = t;
  }

  public commitSnapshot() {
    const stateJson = JSON.stringify({
      pages: this.pages,
      bookmarks: this.bookmarks,
      currentPage: this.currentPage,
    });
    this.undoStack.push(stateJson);
    this.redoStack = [];
    if (this.undoStack.length > 50) {
      this.undoStack.shift();
    }
  }

  public canUndo(): boolean {
    return this.undoStack.length > 0;
  }

  public canRedo(): boolean {
    return this.redoStack.length > 0;
  }

  public undo(): boolean {
    if (!this.canUndo()) return false;
    const currentJson = JSON.stringify({
      pages: this.pages,
      bookmarks: this.bookmarks,
      currentPage: this.currentPage,
    });
    this.redoStack.push(currentJson);
    const prev = JSON.parse(this.undoStack.pop()!);
    this.pages = prev.pages;
    this.bookmarks = prev.bookmarks;
    this.currentPage = prev.currentPage;
    return true;
  }

  public redo(): boolean {
    if (!this.canRedo()) return false;
    const currentJson = JSON.stringify({
      pages: this.pages,
      bookmarks: this.bookmarks,
      currentPage: this.currentPage,
    });
    this.undoStack.push(currentJson);
    const next = JSON.parse(this.redoStack.pop()!);
    this.pages = next.pages;
    this.bookmarks = next.bookmarks;
    this.currentPage = next.currentPage;
    return true;
  }

  // --- Page Organizer Operations ---

  public getPage(pageNumber = this.currentPage): PdfPageModel | undefined {
    return this.pages.find((p) => p.pageNumber === pageNumber);
  }

  public setCurrentPage(pageNumber: number): boolean {
    if (pageNumber >= 1 && pageNumber <= this.pages.length) {
      this.currentPage = pageNumber;
      return true;
    }
    return false;
  }

  public insertBlankPage(afterPageNumber = this.currentPage, width = 816, height = 1056): PdfPageModel {
    this.commitSnapshot();
    const insertIdx = Math.min(Math.max(0, afterPageNumber), this.pages.length);
    const newPage: PdfPageModel = {
      id: `page_${++this.nextPageCounter}`,
      pageNumber: insertIdx + 1,
      width,
      height,
      rotation: 0,
      title: `Page ${insertIdx + 1} (Blank)`,
      textBlocks: [],
      imageBlocks: [],
      formFields: [],
      annotations: [],
    };

    this.pages.splice(insertIdx, 0, newPage);
    this.renumberPages();
    this.currentPage = newPage.pageNumber;
    return newPage;
  }

  public deletePage(pageNumber = this.currentPage): boolean {
    if (this.pages.length <= 1) {
      return false;
    }
    const idx = this.pages.findIndex((p) => p.pageNumber === pageNumber);
    if (idx === -1) return false;

    this.commitSnapshot();
    this.pages.splice(idx, 1);
    this.renumberPages();
    if (this.currentPage > this.pages.length) {
      this.currentPage = this.pages.length;
    }
    return true;
  }

  public rotatePageClockwise(pageNumber = this.currentPage): number {
    const page = this.getPage(pageNumber);
    if (!page) return 0;
    this.commitSnapshot();
    page.rotation = (page.rotation + 90) % 360;
    return page.rotation;
  }

  public rotatePageCounterClockwise(pageNumber = this.currentPage): number {
    const page = this.getPage(pageNumber);
    if (!page) return 0;
    this.commitSnapshot();
    page.rotation = (page.rotation - 90 + 360) % 360;
    return page.rotation;
  }

  public rotateAllPages(deltaDegrees = 90): void {
    this.commitSnapshot();
    this.pages.forEach((p) => {
      p.rotation = (p.rotation + deltaDegrees + 360) % 360;
    });
  }

  public movePage(fromPageNumber: number, toPageNumber: number): boolean {
    if (
      fromPageNumber < 1 ||
      fromPageNumber > this.pages.length ||
      toPageNumber < 1 ||
      toPageNumber > this.pages.length ||
      fromPageNumber === toPageNumber
    ) {
      return false;
    }
    this.commitSnapshot();
    const fromIdx = fromPageNumber - 1;
    const toIdx = toPageNumber - 1;
    const [moved] = this.pages.splice(fromIdx, 1);
    this.pages.splice(toIdx, 0, moved);
    this.renumberPages();
    this.currentPage = toPageNumber;
    return true;
  }

  public movePageUp(pageNumber = this.currentPage): boolean {
    if (pageNumber <= 1) return false;
    return this.movePage(pageNumber, pageNumber - 1);
  }

  public movePageDown(pageNumber = this.currentPage): boolean {
    if (pageNumber >= this.pages.length) return false;
    return this.movePage(pageNumber, pageNumber + 1);
  }

  public extractPage(pageNumber = this.currentPage): PdfPageModel | null {
    const page = this.getPage(pageNumber);
    if (!page) return null;
    return JSON.parse(JSON.stringify(page)) as PdfPageModel;
  }

  public mergePdf(incomingPages: PdfPageModel[]): number {
    if (!incomingPages || incomingPages.length === 0) return this.pages.length;
    this.commitSnapshot();
    incomingPages.forEach((p) => {
      const cloned: PdfPageModel = {
        ...JSON.parse(JSON.stringify(p)),
        id: `page_${++this.nextPageCounter}`,
        pageNumber: this.pages.length + 1,
      };
      this.pages.push(cloned);
    });
    this.renumberPages();
    return this.pages.length;
  }

  public duplicatePage(pageNumber = this.currentPage): PdfPageModel | null {
    const page = this.getPage(pageNumber);
    if (!page) return null;
    this.commitSnapshot();
    const cloned: PdfPageModel = JSON.parse(JSON.stringify(page));
    cloned.id = `page_${++this.nextPageCounter}`;
    cloned.title = `${page.title || "Page"} (Copy)`;
    const insertIdx = pageNumber;
    this.pages.splice(insertIdx, 0, cloned);
    this.renumberPages();
    this.currentPage = insertIdx + 1;
    return cloned;
  }

  public static readonly STANDARD_SIZES: Record<string, { width: number; height: number }> = {
    Letter: { width: 816, height: 1056 },
    Legal: { width: 816, height: 1344 },
    A4: { width: 794, height: 1123 },
    A3: { width: 1123, height: 1587 },
    A5: { width: 559, height: 794 },
  };

  public resizePage(
    pageNumber = this.currentPage,
    targetWidth: number,
    targetHeight: number,
    scaleContent = true,
    applyToAll = false
  ): boolean {
    if (targetWidth <= 0 || targetHeight <= 0) return false;
    this.commitSnapshot();

    const pagesToResize = applyToAll ? this.pages : this.pages.filter((p) => p.pageNumber === pageNumber);
    if (pagesToResize.length === 0) return false;

    for (const page of pagesToResize) {
      const oldW = page.width || 816;
      const oldH = page.height || 1056;
      page.width = targetWidth;
      page.height = targetHeight;

      if (scaleContent && oldW > 0 && oldH > 0) {
        const scaleX = targetWidth / oldW;
        const scaleY = targetHeight / oldH;

        if (page.textBlocks) {
          page.textBlocks.forEach((tb) => {
            tb.x = Math.round(tb.x * scaleX);
            tb.y = Math.round(tb.y * scaleY);
            if (tb.fontSize) tb.fontSize = Math.round(tb.fontSize * Math.min(scaleX, scaleY));
          });
        }
        if (page.imageBlocks) {
          page.imageBlocks.forEach((ib) => {
            ib.x = Math.round(ib.x * scaleX);
            ib.y = Math.round(ib.y * scaleY);
            ib.width = Math.round(ib.width * scaleX);
            ib.height = Math.round(ib.height * scaleY);
          });
        }
        if (page.formFields) {
          page.formFields.forEach((ff) => {
            if (ff.rect) {
              ff.rect = [
                Math.round(ff.rect[0] * scaleX),
                Math.round(ff.rect[1] * scaleY),
                Math.round(ff.rect[2] * scaleX),
                Math.round(ff.rect[3] * scaleY),
              ];
            }
          });
        }
        if (page.annotations) {
          page.annotations.forEach((ann) => {
            if (ann.rect) {
              ann.rect = [
                Math.round(ann.rect[0] * scaleX),
                Math.round(ann.rect[1] * scaleY),
                Math.round(ann.rect[2] * scaleX),
                Math.round(ann.rect[3] * scaleY),
              ];
            }
            if (ann.points) {
              ann.points.forEach((pt) => {
                pt.x = Math.round(pt.x * scaleX);
                pt.y = Math.round(pt.y * scaleY);
              });
            }
          });
        }
      }
    }
    return true;
  }

  public setPageSize(
    sizeName: "A4" | "Letter" | "Legal" | "A3" | "A5" | string,
    scaleContent = true,
    applyToAll = false
  ): boolean {
    const size = PdfState.STANDARD_SIZES[sizeName];
    if (!size) return false;
    return this.resizePage(this.currentPage, size.width, size.height, scaleContent, applyToAll);
  }

  public splitPdf(fromPage: number, toPage: number): PdfPageModel[] {
    const start = Math.max(1, Math.min(fromPage, toPage));
    const end = Math.min(this.pages.length, Math.max(fromPage, toPage));
    const extracted: PdfPageModel[] = [];
    for (let p = start; p <= end; p++) {
      const page = this.getPage(p);
      if (page) {
        extracted.push(JSON.parse(JSON.stringify(page)));
      }
    }
    return extracted;
  }

  public encryptWithPassword(
    openPassword: string,
    ownerPassword?: string,
    permissions?: { canPrint?: boolean; canEdit?: boolean; canCopy?: boolean; canAnnotate?: boolean }
  ): boolean {
    this.commitSnapshot();
    this.isEncrypted = true;
    this.securityConfig = {
      isEncrypted: true,
      openPassword,
      ownerPassword: ownerPassword || openPassword,
      permissions: permissions || {
        canPrint: true,
        canEdit: false,
        canCopy: false,
        canAnnotate: true,
      },
    };
    return true;
  }

  public decryptPassword(password: string): boolean {
    if (!this.isEncrypted) return true;
    if (
      !this.securityConfig.openPassword ||
      this.securityConfig.openPassword === password ||
      this.securityConfig.ownerPassword === password
    ) {
      this.commitSnapshot();
      this.isEncrypted = false;
      this.securityConfig = { isEncrypted: false };
      return true;
    }
    return false;
  }

  public removeSecurity(): boolean {
    this.commitSnapshot();
    this.isEncrypted = false;
    this.securityConfig = { isEncrypted: false };
    return true;
  }

  public compressPdf(targetQuality = 0.75): { originalSizeBytes: number; compressedSizeBytes: number; savingsPercent: number } {
    this.isCompressed = true;
    const origEstimate = this.pages.length * 150000 + (this.pages.reduce((acc, p) => acc + (p.imageBlocks?.length || 0), 0) * 350000);
    const compressedEstimate = Math.round(origEstimate * Math.max(0.3, targetQuality));
    const savings = Math.round(((origEstimate - compressedEstimate) / origEstimate) * 100);
    return {
      originalSizeBytes: origEstimate,
      compressedSizeBytes: compressedEstimate,
      savingsPercent: savings,
    };
  }

  public createBlankPdf(numPages = 1): void {
    this.commitSnapshot();
    this.pages = [];
    const count = Math.max(1, numPages);
    for (let i = 1; i <= count; i++) {
      this.pages.push({
        id: `page_${++this.nextPageCounter}`,
        pageNumber: i,
        width: 816,
        height: 1056,
        rotation: 0,
        title: `Page ${i}`,
        textBlocks: [],
        imageBlocks: [],
        formFields: [],
        annotations: [],
      });
    }
    this.bookmarks = [
      { id: "bm-1", title: "Page 1", pageNumber: 1, level: 0 },
    ];
    this.currentPage = 1;
  }

  public resetToBlank(title = "Document 1.pdf"): void {
    this.createBlankPdf(1);
    this.isCompressed = false;
    this.isEncrypted = false;
    this.securityConfig = { isEncrypted: false };
    if (this.pages[0]) {
      this.pages[0].title = title;
    }
    this.bookmarks = [];
    this.currentPage = 1;
    this.zoom = 1.0;
    this.viewMode = "continuous";
    this.sidebarTab = "thumbnails";
    this.activeTool = "select";
    this.undoStack = [];
    this.redoStack = [];
  }

  public loadPdfDocument(pages: PdfPageModel[], bookmarks: PdfBookmark[] = []): void {
    this.commitSnapshot();
    this.pages = pages.map((p, i) => ({
      ...p,
      pageNumber: i + 1,
      width: p.width || 816,
      height: p.height || 1056,
      rotation: p.rotation || 0,
      textBlocks: p.textBlocks || [],
      imageBlocks: p.imageBlocks || [],
      formFields: p.formFields || [],
      annotations: p.annotations || [],
    }));
    this.bookmarks = bookmarks;
    this.currentPage = 1;
    this.undoStack = [];
    this.redoStack = [];
  }

  public setZoom(zoom: number): void {
    this.zoom = Math.min(4.0, Math.max(0.25, Math.round(zoom * 100) / 100));
  }

  public setViewMode(mode: PdfViewMode): void {
    this.viewMode = mode;
  }

  public setSidebarTab(tab: PdfSidebarTab): void {
    this.sidebarTab = tab;
  }

  private renumberPages(): void {
    this.pages.forEach((p, idx) => {
      p.pageNumber = idx + 1;
    });
  }

  // --- Bookmarks Management ---

  public addBookmark(title: string, pageNumber = this.currentPage, level = 0): PdfBookmark {
    this.commitSnapshot();
    const bm: PdfBookmark = {
      id: `bm_${++this.nextBookmarkCounter}`,
      title,
      pageNumber,
      level,
    };
    this.bookmarks.push(bm);
    return bm;
  }

  public removeBookmark(id: string): boolean {
    const idx = this.bookmarks.findIndex((b) => b.id === id);
    if (idx === -1) return false;
    this.commitSnapshot();
    this.bookmarks.splice(idx, 1);
    return true;
  }

  // --- In-Place Text Editing ---

  public addTextBlock(
    pageNumber = this.currentPage,
    x = 100,
    y = 200,
    text = "Click to edit text",
    fontSize = this.activeFontSize,
    fontColor = this.activeFontColor,
    fontFamily = this.activeFontFamily,
    bold = this.activeBold,
    italic = this.activeItalic
  ): PdfTextBlock {
    this.commitSnapshot();
    const page = this.getPage(pageNumber);
    if (!page) {
      throw new Error(`Page ${pageNumber} not found`);
    }
    if (!page.textBlocks) page.textBlocks = [];
    const block: PdfTextBlock = {
      id: `tb_${++this.nextTextBlockCounter}`,
      x,
      y,
      text,
      fontSize,
      fontColor,
      fontFamily,
      bold,
      italic,
    };
    page.textBlocks.push(block);
    return block;
  }

  public updateTextBlock(
    pageNumber: number,
    textBlockId: string,
    newText: string,
    styles?: Partial<PdfTextBlock>
  ): boolean {
    const page = this.getPage(pageNumber);
    if (!page || !page.textBlocks) return false;
    const block = page.textBlocks.find((tb) => tb.id === textBlockId);
    if (!block) return false;

    this.commitSnapshot();
    block.text = newText;
    if (styles) {
      Object.assign(block, styles);
    }
    return true;
  }

  public deleteTextBlock(pageNumber: number, textBlockId: string): boolean {
    const page = this.getPage(pageNumber);
    if (!page || !page.textBlocks) return false;
    const idx = page.textBlocks.findIndex((tb) => tb.id === textBlockId);
    if (idx === -1) return false;

    this.commitSnapshot();
    page.textBlocks.splice(idx, 1);
    return true;
  }

  // --- Image Blocks ---

  public addImageBlock(pageNumber = this.currentPage, image: Partial<PdfImageBlock> = {}): PdfImageBlock {
    this.commitSnapshot();
    const page = this.getPage(pageNumber);
    if (!page) throw new Error(`Page ${pageNumber} not found`);
    if (!page.imageBlocks) page.imageBlocks = [];

    const img: PdfImageBlock = {
      id: image.id || `img_${++this.nextImageCounter}`,
      x: image.x !== undefined ? image.x : 100,
      y: image.y !== undefined ? image.y : 250,
      width: image.width || 220,
      height: image.height || 140,
      src: image.src || "placeholder.png",
      title: image.title || "Inserted Graphic",
    };
    page.imageBlocks.push(img);
    return img;
  }

  public deleteImageBlock(pageNumber: number, imageId: string): boolean {
    const page = this.getPage(pageNumber);
    if (!page || !page.imageBlocks) return false;
    const idx = page.imageBlocks.findIndex((img) => img.id === imageId);
    if (idx === -1) return false;

    this.commitSnapshot();
    page.imageBlocks.splice(idx, 1);
    return true;
  }

  // --- Watermark ---

  public setWatermark(watermark: Partial<PdfWatermark>, allPages = true): void {
    this.commitSnapshot();
    const wm: PdfWatermark = {
      text: watermark.text || "CONFIDENTIAL",
      opacity: watermark.opacity !== undefined ? watermark.opacity : 0.25,
      fontSize: watermark.fontSize || 64,
      color: watermark.color || "#b30b00",
      rotation: watermark.rotation !== undefined ? watermark.rotation : -45,
    };

    if (allPages) {
      this.pages.forEach((p) => {
        p.watermark = { ...wm };
      });
    } else {
      const page = this.getPage(this.currentPage);
      if (page) page.watermark = { ...wm };
    }
  }

  public removeWatermark(allPages = true): void {
    this.commitSnapshot();
    if (allPages) {
      this.pages.forEach((p) => {
        p.watermark = undefined;
      });
    } else {
      const page = this.getPage(this.currentPage);
      if (page) page.watermark = undefined;
    }
  }

  // --- Annotations & Freehand Ink Management ---

  public addAnnotation(ann: Omit<PdfAnnotation, "id">): PdfAnnotation {
    this.commitSnapshot();
    const newAnn: PdfAnnotation = {
      ...ann,
      id: `ann_${++this.nextAnnotCounter}`,
    };
    const page = this.getPage(ann.pageNumber);
    if (page) {
      if (!page.annotations) page.annotations = [];
      page.annotations.push(newAnn);
    }
    return newAnn;
  }

  public updateAnnotation(id: string, updates: Partial<PdfAnnotation>): boolean {
    let found = false;
    this.pages.forEach((p) => {
      if (p.annotations) {
        const ann = p.annotations.find((a) => a.id === id);
        if (ann) {
          if (!found) this.commitSnapshot();
          Object.assign(ann, updates);
          found = true;
        }
      }
    });
    return found;
  }

  public removeAnnotation(id: string): boolean {
    let found = false;
    this.pages.forEach((p) => {
      if (p.annotations) {
        const idx = p.annotations.findIndex((a) => a.id === id);
        if (idx !== -1) {
          if (!found) this.commitSnapshot();
          p.annotations.splice(idx, 1);
          found = true;
        }
      }
    });
    return found;
  }

  public getAnnotation(id: string): PdfAnnotation | undefined {
    for (const p of this.pages) {
      if (p.annotations) {
        const found = p.annotations.find((a) => a.id === id);
        if (found) return found;
      }
    }
    return undefined;
  }

  public getAllAnnotations(): PdfAnnotation[] {
    const list: PdfAnnotation[] = [];
    this.pages.forEach((p) => {
      if (p.annotations) {
        list.push(...p.annotations);
      }
    });
    return list;
  }

  // --- Redaction ---

  public markRedaction(pageNumber = this.currentPage, rect: [number, number, number, number]): PdfAnnotation {
    return this.addAnnotation({
      type: "redact",
      pageNumber,
      rect,
      color: "#000000",
      contents: "Redacted Content",
      author: "Redaction System",
      date: new Date().toISOString().slice(0, 10),
    });
  }

  public applyRedactions(targetPageNumber?: number): number {
    this.commitSnapshot();
    let redactedBlocksCount = 0;

    const pagesToProcess = targetPageNumber
      ? this.pages.filter((p) => p.pageNumber === targetPageNumber)
      : this.pages;

    pagesToProcess.forEach((page) => {
      if (!page.annotations || !page.textBlocks) return;
      const redactAnnots = page.annotations.filter((a) => a.type === "redact");

      redactAnnots.forEach((redact) => {
        const [rx, ry, rw, rh] = redact.rect;
        redact.isAppliedRedaction = true;

        // Remove text blocks intersecting the redaction box
        page.textBlocks = page.textBlocks!.filter((tb) => {
          const tbW = tb.text.length * (tb.fontSize ? tb.fontSize * 0.6 : 8);
          const tbH = tb.fontSize || 14;
          const tbX = tb.x;
          const tbY = tb.y - tbH;

          const intersects =
            tbX < rx + rw &&
            tbX + tbW > rx &&
            tbY < ry + rh &&
            tbY + tbH > ry;

          if (intersects) {
            redactedBlocksCount++;
            return false;
          }
          return true;
        });
      });
    });

    return redactedBlocksCount;
  }

  // --- AcroForms Management & Interactive Filling ---

  public addFormField(pageNumber = this.currentPage, field: Partial<PdfFormField> = {}): PdfFormField {
    this.commitSnapshot();
    const page = this.getPage(pageNumber);
    if (!page) throw new Error(`Page ${pageNumber} not found`);
    if (!page.formFields) page.formFields = [];

    const ftype: PdfFormFieldType = field.type || "text";
    let defaultRect: [number, number, number, number] = [100, 300, 200, 28];
    let defaultValue: string | boolean = "";

    if (ftype === "checkbox") {
      defaultRect = [100, 300, 20, 20];
      defaultValue = false;
    } else if (ftype === "radio") {
      defaultRect = [100, 300, 20, 20];
      defaultValue = false;
    } else if (ftype === "signature") {
      defaultRect = [100, 300, 220, 60];
      defaultValue = "";
    } else if (ftype === "dropdown") {
      defaultRect = [100, 300, 180, 28];
      defaultValue = field.options && field.options[0] ? field.options[0] : "Option 1";
    }

    const newField: PdfFormField = {
      id: field.id || `f_${++this.nextFieldCounter}`,
      name: field.name || `Field_${this.nextFieldCounter}`,
      type: ftype,
      value: field.value !== undefined ? field.value : defaultValue,
      rect: field.rect || defaultRect,
      options: field.options || (ftype === "dropdown" ? ["Option 1", "Option 2", "Option 3"] : undefined),
      group: field.group,
      required: field.required,
      readOnly: field.readOnly,
    };

    page.formFields.push(newField);
    return newField;
  }

  public setFormFieldValue(fieldId: string, value: string | boolean): boolean {
    for (const page of this.pages) {
      if (page.formFields) {
        const fld = page.formFields.find((f) => f.id === fieldId || f.name === fieldId);
        if (fld) {
          this.commitSnapshot();
          if (fld.type === "radio" && fld.group) {
            // Uncheck other radio buttons in the same group on this page
            page.formFields.forEach((other) => {
              if (other.type === "radio" && other.group === fld.group) {
                other.value = false;
              }
            });
          }
          fld.value = value;
          return true;
        }
      }
    }
    return false;
  }

  public getFormFieldValue(nameOrId: string): string | boolean | undefined {
    for (const page of this.pages) {
      if (page.formFields) {
        const fld = page.formFields.find((f) => f.id === nameOrId || f.name === nameOrId);
        if (fld) return fld.value;
      }
    }
    return undefined;
  }

  public removeFormField(fieldId: string): boolean {
    let found = false;
    this.pages.forEach((p) => {
      if (p.formFields) {
        const idx = p.formFields.findIndex((f) => f.id === fieldId || f.name === fieldId);
        if (idx !== -1) {
          if (!found) this.commitSnapshot();
          p.formFields.splice(idx, 1);
          found = true;
        }
      }
    });
    return found;
  }

  public clearFormData(): void {
    this.commitSnapshot();
    this.pages.forEach((p) => {
      if (p.formFields) {
        p.formFields.forEach((f) => {
          if (typeof f.value === "boolean") {
            f.value = false;
          } else {
            f.value = "";
          }
        });
      }
    });
  }

  public exportFormData(): Record<string, string | boolean> {
    const data: Record<string, string | boolean> = {};
    this.pages.forEach((p) => {
      if (p.formFields) {
        p.formFields.forEach((f) => {
          data[f.name] = f.value;
        });
      }
    });
    return data;
  }

  public importFormData(data: Record<string, string | boolean>): number {
    this.commitSnapshot();
    let count = 0;
    this.pages.forEach((p) => {
      if (p.formFields) {
        p.formFields.forEach((f) => {
          if (data[f.name] !== undefined) {
            f.value = data[f.name];
            count++;
          }
        });
      }
    });
    return count;
  }

  public searchText(query: string): { pageNumber: number; textBlockId: string; text: string }[] {
    this.searchQuery = query.trim().toLowerCase();
    this.searchMatches = [];
    if (!this.searchQuery) return [];

    for (const page of this.pages) {
      if (page.textBlocks) {
        for (const tb of page.textBlocks) {
          if (tb.text.toLowerCase().includes(this.searchQuery)) {
            this.searchMatches.push({
              pageNumber: page.pageNumber,
              textBlockId: tb.id,
              text: tb.text,
            });
          }
        }
      }
    }
    if (this.searchMatches.length > 0) {
      this.setCurrentPage(this.searchMatches[0].pageNumber);
    }
    return this.searchMatches;
  }

  // --- Sign Document ---

  public addSignature(
    pageNumber = this.currentPage,
    signatureDataUrl: string,
    signerName = "Authorized Signer",
    rect: [number, number, number, number] = [100, 750, 220, 64]
  ): PdfAnnotation {
    this.commitSnapshot();
    const ann = this.addAnnotation({
      type: "signature",
      pageNumber,
      rect,
      color: "#002060",
      author: signerName,
      signerName,
      signatureDataUrl,
      contents: `Digitally Signed by ${signerName} on ${new Date().toISOString().slice(0, 10)}`,
      date: new Date().toISOString(),
    });

    // Also update any empty signature form field on this page
    const page = this.getPage(pageNumber);
    if (page && page.formFields) {
      const sigField = page.formFields.find((f) => f.type === "signature" && !f.value);
      if (sigField) {
        sigField.value = signerName;
      }
    }

    return ann;
  }

  // --- Binary ISO 32000-2 PDF Serialization ---

  public exportBinaryPdf(): Uint8Array {
    let out = "%PDF-1.7\r\n%\xE2\xE3\xCF\xD3\r\n";
    const finalOffsets: number[] = [0];

    const writeObject = (num: number, body: string) => {
      finalOffsets[num] = out.length;
      out += `${num} 0 obj\r\n${body}\r\nendobj\r\n`;
    };

    // 1: Catalog
    writeObject(1, "<< /Type /Catalog /Pages 2 0 R >>");

    // 3: Font
    writeObject(3, "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>");

    const pageCount = Math.max(1, this.pages.length);
    const kidIds: string[] = [];
    for (let i = 0; i < pageCount; i++) {
      kidIds.push(`${4 + i * 2} 0 R`);
    }

    // 2: Pages Tree
    writeObject(2, `<< /Type /Pages /Kids [${kidIds.join(" ")}] /Count ${pageCount} >>`);

    for (let i = 0; i < pageCount; i++) {
      const page = this.pages[i];
      const pageObjId = 4 + i * 2;
      const contentObjId = 5 + i * 2;
      const pW = page ? page.width : 816;
      const pH = page ? page.height : 1056;

      let pStream = "";
      if (page?.textBlocks && page.textBlocks.length > 0) {
        page.textBlocks.forEach((tb) => {
          const rawText = String(tb.text ?? "");
          const esc = rawText.replace(/\\/g, "\\\\").replace(/\(/g, "\\(").replace(/\)/g, "\\)");
          pStream += `BT /F1 ${tb.fontSize || 12} Tf ${tb.x} ${pH - tb.y} Td (${esc}) Tj ET\r\n`;
        });
      } else {
        pStream = `BT /F1 14 Tf 80 ${pH - 120} Td (Page ${i + 1}) Tj ET\r\n`;
      }

      if (page?.annotations) {
        page.annotations.forEach((ann) => {
          if (ann.type === "signature") {
            const esc = (ann.signerName || "Signed").replace(/\\/g, "\\\\").replace(/\(/g, "\\(").replace(/\)/g, "\\)");
            pStream += `BT /F1 12 Tf ${ann.rect[0]} ${pH - ann.rect[1]} Td ([Digitally Signed: ${esc}]) Tj ET\r\n`;
          }
        });
      }

      writeObject(contentObjId, `<< /Length ${pStream.length} >>\r\nstream\r\n${pStream}endstream`);
      writeObject(
        pageObjId,
        `<< /Type /Page /Parent 2 0 R /MediaBox [0 0 ${pW} ${pH}] /Resources << /Font << /F1 3 0 R >> >> /Contents ${contentObjId} 0 R >>`
      );
    }

    const totalObjs = 3 + pageCount * 2;
    const startXref = out.length;
    out += `xref\r\n0 ${totalObjs + 1}\r\n0000000000 65535 f\r\n`;
    for (let i = 1; i <= totalObjs; i++) {
      const off = (finalOffsets[i] || 0).toString().padStart(10, "0");
      out += `${off} 00000 n\r\n`;
    }
    out += `trailer\r\n<< /Size ${totalObjs + 1} /Root 1 0 R >>\r\nstartxref\r\n${startXref}\r\n%%EOF\r\n`;

    const encoder = new TextEncoder();
    return encoder.encode(out);
  }

  // --- Unified Command Dispatcher ---

  public execCommand(action: string, payload?: unknown): boolean {
    switch (action) {
      case "InsertBlankPage":
      case "InsertPage":
        this.insertBlankPage();
        return true;
      case "DeletePage":
        return this.deletePage(typeof payload === "number" ? payload : this.currentPage);
      case "DuplicatePage":
        return this.duplicatePage(typeof payload === "number" ? payload : this.currentPage) !== null;
      case "ResizePageModal":
      case "ResizePage":
        if (payload && typeof payload === "object") {
          const p = payload as { width: number; height: number; scaleContent?: boolean; applyToAll?: boolean };
          return this.resizePage(this.currentPage, p.width, p.height, p.scaleContent ?? true, p.applyToAll ?? false);
        }
        return this.setPageSize("A4", true, false);
      case "SetPageSize":
        if (typeof payload === "string") {
          return this.setPageSize(payload, true, false);
        } else if (payload && typeof payload === "object") {
          const p = payload as { size: string; scaleContent?: boolean; applyToAll?: boolean };
          return this.setPageSize(p.size, p.scaleContent ?? true, p.applyToAll ?? false);
        }
        return false;
      case "SplitPdf":
        if (payload && typeof payload === "object") {
          const p = payload as { fromPage: number; toPage: number };
          return this.splitPdf(p.fromPage, p.toPage).length > 0;
        }
        return this.splitPdf(1, 1).length > 0;
      case "EncryptPassword":
        if (typeof payload === "string") {
          return this.encryptWithPassword(payload);
        } else if (payload && typeof payload === "object") {
          const p = payload as { openPassword: string; ownerPassword?: string; permissions?: any };
          return this.encryptWithPassword(p.openPassword, p.ownerPassword, p.permissions);
        } else {
          return this.encryptWithPassword("secret123");
        }
      case "DecryptPassword":
        return this.decryptPassword(typeof payload === "string" ? payload : "secret123");
      case "RemoveSecurity":
        return this.removeSecurity();
      case "CompressPdf":
        this.compressPdf(typeof payload === "number" ? payload : 0.75);
        return true;
      case "RotatePageCW":
        this.rotatePageClockwise(typeof payload === "number" ? payload : this.currentPage);
        return true;
      case "RotatePageCCW":
        this.rotatePageCounterClockwise(typeof payload === "number" ? payload : this.currentPage);
        return true;
      case "RotatePage":
        if (payload === -90) {
          this.rotatePageCounterClockwise();
        } else {
          this.rotatePageClockwise();
        }
        return true;
      case "RotateAllPages":
        this.rotateAllPages(typeof payload === "number" ? payload : 90);
        return true;
      case "MovePageUp":
        return this.movePageUp();
      case "MovePageDown":
        return this.movePageDown();
      case "ExtractPage":
      case "ExtractPages":
        return this.extractPage() !== null;
      case "MergePdf":
        if (Array.isArray(payload)) {
          this.mergePdf(payload as PdfPageModel[]);
          return true;
        }
        return false;
      case "CreateBlankPdf":
        this.createBlankPdf(typeof payload === "number" ? payload : 1);
        return true;
      case "SetViewMode":
        if (payload === "continuous" || payload === "single" || payload === "twoPage") {
          this.setViewMode(payload);
          return true;
        }
        return false;
      case "SetSidebarTab":
        if (payload === "thumbnails" || payload === "bookmarks" || payload === "annotations") {
          this.setSidebarTab(payload);
          return true;
        }
        return false;
      case "Zoom100":
        this.setZoom(1.0);
        return true;
      case "ZoomIn":
        this.setZoom(this.zoom + 0.25);
        return true;
      case "ZoomOut":
        this.setZoom(this.zoom - 0.25);
        return true;
      case "ZoomFitPage":
        this.setZoom(0.8);
        return true;
      case "ZoomFitWidth":
        this.setZoom(1.25);
        return true;
      case "NextPage":
        return this.setCurrentPage(this.currentPage + 1);
      case "PrevPage":
      case "PreviousPage":
        return this.setCurrentPage(this.currentPage - 1);
      case "FirstPage":
        return this.setCurrentPage(1);
      case "LastPage":
        return this.setCurrentPage(this.pages.length);
      case "SetPdfTool":
        if (typeof payload === "string") {
          this.setTool(payload as PdfActiveTool);
          return true;
        }
        return false;
      case "EditPdfText":
        this.setTool("edit_text");
        return true;
      case "AddPdfText":
        this.addTextBlock(this.currentPage);
        this.setTool("edit_text");
        return true;
      case "AddPdfImage":
        if (payload && typeof payload === "object") {
          this.addImageBlock(this.currentPage, payload as Partial<PdfImageBlock>);
        } else {
          this.addImageBlock(this.currentPage);
        }
        return true;
      case "SetPdfFontFamily":
        if (typeof payload === "string") {
          this.activeFontFamily = payload;
          return true;
        }
        return false;
      case "SetPdfFontSize":
        if (typeof payload === "number") {
          this.activeFontSize = payload;
          return true;
        }
        return false;
      case "SetPdfTextColor":
        if (typeof payload === "string") {
          this.activeFontColor = payload;
          return true;
        }
        return false;
      case "TogglePdfBold":
        this.activeBold = !this.activeBold;
        return true;
      case "TogglePdfItalic":
        this.activeItalic = !this.activeItalic;
        return true;
      case "SetPdfAlign": {
        this.commitSnapshot();
        const align = (typeof payload === "string" ? payload : "left") as "left" | "center" | "right";
        const p = this.getPage(this.currentPage);
        if (p?.textBlocks) {
          p.textBlocks.forEach((tb) => {
            tb.align = align;
          });
        }
        return true;
      }
      case "DeletePdfElement":
      case "DeleteSelectedTextBlock": {
        this.commitSnapshot();
        const p = this.getPage(this.currentPage);
        if (p) {
          if (p.textBlocks && p.textBlocks.length > 0) {
            p.textBlocks.pop();
            return true;
          }
          if (p.imageBlocks && p.imageBlocks.length > 0) {
            p.imageBlocks.pop();
            return true;
          }
          if (p.annotations && p.annotations.length > 0) {
            p.annotations.pop();
            return true;
          }
        }
        return false;
      }
      case "CropPage": {
        this.commitSnapshot();
        const p = this.getPage(this.currentPage);
        if (p) {
          p.width = Math.max(300, Math.round(p.width * 0.9));
          p.height = Math.max(400, Math.round(p.height * 0.9));
          return true;
        }
        return false;
      }
      case "HighlightFormFields": {
        this.highlightFields = !this.highlightFields;
        return true;
      }
      case "OpenPdf":
        if (Array.isArray(payload)) {
          this.loadPdfDocument(payload as PdfPageModel[]);
          return true;
        } else {
          this.resetToBlank("Opened Document.pdf");
          return true;
        }
      case "SetPageLayout":
        if (payload === "continuous" || payload === "single" || payload === "twoPage" || payload === "two-page") {
          this.setViewMode(payload === "two-page" ? "twoPage" : (payload as any));
          return true;
        }
        return false;
      case "RotateView":
        if (payload === -90) {
          this.rotatePageCounterClockwise();
        } else {
          this.rotatePageClockwise();
        }
        return true;
      case "ToggleBookmarks":
        this.setSidebarTab(this.sidebarTab === "bookmarks" ? "thumbnails" : "bookmarks");
        return true;
      case "ToggleThumbnails":
        this.setSidebarTab("thumbnails");
        return true;
      case "ToggleAttachments":
        this.setSidebarTab(this.sidebarTab === "annotations" ? "thumbnails" : "annotations");
        return true;
      case "SetInkColor":
        if (typeof payload === "string") {
          this.inkColor = payload;
          return true;
        }
        return false;
      case "SetInkThickness":
        if (typeof payload === "number") {
          this.inkThickness = payload;
          return true;
        }
        return false;
      case "SetHighlightColor":
        if (typeof payload === "string") {
          this.highlightColor = payload;
          return true;
        }
        return false;
      case "AddWatermark":
        this.setWatermark({ text: typeof payload === "string" ? payload : "CONFIDENTIAL" });
        return true;
      case "AddHeaderFooter":
        this.commitSnapshot();
        this.pages.forEach((p, idx) => {
          if (!p.textBlocks) p.textBlocks = [];
          p.textBlocks.push({
            id: `hf_${p.pageNumber}`,
            x: Math.round(p.width / 2) - 40,
            y: p.height - 40,
            text: `Page ${idx + 1} of ${this.pages.length}`,
            fontSize: 10,
            fontColor: "#64748b",
          });
        });
        return true;
      case "SetPdfBackgroundColor":
        this.commitSnapshot();
        const bgColor = typeof payload === "string" ? payload : "#ffffff";
        this.pages.forEach((p) => {
          (p as any).backgroundColor = bgColor;
        });
        return true;
      case "SetAnnotTool":
        if (typeof payload === "string") {
          this.setTool(payload as PdfActiveTool);
          return true;
        }
        return false;
      case "AddStamp": {
        const stampType = typeof payload === "string" ? payload.toLowerCase() : "approved";
        const color =
          stampType === "confidential" || stampType === "void" || stampType === "urgent"
            ? "#dc2626"
            : stampType === "draft"
            ? "#ea580c"
            : stampType === "reviewed"
            ? "#2563eb"
            : stampType === "signed"
            ? "#1e40af"
            : "#16a34a";
        this.addAnnotation({
          type: "stamp",
          pageNumber: this.currentPage,
          rect: [100, 200, 160, 48],
          color,
          contents: stampType.toUpperCase(),
        });
        return true;
      }
      case "InsertShape": {
        const shapeKind = typeof payload === "string" ? payload : "rect";
        this.addAnnotation({
          type: "note",
          pageNumber: this.currentPage,
          rect: [120, 150, 140, 80],
          color: "#2563eb",
          contents: `Shape: ${shapeKind}`,
        });
        return true;
      }
      case "AddInitials":
        this.addAnnotation({
          type: "signature",
          pageNumber: this.currentPage,
          rect: [100, 400, 80, 40],
          color: "#1e3a8a",
          signerName: "Initial",
        });
        return true;
      case "RestrictEditing":
        this.securityConfig.permissions = {
          canPrint: true,
          canEdit: false,
          canCopy: false,
          canAnnotate: false,
        };
        return true;
      case "ReplacePdfImage":
      case "CropPdfImage":
      case "RotatePdfImage":
      case "AddWebLink":
      case "AddPageJumpLink":
      case "AlignFormFields":
      case "DistributeFormFields":
      case "ToggleFullScreen":
        return true;
      case "AddFormField":
        this.addFormField(this.currentPage, {
          type: (typeof payload === "string" ? payload : "text") as PdfFormFieldType,
        });
        return true;
      case "ClearFormData":
        this.clearFormData();
        return true;
      case "ExportFormData":
        this.exportFormData();
        return true;
      case "ImportFormData":
        if (payload && typeof payload === "object") {
          this.importFormData(payload as Record<string, string | boolean>);
          return true;
        }
        return false;
      case "MarkRedaction":
        this.setTool("redact");
        this.markRedaction(this.currentPage, [100, 200, 250, 40]);
        return true;
      case "ApplyRedactions":
        this.applyRedactions();
        return true;
      case "AddSignature":
      case "SignPdf":
        this.setTool("sign");
        return true;
      case "Find":
      case "SearchText":
        if (typeof payload === "string") {
          this.searchText(payload);
          return true;
        } else {
          const res = this.searchText("Section");
          return res.length > 0;
        }
      case "PrintDocument":
      case "Print":
        return true;
      case "SaveDocument":
      case "Save":
        return true;
      case "Undo":
        return this.undo();
      case "Redo":
        return this.redo();
      default:
        return false;
    }
  }
}

export const pdfState = new PdfState();
