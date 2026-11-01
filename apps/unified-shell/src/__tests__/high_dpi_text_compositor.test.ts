import { describe, it, expect, beforeEach } from "vitest";
import { HighDpiCanvasCompositor } from "../components/HighDpiCanvasCompositor";

describe("Sub-Phase 45.1.2: HighDpiCanvasCompositor & Subpixel Text", () => {
  let compositor: HighDpiCanvasCompositor;

  beforeEach(() => {
    compositor = new HighDpiCanvasCompositor(2.0);
  });

  it("initializes with specified or default device pixel ratio", () => {
    expect(compositor.getDevicePixelRatio()).toBe(2.0);

    compositor.setDevicePixelRatio(1.5);
    expect(compositor.getDevicePixelRatio()).toBe(1.5);

    const defaultComp = new HighDpiCanvasCompositor();
    expect(defaultComp.getDevicePixelRatio()).toBeGreaterThanOrEqual(1.0);
  });

  it("snaps coordinates crisply to physical device pixels", () => {
    // Under DPR 2.0, coordinates snap to multiples of 0.5
    expect(compositor.snapCoord(10.1)).toBe(10.0);
    expect(compositor.snapCoord(10.26)).toBe(10.5);
    expect(compositor.snapCoord(10.74)).toBe(10.5);
    expect(compositor.snapCoord(10.8)).toBe(11.0);

    // Under DPR 1.0, coordinates snap to integers
    compositor.setDevicePixelRatio(1.0);
    expect(compositor.snapCoord(10.4)).toBe(10.0);
    expect(compositor.snapCoord(10.6)).toBe(11.0);
  });

  it("snaps hairlines to half-pixel centers for crisp 1px strokes", () => {
    compositor.setDevicePixelRatio(1.0);
    expect(compositor.snapHairline(10)).toBe(10.5);

    compositor.setDevicePixelRatio(2.0);
    // At 2.0x, half pixel is 0.25 offset from boundary
    expect(compositor.snapHairline(10)).toBe(10.25);
  });

  it("detects RTL scripts accurately", () => {
    expect(HighDpiCanvasCompositor.isRtl("English text")).toBe(false);
    expect(HighDpiCanvasCompositor.isRtl("שלום עולם")).toBe(true);
    expect(HighDpiCanvasCompositor.isRtl("مرحبا بالعالم")).toBe(true);
    expect(HighDpiCanvasCompositor.isRtl("Mixed: Hello עולם")).toBe(true);
  });

  it("evaluates subpixel kerning adjustments", () => {
    expect(HighDpiCanvasCompositor.getKerningRatio("A", "V")).toBeLessThan(0);
    expect(HighDpiCanvasCompositor.getKerningRatio("T", "o")).toBeLessThan(0);
    expect(HighDpiCanvasCompositor.getKerningRatio("W", "a")).toBeLessThan(0);
    expect(HighDpiCanvasCompositor.getKerningRatio("o", "o")).toBe(0);
  });

  it("measures text with kerning and snaps baseline", () => {
    const metrics = compositor.measureTextWithKerning("AVatar", 16);
    expect(metrics.width).toBeGreaterThan(0);
    expect(metrics.height).toBeGreaterThan(0);
    // Baseline should be snapped to device pixel
    expect(metrics.baseline).toBe(compositor.snapCoord(metrics.baseline));
  });

  it("configures HTMLCanvasElement backing store with device pixel ratio scaling", () => {
    const canvas = document.createElement("canvas");
    compositor.setupCanvas(canvas, 800, 600);

    // DPR 2.0 => backing store is 1600 x 1200, CSS is 800 x 600
    expect(canvas.width).toBe(1600);
    expect(canvas.height).toBe(1200);
    expect(canvas.style.width).toBe("800px");
    expect(canvas.style.height).toBe("600px");
  });
});

