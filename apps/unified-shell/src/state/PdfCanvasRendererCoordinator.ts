/**
 * PdfCanvasRendererCoordinator.ts
 * GEC-v7.0 Stage 49 (Sub-Phase 49.1.1)
 *
 * Industrial-grade presentation shell coordinator for:
 * - ISO 32000-2 vector path, image XObject & font CMap canvas rendering
 * - High-DPI canvas synchronization with WebGPU / Canvas2D viewports
 * - In-memory page canvas, SVG, and image extraction caching
 * - Interactive zooming (DPI scaling), page navigation, and reactive state distribution
 */

import {
  IPC,
  type RenderedPageCanvasDto,
  type RenderedImageElementDto,
} from "../ipc";

export interface PdfCanvasState {
  activeBytes: Uint8Array | null;
  activePageIndex: number;
  totalPages: number;
  scale: number;
  currentCanvas: RenderedPageCanvasDto | null;
  isLoading: boolean;
  error: string | null;
  cachedPageCount: number;
}

export type PdfCanvasListener = (state: Readonly<PdfCanvasState>) => void;

export class PdfCanvasRendererCoordinator {
  private state: PdfCanvasState = {
    activeBytes: null,
    activePageIndex: 0,
    totalPages: 0,
    scale: 1.0,
    currentCanvas: null,
    isLoading: false,
    error: null,
    cachedPageCount: 0,
  };

  private listeners: Set<PdfCanvasListener> = new Set();
  private canvasCache: Map<string, RenderedPageCanvasDto> = new Map();
  private svgCache: Map<number, string> = new Map();
  private imageCache: Map<number, RenderedImageElementDto[]> = new Map();

  constructor(initialBytes?: Uint8Array) {
    if (initialBytes) {
      this.state.activeBytes = initialBytes;
    }
  }

  public subscribe(listener: PdfCanvasListener): () => void {
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
        console.error("PdfCanvasRendererCoordinator listener error:", err);
      }
    });
  }

  public getState(): Readonly<PdfCanvasState> {
    return { ...this.state };
  }

  /**
   * Loads a new PDF document byte buffer and resets all page caches.
   */
  public async loadDocument(bytes: Uint8Array, initialPage: number = 0): Promise<RenderedPageCanvasDto> {
    this.state.activeBytes = bytes;
    this.state.activePageIndex = initialPage;
    this.state.error = null;
    this.state.isLoading = true;
    this.clearCache();
    this.emit();

    try {
      const canvas = await this.renderPage(initialPage, this.state.scale);
      this.state.isLoading = false;
      this.emit();
      return canvas;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      this.state.error = msg;
      this.state.isLoading = false;
      this.emit();
      throw err;
    }
  }

  /**
   * Renders the specified page canvas at the given or current zoom scale.
   */
  public async renderPage(pageIndex: number, scale?: number): Promise<RenderedPageCanvasDto> {
    if (!this.state.activeBytes) {
      const err = "No active PDF document loaded";
      this.state.error = err;
      this.emit();
      throw new Error(err);
    }

    const effectiveScale = scale ?? this.state.scale;
    const cacheKey = `${pageIndex}@${effectiveScale.toFixed(2)}`;

    if (this.canvasCache.has(cacheKey)) {
      const cached = this.canvasCache.get(cacheKey)!;
      this.state.currentCanvas = cached;
      this.state.activePageIndex = pageIndex;
      this.state.scale = effectiveScale;
      this.emit();
      return cached;
    }

    this.state.isLoading = true;
    this.state.error = null;
    this.emit();

    try {
      const canvas = await IPC.pdfRenderPageCanvas(this.state.activeBytes, pageIndex, effectiveScale);
      this.canvasCache.set(cacheKey, canvas);
      this.svgCache.set(pageIndex, canvas.svg_markup);
      this.state.currentCanvas = canvas;
      this.state.activePageIndex = pageIndex;
      this.state.scale = effectiveScale;
      this.state.cachedPageCount = this.canvasCache.size;
      this.state.isLoading = false;
      this.emit();
      return canvas;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      this.state.error = msg;
      this.state.isLoading = false;
      this.emit();
      throw err;
    }
  }

  /**
   * Directly retrieves or renders standalone SVG markup for a page.
   */
  public async renderPageSvg(pageIndex: number): Promise<string> {
    if (this.svgCache.has(pageIndex)) {
      return this.svgCache.get(pageIndex)!;
    }

    if (!this.state.activeBytes) {
      const err = "No active PDF document loaded";
      this.state.error = err;
      this.emit();
      throw new Error(err);
    }

    try {
      const svg = await IPC.pdfRenderPageSvg(this.state.activeBytes, pageIndex);
      this.svgCache.set(pageIndex, svg);
      return svg;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      this.state.error = msg;
      this.emit();
      throw err;
    }
  }

  /**
   * Extracts all image XObjects embedded on the specified page.
   */
  public async extractPageImages(pageIndex: number): Promise<RenderedImageElementDto[]> {
    if (this.imageCache.has(pageIndex)) {
      return this.imageCache.get(pageIndex)!;
    }

    if (!this.state.activeBytes) {
      const err = "No active PDF document loaded";
      this.state.error = err;
      this.emit();
      throw new Error(err);
    }

    try {
      const images = await IPC.pdfExtractPageImages(this.state.activeBytes, pageIndex);
      this.imageCache.set(pageIndex, images);
      return images;
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      this.state.error = msg;
      this.emit();
      throw err;
    }
  }

  /**
   * Sets canvas zoom scale factor and re-renders the current page.
   */
  public async setScale(newScale: number): Promise<RenderedPageCanvasDto | null> {
    const clampedScale = Math.max(0.1, Math.min(10.0, newScale));
    this.state.scale = clampedScale;
    if (this.state.activeBytes) {
      return this.renderPage(this.state.activePageIndex, clampedScale);
    }
    this.emit();
    return null;
  }

  /**
   * Navigates to a specific page index.
   */
  public async gotoPage(pageIndex: number): Promise<RenderedPageCanvasDto> {
    return this.renderPage(pageIndex, this.state.scale);
  }

  /**
   * Navigates to the next page.
   */
  public async nextPage(): Promise<RenderedPageCanvasDto> {
    return this.gotoPage(this.state.activePageIndex + 1);
  }

  /**
   * Navigates to the previous page.
   */
  public async previousPage(): Promise<RenderedPageCanvasDto> {
    const prev = Math.max(0, this.state.activePageIndex - 1);
    return this.gotoPage(prev);
  }

  /**
   * Clears all in-memory render caches.
   */
  public clearCache(): void {
    this.canvasCache.clear();
    this.svgCache.clear();
    this.imageCache.clear();
    this.state.cachedPageCount = 0;
  }
}
