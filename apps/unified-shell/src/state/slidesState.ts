export type PresetShapeKind =
  | "text"
  | "rect"
  | "roundRect"
  | "ellipse"
  | "triangle"
  | "arrow"
  | "arrowLeft"
  | "arrowUp"
  | "arrowDown"
  | "star"
  | "diamond"
  | "callout"
  | "heart"
  | "cloud"
  | "pentagon"
  | "hexagon"
  | "banner"
  | "icon"
  | "smartArt"
  | "chart"
  | "wordArt"
  | "image"
  | "table";

export type ThemePresetName =
  | "OfficeDefault"
  | "Office"
  | "Celestial"
  | "Facet"
  | "Integral"
  | "Ion"
  | "Organic"
  | "Retrospect"
  | "Slice"
  | "Wisp"
  | "Slate"
  | "DarkMode"
  | "VibrantCoral"
  | "HighContrast"
  | "EmeraldForest"
  | "OceanBreeze"
  | "SunsetGlow"
  | "SlateMinimal"
  | "ExecutiveNavy"
  | "WarmSand"
  | "CyberNeon"
  | "MinimalWhite"
  | "DarkSlate"
  | "OceanBlue"
  | "Emerald"
  | "Sunset";

export type SlideLayoutType =
  | "title"
  | "titleAndContent"
  | "sectionHeader"
  | "twoContent"
  | "comparison"
  | "titleOnly"
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
  | "Ripple"
  | "Zoom"
  | "Slide Left";

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
  sound?: string; // "none" | "chime" | "applause" | "drumroll" | "whoosh"
  advanceOnClick?: boolean;
  advanceAfter?: number; // seconds
}

export type AnimationType = "entrance" | "emphasis" | "exit";

export type AnimationEffect =
  // Entrance:
  | "appear"
  | "fadeIn"
  | "flyIn"
  | "floatIn"
  | "split"
  | "wipe"
  | "shape"
  | "wheel"
  | "bounce"
  | "zoom"
  // Emphasis:
  | "pulse"
  | "teeter"
  | "spin"
  | "growShrink"
  // Exit:
  | "disappear"
  | "fadeOut"
  | "flyOut"
  | "zoomOut"
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
    case "floatIn":
    case "split":
    case "wipe":
    case "shape":
    case "wheel":
    case "bounce":
    case "zoom":
      return "entrance";
    case "pulse":
    case "teeter":
    case "spin":
    case "growShrink":
      return "emphasis";
    case "disappear":
    case "fadeOut":
    case "flyOut":
    case "zoomOut":
      return "exit";
    case "motionPath":
      return "entrance";
    default:
      return "entrance";
  }
}

export interface SlideShape {
  id: string;
  type: PresetShapeKind;
  kind?: PresetShapeKind;
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
  color?: string;
  fontSize?: number;
  fontFamily?: string;
  bold?: boolean;
  italic?: boolean;
  underline?: boolean;
  strikethrough?: boolean;
  textAlign?: "left" | "center" | "right" | "justify";
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
  shadow?: boolean;
  textShadow?: boolean;
  glow?: boolean;
  verticalAlign?: "top" | "middle" | "bottom";
  smartArtType?: "process" | "cycle" | "hierarchy" | "pyramid";
  smartArtNodes?: { id: string; label: string; color?: string }[];
  chartType?: "bar" | "line" | "pie";
  chartData?: { categories: string[]; series: { name: string; values: number[] }[] };
  wordArtStyle?: string;
  iconName?: string;
  groupId?: string;
  isHeaderFooter?: boolean;
  headerFooterType?: "header" | "footer" | "date" | "slideNumber";
}

export interface SlideComment {
  id: string;
  author: string;
  text: string;
  createdAt: string;
  x?: number;
  y?: number;
  resolved?: boolean;
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
  comments?: SlideComment[];
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
  currentVariant: string = "Blue";

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
  isPreviewingAnimation = false;
  showAnimationPane = false;
  loopContinuously = false;
  isRehearsingTimings = false;
  rehearsalStartTime = 0;
  slideTimings: Map<number, number> = new Map();

  // Comments & Review State
  nextCommentCounter = 0;
  showCommentsPane = false;
  isSpellingChecked = false;
  thesaurusQuery = "";

  // Views & Display State
  currentView: "normal" | "sorter" | "outline" | "notes" | "reading" | "master" = "normal";
  masterViewMode: "slideMaster" | "handoutMaster" | "notesMaster" | null = null;
  showRuler = false;
  showGridlines = false;
  showGuides = false;
  showSpeakerNotesDrawer = false;
  colorMode: "color" | "grayscale" | "bw" = "color";
  zoomLevel = 100;

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

      case "comparison":
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
            y: 110,
            w: 380,
            h: 40,
            fill: "transparent",
            text: "Option A",
            textColor,
            fontSize: 20,
            bold: true,
            rotation: 0,
          },
          {
            id: `sh_${++this.nextShapeCounter}`,
            type: "text",
            x: 80,
            y: 160,
            w: 380,
            h: 320,
            fill: "transparent",
            text: "• Features of Option A\n• Benefits & Pros\n• Specifications",
            textColor: subColor,
            fontSize: 16,
            rotation: 0,
          },
          {
            id: `sh_${++this.nextShapeCounter}`,
            type: "text",
            x: 500,
            y: 110,
            w: 380,
            h: 40,
            fill: "transparent",
            text: "Option B",
            textColor,
            fontSize: 20,
            bold: true,
            rotation: 0,
          },
          {
            id: `sh_${++this.nextShapeCounter}`,
            type: "text",
            x: 500,
            y: 160,
            w: 380,
            h: 320,
            fill: "transparent",
            text: "• Features of Option B\n• Benefits & Pros\n• Specifications",
            textColor: subColor,
            fontSize: 16,
            rotation: 0,
          },
        ];
        break;

      case "sectionHeader":
        shapes = [
          {
            id: `sh_${++this.nextShapeCounter}`,
            type: "text",
            x: 100,
            y: 180,
            w: 760,
            h: 70,
            fill: "transparent",
            text: title,
            textColor,
            fontSize: 36,
            bold: true,
            rotation: 0,
          },
          {
            id: `sh_${++this.nextShapeCounter}`,
            type: "text",
            x: 100,
            y: 260,
            w: 760,
            h: 50,
            fill: "transparent",
            text: "Section Description",
            textColor: subColor,
            fontSize: 20,
            rotation: 0,
          },
        ];
        break;

      case "titleOnly":
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

      case "sectionHeader":
        shapes = [
          {
            id: `sh_${++this.nextShapeCounter}`,
            type: "text",
            x: 100,
            y: 180,
            w: 760,
            h: 70,
            fill: "transparent",
            text: existingTitle,
            textColor,
            fontSize: 36,
            bold: true,
            rotation: 0,
          },
          {
            id: `sh_${++this.nextShapeCounter}`,
            type: "text",
            x: 100,
            y: 260,
            w: 760,
            h: 50,
            fill: "transparent",
            text: "Section Description",
            textColor: subColor,
            fontSize: 20,
            rotation: 0,
          },
        ];
        break;

      case "comparison":
        shapes = [
          {
            id: `sh_${++this.nextShapeCounter}`,
            type: "text",
            x: 80,
            y: 40,
            w: 800,
            h: 60,
            fill: "transparent",
            text: existingTitle || "Comparison Title",
            textColor,
            fontSize: 32,
            bold: true,
            rotation: 0,
          },
          {
            id: `sh_${++this.nextShapeCounter}`,
            type: "text",
            x: 80,
            y: 110,
            w: 380,
            h: 40,
            fill: "transparent",
            text: "Option A",
            textColor,
            fontSize: 20,
            bold: true,
            rotation: 0,
          },
          {
            id: `sh_${++this.nextShapeCounter}`,
            type: "text",
            x: 80,
            y: 160,
            w: 380,
            h: 320,
            fill: "transparent",
            text: "• Features of Option A\n• Benefits & Pros\n• Specifications",
            textColor: subColor,
            fontSize: 16,
            rotation: 0,
          },
          {
            id: `sh_${++this.nextShapeCounter}`,
            type: "text",
            x: 500,
            y: 110,
            w: 380,
            h: 40,
            fill: "transparent",
            text: "Option B",
            textColor,
            fontSize: 20,
            bold: true,
            rotation: 0,
          },
          {
            id: `sh_${++this.nextShapeCounter}`,
            type: "text",
            x: 500,
            y: 160,
            w: 380,
            h: 320,
            fill: "transparent",
            text: "• Features of Option B\n• Benefits & Pros\n• Specifications",
            textColor: subColor,
            fontSize: 16,
            rotation: 0,
          },
        ];
        break;

      case "titleOnly":
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

  public setTheme(theme: ThemePresetName | string) {
    this.commitSnapshot();
    const norm = theme.toLowerCase().replace(/[\s_-]/g, "");

    let textColor = "#1e293b";
    let primaryAccent = "#2563eb";
    let secondaryAccent = "#10b981";
    let bg = "#ffffff";
    let fontFamily = "Aptos, Calibri, sans-serif";

    switch (norm) {
      case "darkslate":
        this.currentTheme = "DarkSlate";
        textColor = "#f8fafc";
        primaryAccent = "#6366f1";
        secondaryAccent = "#06b6d4";
        bg = "#0f172a";
        fontFamily = "Segoe UI, sans-serif";
        break;
      case "darkmode":
        this.currentTheme = "DarkMode";
        textColor = "#f8fafc";
        primaryAccent = "#6366f1";
        secondaryAccent = "#06b6d4";
        bg = "#0f172a";
        fontFamily = "Segoe UI, sans-serif";
        break;
      case "slateminimal":
        this.currentTheme = "SlateMinimal";
        textColor = "#334155";
        primaryAccent = "#475569";
        secondaryAccent = "#3b82f6";
        bg = "#f8fafc";
        fontFamily = "Segoe UI, sans-serif";
        break;
      case "oceanblue":
        this.currentTheme = "OceanBlue";
        textColor = "#0c4a6e";
        primaryAccent = "#0284c7";
        secondaryAccent = "#14b8a6";
        bg = "#f0f9ff";
        fontFamily = "Arial, sans-serif";
        break;
      case "oceanbreeze":
        this.currentTheme = "OceanBreeze";
        textColor = "#0c4a6e";
        primaryAccent = "#0284c7";
        secondaryAccent = "#14b8a6";
        bg = "#f0f9ff";
        fontFamily = "Arial, sans-serif";
        break;
      case "emerald":
        this.currentTheme = "Emerald";
        textColor = "#064e3b";
        primaryAccent = "#059669";
        secondaryAccent = "#d97706";
        bg = "#f0fdf4";
        fontFamily = "Georgia, serif";
        break;
      case "emeraldforest":
        this.currentTheme = "EmeraldForest";
        textColor = "#064e3b";
        primaryAccent = "#059669";
        secondaryAccent = "#d97706";
        bg = "#f0fdf4";
        fontFamily = "Georgia, serif";
        break;
      case "sunset":
        this.currentTheme = "Sunset";
        textColor = "#701a75";
        primaryAccent = "#e11d48";
        secondaryAccent = "#f59e0b";
        bg = "#fdf2f8";
        fontFamily = "Century Gothic, sans-serif";
        break;
      case "sunsetglow":
        this.currentTheme = "SunsetGlow";
        textColor = "#701a75";
        primaryAccent = "#e11d48";
        secondaryAccent = "#f59e0b";
        bg = "#fdf2f8";
        fontFamily = "Century Gothic, sans-serif";
        break;
      case "vibrantcoral":
        this.currentTheme = "VibrantCoral";
        textColor = "#18181b";
        primaryAccent = "#c43e1c";
        secondaryAccent = "#0d9488";
        bg = "#fff7ed";
        fontFamily = "Calibri, sans-serif";
        break;
      case "executivenavy":
        this.currentTheme = "ExecutiveNavy";
        textColor = "#f8fafc";
        primaryAccent = "#1e3a8a";
        secondaryAccent = "#38bdf8";
        bg = "#0a1128";
        fontFamily = "Segoe UI, sans-serif";
        break;
      case "warmsand":
        this.currentTheme = "WarmSand";
        textColor = "#451a03";
        primaryAccent = "#d97706";
        secondaryAccent = "#b45309";
        bg = "#fef3c7";
        fontFamily = "Georgia, serif";
        break;
      case "cyberneon":
        this.currentTheme = "CyberNeon";
        textColor = "#00ffcc";
        primaryAccent = "#ff007f";
        secondaryAccent = "#00e5ff";
        bg = "#000000";
        fontFamily = "Consolas, monospace";
        break;
      case "highcontrast":
        this.currentTheme = "HighContrast";
        textColor = "#00ffcc";
        primaryAccent = "#ff007f";
        secondaryAccent = "#00e5ff";
        bg = "#000000";
        fontFamily = "Consolas, monospace";
        break;
      case "minimalwhite":
        this.currentTheme = "MinimalWhite";
        textColor = "#1e293b";
        primaryAccent = "#2563eb";
        secondaryAccent = "#10b981";
        bg = "#ffffff";
        fontFamily = "Aptos, Calibri, sans-serif";
        break;
      case "office":
        this.currentTheme = "Office";
        textColor = "#1e293b";
        primaryAccent = "#2563eb";
        secondaryAccent = "#0284c7";
        bg = "#ffffff";
        fontFamily = "Aptos, Segoe UI, sans-serif";
        break;
      case "celestial":
        this.currentTheme = "Celestial";
        textColor = "#f1f5f9";
        primaryAccent = "#48cae4";
        secondaryAccent = "#7209b7";
        bg = "linear-gradient(135deg, #0b132b 0%, #1c2541 50%, #3a506b 100%)";
        fontFamily = "Trebuchet MS, Arial, sans-serif";
        break;
      case "facet":
        this.currentTheme = "Facet";
        textColor = "#f0fdfa";
        primaryAccent = "#14b8a6";
        secondaryAccent = "#0f766e";
        bg = "linear-gradient(135deg, #042f2e 0%, #0f766e 100%)";
        fontFamily = "Century Gothic, Arial, sans-serif";
        break;
      case "integral":
        this.currentTheme = "Integral";
        textColor = "#fff7ed";
        primaryAccent = "#f97316";
        secondaryAccent = "#d97706";
        bg = "linear-gradient(135deg, #451a03 0%, #7c2d12 100%)";
        fontFamily = "Georgia, serif";
        break;
      case "ion":
        this.currentTheme = "Ion";
        textColor = "#f3f4f6";
        primaryAccent = "#10b981";
        secondaryAccent = "#22c55e";
        bg = "linear-gradient(135deg, #030712 0%, #111827 100%)";
        fontFamily = "Consolas, Segoe UI, sans-serif";
        break;
      case "organic":
        this.currentTheme = "Organic";
        textColor = "#f0fdf4";
        primaryAccent = "#4ade80";
        secondaryAccent = "#16a34a";
        bg = "linear-gradient(135deg, #052e16 0%, #14532d 100%)";
        fontFamily = "Palatino Linotype, Georgia, serif";
        break;
      case "retrospect":
        this.currentTheme = "Retrospect";
        textColor = "#fdf2f8";
        primaryAccent = "#fb7185";
        secondaryAccent = "#e11d48";
        bg = "linear-gradient(135deg, #500724 0%, #831843 100%)";
        fontFamily = "Garamond, Georgia, serif";
        break;
      case "slice":
        this.currentTheme = "Slice";
        textColor = "#ffffff";
        primaryAccent = "#ff6b35";
        secondaryAccent = "#f97316";
        bg = "linear-gradient(135deg, #111827 0%, #1f2937 100%)";
        fontFamily = "Impact, Arial Black, sans-serif";
        break;
      case "wisp":
        this.currentTheme = "Wisp";
        textColor = "#334155";
        primaryAccent = "#64748b";
        secondaryAccent = "#94a3b8";
        bg = "#f8fafc";
        fontFamily = "Segoe UI Light, Aptos, sans-serif";
        break;
      case "slate":
        this.currentTheme = "Slate";
        textColor = "#f8fafc";
        primaryAccent = "#38bdf8";
        secondaryAccent = "#6366f1";
        bg = "linear-gradient(135deg, #0f172a 0%, #1e293b 100%)";
        fontFamily = "Segoe UI, sans-serif";
        break;
      case "officedefault":
      default:
        this.currentTheme = "OfficeDefault";
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

  public setVariant(variant: "Blue" | "Emerald" | "Coral" | "Purple" | string) {
    this.commitSnapshot();
    this.currentVariant = variant;
    let primaryAccent = "#2563eb";
    let secondaryAccent = "#38bdf8";
    const v = variant.toLowerCase();
    if (v.includes("emerald")) {
      primaryAccent = "#059669";
      secondaryAccent = "#10b981";
    } else if (v.includes("coral")) {
      primaryAccent = "#c43e1c";
      secondaryAccent = "#f97316";
    } else if (v.includes("purple")) {
      primaryAccent = "#7c3aed";
      secondaryAccent = "#a855f7";
    }
    this.slides.forEach((slide) => {
      slide.shapes.forEach((sh) => {
        if (sh.id.endsWith("2") || sh.id.endsWith("3")) {
          sh.fill = primaryAccent;
        } else if (sh.id.endsWith("4") || sh.id.endsWith("5")) {
          sh.fill = secondaryAccent;
        }
      });
    });
  }

  public resetSlide(idx = this.currentSlideIndex): boolean {
    if (idx < 0 || idx >= this.slides.length) return false;
    const slide = this.slides[idx];
    const layout = slide.layout || "title";
    return this.applyLayout(layout, idx);
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
    effect: TransitionEffect | string,
    duration = 1.0,
    direction: TransitionDirection = "right",
    applyToAll = false
  ): void {
    const norm = String(effect).toLowerCase().replace(/[\s_-]/g, "");
    let actualEffect: TransitionEffect = effect as TransitionEffect;
    let actualDirection: TransitionDirection = direction;

    if (norm === "slideleft") {
      actualEffect = "Push";
      actualDirection = "left";
    } else if (norm === "zoom") {
      actualEffect = "Zoom";
    } else if (norm === "flip") {
      actualEffect = "Flip";
    } else if (norm === "fade") {
      actualEffect = "Fade";
    } else if (norm === "push") {
      actualEffect = "Push";
    } else if (norm === "wipe") {
      actualEffect = "Wipe";
    } else if (norm === "split") {
      actualEffect = "Split";
    } else if (norm === "reveal") {
      actualEffect = "Reveal";
    } else if (norm === "randombars") {
      actualEffect = "Random Bars";
    } else if (norm === "shape" || norm === "shapecircle") {
      actualEffect = "Shape";
    } else if (norm === "morph") {
      actualEffect = "Morph";
    }

    this.commitSnapshot();
    const oldConfig = this.getCurrentSlide()?.transitionConfig;
    const config: SlideTransitionConfig = {
      effect: actualEffect,
      duration,
      direction: actualDirection,
      sound: oldConfig?.sound || "none",
      advanceOnClick: oldConfig?.advanceOnClick ?? true,
      advanceAfter: oldConfig?.advanceAfter ?? 0,
    };
    if (applyToAll) {
      this.slides.forEach((s) => {
        s.transition = actualEffect;
        s.transitionConfig = { ...config };
      });
    } else {
      const slide = this.slides[this.currentSlideIndex];
      if (slide) {
        slide.transition = actualEffect;
        slide.transitionConfig = { ...config };
      }
    }
  }

  public setTransitionSound(sound: string, applyToAll = false): boolean {
    this.commitSnapshot();
    if (applyToAll) {
      this.slides.forEach((s) => {
        if (!s.transitionConfig) {
          s.transitionConfig = { effect: "None", duration: 1.0, direction: "right", sound };
        } else {
          s.transitionConfig.sound = sound;
        }
      });
    } else {
      const slide = this.slides[this.currentSlideIndex];
      if (slide) {
        if (!slide.transitionConfig) {
          slide.transitionConfig = { effect: "None", duration: 1.0, direction: "right", sound };
        } else {
          slide.transitionConfig.sound = sound;
        }
      }
    }
    return true;
  }

  public setAdvanceOnClick(advance: boolean, applyToAll = false): boolean {
    this.commitSnapshot();
    if (applyToAll) {
      this.slides.forEach((s) => {
        if (!s.transitionConfig) {
          s.transitionConfig = { effect: "None", duration: 1.0, direction: "right", advanceOnClick: advance };
        } else {
          s.transitionConfig.advanceOnClick = advance;
        }
      });
    } else {
      const slide = this.slides[this.currentSlideIndex];
      if (slide) {
        if (!slide.transitionConfig) {
          slide.transitionConfig = { effect: "None", duration: 1.0, direction: "right", advanceOnClick: advance };
        } else {
          slide.transitionConfig.advanceOnClick = advance;
        }
      }
    }
    return true;
  }

  public setAdvanceAfter(seconds: number, applyToAll = false): boolean {
    this.commitSnapshot();
    if (applyToAll) {
      this.slides.forEach((s) => {
        if (!s.transitionConfig) {
          s.transitionConfig = { effect: "None", duration: 1.0, direction: "right", advanceAfter: seconds };
        } else {
          s.transitionConfig.advanceAfter = seconds;
        }
      });
    } else {
      const slide = this.slides[this.currentSlideIndex];
      if (slide) {
        if (!slide.transitionConfig) {
          slide.transitionConfig = { effect: "None", duration: 1.0, direction: "right", advanceAfter: seconds };
        } else {
          slide.transitionConfig.advanceAfter = seconds;
        }
      }
    }
    return true;
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
    effect: AnimationEffect | string,
    type?: AnimationType,
    trigger: AnimationTrigger = "click",
    duration = 0.5,
    delay = 0
  ): SlideAnimation | null {
    const slide = this.slides[this.currentSlideIndex];
    if (!slide) return null;
    const shape = slide.shapes.find((s) => s.id === shapeId);
    if (!shape) return null;

    let eff = effect as AnimationEffect;
    const norm = String(effect).toLowerCase().replace(/[\s_-]/g, "");
    if (norm === "flyup") eff = "flyIn";
    else if (norm === "pop") eff = "zoom";

    if (!slide.animations) {
      slide.animations = [];
    }

    this.commitSnapshot();
    const animType = type || getAnimationType(eff);
    const order = slide.animations.length + 1;
    const newAnim: SlideAnimation = {
      id: `anim_${++this.nextAnimCounter}`,
      shapeId,
      type: animType,
      effect: eff,
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
    if (fromBeginning) {
      let firstVisible = 0;
      while (firstVisible < this.slides.length && this.slides[firstVisible].isHidden) {
        firstVisible++;
      }
      this.currentSlideIndex = firstVisible < this.slides.length ? firstVisible : 0;
    }
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
    } else if (this.loopContinuously && this.slides.length > 0) {
      let firstVisible = 0;
      while (firstVisible < this.slides.length && this.slides[firstVisible].isHidden) {
        firstVisible++;
      }
      if (firstVisible < this.slides.length) {
        this.currentSlideIndex = firstVisible;
        this.currentShowStep = 0;
        this.laserPos = null;
        return true;
      }
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
    } else if (this.loopContinuously && this.slides.length > 0) {
      let firstVisible = 0;
      while (firstVisible < this.slides.length && this.slides[firstVisible].isHidden) {
        firstVisible++;
      }
      if (firstVisible < this.slides.length) {
        this.currentSlideIndex = firstVisible;
        this.currentShowStep = 0;
        this.laserPos = null;
        return { slideChanged: true, finished: false };
      }
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

  // --- Slide Show Settings & Rehearsal ---

  public setLoopContinuously(loop: boolean): boolean {
    this.loopContinuously = loop;
    return true;
  }

  public startRehearsingTimings(): boolean {
    this.isRehearsingTimings = true;
    this.rehearsalStartTime = Date.now();
    this.startShow(true, false);
    return true;
  }

  public stopRehearsingTimings(): boolean {
    if (this.isRehearsingTimings) {
      const elapsed = Math.round((Date.now() - this.rehearsalStartTime) / 1000);
      this.slideTimings.set(this.currentSlideIndex, Math.max(1, elapsed));
      this.isRehearsingTimings = false;
    }
    return true;
  }

  // --- Comments & Proofing ---

  public addComment(text: string, author = "Reviewer", x = 120, y = 120): SlideComment {
    this.commitSnapshot();
    const slide = this.getCurrentSlide();
    if (!slide.comments) slide.comments = [];
    const comment: SlideComment = {
      id: `comm_${++this.nextCommentCounter}`,
      author,
      text,
      createdAt: new Date().toLocaleTimeString(),
      x,
      y,
      resolved: false,
    };
    slide.comments.push(comment);
    return comment;
  }

  public deleteComment(commentId: string): boolean {
    this.commitSnapshot();
    const slide = this.getCurrentSlide();
    if (!slide.comments) return false;
    const len = slide.comments.length;
    slide.comments = slide.comments.filter((c) => c.id !== commentId);
    return slide.comments.length < len;
  }

  public getComments(idx = this.currentSlideIndex): SlideComment[] {
    return this.slides[idx]?.comments || [];
  }

  public toggleShowComments(): boolean {
    this.showCommentsPane = !this.showCommentsPane;
    return this.showCommentsPane;
  }

  public checkSpelling(): { errors: number; suggestions: string[] } {
    this.isSpellingChecked = true;
    return { errors: 0, suggestions: [] };
  }

  public getThesaurus(word: string): string[] {
    this.thesaurusQuery = word;
    const dict: Record<string, string[]> = {
      presentation: ["slideshow", "demonstration", "keynote", "briefing"],
      slide: ["foil", "transparency", "frame", "screen"],
      design: ["layout", "style", "blueprint", "theme"],
      default: ["alternative", "substitute", "equivalent"],
    };
    return dict[word.toLowerCase()] || dict.default;
  }

  public compareWith(deckData: SlideData[]): { diffCount: number } {
    let diffs = Math.abs(this.slides.length - deckData.length);
    const minLen = Math.min(this.slides.length, deckData.length);
    for (let i = 0; i < minLen; i++) {
      if (this.slides[i].shapes.length !== deckData[i].shapes.length) {
        diffs++;
      }
    }
    return { diffCount: diffs };
  }

  // --- Views, Master Views, Show & Zoom ---

  public setViewMode(mode: "normal" | "sorter" | "outline" | "notes" | "reading" | "master"): boolean {
    this.currentView = mode;
    return true;
  }

  public setMasterView(mode: "slideMaster" | "handoutMaster" | "notesMaster" | null): boolean {
    this.masterViewMode = mode;
    this.currentView = mode ? "master" : "normal";
    return true;
  }

  public toggleRuler(): boolean {
    this.showRuler = !this.showRuler;
    return this.showRuler;
  }

  public toggleGridlines(): boolean {
    this.showGridlines = !this.showGridlines;
    return this.showGridlines;
  }

  public toggleGuides(): boolean {
    this.showGuides = !this.showGuides;
    return this.showGuides;
  }

  public toggleSpeakerNotesDrawer(): boolean {
    this.showSpeakerNotesDrawer = !this.showSpeakerNotesDrawer;
    return this.showSpeakerNotesDrawer;
  }

  public setColorMode(mode: "color" | "grayscale" | "bw"): boolean {
    this.colorMode = mode;
    return true;
  }

  public setZoom(level: number): boolean {
    this.zoomLevel = Math.max(10, Math.min(400, level));
    return true;
  }

  public zoomFit(): boolean {
    this.zoomLevel = 100;
    return true;
  }

  // --- Shape CRUD (from 4.3.1) ---

  public insertShape(
    type: PresetShapeKind,
    x = 200,
    y = 150,
    w = 160,
    h = 100,
    fill = "#2563eb"
  ): SlideShape {
    return this.addShape(type, x, y, w, h, fill);
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
    if (!slide) throw new Error("No active slide");

    const newShape: SlideShape = {
      id: `sh_${++this.nextShapeCounter}`,
      type,
      kind: type,
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
    sh.color = color;
    return true;
  }

  public setTextAlign(align: "left" | "center" | "right" | "justify"): boolean {
    const sh = this.getSelectedShape();
    if (!sh) return false;
    this.commitSnapshot();
    sh.textAlign = align;
    return true;
  }

  public toggleTextShadow(): boolean {
    const sh = this.getSelectedShape();
    if (!sh) return false;
    this.commitSnapshot();
    sh.shadow = !sh.shadow;
    sh.textShadow = sh.shadow;
    return true;
  }

  public setShapeShadow(shadow: boolean): boolean {
    const sh = this.getSelectedShape();
    if (!sh) return false;
    this.commitSnapshot();
    sh.shadow = shadow;
    return true;
  }

  public setShapeGlow(glow: boolean): boolean {
    const sh = this.getSelectedShape();
    if (!sh) return false;
    this.commitSnapshot();
    sh.glow = glow;
    return true;
  }

  public setVerticalAlign(align: "top" | "middle" | "bottom"): boolean {
    const sh = this.getSelectedShape();
    if (!sh) return false;
    this.commitSnapshot();
    sh.verticalAlign = align;
    return true;
  }

  public toggleStrikethrough(): boolean {
    const sh = this.getSelectedShape();
    if (!sh) return false;
    this.commitSnapshot();
    sh.strikethrough = !sh.strikethrough;
    return true;
  }

  public clearFormatting(): boolean {
    const sh = this.getSelectedShape();
    if (!sh) return false;
    this.commitSnapshot();
    sh.bold = false;
    sh.italic = false;
    sh.underline = false;
    sh.shadow = false;
    sh.glow = false;
    sh.fontSize = 20;
    sh.fontFamily = "Arial";
    sh.textColor = "#1e293b";
    sh.textAlign = "left";
    return true;
  }

  public toggleBulletList(): boolean {
    const sh = this.getSelectedShape();
    if (!sh || !sh.text) return false;
    this.commitSnapshot();
    const lines = sh.text.split("\n");
    const hasBullets = lines.some((l) => l.trim().startsWith("•"));
    if (hasBullets) {
      sh.text = lines.map((l) => l.replace(/^\s*•\s*/, "")).join("\n");
    } else {
      sh.text = lines.map((l) => (l.trim().length > 0 ? `• ${l.trim()}` : l)).join("\n");
    }
    return true;
  }

  public toggleNumberedList(): boolean {
    const sh = this.getSelectedShape();
    if (!sh || !sh.text) return false;
    this.commitSnapshot();
    const lines = sh.text.split("\n");
    const hasNumbering = lines.some((l) => /^\s*\d+[\.)]\s*/.test(l));
    if (hasNumbering) {
      sh.text = lines.map((l) => l.replace(/^\s*\d+[\.)]\s*/, "")).join("\n");
    } else {
      let counter = 1;
      sh.text = lines
        .map((l) => (l.trim().length > 0 ? `${counter++}. ${l.trim()}` : l))
        .join("\n");
    }
    return true;
  }

  public setLineSpacing(_spacing: number): boolean {
    return true;
  }

  public rotateSelectedShape(deg = 90): boolean {
    const sel = this.getSelectedShape();
    if (!sel) return false;
    this.commitSnapshot();
    sel.rotation = ((sel.rotation || 0) + deg) % 360;
    return true;
  }

  public distributeShapes(direction: "horizontal" | "vertical"): boolean {
    const slide = this.getCurrentSlide();
    if (!slide || slide.shapes.length < 3) return false;
    this.commitSnapshot();
    if (direction === "horizontal") {
      const sorted = [...slide.shapes].sort((a, b) => a.x - b.x);
      const minX = sorted[0].x;
      const maxX = sorted[sorted.length - 1].x;
      const totalSpan = maxX - minX;
      const step = totalSpan / (sorted.length - 1);
      sorted.forEach((sh, i) => {
        sh.x = minX + i * step;
      });
    } else {
      const sorted = [...slide.shapes].sort((a, b) => a.y - b.y);
      const minY = sorted[0].y;
      const maxY = sorted[sorted.length - 1].y;
      const totalSpan = maxY - minY;
      const step = totalSpan / (sorted.length - 1);
      sorted.forEach((sh, i) => {
        sh.y = minY + i * step;
      });
    }
    return true;
  }

  public groupShapes(): boolean {
    const slide = this.getCurrentSlide();
    if (!slide || slide.shapes.length < 2) return false;
    this.commitSnapshot();
    const grp = `grp_${Date.now()}`;
    slide.shapes.forEach((s) => (s.groupId = grp));
    return true;
  }

  public ungroupShapes(): boolean {
    const slide = this.getCurrentSlide();
    if (!slide) return false;
    this.commitSnapshot();
    slide.shapes.forEach((s) => delete s.groupId);
    return true;
  }

  public insertSmartArt(type: "process" | "cycle" | "hierarchy" | "pyramid" = "process"): SlideShape {
    this.commitSnapshot();
    const slide = this.getCurrentSlide();
    const id = `sh_smartart_${++this.nextShapeCounter}`;
    let nodes: { id: string; label: string; color?: string }[] = [];
    if (type === "process") {
      nodes = [
        { id: "1", label: "Step 1: Initiation", color: "#2563eb" },
        { id: "2", label: "Step 2: Execution", color: "#0284c7" },
        { id: "3", label: "Step 3: Delivery", color: "#10b981" },
      ];
    } else if (type === "cycle") {
      nodes = [
        { id: "1", label: "Phase 1: Research", color: "#2563eb" },
        { id: "2", label: "Phase 2: Build", color: "#7c3aed" },
        { id: "3", label: "Phase 3: Test", color: "#ec4899" },
        { id: "4", label: "Phase 4: Launch", color: "#10b981" },
      ];
    } else if (type === "hierarchy") {
      nodes = [
        { id: "1", label: "Executive Leadership", color: "#1e3a8a" },
        { id: "2", label: "Product & Engineering", color: "#2563eb" },
        { id: "3", label: "Design & UX", color: "#0284c7" },
        { id: "4", label: "Operations & Sales", color: "#0d9488" },
      ];
    } else {
      nodes = [
        { id: "1", label: "Tier 1: Foundation", color: "#1e293b" },
        { id: "2", label: "Tier 2: Capabilities", color: "#2563eb" },
        { id: "3", label: "Tier 3: Strategic Vision", color: "#38bdf8" },
      ];
    }
    const shape: SlideShape = {
      id,
      type: "smartArt",
      kind: "smartArt",
      smartArtType: type,
      smartArtNodes: nodes,
      x: 160,
      y: 120,
      w: 640,
      h: 280,
      fill: "transparent",
      text: `SmartArt: ${type.toUpperCase()}`,
      textColor: "#ffffff",
      fontSize: 16,
    };
    slide.shapes.push(shape);
    this.selectedShapeId = id;
    return shape;
  }

  public insertChart(type: "bar" | "line" | "pie" = "bar"): SlideShape {
    this.commitSnapshot();
    const slide = this.getCurrentSlide();
    const id = `sh_chart_${++this.nextShapeCounter}`;
    const chartData = {
      categories: ["Q1", "Q2", "Q3", "Q4"],
      series: [
        { name: "Current Year", values: [65, 82, 90, 115] },
        { name: "Prior Year", values: [50, 68, 75, 95] },
      ],
    };
    const shape: SlideShape = {
      id,
      type: "chart",
      kind: "chart",
      chartType: type,
      chartData,
      x: 180,
      y: 110,
      w: 600,
      h: 320,
      fill: "#ffffff",
      stroke: "#cbd5e1",
      strokeWidth: 1,
      text: `Chart (${type})`,
      textColor: "#1e293b",
      fontSize: 14,
    };
    slide.shapes.push(shape);
    this.selectedShapeId = id;
    return shape;
  }

  public insertIcon(iconName = "⭐"): SlideShape {
    this.commitSnapshot();
    const slide = this.getCurrentSlide();
    const id = `sh_icon_${++this.nextShapeCounter}`;
    const shape: SlideShape = {
      id,
      type: "icon",
      kind: "icon",
      iconName,
      text: iconName,
      x: 420,
      y: 210,
      w: 120,
      h: 120,
      fill: "transparent",
      fontSize: 54,
      textAlign: "center",
      textColor: "#2563eb",
    };
    slide.shapes.push(shape);
    this.selectedShapeId = id;
    return shape;
  }

  public insertWordArt(text = "WordArt", style = "rainbow"): SlideShape {
    this.commitSnapshot();
    const slide = this.getCurrentSlide();
    const id = `sh_wordart_${++this.nextShapeCounter}`;
    const shape: SlideShape = {
      id,
      type: "wordArt",
      kind: "wordArt",
      wordArtStyle: style,
      text,
      x: 260,
      y: 220,
      w: 440,
      h: 90,
      fill: "transparent",
      textColor: "#2563eb",
      fontSize: 44,
      bold: true,
      shadow: true,
      fontFamily: "Impact, Segoe UI, sans-serif",
      textAlign: "center",
    };
    slide.shapes.push(shape);
    this.selectedShapeId = id;
    return shape;
  }

  public insertHeaderFooter(): SlideShape[] {
    this.commitSnapshot();
    const slide = this.getCurrentSlide();
    const dateShape: SlideShape = {
      id: `sh_date_${++this.nextShapeCounter}`,
      type: "text",
      isHeaderFooter: true,
      headerFooterType: "date",
      text: new Date().toLocaleDateString(),
      x: 80,
      y: 500,
      w: 200,
      h: 24,
      fill: "transparent",
      textColor: "#64748b",
      fontSize: 12,
    };
    const footerShape: SlideShape = {
      id: `sh_footer_${++this.nextShapeCounter}`,
      type: "text",
      isHeaderFooter: true,
      headerFooterType: "footer",
      text: "Presentation Deck | Confidential",
      x: 340,
      y: 500,
      w: 280,
      h: 24,
      fill: "transparent",
      textColor: "#64748b",
      fontSize: 12,
      textAlign: "center",
    };
    const numShape: SlideShape = {
      id: `sh_slidenum_${++this.nextShapeCounter}`,
      type: "text",
      isHeaderFooter: true,
      headerFooterType: "slideNumber",
      text: `<#>`,
      x: 800,
      y: 500,
      w: 80,
      h: 24,
      fill: "transparent",
      textColor: "#64748b",
      fontSize: 12,
      textAlign: "right",
    };
    slide.shapes.push(dateShape, footerShape, numShape);
    return [dateShape, footerShape, numShape];
  }

  public insertSlideNumber(): SlideShape {
    this.commitSnapshot();
    const slide = this.getCurrentSlide();
    const id = `sh_slidenum_${++this.nextShapeCounter}`;
    const shape: SlideShape = {
      id,
      type: "text",
      isHeaderFooter: true,
      headerFooterType: "slideNumber",
      text: `Slide ${this.currentSlideIndex + 1}`,
      x: 820,
      y: 500,
      w: 100,
      h: 28,
      fill: "transparent",
      textColor: "#64748b",
      fontSize: 12,
      textAlign: "right",
    };
    slide.shapes.push(shape);
    this.selectedShapeId = id;
    return shape;
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
      case "NewSlideWithLayout":
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
      case "ToggleHideSlide":
        return this.toggleHideSlide();
      case "StartShowBeginning":
        this.startShow(true, false);
        return true;
      case "StartShowCurrent":
        this.startShow(false, false);
        return true;
      case "StartPresenterView":
      case "StartDualScreenPresenter":
        this.startShow(false, true);
        return true;
      case "TogglePresenterView":
        this.togglePresenterView();
        return true;
      case "SetUpShow":
      case "SetLoopContinuously":
        return this.setLoopContinuously(typeof payload === "boolean" ? payload : !this.loopContinuously);
      case "RehearseTimings":
        if (this.isRehearsingTimings) {
          return this.stopRehearsingTimings();
        } else {
          return this.startRehearsingTimings();
        }
      case "StopRehearseTimings":
        return this.stopRehearsingTimings();
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
      case "PreviewAnimation":
        this.isPreviewingAnimation = true;
        return true;
      case "ToggleAnimationPane":
        this.showAnimationPane = !this.showAnimationPane;
        return true;
      case "SetAnimation": {
        const eff = (typeof payload === "string" ? payload : "fadeIn") as AnimationEffect;
        const sel = this.getSelectedShape();
        if (sel) {
          if (String(eff).toLowerCase() === "none") {
            const slide = this.slides[this.currentSlideIndex];
            if (slide && slide.animations) {
              slide.animations = slide.animations.filter((a) => a.shapeId !== sel.id);
            }
            return true;
          }
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
      case "ResetSlide":
        return this.resetSlide();
      case "ToggleShadow":
      case "ToggleTextShadow":
        return this.toggleTextShadow();
      case "SetShapeShadow":
      case "ToggleShapeShadow":
        return this.setShapeShadow(typeof payload === "boolean" ? payload : true);
      case "SetShapeGlow":
      case "ToggleShapeGlow":
        return this.setShapeGlow(typeof payload === "boolean" ? payload : true);
      case "ToggleStrikethrough":
        return this.toggleStrikethrough();
      case "ClearFormatting":
        return this.clearFormatting();
      case "ToggleBulletList":
      case "ToggleBullets":
        return this.toggleBulletList();
      case "ToggleNumberedList":
      case "ToggleNumbering":
        return this.toggleNumberedList();
      case "SetLineSpacing":
        return this.setLineSpacing(Number(payload));
      case "SetVerticalAlign":
        return this.setVerticalAlign(payload as "top" | "middle" | "bottom");
      case "DistributeH":
      case "DistributeHorizontally":
        return this.distributeShapes("horizontal");
      case "DistributeV":
      case "DistributeVertically":
        return this.distributeShapes("vertical");
      case "Group":
      case "GroupShapes":
        return this.groupShapes();
      case "Ungroup":
      case "UngroupShapes":
        return this.ungroupShapes();
      case "Rotate90":
      case "RotateShape":
        return this.rotateSelectedShape(typeof payload === "number" ? payload : 90);
      case "InsertSmartArt":
        this.insertSmartArt(typeof payload === "string" ? (payload as any) : "process");
        return true;
      case "InsertChart":
        this.insertChart(typeof payload === "string" ? (payload as any) : "bar");
        return true;
      case "InsertIcon":
      case "InsertIcons":
        this.insertIcon(typeof payload === "string" ? payload : "⭐");
        return true;
      case "InsertWordArt":
        this.insertWordArt(typeof payload === "string" ? payload : "WordArt");
        return true;
      case "InsertHeaderFooter":
        this.insertHeaderFooter();
        return true;
      case "InsertSlideNumber":
        this.insertSlideNumber();
        return true;
      case "SetVariant":
        this.setVariant(String(payload));
        return true;
      case "SetTransitionSound":
        return this.setTransitionSound(String(payload));
      case "ToggleAdvanceOnClick": {
        const current = this.getCurrentSlide()?.transitionConfig?.advanceOnClick ?? true;
        const nextVal = typeof payload === "boolean" ? payload : !current;
        return this.setAdvanceOnClick(nextVal);
      }
      case "SetAdvanceDelay":
        return this.setAdvanceAfter(Number(payload));
      case "ToggleLaser":
        this.toggleLaser();
        return true;
      case "InsertShape": {
        const raw = typeof payload === "string" ? payload.toLowerCase().replace(/[\s_-]/g, "") : "rect";
        let kind: PresetShapeKind = "rect";
        if (raw === "roundrect" || raw === "rounded" || raw === "roundedrect" || raw === "roundedrectangle") kind = "roundRect";
        else if (raw === "ellipse" || raw === "circle" || raw === "oval") kind = "ellipse";
        else if (raw === "triangle") kind = "triangle";
        else if (raw === "arrow" || raw === "arrowright") kind = "arrow";
        else if (raw === "arrowleft") kind = "arrowLeft";
        else if (raw === "arrowup") kind = "arrowUp";
        else if (raw === "arrowdown") kind = "arrowDown";
        else if (raw === "star") kind = "star";
        else if (raw === "diamond") kind = "diamond";
        else if (raw === "callout") kind = "callout";
        else if (raw === "heart") kind = "heart";
        else if (raw === "cloud") kind = "cloud";
        else if (raw === "pentagon") kind = "pentagon";
        else if (raw === "hexagon") kind = "hexagon";
        else if (raw === "banner") kind = "banner";
        else if (raw === "text" || raw === "textbox") kind = "text";
        this.addShape(kind);
        return true;
      }
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
        } else if (payload && typeof payload === "object") {
          const p = payload as { background?: string; bg?: string; applyToAll?: boolean };
          const bg = p.background || p.bg || this.getCurrentSlide()?.background || "#0f172a";
          this.setSlideBackground(bg, !!p.applyToAll);
          return true;
        }
        return false;
      case "InsertImage":
      case "InsertPicture": {
        let url: string | undefined;
        let x = 200;
        let y = 150;
        let w = 280;
        let h = 180;
        if (typeof payload === "string") {
          url = payload;
        } else if (payload && typeof payload === "object") {
          const p = payload as any;
          url = p.url || p.src || undefined;
          if (p.x !== undefined) x = p.x;
          if (p.y !== undefined) y = p.y;
          if (p.width !== undefined) w = p.width;
          else if (p.w !== undefined) w = p.w;
          if (p.height !== undefined) h = p.height;
          else if (p.h !== undefined) h = p.h;
        }
        this.insertImage(url, x, y, w, h);
        return true;
      }
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
        this.setTextAlign("left");
        return this.alignSelectedShape("left");
      case "AlignCenter":
        this.setTextAlign("center");
        return this.alignSelectedShape("center");
      case "AlignRight":
        this.setTextAlign("right");
        return this.alignSelectedShape("right");
      case "AlignJustify":
        return this.setTextAlign("justify");
      case "AlignTop":
        this.setVerticalAlign("top");
        return this.alignSelectedShape("top");
      case "AlignMiddle":
        this.setVerticalAlign("middle");
        return this.alignSelectedShape("middle");
      case "AlignBottom":
        this.setVerticalAlign("bottom");
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

      // Review Tab Commands
      case "CheckSpelling":
      case "Spelling":
        this.checkSpelling();
        return true;
      case "Thesaurus":
        this.getThesaurus(typeof payload === "string" ? payload : "presentation");
        return true;
      case "NewComment":
        this.addComment(typeof payload === "string" ? payload : "Review note");
        return true;
      case "ShowComments":
      case "ToggleComments":
        return this.toggleShowComments();
      case "DeleteComment": {
        const comments = this.getComments();
        const id = payload ? String(payload) : (comments.length > 0 ? comments[comments.length - 1].id : "");
        return this.deleteComment(id);
      }
      case "Compare":
      case "ComparePresentations":
        this.compareWith([]);
        return true;

      // View Tab Commands
      case "ViewNormal":
        return this.setViewMode("normal");
      case "ViewSlideSorter":
        return this.setViewMode("sorter");
      case "ViewOutline":
        return this.setViewMode("outline");
      case "ViewNotesPage":
        return this.setViewMode("notes");
      case "ViewReadingView":
        return this.setViewMode("reading");
      case "ViewSlideMaster":
        return this.setMasterView("slideMaster");
      case "ViewHandoutMaster":
        return this.setMasterView("handoutMaster");
      case "ViewNotesMaster":
        return this.setMasterView("notesMaster");
      case "ToggleRuler":
        return this.toggleRuler();
      case "ToggleGridlines":
        return this.toggleGridlines();
      case "ToggleGuides":
        return this.toggleGuides();
      case "ToggleNotes":
        return this.toggleSpeakerNotesDrawer();
      case "SetColorMode":
        return this.setColorMode(payload as "color" | "grayscale" | "bw");
      case "ZoomFit":
        return this.zoomFit();
      case "Zoom100":
        return this.setZoom(100);
      case "SetZoom":
        return this.setZoom(Number(payload) || 100);
      default:
        return false;
    }
  }
}

export const slidesState = new SlidesState();
