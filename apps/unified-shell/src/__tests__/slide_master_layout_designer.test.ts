/**
 * slide_master_layout_designer.test.ts
 * GEC-v7.0 Sub-Phase 48.1.1: Interactive Slide Master & Layout Designer View Test Suite
 */

import { describe, it, expect, beforeEach, vi } from "vitest";
import {
  SlideMasterCoordinator,
  type SlideMasterState,
} from "../state/SlideMasterCoordinator";
import { IPC } from "../ipc";

describe("SlideMasterCoordinator (Sub-Phase 48.1.1)", () => {
  let coordinator: SlideMasterCoordinator;
  const mockPackageBytes = new Uint8Array([0x50, 0x4b, 0x03, 0x04, 0x00, 0x00]);

  beforeEach(() => {
    coordinator = new SlideMasterCoordinator();
  });

  it("test_1_initial_state: should initialize with clean default state", () => {
    const state = coordinator.getState();
    expect(state.isMasterViewOpen).toBe(false);
    expect(state.masterTree).toBeNull();
    expect(state.activeMasterPart).toBeNull();
    expect(state.activeLayoutPart).toBeNull();
    expect(state.selectedPlaceholderId).toBeNull();
    expect(state.isDirty).toBe(false);
    expect(state.activePackageBytes).toBeNull();
  });

  it("test_2_load_presentation_package: should load master tree and set initial active master & layout", async () => {
    const tree = await coordinator.loadPackage(mockPackageBytes);
    expect(tree).toBeDefined();
    expect(tree.masters.length).toBeGreaterThan(0);

    const state = coordinator.getState();
    expect(state.masterTree).toEqual(tree);
    expect(state.activeMasterPart).toBe(tree.masters[0].part_name);
    expect(state.activeLayoutPart).toBe(tree.masters[0].layouts[0].part_name);
    expect(state.activePackageBytes).toEqual(mockPackageBytes);
  });

  it("test_3_open_and_close_master_view: should toggle master view mode and notify listeners", async () => {
    await coordinator.loadPackage(mockPackageBytes);

    let observedState: SlideMasterState | null = null;
    const unsub = coordinator.subscribe((st) => {
      observedState = st;
    });

    coordinator.openMasterView("ppt/slideMasters/slideMaster1.xml", "ppt/slideLayouts/slideLayout1.xml");
    const stateAfterOpen = observedState as SlideMasterState | null;
    expect(stateAfterOpen?.isMasterViewOpen).toBe(true);
    expect(stateAfterOpen?.activeMasterPart).toBe("ppt/slideMasters/slideMaster1.xml");
    expect(stateAfterOpen?.activeLayoutPart).toBe("ppt/slideLayouts/slideLayout1.xml");

    coordinator.closeMasterView();
    const stateAfterClose = observedState as SlideMasterState | null;
    expect(stateAfterClose?.isMasterViewOpen).toBe(false);
    expect(stateAfterClose?.selectedPlaceholderId).toBeNull();

    unsub();
  });

  it("test_4_select_master_and_layout: should update active parts and derive master from layout", async () => {
    await coordinator.loadPackage(mockPackageBytes);

    // Select master
    coordinator.selectMaster("ppt/slideMasters/slideMaster1.xml");
    expect(coordinator.getState().activeMasterPart).toBe("ppt/slideMasters/slideMaster1.xml");
    expect(coordinator.getState().activeLayoutPart).toBeNull();

    // Select layout
    coordinator.selectLayout("ppt/slideLayouts/slideLayout2.xml");
    expect(coordinator.getState().activeLayoutPart).toBe("ppt/slideLayouts/slideLayout2.xml");
    expect(coordinator.getState().activeMasterPart).toBe("ppt/slideMasters/slideMaster1.xml");
  });

  it("test_5_select_placeholder_shape: should select placeholder and retrieve metadata", async () => {
    await coordinator.loadPackage(mockPackageBytes);
    coordinator.selectLayout("ppt/slideLayouts/slideLayout1.xml");

    // Initially no placeholder selected
    expect(coordinator.getSelectedPlaceholder()).toBeNull();

    // Select Title placeholder (shape_id: 1)
    coordinator.selectPlaceholder(1);
    expect(coordinator.getState().selectedPlaceholderId).toBe(1);

    const ph = coordinator.getSelectedPlaceholder();
    expect(ph).toBeDefined();
    expect(ph?.shape_id).toBe(1);
    expect(ph?.ph_type).toBe("title");
    expect(ph?.name).toBe("Title");
  });

  it("test_6_create_custom_layout: should invoke IPC and reload tree", async () => {
    await coordinator.loadPackage(mockPackageBytes);
    const spy = vi.spyOn(IPC, "pptCreateSlideLayout");

    const updated = await coordinator.createLayout("twoContent", "Custom Comparison");
    expect(spy).toHaveBeenCalledWith(
      mockPackageBytes,
      "ppt/slideMasters/slideMaster1.xml",
      "twoContent",
      "Custom Comparison"
    );
    expect(updated).toBeInstanceOf(Uint8Array);
    expect(coordinator.getState().isDirty).toBe(true);
  });

  it("test_7_update_placeholder_bounds: should invoke IPC with EMU coordinates", async () => {
    await coordinator.loadPackage(mockPackageBytes);
    coordinator.selectLayout("ppt/slideLayouts/slideLayout1.xml");

    const spy = vi.spyOn(IPC, "pptUpdatePlaceholderTransform");
    const updated = await coordinator.updatePlaceholderBounds(
      1,
      900000,
      500000,
      10000000,
      1200000
    );

    expect(spy).toHaveBeenCalledWith(
      mockPackageBytes,
      "ppt/slideLayouts/slideLayout1.xml",
      1,
      900000,
      500000,
      10000000,
      1200000
    );
    expect(updated).toBeInstanceOf(Uint8Array);
    expect(coordinator.getState().isDirty).toBe(true);
  });

  it("test_8_apply_layout_to_slide: should invoke IPC to reassign slide layout", async () => {
    await coordinator.loadPackage(mockPackageBytes);
    const spy = vi.spyOn(IPC, "pptApplySlideMasterLayout");

    const updated = await coordinator.applyLayoutToSlide(0, "ppt/slideLayouts/slideLayout2.xml");
    expect(spy).toHaveBeenCalledWith(
      mockPackageBytes,
      0,
      "ppt/slideLayouts/slideLayout2.xml"
    );
    expect(updated).toBeInstanceOf(Uint8Array);
    expect(coordinator.getState().isDirty).toBe(true);
  });

  it("test_9_get_available_presets: should return all 9 standard layout presets", () => {
    const presets = coordinator.getAvailablePresets();
    expect(presets).toHaveLength(9);
    const ids = presets.map((p) => p.id);
    expect(ids).toContain("title");
    expect(ids).toContain("titleAndContent");
    expect(ids).toContain("sectionHeader");
    expect(ids).toContain("twoContent");
    expect(ids).toContain("comparison");
    expect(ids).toContain("titleOnly");
    expect(ids).toContain("blank");
    expect(ids).toContain("contentWithCaption");
    expect(ids).toContain("pictureWithCaption");
  });

  it("test_10_subscribe_and_unsubscribe_lifecycle: should accurately manage subscription callbacks", () => {
    let callCount = 0;
    const unsub = coordinator.subscribe(() => {
      callCount++;
    });
    // Immediately called once on subscribe
    expect(callCount).toBe(1);

    coordinator.openMasterView();
    expect(callCount).toBe(2);

    coordinator.selectPlaceholder(2);
    expect(callCount).toBe(3);

    unsub();
    coordinator.closeMasterView();
    // After unsubscribe, count should not increase
    expect(callCount).toBe(3);
  });
});

