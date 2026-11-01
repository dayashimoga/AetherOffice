/**
 * High-DPI Canvas Compositor & Subpixel Typographic Engine
 *
 * Implements GEC-v7.0 Article I, V, and VI:
 * - 120fps rendering budget (<= 8.33ms per frame).
 * - High-DPI backing store scaling (Retina, 4K, 1.25x, 1.5x, 2.0x, 3.0x).
 * - Subpixel physical device pixel snapping to eliminate font baseline blurriness.
 * - Subpixel kerning adjustments for common typographic pairs.
 */

export interface TextMetrics2D {
  width: number;
  height: number;
  baseline: number;
}

export class HighDpiCanvasCompositor {
  private dpr: number;

  constructor(dpr?: number) {
    if (dpr !== undefined && dpr > 0) {
      this.dpr = dpr;
    } else if (typeof window !== "undefined" && window.devicePixelRatio) {
      this.dpr = window.devicePixelRatio;
    } else {
      this.dpr = 1.0;
    }
  }

  public getDevicePixelRatio(): number {
    return this.dpr;
  }

  public setDevicePixelRatio(dpr: number): void {
    this.dpr = dpr > 0 ? dpr : 1.0;
  }

  /**
   * Snaps a logical coordinate (in DIPs) to the nearest physical device pixel boundary.
   */
  public snapCoord(coord: number): number {
    return Math.round(coord * this.dpr) / this.dpr;
  }

  /**
   * Snaps a 1px border or hairline coordinate to sit crisply on pixel centers.
   */
  public snapHairline(coord: number): number {
    const pixel = Math.round(coord * this.dpr);
    return (pixel + 0.5) / this.dpr;
  }

  /**
   * Configures an HTMLCanvasElement for crisp High-DPI rendering.
   */
  public setupCanvas(
    canvas: HTMLCanvasElement,
    cssWidth: number,
    cssHeight: number
  ): CanvasRenderingContext2D | null {
    const physicalWidth = Math.round(cssWidth * this.dpr);
    const physicalHeight = Math.round(cssHeight * this.dpr);

    if (canvas.width !== physicalWidth || canvas.height !== physicalHeight) {
      canvas.width = physicalWidth;
      canvas.height = physicalHeight;
      canvas.style.width = `${cssWidth}px`;
      canvas.style.height = `${cssHeight}px`;
    }

    const ctx = canvas.getContext ? canvas.getContext("2d") : null;
    if (ctx && ctx.setTransform) {
      ctx.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
    }
    return ctx;
  }

  /**
   * Returns kerning ratio for common character pairs.
   */
  public static getKerningRatio(prev: string, next: string): number {
    const pair = `${prev}${next}`;
    switch (pair) {
      case "AV":
      case "VA":
      case "AW":
      case "WA":
      case "AY":
      case "YA":
        return -0.08;
      case "To":
      case "Te":
      case "Ta":
      case "Tu":
      case "Tr":
        return -0.07;
      case "Fa":
      case "Fe":
      case "Fo":
      case "Pa":
      case "Pe":
        return -0.05;
      case "Wa":
      case "We":
      case "Wo":
      case "Ye":
      case "Yo":
        return -0.06;
      case "LT":
      case "LV":
      case "LW":
      case "LY":
        return -0.06;
      case "r.":
      case "r,":
      case "v.":
      case "v,":
      case "w.":
      case "w,":
        return -0.04;
      default:
        return 0.0;
    }
  }

  /**
   * Checks if string contains RTL characters (Hebrew / Arabic).
   */
  public static isRtl(text: string): boolean {
    return /[\u0590-\u05FF\u0600-\u06FF\u0700-\u074F\u0750-\u077F\u08A0-\u08FF\uFB50-\uFDFF\uFE70-\uFEFF]/.test(
      text
    );
  }

  /**
   * Computes subpixel metrics with baseline snapping.
   */
  public measureTextWithKerning(
    text: string,
    fontSizePx: number
  ): TextMetrics2D {
    let width = 0;
    for (let i = 0; i < text.length; i++) {
      const ch = text[i];
      let advance = fontSizePx * 0.52;
      if (ch === " ") advance = fontSizePx * 0.3;
      else if (/[A-Z]/.test(ch)) advance = fontSizePx * 0.65;
      else if (/[.,;:!?]/.test(ch)) advance = fontSizePx * 0.35;

      if (i > 0) {
        const kerning = HighDpiCanvasCompositor.getKerningRatio(text[i - 1], ch);
        advance += kerning * fontSizePx;
      }
      width += advance;
    }

    const height = fontSizePx * 1.25;
    const baseline = this.snapCoord(fontSizePx * 0.95);

    return {
      width: this.snapCoord(width),
      height: this.snapCoord(height),
      baseline,
    };
  }
}
