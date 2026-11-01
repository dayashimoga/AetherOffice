import { describe, it, expect, beforeEach, vi } from 'vitest';
import { SlideExportCoordinator } from '../state/SlideExportCoordinator';

describe('SlideExportCoordinator - ODP & Deck Export (Sub-Phase 48.2.2)', () => {
  let coordinator: SlideExportCoordinator;
  const dummyPptxBytes = new Uint8Array([0x50, 0x4b, 0x03, 0x04, 0x14, 0x00]);
  const dummyOdpBytes = new Uint8Array([0x50, 0x4b, 0x03, 0x04, 0x00, 0x00]);

  beforeEach(() => {
    coordinator = new SlideExportCoordinator();
  });

  it('1. Initial state is cleanly uninitialized with zero error and stopped export status', () => {
    const state = coordinator.getState();
    expect(state.activePackageBytes).toBeNull();
    expect(state.isExporting).toBe(false);
    expect(state.exportProgress).toBe(0.0);
    expect(state.exportError).toBeNull();
    expect(state.lastExportedSvg).toBeNull();
    expect(state.lastExportedPdf).toBeNull();
    expect(state.lastConvertedOdp).toBeNull();
    expect(state.lastConvertedPptx).toBeNull();
    expect(state.exportedSlideIndex).toBeNull();
  });

  it('2. Setting active package bytes populates state and resets caches', () => {
    coordinator.setActivePackage(dummyPptxBytes);
    const state = coordinator.getState();
    expect(state.activePackageBytes).toBe(dummyPptxBytes);
    expect(state.exportError).toBeNull();
  });

  it('3. Exports single slide SVG markup and updates state with 100% progress', async () => {
    coordinator.setActivePackage(dummyPptxBytes);
    const svg = await coordinator.exportSlideSvg(0);

    expect(svg).toBeDefined();
    expect(svg).toContain('<svg');
    expect(svg).toContain('</svg>');
    const state = coordinator.getState();
    expect(state.lastExportedSvg).toBe(svg);
    expect(state.exportedSlideIndex).toBe(0);
    expect(state.isExporting).toBe(false);
    expect(state.exportProgress).toBe(1.0);
  });

  it('4. Returns cached SVG markup on repeated calls for the same slide index', async () => {
    coordinator.setActivePackage(dummyPptxBytes);
    const svg1 = await coordinator.exportSlideSvg(1);
    const svg2 = await coordinator.exportSlideSvg(1);

    expect(svg1).toBe(svg2);
    expect(coordinator.getState().exportedSlideIndex).toBe(1);
  });

  it('5. Batch-exports multiple slides to SVG with incremental progress updates', async () => {
    coordinator.setActivePackage(dummyPptxBytes);
    const progressValues: number[] = [];
    coordinator.subscribe((s) => {
      progressValues.push(s.exportProgress);
    });

    const svgs = await coordinator.exportAllSlidesSvg(3);
    expect(svgs.length).toBe(3);
    expect(svgs[0]).toContain('<svg');
    expect(svgs[1]).toContain('<svg');
    expect(svgs[2]).toContain('<svg');
    expect(progressValues).toContain(1.0);
    expect(coordinator.getState().isExporting).toBe(false);
  });

  it('6. Exports full multi-page PDF deck bytes', async () => {
    coordinator.setActivePackage(dummyPptxBytes);
    const pdfBytes = await coordinator.exportDeckPdf();

    expect(pdfBytes).toBeInstanceOf(Uint8Array);
    expect(pdfBytes.length).toBeGreaterThan(0);
    const pdfStr = new TextDecoder().decode(pdfBytes);
    expect(pdfStr).toContain('%PDF-1.7');
    expect(coordinator.getState().lastExportedPdf).toBe(pdfBytes);
    expect(coordinator.getState().isExporting).toBe(false);
  });

  it('7. Converts ODP byte archive to PPTX presentation package', async () => {
    const pptxBytes = await coordinator.convertOdpToPptx(dummyOdpBytes);

    expect(pptxBytes).toBeInstanceOf(Uint8Array);
    expect(coordinator.getState().lastConvertedPptx).toBe(pptxBytes);
    expect(coordinator.getState().activePackageBytes).toBe(pptxBytes);
    expect(coordinator.getState().isExporting).toBe(false);
  });

  it('8. Converts PPTX presentation package to ODP byte archive', async () => {
    coordinator.setActivePackage(dummyPptxBytes);
    const odpBytes = await coordinator.convertPptxToOdp();

    expect(odpBytes).toBeInstanceOf(Uint8Array);
    expect(coordinator.getState().lastConvertedOdp).toBe(odpBytes);
    expect(coordinator.getState().isExporting).toBe(false);
  });

  it('9. Error handling updates exportError state and resets isExporting flag', async () => {
    // Unset package bytes to trigger validation error
    await expect(coordinator.exportSlideSvg(0)).rejects.toThrow(
      'No active presentation package loaded for SVG export'
    );
    expect(coordinator.getState().isExporting).toBe(false);
    expect(coordinator.getState().exportError).toBe(
      'No active presentation package loaded for SVG export'
    );
  });

  it('10. Browser download fallback creates mock anchor element and revokes URL', () => {
    const createObjectURLSpy = vi.fn().mockReturnValue('blob:http://localhost/mock-uuid');
    const revokeObjectURLSpy = vi.fn();
    const clickSpy = vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(() => {});

    window.URL.createObjectURL = createObjectURLSpy;
    window.URL.revokeObjectURL = revokeObjectURLSpy;

    coordinator.triggerDownload(dummyPptxBytes, 'Presentation.pptx', 'application/vnd.openxmlformats');

    expect(createObjectURLSpy).toHaveBeenCalled();
    expect(clickSpy).toHaveBeenCalled();
    expect(revokeObjectURLSpy).toHaveBeenCalledWith('blob:http://localhost/mock-uuid');
  });
});
