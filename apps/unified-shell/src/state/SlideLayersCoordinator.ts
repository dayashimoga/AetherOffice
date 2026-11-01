/**
 * SlideLayersCoordinator.ts
 * GEC-v7.0 Stage 48 (Sub-Phase 48.1.2)
 *
 * Industrial-grade presentation shell coordinator for:
 * - SmartArt Builder & Visual Diagram Synthesis
 * - Shape Grouping and Ungrouping with coordinate preservation
 * - Hierarchical Z-Order Layer Tree management
 * - Magnetic smart alignment guides with sub-pixel / EMU precision
 */

import {
  IPC,
  type SlideLayerTreeDto,
  type LayerItemDto,
  type SmartArtBuildRequest,
  type SmartGuideLine,
  type SnapResult,
  type RectBounds,
} from "../ipc";

export interface SlideLayersState {
  activeSlideIndex: number;
  layerTree: SlideLayerTreeDto | null;
  selectedShapeIds: number[];
  activeGuides: SmartGuideLine[];
  activePackageBytes: Uint8Array | null;
  isDirty: boolean;
  snappingEnabled: boolean;
}

export type LayersStateListener = (state: Readonly<SlideLayersState>) => void;

export class SlideLayersCoordinator {
  private state: SlideLayersState = {
    activeSlideIndex: 0,
    layerTree: null,
    selectedShapeIds: [],
    activeGuides: [],
    activePackageBytes: null,
    isDirty: false,
    snappingEnabled: true,
  };

  private listeners: Set<LayersStateListener> = new Set();

  /**
   * Subscribes to slide layer state changes.
   */
  subscribe(listener: LayersStateListener): () => void {
    this.listeners.add(listener);
    listener(this.getState());
    return () => this.listeners.delete(listener);
  }

  /**
   * Returns current state snapshot.
   */
  getState(): Readonly<SlideLayersState> {
    return { ...this.state };
  }

  private notify(): void {
    const snapshot = this.getState();
    for (const listener of this.listeners) {
      listener(snapshot);
    }
  }

  /**
   * Loads and extracts the layer tree for a specific slide.
   */
  async loadSlideLayers(
    packageBytes: Uint8Array,
    slideIdx: number = 0
  ): Promise<SlideLayerTreeDto> {
    this.state.activePackageBytes = packageBytes;
    this.state.activeSlideIndex = slideIdx;
    const tree = await IPC.pptGetSlideLayerTree(packageBytes, slideIdx);
    this.state.layerTree = tree;
    this.state.selectedShapeIds = [];
    this.notify();
    return tree;
  }

  /**
   * Selects a shape by ID, optionally in multi-selection mode.
   */
  selectShape(shapeId: number, multiSelect: boolean = false): void {
    if (multiSelect) {
      if (this.state.selectedShapeIds.includes(shapeId)) {
        this.state.selectedShapeIds = this.state.selectedShapeIds.filter(
          (id) => id !== shapeId
        );
      } else {
        this.state.selectedShapeIds = [...this.state.selectedShapeIds, shapeId];
      }
    } else {
      this.state.selectedShapeIds = [shapeId];
    }
    this.notify();
  }

  /**
   * Selects multiple shapes by ID.
   */
  selectMultipleShapes(shapeIds: number[]): void {
    this.state.selectedShapeIds = [...shapeIds];
    this.notify();
  }

  /**
   * Clears active selection.
   */
  clearSelection(): void {
    this.state.selectedShapeIds = [];
    this.notify();
  }

  /**
   * Enables or disables magnetic snapping.
   */
  setSnappingEnabled(enabled: boolean): void {
    if (this.state.snappingEnabled !== enabled) {
      this.state.snappingEnabled = enabled;
      if (!enabled) {
        this.clearSmartGuides();
      }
      this.notify();
    }
  }

  /**
   * Groups selected shapes into a single composite GroupShape.
   */
  async groupSelectedShapes(): Promise<Uint8Array> {
    const pkgBytes = this.state.activePackageBytes;
    if (!pkgBytes) {
      throw new Error("No active presentation package loaded in SlideLayersCoordinator");
    }
    if (this.state.selectedShapeIds.length < 2) {
      throw new Error("At least two shapes must be selected to create a group");
    }

    const updatedBytes = await IPC.pptGroupShapes(
      pkgBytes,
      this.state.activeSlideIndex,
      this.state.selectedShapeIds
    );
    this.state.activePackageBytes = updatedBytes;
    this.state.isDirty = true;

    // Refresh layer tree
    await this.loadSlideLayers(updatedBytes, this.state.activeSlideIndex);
    return updatedBytes;
  }

  /**
   * Ungroups the currently selected group shape.
   */
  async ungroupSelectedGroup(): Promise<Uint8Array> {
    const pkgBytes = this.state.activePackageBytes;
    if (!pkgBytes) {
      throw new Error("No active presentation package loaded in SlideLayersCoordinator");
    }
    if (this.state.selectedShapeIds.length !== 1) {
      throw new Error("Exactly one group shape must be selected to ungroup");
    }

    const groupId = this.state.selectedShapeIds[0];
    const updatedBytes = await IPC.pptUngroupShapes(
      pkgBytes,
      this.state.activeSlideIndex,
      groupId
    );
    this.state.activePackageBytes = updatedBytes;
    this.state.isDirty = true;

    // Refresh layer tree
    await this.loadSlideLayers(updatedBytes, this.state.activeSlideIndex);
    return updatedBytes;
  }

  /**
   * Reorders the Z-order of the primary selected shape.
   */
  async reorderZOrder(
    action: "front" | "back" | "forward" | "backward"
  ): Promise<Uint8Array> {
    const pkgBytes = this.state.activePackageBytes;
    if (!pkgBytes) {
      throw new Error("No active presentation package loaded in SlideLayersCoordinator");
    }
    if (this.state.selectedShapeIds.length === 0) {
      throw new Error("A shape must be selected to reorder Z-order");
    }

    const targetId = this.state.selectedShapeIds[0];
    const updatedBytes = await IPC.pptReorderZOrder(
      pkgBytes,
      this.state.activeSlideIndex,
      targetId,
      action
    );
    this.state.activePackageBytes = updatedBytes;
    this.state.isDirty = true;

    // Refresh layer tree and maintain active target selection
    await this.loadSlideLayers(updatedBytes, this.state.activeSlideIndex);
    this.state.selectedShapeIds = [targetId];
    this.notify();
    return updatedBytes;
  }

  /**
   * Generates a SmartArt diagram on the active slide.
   */
  async buildSmartArt(request: SmartArtBuildRequest): Promise<Uint8Array> {
    const pkgBytes = this.state.activePackageBytes;
    if (!pkgBytes) {
      throw new Error("No active presentation package loaded in SlideLayersCoordinator");
    }

    const updatedBytes = await IPC.pptBuildSmartArt(
      pkgBytes,
      this.state.activeSlideIndex,
      request
    );
    this.state.activePackageBytes = updatedBytes;
    this.state.isDirty = true;

    // Refresh layer tree
    await this.loadSlideLayers(updatedBytes, this.state.activeSlideIndex);
    return updatedBytes;
  }

  /**
   * Computes magnetic smart alignment guides during interactive drag / transform.
   */
  async computeMagneticSnap(
    moving: RectBounds,
    neighbors: RectBounds[],
    canvas?: RectBounds | null,
    threshold?: number
  ): Promise<SnapResult> {
    if (!this.state.snappingEnabled) {
      return {
        snapped_x: moving.x,
        snapped_y: moving.y,
        dx: 0,
        dy: 0,
        guides: [],
      };
    }
    const snap = await IPC.renderSmartGuidesSnap(moving, neighbors, canvas, threshold);
    this.state.activeGuides = snap.guides;
    this.notify();
    return snap;
  }

  /**
   * Clears active smart alignment guides from the canvas overlay.
   */
  clearSmartGuides(): void {
    if (this.state.activeGuides.length > 0) {
      this.state.activeGuides = [];
      this.notify();
    }
  }

  /**
   * Returns list of selected LayerItemDto items.
   */
  getSelectedLayers(): LayerItemDto[] {
    if (!this.state.layerTree || this.state.selectedShapeIds.length === 0) {
      return [];
    }
    const result: LayerItemDto[] = [];
    const collect = (items: LayerItemDto[]) => {
      for (const item of items) {
        if (this.state.selectedShapeIds.includes(item.id)) {
          result.push(item);
        }
        if (item.children.length > 0) {
          collect(item.children);
        }
      }
    };
    collect(this.state.layerTree.items);
    return result;
  }

  /**
   * Resets coordinator state.
   */
  reset(): void {
    this.state = {
      activeSlideIndex: 0,
      layerTree: null,
      selectedShapeIds: [],
      activeGuides: [],
      activePackageBytes: null,
      isDirty: false,
      snappingEnabled: true,
    };
    this.notify();
  }
}

export const slideLayersCoordinator = new SlideLayersCoordinator();
