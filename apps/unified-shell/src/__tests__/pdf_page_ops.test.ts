import { describe, it, expect, beforeEach } from 'vitest';
import { PdfPageOpsCoordinator } from '../state/PdfPageOpsCoordinator';

describe('PdfPageOpsCoordinator - Page Ops, Bates Numbering & OCR (Sub-Phase 49.1.2)', () => {
  let coordinator: PdfPageOpsCoordinator;
  const dummyPdfBytes = new Uint8Array([
    0x25, 0x50, 0x44, 0x46, 0x2d, 0x31, 0x2e, 0x37, 0x0a, 0x25, 0xe2, 0xe3,
  ]);
  const doc2Bytes = new Uint8Array([0x25, 0x50, 0x44, 0x46, 0x02]);
  const doc3Bytes = new Uint8Array([0x25, 0x50, 0x44, 0x46, 0x03]);

  beforeEach(() => {
    coordinator = new PdfPageOpsCoordinator(dummyPdfBytes);
  });

  it('1. Initializes cleanly with active bytes, default Bates config, and empty queues', () => {
    const state = coordinator.getState();
    expect(state.activeBytes).toBe(dummyPdfBytes);
    expect(state.mergeQueue).toEqual([]);
    expect(state.selectedPageIndices).toEqual([]);
    expect(state.splitBoundaries).toEqual([]);
    expect(state.batesConfig.prefix).toBe('CONFIDENTIAL-');
    expect(state.batesConfig.position).toBe('BottomRight');
    expect(state.isLoading).toBe(false);
    expect(state.error).toBeNull();
    expect(state.lastResultBytes).toBeNull();
    expect(state.lastSplitResults).toBeNull();
  });

  it('2. Sets active bytes, selected page indices, and split boundaries', () => {
    coordinator.setActiveBytes(doc2Bytes);
    coordinator.setSelectedPageIndices([0, 2]);
    coordinator.setSplitBoundaries([1, 3]);

    const state = coordinator.getState();
    expect(state.activeBytes).toBe(doc2Bytes);
    expect(state.selectedPageIndices).toEqual([0, 2]);
    expect(state.splitBoundaries).toEqual([1, 3]);
  });

  it('3. Extracts selected pages and stores result bytes in coordinator state', async () => {
    coordinator.setSelectedPageIndices([1]);
    const result = await coordinator.extractPages();

    expect(result).toBeInstanceOf(Uint8Array);
    const state = coordinator.getState();
    expect(state.lastResultBytes).toBe(result);
    expect(state.isLoading).toBe(false);
    expect(state.error).toBeNull();
  });

  it('4. Rejects extraction when no active bytes or no pages are selected', async () => {
    coordinator.setActiveBytes(null);
    await expect(coordinator.extractPages([0])).rejects.toThrow('No active PDF document loaded');

    coordinator.setActiveBytes(dummyPdfBytes);
    coordinator.setSelectedPageIndices([]);
    await expect(coordinator.extractPages()).rejects.toThrow('No pages specified');
  });

  it('5. Splits document by boundary indices and records split results', async () => {
    coordinator.setSplitBoundaries([2]);
    const parts = await coordinator.splitDocument();

    expect(Array.isArray(parts)).toBe(true);
    expect(parts.length).toBeGreaterThan(0);
    expect(parts[0]).toBeInstanceOf(Uint8Array);

    const state = coordinator.getState();
    expect(state.lastSplitResults).toEqual(parts);
    expect(state.isLoading).toBe(false);
  });

  it('6. Splits document into single pages when boundaries are empty', async () => {
    const parts = await coordinator.splitDocument([]);

    expect(Array.isArray(parts)).toBe(true);
    expect(parts[0]).toBeInstanceOf(Uint8Array);
    expect(coordinator.getState().lastSplitResults).toEqual(parts);
  });

  it('7. Queues multiple documents for merge and merges them into one byte buffer', async () => {
    coordinator.queueForMerge(doc2Bytes);
    coordinator.queueForMerge(doc3Bytes);

    expect(coordinator.getState().mergeQueue.length).toBe(2);

    const merged = await coordinator.mergeDocuments();
    expect(merged).toBeInstanceOf(Uint8Array);
    expect(coordinator.getState().lastResultBytes).toBe(merged);
    expect(coordinator.getState().isLoading).toBe(false);
  });

  it('8. Clears the merge queue when requested', () => {
    coordinator.queueForMerge(doc2Bytes);
    expect(coordinator.getState().mergeQueue.length).toBe(1);

    coordinator.clearMergeQueue();
    expect(coordinator.getState().mergeQueue.length).toBe(0);
  });

  it('9. Applies Bates numbering overlay with customized prefix and position', async () => {
    coordinator.setBatesConfig({
      prefix: 'EXHIBIT-',
      suffix: '-CONF',
      start_number: 10,
      position: 'TopLeft',
    });

    const state = coordinator.getState();
    expect(state.batesConfig.prefix).toBe('EXHIBIT-');
    expect(state.batesConfig.position).toBe('TopLeft');

    const result = await coordinator.applyBatesNumbering();
    expect(result).toBeInstanceOf(Uint8Array);
    expect(coordinator.getState().lastResultBytes).toBe(result);
  });

  it('10. Stores OCR bounding boxes and injects invisible OCR text layer (3 Tr)', async () => {
    coordinator.setOcrData(0, {
      page_index: 0,
      words: [
        {
          text: 'CONTRACT',
          bbox: { x: 50, y: 700, width: 80, height: 18 },
          confidence: 0.99,
        },
        {
          text: 'AGREEMENT',
          bbox: { x: 140, y: 700, width: 100, height: 18 },
          confidence: 0.98,
        },
      ],
    });

    expect(coordinator.getState().ocrPages[0]).toBeDefined();
    expect(coordinator.getState().ocrPages[0].words.length).toBe(2);

    const result = await coordinator.injectOcrLayer(0);
    expect(result).toBeInstanceOf(Uint8Array);
    expect(coordinator.getState().lastResultBytes).toBe(result);
    expect(coordinator.getState().isLoading).toBe(false);
  });
});

