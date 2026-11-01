/**
 * PdfPageOpsCoordinator.ts
 * GEC-v7.0 Stage 49 (Sub-Phase 49.1.2)
 *
 * Industrial-grade presentation shell coordinator for:
 * - PDF Page Split, Merge, Extract operations with clean object-tree rebasing
 * - Non-destructive Bates numbering overlays (legal/regulatory compliance)
 * - Invisible OCR text layer injection (3 Tr) with bounding box preservation
 * - Reactive state synchronization with PDF Studio sidebars and workspace tabs
 */

import {
  IPC,
  type BatesNumberingConfig,
  type BatesPosition,
  type OcrBoundingBox,
  type OcrWord,
  type OcrPageData,
} from "../ipc";

export {
  type BatesNumberingConfig,
  type BatesPosition,
  type OcrBoundingBox,
  type OcrWord,
  type OcrPageData,
};

export interface PdfPageOpsState {
  activeBytes: Uint8Array | null;
  mergeQueue: Uint8Array[];
  selectedPageIndices: number[];
  splitBoundaries: number[];
  batesConfig: BatesNumberingConfig;
  ocrPages: Record<number, OcrPageData>;
  isLoading: boolean;
  error: string | null;
  lastResultBytes: Uint8Array | null;
  lastSplitResults: Uint8Array[] | null;
}

export type PdfPageOpsListener = (state: Readonly<PdfPageOpsState>) => void;

export class PdfPageOpsCoordinator {
  private state: PdfPageOpsState = {
    activeBytes: null,
    mergeQueue: [],
    selectedPageIndices: [],
    splitBoundaries: [],
    batesConfig: {
      prefix: "CONFIDENTIAL-",
      suffix: "",
      start_number: 1,
      digits: 6,
      position: "BottomRight",
      font_size_pt: 10.0,
      margin_x_pt: 36.0,
      margin_y_pt: 20.0,
    },
    ocrPages: {},
    isLoading: false,
    error: null,
    lastResultBytes: null,
    lastSplitResults: null,
  };

  private listeners: Set<PdfPageOpsListener> = new Set();

  constructor(initialBytes?: Uint8Array) {
    if (initialBytes) {
      this.state.activeBytes = initialBytes;
    }
  }

  public subscribe(listener: PdfPageOpsListener): () => void {
    this.listeners.add(listener);
    listener(this.getState());
    return () => {
      this.listeners.delete(listener);
    };
  }

  private emit(): void {
    const currentState = this.getState();
    this.listeners.forEach((listener) => {
      try {
        listener(currentState);
      } catch (err) {
        console.error("PdfPageOpsCoordinator listener error:", err);
      }
    });
  }

  public getState(): Readonly<PdfPageOpsState> {
    return {
      ...this.state,
      mergeQueue: [...this.state.mergeQueue],
      selectedPageIndices: [...this.state.selectedPageIndices],
      splitBoundaries: [...this.state.splitBoundaries],
      batesConfig: { ...this.state.batesConfig },
      ocrPages: { ...this.state.ocrPages },
      lastSplitResults: this.state.lastSplitResults ? [...this.state.lastSplitResults] : null,
    };
  }

  public setActiveBytes(bytes: Uint8Array | null): void {
    this.state.activeBytes = bytes;
    this.state.error = null;
    this.emit();
  }

  public setSelectedPageIndices(indices: number[]): void {
    this.state.selectedPageIndices = [...indices];
    this.emit();
  }

  public setSplitBoundaries(boundaries: number[]): void {
    this.state.splitBoundaries = [...boundaries];
    this.emit();
  }

  public setBatesConfig(config: Partial<BatesNumberingConfig>): void {
    this.state.batesConfig = {
      ...this.state.batesConfig,
      ...config,
    };
    this.emit();
  }

  public queueForMerge(bytes: Uint8Array): void {
    this.state.mergeQueue.push(bytes);
    this.emit();
  }

  public clearMergeQueue(): void {
    this.state.mergeQueue = [];
    this.emit();
  }

  public setOcrData(pageIndex: number, ocrData: OcrPageData): void {
    this.state.ocrPages[pageIndex] = ocrData;
    this.emit();
  }

  /**
   * Extracts specified pages into a new standalone PDF document.
   */
  public async extractPages(pageIndices?: number[]): Promise<Uint8Array> {
    const bytes = this.state.activeBytes;
    if (!bytes) {
      throw new Error("No active PDF document loaded to extract pages from");
    }
    const targetIndices = pageIndices ?? this.state.selectedPageIndices;
    if (targetIndices.length === 0) {
      throw new Error("No pages specified for extraction");
    }

    this.state.isLoading = true;
    this.state.error = null;
    this.emit();

    try {
      const result = await IPC.pdfExtractPages(bytes, targetIndices);
      this.state.lastResultBytes = result;
      this.state.isLoading = false;
      this.emit();
      return result;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      this.state.error = msg;
      this.state.isLoading = false;
      this.emit();
      throw err;
    }
  }

  /**
   * Splits active document into multiple PDF documents at specified page boundaries.
   */
  public async splitDocument(splitBoundaries?: number[]): Promise<Uint8Array[]> {
    const bytes = this.state.activeBytes;
    if (!bytes) {
      throw new Error("No active PDF document loaded to split");
    }
    const boundaries = splitBoundaries ?? this.state.splitBoundaries;

    this.state.isLoading = true;
    this.state.error = null;
    this.emit();

    try {
      const results = await IPC.pdfSplitDocument(bytes, boundaries);
      this.state.lastSplitResults = results;
      this.state.isLoading = false;
      this.emit();
      return results;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      this.state.error = msg;
      this.state.isLoading = false;
      this.emit();
      throw err;
    }
  }

  /**
   * Merges multiple PDF documents (either provided or from mergeQueue) into a single document.
   */
  public async mergeDocuments(documents?: Uint8Array[]): Promise<Uint8Array> {
    const docs = documents ?? (this.state.activeBytes ? [this.state.activeBytes, ...this.state.mergeQueue] : [...this.state.mergeQueue]);
    if (docs.length === 0) {
      throw new Error("No documents available to merge");
    }

    this.state.isLoading = true;
    this.state.error = null;
    this.emit();

    try {
      const result = await IPC.pdfMergeDocuments(docs);
      this.state.lastResultBytes = result;
      this.state.isLoading = false;
      this.emit();
      return result;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      this.state.error = msg;
      this.state.isLoading = false;
      this.emit();
      throw err;
    }
  }

  /**
   * Applies sequential Bates numbering overlay to the active document.
   */
  public async applyBatesNumbering(customConfig?: Partial<BatesNumberingConfig>): Promise<Uint8Array> {
    const bytes = this.state.activeBytes;
    if (!bytes) {
      throw new Error("No active PDF document loaded for Bates numbering");
    }
    if (customConfig) {
      this.setBatesConfig(customConfig);
    }

    this.state.isLoading = true;
    this.state.error = null;
    this.emit();

    try {
      const result = await IPC.pdfApplyBatesNumbering(bytes, this.state.batesConfig);
      this.state.lastResultBytes = result;
      this.state.isLoading = false;
      this.emit();
      return result;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      this.state.error = msg;
      this.state.isLoading = false;
      this.emit();
      throw err;
    }
  }

  /**
   * Injects an invisible OCR text layer (3 Tr) over the raster image on the specified page.
   */
  public async injectOcrLayer(pageIndex: number, ocrData?: OcrPageData): Promise<Uint8Array> {
    const bytes = this.state.activeBytes;
    if (!bytes) {
      throw new Error("No active PDF document loaded for OCR layer injection");
    }
    const data = ocrData ?? this.state.ocrPages[pageIndex];
    if (!data) {
      throw new Error(`No OCR word data found for page index ${pageIndex}`);
    }

    this.state.isLoading = true;
    this.state.error = null;
    this.emit();

    try {
      const result = await IPC.pdfInjectOcrLayer(bytes, pageIndex, data);
      this.state.lastResultBytes = result;
      this.state.isLoading = false;
      this.emit();
      return result;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      this.state.error = msg;
      this.state.isLoading = false;
      this.emit();
      throw err;
    }
  }
}
