import { SlidesState, slidesState } from "../state/slidesState";

export class SlideStageCanvas {
  private container: HTMLElement;
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private state: SlidesState;
  private slideWidth = 960; // 16:9 widescreen width
  private slideHeight = 540; // 16:9 widescreen height
  private isDragging = false;
  private dragStartX = 0;
  private dragStartY = 0;

  constructor(container: HTMLElement, state: SlidesState = slidesState) {
    this.container = container;
    this.state = state;

    this.canvas = document.createElement("canvas");
    this.canvas.className = "slide-stage-canvas";
    this.canvas.tabIndex = 0;
    this.canvas.style.display = "block";
    this.canvas.style.margin = "32px auto";
    this.canvas.style.boxShadow = "0 8px 30px rgba(0, 0, 0, 0.25)";
    this.canvas.style.outline = "none";
    this.canvas.style.cursor = "default";

    this.container.innerHTML = "";
    this.container.appendChild(this.canvas);

    const context = this.canvas.getContext("2d");
    if (!context) throw new Error("Could not acquire 2D canvas context for Slides");
    this.ctx = context;

    this.initCanvasSize();
    this.attachEvents();
    this.render();
  }

  private initCanvasSize() {
    const dpr = window.devicePixelRatio || 1;
    this.canvas.width = this.slideWidth * dpr;
    this.canvas.height = this.slideHeight * dpr;
    this.canvas.style.width = `${this.slideWidth}px`;
    this.canvas.style.height = `${this.slideHeight}px`;
    this.ctx.scale(dpr, dpr);
  }

  public selectSlide(index: number) {
    this.state.selectSlide(index);
    this.render();
  }

  private attachEvents() {
    this.canvas.addEventListener("mousedown", (e: MouseEvent) => {
      const rect = this.canvas.getBoundingClientRect();
      const x = (e.clientX - rect.left);
      const y = (e.clientY - rect.top);

      const slide = this.state.slides[this.state.currentSlideIndex];
      if (!slide) return;

      let hitId: string | null = null;
      for (let i = slide.shapes.length - 1; i >= 0; i--) {
        const sh = slide.shapes[i];
        if (x >= sh.x && x <= sh.x + sh.w && y >= sh.y && y <= sh.y + sh.h) {
          hitId = sh.id;
          break;
        }
      }

      this.state.selectShape(hitId || "");
      if (hitId) {
        this.isDragging = true;
        this.dragStartX = x;
        this.dragStartY = y;
      }
      this.render();
    });

    this.canvas.addEventListener("mousemove", (e: MouseEvent) => {
      if (!this.isDragging || !this.state.selectedShapeId) return;
      const rect = this.canvas.getBoundingClientRect();
      const x = (e.clientX - rect.left);
      const y = (e.clientY - rect.top);
      const dx = x - this.dragStartX;
      const dy = y - this.dragStartY;

      const slide = this.state.slides[this.state.currentSlideIndex];
      const shape = slide?.shapes.find((s) => s.id === this.state.selectedShapeId);
      if (shape) {
        shape.x += dx;
        shape.y += dy;
        this.dragStartX = x;
        this.dragStartY = y;
        this.render();
      }
    });

    window.addEventListener("mouseup", () => {
      this.isDragging = false;
    });
  }

  public render() {
    const ctx = this.ctx;
    ctx.clearRect(0, 0, this.slideWidth, this.slideHeight);

    // 1. Slide Surface
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(0, 0, this.slideWidth, this.slideHeight);

    const slide = this.state.slides[this.state.currentSlideIndex];
    if (!slide) return;

    // 2. Render Slide Shapes
    slide.shapes.forEach((sh) => {
      ctx.save();
      if (sh.type === "rect") {
        ctx.fillStyle = sh.fill;
        ctx.fillRect(sh.x, sh.y, sh.w, sh.h);
      } else if (sh.type === "text") {
        ctx.font = "bold 36px Aptos, Calibri, sans-serif";
        ctx.fillStyle = "#1e293b";
        ctx.textAlign = "left";
        ctx.fillText(sh.text || "", sh.x, sh.y + 40);
      } else if (sh.type === "ellipse") {
        ctx.fillStyle = sh.fill;
        ctx.beginPath();
        ctx.ellipse(sh.x + sh.w / 2, sh.y + sh.h / 2, sh.w / 2, sh.h / 2, 0, 0, 2 * Math.PI);
        ctx.fill();
      }
      ctx.restore();

      // 3. Selection Bounding Box & Transform Gizmos
      if (sh.id === this.state.selectedShapeId) {
        ctx.strokeStyle = "#c43e1c"; // PowerPoint terracotta red
        ctx.lineWidth = 2;
        ctx.setLineDash([4, 4]);
        ctx.strokeRect(sh.x - 4, sh.y - 4, sh.w + 8, sh.h + 8);
        ctx.setLineDash([]);

        // 8 Resize Handles
        const handlePositions = [
          [sh.x - 7, sh.y - 7], // TL
          [sh.x + sh.w / 2 - 3, sh.y - 7], // TC
          [sh.x + sh.w + 1, sh.y - 7], // TR
          [sh.x - 7, sh.y + sh.h / 2 - 3], // ML
          [sh.x + sh.w + 1, sh.y + sh.h / 2 - 3], // MR
          [sh.x - 7, sh.y + sh.h + 1], // BL
          [sh.x + sh.w / 2 - 3, sh.y + sh.h + 1], // BC
          [sh.x + sh.w + 1, sh.y + sh.h + 1], // BR
        ];

        ctx.fillStyle = "#ffffff";
        ctx.strokeStyle = "#c43e1c";
        ctx.lineWidth = 1.5;
        handlePositions.forEach(([hx, hy]) => {
          ctx.fillRect(hx, hy, 7, 7);
          ctx.strokeRect(hx, hy, 7, 7);
        });

        // Rotation Handle at Top
        ctx.beginPath();
        ctx.moveTo(sh.x + sh.w / 2, sh.y - 4);
        ctx.lineTo(sh.x + sh.w / 2, sh.y - 20);
        ctx.stroke();

        ctx.beginPath();
        ctx.arc(sh.x + sh.w / 2, sh.y - 24, 4, 0, 2 * Math.PI);
        ctx.fillStyle = "#c43e1c";
        ctx.fill();
      }
    });
  }
}

