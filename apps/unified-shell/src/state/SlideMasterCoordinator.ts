/**
 * SlideMasterCoordinator.ts
 * GEC-v7.0 Stage 48 (Sub-Phase 48.1.1)
 *
 * Industrial-grade state coordinator for:
 * - Interactive Slide Master & Layout Designer View
 * - 3-Tier Master -> Layout -> Slide inheritance hierarchy inspection
 * - Placeholder positioning, transform editing, and geometry sync
 * - Layout duplication, custom layout authoring, and template switching
 * - High-speed binary IPC bridge to ppt-engine with clean-room zero-panic guarantees
 */

import {
  IPC,
  type MasterLayoutTreeDto,
  type SlideMasterDto,
  type SlideLayoutDto,
  type PlaceholderDto,
} from "../ipc";

export interface SlideMasterState {
  isMasterViewOpen: boolean;
  masterTree: MasterLayoutTreeDto | null;
  activeMasterPart: string | null;
  activeLayoutPart: string | null;
  selectedPlaceholderId: number | null;
  isDirty: boolean;
  activePackageBytes: Uint8Array | null;
}

export type MasterStateListener = (state: Readonly<SlideMasterState>) => void;

export class SlideMasterCoordinator {
  private state: SlideMasterState = {
    isMasterViewOpen: false,
    masterTree: null,
    activeMasterPart: null,
    activeLayoutPart: null,
    selectedPlaceholderId: null,
    isDirty: false,
    activePackageBytes: null,
  };

  private listeners: Set<MasterStateListener> = new Set();

  /**
   * Subscribes to slide master state changes.
   */
  subscribe(listener: MasterStateListener): () => void {
    this.listeners.add(listener);
    listener(this.getState());
    return () => this.listeners.delete(listener);
  }

  /**
   * Returns a snapshot of the current coordinator state.
   */
  getState(): Readonly<SlideMasterState> {
    return { ...this.state };
  }

  /**
   * Notifies all registered listeners of a state update.
   */
  private notify(): void {
    const snapshot = this.getState();
    for (const listener of this.listeners) {
      listener(snapshot);
    }
  }

  /**
   * Sets the active presentation package bytes and parses the master tree.
   */
  async loadPackage(packageBytes: Uint8Array): Promise<MasterLayoutTreeDto> {
    this.state.activePackageBytes = packageBytes;
    const tree = await IPC.pptGetSlideMasterTree(packageBytes);
    this.state.masterTree = tree;

    if (tree.masters.length > 0) {
      const firstMaster = tree.masters[0];
      this.state.activeMasterPart = firstMaster.part_name;
      this.state.activeLayoutPart =
        firstMaster.layouts.length > 0 ? firstMaster.layouts[0].part_name : null;
    }

    this.notify();
    return tree;
  }

  /**
   * Opens the Slide Master Designer view.
   */
  openMasterView(masterPartName?: string, layoutPartName?: string): void {
    this.state.isMasterViewOpen = true;
    if (masterPartName) {
      this.state.activeMasterPart = masterPartName;
    }
    if (layoutPartName) {
      this.state.activeLayoutPart = layoutPartName;
    }
    this.state.selectedPlaceholderId = null;
    this.notify();
  }

  /**
   * Closes the Slide Master Designer view and returns to normal slide editing.
   */
  closeMasterView(): void {
    this.state.isMasterViewOpen = false;
    this.state.selectedPlaceholderId = null;
    this.notify();
  }

  /**
   * Selects a slide master part for editing.
   */
  selectMaster(masterPartName: string): void {
    this.state.activeMasterPart = masterPartName;
    this.state.activeLayoutPart = null;
    this.state.selectedPlaceholderId = null;
    this.notify();
  }

  /**
   * Selects a slide layout part for editing.
   */
  selectLayout(layoutPartName: string): void {
    this.state.activeLayoutPart = layoutPartName;
    if (this.state.masterTree) {
      for (const m of this.state.masterTree.masters) {
        if (m.layouts.some((l) => l.part_name === layoutPartName)) {
          this.state.activeMasterPart = m.part_name;
          break;
        }
      }
    }
    this.state.selectedPlaceholderId = null;
    this.notify();
  }

  /**
   * Selects a placeholder shape within the active master or layout.
   */
  selectPlaceholder(shapeId: number | null): void {
    this.state.selectedPlaceholderId = shapeId;
    this.notify();
  }

  /**
   * Returns the active slide master DTO if one is selected.
   */
  getActiveMaster(): SlideMasterDto | null {
    if (!this.state.masterTree || !this.state.activeMasterPart) {
      return null;
    }
    return (
      this.state.masterTree.masters.find(
        (m) => m.part_name === this.state.activeMasterPart
      ) || null
    );
  }

  /**
   * Returns the active slide layout DTO if one is selected.
   */
  getActiveLayout(): SlideLayoutDto | null {
    if (!this.state.masterTree || !this.state.activeLayoutPart) {
      return null;
    }
    for (const m of this.state.masterTree.masters) {
      const layout = m.layouts.find(
        (l) => l.part_name === this.state.activeLayoutPart
      );
      if (layout) return layout;
    }
    return null;
  }

  /**
   * Returns the currently selected placeholder shape DTO.
   */
  getSelectedPlaceholder(): PlaceholderDto | null {
    if (this.state.selectedPlaceholderId === null) {
      return null;
    }
    const target = this.getActiveLayout() || this.getActiveMaster();
    if (!target) return null;
    return (
      target.placeholders.find(
        (p) => p.shape_id === this.state.selectedPlaceholderId
      ) || null
    );
  }

  /**
   * Creates a new layout under the specified or active master.
   */
  async createLayout(
    layoutType: string,
    name: string,
    masterPartName?: string
  ): Promise<Uint8Array> {
    const pkgBytes = this.state.activePackageBytes;
    if (!pkgBytes) {
      throw new Error("No active presentation package loaded in SlideMasterCoordinator");
    }
    const masterPart = masterPartName || this.state.activeMasterPart;
    if (!masterPart) {
      throw new Error("No active master selected to attach layout");
    }

    const updatedBytes = await IPC.pptCreateSlideLayout(
      pkgBytes,
      masterPart,
      layoutType,
      name
    );
    this.state.activePackageBytes = updatedBytes;
    this.state.isDirty = true;

    // Refresh tree
    await this.loadPackage(updatedBytes);
    return updatedBytes;
  }

  /**
   * Updates placeholder bounding box transform (in EMU coordinates).
   */
  async updatePlaceholderBounds(
    shapeId: number,
    x: number,
    y: number,
    cx: number,
    cy: number,
    targetPart?: string
  ): Promise<Uint8Array> {
    const pkgBytes = this.state.activePackageBytes;
    if (!pkgBytes) {
      throw new Error("No active presentation package loaded in SlideMasterCoordinator");
    }
    const part = targetPart || this.state.activeLayoutPart || this.state.activeMasterPart;
    if (!part) {
      throw new Error("No active master or layout part selected for placeholder transform");
    }

    const updatedBytes = await IPC.pptUpdatePlaceholderTransform(
      pkgBytes,
      part,
      shapeId,
      x,
      y,
      cx,
      cy
    );
    this.state.activePackageBytes = updatedBytes;
    this.state.isDirty = true;

    // Refresh tree
    await this.loadPackage(updatedBytes);
    return updatedBytes;
  }

  /**
   * Applies a layout template to a slide by index.
   */
  async applyLayoutToSlide(
    slideIdx: number,
    layoutPartName: string
  ): Promise<Uint8Array> {
    const pkgBytes = this.state.activePackageBytes;
    if (!pkgBytes) {
      throw new Error("No active presentation package loaded in SlideMasterCoordinator");
    }

    const updatedBytes = await IPC.pptApplySlideMasterLayout(
      pkgBytes,
      slideIdx,
      layoutPartName
    );
    this.state.activePackageBytes = updatedBytes;
    this.state.isDirty = true;
    return updatedBytes;
  }

  /**
   * Returns list of standard PowerPoint layout presets.
   */
  getAvailablePresets(): Array<{ id: string; name: string }> {
    return [
      { id: "title", name: "Title Slide" },
      { id: "titleAndContent", name: "Title and Content" },
      { id: "sectionHeader", name: "Section Header" },
      { id: "twoContent", name: "Two Content" },
      { id: "comparison", name: "Comparison" },
      { id: "titleOnly", name: "Title Only" },
      { id: "blank", name: "Blank" },
      { id: "contentWithCaption", name: "Content with Caption" },
      { id: "pictureWithCaption", name: "Picture with Caption" },
    ];
  }

  /**
   * Resets coordinator state.
   */
  reset(): void {
    this.state = {
      isMasterViewOpen: false,
      masterTree: null,
      activeMasterPart: null,
      activeLayoutPart: null,
      selectedPlaceholderId: null,
      isDirty: false,
      activePackageBytes: null,
    };
    this.notify();
  }
}

export const slideMasterCoordinator = new SlideMasterCoordinator();

