export type PresetShapeKind =
  | "text"
  | "rect"
  | "roundRect"
  | "ellipse"
  | "triangle"
  | "arrow"
  | "star"
  | "diamond"
  | "callout";

export interface SlideShape {
  id: string;
  type: PresetShapeKind;
  x: number;
  y: number;
  w: number;
  h: number;
  fill: string;
  stroke?: string;
  strokeWidth?: number;
  rotation?: number; // degrees [0, 360)
  text?: string;
  textColor?: string;
  fontSize?: number;
  zIndex?: number;
}

export interface SlideData {
  id: string;
  title: string;
  shapes: SlideShape[];
}

export class SlidesState {
  slides: SlideData[] = [
    {
      id: "s1",
      title: "Title Slide",
      shapes: [
        {
          id: "sh1",
          type: "text",
          x: 100,
          y: 120,
          w: 760,
          h: 70,
          fill: "transparent",
          text: "OpenOfficeRS Presentation",
          textColor: "#1e293b",
          fontSize: 38,
          rotation: 0,
        },
        {
          id: "sh2",
          type: "rect",
          x: 100,
          y: 200,
          w: 360,
          h: 6,
          fill: "#c43e1c",
          rotation: 0,
        },
        {
          id: "sh3",
          type: "roundRect",
          x: 100,
          y: 240,
          w: 220,
          h: 120,
          fill: "#2563eb",
          stroke: "#1d4ed8",
          strokeWidth: 2,
          rotation: 0,
        },
        {
          id: "sh4",
          type: "ellipse",
          x: 360,
          y: 240,
          w: 140,
          h: 120,
          fill: "#10b981",
          stroke: "#059669",
          strokeWidth: 2,
          rotation: 0,
        },
        {
          id: "sh5",
          type: "star",
          x: 540,
          y: 240,
          w: 130,
          h: 120,
          fill: "#f59e0b",
          stroke: "#d97706",
          strokeWidth: 2,
          rotation: 0,
        },
      ],
    },
    { id: "s2", title: "Problem Statement", shapes: [] },
    { id: "s3", title: "Solution & Demo", shapes: [] },
  ];

  currentSlideIndex = 0;
  selectedShapeId: string | null = null;
  private undoStack: SlideData[][] = [];
  private redoStack: SlideData[][] = [];
  private nextShapeCounter = 10;

  public selectSlide(idx: number) {
    if (idx >= 0 && idx < this.slides.length) {
      this.currentSlideIndex = idx;
      this.selectedShapeId = null;
    }
  }

  public selectShape(id: string | null) {
    this.selectedShapeId = id;
  }

  public getSelectedShape(): SlideShape | null {
    if (!this.selectedShapeId) return null;
    const slide = this.slides[this.currentSlideIndex];
    return slide?.shapes.find((s) => s.id === this.selectedShapeId) || null;
  }

  public commitSnapshot() {
    // Deep clone slides state for undo history
    const snapshot = JSON.parse(JSON.stringify(this.slides)) as SlideData[];
    this.undoStack.push(snapshot);
    this.redoStack = [];
    if (this.undoStack.length > 50) {
      this.undoStack.shift();
    }
  }

  public canUndo(): boolean {
    return this.undoStack.length > 0;
  }

  public canRedo(): boolean {
    return this.redoStack.length > 0;
  }

  public undo(): boolean {
    if (this.undoStack.length === 0) return false;
    const current = JSON.parse(JSON.stringify(this.slides)) as SlideData[];
    this.redoStack.push(current);
    const prev = this.undoStack.pop();
    if (prev) {
      this.slides = prev;
      return true;
    }
    return false;
  }

  public redo(): boolean {
    if (this.redoStack.length === 0) return false;
    const current = JSON.parse(JSON.stringify(this.slides)) as SlideData[];
    this.undoStack.push(current);
    const next = this.redoStack.pop();
    if (next) {
      this.slides = next;
      return true;
    }
    return false;
  }

  public addShape(
    type: PresetShapeKind,
    x = 200,
    y = 150,
    w = 160,
    h = 100,
    fill = "#2563eb"
  ): SlideShape {
    this.commitSnapshot();
    const slide = this.slides[this.currentSlideIndex];
    if (!slide) {
      throw new Error("No active slide");
    }

    const newShape: SlideShape = {
      id: `sh_${++this.nextShapeCounter}`,
      type,
      x,
      y,
      w: type === "text" ? 280 : w,
      h: type === "text" ? 60 : h,
      fill: type === "text" ? "transparent" : fill,
      stroke: type === "text" ? undefined : "#1e40af",
      strokeWidth: type === "text" ? undefined : 2,
      rotation: 0,
      text: type === "text" ? "Click to add text" : undefined,
      textColor: "#1e293b",
      fontSize: type === "text" ? 24 : 16,
    };

    slide.shapes.push(newShape);
    this.selectedShapeId = newShape.id;
    return newShape;
  }

  public deleteSelectedShape(): boolean {
    if (!this.selectedShapeId) return false;
    this.commitSnapshot();
    const slide = this.slides[this.currentSlideIndex];
    if (!slide) return false;

    const idx = slide.shapes.findIndex((s) => s.id === this.selectedShapeId);
    if (idx !== -1) {
      slide.shapes.splice(idx, 1);
      this.selectedShapeId = null;
      return true;
    }
    return false;
  }

  public duplicateSelectedShape(): SlideShape | null {
    const orig = this.getSelectedShape();
    if (!orig) return null;
    this.commitSnapshot();
    const slide = this.slides[this.currentSlideIndex];
    if (!slide) return null;

    const dup: SlideShape = {
      ...JSON.parse(JSON.stringify(orig)),
      id: `sh_${++this.nextShapeCounter}`,
      x: orig.x + 20,
      y: orig.y + 20,
    };

    slide.shapes.push(dup);
    this.selectedShapeId = dup.id;
    return dup;
  }

  public bringToFront(): boolean {
    const orig = this.getSelectedShape();
    if (!orig) return false;
    const slide = this.slides[this.currentSlideIndex];
    if (!slide) return false;

    const idx = slide.shapes.findIndex((s) => s.id === orig.id);
    if (idx !== -1 && idx < slide.shapes.length - 1) {
      this.commitSnapshot();
      const [item] = slide.shapes.splice(idx, 1);
      slide.shapes.push(item);
      return true;
    }
    return false;
  }

  public sendToBack(): boolean {
    const orig = this.getSelectedShape();
    if (!orig) return false;
    const slide = this.slides[this.currentSlideIndex];
    if (!slide) return false;

    const idx = slide.shapes.findIndex((s) => s.id === orig.id);
    if (idx > 0) {
      this.commitSnapshot();
      const [item] = slide.shapes.splice(idx, 1);
      slide.shapes.unshift(item);
      return true;
    }
    return false;
  }

  public bringForward(): boolean {
    const orig = this.getSelectedShape();
    if (!orig) return false;
    const slide = this.slides[this.currentSlideIndex];
    if (!slide) return false;

    const idx = slide.shapes.findIndex((s) => s.id === orig.id);
    if (idx !== -1 && idx < slide.shapes.length - 1) {
      this.commitSnapshot();
      const temp = slide.shapes[idx];
      slide.shapes[idx] = slide.shapes[idx + 1];
      slide.shapes[idx + 1] = temp;
      return true;
    }
    return false;
  }

  public sendBackward(): boolean {
    const orig = this.getSelectedShape();
    if (!orig) return false;
    const slide = this.slides[this.currentSlideIndex];
    if (!slide) return false;

    const idx = slide.shapes.findIndex((s) => s.id === orig.id);
    if (idx > 0) {
      this.commitSnapshot();
      const temp = slide.shapes[idx];
      slide.shapes[idx] = slide.shapes[idx - 1];
      slide.shapes[idx - 1] = temp;
      return true;
    }
    return false;
  }

  public setShapeFill(color: string): boolean {
    const shape = this.getSelectedShape();
    if (!shape) return false;
    this.commitSnapshot();
    shape.fill = color;
    return true;
  }

  public setShapeStroke(color: string, width = 2): boolean {
    const shape = this.getSelectedShape();
    if (!shape) return false;
    this.commitSnapshot();
    shape.stroke = color;
    shape.strokeWidth = width;
    return true;
  }

  public setShapeText(text: string): boolean {
    const shape = this.getSelectedShape();
    if (!shape) return false;
    this.commitSnapshot();
    shape.text = text;
    return true;
  }

  public execCommand(action: string, payload?: unknown): boolean {
    switch (action) {
      case "InsertShape":
        this.addShape(
          typeof payload === "string" ? (payload as PresetShapeKind) : "rect"
        );
        return true;
      case "InsertTextBox":
        this.addShape("text");
        return true;
      case "DeleteShape":
        return this.deleteSelectedShape();
      case "DuplicateShape":
        return this.duplicateSelectedShape() !== null;
      case "BringToFront":
        return this.bringToFront();
      case "SendToBack":
        return this.sendToBack();
      case "BringForward":
        return this.bringForward();
      case "SendBackward":
        return this.sendBackward();
      case "SetFillColor":
        if (typeof payload === "string") return this.setShapeFill(payload);
        return false;
      case "SetStrokeColor":
        if (typeof payload === "string") return this.setShapeStroke(payload);
        return false;
      case "Undo":
        return this.undo();
      case "Redo":
        return this.redo();
      default:
        return false;
    }
  }
}

export const slidesState = new SlidesState();
