/**
 * PdfDiffCoordinator.ts
 * GEC-v7.0 Stage 49 (Sub-Phase 49.2.1)
 *
 * Industrial-grade presentation shell coordinator for:
 * - Side-by-side visual and textual PDF diffing (Document A vs. Document B)
 * - Synchronized split-pane viewport scrolling
 * - Bounding-box visual overlay generation (Insertions, Deletions, Modifications)
 * - Difference jumping (Next/Prev diff navigation) and category filtering
 * - Reactive state synchronization with dual-pane PDF Studio split views
 */

import {
  IPC,
  type DiffBoundingBox,
  type DiffItemDto,
  type DiffKind,
  type PageDiffDto,
  type PdfDiffOptions,
  type PdfDiffResultDto,
  type PdfDiffSummaryDto,
} from "../ipc";

export {
  type DiffBoundingBox,
  type DiffItemDto,
  type DiffKind,
  type PageDiffDto,
  type PdfDiffOptions,
  type PdfDiffResultDto,
  type PdfDiffSummaryDto,
};

export interface PdfDiffState {
  docABytes: Uint8Array | null;
  docBBytes: Uint8Array | null;
  options: PdfDiffOptions;
  diffResult: PdfDiffResultDto | null;
  activePageIndex: number;
  selectedDiffId: string | null;
  synchronizedScroll: boolean;
  scrollOffset: { x: number; y: number };
  filterKind: DiffKind | "All";
  isLoading: boolean;
  error: string | null;
  overlaySvgA: string | null;
  overlaySvgB: string | null;
}

export type PdfDiffListener = (state: Readonly<PdfDiffState>) => void;

export class PdfDiffCoordinator {
  private state: PdfDiffState = {
    docABytes: null,
    docBBytes: null,
    options: {
      ignore_case: false,
      ignore_whitespace: true,
      detect_moves: false,
    },
    diffResult: null,
    activePageIndex: 0,
    selectedDiffId: null,
    synchronizedScroll: true,
    scrollOffset: { x: 0, y: 0 },
    filterKind: "All",
    isLoading: false,
    error: null,
    overlaySvgA: null,
    overlaySvgB: null,
  };

  private listeners: Set<PdfDiffListener> = new Set();

  constructor(docA?: Uint8Array, docB?: Uint8Array, options?: PdfDiffOptions) {
    if (docA) this.state.docABytes = docA;
    if (docB) this.state.docBBytes = docB;
    if (options) this.state.options = { ...this.state.options, ...options };
  }

  public subscribe(listener: PdfDiffListener): () => void {
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
        console.error("PdfDiffCoordinator listener error:", err);
      }
    });
  }

  public getState(): Readonly<PdfDiffState> {
    return {
      ...this.state,
      options: { ...this.state.options },
      scrollOffset: { ...this.state.scrollOffset },
      diffResult: this.state.diffResult ? { ...this.state.diffResult } : null,
    };
  }

  public setOptions(options: Partial<PdfDiffOptions>): void {
    this.state.options = { ...this.state.options, ...options };
    this.emit();
  }

  public setFilterKind(filter: DiffKind | "All"): void {
    this.state.filterKind = filter;
    this.emit();
  }

  public setSynchronizedScroll(sync: boolean): void {
    this.state.synchronizedScroll = sync;
    this.emit();
  }

  public setScrollOffset(x: number, y: number): void {
    this.state.scrollOffset = { x, y };
    this.emit();
  }

  /**
   * Loads both documents and executes the diff comparison.
   */
  public async loadDocuments(
    docA: Uint8Array,
    docB: Uint8Array,
    options?: PdfDiffOptions
  ): Promise<PdfDiffResultDto> {
    this.state.docABytes = docA;
    this.state.docBBytes = docB;
    if (options) {
      this.state.options = { ...this.state.options, ...options };
    }
    return this.compare();
  }

  /**
   * Compares the active documents and populates diff results and SVG overlays.
   */
  public async compare(options?: PdfDiffOptions): Promise<PdfDiffResultDto> {
    const { docABytes, docBBytes } = this.state;
    if (!docABytes || !docBBytes) {
      const err = new Error("Both Document A and Document B must be loaded to run diff");
      this.state.error = err.message;
      this.state.isLoading = false;
      this.emit();
      throw err;
    }

    const effectiveOptions = options ? { ...this.state.options, ...options } : this.state.options;
    this.state.options = effectiveOptions;
    this.state.isLoading = true;
    this.state.error = null;
    this.emit();

    try {
      const result = await IPC.pdfCompareDocuments(docABytes, docBBytes, effectiveOptions);
      this.state.diffResult = result;
      this.state.activePageIndex = 0;
      this.state.selectedDiffId = result.pages[0]?.items[0]?.id ?? null;

      await this.refreshOverlays(0);

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
   * Sets the active page and updates the side-by-side SVG overlays.
   */
  public async setActivePage(pageIndex: number): Promise<void> {
    if (this.state.activePageIndex === pageIndex) return;
    this.state.activePageIndex = pageIndex;
    await this.refreshOverlays(pageIndex);
    this.emit();
  }

  /**
   * Refreshes SVG overlays for Document A and Document B for the given page.
   */
  private async refreshOverlays(pageIndex: number): Promise<void> {
    const { docABytes, docBBytes, diffResult } = this.state;
    if (!diffResult) return;

    const pageDiff = diffResult.pages[pageIndex];
    const items = pageDiff ? pageDiff.items : [];

    try {
      if (docABytes) {
        this.state.overlaySvgA = await IPC.pdfGenerateDiffOverlaySvg(
          docABytes,
          pageIndex,
          items,
          false
        );
      }
      if (docBBytes) {
        this.state.overlaySvgB = await IPC.pdfGenerateDiffOverlaySvg(
          docBBytes,
          pageIndex,
          items,
          true
        );
      }
    } catch (err) {
      console.warn("Could not generate diff overlay SVG for page", pageIndex, err);
    }
  }

  /**
   * Retrieves all diff items across all pages matching the active filter.
   */
  public getAllFilteredDiffs(): DiffItemDto[] {
    if (!this.state.diffResult) return [];
    const filter = this.state.filterKind;
    const allItems: DiffItemDto[] = [];

    for (const page of this.state.diffResult.pages) {
      for (const item of page.items) {
        if (filter === "All" || item.kind === filter) {
          allItems.push(item);
        }
      }
    }

    return allItems;
  }

  /**
   * Retrieves diff items on the active page matching the active filter.
   */
  public getFilteredDiffsForActivePage(): DiffItemDto[] {
    if (!this.state.diffResult) return [];
    const pageDiff = this.state.diffResult.pages[this.state.activePageIndex];
    if (!pageDiff) return [];

    const filter = this.state.filterKind;
    if (filter === "All") {
      return [...pageDiff.items];
    }
    return pageDiff.items.filter((item) => item.kind === filter);
  }

  /**
   * Jumps to the next difference in the document.
   */
  public nextDiff(): DiffItemDto | null {
    const allDiffs = this.getAllFilteredDiffs();
    if (allDiffs.length === 0) return null;

    const currentId = this.state.selectedDiffId;
    let nextIdx = 0;

    if (currentId) {
      const idx = allDiffs.findIndex((d) => d.id === currentId);
      if (idx >= 0 && idx + 1 < allDiffs.length) {
        nextIdx = idx + 1;
      }
    }

    const nextItem = allDiffs[nextIdx];
    this.selectDiffItem(nextItem);
    return nextItem;
  }

  /**
   * Jumps to the previous difference in the document.
   */
  public previousDiff(): DiffItemDto | null {
    const allDiffs = this.getAllFilteredDiffs();
    if (allDiffs.length === 0) return null;

    const currentId = this.state.selectedDiffId;
    let prevIdx = allDiffs.length - 1;

    if (currentId) {
      const idx = allDiffs.findIndex((d) => d.id === currentId);
      if (idx > 0) {
        prevIdx = idx - 1;
      }
    }

    const prevItem = allDiffs[prevIdx];
    this.selectDiffItem(prevItem);
    return prevItem;
  }

  /**
   * Selects a diff item and syncs the active page index.
   */
  public selectDiffItem(item: DiffItemDto): void {
    this.state.selectedDiffId = item.id;
    const targetPage = item.page_index_b ?? item.page_index_a ?? 0;
    if (targetPage !== this.state.activePageIndex) {
      this.setActivePage(targetPage);
    } else {
      this.emit();
    }
  }
}
