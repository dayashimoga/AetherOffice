/**
 * PaneController.ts
 * GEC-v7.0 Stage 42 (Sub-Phase 42.1.2)
 *
 * Modular document viewport controller responsible for:
 * - Viewport canvas lifecycle (mounting, rendering, unmounting, memory eviction)
 * - Format-specific canvas management (WriterCanvas, CalcGridCanvas, SlideStageCanvas, PdfWorkspace, NotesWorkspace)
 * - Teardown and zero-leak event cleanup
 */

import { TabItem } from "../components/TabBar";
import { DocumentType } from "../ipc";
import { WriterCanvas } from "../components/viewports/WriterCanvas";
import { CalcGridCanvas } from "../components/viewports/CalcGridCanvas";
import { SlideStageCanvas } from "../components/viewports/SlideStageCanvas";
import { PdfViewerCanvas } from "../components/viewports/PdfViewerCanvas";
import { SheetTabBar } from "../components/SheetTabBar";
import { CalcChartOverlay } from "../components/CalcChartOverlay";
import { SpeakerNotesPane } from "../components/SpeakerNotesPane";
import { AnimationPane } from "../components/AnimationPane";
import { PresentationPlayer } from "../components/PresentationPlayer";
import { SlideMediaOverlay } from "../components/SlideMediaOverlay";
import { PdfWorkspace } from "../apps/pdf/PdfWorkspace";
import { NotesWorkspace } from "../apps/notes/NotesWorkspace";
import { writerState } from "../state/writerState";
import { calcState } from "../state/calcState";
import { slidesState } from "../state/slidesState";
import { pdfState } from "../state/pdfState";
import { notesState } from "../state/notesState";

export interface PaneControllerCallbacks {
  onStatusChange?: () => void;
  onDirty?: (tabId: string) => void;
  onDocModified?: (tabId: string, action: string, payload: unknown) => void;
  onCellSelected?: (coord: string, val: string) => void;
  onCellEditing?: (coord: string, val: string) => void;
  onCellModified?: (coord: string, val: string) => void;
  onRangeStatsChanged?: (stats: unknown, rangeText: string) => void;
  onSheetSwitched?: () => void;
  onSheetAdded?: () => void;
  onSheetRenamed?: () => void;
  onSheetDeleted?: () => void;
  onSlideSorterUpdate?: () => void;
}

export class PaneController {
  private paneId: string;
  private currentDocType: DocumentType = "Word";
  private activeTabId: string | null = null;

  // Viewports & Canvas references
  private writerCanvas?: WriterCanvas;
  private calcGridCanvas?: CalcGridCanvas;
  private calcChartOverlay?: CalcChartOverlay;
  private sheetTabBar?: SheetTabBar;
  private slideStageCanvas?: SlideStageCanvas;
  private slideMediaOverlay?: SlideMediaOverlay;
  private speakerNotesPane?: SpeakerNotesPane;
  private animationPane?: AnimationPane;
  private presentationPlayer?: PresentationPlayer;
  private pdfViewerCanvas?: PdfViewerCanvas;
  private pdfWorkspace?: PdfWorkspace;
  private notesWorkspace?: NotesWorkspace;

  constructor(paneId = "primary-pane") {
    this.paneId = paneId;
  }

  public getPaneId(): string {
    return this.paneId;
  }

  public getCurrentDocType(): DocumentType {
    return this.currentDocType;
  }

  public getActiveTabId(): string | null {
    return this.activeTabId;
  }

  // Accessors
  public getWriterCanvas(): WriterCanvas | undefined {
    return this.writerCanvas;
  }

  public getCalcGridCanvas(): CalcGridCanvas | undefined {
    return this.calcGridCanvas;
  }

  public getCalcChartOverlay(): CalcChartOverlay | undefined {
    return this.calcChartOverlay;
  }

  public getSheetTabBar(): SheetTabBar | undefined {
    return this.sheetTabBar;
  }

  public getSlideStageCanvas(): SlideStageCanvas | undefined {
    return this.slideStageCanvas;
  }

  public getSlideMediaOverlay(): SlideMediaOverlay | undefined {
    return this.slideMediaOverlay;
  }

  public getSpeakerNotesPane(): SpeakerNotesPane | undefined {
    return this.speakerNotesPane;
  }

  public getAnimationPane(): AnimationPane | undefined {
    return this.animationPane;
  }

  public getPresentationPlayer(): PresentationPlayer | undefined {
    return this.presentationPlayer;
  }

  public getOrCreatePresentationPlayer(): PresentationPlayer {
    if (!this.presentationPlayer) {
      this.presentationPlayer = new PresentationPlayer(slidesState);
    }
    return this.presentationPlayer;
  }

  public getPdfViewerCanvas(): PdfViewerCanvas | undefined {
    return this.pdfViewerCanvas;
  }

  public getPdfWorkspace(): PdfWorkspace | undefined {
    return this.pdfWorkspace;
  }

  public getNotesWorkspace(): NotesWorkspace | undefined {
    return this.notesWorkspace;
  }

  /**
   * Mounts the document viewport for the active tab inside the provided viewport container.
   * Handles clean teardown of previous canvas and event listeners.
   */
  public mountViewport(
    container: HTMLElement,
    tab: TabItem,
    callbacks: PaneControllerCallbacks = {}
  ): void {
    this.unmountCurrent();

    this.currentDocType = tab.docType;
    this.activeTabId = tab.id;

    if (tab.docType === "Word") {
      this.writerCanvas = new WriterCanvas(container, writerState);
      this.writerCanvas.setOnStatusChange(() => {
        callbacks.onStatusChange?.();
      });
      this.writerCanvas.setOnDocModified((action, payload) => {
        callbacks.onDirty?.(tab.id);
        callbacks.onStatusChange?.();
        callbacks.onDocModified?.(tab.id, action, payload);
      });
    } else if (tab.docType === "Excel") {
      container.innerHTML = `
        <div class="calc-viewport-wrapper" style="position: relative; width: 100%; height: 100%; display: flex; flex-direction: column;">
          <div style="position: relative; flex: 1; overflow: hidden;">
            <div class="calc-grid-container" id="calc-grid-container" style="width: 100%; height: 100%;"></div>
            <div id="calc-chart-overlay-container"></div>
          </div>
          <div id="calc-sheet-tab-bar-container"></div>
        </div>
      `;
      const gridContainer = container.querySelector("#calc-grid-container") as HTMLElement;
      const sheetTabContainer = container.querySelector("#calc-sheet-tab-bar-container") as HTMLElement;

      if (gridContainer) {
        this.calcGridCanvas = new CalcGridCanvas(gridContainer, calcState);
        const chartOverlayEl = container.querySelector("#calc-chart-overlay-container");
        if (chartOverlayEl) {
          this.calcChartOverlay = new CalcChartOverlay("calc-chart-overlay-container", calcState);
        }

        this.calcGridCanvas.setOnCellSelected((coord, val) => {
          callbacks.onCellSelected?.(coord, val);
          callbacks.onStatusChange?.();
        });
        this.calcGridCanvas.setOnCellEditing((coord, val) => {
          callbacks.onCellEditing?.(coord, val);
        });
        this.calcGridCanvas.setOnCellModified((coord, val) => {
          callbacks.onDirty?.(tab.id);
          this.calcChartOverlay?.refresh();
          callbacks.onCellModified?.(coord, val);
          callbacks.onStatusChange?.();
        });
        this.calcGridCanvas.setOnRangeStatsChanged((stats, rangeText) => {
          callbacks.onRangeStatsChanged?.(stats, rangeText);
          callbacks.onStatusChange?.();
        });
      }

      if (sheetTabContainer) {
        this.sheetTabBar = new SheetTabBar(sheetTabContainer, calcState);
        this.sheetTabBar.setCallbacks({
          onSheetSwitched: () => {
            this.calcGridCanvas?.render();
            callbacks.onSheetSwitched?.();
            callbacks.onStatusChange?.();
          },
          onSheetAdded: () => {
            this.calcGridCanvas?.render();
            callbacks.onSheetAdded?.();
            callbacks.onStatusChange?.();
            callbacks.onDirty?.(tab.id);
          },
          onSheetRenamed: () => {
            callbacks.onSheetRenamed?.();
            callbacks.onDirty?.(tab.id);
          },
          onSheetDeleted: () => {
            this.calcGridCanvas?.render();
            callbacks.onSheetDeleted?.();
            callbacks.onStatusChange?.();
            callbacks.onDirty?.(tab.id);
          },
        });
      }
    } else if (tab.docType === "PowerPoint") {
      this.slideStageCanvas = new SlideStageCanvas(container, slidesState);
      this.slideStageCanvas.setOnDirty(() => {
        callbacks.onDirty?.(tab.id);
        callbacks.onStatusChange?.();
      });
      this.slideMediaOverlay = new SlideMediaOverlay(container, slidesState);
      this.speakerNotesPane = new SpeakerNotesPane(container, slidesState);
      this.speakerNotesPane.setOnDirty(() => callbacks.onDirty?.(tab.id));
      this.animationPane = new AnimationPane("animation-pane-container", slidesState);
      this.animationPane.setOnDirty(() => callbacks.onDirty?.(tab.id));
      this.animationPane.setCallbacks({
        onPlayAll: () => {
          this.slideStageCanvas?.playAllAnimationsPreview(slidesState.getSlideAnimations());
        },
        onPlaySelected: (anim) => {
          this.slideStageCanvas?.playAnimationPreview(anim);
        },
        onDirty: () => callbacks.onDirty?.(tab.id),
      });

      if (!this.presentationPlayer) {
        this.presentationPlayer = new PresentationPlayer(slidesState);
        this.presentationPlayer.setCallbacks({
          onExit: () => {
            this.renderCurrent();
            callbacks.onSlideSorterUpdate?.();
            this.speakerNotesPane?.refresh();
            this.animationPane?.refresh();
            callbacks.onStatusChange?.();
          },
          onSlideChange: () => {
            this.renderCurrent();
            callbacks.onSlideSorterUpdate?.();
            this.speakerNotesPane?.refresh();
            this.animationPane?.refresh();
            callbacks.onStatusChange?.();
          },
        });
      }
    } else if (tab.docType === "PDF") {
      this.pdfViewerCanvas = new PdfViewerCanvas(container, pdfState);
      this.pdfWorkspace = new PdfWorkspace(container, pdfState);
      this.pdfWorkspace.setCallbacks({
        onDirty: () => callbacks.onDirty?.(tab.id),
        onPageChange: () => {
          callbacks.onStatusChange?.();
        },
      });
    } else if (tab.docType === "Notes") {
      this.notesWorkspace = new NotesWorkspace(container, notesState);
      this.notesWorkspace.setCallbacks({
        onDirty: () => callbacks.onDirty?.(tab.id),
        onStatusChange: () => {
          callbacks.onStatusChange?.();
        },
      });
    }
  }

  /**
   * Re-renders the active viewport component.
   */
  public renderCurrent(): void {
    if (this.currentDocType === "Excel") {
      this.calcGridCanvas?.render();
      this.sheetTabBar?.render();
    } else if (this.currentDocType === "Word") {
      this.writerCanvas?.render();
    } else if (this.currentDocType === "PowerPoint") {
      this.slideStageCanvas?.render();
      this.slideMediaOverlay?.refresh();
    } else if (this.currentDocType === "PDF") {
      this.pdfWorkspace?.render();
      this.pdfViewerCanvas?.render();
    } else if (this.currentDocType === "Notes") {
      this.notesWorkspace?.render();
    }
  }

  /**
   * Sets zoom level across any active viewport format.
   */
  public setZoom(val: number): void {
    const zoomRatio = val / 100;
    if (this.currentDocType === "Word" && this.writerCanvas) {
      this.writerCanvas.getState().setZoom(zoomRatio);
      this.writerCanvas.render();
    } else if (this.currentDocType === "Excel" && this.calcGridCanvas) {
      calcState.setZoom(zoomRatio);
      this.calcGridCanvas.render();
    } else if (this.currentDocType === "PowerPoint") {
      slidesState.setZoom(val);
      this.slideStageCanvas?.render();
    } else if (this.currentDocType === "PDF") {
      pdfState.setZoom(zoomRatio);
      this.pdfWorkspace?.render();
    } else if (this.currentDocType === "Notes") {
      notesState.setZoom(val);
      this.notesWorkspace?.render();
    }
  }

  /**
   * Cleans up and unmounts the current document viewport, freeing DOM nodes and event listeners.
   */
  public unmountCurrent(): void {
    if (this.writerCanvas) {
      this.writerCanvas.destroy();
      this.writerCanvas = undefined;
    }
    this.calcGridCanvas = undefined;
    this.sheetTabBar = undefined;
    this.calcChartOverlay = undefined;
    this.slideStageCanvas = undefined;
    if (this.slideMediaOverlay) {
      this.slideMediaOverlay.clear();
      this.slideMediaOverlay = undefined;
    }
    this.speakerNotesPane = undefined;
    this.animationPane = undefined;
    this.pdfViewerCanvas = undefined;
    this.pdfWorkspace = undefined;
    this.notesWorkspace = undefined;
  }

  public destroy(): void {
    this.unmountCurrent();
    this.activeTabId = null;
  }
}
