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

      if (sh.shadow) {
        ctx.shadowColor = "rgba(0, 0, 0, 0.45)";
        ctx.shadowBlur = 12;
        ctx.shadowOffsetX = 4;
        ctx.shadowOffsetY = 6;
      } else if (sh.glow) {
        ctx.shadowColor = "rgba(59, 130, 246, 0.8)";
        ctx.shadowBlur = 16;
        ctx.shadowOffsetX = 0;
        ctx.shadowOffsetY = 0;
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

        case "arrowLeft": {
          const headL = sh.w * 0.35;
          const shaftH = sh.h * 0.4;
          const sTop = sh.y + (sh.h - shaftH) / 2;
          const sBot = sTop + shaftH;
          const hX = sh.x + headL;

          ctx.beginPath();
          ctx.moveTo(sh.x + sh.w, sTop);
          ctx.lineTo(hX, sTop);
          ctx.lineTo(hX, sh.y);
          ctx.lineTo(sh.x, sh.y + sh.h / 2);
          ctx.lineTo(hX, sh.y + sh.h);
          ctx.lineTo(hX, sBot);
          ctx.lineTo(sh.x + sh.w, sBot);
          ctx.closePath();
          ctx.fill();
          if (sh.stroke) ctx.stroke();
          break;
        }

        case "arrowUp": {
          const headH = sh.h * 0.35;
          const shaftW = sh.w * 0.4;
          const sLeft = sh.x + (sh.w - shaftW) / 2;
          const sRight = sLeft + shaftW;
          const hY = sh.y + headH;

          ctx.beginPath();
          ctx.moveTo(sLeft, sh.y + sh.h);
          ctx.lineTo(sLeft, hY);
          ctx.lineTo(sh.x, hY);
          ctx.lineTo(sh.x + sh.w / 2, sh.y);
          ctx.lineTo(sh.x + sh.w, hY);
          ctx.lineTo(sRight, hY);
          ctx.lineTo(sRight, sh.y + sh.h);
          ctx.closePath();
          ctx.fill();
          if (sh.stroke) ctx.stroke();
          break;
        }

        case "arrowDown": {
          const headH = sh.h * 0.35;
          const shaftW = sh.w * 0.4;
          const sLeft = sh.x + (sh.w - shaftW) / 2;
          const sRight = sLeft + shaftW;
          const hY = sh.y + sh.h - headH;

          ctx.beginPath();
          ctx.moveTo(sLeft, sh.y);
          ctx.lineTo(sRight, sh.y);
          ctx.lineTo(sRight, hY);
          ctx.lineTo(sh.x + sh.w, hY);
          ctx.lineTo(sh.x + sh.w / 2, sh.y + sh.h);
          ctx.lineTo(sh.x, hY);
          ctx.lineTo(sLeft, hY);
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

        case "cloud": {
          const cx = sh.x + sh.w / 2;
          const cy = sh.y + sh.h / 2;
          ctx.beginPath();
          ctx.arc(sh.x + sh.w * 0.3, cy, sh.h * 0.35, 0, Math.PI * 2);
          ctx.arc(cx, sh.y + sh.h * 0.35, sh.h * 0.35, 0, Math.PI * 2);
          ctx.arc(sh.x + sh.w * 0.7, cy, sh.h * 0.35, 0, Math.PI * 2);
          ctx.arc(cx, sh.y + sh.h * 0.65, sh.h * 0.25, 0, Math.PI * 2);
          ctx.closePath();
          ctx.fill();
          if (sh.stroke) ctx.stroke();
          break;
        }

        case "pentagon": {
          const cx = sh.x + sh.w / 2;
          const cy = sh.y + sh.h / 2;
          const r = Math.min(sh.w, sh.h) / 2;
          ctx.beginPath();
          for (let i = 0; i < 5; i++) {
            const angle = (i * 2 * Math.PI) / 5 - Math.PI / 2;
            const px = cx + r * Math.cos(angle);
            const py = cy + r * Math.sin(angle);
            if (i === 0) ctx.moveTo(px, py);
            else ctx.lineTo(px, py);
          }
          ctx.closePath();
          ctx.fill();
          if (sh.stroke) ctx.stroke();
          break;
        }

        case "hexagon": {
          const cx = sh.x + sh.w / 2;
          const cy = sh.y + sh.h / 2;
          const r = Math.min(sh.w, sh.h) / 2;
          ctx.beginPath();
          for (let i = 0; i < 6; i++) {
            const angle = (i * 2 * Math.PI) / 6;
            const px = cx + r * Math.cos(angle);
            const py = cy + r * Math.sin(angle);
            if (i === 0) ctx.moveTo(px, py);
            else ctx.lineTo(px, py);
          }
          ctx.closePath();
          ctx.fill();
          if (sh.stroke) ctx.stroke();
          break;
        }

        case "banner": {
          const indent = sh.w * 0.12;
          ctx.beginPath();
          ctx.moveTo(sh.x, sh.y);
          ctx.lineTo(sh.x + sh.w, sh.y);
          ctx.lineTo(sh.x + sh.w - indent, sh.y + sh.h / 2);
          ctx.lineTo(sh.x + sh.w, sh.y + sh.h);
          ctx.lineTo(sh.x, sh.y + sh.h);
          ctx.lineTo(sh.x + indent, sh.y + sh.h / 2);
          ctx.closePath();
          ctx.fill();
          if (sh.stroke) ctx.stroke();
          break;
        }

        case "icon": {
          ctx.save();
          ctx.font = `${sh.fontSize || 48}px "Segoe UI Emoji", Arial, sans-serif`;
          ctx.textAlign = "center";
          ctx.textBaseline = "middle";
          ctx.fillText(sh.iconName || sh.text || "⭐", sh.x + sh.w / 2, sh.y + sh.h / 2);
          ctx.restore();
          break;
        }

        case "wordArt": {
          ctx.save();
          ctx.font = `bold ${sh.fontSize || 44}px ${sh.fontFamily || "Impact, sans-serif"}`;
          ctx.textAlign = "center";
          ctx.textBaseline = "middle";
          ctx.fillStyle = sh.textColor || "#2563eb";
          ctx.fillText(sh.text || "WordArt", sh.x + sh.w / 2, sh.y + sh.h / 2);
          if (sh.stroke) {
            ctx.strokeStyle = sh.stroke;
            ctx.lineWidth = sh.strokeWidth || 1.5;
            ctx.strokeText(sh.text || "WordArt", sh.x + sh.w / 2, sh.y + sh.h / 2);
          }
          ctx.restore();
          break;
        }

        case "smartArt": {
          const type = sh.smartArtType || "process";
          const nodes = sh.smartArtNodes || [];
          if (nodes.length > 0) {
            const count = nodes.length;
            if (type === "process") {
              const nodeW = (sh.w - (count - 1) * 20) / count;
              nodes.forEach((n, idx) => {
                const nx = sh.x + idx * (nodeW + 20);
                ctx.fillStyle = n.color || "#2563eb";
                ctx.fillRect(nx, sh.y + 20, nodeW, sh.h - 40);
                ctx.fillStyle = "#ffffff";
                ctx.font = "bold 14px Aptos, sans-serif";
                ctx.textAlign = "center";
                ctx.textBaseline = "middle";
                ctx.fillText(n.label, nx + nodeW / 2, sh.y + sh.h / 2);
                if (idx < count - 1) {
                  ctx.fillStyle = "#94a3b8";
                  ctx.fillText("➔", nx + nodeW + 10, sh.y + sh.h / 2);
                }
              });
            } else if (type === "pyramid") {
              const tierH = (sh.h - 20) / count;
              nodes.forEach((n, idx) => {
                const pct = (idx + 1) / count;
                const tierW = sh.w * pct;
                const tx = sh.x + (sh.w - tierW) / 2;
                const ty = sh.y + 10 + idx * tierH;
                ctx.fillStyle = n.color || "#2563eb";
                ctx.fillRect(tx, ty, tierW, tierH - 6);
                ctx.fillStyle = "#ffffff";
                ctx.font = "bold 13px Aptos, sans-serif";
                ctx.textAlign = "center";
                ctx.textBaseline = "middle";
                ctx.fillText(n.label, sh.x + sh.w / 2, ty + tierH / 2 - 3);
              });
            } else {
              // cycle or hierarchy
              const nodeW = (sh.w - 40) / Math.max(1, count);
              nodes.forEach((n, idx) => {
                const nx = sh.x + 20 + idx * nodeW;
                ctx.fillStyle = n.color || "#2563eb";
                ctx.beginPath();
                ctx.arc(nx + nodeW / 2, sh.y + sh.h / 2, Math.min(nodeW / 2 - 6, (sh.h - 40) / 2), 0, Math.PI * 2);
                ctx.fill();
                ctx.fillStyle = "#ffffff";
                ctx.font = "bold 12px Aptos, sans-serif";
                ctx.textAlign = "center";
                ctx.textBaseline = "middle";
                ctx.fillText(n.label, nx + nodeW / 2, sh.y + sh.h / 2);
              });
            }
          }
          break;
        }

        case "chart": {
          ctx.fillStyle = sh.fill || "#ffffff";
          ctx.fillRect(sh.x, sh.y, sh.w, sh.h);
          if (sh.stroke) ctx.strokeRect(sh.x, sh.y, sh.w, sh.h);

          const cData = sh.chartData || {
            categories: ["Q1", "Q2", "Q3", "Q4"],
            series: [{ name: "Sales", values: [40, 60, 80, 100] }],
          };
          const colors = ["#2563eb", "#10b981", "#f59e0b", "#ec4899"];
          const padL = 50;
          const padB = 40;
          const padT = 30;
          const padR = 20;
          const plotW = sh.w - padL - padR;
          const plotH = sh.h - padT - padB;

          // Axes
          ctx.strokeStyle = "#94a3b8";
          ctx.lineWidth = 1;
          ctx.beginPath();
          ctx.moveTo(sh.x + padL, sh.y + padT);
          ctx.lineTo(sh.x + padL, sh.y + padT + plotH);
          ctx.lineTo(sh.x + padL + plotW, sh.y + padT + plotH);
          ctx.stroke();

          // Title
          ctx.fillStyle = "#1e293b";
          ctx.font = "bold 13px Aptos, sans-serif";
          ctx.textAlign = "center";
          ctx.textBaseline = "top";
          ctx.fillText(`Chart: ${(sh.chartType || "bar").toUpperCase()}`, sh.x + sh.w / 2, sh.y + 8);

          const cats = cData.categories;
          const numCats = cats.length;
          const catW = plotW / numCats;

          if (sh.chartType === "line") {
            cData.series.forEach((s, sIdx) => {
              ctx.strokeStyle = colors[sIdx % colors.length];
              ctx.lineWidth = 2.5;
              ctx.beginPath();
              s.values.forEach((v, cIdx) => {
                const px = sh.x + padL + cIdx * catW + catW / 2;
                const py = sh.y + padT + plotH - (v / 120) * plotH;
                if (cIdx === 0) ctx.moveTo(px, py);
                else ctx.lineTo(px, py);
              });
              ctx.stroke();
            });
          } else if (sh.chartType === "pie") {
            const cx = sh.x + sh.w / 2;
            const cy = sh.y + padT + plotH / 2;
            const r = Math.min(plotW, plotH) / 2 - 10;
            const values = cData.series[0]?.values || [25, 25, 25, 25];
            const sum = values.reduce((a, b) => a + b, 0) || 1;
            let startAng = -Math.PI / 2;
            values.forEach((v, idx) => {
              const slice = (v / sum) * 2 * Math.PI;
              ctx.fillStyle = colors[idx % colors.length];
              ctx.beginPath();
              ctx.moveTo(cx, cy);
              ctx.arc(cx, cy, r, startAng, startAng + slice);
              ctx.closePath();
              ctx.fill();
              startAng += slice;
            });
          } else {
            // bar
            const barW = Math.min(24, (catW - 12) / Math.max(1, cData.series.length));
            cData.series.forEach((s, sIdx) => {
              ctx.fillStyle = colors[sIdx % colors.length];
              s.values.forEach((v, cIdx) => {
                const bH = (v / 120) * plotH;
                const bx = sh.x + padL + cIdx * catW + 6 + sIdx * barW;
                const by = sh.y + padT + plotH - bH;
                ctx.fillRect(bx, by, barW, bH);
              });
            });
          }

          // Category labels
          ctx.fillStyle = "#64748b";
          ctx.font = "11px Aptos, sans-serif";
          ctx.textAlign = "center";
          ctx.textBaseline = "top";
          cats.forEach((cat, idx) => {
            ctx.fillText(cat, sh.x + padL + idx * catW + catW / 2, sh.y + padT + plotH + 8);
          });
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
            const totalTextH = lines.length * lineHeight;
            let startY = sh.y + 8;
            if (sh.verticalAlign === "middle") {
              startY = sh.y + Math.max(0, (sh.h - totalTextH) / 2);
            } else if (sh.verticalAlign === "bottom") {
              startY = sh.y + Math.max(0, sh.h - totalTextH - 8);
            }
            lines.forEach((line, i) => {
              const posX = align === "center" ? sh.x + sh.w / 2 : align === "right" ? sh.x + sh.w - 8 : sh.x + 8;
              ctx.fillText(line, posX, startY + i * lineHeight);
              if (sh.underline) {
                const metrics = ctx.measureText ? ctx.measureText(line) : { width: 50 };
                const underY = startY + i * lineHeight + (sh.fontSize || 28);
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

    // 5. Render Gridlines Overlay (View Tab)
    if (this.state.showGridlines) {
      ctx.save();
      ctx.strokeStyle = "rgba(148, 163, 184, 0.25)";
      ctx.lineWidth = 1;
      const step = 20;
      for (let x = step; x < this.slideWidth; x += step) {
        ctx.beginPath();
        ctx.moveTo(x, 0);
        ctx.lineTo(x, this.slideHeight);
        ctx.stroke();
      }
      for (let y = step; y < this.slideHeight; y += step) {
        ctx.beginPath();
        ctx.moveTo(0, y);
        ctx.lineTo(this.slideWidth, y);
        ctx.stroke();
      }
      ctx.restore();
    }

    // 6. Render Center Guides (View Tab)
    if (this.state.showGuides) {
      ctx.save();
      ctx.strokeStyle = "rgba(239, 68, 68, 0.6)";
      ctx.lineWidth = 1.5;
      if (ctx.setLineDash) ctx.setLineDash([6, 6]);
      // Vertical center guide
      ctx.beginPath();
      ctx.moveTo(this.slideWidth / 2, 0);
      ctx.lineTo(this.slideWidth / 2, this.slideHeight);
      ctx.stroke();
      // Horizontal center guide
      ctx.beginPath();
      ctx.moveTo(0, this.slideHeight / 2);
      ctx.lineTo(this.slideWidth, this.slideHeight / 2);
      ctx.stroke();
      if (ctx.setLineDash) ctx.setLineDash([]);
      ctx.restore();
    }

    // 7. Render Review Comment Pins
    const comments = this.state.getComments();
    if (comments.length > 0) {
      ctx.save();
      comments.forEach((comm, idx) => {
        const cx = comm.x ?? 80 + idx * 40;
        const cy = comm.y ?? 40;
        ctx.fillStyle = "#f59e0b";
        ctx.beginPath();
        ctx.arc(cx, cy, 12, 0, Math.PI * 2);
        ctx.fill();
        ctx.fillStyle = "#ffffff";
        ctx.font = "bold 11px sans-serif";
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillText(String(idx + 1), cx, cy);
      });
      ctx.restore();
    }

    // 8. Color / Grayscale View Mode Filter
    if (this.wrapper) {
      if (this.state.colorMode === "grayscale") {
        this.wrapper.style.filter = "grayscale(100%)";
      } else if (this.state.colorMode === "bw") {
        this.wrapper.style.filter = "grayscale(100%) contrast(200%)";
      } else {
        this.wrapper.style.filter = "none";
      }
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
