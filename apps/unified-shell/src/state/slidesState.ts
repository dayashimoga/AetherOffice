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

export type ThemePresetName =
  | "OfficeDefault"
  | "DarkMode"
  | "VibrantCoral"
  | "HighContrast";

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
  isHidden?: boolean;
  transition?: string;
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
    {
      id: "s2",
      title: "Problem Statement",
      shapes: [
        {
          id: "sh2_1",
          type: "text",
          x: 80,
          y: 80,
          w: 800,
          h: 60,
          fill: "transparent",
          text: "Market Need & Clean-Room Architecture",
          textColor: "#1e293b",
          fontSize: 32,
        },
        {
          id: "sh2_2",
          type: "roundRect",
          x: 80,
          y: 180,
          w: 380,
          h: 220,
          fill: "#f8fafc",
          stroke: "#cbd5e1",
          strokeWidth: 1.5,
          text: "Legacy suites suffer from high idle memory (>180MB) and sluggish 60Hz viewport redraws.",
          textColor: "#334155",
          fontSize: 18,
        },
        {
          id: "sh2_3",
          type: "roundRect",
          x: 500,
          y: 180,
          w: 380,
          h: 220,
          fill: "#f0fdf4",
          stroke: "#86efac",
          strokeWidth: 1.5,
          text: "OpenOfficeRS delivers sub-millisecond Piece-Tree Word & 120 FPS WebGPU vector slides.",
          textColor: "#166534",
          fontSize: 18,
        },
      ],
    },
    {
      id: "s3",
      title: "Solution & Demo",
      shapes: [
        {
          id: "sh3_1",
          type: "text",
          x: 80,
          y: 80,
          w: 800,
          h: 60,
          fill: "transparent",
          text: "Live Demo & Architecture Validation",
          textColor: "#1e293b",
          fontSize: 32,
        },
        {
          id: "sh3_2",
          type: "star",
          x: 120,
          y: 180,
          w: 200,
          h: 200,
          fill: "#c43e1c",
          stroke: "#991b1b",
          strokeWidth: 2,
        },
      ],
    },
  ];

  currentSlideIndex = 0;
  selectedShapeId: string | null = null;
  currentTheme: ThemePresetName = "OfficeDefault";

  // Slide Show State
  isShowActive = false;
  isLaserActive = false;
  laserPos: { x: number; y: number } | null = null;

  private undoStack: SlideData[][] = [];
  private redoStack: SlideData[][] = [];
  private nextShapeCounter = 20;
  private nextSlideCounter = 10;

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
      if (this.currentSlideIndex >= this.slides.length) {
        this.currentSlideIndex = this.slides.length - 1;
      }
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
      if (this.currentSlideIndex >= this.slides.length) {
        this.currentSlideIndex = this.slides.length - 1;
      }
      return true;
    }
    return false;
  }

  // --- Slide Operations ---

  public addSlide(title = "New Slide"): SlideData {
    this.commitSnapshot();
    const newSlide: SlideData = {
      id: `s_${++this.nextSlideCounter}`,
      title,
      shapes: [
        {
          id: `sh_${++this.nextShapeCounter}`,
          type: "text",
          x: 100,
          y: 100,
          w: 760,
          h: 60,
          fill: "transparent",
          text: title,
          textColor: this.currentTheme === "DarkMode" || this.currentTheme === "HighContrast" ? "#ffffff" : "#1e293b",
          fontSize: 32,
        },
      ],
      isHidden: false,
    };
    this.slides.push(newSlide);
    this.currentSlideIndex = this.slides.length - 1;
    this.selectedShapeId = null;
    return newSlide;
  }

  public duplicateSlide(idx = this.currentSlideIndex): SlideData | null {
    if (idx < 0 || idx >= this.slides.length) return null;
    this.commitSnapshot();
    const orig = this.slides[idx];
    const dup: SlideData = {
      ...JSON.parse(JSON.stringify(orig)),
      id: `s_${++this.nextSlideCounter}`,
      title: `${orig.title} (Copy)`,
    };
    this.slides.splice(idx + 1, 0, dup);
    this.currentSlideIndex = idx + 1;
    this.selectedShapeId = null;
    return dup;
  }

  public deleteSlide(idx = this.currentSlideIndex): boolean {
    if (this.slides.length <= 1 || idx < 0 || idx >= this.slides.length) return false;
    this.commitSnapshot();
    this.slides.splice(idx, 1);
    if (this.currentSlideIndex >= this.slides.length) {
      this.currentSlideIndex = this.slides.length - 1;
    }
    this.selectedShapeId = null;
    return true;
  }

  public toggleHideSlide(idx = this.currentSlideIndex): boolean {
    if (idx < 0 || idx >= this.slides.length) return false;
    this.commitSnapshot();
    this.slides[idx].isHidden = !this.slides[idx].isHidden;
    return true;
  }

  public reorderSlide(fromIdx: number, toIdx: number): boolean {
    if (
      fromIdx < 0 ||
      fromIdx >= this.slides.length ||
      toIdx < 0 ||
      toIdx >= this.slides.length ||
      fromIdx === toIdx
    ) {
      return false;
    }
    this.commitSnapshot();
    const [item] = this.slides.splice(fromIdx, 1);
    this.slides.splice(toIdx, 0, item);
    this.currentSlideIndex = toIdx;
    return true;
  }

  public moveSlideUp(idx = this.currentSlideIndex): boolean {
    if (idx > 0) return this.reorderSlide(idx, idx - 1);
    return false;
  }

  public moveSlideDown(idx = this.currentSlideIndex): boolean {
    if (idx + 1 < this.slides.length) return this.reorderSlide(idx, idx + 1);
    return false;
  }

  // --- Master Theme Switcher ---

  public setTheme(theme: ThemePresetName) {
    this.commitSnapshot();
    this.currentTheme = theme;

    let textColor = "#1e293b";
    let primaryAccent = "#2563eb";
    let secondaryAccent = "#10b981";

    if (theme === "DarkMode") {
      textColor = "#f1f1f1";
      primaryAccent = "#6366f1";
      secondaryAccent = "#06b6d4";
    } else if (theme === "VibrantCoral") {
      textColor = "#18181b";
      primaryAccent = "#c43e1c";
      secondaryAccent = "#0d9488";
    } else if (theme === "HighContrast") {
      textColor = "#ffffff";
      primaryAccent = "#ffff00";
      secondaryAccent = "#00ffff";
    }

    // Apply theme styling across all slides
    this.slides.forEach((slide) => {
      slide.shapes.forEach((sh) => {
        if (sh.type === "text") {
          sh.textColor = textColor;
        } else if (sh.id.endsWith("2") || sh.id.endsWith("3")) {
          sh.fill = primaryAccent;
        } else if (sh.id.endsWith("4") || sh.id.endsWith("5")) {
          sh.fill = secondaryAccent;
        }
      });
    });
  }

  // --- Slide Show Mode ---

  public startShow(fromBeginning = true) {
    this.isShowActive = true;
    this.isLaserActive = false;
    this.laserPos = null;
    this.currentSlideIndex = fromBeginning ? 0 : this.currentSlideIndex;
  }

  public exitShow() {
    this.isShowActive = false;
    this.isLaserActive = false;
    this.laserPos = null;
  }

  public nextSlide(): boolean {
    if (!this.isShowActive) return false;
    let nextIdx = this.currentSlideIndex + 1;
    // Skip hidden slides during presentation show
    while (nextIdx < this.slides.length && this.slides[nextIdx].isHidden) {
      nextIdx++;
    }
    if (nextIdx < this.slides.length) {
      this.currentSlideIndex = nextIdx;
      this.laserPos = null;
      return true;
    }
    return false;
  }

  public prevSlide(): boolean {
    if (!this.isShowActive) return false;
    let prevIdx = this.currentSlideIndex - 1;
    while (prevIdx >= 0 && this.slides[prevIdx].isHidden) {
      prevIdx--;
    }
    if (prevIdx >= 0) {
      this.currentSlideIndex = prevIdx;
      this.laserPos = null;
      return true;
    }
    return false;
  }

  public toggleLaser() {
    this.isLaserActive = !this.isLaserActive;
    if (!this.isLaserActive) {
      this.laserPos = null;
    }
  }

  public setLaserPos(x: number, y: number) {
    if (this.isLaserActive) {
      this.laserPos = { x, y };
    }
  }

  // --- Shape CRUD (from 4.3.1) ---

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
    if (!slide) throw new Error("No active slide");

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
      textColor: this.currentTheme === "DarkMode" || this.currentTheme === "HighContrast" ? "#ffffff" : "#1e293b",
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
      case "SetTheme":
        if (typeof payload === "string") {
          this.setTheme(payload as ThemePresetName);
          return true;
        }
        return false;
      case "NewSlide":
        this.addSlide(typeof payload === "string" ? payload : "New Slide");
        return true;
      case "DuplicateSlide":
        return this.duplicateSlide() !== null;
      case "DeleteSlide":
        return this.deleteSlide();
      case "HideSlide":
        return this.toggleHideSlide();
      case "StartShowBeginning":
        this.startShow(true);
        return true;
      case "StartShowCurrent":
        this.startShow(false);
        return true;
      case "NextSlide":
        return this.nextSlide();
      case "PrevSlide":
      case "PreviousSlide":
        return this.prevSlide();
      case "ExitShow":
        this.exitShow();
        return true;
      case "ToggleLaser":
        this.toggleLaser();
        return true;
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
