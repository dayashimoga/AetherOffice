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

export type TransitionEffect =
  | "None"
  | "Fade"
  | "Push"
  | "Wipe"
  | "Split"
  | "Morph"
  | "Reveal"
  | "Cover"
  | "Random Bars"
  | "Shape"
  | "Cube"
  | "Flip"
  | "Vortex"
  | "Honeycomb"
  | "Origami"
  | "Ripple";

export type TransitionDirection =
  | "left"
  | "right"
  | "top"
  | "bottom"
  | "horizontal"
  | "vertical";

export interface SlideTransitionConfig {
  effect: TransitionEffect;
  duration: number; // in seconds, default 1.0
  direction?: TransitionDirection;
}

export type AnimationType = "entrance" | "emphasis" | "exit";

export type AnimationEffect =
  // Entrance:
  | "appear"
  | "fadeIn"
  | "flyIn"
  | "zoom"
  | "bounce"
  // Emphasis:
  | "pulse"
  | "spin"
  // Exit:
  | "fadeOut"
  | "flyOut"
  // Motion Path:
  | "motionPath";

export type AnimationTrigger = "click" | "withPrev" | "afterPrev";

export interface SlideAnimation {
  id: string;
  shapeId: string;
  type: AnimationType;
  effect: AnimationEffect;
  trigger: AnimationTrigger;
  duration: number; // in seconds, default 0.5
  delay: number; // in seconds, default 0
  order: number;
  motionPath?: string;
  easing?: "linear" | "ease-in-out" | "cubic-bezier" | "bounce" | "spring";
  rotateToPath?: boolean;
}

export function getAnimationType(effect: AnimationEffect): AnimationType {
  switch (effect) {
    case "appear":
    case "fadeIn":
    case "flyIn":
    case "zoom":
    case "bounce":
      return "entrance";
    case "pulse":
    case "spin":
      return "emphasis";
    case "fadeOut":
    case "flyOut":
      return "exit";
    default:
      return "entrance";
  }
}

export interface SlideShape {
  id: string;
  type: PresetShapeKind;
  shapeType?: PresetShapeKind;
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
  mediaType?: "video" | "audio";
  mediaUrl?: string;
  mediaFormat?: string;
  posterUrl?: string;
  autoPlay?: boolean;
  loopPlayback?: boolean;
  volume?: number;
  isMuted?: boolean;
  isPlaying?: boolean;
  tableData?: string[][];
  tableHeaders?: string[];
  zIndex?: number;
}

export interface SlideData {
  id: string;
  title: string;
  shapes: SlideShape[];
  layout?: SlideLayoutType;
  isHidden?: boolean;
  transition?: string;
  transitionConfig?: SlideTransitionConfig;
  animations?: SlideAnimation[];
  notes?: string;
  background?: string;
}

export class SlidesState {
  slides: SlideData[] = [
    {
      id: "s1",
      title: "Title Slide",
      layout: "title",
      shapes: [
        {
          id: "sh1",
          type: "text",
          x: 100,
          y: 150,
          w: 760,
          h: 80,
          fill: "transparent",
          text: "Click to add title",
          textColor: "#1e293b",
          fontSize: 40,
          bold: true,
          rotation: 0,
        },
        {
          id: "sh2",
          type: "text",
          x: 100,
          y: 260,
          w: 760,
          h: 60,
          fill: "transparent",
          text: "Click to add subtitle",
          textColor: "#64748b",
          fontSize: 22,
          rotation: 0,
        },
      ],
    },
  ];

  public resetToBlank(_title = "Presentation 1.pptx") {
    const isDark =
      this.currentTheme === "DarkMode" || this.currentTheme === "HighContrast";
    const textColor = isDark ? "#ffffff" : "#1e293b";
    const subColor = isDark ? "#94a3b8" : "#64748b";

    this.slides = [
      {
        id: "s1",
        title: "Title Slide",
        layout: "title",
        shapes: [
          {
            id: "sh1",
            type: "text",
            x: 100,
            y: 150,
            w: 760,
            h: 80,
            fill: "transparent",
            text: "Click to add title",
            textColor,
            fontSize: 40,
            bold: true,
            rotation: 0,
          },
          {
            id: "sh2",
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
        ],
      },
    ];
    this.currentSlideIndex = 0;
    this.selectedShapeId = null;
    this.undoStack = [];
    this.redoStack = [];
    this.nextShapeCounter = 10;
    this.nextSlideCounter = 1;
    this.isShowActive = false;
    this.isPresenterMode = false;
    this.currentShowStep = 0;
    this.isLaserActive = false;
    this.laserPos = null;
    this.selectedAnimationId = null;
  }

  currentSlideIndex = 0;
  selectedShapeId: string | null = null;
  currentTheme: ThemePresetName = "OfficeDefault";

  // Slide Dimension State
  slideAspectRatio: "16:9" | "4:3" = "16:9";
  slideWidth = 960;
  slideHeight = 540;

  // Slide Show State
  isShowActive = false;
  isPresenterMode = false;
  currentShowStep = 0;
  isLaserActive = false;
  laserPos: { x: number; y: number } | null = null;
  selectedAnimationId: string | null = null;

  private undoStack: SlideData[][] = [];
  private redoStack: SlideData[][] = [];
  private nextShapeCounter = 20;
  private nextSlideCounter = 10;
  private nextAnimCounter = 10;

  public selectSlide(idx: number) {
    if (idx >= 0 && idx < this.slides.length) {
      this.currentSlideIndex = idx;
      this.selectedShapeId = null;
    }
  }

  public selectShape(id: string | null) {
    this.selectedShapeId = id;
  }

  public getCurrentSlide(): SlideData {
    return this.slides[this.currentSlideIndex] || this.slides[0];
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
      layout,
      isHidden: false,
    };
    this.slides.push(newSlide);
    this.currentSlideIndex = this.slides.length - 1;
    this.selectedShapeId = null;
    return newSlide;
  }

  public setSlideLayout(idx: number, layout: SlideLayoutType): boolean {
    return this.applyLayout(layout, idx);
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
    slide.layout = layout;
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

  // --- Slide Transitions ---

  public setTransition(
    effect: TransitionEffect,
    duration = 1.0,
    direction: TransitionDirection = "right",
    applyToAll = false
  ): void {
    this.commitSnapshot();
    const config: SlideTransitionConfig = { effect, duration, direction };
    if (applyToAll) {
      this.slides.forEach((s) => {
        s.transition = effect;
        s.transitionConfig = { ...config };
      });
    } else {
      const slide = this.slides[this.currentSlideIndex];
      if (slide) {
        slide.transition = effect;
        slide.transitionConfig = { ...config };
      }
    }
  }

  public setTransitionDuration(duration: number, applyToAll = false): void {
    this.commitSnapshot();
    const safeDur = Math.max(0.1, duration);
    if (applyToAll) {
      this.slides.forEach((s) => {
        if (!s.transitionConfig) {
          s.transitionConfig = { effect: "None", duration: safeDur, direction: "right" };
        } else {
          s.transitionConfig.duration = safeDur;
        }
      });
    } else {
      const slide = this.slides[this.currentSlideIndex];
      if (slide) {
        if (!slide.transitionConfig) {
          slide.transitionConfig = { effect: "None", duration: safeDur, direction: "right" };
        } else {
          slide.transitionConfig.duration = safeDur;
        }
      }
    }
  }

  public setTransitionDirection(direction: TransitionDirection, applyToAll = false): void {
    this.commitSnapshot();
    if (applyToAll) {
      this.slides.forEach((s) => {
        if (!s.transitionConfig) {
          s.transitionConfig = { effect: "None", duration: 1.0, direction };
        } else {
          s.transitionConfig.direction = direction;
        }
      });
    } else {
      const slide = this.slides[this.currentSlideIndex];
      if (slide) {
        if (!slide.transitionConfig) {
          slide.transitionConfig = { effect: "None", duration: 1.0, direction };
        } else {
          slide.transitionConfig.direction = direction;
        }
      }
    }
  }

  public applyTransitionToAll(): void {
    const cur = this.getSlideTransition();
    this.setTransition(cur.effect, cur.duration, cur.direction, true);
  }

  public getSlideTransition(idx = this.currentSlideIndex): SlideTransitionConfig {
    const slide = this.slides[idx];
    if (slide?.transitionConfig) {
      return { ...slide.transitionConfig };
    }
    const effect = (slide?.transition as TransitionEffect) || "None";
    return { effect, duration: 1.0, direction: "right" };
  }

  // --- Slide Animations ---

  public addAnimation(
    shapeId: string,
    effect: AnimationEffect,
    type?: AnimationType,
    trigger: AnimationTrigger = "click",
    duration = 0.5,
    delay = 0
  ): SlideAnimation | null {
    const slide = this.slides[this.currentSlideIndex];
    if (!slide) return null;
    const shape = slide.shapes.find((s) => s.id === shapeId);
    if (!shape) return null;

    if (!slide.animations) {
      slide.animations = [];
    }

    this.commitSnapshot();
    const animType = type || getAnimationType(effect);
    const order = slide.animations.length + 1;
    const newAnim: SlideAnimation = {
      id: `anim_${++this.nextAnimCounter}`,
      shapeId,
      type: animType,
      effect,
      trigger,
      duration: Math.max(0.1, duration),
      delay: Math.max(0, delay),
      order,
    };
    slide.animations.push(newAnim);
    this.selectedAnimationId = newAnim.id;
    return newAnim;
  }

  public addMotionPathAnimation(
    shapeId = this.selectedShapeId || "",
    path = "M 0 0 C 100 -50 200 150 300 0",
    duration = 1.5,
    easing: "linear" | "ease-in-out" | "cubic-bezier" | "bounce" | "spring" = "ease-in-out",
    rotateToPath = true
  ): SlideAnimation | null {
    if (!shapeId) return null;
    const slide = this.slides[this.currentSlideIndex];
    if (!slide) return null;
    if (!slide.animations) slide.animations = [];

    this.commitSnapshot();
    const newAnim: SlideAnimation = {
      id: `anim_${++this.nextAnimCounter}`,
      shapeId,
      type: "emphasis",
      effect: "motionPath",
      trigger: "click",
      duration: Math.max(0.1, duration),
      delay: 0,
      order: slide.animations.length + 1,
      motionPath: path,
      easing,
      rotateToPath,
    };
    slide.animations.push(newAnim);
    this.selectedAnimationId = newAnim.id;
    return newAnim;
  }

  public removeAnimation(animId: string): boolean {
    const slide = this.slides[this.currentSlideIndex];
    if (!slide || !slide.animations) return false;
    const idx = slide.animations.findIndex((a) => a.id === animId);
    if (idx === -1) return false;

    this.commitSnapshot();
    slide.animations.splice(idx, 1);
    slide.animations.forEach((a, i) => {
      a.order = i + 1;
    });
    if (this.selectedAnimationId === animId) {
      this.selectedAnimationId =
        slide.animations.length > 0
          ? slide.animations[Math.min(idx, slide.animations.length - 1)].id
          : null;
    }
    return true;
  }

  public reorderAnimation(animId: string, direction: "up" | "down"): boolean {
    const slide = this.slides[this.currentSlideIndex];
    if (!slide || !slide.animations) return false;
    const idx = slide.animations.findIndex((a) => a.id === animId);
    if (idx === -1) return false;

    if (direction === "up" && idx > 0) {
      this.commitSnapshot();
      const temp = slide.animations[idx];
      slide.animations[idx] = slide.animations[idx - 1];
      slide.animations[idx - 1] = temp;
      slide.animations.forEach((a, i) => (a.order = i + 1));
      return true;
    } else if (direction === "down" && idx < slide.animations.length - 1) {
      this.commitSnapshot();
      const temp = slide.animations[idx];
      slide.animations[idx] = slide.animations[idx + 1];
      slide.animations[idx + 1] = temp;
      slide.animations.forEach((a, i) => (a.order = i + 1));
      return true;
    }
    return false;
  }

  public getSlideAnimations(idx = this.currentSlideIndex): SlideAnimation[] {
    const slide = this.slides[idx];
    return slide?.animations ? [...slide.animations] : [];
  }

  public getShapeAnimations(shapeId: string, idx = this.currentSlideIndex): SlideAnimation[] {
    const anims = this.getSlideAnimations(idx);
    return anims.filter((a) => a.shapeId === shapeId);
  }

  public getSelectedAnimation(): SlideAnimation | null {
    if (!this.selectedAnimationId) return null;
    const anims = this.getSlideAnimations();
    return anims.find((a) => a.id === this.selectedAnimationId) || null;
  }

  public selectAnimation(animId: string | null): void {
    this.selectedAnimationId = animId;
    if (animId) {
      const anim = this.getSelectedAnimation();
      if (anim) {
        this.selectedShapeId = anim.shapeId;
      }
    }
  }

  public updateAnimation(animId: string, updates: Partial<SlideAnimation>): boolean {
    const slide = this.slides[this.currentSlideIndex];
    if (!slide || !slide.animations) return false;
    const anim = slide.animations.find((a) => a.id === animId);
    if (!anim) return false;

    this.commitSnapshot();
    if (updates.effect !== undefined) {
      anim.effect = updates.effect;
      anim.type = updates.type || getAnimationType(updates.effect);
    }
    if (updates.trigger !== undefined) anim.trigger = updates.trigger;
    if (updates.duration !== undefined) anim.duration = Math.max(0.1, updates.duration);
    if (updates.delay !== undefined) anim.delay = Math.max(0, updates.delay);
    if (updates.order !== undefined) anim.order = updates.order;
    return true;
  }

  public setAnimationTrigger(animId: string, trigger: AnimationTrigger): boolean {
    return this.updateAnimation(animId, { trigger });
  }

  public setAnimationDuration(animId: string, duration: number): boolean {
    return this.updateAnimation(animId, { duration });
  }

  public setAnimationDelay(animId: string, delay: number): boolean {
    return this.updateAnimation(animId, { delay });
  }

  public clearSlideAnimations(idx = this.currentSlideIndex): void {
    const slide = this.slides[idx];
    if (slide && slide.animations && slide.animations.length > 0) {
      this.commitSnapshot();
      slide.animations = [];
      this.selectedAnimationId = null;
    }
  }

  // --- Slide Show Mode & Presenter Engine ---

  public startShow(fromBeginning = true, presenterMode = false) {
    this.isShowActive = true;
    this.isPresenterMode = presenterMode;
    this.isLaserActive = false;
    this.laserPos = null;
    this.currentShowStep = 0;
    this.currentSlideIndex = fromBeginning ? 0 : this.currentSlideIndex;
  }

  public exitShow() {
    this.isShowActive = false;
    this.isPresenterMode = false;
    this.isLaserActive = false;
    this.laserPos = null;
    this.currentShowStep = 0;
  }

  public togglePresenterView(): boolean {
    this.isPresenterMode = !this.isPresenterMode;
    return this.isPresenterMode;
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
      this.currentShowStep = 0;
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
      const prevAnims = this.getSlideAnimations(prevIdx);
      this.currentShowStep = prevAnims.length;
      this.laserPos = null;
      return true;
    }
    return false;
  }

  public stepForward(): { slideChanged: boolean; finished: boolean; currentAnimation?: SlideAnimation } {
    if (!this.isShowActive) return { slideChanged: false, finished: false };
    const anims = this.getSlideAnimations();

    if (this.currentShowStep < anims.length) {
      const anim = anims[this.currentShowStep];
      this.currentShowStep++;
      return { slideChanged: false, finished: false, currentAnimation: anim };
    }

    let nextIdx = this.currentSlideIndex + 1;
    while (nextIdx < this.slides.length && this.slides[nextIdx].isHidden) {
      nextIdx++;
    }

    if (nextIdx < this.slides.length) {
      this.currentSlideIndex = nextIdx;
      this.currentShowStep = 0;
      this.laserPos = null;
      return { slideChanged: true, finished: false };
    }

    return { slideChanged: false, finished: true };
  }

  public stepBackward(): { slideChanged: boolean } {
    if (!this.isShowActive) return { slideChanged: false };

    if (this.currentShowStep > 0) {
      this.currentShowStep--;
      return { slideChanged: false };
    }

    let prevIdx = this.currentSlideIndex - 1;
    while (prevIdx >= 0 && this.slides[prevIdx].isHidden) {
      prevIdx--;
    }

    if (prevIdx >= 0) {
      this.currentSlideIndex = prevIdx;
      const prevAnims = this.getSlideAnimations(prevIdx);
      this.currentShowStep = prevAnims.length;
      this.laserPos = null;
      return { slideChanged: true };
    }

    return { slideChanged: false };
  }

  public getVisibleShapesForStep(
    slideIdx = this.currentSlideIndex,
    step = this.currentShowStep
  ): {
    visibleShapeIds: Set<string>;
    animatingShapeId?: string;
    currentEffect?: AnimationEffect;
  } {
    const slide = this.slides[slideIdx];
    if (!slide) return { visibleShapeIds: new Set() };
    const anims = slide.animations || [];
    const visibleIds = new Set<string>();

    slide.shapes.forEach((sh) => {
      const entranceAnim = anims.find((a) => a.shapeId === sh.id && a.type === "entrance");
      if (!entranceAnim) {
        visibleIds.add(sh.id);
      }
    });

    for (let i = 0; i < step && i < anims.length; i++) {
      const a = anims[i];
      if (a.type === "entrance") {
        visibleIds.add(a.shapeId);
      } else if (a.type === "exit") {
        visibleIds.delete(a.shapeId);
      }
    }

    const currentAnim = step > 0 && step <= anims.length ? anims[step - 1] : undefined;

    return {
      visibleShapeIds: visibleIds,
      animatingShapeId: currentAnim?.shapeId,
      currentEffect: currentAnim?.effect,
    };
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
      shapeType: type,
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

  public insertMedia(
    type: "video" | "audio",
    url = type === "video" ? "sample_video.mp4" : "sample_audio.mp3",
    x = 180,
    y = 120,
    w = type === "video" ? 480 : 280,
    h = type === "video" ? 270 : 60
  ): SlideShape {
    this.commitSnapshot();
    const slide = this.slides[this.currentSlideIndex];
    if (!slide) throw new Error("No active slide");

    const isVideo = type === "video";
    const shape: SlideShape = {
      id: `sh_${++this.nextShapeCounter}`,
      type: "rect",
      x,
      y,
      w,
      h,
      fill: isVideo ? "#0f172a" : "#f1f5f9",
      stroke: "#64748b",
      strokeWidth: 1,
      rotation: 0,
      mediaType: type,
      mediaUrl: url,
      mediaFormat: url.split(".").pop() || (isVideo ? "mp4" : "mp3"),
      posterUrl: isVideo ? "poster_frame.png" : undefined,
      autoPlay: false,
      loopPlayback: false,
      volume: 1.0,
      isMuted: false,
      isPlaying: false,
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
      case "TogglePresenterView":
        this.togglePresenterView();
        return true;
      case "StepShowForward":
        return this.stepForward().slideChanged || true;
      case "StepShowBackward":
        return this.stepBackward().slideChanged || true;
      case "SetTransition":
        if (typeof payload === "string") {
          this.setTransition(payload as TransitionEffect);
          return true;
        }
        return false;
      case "SetEffectOption":
        if (typeof payload === "string") {
          this.setTransitionDirection(payload as TransitionDirection);
          return true;
        }
        return false;
      case "SetTransitionDuration":
        if (typeof payload === "number") {
          this.setTransitionDuration(payload);
          return true;
        }
        return false;
      case "ApplyTransitionToAll":
        this.applyTransitionToAll();
        return true;
      case "SetAnimation": {
        const eff = (typeof payload === "string" ? payload : "fadeIn") as AnimationEffect;
        const sel = this.getSelectedShape();
        if (sel) {
          this.addAnimation(sel.id, eff);
          return true;
        }
        return false;
      }
      case "AddAnimation": {
        if (typeof payload === "string") {
          const sel = this.getSelectedShape();
          if (sel) {
            this.addAnimation(sel.id, payload as AnimationEffect);
            return true;
          }
        } else if (payload && typeof payload === "object") {
          const p = payload as {
            shapeId: string;
            effect: AnimationEffect;
            trigger?: AnimationTrigger;
            duration?: number;
          };
          if (p.shapeId && p.effect) {
            this.addAnimation(p.shapeId, p.effect, undefined, p.trigger, p.duration);
            return true;
          }
        }
        return false;
      }
      case "AddMotionPath": {
        const sel = this.getSelectedShape();
        if (sel) {
          const path = typeof payload === "string" ? payload : (payload as any)?.path;
          this.addMotionPathAnimation(sel.id, path);
          return true;
        }
        return false;
      }
      case "RemoveAnimation":
        if (typeof payload === "string") {
          return this.removeAnimation(payload);
        }
        return false;
      case "ReorderAnimation":
        if (payload && typeof payload === "object") {
          const p = payload as { animId: string; direction: "up" | "down" };
          return this.reorderAnimation(p.animId, p.direction);
        }
        return false;
      case "SetAnimStart":
      case "SetAnimationTrigger":
        if (typeof payload === "string" && this.selectedAnimationId) {
          return this.setAnimationTrigger(this.selectedAnimationId, payload as AnimationTrigger);
        }
        return false;
      case "SetAnimationDuration":
        if (typeof payload === "number" && this.selectedAnimationId) {
          return this.setAnimationDuration(this.selectedAnimationId, payload);
        }
        return false;
      case "SetAnimationDelay":
        if (typeof payload === "number" && this.selectedAnimationId) {
          return this.setAnimationDelay(this.selectedAnimationId, payload);
        }
        return false;
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
      case "InsertVideo":
        this.insertMedia("video", typeof payload === "string" ? payload : "sample_video.mp4");
        return true;
      case "InsertAudio":
        this.insertMedia("audio", typeof payload === "string" ? payload : "sample_audio.mp3");
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
