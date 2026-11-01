import { describe, it, expect, beforeEach } from 'vitest';
import { PdfCanvasRendererCoordinator } from '../state/PdfCanvasRendererCoordinator';

describe('PdfCanvasRendererCoordinator - Canvas Renderer (Sub-Phase 49.1.1)', () => {
  let coordinator: PdfCanvasRendererCoordinator;
  const dummyPdfBytes = new Uint8Array([
    0x25, 0x50, 0x44, 0x46, 0x2d, 0x31, 0x2e, 0x37, 0x0a, 0x25, 0xe2, 0xe3,
  ]);

  beforeEach(() => {
    coordinator = new PdfCanvasRendererCoordinator();
  });

  it('1. Initial state is cleanly uninitialized with null canvas, scale 1.0, and stopped loading', () => {
    const state = coordinator.getState();
    expect(state.activeBytes).toBeNull();
    expect(state.activePageIndex).toBe(0);
    expect(state.scale).toBe(1.0);
    expect(state.currentCanvas).toBeNull();
    expect(state.isLoading).toBe(false);
    expect(state.error).toBeNull();
    expect(state.cachedPageCount).toBe(0);
  });

  it('2. Loads PDF document bytes and renders initial page', async () => {
    const canvas = await coordinator.loadDocument(dummyPdfBytes, 0);

    expect(canvas).toBeDefined();
    expect(canvas.page_index).toBe(0);
    expect(canvas.width_pt).toBe(612.0);
    expect(canvas.height_pt).toBe(792.0);

    const state = coordinator.getState();
    expect(state.activeBytes).toBe(dummyPdfBytes);
    expect(state.currentCanvas).toBe(canvas);
    expect(state.isLoading).toBe(false);
    expect(state.cachedPageCount).toBe(1);
  });

  it('3. Renders page canvas containing vector paths, text elements, and SVG markup', async () => {
    await coordinator.loadDocument(dummyPdfBytes, 0);
    const canvas = await coordinator.renderPage(0);

    expect(canvas.vector_paths.length).toBeGreaterThan(0);
    expect(canvas.vector_paths[0].d).toContain('M ');
    expect(canvas.text_elements.length).toBeGreaterThan(0);
    expect(canvas.text_elements[0].text).toContain('PDF Page');
    expect(canvas.svg_markup).toContain('<svg');
    expect(canvas.svg_markup).toContain('</svg>');
  });

  it('4. Reuses cached rendered page canvas on identical page and scale requests', async () => {
    await coordinator.loadDocument(dummyPdfBytes, 0);
    const canvas1 = await coordinator.renderPage(0, 1.0);
    const canvas2 = await coordinator.renderPage(0, 1.0);

    expect(canvas1).toBe(canvas2);
    expect(coordinator.getState().cachedPageCount).toBe(1);
  });

  it('5. Renders standalone page SVG markup directly via IPC helper', async () => {
    await coordinator.loadDocument(dummyPdfBytes, 0);
    const svg = await coordinator.renderPageSvg(0);

    expect(svg).toBeDefined();
    expect(svg).toContain('<svg');
    expect(svg).toContain('viewBox="0 0 612.00 792.00"');
    expect(svg).toContain('</svg>');
  });

  it('6. Extracts image XObjects embedded on the page', async () => {
    await coordinator.loadDocument(dummyPdfBytes, 0);
    const images = await coordinator.extractPageImages(0);

    expect(Array.isArray(images)).toBe(true);
  });

  it('7. Navigates pages forward and backward', async () => {
    await coordinator.loadDocument(dummyPdfBytes, 0);
    expect(coordinator.getState().activePageIndex).toBe(0);

    const nextCanvas = await coordinator.nextPage();
    expect(nextCanvas.page_index).toBe(1);
    expect(coordinator.getState().activePageIndex).toBe(1);

    const prevCanvas = await coordinator.previousPage();
    expect(prevCanvas.page_index).toBe(0);
    expect(coordinator.getState().activePageIndex).toBe(0);

    // Backward from 0 remains at 0
    const clampedCanvas = await coordinator.previousPage();
    expect(clampedCanvas.page_index).toBe(0);
  });

  it('8. Updates zoom scale factor and invalidates/updates canvas cache key', async () => {
    await coordinator.loadDocument(dummyPdfBytes, 0);
    const scaledCanvas = await coordinator.setScale(2.0);

    expect(scaledCanvas).toBeDefined();
    expect(scaledCanvas?.scale).toBe(2.0);
    expect(coordinator.getState().scale).toBe(2.0);
    // Two distinct zoom levels cached: 1.0 and 2.0
    expect(coordinator.getState().cachedPageCount).toBe(2);
  });

  it('9. Clears all in-memory caches upon request', async () => {
    await coordinator.loadDocument(dummyPdfBytes, 0);
    expect(coordinator.getState().cachedPageCount).toBe(1);

    coordinator.clearCache();
    expect(coordinator.getState().cachedPageCount).toBe(0);
  });

  it('10. Error handling triggers error state when document is uninitialized', async () => {
    await expect(coordinator.renderPage(0)).rejects.toThrow('No active PDF document loaded');
    expect(coordinator.getState().isLoading).toBe(false);
    expect(coordinator.getState().error).toBe('No active PDF document loaded');
  });
});

