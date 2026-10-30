import {
  SlideData,
  SlidesState,
  slidesState,
} from "../state/slidesState";
import { SlideTransitionEngine } from "./SlideTransitionEngine";

export type PresentationMode = "slide" | "presenter";

export class PresentationPlayer {
  private rootEl: HTMLElement;
  private mode: PresentationMode = "slide";
  private state: SlidesState;
  private slideCanvas: HTMLCanvasElement;
  private slideCtx: CanvasRenderingContext2D;
  private controlsEl: HTMLElement;
  private laserDotEl: HTMLElement;
  private isLaserActive = false;
  private isBlackScreen = false;
  private isWhiteScreen = false;
  private timerSeconds = 0;
  private timerInterval?: any;
  private timerPaused = false;
  private notesFontSize = 14;
  private onExitCallback?: () => void;
  private onSlideChangeCallback?: (slideIdx: number) => void;

  constructor(state: SlidesState = slidesState) {
    this.state = state;

    this.rootEl = document.createElement("div");
    this.rootEl.id = "presentation-player";
    this.rootEl.className = "presentation-player presentation-slide-mode";
    this.rootEl.style.position = "fixed";
    this.rootEl.style.top = "0";
    this.rootEl.style.left = "0";
    this.rootEl.style.width = "100vw";
    this.rootEl.style.height = "100vh";
    this.rootEl.style.backgroundColor = "#000000";
    this.rootEl.style.zIndex = "99999";
    this.rootEl.style.display = "none";
    this.rootEl.style.flexDirection = "column";
    this.rootEl.style.alignItems = "center";
    this.rootEl.style.justifyContent = "center";
    this.rootEl.style.fontFamily = "Segoe UI, sans-serif";
    this.rootEl.style.userSelect = "none";
    this.rootEl.tabIndex = 0;

    // Slide Canvas
    this.slideCanvas = document.createElement("canvas");
    this.slideCanvas.className = "presentation-slide-canvas";
    this.slideCanvas.style.display = "block";
    this.slideCanvas.style.boxShadow = "0 8px 32px rgba(0, 0, 0, 0.6)";
    this.slideCanvas.style.cursor = "pointer";
    const ctx = this.slideCanvas.getContext("2d");
    this.slideCtx = (ctx || {
      fillRect: () => {},
      strokeRect: () => {},
      fillText: () => {},
      clearRect: () => {},
      beginPath: () => {},
      closePath: () => {},
      moveTo: () => {},
      lineTo: () => {},
      stroke: () => {},
      fill: () => {},
      save: () => {},
      restore: () => {},
      clip: () => {},
      scale: () => {},
      setTransform: () => {},
      translate: () => {},
      rotate: () => {},
      arc: () => {},
      roundRect: () => {},
      ellipse: () => {},
      measureText: (txt: string) => ({ width: txt.length * 8 }),
      setLineDash: () => {},
      createLinearGradient: () => ({ addColorStop: () => {} }),
      font: "24px Aptos, sans-serif",
      fillStyle: "#000",
      strokeStyle: "#000",
      lineWidth: 1,
      textAlign: "left",
      textBaseline: "top",
    }) as unknown as CanvasRenderingContext2D;

    // Laser pointer dot
    this.laserDotEl = document.createElement("div");
    this.laserDotEl.className = "pres-laser-dot";
    this.laserDotEl.style.position = "fixed";
    this.laserDotEl.style.width = "12px";
    this.laserDotEl.style.height = "12px";
    this.laserDotEl.style.backgroundColor = "#ff2222";
    this.laserDotEl.style.borderRadius = "50%";
    this.laserDotEl.style.boxShadow = "0 0 12px 3px rgba(255, 34, 34, 0.9)";
    this.laserDotEl.style.pointerEvents = "none";
    this.laserDotEl.style.display = "none";
    this.laserDotEl.style.zIndex = "100000";
    this.rootEl.appendChild(this.laserDotEl);

    // Controls Toolbar (Standard mode)
    this.controlsEl = document.createElement("div");
    this.controlsEl.className = "presentation-controls";
    this.controlsEl.style.position = "fixed";
    this.controlsEl.style.bottom = "16px";
    this.controlsEl.style.left = "16px";
    this.controlsEl.style.display = "flex";
    this.controlsEl.style.alignItems = "center";
    this.controlsEl.style.gap = "8px";
    this.controlsEl.style.backgroundColor = "rgba(15, 23, 42, 0.75)";
    this.controlsEl.style.backdropFilter = "blur(8px)";
    this.controlsEl.style.padding = "6px 12px";
    this.controlsEl.style.borderRadius = "20px";
    this.controlsEl.style.boxShadow = "0 4px 16px rgba(0, 0, 0, 0.4)";
    this.controlsEl.style.opacity = "0.7";
    this.controlsEl.style.transition = "opacity 0.2s ease";
    this.controlsEl.onmouseenter = () => (this.controlsEl.style.opacity = "1");
    this.controlsEl.onmouseleave = () => (this.controlsEl.style.opacity = "0.7");

    this.attachEvents();
    document.body.appendChild(this.rootEl);
  }

  public setCallbacks(callbacks: {
    onExit?: () => void;
    onSlideChange?: (slideIdx: number) => void;
  }) {
    if (callbacks.onExit) this.onExitCallback = callbacks.onExit;
    if (callbacks.onSlideChange) this.onSlideChangeCallback = callbacks.onSlideChange;
  }

  public isOpen(): boolean {
    return this.rootEl.style.display !== "none";
  }

  public isPresenter(): boolean {
    return this.mode === "presenter";
  }

  public getRoot(): HTMLElement {
    return this.rootEl;
  }

  public startShow(fromBeginning = true, presenterMode = false) {
    this.state.startShow(fromBeginning, presenterMode);
    this.mode = presenterMode ? "presenter" : "slide";
    this.isBlackScreen = false;
    this.isWhiteScreen = false;
    this.isLaserActive = false;
    this.laserDotEl.style.display = "none";
    this.timerSeconds = 0;
    this.timerPaused = false;
    this.startTimer();

    this.rootEl.style.display = "flex";
    this.render();
    this.rootEl.focus();

    if (this.onSlideChangeCallback) {
      this.onSlideChangeCallback(this.state.currentSlideIndex);
    }
  }

  public exitShow() {
    this.stopTimer();
    this.state.exitShow();
    this.rootEl.style.display = "none";
    this.isLaserActive = false;
    this.laserDotEl.style.display = "none";
    if (document.fullscreenElement) {
      document.exitFullscreen().catch(() => {});
    }
    if (this.onExitCallback) {
      this.onExitCallback();
    }
  }

  public togglePresenterView() {
    this.mode = this.mode === "slide" ? "presenter" : "slide";
    this.state.isPresenterMode = this.mode === "presenter";
    this.render();
  }

  public toggleLaser() {
    this.isLaserActive = !this.isLaserActive;
    this.laserDotEl.style.display = this.isLaserActive ? "block" : "none";
    this.slideCanvas.style.cursor = this.isLaserActive ? "none" : "pointer";
  }

  public toggleBlackScreen() {
    this.isBlackScreen = !this.isBlackScreen;
    this.isWhiteScreen = false;
    this.render();
  }

  public toggleWhiteScreen() {
    this.isWhiteScreen = !this.isWhiteScreen;
    this.isBlackScreen = false;
    this.render();
  }

  public stepForward() {
    if (this.isBlackScreen || this.isWhiteScreen) {
      this.isBlackScreen = false;
      this.isWhiteScreen = false;
      this.render();
      return;
    }

    const res = this.state.stepForward();

    if (res.finished) {
      this.exitShow();
      return;
    }

    if (res.slideChanged) {
      this.playSlideTransition();
      if (this.onSlideChangeCallback) {
        this.onSlideChangeCallback(this.state.currentSlideIndex);
      }
    } else {
      this.render();
    }
  }

  public stepBackward() {
    if (this.isBlackScreen || this.isWhiteScreen) {
      this.isBlackScreen = false;
      this.isWhiteScreen = false;
      this.render();
      return;
    }

    const res = this.state.stepBackward();
    if (res.slideChanged && this.onSlideChangeCallback) {
      this.onSlideChangeCallback(this.state.currentSlideIndex);
    }
    this.render();
  }

  public goToSlide(idx: number) {
    if (idx >= 0 && idx < this.state.slides.length) {
      this.state.currentSlideIndex = idx;
      this.state.currentShowStep = 0;
      this.render();
      if (this.onSlideChangeCallback) {
        this.onSlideChangeCallback(idx);
      }
    }
  }

  private startTimer() {
    this.stopTimer();
    this.timerInterval = setInterval(() => {
      if (!this.timerPaused) {
        this.timerSeconds++;
        const timerLabel = this.rootEl.querySelector(".pres-timer");
        if (timerLabel) {
          timerLabel.textContent = `⏱️ ${this.formatTime(this.timerSeconds)}`;
        }
      }
    }, 1000);
  }

  private stopTimer() {
    if (this.timerInterval) {
      clearInterval(this.timerInterval);
      this.timerInterval = undefined;
    }
  }

  private formatTime(secs: number): string {
    const h = Math.floor(secs / 3600);
    const m = Math.floor((secs % 3600) / 60);
    const s = secs % 60;
    const pad = (n: number) => (n < 10 ? `0${n}` : `${n}`);
    return h > 0 ? `${pad(h)}:${pad(m)}:${pad(s)}` : `${pad(m)}:${pad(s)}`;
  }

  private attachEvents() {
    // Window Resize -> re-render
    window.addEventListener("resize", () => {
      if (this.isOpen()) {
        this.render();
      }
    });

    // Keyboard Shortcuts inside Presentation
    window.addEventListener("keydown", (e: KeyboardEvent) => {
      if (!this.isOpen()) return;

      switch (e.key) {
        case "ArrowRight":
        case "ArrowDown":
        case "PageDown":
        case " ":
        case "Enter":
          e.preventDefault();
          this.stepForward();
          break;
        case "ArrowLeft":
        case "ArrowUp":
        case "PageUp":
        case "Backspace":
          e.preventDefault();
          this.stepBackward();
          break;
        case "Home":
          e.preventDefault();
          this.goToSlide(0);
          break;
        case "End":
          e.preventDefault();
          this.goToSlide(this.state.slides.length - 1);
          break;
        case "Escape":
          e.preventDefault();
          this.exitShow();
          break;
        case "b":
        case "B":
        case ".":
          e.preventDefault();
          this.toggleBlackScreen();
          break;
        case "w":
        case "W":
        case ",":
          e.preventDefault();
          this.toggleWhiteScreen();
          break;
        case "l":
        case "L":
          e.preventDefault();
          this.toggleLaser();
          break;
      }
    });

    // Mouse Pointer for Laser
    this.rootEl.addEventListener("mousemove", (e: MouseEvent) => {
      if (!this.isLaserActive) return;
      this.laserDotEl.style.left = `${e.clientX - 6}px`;
      this.laserDotEl.style.top = `${e.clientY - 6}px`;
    });

    // Canvas click advances
    this.slideCanvas.addEventListener("click", () => {
      this.stepForward();
    });
  }

  public render() {
    this.rootEl.innerHTML = "";
    this.rootEl.className = `presentation-player presentation-${this.mode}-mode`;
    this.rootEl.appendChild(this.laserDotEl);

    if (this.mode === "presenter") {
      this.renderPresenterView();
    } else {
      this.renderSlideShowView();
    }
  }

  private renderSlideShowView() {
    this.rootEl.style.backgroundColor = this.isWhiteScreen
      ? "#ffffff"
      : "#000000";

    if (this.isBlackScreen) {
      // Screen is blacked out
      return;
    }
    if (this.isWhiteScreen) {
      // Screen is whited out
      return;
    }

    // Size canvas to fit screen preserving aspect ratio
    const vw = window.innerWidth || 1280;
    const vh = window.innerHeight || 720;
    const aspect = this.state.slideWidth / this.state.slideHeight;

    let targetW = vw;
    let targetH = vw / aspect;
    if (targetH > vh) {
      targetH = vh;
      targetW = vh * aspect;
    }

    const dpr = window.devicePixelRatio || 1;
    this.slideCanvas.width = this.state.slideWidth * dpr;
    this.slideCanvas.height = this.state.slideHeight * dpr;
    this.slideCanvas.style.width = `${Math.round(targetW)}px`;
    this.slideCanvas.style.height = `${Math.round(targetH)}px`;

    this.slideCtx.setTransform(dpr, 0, 0, dpr, 0, 0);

    const slide = this.state.slides[this.state.currentSlideIndex];
    if (slide) {
      this.drawSlide(this.slideCtx, slide, this.state.currentShowStep);
    }

    this.rootEl.appendChild(this.slideCanvas);

    // Standard Controls Toolbar
    this.renderStandardToolbar();
  }

  private renderStandardToolbar() {
    this.controlsEl.innerHTML = "";

    const btnPrev = this.createControlBtn("◀", "Previous (Left Arrow)", "pres-btn-prev", () => this.stepBackward());
    const btnNext = this.createControlBtn("▶", "Next (Right Arrow)", "pres-btn-next", () => this.stepForward());
    const btnLaser = this.createControlBtn("🔴", "Laser Pointer (L)", "pres-btn-laser", () => this.toggleLaser());
    const btnBlack = this.createControlBtn("⬛", "Black Screen (B)", "pres-btn-black", () => this.toggleBlackScreen());
    const btnWhite = this.createControlBtn("⬜", "White Screen (W)", "pres-btn-white", () => this.toggleWhiteScreen());
    const btnPresenter = this.createControlBtn("🖥️", "Presenter View", "pres-btn-presenter", () => this.togglePresenterView());

    const counter = document.createElement("span");
    counter.className = "pres-slide-counter";
    counter.style.color = "#ffffff";
    counter.style.fontSize = "12px";
    counter.style.padding = "0 6px";
    counter.textContent = `Slide ${this.state.currentSlideIndex + 1} of ${this.state.slides.length}`;

    const btnClose = this.createControlBtn("✕", "End Show (Esc)", "pres-btn-close", () => this.exitShow());

    this.controlsEl.appendChild(btnPrev);
    this.controlsEl.appendChild(btnNext);
    this.controlsEl.appendChild(btnLaser);
    this.controlsEl.appendChild(btnBlack);
    this.controlsEl.appendChild(btnWhite);
    this.controlsEl.appendChild(btnPresenter);
    this.controlsEl.appendChild(counter);
    this.controlsEl.appendChild(btnClose);

    this.rootEl.appendChild(this.controlsEl);
  }

  private renderPresenterView() {
    this.rootEl.style.backgroundColor = "#18181b";

    // 1. Header Bar
    const header = document.createElement("div");
    header.className = "pres-presenter-header";
    header.style.width = "100%";
    header.style.display = "flex";
    header.style.alignItems = "center";
    header.style.justifyContent = "space-between";
    header.style.padding = "10px 24px";
    header.style.backgroundColor = "#09090b";
    header.style.borderBottom = "1px solid #27272a";
    header.style.color = "#ffffff";

    const titleWrap = document.createElement("div");
    titleWrap.style.display = "flex";
    titleWrap.style.alignItems = "center";
    titleWrap.style.gap = "8px";
    titleWrap.innerHTML = `<span style="font-size:16px;">🖥️</span><span class="pres-view-title" style="font-weight:600;font-size:14px;">OpenOfficeRS Presenter View</span>`;

    const timerWrap = document.createElement("div");
    timerWrap.style.display = "flex";
    timerWrap.style.alignItems = "center";
    timerWrap.style.gap = "8px";

    const timerSpan = document.createElement("span");
    timerSpan.className = "pres-timer";
    timerSpan.style.fontFamily = "monospace";
    timerSpan.style.fontSize = "16px";
    timerSpan.style.color = "#38bdf8";
    timerSpan.textContent = `⏱️ ${this.formatTime(this.timerSeconds)}`;

    const pauseBtn = document.createElement("button");
    pauseBtn.className = "pres-btn-timer-pause";
    pauseBtn.textContent = this.timerPaused ? "▶" : "⏸";
    pauseBtn.title = "Pause/Resume Timer";
    pauseBtn.style.padding = "2px 6px";
    pauseBtn.style.cursor = "pointer";
    pauseBtn.onclick = () => {
      this.timerPaused = !this.timerPaused;
      pauseBtn.textContent = this.timerPaused ? "▶" : "⏸";
    };

    const resetBtn = document.createElement("button");
    resetBtn.className = "pres-btn-timer-reset";
    resetBtn.textContent = "🔄";
    resetBtn.title = "Reset Timer";
    resetBtn.style.padding = "2px 6px";
    resetBtn.style.cursor = "pointer";
    resetBtn.onclick = () => {
      this.timerSeconds = 0;
      timerSpan.textContent = `⏱️ 00:00`;
    };

    const clockSpan = document.createElement("span");
    clockSpan.className = "pres-clock";
    clockSpan.style.color = "#94a3b8";
    clockSpan.style.fontSize = "13px";
    clockSpan.textContent = new Date().toLocaleTimeString();

    timerWrap.appendChild(timerSpan);
    timerWrap.appendChild(pauseBtn);
    timerWrap.appendChild(resetBtn);
    timerWrap.appendChild(clockSpan);

    const endBtn = document.createElement("button");
    endBtn.className = "pres-btn-end-show";
    endBtn.textContent = "✕ End Slide Show";
    endBtn.style.backgroundColor = "#ef4444";
    endBtn.style.color = "#ffffff";
    endBtn.style.border = "none";
    endBtn.style.padding = "6px 14px";
    endBtn.style.borderRadius = "4px";
    endBtn.style.cursor = "pointer";
    endBtn.style.fontWeight = "600";
    endBtn.onclick = () => this.exitShow();

    header.appendChild(titleWrap);
    header.appendChild(timerWrap);
    header.appendChild(endBtn);
    this.rootEl.appendChild(header);

    // 2. Main Two-Column Stage Area
    const mainArea = document.createElement("div");
    mainArea.className = "pres-presenter-main";
    mainArea.style.display = "flex";
    mainArea.style.flex = "1";
    mainArea.style.width = "100%";
    mainArea.style.padding = "16px";
    mainArea.style.gap = "16px";
    mainArea.style.overflow = "hidden";

    // Left Column: Active Slide
    const leftCol = document.createElement("div");
    leftCol.className = "pres-presenter-left";
    leftCol.style.flex = "3";
    leftCol.style.display = "flex";
    leftCol.style.flexDirection = "column";
    leftCol.style.alignItems = "center";
    leftCol.style.justifyContent = "center";
    leftCol.style.backgroundColor = "#000000";
    leftCol.style.borderRadius = "8px";
    leftCol.style.padding = "12px";
    leftCol.style.position = "relative";

    const dpr = window.devicePixelRatio || 1;
    const aspect = this.state.slideWidth / this.state.slideHeight;
    const targetW = 680;
    const targetH = targetW / aspect;

    this.slideCanvas.width = this.state.slideWidth * dpr;
    this.slideCanvas.height = this.state.slideHeight * dpr;
    this.slideCanvas.style.width = `${targetW}px`;
    this.slideCanvas.style.height = `${targetH}px`;
    this.slideCtx.setTransform(dpr, 0, 0, dpr, 0, 0);

    const slide = this.state.slides[this.state.currentSlideIndex];
    if (slide) {
      this.drawSlide(this.slideCtx, slide, this.state.currentShowStep);
    }
    leftCol.appendChild(this.slideCanvas);

    // Left Column toolbar
    const leftToolbar = document.createElement("div");
    leftToolbar.style.display = "flex";
    leftToolbar.style.alignItems = "center";
    leftToolbar.style.gap = "8px";
    leftToolbar.style.marginTop = "10px";

    leftToolbar.appendChild(this.createControlBtn("◀ Previous", "Previous Slide", "pres-btn-prev", () => this.stepBackward()));
    leftToolbar.appendChild(this.createControlBtn("Next ▶", "Next Step / Slide", "pres-btn-next", () => this.stepForward()));
    leftToolbar.appendChild(this.createControlBtn("🔴 Laser", "Laser Pointer", "pres-btn-laser", () => this.toggleLaser()));
    leftToolbar.appendChild(this.createControlBtn("⬛ Black", "Black Screen", "pres-btn-black", () => this.toggleBlackScreen()));
    leftToolbar.appendChild(this.createControlBtn("🖥️ Show View", "Switch to Fullscreen Show", "pres-btn-presenter", () => this.togglePresenterView()));

    const leftSlideInfo = document.createElement("span");
    leftSlideInfo.style.color = "#94a3b8";
    leftSlideInfo.style.fontSize = "12px";
    leftSlideInfo.textContent = `Slide ${this.state.currentSlideIndex + 1} of ${this.state.slides.length}`;
    leftToolbar.appendChild(leftSlideInfo);

    leftCol.appendChild(leftToolbar);
    mainArea.appendChild(leftCol);

    // Right Column: Next Slide Preview & Speaker Notes
    const rightCol = document.createElement("div");
    rightCol.className = "pres-presenter-right";
    rightCol.style.flex = "2";
    rightCol.style.display = "flex";
    rightCol.style.flexDirection = "column";
    rightCol.style.gap = "12px";

    // Next Slide Card
    const nextCard = document.createElement("div");
    nextCard.className = "pres-next-card";
    nextCard.style.backgroundColor = "#27272a";
    nextCard.style.borderRadius = "8px";
    nextCard.style.padding = "10px";
    nextCard.style.display = "flex";
    nextCard.style.flexDirection = "column";
    nextCard.style.alignItems = "center";

    const nextTitle = document.createElement("div");
    nextTitle.style.width = "100%";
    nextTitle.style.fontSize = "12px";
    nextTitle.style.fontWeight = "600";
    nextTitle.style.color = "#38bdf8";
    nextTitle.style.marginBottom = "6px";
    nextTitle.textContent = "Next Slide:";
    nextCard.appendChild(nextTitle);

    const nextCanvas = document.createElement("canvas");
    nextCanvas.className = "pres-next-slide-canvas";
    const nextW = 280;
    const nextH = nextW / aspect;
    nextCanvas.width = this.state.slideWidth * dpr;
    nextCanvas.height = this.state.slideHeight * dpr;
    nextCanvas.style.width = `${nextW}px`;
    nextCanvas.style.height = `${nextH}px`;
    nextCanvas.style.borderRadius = "4px";
    const nextCtx = nextCanvas.getContext("2d");
    if (nextCtx) {
      nextCtx.setTransform(dpr, 0, 0, dpr, 0, 0);
      const nextSlide = this.state.slides[this.state.currentSlideIndex + 1];
      if (nextSlide) {
        this.drawSlide(nextCtx, nextSlide, 999);
      } else {
        nextCtx.fillStyle = "#18181b";
        nextCtx.fillRect(0, 0, this.state.slideWidth, this.state.slideHeight);
        nextCtx.fillStyle = "#64748b";
        nextCtx.font = "24px Segoe UI";
        nextCtx.textAlign = "center";
        nextCtx.fillText("End of Presentation", this.state.slideWidth / 2, this.state.slideHeight / 2);
      }
    }
    nextCard.appendChild(nextCanvas);
    rightCol.appendChild(nextCard);

    // Speaker Notes Card
    const notesCard = document.createElement("div");
    notesCard.className = "pres-notes-card";
    notesCard.style.flex = "1";
    notesCard.style.backgroundColor = "#27272a";
    notesCard.style.borderRadius = "8px";
    notesCard.style.padding = "12px";
    notesCard.style.display = "flex";
    notesCard.style.flexDirection = "column";

    const notesHeader = document.createElement("div");
    notesHeader.style.display = "flex";
    notesHeader.style.justifyContent = "space-between";
    notesHeader.style.alignItems = "center";
    notesHeader.style.marginBottom = "8px";

    const notesTitle = document.createElement("span");
    notesTitle.style.fontSize = "13px";
    notesTitle.style.fontWeight = "600";
    notesTitle.style.color = "#ffffff";
    notesTitle.textContent = "📝 Speaker Notes";

    const zoomWrap = document.createElement("div");
    zoomWrap.style.display = "flex";
    zoomWrap.style.gap = "4px";

    const zoomInBtn = document.createElement("button");
    zoomInBtn.className = "pres-zoom-in";
    zoomInBtn.textContent = "A+";
    zoomInBtn.style.padding = "2px 6px";
    zoomInBtn.style.cursor = "pointer";
    zoomInBtn.onclick = () => {
      this.notesFontSize = Math.min(28, this.notesFontSize + 2);
      notesContent.style.fontSize = `${this.notesFontSize}px`;
    };

    const zoomOutBtn = document.createElement("button");
    zoomOutBtn.className = "pres-zoom-out";
    zoomOutBtn.textContent = "A-";
    zoomOutBtn.style.padding = "2px 6px";
    zoomOutBtn.style.cursor = "pointer";
    zoomOutBtn.onclick = () => {
      this.notesFontSize = Math.max(10, this.notesFontSize - 2);
      notesContent.style.fontSize = `${this.notesFontSize}px`;
    };

    zoomWrap.appendChild(zoomInBtn);
    zoomWrap.appendChild(zoomOutBtn);
    notesHeader.appendChild(notesTitle);
    notesHeader.appendChild(zoomWrap);
    notesCard.appendChild(notesHeader);

    const notesContent = document.createElement("div");
    notesContent.className = "pres-notes-content";
    notesContent.style.flex = "1";
    notesContent.style.overflowY = "auto";
    notesContent.style.fontSize = `${this.notesFontSize}px`;
    notesContent.style.lineHeight = "1.6";
    notesContent.style.color = "#e4e4e7";
    notesContent.style.whiteSpace = "pre-wrap";
    const currentNotes = this.state.getSlideNotes();
    notesContent.textContent = currentNotes || "(No speaker notes entered for this slide)";
    notesCard.appendChild(notesContent);

    rightCol.appendChild(notesCard);
    mainArea.appendChild(rightCol);
    this.rootEl.appendChild(mainArea);

    // 3. Bottom Filmstrip / Carousel
    const filmstrip = document.createElement("div");
    filmstrip.className = "pres-filmstrip";
    filmstrip.style.width = "100%";
    filmstrip.style.height = "90px";
    filmstrip.style.backgroundColor = "#09090b";
    filmstrip.style.borderTop = "1px solid #27272a";
    filmstrip.style.display = "flex";
    filmstrip.style.alignItems = "center";
    filmstrip.style.gap = "12px";
    filmstrip.style.padding = "0 16px";
    filmstrip.style.overflowX = "auto";

    this.state.slides.forEach((sl, idx) => {
      const thumbWrap = document.createElement("div");
      thumbWrap.className = `pres-thumb-item ${idx === this.state.currentSlideIndex ? "is-active" : ""}`;
      thumbWrap.style.display = "flex";
      thumbWrap.style.flexDirection = "column";
      thumbWrap.style.alignItems = "center";
      thumbWrap.style.cursor = "pointer";
      thumbWrap.style.padding = "2px";
      thumbWrap.style.borderRadius = "4px";
      thumbWrap.style.border = idx === this.state.currentSlideIndex ? "2px solid #38bdf8" : "2px solid transparent";

      const thumbCanvas = document.createElement("canvas");
      const thumbW = 96;
      const thumbH = thumbW / aspect;
      thumbCanvas.width = this.state.slideWidth * dpr;
      thumbCanvas.height = this.state.slideHeight * dpr;
      thumbCanvas.style.width = `${thumbW}px`;
      thumbCanvas.style.height = `${thumbH}px`;
      thumbCanvas.style.borderRadius = "2px";

      const tctx = thumbCanvas.getContext("2d");
      if (tctx) {
        tctx.setTransform(dpr, 0, 0, dpr, 0, 0);
        this.drawSlide(tctx, sl, 999);
      }
      thumbWrap.appendChild(thumbCanvas);

      const thumbLabel = document.createElement("span");
      thumbLabel.style.fontSize = "10px";
      thumbLabel.style.color = idx === this.state.currentSlideIndex ? "#38bdf8" : "#94a3b8";
      thumbLabel.textContent = `${idx + 1}`;
      thumbWrap.appendChild(thumbLabel);

      thumbWrap.onclick = () => this.goToSlide(idx);
      filmstrip.appendChild(thumbWrap);
    });

    this.rootEl.appendChild(filmstrip);
  }

  private createControlBtn(text: string, title: string, className: string, onClick: () => void): HTMLButtonElement {
    const btn = document.createElement("button");
    btn.className = `pres-btn ${className}`;
    btn.textContent = text;
    btn.title = title;
    btn.style.backgroundColor = "transparent";
    btn.style.border = "none";
    btn.style.color = "#ffffff";
    btn.style.cursor = "pointer";
    btn.style.padding = "4px 8px";
    btn.style.borderRadius = "4px";
    btn.style.fontSize = "12px";
    btn.style.transition = "background-color 0.15s ease";
    btn.onmouseenter = () => (btn.style.backgroundColor = "rgba(255, 255, 255, 0.15)");
    btn.onmouseleave = () => (btn.style.backgroundColor = "transparent");
    btn.onclick = (e) => {
      e.stopPropagation();
      onClick();
    };
    return btn;
  }

  public drawSlide(ctx: CanvasRenderingContext2D, slide: SlideData, step = 0) {
    const w = this.state.slideWidth;
    const h = this.state.slideHeight;

    ctx.clearRect(0, 0, w, h);

    // Background
    if (slide.background) {
      if (slide.background.startsWith("linear-gradient")) {
        const grad = ctx.createLinearGradient(0, 0, w, h);
        grad.addColorStop(0, "#f8fafc");
        grad.addColorStop(1, "#cbd5e1");
        ctx.fillStyle = grad;
      } else {
        ctx.fillStyle = slide.background;
      }
    } else {
      ctx.fillStyle = "#ffffff";
    }
    ctx.fillRect(0, 0, w, h);

    // Determine shape visibility & animating effects for current step
    const { visibleShapeIds, animatingShapeId, currentEffect } = this.state.getVisibleShapesForStep(
      this.state.slides.indexOf(slide),
      step
    );

    slide.shapes.forEach((sh) => {
      // If not visible in this step, skip rendering
      if (!visibleShapeIds.has(sh.id)) return;

      ctx.save();

      // Check if shape is animating in current step
      const isAnimating = sh.id === animatingShapeId;
      if (isAnimating && currentEffect) {
        ctx.shadowColor =
          currentEffect === "pulse"
            ? "#eab308"
            : currentEffect === "spin"
            ? "#3b82f6"
            : "#22c55e";
        ctx.shadowBlur = 12;
      }

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
          if (ctx.roundRect) ctx.roundRect(sh.x, sh.y, sh.w, sh.h, rad);
          else ctx.rect(sh.x, sh.y, sh.w, sh.h);
          ctx.fill();
          if (sh.stroke) ctx.stroke();
          break;
        }

        case "ellipse":
          ctx.beginPath();
          if (ctx.ellipse) {
            ctx.ellipse(sh.x + sh.w / 2, sh.y + sh.h / 2, sh.w / 2, sh.h / 2, 0, 0, Math.PI * 2);
          } else {
            ctx.arc(sh.x + sh.w / 2, sh.y + sh.h / 2, Math.min(sh.w, sh.h) / 2, 0, Math.PI * 2);
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
          const stemY = sh.y + sh.h * 0.3;
          const stemH = sh.h * 0.4;
          const headW = sh.w * 0.4;
          ctx.beginPath();
          ctx.moveTo(sh.x, stemY);
          ctx.lineTo(sh.x + sh.w - headW, stemY);
          ctx.lineTo(sh.x + sh.w - headW, sh.y);
          ctx.lineTo(sh.x + sh.w, sh.y + sh.h / 2);
          ctx.lineTo(sh.x + sh.w - headW, sh.y + sh.h);
          ctx.lineTo(sh.x + sh.w - headW, stemY + stemH);
          ctx.lineTo(sh.x, stemY + stemH);
          ctx.closePath();
          ctx.fill();
          if (sh.stroke) ctx.stroke();
          break;
        }

        case "star": {
          const cx = sh.x + sh.w / 2;
          const cy = sh.y + sh.h / 2;
          const outerR = Math.min(sh.w, sh.h) / 2;
          const innerR = outerR * 0.4;
          ctx.beginPath();
          for (let i = 0; i < 10; i++) {
            const r = i % 2 === 0 ? outerR : innerR;
            const angle = (i * Math.PI) / 5 - Math.PI / 2;
            const px = cx + Math.cos(angle) * r;
            const py = cy + Math.sin(angle) * r;
            if (i === 0) ctx.moveTo(px, py);
            else ctx.lineTo(px, py);
          }
          ctx.closePath();
          ctx.fill();
          if (sh.stroke) ctx.stroke();
          break;
        }

        case "diamond":
          ctx.beginPath();
          ctx.moveTo(sh.x + sh.w / 2, sh.y);
          ctx.lineTo(sh.x + sh.w, sh.y + sh.h / 2);
          ctx.lineTo(sh.x + sh.w / 2, sh.y + sh.h);
          ctx.lineTo(sh.x, sh.y + sh.h / 2);
          ctx.closePath();
          ctx.fill();
          if (sh.stroke) ctx.stroke();
          break;

        case "image":
          ctx.fillStyle = "#f1f5f9";
          ctx.fillRect(sh.x, sh.y, sh.w, sh.h);
          ctx.strokeStyle = "#cbd5e1";
          ctx.strokeRect(sh.x, sh.y, sh.w, sh.h);
          ctx.fillStyle = "#64748b";
          ctx.font = "bold 14px Segoe UI";
          ctx.textAlign = "center";
          ctx.fillText("🖼️ Image", sh.x + sh.w / 2, sh.y + sh.h / 2);
          break;

        case "table": {
          const rows = sh.tableData || [["A", "B", "C"], ["1", "2", "3"]];
          const rowH = sh.h / rows.length;
          const colW = sh.w / (rows[0]?.length || 1);
          rows.forEach((row, r) => {
            row.forEach((cell, c) => {
              ctx.fillStyle = r === 0 ? "#2563eb" : r % 2 === 0 ? "#f8fafc" : "#ffffff";
              ctx.fillRect(sh.x + c * colW, sh.y + r * rowH, colW, rowH);
              ctx.strokeStyle = "#cbd5e1";
              ctx.strokeRect(sh.x + c * colW, sh.y + r * rowH, colW, rowH);
              ctx.fillStyle = r === 0 ? "#ffffff" : "#1e293b";
              ctx.font = "12px Segoe UI";
              ctx.textAlign = "center";
              ctx.fillText(cell, sh.x + c * colW + colW / 2, sh.y + r * rowH + rowH / 2 + 4);
            });
          });
          break;
        }
      }

      // Draw Text inside shape
      if (sh.text) {
        ctx.fillStyle = sh.textColor || (sh.type === "text" ? "#1e293b" : "#ffffff");
        const weight = sh.bold ? "bold " : "";
        const style = sh.italic ? "italic " : "";
        const fSize = sh.fontSize || (sh.type === "text" ? 28 : 16);
        ctx.font = `${style}${weight}${fSize}px ${sh.fontFamily || "Segoe UI, sans-serif"}`;
        ctx.textAlign = sh.type === "text" ? "left" : "center";
        const tx = sh.type === "text" ? sh.x + 8 : sh.x + sh.w / 2;
        const ty = sh.type === "text" ? sh.y + fSize + 4 : sh.y + sh.h / 2 + fSize / 3;
        ctx.fillText(sh.text, tx, ty);
      }

      ctx.restore();
    });
  }

  private playSlideTransition() {
    const slide = this.state.slides[this.state.currentSlideIndex];
    if (!slide) return;

    const trans = this.state.getSlideTransition(this.state.currentSlideIndex);
    if (!trans || trans.effect === "None") {
      this.render();
      return;
    }

    SlideTransitionEngine.executeTransition(this.slideCanvas, trans, () => {
      this.render();
    });
  }
}
