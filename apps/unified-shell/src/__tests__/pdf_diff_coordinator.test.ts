import { describe, it, expect, beforeEach } from 'vitest';
import { PdfDiffCoordinator } from '../state/PdfDiffCoordinator';

describe('PdfDiffCoordinator - Side-by-Side PDF Diff Engine (Sub-Phase 49.2.1)', () => {
  let coordinator: PdfDiffCoordinator;
  const dummyA = new Uint8Array([0x25, 0x50, 0x44, 0x46, 0x01]);
  const dummyB = new Uint8Array([0x25, 0x50, 0x44, 0x46, 0x02]);

  beforeEach(() => {
    coordinator = new PdfDiffCoordinator(dummyA, dummyB);
  });

  it('1. Initializes cleanly with default options, empty results, and synchronized scrolling enabled', () => {
    const state = coordinator.getState();
    expect(state.docABytes).toBe(dummyA);
    expect(state.docBBytes).toBe(dummyB);
    expect(state.diffResult).toBeNull();
    expect(state.activePageIndex).toBe(0);
    expect(state.selectedDiffId).toBeNull();
    expect(state.synchronizedScroll).toBe(true);
    expect(state.filterKind).toBe('All');
    expect(state.isLoading).toBe(false);
    expect(state.error).toBeNull();
    expect(state.options.ignore_case).toBe(false);
    expect(state.options.ignore_whitespace).toBe(true);
  });

  it('2. Loads documents and runs initial comparison', async () => {
    const result = await coordinator.loadDocuments(dummyA, dummyB);

    expect(result).toBeDefined();
    expect(result.summary.total_diffs).toBeGreaterThanOrEqual(1);

    const state = coordinator.getState();
    expect(state.diffResult).toEqual(result);
    expect(state.selectedDiffId).toBe('diff-mock-1');
    expect(state.overlaySvgA).toContain('<svg');
    expect(state.overlaySvgB).toContain('<svg');
    expect(state.isLoading).toBe(false);
  });

  it('3. Synchronizes active page index and refreshes overlays', async () => {
    await coordinator.compare();
    await coordinator.setActivePage(0);

    expect(coordinator.getState().activePageIndex).toBe(0);
    expect(coordinator.getState().overlaySvgA).toBeDefined();
    expect(coordinator.getState().overlaySvgB).toBeDefined();
  });

  it('4. Navigates sequentially between diff items with nextDiff', async () => {
    await coordinator.compare();

    const diff1 = coordinator.getState().selectedDiffId;
    expect(diff1).toBe('diff-mock-1');

    const next = coordinator.nextDiff();
    expect(next).toBeDefined();
    expect(coordinator.getState().selectedDiffId).toBe(next?.id);
  });

  it('5. Navigates backwards between diff items with previousDiff', async () => {
    await coordinator.compare();

    const prev = coordinator.previousDiff();
    expect(prev).toBeDefined();
    expect(coordinator.getState().selectedDiffId).toBe(prev?.id);
  });

  it('6. Filters differences by category (Inserted, Deleted, Modified)', async () => {
    await coordinator.compare();

    coordinator.setFilterKind('Inserted');
    const inserted = coordinator.getAllFilteredDiffs();
    expect(inserted.every((d) => d.kind === 'Inserted')).toBe(true);

    coordinator.setFilterKind('Deleted');
    const deleted = coordinator.getAllFilteredDiffs();
    expect(deleted.every((d) => d.kind === 'Deleted')).toBe(true);

    coordinator.setFilterKind('All');
    expect(coordinator.getAllFilteredDiffs().length).toBeGreaterThanOrEqual(1);
  });

  it('7. Filters differences for the active page only', async () => {
    await coordinator.compare();

    const pageDiffs = coordinator.getFilteredDiffsForActivePage();
    expect(Array.isArray(pageDiffs)).toBe(true);
    expect(pageDiffs.length).toBeGreaterThanOrEqual(1);
  });

  it('8. Updates synchronized scroll and viewport scroll offsets', () => {
    coordinator.setSynchronizedScroll(false);
    expect(coordinator.getState().synchronizedScroll).toBe(false);

    coordinator.setScrollOffset(120, 250);
    expect(coordinator.getState().scrollOffset).toEqual({ x: 120, y: 250 });
  });

  it('9. Updates comparison options (ignore_case, ignore_whitespace)', () => {
    coordinator.setOptions({ ignore_case: true, detect_moves: true });

    const state = coordinator.getState();
    expect(state.options.ignore_case).toBe(true);
    expect(state.options.detect_moves).toBe(true);
    expect(state.options.ignore_whitespace).toBe(true);
  });

  it('10. Handles missing documents gracefully with descriptive error state', async () => {
    const emptyCoordinator = new PdfDiffCoordinator();
    await expect(emptyCoordinator.compare()).rejects.toThrow('Both Document A and Document B must be loaded');

    expect(emptyCoordinator.getState().error).toContain('Both Document A and Document B');
    expect(emptyCoordinator.getState().isLoading).toBe(false);
  });
});

