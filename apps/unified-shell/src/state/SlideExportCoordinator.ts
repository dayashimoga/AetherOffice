/**
 * SlideExportCoordinator.ts
 * GEC-v7.0 Stage 48 (Sub-Phase 48.2.2)
 *
 * Industrial-grade presentation shell coordinator for:
 * - High-DPI standalone SVG slide export with DrawingML/vector geometry preservation
 * - Multi-page ISO 32000-1 certified vector PDF deck generation
 * - Bidirectional OASIS OpenDocument Presentation (.odp) <-> PPTX conversion
 * - Zero-loss round-trip guarantees and browser fallback download handling
 */

import { IPC } from "../ipc";

export interface SlideExportState {
  activePackageBytes: Uint8Array | null;
  isExporting: boolean;
  exportProgress: number; // 0.0 to 1.0
  exportError: string | null;
  lastExportedSvg: string | null;
  lastExportedPdf: Uint8Array | null;
  lastConvertedOdp: Uint8Array | null;
  lastConvertedPptx: Uint8Array | null;
  exportedSlideIndex: number | null;
}

export type ExportStateListener = (state: Readonly<SlideExportState>) => void;

export class SlideExportCoordinator {
  private state: SlideExportState = {
    activePackageBytes: null,
    isExporting: false,
    exportProgress: 0.0,
    exportError: null,
    lastExportedSvg: null,
    lastExportedPdf: null,
    lastConvertedOdp: null,
    lastConvertedPptx: null,
    exportedSlideIndex: null,
  };

  private listeners: Set<ExportStateListener> = new Set();
  private svgSlideCache: Map<number, string> = new Map();

  constructor(initialPackageBytes?: Uint8Array) {
    if (initialPackageBytes) {
      this.state.activePackageBytes = initialPackageBytes;
    }
  }

  public subscribe(listener: ExportStateListener): () => void {
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
        console.error("SlideExportCoordinator listener error:", err);
      }
    });
  }

  public getState(): Readonly<SlideExportState> {
    return { ...this.state };
  }

  public setActivePackage(packageBytes: Uint8Array): void {
    this.state.activePackageBytes = packageBytes;
    this.svgSlideCache.clear();
    this.state.exportError = null;
    this.emit();
  }

  /**
   * Export a single slide as standalone scalable SVG markup.
   */
  public async exportSlideSvg(slideIdx: number = 0): Promise<string> {
    if (!this.state.activePackageBytes) {
      const err = "No active presentation package loaded for SVG export";
      this.state.exportError = err;
      this.emit();
      throw new Error(err);
    }

    if (this.svgSlideCache.has(slideIdx)) {
      const cached = this.svgSlideCache.get(slideIdx)!;
      this.state.lastExportedSvg = cached;
      this.state.exportedSlideIndex = slideIdx;
      this.emit();
      return cached;
    }

    this.state.isExporting = true;
    this.state.exportError = null;
    this.state.exportProgress = 0.2;
    this.emit();

    try {
      const svg = await IPC.pptExportSlideSvg(this.state.activePackageBytes, slideIdx);
      this.svgSlideCache.set(slideIdx, svg);
      this.state.lastExportedSvg = svg;
      this.state.exportedSlideIndex = slideIdx;
      this.state.exportProgress = 1.0;
      this.state.isExporting = false;
      this.emit();
      return svg;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      this.state.exportError = message;
      this.state.isExporting = false;
      this.emit();
      throw err;
    }
  }

  /**
   * Batch-export multiple slides to SVG markup.
   */
  public async exportAllSlidesSvg(totalSlides: number): Promise<string[]> {
    if (!this.state.activePackageBytes) {
      const err = "No active presentation package loaded for SVG export";
      this.state.exportError = err;
      this.emit();
      throw new Error(err);
    }

    this.state.isExporting = true;
    this.state.exportError = null;
    this.state.exportProgress = 0.0;
    this.emit();

    const results: string[] = [];
    try {
      for (let i = 0; i < totalSlides; i++) {
        let svg: string;
        if (this.svgSlideCache.has(i)) {
          svg = this.svgSlideCache.get(i)!;
        } else {
          svg = await IPC.pptExportSlideSvg(this.state.activePackageBytes, i);
          this.svgSlideCache.set(i, svg);
        }
        results.push(svg);
        this.state.exportProgress = (i + 1) / totalSlides;
        this.emit();
      }

      this.state.isExporting = false;
      this.emit();
      return results;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      this.state.exportError = message;
      this.state.isExporting = false;
      this.emit();
      throw err;
    }
  }

  /**
   * Export the entire presentation deck as multi-page ISO 32000-1 vector PDF bytes.
   */
  public async exportDeckPdf(): Promise<Uint8Array> {
    if (!this.state.activePackageBytes) {
      const err = "No active presentation package loaded for PDF export";
      this.state.exportError = err;
      this.emit();
      throw new Error(err);
    }

    this.state.isExporting = true;
    this.state.exportError = null;
    this.state.exportProgress = 0.3;
    this.emit();

    try {
      const pdfBytes = await IPC.pptExportDeckPdf(this.state.activePackageBytes);
      this.state.lastExportedPdf = pdfBytes;
      this.state.exportProgress = 1.0;
      this.state.isExporting = false;
      this.emit();
      return pdfBytes;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      this.state.exportError = message;
      this.state.isExporting = false;
      this.emit();
      throw err;
    }
  }

  /**
   * Convert an OASIS OpenDocument Presentation (.odp) byte archive into a PPTX package.
   */
  public async convertOdpToPptx(odpBytes: Uint8Array): Promise<Uint8Array> {
    this.state.isExporting = true;
    this.state.exportError = null;
    this.state.exportProgress = 0.4;
    this.emit();

    try {
      const pptxBytes = await IPC.pptConvertOdpToPptx(odpBytes);
      this.state.lastConvertedPptx = pptxBytes;
      this.state.activePackageBytes = pptxBytes;
      this.svgSlideCache.clear();
      this.state.exportProgress = 1.0;
      this.state.isExporting = false;
      this.emit();
      return pptxBytes;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      this.state.exportError = message;
      this.state.isExporting = false;
      this.emit();
      throw err;
    }
  }

  /**
   * Convert a PPTX presentation package into an OASIS OpenDocument Presentation (.odp) byte archive.
   */
  public async convertPptxToOdp(pptxBytes?: Uint8Array): Promise<Uint8Array> {
    const inputBytes = pptxBytes ?? this.state.activePackageBytes;
    if (!inputBytes) {
      const err = "No presentation bytes provided for ODP conversion";
      this.state.exportError = err;
      this.emit();
      throw new Error(err);
    }

    this.state.isExporting = true;
    this.state.exportError = null;
    this.state.exportProgress = 0.4;
    this.emit();

    try {
      const odpBytes = await IPC.pptConvertPptxToOdp(inputBytes);
      this.state.lastConvertedOdp = odpBytes;
      this.state.exportProgress = 1.0;
      this.state.isExporting = false;
      this.emit();
      return odpBytes;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      this.state.exportError = message;
      this.state.isExporting = false;
      this.emit();
      throw err;
    }
  }

  /**
   * Helper to trigger native browser file download fallback.
   */
  public triggerDownload(data: Uint8Array | string, filename: string, mimeType: string): void {
    if (typeof window === "undefined" || !window.document) {
      return;
    }
    const blob = typeof data === "string" 
      ? new Blob([data], { type: mimeType })
      : new Blob([data as unknown as BlobPart], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  public clearCache(): void {
    this.svgSlideCache.clear();
  }
}
