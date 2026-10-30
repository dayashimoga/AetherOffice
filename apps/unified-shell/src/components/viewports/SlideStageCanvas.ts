import {
  SlideAnimation,
  SlideShape,
  SlideTransitionConfig,
  SlidesState,
  slidesState,
} from "../../state/slidesState";
import { SlideTransitionEngine } from "../SlideTransitionEngine";

export interface AlignmentGuide {
  orientation: "vertical" | "horizontal";
  pos: number;
}

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
  private wrapper: HTMLElement;
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private state: SlidesState;
  private slideWidth = 960; // 16:9 widescreen width DIPs
  private slideHeight = 540; // 16:9 widescreen height DIPs
  private dragMode: DragMode | null = null;
  private onDirtyCallback?: () => void;
  private activeGuides: AlignmentGuide[] = [];
  private inlineEditor: HTMLTextAreaElement | null = null;
  private editingShapeId: string | null = null;
  private hasMoved = false;

  constructor(container: HTMLElement, state: SlidesState = slidesState) {
    this.container = container;
    this.state = state;

    this.wrapper = document.createElement("div");
    this.wrapper.className = "slide-stage-wrapper";
    this.wrapper.style.position = "relative";
    this.wrapper.style.width = `${this.slideWidth}px`;
    this.wrapper.style.height = `${this.slideHeight}px`;
    this.wrapper.style.margin = "32px auto";
    this.wrapper.style.boxShadow = "0 8px 30px rgba(0, 0, 0, 0.25)";

    this.canvas = document.createElement("canvas");
    this.canvas.className = "slide-stage-canvas";
    this.canvas.tabIndex = 0;
    this.canvas.style.display = "block";
    this.canvas.style.position = "absolute";
    this.canvas.style.top = "0";
    this.canvas.style.left = "0";
    this.canvas.style.outline = "none";
    this.canvas.style.cursor = "default";

    this.wrapper.appendChild(this.canvas);
    this.container.innerHTML = "";
    this.container.appendChild(this.wrapper);

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

  public getCanvas(): HTMLCanvasElement {
    return this.canvas;
  }

  public getWrapper(): HTMLElement {
    return this.wrapper;
  }

  public getInlineEditor(): HTMLTextAreaElement | null {
    return this.inlineEditor;
  }

  public getActiveGuides(): AlignmentGuide[] {
    return this.activeGuides;
  }

  public getState(): SlidesState {
    return this.state;
  }

  private initCanvasSize() {
    const w = this.state.slideWidth;
    const h = this.state.slideHeight;
    const dpr = typeof window !== "undefined" ? window.devicePixelRatio || 1 : 1;
    this.wrapper.style.width = `${w}px`;
    this.wrapper.style.height = `${h}px`;
    this.canvas.width = w * dpr;
    this.canvas.height = h * dpr;
    this.canvas.style.width = `${w}px`;
    this.canvas.style.height = `${h}px`;
    if (this.ctx.setTransform) {
      this.ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    } else if (this.ctx.scale) {
      this.ctx.scale(dpr, dpr);
    }
  }

  public selectSlide(index: number) {
    this.finishInlineEditing();
    this.state.selectSlide(index);
    this.render();
  }

  public getCanvasPoint(e: MouseEvent): { x: number; y: number } {
    const rect = this.canvas.getBoundingClientRect ? this.canvas.getBoundingClientRect() : null;
    const clientX = e.clientX !== undefined ? e.clientX : 0;
    const clientY = e.clientY !== undefined ? e.clientY : 0;
    const left = rect ? rect.left : 0;
    const top = rect ? rect.top : 0;
    return {
      x: clientX - left,
      y: clientY - top,
    };
  }

  public getHandlePositions(sh: SlideShape): { handle: string; x: number; y: number }[] {
    const x = sh.x;
    const y = sh.y;
    const w = sh.w;
    const h = sh.h;
    const r = x + w;
    const b = y + h;
    const cx = x + w / 2;
    const cy = y + h / 2;

    return [
      { handle: "tl", x: x, y: y },
      { handle: "tc", x: cx, y: y },
      { handle: "tr", x: r, y: y },
      { handle: "mr", x: r, y: cy },
      { handle: "br", x: r, y: b },
      { handle: "bc", x: cx, y: b },
      { handle: "bl", x: x, y: b },
      { handle: "ml", x: x, y: cy },
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
      this.handleMouseDown(pt, e.shiftKey);
    });

    // 3. Mouse up: finalize drag
    window.addEventListener("mouseup", () => {
      this.handleMouseUp();
    });

    // 4. Click: In-Place Text Editing on Text/Placeholder click
    this.canvas.addEventListener("click", (e: MouseEvent) => {
      const pt = this.getCanvasPoint(e);
      this.handleClick(pt);
    });

    // 5. Double Click: In-Place Text Editing
    this.canvas.addEventListener("dblclick", (e: MouseEvent) => {
      const pt = this.getCanvasPoint(e);
      this.handleDblClick(pt);
    });

    // 6. Keyboard Shortcuts on Canvas
    this.canvas.addEventListener("keydown", (e: KeyboardEvent) => {
      const selected = this.state.getSelectedShape();
      if (!selected) return;

      if (!this.inlineEditor && !e.ctrlKey && !e.altKey && !e.metaKey && (e.key.length === 1 || e.key === "Enter")) {
        e.preventDefault();
        const editor = this.startInlineEditing(selected);
        if (
          selected.text === "Click to add title" ||
          selected.text === "Click to add subtitle" ||
          selected.text === "Click to add text"
        ) {
          editor.value = e.key === "Enter" ? "" : e.key;
          selected.text = editor.value;
        } else if (e.key.length === 1) {
          editor.value += e.key;
          selected.text = editor.value;
        }
        return;
      }

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
        if (this.onDirtyCallback) this.onDirtyCallback();
      } else if (e.key === "ArrowRight") {
        e.preventDefault();
        selected.x += e.shiftKey ? 10 : 1;
        this.render();
        if (this.onDirtyCallback) this.onDirtyCallback();
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        selected.y -= e.shiftKey ? 10 : 1;
        this.render();
        if (this.onDirtyCallback) this.onDirtyCallback();
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        selected.y += e.shiftKey ? 10 : 1;
        this.render();
        if (this.onDirtyCallback) this.onDirtyCallback();
      }
    });
  }

  public handleMouseDown(pt: { x: number; y: number }, _shiftKey = false) {
    this.hasMoved = false;
    // If we're currently inline editing, finish it first
    if (this.inlineEditor) {
      this.finishInlineEditing();
    }

    const selected = this.state.getSelectedShape();

    if (selected) {
      // Check rotation handle
      const rotCx = selected.x + selected.w / 2;
      const rotCy = selected.y - 24;
      if (Math.hypot(pt.x - rotCx, pt.y - rotCy) <= 8) {
        this.state.commitSnapshot();
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
          this.state.commitSnapshot();
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
      this.state.commitSnapshot();
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
  }

  public handleMouseUp() {
    if (this.dragMode) {
      this.dragMode = null;
      this.activeGuides = [];
      this.render();
      if (this.onDirtyCallback) this.onDirtyCallback();
    }
  }

  public handleDblClick(pt: { x: number; y: number }) {
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
      this.startInlineEditing(hitShape);
    }
  }

  public startInlineEditing(shape: SlideShape): HTMLTextAreaElement {
    this.finishInlineEditing();
    this.state.commitSnapshot();

    const editor = document.createElement("textarea");
    editor.className = "slide-inline-text-editor";
    editor.style.position = "absolute";
    editor.style.left = `${shape.x}px`;
    editor.style.top = `${shape.y}px`;
    editor.style.width = `${shape.w}px`;
    editor.style.height = `${shape.h}px`;
    editor.style.boxSizing = "border-box";
    editor.style.padding = "6px 8px";
    editor.style.border = "2px dashed #c43e1c";
    editor.style.borderRadius = "2px";
    editor.style.outline = "none";
    editor.style.resize = "none";
    editor.style.zIndex = "100";
    editor.style.fontFamily = shape.fontFamily || "Aptos, Calibri, sans-serif";
    editor.style.fontSize = `${shape.fontSize || (shape.type === "text" ? 28 : 18)}px`;
    editor.style.fontWeight = shape.bold ? "bold" : shape.type === "text" ? "bold" : "normal";
    editor.style.fontStyle = shape.italic ? "italic" : "normal";
    editor.style.textDecoration = shape.underline ? "underline" : "none";
    editor.style.color = shape.textColor || (shape.type === "text" ? "#1e293b" : "#ffffff");
    editor.style.background =
      shape.fill === "transparent" ? "rgba(255, 255, 255, 0.95)" : shape.fill;

    if (shape.rotation) {
      editor.style.transform = `rotate(${shape.rotation}deg)`;
      editor.style.transformOrigin = "center center";
    }

    editor.value = shape.text || "";

    editor.addEventListener("input", () => {
      shape.text = editor.value;
      if (this.onDirtyCallback) this.onDirtyCallback();
    });

    editor.addEventListener("blur", () => {
      this.finishInlineEditing(shape, editor.value);
    });

    editor.addEventListener("keydown", (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        this.finishInlineEditing(shape, editor.value);
      } else if (e.key === "Enter" && (e.ctrlKey || e.metaKey || shape.type !== "text")) {
        e.preventDefault();
        this.finishInlineEditing(shape, editor.value);
      }
    });

    this.wrapper.appendChild(editor);
    this.inlineEditor = editor;
    this.editingShapeId = shape.id;
    editor.focus();
    editor.select();
    this.render();
    return editor;
  }

  public finishInlineEditing(targetShape?: SlideShape, text?: string) {
    if (!this.inlineEditor) return;
    const shape = targetShape || this.state.getSelectedShape();
    if (shape && text !== undefined) {
      shape.text = text;
    }
    if (this.inlineEditor.parentNode) {
      this.inlineEditor.parentNode.removeChild(this.inlineEditor);
    }
    this.inlineEditor = null;
    this.editingShapeId = null;
    this.render();
    if (this.onDirtyCallback) this.onDirtyCallback();
  }

  // --- Simulation Helpers for Automated Testing ---
  public simulateMouseDown(x: number, y: number, shiftKey = false) {
    this.handleMouseDown({ x, y }, shiftKey);
  }

  public simulateMouseMove(x: number, y: number, shiftKey = false) {
    if (this.dragMode) {
      this.handleDragMove({ x, y }, shiftKey);
    }
  }

  public simulateMouseUp() {
    this.handleMouseUp();
  }

  public handleClick(pt: { x: number; y: number }) {
    if (this.hasMoved) return;
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

    if (hitShape && (hitShape.type === "text" || hitShape.text !== undefined)) {
      this.state.selectShape(hitShape.id);
      this.startInlineEditing(hitShape);
    }
  }

  public simulateClick(x: number, y: number) {
    this.handleMouseDown({ x, y });
    this.handleMouseUp();
    this.handleClick({ x, y });
  }

  public simulateDblClick(x: number, y: number) {
    this.handleDblClick({ x, y });
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

  public handleDragMove(pt: { x: number; y: number }, shiftKey: boolean) {
    this.hasMoved = true;
    const selected = this.state.getSelectedShape();
    if (!selected || !this.dragMode) return;

    if (this.dragMode.type === "move") {
      const dx = pt.x - this.dragMode.startX;
      const dy = pt.y - this.dragMode.startY;
      let targetX = Math.round(this.dragMode.initX + dx);
      let targetY = Math.round(this.dragMode.initY + dy);
      const w = selected.w;
      const h = selected.h;

      // Smart Alignment & Snapping Guides
      const guides: AlignmentGuide[] = [];
      const threshold = 6;

      // Candidate X coordinates: slide center and other shapes
      const snapCandidatesX: number[] = [this.slideWidth / 2];
      const slide = this.state.slides[this.state.currentSlideIndex];
      if (slide) {
        slide.shapes.forEach((other) => {
          if (other.id !== selected.id) {
            snapCandidatesX.push(other.x);
            snapCandidatesX.push(other.x + other.w / 2);
            snapCandidatesX.push(other.x + other.w);
          }
        });
      }

      const myLeft = targetX;
      const myCx = targetX + w / 2;
      const myRight = targetX + w;

      for (const cand of snapCandidatesX) {
        if (Math.abs(myCx - cand) <= threshold) {
          targetX = Math.round(cand - w / 2);
          guides.push({ orientation: "vertical", pos: cand });
          break;
        }
        if (Math.abs(myLeft - cand) <= threshold) {
          targetX = Math.round(cand);
          guides.push({ orientation: "vertical", pos: cand });
          break;
        }
        if (Math.abs(myRight - cand) <= threshold) {
          targetX = Math.round(cand - w);
          guides.push({ orientation: "vertical", pos: cand });
          break;
        }
      }

      // Candidate Y coordinates: slide center and other shapes
      const snapCandidatesY: number[] = [this.slideHeight / 2];
      if (slide) {
        slide.shapes.forEach((other) => {
          if (other.id !== selected.id) {
            snapCandidatesY.push(other.y);
            snapCandidatesY.push(other.y + other.h / 2);
            snapCandidatesY.push(other.y + other.h);
          }
        });
      }

      const myTop = targetY;
      const myCy = targetY + h / 2;
      const myBottom = targetY + h;

      for (const cand of snapCandidatesY) {
        if (Math.abs(myCy - cand) <= threshold) {
          targetY = Math.round(cand - h / 2);
          guides.push({ orientation: "horizontal", pos: cand });
          break;
        }
        if (Math.abs(myTop - cand) <= threshold) {
          targetY = Math.round(cand);
          guides.push({ orientation: "horizontal", pos: cand });
          break;
        }
        if (Math.abs(myBottom - cand) <= threshold) {
          targetY = Math.round(cand - h);
          guides.push({ orientation: "horizontal", pos: cand });
          break;
        }
      }

      selected.x = targetX;
      selected.y = targetY;
      this.activeGuides = guides;
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
    if (
      this.slideWidth !== this.state.slideWidth ||
      this.slideHeight !== this.state.slideHeight
    ) {
      this.slideWidth = this.state.slideWidth;
      this.slideHeight = this.state.slideHeight;
      this.initCanvasSize();
    }

    const ctx = this.ctx;
    ctx.clearRect(0, 0, this.slideWidth, this.slideHeight);

    const slide = this.state.slides[this.state.currentSlideIndex];
    if (!slide) return;

    // 1. Slide Surface / Background
    if (slide.background) {
      if (slide.background.startsWith("linear-gradient")) {
        const grad = ctx.createLinearGradient(0, 0, this.slideWidth, this.slideHeight);
        grad.addColorStop(0, "#f8fafc");
        grad.addColorStop(1, "#cbd5e1");
        ctx.fillStyle = grad;
      } else {
        ctx.fillStyle = slide.background;
      }
    } else {
      ctx.fillStyle = "#ffffff";
    }
    ctx.fillRect(0, 0, this.slideWidth, this.slideHeight);

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
          if (ctx.roundRect) {
            ctx.roundRect(sh.x, sh.y, sh.w, sh.h, rad);
          } else {
            ctx.rect(sh.x, sh.y, sh.w, sh.h);
          }
          ctx.fill();
          if (sh.stroke) ctx.stroke();
          break;
        }

        case "ellipse":
          ctx.beginPath();
          if (ctx.ellipse) {
            ctx.ellipse(
              sh.x + sh.w / 2,
              sh.y + sh.h / 2,
              sh.w / 2,
              sh.h / 2,
              0,
              0,
              2 * Math.PI
            );
          } else {
            ctx.arc(sh.x + sh.w / 2, sh.y + sh.h / 2, Math.min(sh.w, sh.h) / 2, 0, 2 * Math.PI);
          }
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

        case "callout": {
          const rad = 8;
          const pointerW = 20;
          const pointerH = 16;
          const bodyH = Math.max(20, sh.h - pointerH);
          ctx.beginPath();
          ctx.moveTo(sh.x + rad, sh.y);
          ctx.lineTo(sh.x + sh.w - rad, sh.y);
          ctx.quadraticCurveTo(sh.x + sh.w, sh.y, sh.x + sh.w, sh.y + rad);
          ctx.lineTo(sh.x + sh.w, sh.y + bodyH - rad);
          ctx.quadraticCurveTo(sh.x + sh.w, sh.y + bodyH, sh.x + sh.w - rad, sh.y + bodyH);
          ctx.lineTo(sh.x + 40 + pointerW, sh.y + bodyH);
          ctx.lineTo(sh.x + 25, sh.y + sh.h);
          ctx.lineTo(sh.x + 40, sh.y + bodyH);
          ctx.lineTo(sh.x + rad, sh.y + bodyH);
          ctx.quadraticCurveTo(sh.x, sh.y + bodyH, sh.x, sh.y + bodyH - rad);
          ctx.lineTo(sh.x, sh.y + rad);
          ctx.quadraticCurveTo(sh.x, sh.y, sh.x + rad, sh.y);
          ctx.closePath();
          ctx.fill();
          if (sh.stroke) ctx.stroke();
          break;
        }

        case "heart": {
          const topH = sh.h * 0.3;
          ctx.beginPath();
          ctx.moveTo(sh.x + sh.w / 2, sh.y + topH);
          ctx.bezierCurveTo(
            sh.x + sh.w / 2,
            sh.y,
            sh.x,
            sh.y,
            sh.x,
            sh.y + topH
          );
          ctx.bezierCurveTo(
            sh.x,
            sh.y + (sh.h + topH) / 2,
            sh.x + sh.w / 2,
            sh.y + (sh.h + topH) / 2,
            sh.x + sh.w / 2,
            sh.y + sh.h
          );
          ctx.bezierCurveTo(
            sh.x + sh.w / 2,
            sh.y + (sh.h + topH) / 2,
            sh.x + sh.w,
            sh.y + (sh.h + topH) / 2,
            sh.x + sh.w,
            sh.y + topH
          );
          ctx.bezierCurveTo(
            sh.x + sh.w,
            sh.y,
            sh.x + sh.w / 2,
            sh.y,
            sh.x + sh.w / 2,
            sh.y + topH
          );
          ctx.closePath();
          ctx.fill();
          if (sh.stroke) ctx.stroke();
          break;
        }

        case "image": {
          ctx.fillRect(sh.x, sh.y, sh.w, sh.h);
          if (sh.stroke) ctx.strokeRect(sh.x, sh.y, sh.w, sh.h);

          ctx.save();
          ctx.fillStyle = "#64748b";
          ctx.font = "24px Segoe UI, sans-serif";
          ctx.textAlign = "center";
          ctx.textBaseline = "middle";
          ctx.fillText("🖼️", sh.x + sh.w / 2, sh.y + sh.h / 2 - 12);

          ctx.font = "12px Segoe UI, sans-serif";
          ctx.fillStyle = "#475569";
          ctx.fillText(sh.imageAlt || "Picture", sh.x + sh.w / 2, sh.y + sh.h / 2 + 16);
          ctx.restore();
          break;
        }

        case "table": {
          ctx.fillRect(sh.x, sh.y, sh.w, sh.h);
          const headers = sh.tableHeaders || ["Col 1", "Col 2", "Col 3"];
          const data = sh.tableData || [];
          const totalRows = 1 + data.length;
          const cols = headers.length;
          const colW = sh.w / cols;
          const rowH = sh.h / Math.max(1, totalRows);

          // Header row background
          ctx.fillStyle = "#c43e1c"; // Accent header
          ctx.fillRect(sh.x, sh.y, sh.w, rowH);

          // Header text
          ctx.fillStyle = "#ffffff";
          ctx.font = "bold 13px Aptos, sans-serif";
          ctx.textAlign = "center";
          ctx.textBaseline = "middle";
          headers.forEach((h, c) => {
            ctx.fillText(h, sh.x + c * colW + colW / 2, sh.y + rowH / 2);
          });

          // Data rows
          ctx.font = "12px Aptos, sans-serif";
          data.forEach((row, r) => {
            const rowY = sh.y + (r + 1) * rowH;
            if (r % 2 === 1) {
              ctx.fillStyle = "#f8fafc";
              ctx.fillRect(sh.x, rowY, sh.w, rowH);
            }
            ctx.fillStyle = "#1e293b";
            row.forEach((val, c) => {
              ctx.fillText(val, sh.x + c * colW + colW / 2, rowY + rowH / 2);
            });
          });

          // Grid lines
          ctx.strokeStyle = sh.stroke || "#cbd5e1";
          ctx.lineWidth = sh.strokeWidth || 1;
          ctx.strokeRect(sh.x, sh.y, sh.w, sh.h);
          for (let c = 1; c < cols; c++) {
            ctx.beginPath();
            ctx.moveTo(sh.x + c * colW, sh.y);
            ctx.lineTo(sh.x + c * colW, sh.y + sh.h);
            ctx.stroke();
          }
          for (let r = 1; r < totalRows; r++) {
            ctx.beginPath();
            ctx.moveTo(sh.x, sh.y + r * rowH);
            ctx.lineTo(sh.x + sh.w, sh.y + r * rowH);
            ctx.stroke();
          }
          break;
        }

        case "text":
          if (this.editingShapeId !== sh.id && sh.text) {
            const fontStyle = `${sh.italic ? "italic " : ""}${sh.bold ? "bold " : ""}${sh.fontSize || 28}px ${sh.fontFamily || "Aptos, Calibri, sans-serif"}`;
            ctx.font = fontStyle;
            ctx.fillStyle = sh.textColor || "#1e293b";
            const align = sh.textAlign || "left";
            ctx.textAlign = align === "justify" ? "left" : align;
            ctx.textBaseline = "top";
            const lines = sh.text.split("\n");
            const lineHeight = (sh.fontSize || 28) * 1.35;
            lines.forEach((line, i) => {
              const posX = align === "center" ? sh.x + sh.w / 2 : align === "right" ? sh.x + sh.w - 8 : sh.x + 8;
              ctx.fillText(line, posX, sh.y + 8 + i * lineHeight);
              if (sh.underline) {
                const metrics = ctx.measureText ? ctx.measureText(line) : { width: 50 };
                const underY = sh.y + 8 + i * lineHeight + (sh.fontSize || 28);
                const lineStart = align === "center" ? sh.x + sh.w / 2 - metrics.width / 2 : align === "right" ? sh.x + sh.w - 8 - metrics.width : sh.x + 8;
                ctx.beginPath();
                ctx.moveTo(lineStart, underY);
                ctx.lineTo(lineStart + metrics.width, underY);
                ctx.strokeStyle = sh.textColor || "#1e293b";
                ctx.lineWidth = 1.5;
                ctx.stroke();
              }
            });
          }
          break;
      }

      // Render text for non-text shapes if present
      if (sh.type !== "text" && sh.text && this.editingShapeId !== sh.id) {
        ctx.save();
        const fontStyle = `${sh.italic ? "italic " : ""}${sh.bold ? "bold " : ""}${sh.fontSize || 16}px ${sh.fontFamily || "Aptos, Calibri, sans-serif"}`;
        ctx.font = fontStyle;
        ctx.fillStyle = sh.textColor || "#ffffff";
        const align = sh.textAlign || "center";
        ctx.textAlign = align === "justify" ? "center" : align;
        ctx.textBaseline = "middle";
        const lines = sh.text.split("\n");
        const lineHeight = (sh.fontSize || 16) * 1.3;
        const startY = sh.y + sh.h / 2 - ((lines.length - 1) * lineHeight) / 2;
        lines.forEach((line, i) => {
          const posX = align === "left" ? sh.x + 8 : align === "right" ? sh.x + sh.w - 8 : sh.x + sh.w / 2;
          ctx.fillText(line, posX, startY + i * lineHeight);
        });
        ctx.restore();
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
        if (ctx.setLineDash) ctx.setLineDash([4, 4]);
        ctx.strokeRect(sh.x - 3, sh.y - 3, sh.w + 6, sh.h + 6);
        if (ctx.setLineDash) ctx.setLineDash([]);

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

        // Motion Path Trajectory Guide if present
        const motionAnim = this.state.getSlideAnimations().find(
          (a) => a.shapeId === sh.id && a.effect === "motionPath" && a.motionPath
        );
        if (motionAnim?.motionPath) {
          const trajectory = SlideTransitionEngine.compileTrajectory(
            motionAnim.motionPath,
            motionAnim.easing,
            40,
            false
          );
          ctx.save();
          ctx.strokeStyle = "#38bdf8";
          ctx.lineWidth = 2;
          if (ctx.setLineDash) ctx.setLineDash([4, 3]);
          ctx.beginPath();
          trajectory.forEach((pt, idx) => {
            const px = sh.x + sh.w / 2 + pt.x;
            const py = sh.y + sh.h / 2 + pt.y;
            if (idx === 0) ctx.moveTo(px, py);
            else ctx.lineTo(px, py);
          });
          ctx.stroke();
          if (ctx.setLineDash) ctx.setLineDash([]);
          const lastPt = trajectory[trajectory.length - 1];
          if (lastPt) {
            ctx.fillStyle = "#38bdf8";
            ctx.beginPath();
            ctx.arc(sh.x + sh.w / 2 + lastPt.x, sh.y + sh.h / 2 + lastPt.y, 4, 0, Math.PI * 2);
            ctx.fill();
          }
          ctx.restore();
        }

        ctx.restore();
      }
    });

    // 4. Render Smart Alignment / Snapping Guides
    if (this.activeGuides.length > 0) {
      ctx.save();
      ctx.strokeStyle = "#ef4444"; // Red/terracotta guide
      ctx.lineWidth = 1;
      if (ctx.setLineDash) ctx.setLineDash([4, 4]);
      this.activeGuides.forEach((g) => {
        ctx.beginPath();
        if (g.orientation === "vertical") {
          ctx.moveTo(g.pos, 0);
          ctx.lineTo(g.pos, this.slideHeight);
        } else {
          ctx.moveTo(0, g.pos);
          ctx.lineTo(this.slideWidth, g.pos);
        }
        ctx.stroke();
      });
      if (ctx.setLineDash) ctx.setLineDash([]);
      ctx.restore();
    }
  }

  public async playTransitionPreview(config?: SlideTransitionConfig): Promise<void> {
    const trans = config || this.state.getSlideTransition();
    await SlideTransitionEngine.executeTransition(this.wrapper, trans, () => {
      this.render();
    });
  }

  public async playAnimationPreview(anim: SlideAnimation): Promise<void> {
    const shape = this.state.slides[this.state.currentSlideIndex]?.shapes.find(
      (s) => s.id === anim.shapeId
    );
    if (!shape) return;

    if (anim.effect === "motionPath" && anim.motionPath) {
      const origX = shape.x;
      const origY = shape.y;
      const origRot = shape.rotation ?? 0;
      const trajectory = SlideTransitionEngine.compileTrajectory(
        anim.motionPath,
        anim.easing || "ease-in-out",
        30,
        anim.rotateToPath ?? true
      );

      for (const pt of trajectory) {
        shape.x = origX + pt.x;
        shape.y = origY + pt.y;
        if (anim.rotateToPath) {
          shape.rotation = origRot + pt.angle;
        }
        this.render();
        await new Promise((r) =>
          setTimeout(r, Math.max(16, (anim.duration * 1000) / trajectory.length))
        );
      }

      shape.x = origX;
      shape.y = origY;
      shape.rotation = origRot;
      this.render();
      return;
    }

    const origStroke = shape.stroke;
    const origStrokeWidth = shape.strokeWidth;
    shape.stroke =
      anim.type === "entrance" ? "#22c55e" : anim.type === "emphasis" ? "#eab308" : "#ef4444";
    shape.strokeWidth = 4;
    this.render();

    await new Promise((resolve) => setTimeout(resolve, Math.max(80, anim.duration * 300)));
    shape.stroke = origStroke;
    shape.strokeWidth = origStrokeWidth;
    this.render();
  }

  public async playAllAnimationsPreview(animations: SlideAnimation[]): Promise<void> {
    for (const anim of animations) {
      await this.playAnimationPreview(anim);
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
  }
}
