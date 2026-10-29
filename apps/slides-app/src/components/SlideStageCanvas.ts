import { SlideShape, SlidesState, slidesState } from "../state/slidesState";

type DragMode =
  | { type: "move"; startX: number; startY: number; initX: number; initY: number }
  | {
      type: "resize";
      handle: string;
      startX: number;
      startY: number;
      initX: number;
      initY: number;
      initW: number;
      initH: number;
    }
  | { type: "rotate"; cx: number; cy: number; startAngle: number; initRot: number };

export class SlideStageCanvas {
  private container: HTMLElement;
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private state: SlidesState;
  private slideWidth = 960; // 16:9 widescreen width DIPs
  private slideHeight = 540; // 16:9 widescreen height DIPs
  private dragMode: DragMode | null = null;
  private onDirtyCallback?: () => void;

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

  public setOnDirty(cb: () => void) {
    this.onDirtyCallback = cb;
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

  private getCanvasPoint(e: MouseEvent): { x: number; y: number } {
    const rect = this.canvas.getBoundingClientRect();
    return {
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
    };
  }

  private getHandlePositions(sh: SlideShape): { handle: string; x: number; y: number }[] {
    const x = sh.x;
    const y = sh.y;
    const w = sh.w;
    const h = sh.h;
    const r = x + w;
    const b = y + h;
    const cx = x + w / 2;
    const cy = y + h / 2;

    return [
      { handle: "tl", x, y },
      { handle: "tc", x: cx, y },
      { handle: "tr", x: r, y },
      { handle: "mr", x: r, y: cy },
      { handle: "br", x: r, y: b },
      { handle: "bc", x: cx, y: b },
      { handle: "bl", x, y: b },
      { handle: "ml", x, y: cy },
    ];
  }

  private attachEvents() {
    // 1. Mouse move: hover cursor detection or active dragging
    this.canvas.addEventListener("mousemove", (e: MouseEvent) => {
      const pt = this.getCanvasPoint(e);

      if (this.dragMode) {
        this.handleDragMove(pt, e.shiftKey);
        return;
      }

      // Update cursor based on hover target
      const selected = this.state.getSelectedShape();
      if (selected) {
        // Test rotation handle
        const rotCx = selected.x + selected.w / 2;
        const rotCy = selected.y - 24;
        const dRot = Math.hypot(pt.x - rotCx, pt.y - rotCy);
        if (dRot <= 7) {
          this.canvas.style.cursor = "grab";
          return;
        }

        // Test 8 handles
        const handles = this.getHandlePositions(selected);
        for (const h of handles) {
          if (Math.abs(pt.x - h.x) <= 6 && Math.abs(pt.y - h.y) <= 6) {
            this.canvas.style.cursor = this.getCursorForHandle(h.handle);
            return;
          }
        }
      }

      // Test shape bodies
      const slide = this.state.slides[this.state.currentSlideIndex];
      if (slide) {
        for (let i = slide.shapes.length - 1; i >= 0; i--) {
          const sh = slide.shapes[i];
          if (pt.x >= sh.x && pt.x <= sh.x + sh.w && pt.y >= sh.y && pt.y <= sh.y + sh.h) {
            this.canvas.style.cursor = "move";
            return;
          }
        }
      }

      this.canvas.style.cursor = "default";
    });

    // 2. Mouse down: initiate selection or transform dragging
    this.canvas.addEventListener("mousedown", (e: MouseEvent) => {
      this.canvas.focus();
      const pt = this.getCanvasPoint(e);
      const selected = this.state.getSelectedShape();

      if (selected) {
        // Check rotation handle
        const rotCx = selected.x + selected.w / 2;
        const rotCy = selected.y - 24;
        if (Math.hypot(pt.x - rotCx, pt.y - rotCy) <= 8) {
          this.dragMode = {
            type: "rotate",
            cx: rotCx,
            cy: selected.y + selected.h / 2,
            startAngle: Math.atan2(pt.y - (selected.y + selected.h / 2), pt.x - rotCx),
            initRot: selected.rotation || 0,
          };
          this.canvas.style.cursor = "grabbing";
          return;
        }

        // Check 8 resize handles
        const handles = this.getHandlePositions(selected);
        for (const h of handles) {
          if (Math.abs(pt.x - h.x) <= 6 && Math.abs(pt.y - h.y) <= 6) {
            this.dragMode = {
              type: "resize",
              handle: h.handle,
              startX: pt.x,
              startY: pt.y,
              initX: selected.x,
              initY: selected.y,
              initW: selected.w,
              initH: selected.h,
            };
            return;
          }
        }
      }

      // Check shape body hit
      const slide = this.state.slides[this.state.currentSlideIndex];
      if (!slide) return;

      let hitShape: SlideShape | null = null;
      for (let i = slide.shapes.length - 1; i >= 0; i--) {
        const sh = slide.shapes[i];
        if (pt.x >= sh.x && pt.x <= sh.x + sh.w && pt.y >= sh.y && pt.y <= sh.y + sh.h) {
          hitShape = sh;
          break;
        }
      }

      if (hitShape) {
        this.state.selectShape(hitShape.id);
        this.dragMode = {
          type: "move",
          startX: pt.x,
          startY: pt.y,
          initX: hitShape.x,
          initY: hitShape.y,
        };
      } else {
        this.state.selectShape(null);
      }

      this.render();
      if (this.onDirtyCallback) this.onDirtyCallback();
    });

    // 3. Mouse up: finalize drag
    window.addEventListener("mouseup", () => {
      if (this.dragMode) {
        this.dragMode = null;
        this.state.commitSnapshot();
        this.render();
        if (this.onDirtyCallback) this.onDirtyCallback();
      }
    });

    // 4. Keyboard Shortcuts on Canvas
    this.canvas.addEventListener("keydown", (e: KeyboardEvent) => {
      const selected = this.state.getSelectedShape();
      if (!selected) return;

      if (e.key === "Delete" || e.key === "Backspace") {
        e.preventDefault();
        this.state.deleteSelectedShape();
        this.render();
        if (this.onDirtyCallback) this.onDirtyCallback();
      } else if (e.ctrlKey && e.key === "d") {
        e.preventDefault();
        this.state.duplicateSelectedShape();
        this.render();
        if (this.onDirtyCallback) this.onDirtyCallback();
      } else if (e.key === "ArrowLeft") {
        e.preventDefault();
        selected.x -= e.shiftKey ? 10 : 1;
        this.render();
      } else if (e.key === "ArrowRight") {
        e.preventDefault();
        selected.x += e.shiftKey ? 10 : 1;
        this.render();
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        selected.y -= e.shiftKey ? 10 : 1;
        this.render();
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        selected.y += e.shiftKey ? 10 : 1;
        this.render();
      }
    });
  }

  private getCursorForHandle(handle: string): string {
    switch (handle) {
      case "tl":
      case "br":
        return "nwse-resize";
      case "tr":
      case "bl":
        return "nesw-resize";
      case "tc":
      case "bc":
        return "ns-resize";
      case "ml":
      case "mr":
        return "ew-resize";
      default:
        return "default";
    }
  }

  private handleDragMove(pt: { x: number; y: number }, shiftKey: boolean) {
    const selected = this.state.getSelectedShape();
    if (!selected || !this.dragMode) return;

    if (this.dragMode.type === "move") {
      const dx = pt.x - this.dragMode.startX;
      const dy = pt.y - this.dragMode.startY;
      selected.x = Math.round(this.dragMode.initX + dx);
      selected.y = Math.round(this.dragMode.initY + dy);
      this.render();
    } else if (this.dragMode.type === "resize") {
      const dx = pt.x - this.dragMode.startX;
      const dy = pt.y - this.dragMode.startY;
      const minSize = 16;
      const { initX, initY, initW, initH, handle } = this.dragMode;
      const aspect = initW / (initH || 1);

      let newX = initX;
      let newY = initY;
      let newW = initW;
      let newH = initH;

      if (handle === "br") {
        newW = Math.max(minSize, initW + dx);
        newH = Math.max(minSize, initH + dy);
        if (shiftKey) {
          const s = Math.max(newW / initW, newH / initH);
          newW = Math.round(initW * s);
          newH = Math.round(initH * s);
        }
      } else if (handle === "tl") {
        newW = Math.max(minSize, initW - dx);
        newH = Math.max(minSize, initH - dy);
        if (shiftKey) {
          const s = Math.max(newW / initW, newH / initH);
          newW = Math.round(initW * s);
          newH = Math.round(initH * s);
        }
        newX = initX + initW - newW;
        newY = initY + initH - newH;
      } else if (handle === "tr") {
        newW = Math.max(minSize, initW + dx);
        newH = Math.max(minSize, initH - dy);
        if (shiftKey) {
          const s = Math.max(newW / initW, newH / initH);
          newW = Math.round(initW * s);
          newH = Math.round(initH * s);
        }
        newY = initY + initH - newH;
      } else if (handle === "bl") {
        newW = Math.max(minSize, initW - dx);
        newH = Math.max(minSize, initH + dy);
        if (shiftKey) {
          const s = Math.max(newW / initW, newH / initH);
          newW = Math.round(initW * s);
          newH = Math.round(initH * s);
        }
        newX = initX + initW - newW;
      } else if (handle === "tc") {
        newH = Math.max(minSize, initH - dy);
        newY = initY + initH - newH;
        if (shiftKey) {
          newW = Math.round(newH * aspect);
          newX = Math.round(initX + (initW - newW) / 2);
        }
      } else if (handle === "bc") {
        newH = Math.max(minSize, initH + dy);
        if (shiftKey) {
          newW = Math.round(newH * aspect);
          newX = Math.round(initX + (initW - newW) / 2);
        }
      } else if (handle === "ml") {
        newW = Math.max(minSize, initW - dx);
        newX = initX + initW - newW;
        if (shiftKey) {
          newH = Math.round(newW / aspect);
          newY = Math.round(initY + (initH - newH) / 2);
        }
      } else if (handle === "mr") {
        newW = Math.max(minSize, initW + dx);
        if (shiftKey) {
          newH = Math.round(newW / aspect);
          newY = Math.round(initY + (initH - newH) / 2);
        }
      }

      selected.x = newX;
      selected.y = newY;
      selected.w = newW;
      selected.h = newH;
      this.render();
    } else if (this.dragMode.type === "rotate") {
      const currentAngle = Math.atan2(pt.y - this.dragMode.cy, pt.x - this.dragMode.cx);
      const deltaAngle = (currentAngle - this.dragMode.startAngle) * (180 / Math.PI);
      let newRot = (this.dragMode.initRot + deltaAngle) % 360;
      if (newRot < 0) newRot += 360;
      if (shiftKey) {
        newRot = Math.round(newRot / 15) * 15;
      }
      selected.rotation = Math.round(newRot);
      this.render();
    }
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

      // Apply rotation if present
      if (sh.rotation) {
        const cx = sh.x + sh.w / 2;
        const cy = sh.y + sh.h / 2;
        ctx.translate(cx, cy);
        ctx.rotate((sh.rotation * Math.PI) / 180);
        ctx.translate(-cx, -cy);
      }

      ctx.fillStyle = sh.fill;
      if (sh.stroke) {
        ctx.strokeStyle = sh.stroke;
        ctx.lineWidth = sh.strokeWidth || 1;
      }

      switch (sh.type) {
        case "rect":
          ctx.fillRect(sh.x, sh.y, sh.w, sh.h);
          if (sh.stroke) ctx.strokeRect(sh.x, sh.y, sh.w, sh.h);
          break;

        case "roundRect": {
          const rad = Math.min(sh.w, sh.h) * 0.15;
          ctx.beginPath();
          ctx.roundRect(sh.x, sh.y, sh.w, sh.h, rad);
          ctx.fill();
          if (sh.stroke) ctx.stroke();
          break;
        }

        case "ellipse":
          ctx.beginPath();
          ctx.ellipse(
            sh.x + sh.w / 2,
            sh.y + sh.h / 2,
            sh.w / 2,
            sh.h / 2,
            0,
            0,
            2 * Math.PI
          );
          ctx.fill();
          if (sh.stroke) ctx.stroke();
          break;

        case "triangle":
          ctx.beginPath();
          ctx.moveTo(sh.x + sh.w / 2, sh.y);
          ctx.lineTo(sh.x + sh.w, sh.y + sh.h);
          ctx.lineTo(sh.x, sh.y + sh.h);
          ctx.closePath();
          ctx.fill();
          if (sh.stroke) ctx.stroke();
          break;

        case "arrow": {
          const headL = sh.w * 0.35;
          const shaftH = sh.h * 0.4;
          const sTop = sh.y + (sh.h - shaftH) / 2;
          const sBot = sTop + shaftH;
          const hX = sh.x + sh.w - headL;

          ctx.beginPath();
          ctx.moveTo(sh.x, sTop);
          ctx.lineTo(hX, sTop);
          ctx.lineTo(hX, sh.y);
          ctx.lineTo(sh.x + sh.w, sh.y + sh.h / 2);
          ctx.lineTo(hX, sh.y + sh.h);
          ctx.lineTo(hX, sBot);
          ctx.lineTo(sh.x, sBot);
          ctx.closePath();
          ctx.fill();
          if (sh.stroke) ctx.stroke();
          break;
        }

        case "star": {
          const rOuter = Math.min(sh.w, sh.h) / 2;
          const rInner = rOuter * 0.382;
          const cx = sh.x + sh.w / 2;
          const cy = sh.y + sh.h / 2;
          const step = Math.PI / 5;
          let ang = -Math.PI / 2;

          ctx.beginPath();
          for (let i = 0; i < 10; i++) {
            const r = i % 2 === 0 ? rOuter : rInner;
            const px = cx + r * Math.cos(ang);
            const py = cy + r * Math.sin(ang);
            if (i === 0) ctx.moveTo(px, py);
            else ctx.lineTo(px, py);
            ang += step;
          }
          ctx.closePath();
          ctx.fill();
          if (sh.stroke) ctx.stroke();
          break;
        }

        case "diamond": {
          const cx = sh.x + sh.w / 2;
          const cy = sh.y + sh.h / 2;
          ctx.beginPath();
          ctx.moveTo(cx, sh.y);
          ctx.lineTo(sh.x + sh.w, cy);
          ctx.lineTo(cx, sh.y + sh.h);
          ctx.lineTo(sh.x, cy);
          ctx.closePath();
          ctx.fill();
          if (sh.stroke) ctx.stroke();
          break;
        }

        case "text":
          ctx.font = `bold ${sh.fontSize || 28}px Aptos, Calibri, sans-serif`;
          ctx.fillStyle = sh.textColor || "#1e293b";
          ctx.textAlign = "left";
          ctx.textBaseline = "top";
          ctx.fillText(sh.text || "", sh.x + 8, sh.y + 8);
          break;
      }

      ctx.restore();

      // 3. Selection Bounding Box & 8 Transform Gizmos
      if (sh.id === this.state.selectedShapeId) {
        ctx.save();
        if (sh.rotation) {
          const cx = sh.x + sh.w / 2;
          const cy = sh.y + sh.h / 2;
          ctx.translate(cx, cy);
          ctx.rotate((sh.rotation * Math.PI) / 180);
          ctx.translate(-cx, -cy);
        }

        ctx.strokeStyle = "#c43e1c"; // PowerPoint terracotta red
        ctx.lineWidth = 1.5;
        ctx.setLineDash([4, 4]);
        ctx.strokeRect(sh.x - 3, sh.y - 3, sh.w + 6, sh.h + 6);
        ctx.setLineDash([]);

        // Rotation Handle Stalk and Circle at Top
        ctx.beginPath();
        ctx.moveTo(sh.x + sh.w / 2, sh.y - 3);
        ctx.lineTo(sh.x + sh.w / 2, sh.y - 20);
        ctx.stroke();

        ctx.beginPath();
        ctx.arc(sh.x + sh.w / 2, sh.y - 24, 4.5, 0, 2 * Math.PI);
        ctx.fillStyle = "#c43e1c";
        ctx.fill();

        // 8 Resize Handles
        const handles = this.getHandlePositions(sh);
        ctx.fillStyle = "#ffffff";
        ctx.strokeStyle = "#c43e1c";
        ctx.lineWidth = 1.5;
        handles.forEach((h) => {
          ctx.fillRect(h.x - 3.5, h.y - 3.5, 7, 7);
          ctx.strokeRect(h.x - 3.5, h.y - 3.5, 7, 7);
        });

        ctx.restore();
      }
    });
  }
}
