/**
 * Presenter Studio Dual-Screen Mode & Audience Annotation Laser (Subphase 19.1.3)
 *
 * Implements:
 * - Multi-window / dual-screen Presenter Console
 * - Synchronized audience display mirroring with low-latency event bus
 * - Active slide viewport, next slide preview card, elapsed/remaining clock timer
 * - Teleprompter formatted speaker notes with adjustable font sizing
 * - Live audience laser pointer (glowing dot) & stylus ink annotation overlay
 * - Keyboard shortcuts: B (black screen), W (white screen), P (pen), L (laser), E (erase), Esc (exit)
 */

import { SlidesState, SlideData } from "../state/slidesState";

export interface InkStroke {
  points: Array<{ x: number; y: number }>;
  color: string;
  width: number;
}

export type AnnotationTool = "arrow" | "laser" | "pen" | "eraser";

export interface PresenterSyncPayload {
  type: "SLIDE_CHANGE" | "LASER_MOVE" | "INK_STROKE" | "CLEAR_INK" | "SCREEN_MODE";
  slideIndex?: number;
  step?: number;
  laser?: { active: boolean; x: number; y: number };
  stroke?: InkStroke;
  screen?: { black: boolean; white: boolean };
}

export class PresenterConsoleWindow {
  private container: HTMLElement;
  private rootEl: HTMLElement;
  private audienceEl?: HTMLElement;
  private state: SlidesState;

  // View elements
  private currentCanvas!: HTMLCanvasElement;
  private currentCtx!: CanvasRenderingContext2D;
  private nextCanvas!: HTMLCanvasElement;
  private nextCtx!: CanvasRenderingContext2D;
  private notesContentEl!: HTMLElement;
  private timerEl!: HTMLElement;
  private clockEl!: HTMLElement;
  private audienceLaserDot?: HTMLElement;
  private audienceInkCanvas?: HTMLCanvasElement;
  private audienceInkCtx?: CanvasRenderingContext2D;

  // State
  private isOpenState = false;
  private activeTool: AnnotationTool = "arrow";
  private isBlackScreen = false;
  private isWhiteScreen = false;
  private timerSeconds = 0;
  private timerInterval?: any;
  private timerPaused = false;
  private notesFontSize = 16;
  private inkStrokes: InkStroke[] = [];
  private currentStroke: Array<{ x: number; y: number }> = [];
  private isDrawing = false;
  private penColor = "#ef4444";
  private penWidth = 3;

  // Sync Channel
  private syncChannel?: BroadcastChannel;
  private onExitCallback?: () => void;
  private onSlideChangeCallback?: (idx: number) => void;

  constructor(
    containerOrId: HTMLElement | string = "app",
    state: SlidesState,
    audienceContainer?: HTMLElement
  ) {
    this.state = state;
    if (typeof containerOrId === "string") {
      this.container = document.getElementById(containerOrId) || document.body;
    } else {
      this.container = containerOrId;
    }
    this.audienceEl = audienceContainer;

    this.rootEl = document.createElement("div");
    this.rootEl.className = "presenter-console-window";
    this.rootEl.style.position = "fixed";
    this.rootEl.style.inset = "0";
    this.rootEl.style.backgroundColor = "#09090b";
    this.rootEl.style.color = "#f8fafc";
    this.rootEl.style.zIndex = "99999";
    this.rootEl.style.display = "none";
    this.rootEl.style.flexDirection = "column";
    this.rootEl.style.fontFamily = "Segoe UI, -apple-system, sans-serif";
    this.container.appendChild(this.rootEl);

    try {
      this.syncChannel = new BroadcastChannel("aether-presenter-studio-sync");
      this.syncChannel.onmessage = (ev) => this.handleSyncMessage(ev.data);
    } catch {
      // Fallback if BroadcastChannel is not supported in environment
    }

    this.initLayout();
    this.attachKeyboardListener();
  }

  public setCallbacks(callbacks: {
    onExit?: () => void;
    onSlideChange?: (idx: number) => void;
  }): void {
    this.onExitCallback = callbacks.onExit;
    this.onSlideChangeCallback = callbacks.onSlideChange;
  }

  public setAudienceContainer(audienceEl: HTMLElement): void {
    this.audienceEl = audienceEl;
    this.initAudienceOverlay();
  }

  public open(): void {
    this.isOpenState = true;
    this.rootEl.style.display = "flex";
    this.startTimer();
    this.render();
  }

  public close(): void {
    this.isOpenState = false;
    this.rootEl.style.display = "none";
    this.stopTimer();
    if (this.onExitCallback) {
      this.onExitCallback();
    }
  }

  public isOpen(): boolean {
    return this.isOpenState;
  }

  public getActiveTool(): AnnotationTool {
    return this.activeTool;
  }

  public setTool(tool: AnnotationTool): void {
    this.activeTool = tool;
    this.updateToolButtons();
  }

  public getTimerSeconds(): number {
    return this.timerSeconds;
  }

  public isBlackScreenActive(): boolean {
    return this.isBlackScreen;
  }

  public isWhiteScreenActive(): boolean {
    return this.isWhiteScreen;
  }

  public toggleBlackScreen(): void {
    this.isBlackScreen = !this.isBlackScreen;
    if (this.isBlackScreen) this.isWhiteScreen = false;
    this.broadcastSync({
      type: "SCREEN_MODE",
      screen: { black: this.isBlackScreen, white: this.isWhiteScreen },
    });
    this.render();
  }

  public toggleWhiteScreen(): void {
    this.isWhiteScreen = !this.isWhiteScreen;
    if (this.isWhiteScreen) this.isBlackScreen = false;
    this.broadcastSync({
      type: "SCREEN_MODE",
      screen: { black: this.isBlackScreen, white: this.isWhiteScreen },
    });
    this.render();
  }

  public clearInk(): void {
    this.inkStrokes = [];
    if (this.audienceInkCtx && this.audienceInkCanvas) {
      this.audienceInkCtx.clearRect(0, 0, this.audienceInkCanvas.width, this.audienceInkCanvas.height);
    }
    this.broadcastSync({ type: "CLEAR_INK" });
    this.render();
  }

  public nextSlide(): void {
    if (this.state.currentSlideIndex < this.state.slides.length - 1) {
      this.state.selectSlide(this.state.currentSlideIndex + 1);
      this.notifySlideChanged();
    }
  }

  public prevSlide(): void {
    if (this.state.currentSlideIndex > 0) {
      this.state.selectSlide(this.state.currentSlideIndex - 1);
      this.notifySlideChanged();
    }
  }

  public jumpToSlide(idx: number): void {
    if (idx >= 0 && idx < this.state.slides.length) {
      this.state.selectSlide(idx);
      this.notifySlideChanged();
    }
  }

  private notifySlideChanged(): void {
    this.clearInk();
    this.broadcastSync({
      type: "SLIDE_CHANGE",
      slideIndex: this.state.currentSlideIndex,
    });
    if (this.onSlideChangeCallback) {
      this.onSlideChangeCallback(this.state.currentSlideIndex);
    }
    this.render();
  }

  private initLayout(): void {
    // 1. Top Header Bar
    const header = document.createElement("div");
    header.className = "presenter-header";
    header.style.display = "flex";
    header.style.alignItems = "center";
    header.style.justifyContent = "space-between";
    header.style.padding = "10px 20px";
    header.style.backgroundColor = "#18181b";
    header.style.borderBottom = "1px solid #27272a";

    const titleWrap = document.createElement("div");
    titleWrap.style.display = "flex";
    titleWrap.style.alignItems = "center";
    titleWrap.style.gap = "10px";
    titleWrap.innerHTML = `
      <span style="font-size:18px;">🖥️</span>
      <span style="font-weight:700;font-size:15px;color:#38bdf8;">Presenter Studio</span>
      <span style="font-size:12px;background:#27272a;padding:2px 8px;border-radius:12px;color:#94a3b8;">Dual-Screen Active</span>
    `;

    const timerArea = document.createElement("div");
    timerArea.style.display = "flex";
    timerArea.style.alignItems = "center";
    timerArea.style.gap = "12px";

    this.timerEl = document.createElement("span");
    this.timerEl.className = "presenter-timer-display";
    this.timerEl.style.fontFamily = "monospace";
    this.timerEl.style.fontSize = "16px";
    this.timerEl.style.fontWeight = "600";
    this.timerEl.style.color = "#38bdf8";
    this.timerEl.textContent = "⏱️ 00:00:00";

    const pauseBtn = document.createElement("button");
    pauseBtn.className = "presenter-btn-timer-pause";
    pauseBtn.textContent = "⏸";
    pauseBtn.title = "Pause / Resume Timer";
    pauseBtn.style.background = "#27272a";
    pauseBtn.style.color = "#f8fafc";
    pauseBtn.style.border = "none";
    pauseBtn.style.borderRadius = "4px";
    pauseBtn.style.padding = "4px 8px";
    pauseBtn.style.cursor = "pointer";
    pauseBtn.onclick = () => {
      this.timerPaused = !this.timerPaused;
      pauseBtn.textContent = this.timerPaused ? "▶" : "⏸";
    };

    const resetBtn = document.createElement("button");
    resetBtn.className = "presenter-btn-timer-reset";
    resetBtn.textContent = "🔄";
    resetBtn.title = "Reset Timer";
    resetBtn.style.background = "#27272a";
    resetBtn.style.color = "#f8fafc";
    resetBtn.style.border = "none";
    resetBtn.style.borderRadius = "4px";
    resetBtn.style.padding = "4px 8px";
    resetBtn.style.cursor = "pointer";
    resetBtn.onclick = () => {
      this.timerSeconds = 0;
      this.updateTimerDisplay();
    };

    this.clockEl = document.createElement("span");
    this.clockEl.className = "presenter-clock-display";
    this.clockEl.style.fontSize = "13px";
    this.clockEl.style.color = "#94a3b8";
    this.clockEl.textContent = new Date().toLocaleTimeString();

    timerArea.appendChild(this.timerEl);
    timerArea.appendChild(pauseBtn);
    timerArea.appendChild(resetBtn);
    timerArea.appendChild(this.clockEl);

    const closeBtn = document.createElement("button");
    closeBtn.className = "presenter-btn-exit";
    closeBtn.textContent = "✕ Exit Studio";
    closeBtn.style.background = "#ef4444";
    closeBtn.style.color = "#ffffff";
    closeBtn.style.border = "none";
    closeBtn.style.borderRadius = "4px";
    closeBtn.style.padding = "6px 14px";
    closeBtn.style.fontWeight = "600";
    closeBtn.style.cursor = "pointer";
    closeBtn.onclick = () => this.close();

    header.appendChild(titleWrap);
    header.appendChild(timerArea);
    header.appendChild(closeBtn);
    this.rootEl.appendChild(header);

    // 2. Middle 3-Pane Body
    const body = document.createElement("div");
    body.className = "presenter-body";
    body.style.display = "flex";
    body.style.flex = "1";
    body.style.padding = "16px";
    body.style.gap = "16px";
    body.style.overflow = "hidden";

    // Pane 1: Active Slide Viewport
    const activePane = document.createElement("div");
    activePane.className = "presenter-active-pane";
    activePane.style.flex = "3";
    activePane.style.display = "flex";
    activePane.style.flexDirection = "column";
    activePane.style.backgroundColor = "#18181b";
    activePane.style.borderRadius = "8px";
    activePane.style.padding = "12px";
    activePane.style.border = "1px solid #27272a";

    const activeTitle = document.createElement("div");
    activeTitle.style.fontSize = "12px";
    activeTitle.style.fontWeight = "600";
    activeTitle.style.color = "#94a3b8";
    activeTitle.style.marginBottom = "8px";
    activeTitle.textContent = "CURRENT SLIDE (AUDIENCE VIEW)";
    activePane.appendChild(activeTitle);

    const canvasWrap = document.createElement("div");
    canvasWrap.style.flex = "1";
    canvasWrap.style.display = "flex";
    canvasWrap.style.alignItems = "center";
    canvasWrap.style.justifyContent = "center";
    canvasWrap.style.position = "relative";
    canvasWrap.style.backgroundColor = "#000000";
    canvasWrap.style.borderRadius = "6px";
    canvasWrap.style.overflow = "hidden";

    this.currentCanvas = document.createElement("canvas");
    this.currentCanvas.className = "presenter-current-canvas";
    this.currentCtx = this.currentCanvas.getContext("2d")!;
    canvasWrap.appendChild(this.currentCanvas);

    // Attach Pointer events for Laser and Pen on Active Slide
    this.attachCanvasInteraction(canvasWrap);
    activePane.appendChild(canvasWrap);

    // Annotation Toolbar under Active Slide
    const toolBar = this.createAnnotationToolbar();
    activePane.appendChild(toolBar);
    body.appendChild(activePane);

    // Pane 2: Next Slide Card Preview
    const nextPane = document.createElement("div");
    nextPane.className = "presenter-next-pane";
    nextPane.style.flex = "2";
    nextPane.style.display = "flex";
    nextPane.style.flexDirection = "column";
    nextPane.style.backgroundColor = "#18181b";
    nextPane.style.borderRadius = "8px";
    nextPane.style.padding = "12px";
    nextPane.style.border = "1px solid #27272a";

    const nextTitle = document.createElement("div");
    nextTitle.style.fontSize = "12px";
    nextTitle.style.fontWeight = "600";
    nextTitle.style.color = "#94a3b8";
    nextTitle.style.marginBottom = "8px";
    nextTitle.textContent = "UPCOMING NEXT SLIDE";
    nextPane.appendChild(nextTitle);

    const nextCanvasWrap = document.createElement("div");
    nextCanvasWrap.style.flex = "1";
    nextCanvasWrap.style.display = "flex";
    nextCanvasWrap.style.alignItems = "center";
    nextCanvasWrap.style.justifyContent = "center";
    nextCanvasWrap.style.backgroundColor = "#000000";
    nextCanvasWrap.style.borderRadius = "6px";
    nextCanvasWrap.style.overflow = "hidden";

    this.nextCanvas = document.createElement("canvas");
    this.nextCanvas.className = "presenter-next-canvas";
    this.nextCtx = this.nextCanvas.getContext("2d")!;
    nextCanvasWrap.appendChild(this.nextCanvas);
    nextPane.appendChild(nextCanvasWrap);
    body.appendChild(nextPane);

    // Pane 3: Teleprompter Speaker Notes
    const notesPane = document.createElement("div");
    notesPane.className = "presenter-notes-pane";
    notesPane.style.flex = "2";
    notesPane.style.display = "flex";
    notesPane.style.flexDirection = "column";
    notesPane.style.backgroundColor = "#18181b";
    notesPane.style.borderRadius = "8px";
    notesPane.style.padding = "12px";
    notesPane.style.border = "1px solid #27272a";

    const notesHeader = document.createElement("div");
    notesHeader.style.display = "flex";
    notesHeader.style.alignItems = "center";
    notesHeader.style.justifyContent = "space-between";
    notesHeader.style.marginBottom = "8px";

    const notesTitle = document.createElement("span");
    notesTitle.style.fontSize = "12px";
    notesTitle.style.fontWeight = "600";
    notesTitle.style.color = "#94a3b8";
    notesTitle.textContent = "SPEAKER NOTES";

    const fontBtns = document.createElement("div");
    fontBtns.style.display = "flex";
    fontBtns.style.gap = "4px";

    const incFontBtn = document.createElement("button");
    incFontBtn.textContent = "A+";
    incFontBtn.style.padding = "2px 6px";
    incFontBtn.style.cursor = "pointer";
    incFontBtn.onclick = () => {
      this.notesFontSize = Math.min(32, this.notesFontSize + 2);
      this.notesContentEl.style.fontSize = `${this.notesFontSize}px`;
    };

    const decFontBtn = document.createElement("button");
    decFontBtn.textContent = "A-";
    decFontBtn.style.padding = "2px 6px";
    decFontBtn.style.cursor = "pointer";
    decFontBtn.onclick = () => {
      this.notesFontSize = Math.max(12, this.notesFontSize - 2);
      this.notesContentEl.style.fontSize = `${this.notesFontSize}px`;
    };

    fontBtns.appendChild(decFontBtn);
    fontBtns.appendChild(incFontBtn);
    notesHeader.appendChild(notesTitle);
    notesHeader.appendChild(fontBtns);
    notesPane.appendChild(notesHeader);

    this.notesContentEl = document.createElement("div");
    this.notesContentEl.className = "presenter-notes-content";
    this.notesContentEl.style.flex = "1";
    this.notesContentEl.style.overflowY = "auto";
    this.notesContentEl.style.backgroundColor = "#09090b";
    this.notesContentEl.style.padding = "12px";
    this.notesContentEl.style.borderRadius = "6px";
    this.notesContentEl.style.lineHeight = "1.6";
    this.notesContentEl.style.fontSize = `${this.notesFontSize}px`;
    this.notesContentEl.style.color = "#e2e8f0";
    notesPane.appendChild(this.notesContentEl);
    body.appendChild(notesPane);

    this.rootEl.appendChild(body);

    // 3. Bottom Slide Sorter Quick Navigation Filmstrip
    const filmstrip = document.createElement("div");
    filmstrip.className = "presenter-filmstrip";
    filmstrip.style.display = "flex";
    filmstrip.style.alignItems = "center";
    filmstrip.style.gap = "10px";
    filmstrip.style.padding = "8px 16px";
    filmstrip.style.backgroundColor = "#18181b";
    filmstrip.style.borderTop = "1px solid #27272a";
    filmstrip.style.overflowX = "auto";
    this.rootEl.appendChild(filmstrip);

    if (this.audienceEl) {
      this.initAudienceOverlay();
    }
  }

  private createAnnotationToolbar(): HTMLElement {
    const bar = document.createElement("div");
    bar.className = "presenter-annotation-toolbar";
    bar.style.display = "flex";
    bar.style.alignItems = "center";
    bar.style.gap = "8px";
    bar.style.marginTop = "8px";

    const createBtn = (label: string, tool: AnnotationTool, keyHint: string) => {
      const btn = document.createElement("button");
      btn.className = `presenter-tool-btn presenter-tool-${tool}`;
      btn.textContent = `${label} (${keyHint})`;
      btn.style.background = this.activeTool === tool ? "#38bdf8" : "#27272a";
      btn.style.color = this.activeTool === tool ? "#000000" : "#ffffff";
      btn.style.border = "none";
      btn.style.borderRadius = "4px";
      btn.style.padding = "4px 8px";
      btn.style.fontSize = "12px";
      btn.style.cursor = "pointer";
      btn.onclick = () => this.setTool(tool);
      return btn;
    };

    bar.appendChild(createBtn("↖ Arrow", "arrow", "A"));
    bar.appendChild(createBtn("🔴 Laser", "laser", "L"));
    bar.appendChild(createBtn("🖊️ Pen", "pen", "P"));
    bar.appendChild(createBtn("🧹 Eraser", "eraser", "E"));

    const sep = document.createElement("span");
    sep.style.color = "#52525b";
    sep.textContent = "|";
    bar.appendChild(sep);

    const blackBtn = document.createElement("button");
    blackBtn.className = "presenter-btn-black";
    blackBtn.textContent = "⬛ Black (B)";
    blackBtn.style.background = "#27272a";
    blackBtn.style.color = "#ffffff";
    blackBtn.style.border = "none";
    blackBtn.style.borderRadius = "4px";
    blackBtn.style.padding = "4px 8px";
    blackBtn.style.fontSize = "12px";
    blackBtn.style.cursor = "pointer";
    blackBtn.onclick = () => this.toggleBlackScreen();
    bar.appendChild(blackBtn);

    const whiteBtn = document.createElement("button");
    whiteBtn.className = "presenter-btn-white";
    whiteBtn.textContent = "⬜ White (W)";
    whiteBtn.style.background = "#27272a";
    whiteBtn.style.color = "#ffffff";
    whiteBtn.style.border = "none";
    whiteBtn.style.borderRadius = "4px";
    whiteBtn.style.padding = "4px 8px";
    whiteBtn.style.fontSize = "12px";
    whiteBtn.style.cursor = "pointer";
    whiteBtn.onclick = () => this.toggleWhiteScreen();
    bar.appendChild(whiteBtn);

    const prevBtn = document.createElement("button");
    prevBtn.className = "presenter-btn-prev";
    prevBtn.textContent = "◀ Prev";
    prevBtn.style.background = "#3b82f6";
    prevBtn.style.color = "#ffffff";
    prevBtn.style.border = "none";
    prevBtn.style.borderRadius = "4px";
    prevBtn.style.padding = "4px 10px";
    prevBtn.style.fontSize = "12px";
    prevBtn.style.cursor = "pointer";
    prevBtn.onclick = () => this.prevSlide();
    bar.appendChild(prevBtn);

    const nextBtn = document.createElement("button");
    nextBtn.className = "presenter-btn-next";
    nextBtn.textContent = "Next ▶";
    nextBtn.style.background = "#3b82f6";
    nextBtn.style.color = "#ffffff";
    nextBtn.style.border = "none";
    nextBtn.style.borderRadius = "4px";
    nextBtn.style.padding = "4px 10px";
    nextBtn.style.fontSize = "12px";
    nextBtn.style.cursor = "pointer";
    nextBtn.onclick = () => this.nextSlide();
    bar.appendChild(nextBtn);

    return bar;
  }

  private updateToolButtons(): void {
    const btns = this.rootEl.querySelectorAll(".presenter-tool-btn");
    btns.forEach((b) => {
      const el = b as HTMLElement;
      const isSelected = el.classList.contains(`presenter-tool-${this.activeTool}`);
      el.style.background = isSelected ? "#38bdf8" : "#27272a";
      el.style.color = isSelected ? "#000000" : "#ffffff";
    });
  }

  private attachCanvasInteraction(wrap: HTMLElement): void {
    wrap.addEventListener("pointerdown", (e: PointerEvent) => {
      if (this.activeTool === "pen") {
        this.isDrawing = true;
        this.currentStroke = [];
        const pt = this.getNormalizedCoord(e);
        this.currentStroke.push(pt);
      }
    });

    wrap.addEventListener("pointermove", (e: PointerEvent) => {
      const pt = this.getNormalizedCoord(e);

      if (this.activeTool === "laser") {
        this.broadcastSync({
          type: "LASER_MOVE",
          laser: { active: true, x: pt.x, y: pt.y },
        });
      } else if (this.activeTool === "pen" && this.isDrawing) {
        this.currentStroke.push(pt);
        this.drawLiveStroke(this.currentCtx, this.currentStroke, this.penColor, this.penWidth);
      }
    });

    const stopDrawing = () => {
      if (this.isDrawing && this.currentStroke.length > 1) {
        const stroke: InkStroke = {
          points: [...this.currentStroke],
          color: this.penColor,
          width: this.penWidth,
        };
        this.inkStrokes.push(stroke);
        this.broadcastSync({
          type: "INK_STROKE",
          stroke,
        });
        if (this.audienceInkCtx) {
          this.drawLiveStroke(this.audienceInkCtx, stroke.points, stroke.color, stroke.width);
        }
      }
      this.isDrawing = false;
    };

    wrap.addEventListener("pointerup", stopDrawing);
    wrap.addEventListener("pointerleave", () => {
      stopDrawing();
      if (this.activeTool === "laser") {
        this.broadcastSync({
          type: "LASER_MOVE",
          laser: { active: false, x: 0, y: 0 },
        });
      }
    });
  }

  private getNormalizedCoord(e: PointerEvent): { x: number; y: number } {
    const rect = this.currentCanvas.getBoundingClientRect();
    const x = (e.clientX - rect.left) / rect.width;
    const y = (e.clientY - rect.top) / rect.height;
    return {
      x: Math.max(0, Math.min(1, x)) * this.state.slideWidth,
      y: Math.max(0, Math.min(1, y)) * this.state.slideHeight,
    };
  }

  private drawLiveStroke(
    ctx: CanvasRenderingContext2D,
    pts: Array<{ x: number; y: number }>,
    color: string,
    width: number
  ): void {
    if (pts.length < 2) return;
    ctx.save();
    ctx.strokeStyle = color;
    ctx.lineWidth = width;
    ctx.lineCap = "round";
    ctx.lineJoin = "round";
    ctx.beginPath();
    ctx.moveTo(pts[0].x, pts[0].y);
    for (let i = 1; i < pts.length; i++) {
      ctx.lineTo(pts[i].x, pts[i].y);
    }
    ctx.stroke();
    ctx.restore();
  }

  private initAudienceOverlay(): void {
    if (!this.audienceEl) return;
    this.audienceEl.style.position = "relative";

    // Audience Laser Dot
    this.audienceLaserDot = document.createElement("div");
    this.audienceLaserDot.className = "audience-laser-dot";
    this.audienceLaserDot.style.position = "absolute";
    this.audienceLaserDot.style.width = "12px";
    this.audienceLaserDot.style.height = "12px";
    this.audienceLaserDot.style.borderRadius = "50%";
    this.audienceLaserDot.style.backgroundColor = "#ef4444";
    this.audienceLaserDot.style.boxShadow = "0 0 10px 4px #ef4444";
    this.audienceLaserDot.style.pointerEvents = "none";
    this.audienceLaserDot.style.display = "none";
    this.audienceLaserDot.style.zIndex = "1000";
    this.audienceEl.appendChild(this.audienceLaserDot);

    // Audience Ink Canvas
    this.audienceInkCanvas = document.createElement("canvas");
    this.audienceInkCanvas.className = "audience-ink-canvas";
    this.audienceInkCanvas.style.position = "absolute";
    this.audienceInkCanvas.style.inset = "0";
    this.audienceInkCanvas.style.pointerEvents = "none";
    this.audienceInkCanvas.style.zIndex = "999";
    this.audienceInkCtx = this.audienceInkCanvas.getContext("2d")!;
    this.audienceEl.appendChild(this.audienceInkCanvas);
  }

  private attachKeyboardListener(): void {
    window.addEventListener("keydown", (e: KeyboardEvent) => {
      if (!this.isOpenState) return;
      const key = e.key.toUpperCase();
      switch (key) {
        case "B":
          this.toggleBlackScreen();
          break;
        case "W":
          this.toggleWhiteScreen();
          break;
        case "P":
          this.setTool("pen");
          break;
        case "L":
          this.setTool("laser");
          break;
        case "E":
          this.clearInk();
          break;
        case "ESCAPE":
          this.close();
          break;
        case "ARROWLEFT":
        case "PAGEUP":
          this.prevSlide();
          break;
        case "ARROWRIGHT":
        case " ":
        case "PAGEDOWN":
          this.nextSlide();
          break;
      }
    });
  }

  private broadcastSync(msg: PresenterSyncPayload): void {
    try {
      if (this.syncChannel) {
        this.syncChannel.postMessage(msg);
      }
    } catch {}

    // Direct local update on audience elements if wired
    this.handleSyncMessage(msg);
  }

  private handleSyncMessage(msg: PresenterSyncPayload): void {
    if (!msg) return;
    if (msg.type === "LASER_MOVE" && this.audienceLaserDot) {
      if (msg.laser?.active) {
        this.audienceLaserDot.style.display = "block";
        const w = this.audienceEl?.clientWidth || this.state.slideWidth;
        const h = this.audienceEl?.clientHeight || this.state.slideHeight;
        this.audienceLaserDot.style.left = `${(msg.laser.x / this.state.slideWidth) * w}px`;
        this.audienceLaserDot.style.top = `${(msg.laser.y / this.state.slideHeight) * h}px`;
      } else {
        this.audienceLaserDot.style.display = "none";
      }
    } else if (msg.type === "INK_STROKE" && msg.stroke && this.audienceInkCtx) {
      this.drawLiveStroke(
        this.audienceInkCtx,
        msg.stroke.points,
        msg.stroke.color,
        msg.stroke.width
      );
    } else if (msg.type === "CLEAR_INK" && this.audienceInkCtx && this.audienceInkCanvas) {
      this.audienceInkCtx.clearRect(0, 0, this.audienceInkCanvas.width, this.audienceInkCanvas.height);
    } else if (msg.type === "SCREEN_MODE" && this.audienceEl && msg.screen) {
      if (msg.screen.black) {
        this.audienceEl.style.backgroundColor = "#000000";
      } else if (msg.screen.white) {
        this.audienceEl.style.backgroundColor = "#ffffff";
      } else {
        this.audienceEl.style.backgroundColor = "";
      }
    }
  }

  public render(): void {
    const curIdx = this.state.currentSlideIndex;
    const curSlide = this.state.slides[curIdx];
    const nextSlide = this.state.slides[curIdx + 1];

    const w = this.state.slideWidth;
    const h = this.state.slideHeight;

    // 1. Render Current Slide
    this.currentCanvas.width = w;
    this.currentCanvas.height = h;
    this.currentCanvas.style.width = "100%";
    this.currentCanvas.style.height = "100%";
    this.currentCanvas.style.objectFit = "contain";
    if (curSlide) {
      this.drawSlide(this.currentCtx, curSlide);
    }

    // Render ink strokes onto current preview
    for (const stroke of this.inkStrokes) {
      this.drawLiveStroke(this.currentCtx, stroke.points, stroke.color, stroke.width);
    }

    // Blank screen overlays if active
    if (this.isBlackScreen) {
      this.currentCtx.fillStyle = "#000000";
      this.currentCtx.fillRect(0, 0, w, h);
    } else if (this.isWhiteScreen) {
      this.currentCtx.fillStyle = "#ffffff";
      this.currentCtx.fillRect(0, 0, w, h);
    }

    // 2. Render Next Slide
    this.nextCanvas.width = w;
    this.nextCanvas.height = h;
    this.nextCanvas.style.width = "100%";
    this.nextCanvas.style.height = "100%";
    this.nextCanvas.style.objectFit = "contain";
    if (nextSlide) {
      this.drawSlide(this.nextCtx, nextSlide);
    } else {
      this.nextCtx.fillStyle = "#18181b";
      this.nextCtx.fillRect(0, 0, w, h);
      this.nextCtx.fillStyle = "#94a3b8";
      this.nextCtx.font = "24px Segoe UI, sans-serif";
      this.nextCtx.textAlign = "center";
      this.nextCtx.fillText("End of Presentation", w / 2, h / 2);
    }

    // 3. Render Speaker Notes
    if (curSlide?.notes) {
      this.notesContentEl.innerHTML = curSlide.notes.replace(/\n/g, "<br/>");
    } else {
      this.notesContentEl.innerHTML = `<span style="color:#52525b;font-style:italic;">No notes entered for this slide.</span>`;
    }

    // 4. Render Filmstrip
    this.renderFilmstrip();
  }

  private drawSlide(ctx: CanvasRenderingContext2D, slide: SlideData): void {
    ctx.fillStyle = slide.background || "#ffffff";
    ctx.fillRect(0, 0, this.state.slideWidth, this.state.slideHeight);

    if (!slide.shapes) return;
    for (const sh of slide.shapes) {
      ctx.save();
      ctx.fillStyle = sh.fill || "#3b82f6";
      ctx.strokeStyle = sh.stroke || "#1e293b";
      ctx.lineWidth = sh.strokeWidth || 1;

      if (sh.type === "ellipse") {
        ctx.beginPath();
        ctx.ellipse(
          sh.x + sh.w / 2,
          sh.y + sh.h / 2,
          sh.w / 2,
          sh.h / 2,
          0,
          0,
          Math.PI * 2
        );
        ctx.fill();
        ctx.stroke();
      } else {
        ctx.fillRect(sh.x, sh.y, sh.w, sh.h);
        ctx.strokeRect(sh.x, sh.y, sh.w, sh.h);
      }

      if (sh.text) {
        ctx.fillStyle = sh.textColor || "#000000";
        ctx.font = `${sh.fontSize || 18}px sans-serif`;
        ctx.fillText(sh.text, sh.x + 10, sh.y + 24);
      }
      ctx.restore();
    }
  }

  private renderFilmstrip(): void {
    const strip = this.rootEl.querySelector(".presenter-filmstrip");
    if (!strip) return;
    strip.innerHTML = "";

    this.state.slides.forEach((s, idx) => {
      const card = document.createElement("div");
      card.className = "presenter-filmstrip-card";
      const isSel = idx === this.state.currentSlideIndex;
      card.style.display = "flex";
      card.style.flexDirection = "column";
      card.style.alignItems = "center";
      card.style.padding = "4px";
      card.style.borderRadius = "4px";
      card.style.cursor = "pointer";
      card.style.border = isSel ? "2px solid #38bdf8" : "1px solid #3f3f46";
      card.style.backgroundColor = isSel ? "#27272a" : "#09090b";
      card.style.minWidth = "80px";

      const thumb = document.createElement("div");
      thumb.style.width = "72px";
      thumb.style.height = "40px";
      thumb.style.backgroundColor = s.background || "#ffffff";
      thumb.style.borderRadius = "2px";
      card.appendChild(thumb);

      const label = document.createElement("span");
      label.style.fontSize = "11px";
      label.style.color = isSel ? "#38bdf8" : "#94a3b8";
      label.style.marginTop = "2px";
      label.textContent = `Slide ${idx + 1}`;
      card.appendChild(label);

      card.onclick = () => this.jumpToSlide(idx);
      strip.appendChild(card);
    });
  }

  private startTimer(): void {
    if (this.timerInterval) clearInterval(this.timerInterval);
    this.timerInterval = setInterval(() => {
      if (!this.timerPaused) {
        this.timerSeconds++;
        this.updateTimerDisplay();
      }
      if (this.clockEl) {
        this.clockEl.textContent = new Date().toLocaleTimeString();
      }
    }, 1000);
  }

  private stopTimer(): void {
    if (this.timerInterval) {
      clearInterval(this.timerInterval);
      this.timerInterval = undefined;
    }
  }

  private updateTimerDisplay(): void {
    const hrs = Math.floor(this.timerSeconds / 3600);
    const mins = Math.floor((this.timerSeconds % 3600) / 60);
    const secs = this.timerSeconds % 60;
    const pad = (n: number) => n.toString().padStart(2, "0");
    if (this.timerEl) {
      this.timerEl.textContent = `⏱️ ${pad(hrs)}:${pad(mins)}:${pad(secs)}`;
    }
  }
}
