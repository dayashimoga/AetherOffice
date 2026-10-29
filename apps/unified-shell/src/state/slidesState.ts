export type PresetShapeKind =
  | "text"
  | "rect"
  | "roundRect"
  | "ellipse"
  | "triangle"
  | "arrow"
  | "star"
  | "diamond"
  | "callout"
  | "heart"
  | "image"
  | "table";

export type ThemePresetName =
  | "OfficeDefault"
  | "DarkMode"
  | "VibrantCoral"
  | "HighContrast"
  | "EmeraldForest"
  | "OceanBreeze"
  | "SunsetGlow"
  | "SlateMinimal";

export type SlideLayoutType =
  | "title"
  | "titleAndContent"
  | "twoContent"
  | "blank";

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
  fontFamily?: string;
  bold?: boolean;
  italic?: boolean;
  underline?: boolean;
  imageUrl?: string;
  imageAlt?: string;
  tableData?: string[][];
  tableHeaders?: string[];
  zIndex?: number;
}

export interface SlideData {
  id: string;
  title: string;
  shapes: SlideShape[];
  isHidden?: boolean;
  transition?: string;
  notes?: string;
  background?: string;
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

  // Slide Dimension State
  slideAspectRatio: "16:9" | "4:3" = "16:9";
  slideWidth = 960;
  slideHeight = 540;

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

  public addSlideWithLayout(layout: SlideLayoutType, title = "New Slide"): SlideData {
    this.commitSnapshot();
    const isDark = this.currentTheme === "DarkMode" || this.currentTheme === "HighContrast";
    const textColor = isDark ? "#ffffff" : "#1e293b";
    const subColor = isDark ? "#94a3b8" : "#64748b";

    let shapes: SlideShape[] = [];

    switch (layout) {
      case "title":
        shapes = [
          {
            id: `sh_${++this.nextShapeCounter}`,
            type: "text",
            x: 100,
            y: 150,
            w: 760,
            h: 80,
            fill: "transparent",
            text: title,
            textColor,
            fontSize: 40,
            bold: true,
            rotation: 0,
          },
          {
            id: `sh_${++this.nextShapeCounter}`,
            type: "text",
            x: 100,
            y: 260,
            w: 760,
            h: 60,
            fill: "transparent",
            text: "Click to add subtitle",
            textColor: subColor,
            fontSize: 22,
            rotation: 0,
          },
        ];
        break;

      case "titleAndContent":
        shapes = [
          {
            id: `sh_${++this.nextShapeCounter}`,
            type: "text",
            x: 80,
            y: 40,
            w: 800,
            h: 60,
            fill: "transparent",
            text: title,
            textColor,
            fontSize: 32,
            bold: true,
            rotation: 0,
          },
          {
            id: `sh_${++this.nextShapeCounter}`,
            type: "text",
            x: 80,
            y: 120,
            w: 800,
            h: 360,
            fill: "transparent",
            text: "• First bullet point\n• Second bullet point\n• Third bullet point",
            textColor,
            fontSize: 20,
            rotation: 0,
          },
        ];
        break;

      case "twoContent":
        shapes = [
          {
            id: `sh_${++this.nextShapeCounter}`,
            type: "text",
            x: 80,
            y: 40,
            w: 800,
            h: 60,
            fill: "transparent",
            text: title,
            textColor,
            fontSize: 32,
            bold: true,
            rotation: 0,
          },
          {
            id: `sh_${++this.nextShapeCounter}`,
            type: "text",
            x: 80,
            y: 120,
            w: 380,
            h: 360,
            fill: "transparent",
            text: "• Left column point 1\n• Left column point 2",
            textColor,
            fontSize: 18,
            rotation: 0,
          },
          {
            id: `sh_${++this.nextShapeCounter}`,
            type: "text",
            x: 500,
            y: 120,
            w: 380,
            h: 360,
            fill: "transparent",
            text: "• Right column point 1\n• Right column point 2",
            textColor,
            fontSize: 18,
            rotation: 0,
          },
        ];
        break;

      case "blank":
        shapes = [];
        break;
    }

    const newSlide: SlideData = {
      id: `s_${++this.nextSlideCounter}`,
      title,
      shapes,
      isHidden: false,
    };
    this.slides.push(newSlide);
    this.currentSlideIndex = this.slides.length - 1;
    this.selectedShapeId = null;
    return newSlide;
  }

  public applyLayout(layout: SlideLayoutType, idx = this.currentSlideIndex): boolean {
    if (idx < 0 || idx >= this.slides.length) return false;
    this.commitSnapshot();
    const slide = this.slides[idx];
    const existingTitle =
      slide.shapes.find((s) => s.type === "text" && s.text)?.text || slide.title;

    const isDark = this.currentTheme === "DarkMode" || this.currentTheme === "HighContrast";
    const textColor = isDark ? "#ffffff" : "#1e293b";
    const subColor = isDark ? "#94a3b8" : "#64748b";

    let shapes: SlideShape[] = [];
    switch (layout) {
      case "title":
        shapes = [
          {
            id: `sh_${++this.nextShapeCounter}`,
            type: "text",
            x: 100,
            y: 150,
            w: 760,
            h: 80,
            fill: "transparent",
            text: existingTitle,
            textColor,
            fontSize: 40,
            bold: true,
            rotation: 0,
          },
          {
            id: `sh_${++this.nextShapeCounter}`,
            type: "text",
            x: 100,
            y: 260,
            w: 760,
            h: 60,
            fill: "transparent",
            text: "Click to add subtitle",
            textColor: subColor,
            fontSize: 22,
            rotation: 0,
          },
        ];
        break;

      case "titleAndContent":
        shapes = [
          {
            id: `sh_${++this.nextShapeCounter}`,
            type: "text",
            x: 80,
            y: 40,
            w: 800,
            h: 60,
            fill: "transparent",
            text: existingTitle,
            textColor,
            fontSize: 32,
            bold: true,
            rotation: 0,
          },
          {
            id: `sh_${++this.nextShapeCounter}`,
            type: "text",
            x: 80,
            y: 120,
            w: 800,
            h: 360,
            fill: "transparent",
            text: "• First bullet point\n• Second bullet point\n• Third bullet point",
            textColor,
            fontSize: 20,
            rotation: 0,
          },
        ];
        break;

      case "twoContent":
        shapes = [
          {
            id: `sh_${++this.nextShapeCounter}`,
            type: "text",
            x: 80,
            y: 40,
            w: 800,
            h: 60,
            fill: "transparent",
            text: existingTitle,
            textColor,
            fontSize: 32,
            bold: true,
            rotation: 0,
          },
          {
            id: `sh_${++this.nextShapeCounter}`,
            type: "text",
            x: 80,
            y: 120,
            w: 380,
            h: 360,
            fill: "transparent",
            text: "• Left column point 1\n• Left column point 2",
            textColor,
            fontSize: 18,
            rotation: 0,
          },
          {
            id: `sh_${++this.nextShapeCounter}`,
            type: "text",
            x: 500,
            y: 120,
            w: 380,
            h: 360,
            fill: "transparent",
            text: "• Right column point 1\n• Right column point 2",
            textColor,
            fontSize: 18,
            rotation: 0,
          },
        ];
        break;

      case "blank":
        shapes = [];
        break;
    }

    slide.shapes = shapes;
    this.selectedShapeId = null;
    return true;
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
    let bg = "#ffffff";
    let fontFamily = "Aptos, Calibri, sans-serif";

    switch (theme) {
      case "DarkMode":
        textColor = "#f8fafc";
        primaryAccent = "#6366f1";
        secondaryAccent = "#06b6d4";
        bg = "#0f172a";
        fontFamily = "Segoe UI, sans-serif";
        break;
      case "VibrantCoral":
        textColor = "#18181b";
        primaryAccent = "#c43e1c";
        secondaryAccent = "#0d9488";
        bg = "#fff7ed";
        fontFamily = "Calibri, sans-serif";
        break;
      case "HighContrast":
        textColor = "#ffffff";
        primaryAccent = "#ffff00";
        secondaryAccent = "#00ffff";
        bg = "#000000";
        fontFamily = "Segoe UI, sans-serif";
        break;
      case "EmeraldForest":
        textColor = "#064e3b";
        primaryAccent = "#059669";
        secondaryAccent = "#d97706";
        bg = "#f0fdf4";
        fontFamily = "Georgia, serif";
        break;
      case "OceanBreeze":
        textColor = "#0c4a6e";
        primaryAccent = "#0284c7";
        secondaryAccent = "#14b8a6";
        bg = "#f0f9ff";
        fontFamily = "Arial, sans-serif";
        break;
      case "SunsetGlow":
        textColor = "#701a75";
        primaryAccent = "#e11d48";
        secondaryAccent = "#f59e0b";
        bg = "#fdf2f8";
        fontFamily = "Century Gothic, sans-serif";
        break;
      case "SlateMinimal":
        textColor = "#334155";
        primaryAccent = "#475569";
        secondaryAccent = "#3b82f6";
        bg = "#f8fafc";
        fontFamily = "Segoe UI, sans-serif";
        break;
      case "OfficeDefault":
      default:
        textColor = "#1e293b";
        primaryAccent = "#2563eb";
        secondaryAccent = "#10b981";
        bg = "#ffffff";
        fontFamily = "Aptos, Calibri, sans-serif";
        break;
    }

    // Apply theme styling across all slides
    this.slides.forEach((slide) => {
      slide.background = bg;
      slide.shapes.forEach((sh) => {
        if (sh.type === "text") {
          sh.textColor = textColor;
          sh.fontFamily = fontFamily;
        } else if (sh.id.endsWith("2") || sh.id.endsWith("3")) {
          sh.fill = primaryAccent;
        } else if (sh.id.endsWith("4") || sh.id.endsWith("5")) {
          sh.fill = secondaryAccent;
        }
      });
    });
  }

  public setSlideSize(ratio: "16:9" | "4:3") {
    this.commitSnapshot();
    this.slideAspectRatio = ratio;
    if (ratio === "4:3") {
      this.slideWidth = 720;
      this.slideHeight = 540;
    } else {
      this.slideWidth = 960;
      this.slideHeight = 540;
    }
  }

  public setSlideBackground(bg: string, applyToAll = false) {
    this.commitSnapshot();
    if (applyToAll) {
      this.slides.forEach((s) => (s.background = bg));
    } else {
      const slide = this.slides[this.currentSlideIndex];
      if (slide) slide.background = bg;
    }
  }

  public setSlideNotes(notes: string, idx = this.currentSlideIndex): boolean {
    if (idx < 0 || idx >= this.slides.length) return false;
    this.commitSnapshot();
    this.slides[idx].notes = notes;
    return true;
  }

  public getSlideNotes(idx = this.currentSlideIndex): string {
    return this.slides[idx]?.notes || "";
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
      w: type === "text" ? 280 : type === "table" ? 480 : type === "image" ? 280 : w,
      h: type === "text" ? 60 : type === "table" ? 160 : type === "image" ? 180 : h,
      fill:
        type === "text"
          ? "transparent"
          : type === "image"
          ? "#f8fafc"
          : type === "table"
          ? "#ffffff"
          : fill,
      stroke: type === "text" ? undefined : "#1e40af",
      strokeWidth: type === "text" ? undefined : 2,
      rotation: 0,
      text: type === "text" ? "Click to add text" : undefined,
      textColor:
        this.currentTheme === "DarkMode" || this.currentTheme === "HighContrast"
          ? "#ffffff"
          : "#1e293b",
      fontSize: type === "text" ? 24 : 16,
      imageUrl: type === "image" ? "assets/sample_graphic.png" : undefined,
      imageAlt: type === "image" ? "Presentation Graphic" : undefined,
      tableHeaders:
        type === "table" ? ["Column 1", "Column 2", "Column 3"] : undefined,
      tableData:
        type === "table"
          ? [
              ["Data 1", "Data 2", "Data 3"],
              ["Val 1", "Val 2", "Val 3"],
            ]
          : undefined,
    };

    slide.shapes.push(newShape);
    this.selectedShapeId = newShape.id;
    return newShape;
  }

  public insertImage(
    url = "assets/sample_graphic.png",
    x = 200,
    y = 150,
    w = 280,
    h = 180
  ): SlideShape {
    this.commitSnapshot();
    const slide = this.slides[this.currentSlideIndex];
    if (!slide) throw new Error("No active slide");

    const shape: SlideShape = {
      id: `sh_${++this.nextShapeCounter}`,
      type: "image",
      x,
      y,
      w,
      h,
      fill: "#f8fafc",
      stroke: "#94a3b8",
      strokeWidth: 1,
      rotation: 0,
      imageUrl: url,
      imageAlt: "Presentation Graphic",
    };
    slide.shapes.push(shape);
    this.selectedShapeId = shape.id;
    return shape;
  }

  public insertTable(
    rows = 3,
    cols = 3,
    x = 100,
    y = 120,
    w = 540,
    h = 180
  ): SlideShape {
    this.commitSnapshot();
    const slide = this.slides[this.currentSlideIndex];
    if (!slide) throw new Error("No active slide");

    const tableHeaders: string[] = [];
    for (let c = 0; c < cols; c++) {
      tableHeaders.push(`Header ${c + 1}`);
    }
    const tableData: string[][] = [];
    for (let r = 0; r < rows; r++) {
      const row: string[] = [];
      for (let c = 0; c < cols; c++) {
        row.push(`R${r + 1}C${c + 1}`);
      }
      tableData.push(row);
    }

    const shape: SlideShape = {
      id: `sh_${++this.nextShapeCounter}`,
      type: "table",
      x,
      y,
      w,
      h,
      fill: "#ffffff",
      stroke: "#cbd5e1",
      strokeWidth: 1,
      rotation: 0,
      tableHeaders,
      tableData,
    };
    slide.shapes.push(shape);
    this.selectedShapeId = shape.id;
    return shape;
  }

  public alignSelectedShape(
    alignment: "left" | "center" | "right" | "top" | "middle" | "bottom"
  ): boolean {
    const sh = this.getSelectedShape();
    if (!sh) return false;
    this.commitSnapshot();

    switch (alignment) {
      case "left":
        sh.x = 80;
        break;
      case "center":
        sh.x = Math.round((this.slideWidth - sh.w) / 2);
        break;
      case "right":
        sh.x = this.slideWidth - 80 - sh.w;
        break;
      case "top":
        sh.y = 50;
        break;
      case "middle":
        sh.y = Math.round((this.slideHeight - sh.h) / 2);
        break;
      case "bottom":
        sh.y = this.slideHeight - 50 - sh.h;
        break;
    }
    return true;
  }

  public setShapeStrokeWeight(weight: number): boolean {
    const sh = this.getSelectedShape();
    if (!sh) return false;
    this.commitSnapshot();
    sh.strokeWidth = weight;
    return true;
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

  public updateShapeTransform(
    id: string,
    x: number,
    y: number,
    w: number,
    h: number,
    rotation?: number
  ): boolean {
    const slide = this.slides[this.currentSlideIndex];
    if (!slide) return false;
    const sh = slide.shapes.find((s) => s.id === id);
    if (!sh) return false;
    sh.x = x;
    sh.y = y;
    sh.w = w;
    sh.h = h;
    if (rotation !== undefined) {
      sh.rotation = rotation;
    }
    return true;
  }

  public toggleBold(): boolean {
    const sh = this.getSelectedShape();
    if (!sh) return false;
    this.commitSnapshot();
    sh.bold = !sh.bold;
    return true;
  }

  public toggleItalic(): boolean {
    const sh = this.getSelectedShape();
    if (!sh) return false;
    this.commitSnapshot();
    sh.italic = !sh.italic;
    return true;
  }

  public toggleUnderline(): boolean {
    const sh = this.getSelectedShape();
    if (!sh) return false;
    this.commitSnapshot();
    sh.underline = !sh.underline;
    return true;
  }

  public setFontFamily(family: string): boolean {
    const sh = this.getSelectedShape();
    if (!sh) return false;
    this.commitSnapshot();
    sh.fontFamily = family;
    return true;
  }

  public setFontSize(size: number): boolean {
    const sh = this.getSelectedShape();
    if (!sh) return false;
    this.commitSnapshot();
    sh.fontSize = size;
    return true;
  }

  public setFontColor(color: string): boolean {
    const sh = this.getSelectedShape();
    if (!sh) return false;
    this.commitSnapshot();
    sh.textColor = color;
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
      case "SlideLayout":
        if (typeof payload === "string") {
          return this.applyLayout(payload as SlideLayoutType);
        }
        return false;
      case "AddSlideWithLayout":
        if (typeof payload === "string") {
          this.addSlideWithLayout(payload as SlideLayoutType);
          return true;
        }
        return false;
      case "DuplicateSlide":
        return this.duplicateSlide() !== null;
      case "DeleteSlide":
        return this.deleteSlide();
      case "MoveSlideUp":
        return this.moveSlideUp();
      case "MoveSlideDown":
        return this.moveSlideDown();
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
      case "SetShapeOutline":
        if (typeof payload === "string") return this.setShapeStroke(payload);
        return false;
      case "ToggleBold":
        return this.toggleBold();
      case "ToggleItalic":
        return this.toggleItalic();
      case "ToggleUnderline":
        return this.toggleUnderline();
      case "SetFontFamily":
        if (typeof payload === "string") return this.setFontFamily(payload);
        return false;
      case "SetFontSize":
        if (typeof payload === "number") return this.setFontSize(payload);
        return false;
      case "SetFontColor":
        if (typeof payload === "string") return this.setFontColor(payload);
        return false;
      case "SetSlideSize":
        if (payload === "16:9" || payload === "4:3") {
          this.setSlideSize(payload);
          return true;
        }
        return false;
      case "SetSlideBackground":
      case "FormatBackground":
        if (typeof payload === "string") {
          this.setSlideBackground(payload);
          return true;
        }
        return false;
      case "InsertImage":
      case "InsertPicture":
        this.insertImage(typeof payload === "string" ? payload : undefined);
        return true;
      case "InsertTable":
        if (payload && typeof payload === "object") {
          const p = payload as { rows?: number; cols?: number };
          this.insertTable(p.rows || 3, p.cols || 3);
        } else {
          this.insertTable(3, 3);
        }
        return true;
      case "SetOutlineWeight":
      case "SetStrokeWidth":
        if (typeof payload === "number") {
          return this.setShapeStrokeWeight(payload);
        }
        return false;
      case "AlignLeft":
        return this.alignSelectedShape("left");
      case "AlignCenter":
        return this.alignSelectedShape("center");
      case "AlignRight":
        return this.alignSelectedShape("right");
      case "AlignTop":
        return this.alignSelectedShape("top");
      case "AlignMiddle":
        return this.alignSelectedShape("middle");
      case "AlignBottom":
        return this.alignSelectedShape("bottom");
      case "SetSlideNotes":
        if (typeof payload === "string") {
          return this.setSlideNotes(payload);
        }
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
