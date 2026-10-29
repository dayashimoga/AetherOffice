export interface RawInkPoint {
  x: number;
  y: number;
  pressure: number;
  time: number;
}

export class StylusInkLayer {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private isDrawing = false;
  private currentStroke: RawInkPoint[] = [];
  private color = "#002060";
  private baseWidth = 2.5;

  constructor(overlayCanvasId = "stylus-ink-canvas") {
    let canvas = document.getElementById(overlayCanvasId) as HTMLCanvasElement;
    if (!canvas) {
      canvas = document.createElement("canvas");
      canvas.id = overlayCanvasId;
      canvas.className = "stylus-ink-overlay";
      const viewport = document.getElementById("viewport-container");
      if (viewport) viewport.appendChild(canvas);
      else document.body.appendChild(canvas);
    }
    this.canvas = canvas;
    this.ctx = canvas.getContext("2d")!;
    this.resize();
    this.attachEvents();
  }

  public resize() {
    this.canvas.width = this.canvas.parentElement?.clientWidth || window.innerWidth;
    this.canvas.height = this.canvas.parentElement?.clientHeight || window.innerHeight;
  }

  public clear() {
    this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
  }

  public setColor(color: string) {
    this.color = color;
  }

  private attachEvents() {
    this.canvas.addEventListener("pointerdown", (e: PointerEvent) => {
      this.isDrawing = true;
      this.currentStroke = [];
      this.addPoint(e);
    });

    this.canvas.addEventListener("pointermove", (e: PointerEvent) => {
      if (!this.isDrawing) return;
      this.addPoint(e);
      this.renderCurrentSegment();
    });

    const stopDrawing = () => {
      this.isDrawing = false;
      this.currentStroke = [];
    };

    this.canvas.addEventListener("pointerup", stopDrawing);
    this.canvas.addEventListener("pointercancel", stopDrawing);
  }

  private addPoint(e: PointerEvent) {
    const rect = this.canvas.getBoundingClientRect();
    const pressure = e.pressure > 0 ? e.pressure : 0.5;
    this.currentStroke.push({
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
      pressure,
      time: Date.now(),
    });
  }

  private renderCurrentSegment() {
    const len = this.currentStroke.length;
    if (len < 2) return;

    const p0 = this.currentStroke[len - 2];
    const p1 = this.currentStroke[len - 1];

    const width = this.baseWidth * (1.0 + 0.8 * (p1.pressure - 0.5));

    this.ctx.beginPath();
    this.ctx.strokeStyle = this.color;
    this.ctx.lineWidth = width;
    this.ctx.lineCap = "round";
    this.ctx.lineJoin = "round";
    this.ctx.moveTo(p0.x, p0.y);
    this.ctx.lineTo(p1.x, p1.y);
    this.ctx.stroke();
  }
}

