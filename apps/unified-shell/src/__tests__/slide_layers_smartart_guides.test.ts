import { describe, it, expect, beforeEach } from 'vitest';
import { SlideLayersCoordinator, type SlideLayersState } from '../state/SlideLayersCoordinator';
import type { SmartArtItemDto } from '../ipc';

describe('SlideLayersCoordinator & SmartArt / Smart Guides (Sub-Phase 48.1.2)', () => {
  let coordinator: SlideLayersCoordinator;

  beforeEach(() => {
    coordinator = new SlideLayersCoordinator();
  });

  it('1. Initial state has null layer tree, empty selections, and snapping enabled', () => {
    const state = coordinator.getState();
    expect(state.layerTree).toBeNull();
    expect(state.selectedShapeIds).toEqual([]);
    expect(state.activeGuides).toEqual([]);
    expect(state.snappingEnabled).toBe(true);
    expect(state.isDirty).toBe(false);
  });

  it('2. Loads slide layer tree via IPC', async () => {
    const dummyBytes = new Uint8Array([0x50, 0x4b, 0x03, 0x04]);
    const tree = await coordinator.loadSlideLayers(dummyBytes, 0);

    expect(tree).toBeDefined();
    expect(tree.slide_index).toBe(0);
    expect(tree.items.length).toBeGreaterThan(0);

    const state = coordinator.getState();
    expect(state.layerTree).toEqual(tree);
    expect(state.activePackageBytes).toEqual(dummyBytes);
  });

  it('3. Selects single shape and toggles selection', () => {
    coordinator.selectShape(101);
    expect(coordinator.getState().selectedShapeIds).toEqual([101]);

    // Single select switches selection
    coordinator.selectShape(102);
    expect(coordinator.getState().selectedShapeIds).toEqual([102]);

    // Multi-select / toggle adds shape
    coordinator.selectShape(103, true);
    expect(coordinator.getState().selectedShapeIds).toEqual([102, 103]);

    // Multi-select / toggle removes already selected shape
    coordinator.selectShape(102, true);
    expect(coordinator.getState().selectedShapeIds).toEqual([103]);
  });

  it('4. Multi-shape selection and clear selection', () => {
    coordinator.selectMultipleShapes([101, 102, 103]);
    expect(coordinator.getState().selectedShapeIds).toEqual([101, 102, 103]);

    coordinator.clearSelection();
    expect(coordinator.getState().selectedShapeIds).toEqual([]);
  });

  it('5. Groups selected shapes when >= 2 shapes are selected', async () => {
    const dummyBytes = new Uint8Array([0x50, 0x4b, 0x03, 0x04]);
    await coordinator.loadSlideLayers(dummyBytes, 0);

    coordinator.selectMultipleShapes([101, 102]);
    const resultBytes = await coordinator.groupSelectedShapes();

    expect(resultBytes).toBeInstanceOf(Uint8Array);
    expect(resultBytes.length).toBeGreaterThan(0);
    expect(coordinator.getState().isDirty).toBe(true);
  });

  it('6. Ungroups selected group shape', async () => {
    const dummyBytes = new Uint8Array([0x50, 0x4b, 0x03, 0x04]);
    await coordinator.loadSlideLayers(dummyBytes, 0);

    coordinator.selectShape(999); // Group shape ID
    const resultBytes = await coordinator.ungroupSelectedGroup();

    expect(resultBytes).toBeInstanceOf(Uint8Array);
    expect(resultBytes.length).toBeGreaterThan(0);
    expect(coordinator.getState().isDirty).toBe(true);
  });

  it('7. Reorders Z-order for selected shape', async () => {
    const dummyBytes = new Uint8Array([0x50, 0x4b, 0x03, 0x04]);
    await coordinator.loadSlideLayers(dummyBytes, 0);
    coordinator.selectShape(101);

    for (const action of ['front', 'forward', 'backward', 'back'] as const) {
      const resultBytes = await coordinator.reorderZOrder(action);
      expect(resultBytes).toBeInstanceOf(Uint8Array);
      expect(resultBytes.length).toBeGreaterThan(0);
      expect(coordinator.getState().isDirty).toBe(true);
    }
  });

  it('8. Builds SmartArt diagram and populates slide', async () => {
    const dummyBytes = new Uint8Array([0x50, 0x4b, 0x03, 0x04]);
    await coordinator.loadSlideLayers(dummyBytes, 0);

    const items: SmartArtItemDto[] = [
      { title: 'Planning', description: 'Phase 1', level: 0 },
      { title: 'Development', description: 'Phase 2', level: 0 },
      { title: 'Testing', description: 'Phase 3', level: 0 },
    ];

    const resultBytes = await coordinator.buildSmartArt({
      layout_type: 'process',
      items,
      x: 500000,
      y: 500000,
      cx: 3000000,
      cy: 1500000,
    });

    expect(resultBytes).toBeInstanceOf(Uint8Array);
    expect(resultBytes.length).toBeGreaterThan(0);
    expect(coordinator.getState().layerTree).not.toBeNull();
    expect(coordinator.getState().isDirty).toBe(true);
  });

  it('9. Computes magnetic snap with smart alignment guides', async () => {
    const moving = { x: 98, y: 100, w: 200, h: 100 };
    const targets = [
      { x: 100, y: 300, w: 200, h: 100 },
    ];

    const snap = await coordinator.computeMagneticSnap(moving, targets, null, 5);
    expect(snap).toBeDefined();
    // 98 should snap to 100 (left alignment)
    expect(snap.snapped_x).toBe(100);
    expect(snap.guides.length).toBeGreaterThan(0);

    // Active guides stored in coordinator state
    expect(coordinator.getState().activeGuides).toEqual(snap.guides);

    // Disable snapping test
    coordinator.setSnappingEnabled(false);
    const snapDisabled = await coordinator.computeMagneticSnap(moving, targets);
    expect(snapDisabled.snapped_x).toBe(moving.x);
    expect(snapDisabled.guides).toEqual([]);
  });

  it('10. Clears active guides and manages listener subscription lifecycle', () => {
    let callCount = 0;
    let observedState: SlideLayersState | null = null;

    const unsubscribe = coordinator.subscribe((state) => {
      callCount++;
      observedState = state;
    });

    // 1. Initial subscription emits once
    expect(callCount).toBe(1);

    // 2. selectShape emits
    coordinator.selectShape(42);
    expect(callCount).toBe(2);
    expect((observedState as SlideLayersState | null)?.selectedShapeIds).toEqual([42]);

    // 3. setSnappingEnabled emits
    coordinator.setSnappingEnabled(false);
    expect(callCount).toBe(3);

    // 4. Force non-empty guides then clear
    (coordinator as unknown as { state: SlideLayersState }).state.activeGuides = [
      { orientation: 'Vertical', kind: 'Edge', coordinate: 100, start: 50, end: 350 },
    ];
    coordinator.clearSmartGuides();
    expect(callCount).toBe(4);
    expect((observedState as SlideLayersState | null)?.activeGuides).toEqual([]);

    unsubscribe();
    coordinator.selectShape(99);
    expect(callCount).toBe(4); // No new invocation after unsubscribe
  });
});
