export interface PdfFormField {
  id: string;
  name: string;
  type: "text" | "checkbox";
  value: string | boolean;
  rect: [number, number, number, number];
}

export interface PdfAnnotation {
  id: string;
  type: "highlight" | "underline" | "strike" | "note" | "stamp" | "ink";
  pageNumber: number;
  rect: [number, number, number, number];
  color: string;
  author?: string;
  date?: string;
  contents?: string;
}

export interface PdfBookmark {
  id: string;
  title: string;
  pageNumber: number;
  level: number;
}

export interface PdfPageModel {
  id: string;
  pageNumber: number; // 1-indexed
  width: number; // default 816
  height: number; // default 1056
  rotation: number; // 0, 90, 180, 270
  title?: string;
  textBlocks?: { x: number; y: number; text: string; fontSize?: number; fontColor?: string; bold?: boolean }[];
  formFields?: PdfFormField[];
  annotations?: PdfAnnotation[];
}

export type PdfViewMode = "continuous" | "single" | "twoPage";
export type PdfSidebarTab = "thumbnails" | "bookmarks" | "annotations";
export type PdfActiveTool =
  | "hand"
  | "select"
  | "snapshot"
  | "edit_text"
  | "highlight"
  | "form_fill"
  | "sign"
  | "pencil"
  | "note";

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
        { x: 80, y: 80, text: "ISO 32000-2 Certified PDF Document", fontSize: 20, fontColor: "#b30b00", bold: true },
        { x: 80, y: 105, text: "Interactive Form & Digital Signing Specification", fontSize: 12, fontColor: "#555555" },
        { x: 80, y: 170, text: "Fillable Form Fields (ISO 32000 AcroForms):", fontSize: 13, fontColor: "#222222", bold: true },
      ],
      formFields: [
        { id: "f1", name: "Full_Name", type: "text", value: "John Doe", rect: [100, 200, 250, 30] },
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
        { x: 80, y: 80, text: "Section 2: Document Security & Signatures", fontSize: 20, fontColor: "#1e293b", bold: true },
        { x: 80, y: 110, text: "Cryptographic X.509 PKCS#7 detached signature verification.", fontSize: 13, fontColor: "#475569" },
        { x: 80, y: 150, text: "All modified content streams are recorded in incremental updates.", fontSize: 13, fontColor: "#475569" },
      ],
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
        { x: 80, y: 80, text: "Section 3: Vector Graphics & Color Spaces", fontSize: 20, fontColor: "#1e293b", bold: true },
        { x: 80, y: 110, text: "DeviceRGB, DeviceCMYK, and ICCBased profile compliance.", fontSize: 13, fontColor: "#475569" },
      ],
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

  private undoStack: string[] = [];
  private redoStack: string[] = [];
  private nextPageCounter = 10;
  private nextBookmarkCounter = 10;
  private nextAnnotCounter = 10;

  public get pageCount(): number {
    return this.pages.length;
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
      // Prevent deleting last remaining page in document
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
        formFields: [],
        annotations: [],
      });
    }
    this.bookmarks = [
      { id: "bm-1", title: "Page 1", pageNumber: 1, level: 0 },
    ];
    this.currentPage = 1;
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

  // --- Annotations Management ---

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

  public getAllAnnotations(): PdfAnnotation[] {
    const list: PdfAnnotation[] = [];
    this.pages.forEach((p) => {
      if (p.annotations) {
        list.push(...p.annotations);
      }
    });
    return list;
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
      case "SetPdfTool":
        if (typeof payload === "string") {
          this.setTool(payload as PdfActiveTool);
          return true;
        }
        return false;
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
