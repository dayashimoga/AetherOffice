import { describe, it, expect, beforeEach } from "vitest";
import { SlidesState, slidesState } from "../state/slidesState";
import { SlideMediaOverlay } from "../components/SlideMediaOverlay";
import { UnifiedShellApp } from "../main";

describe("DrawingML Embedded Audio/Video Media Engine (Subphase 19.1.1)", () => {
  let state: SlidesState;
  let overlay: SlideMediaOverlay;
  let container: HTMLElement;

  beforeEach(() => {
    const mockCtx = {
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
      font: "24px Aptos, sans-serif",
      fillStyle: "#000",
      strokeStyle: "#000",
      lineWidth: 1,
      textAlign: "left",
      textBaseline: "top",
    };
    HTMLCanvasElement.prototype.getContext = (() => mockCtx) as any;

    // HTMLMediaElement mocks for JSDOM
    window.HTMLMediaElement.prototype.play = async () => {};
    window.HTMLMediaElement.prototype.pause = () => {};

    document.body.innerHTML = `
      <div id="app">
        <div id="titlebar"></div>
        <div id="tab-bar-container"></div>
        <div id="ribbon-container"></div>
        <div id="sub-header-container"></div>
        <div id="main-content">
          <div id="sidebar-container"></div>
          <div id="viewport-container" style="position: relative; width: 1000px; height: 600px;"></div>
        </div>
        <div id="status-bar">
          <span id="status-doc-info"></span>
          <span id="status-calc-stats"></span>
          <span id="status-word-count"></span>
          <input id="zoom-slider" type="range" value="100" />
          <span id="status-zoom-level">100%</span>
        </div>
        <div id="backstage-container"></div>
        <div id="ink-layer-container"></div>
        <div id="reflow-reader-container"></div>
        <div id="row-card-inspector-container"></div>
        <div id="laser-pointer-dot"></div>
        <div id="animation-pane-container"></div>
      </div>
    `;

    container = document.getElementById("viewport-container")!;
    state = new SlidesState();
    overlay = new SlideMediaOverlay(container, state);
  });

  it("1. inserts video shape with correct dimensions and properties", () => {
    const videoShape = state.insertMedia("video", "sample_presentation.mp4", 100, 150, 640, 360);
    expect(videoShape).toBeDefined();
    expect(videoShape.mediaType).toBe("video");
    expect(videoShape.mediaUrl).toBe("sample_presentation.mp4");
    expect(videoShape.mediaFormat).toBe("mp4");
    expect(videoShape.x).toBe(100);
    expect(videoShape.y).toBe(150);
    expect(videoShape.w).toBe(640);
    expect(videoShape.h).toBe(360);
    expect(videoShape.volume).toBe(1.0);
    expect(videoShape.isMuted).toBe(false);
    expect(videoShape.autoPlay).toBe(false);
    expect(videoShape.loopPlayback).toBe(false);
    expect(videoShape.isPlaying).toBe(false);
    expect(videoShape.posterUrl).toBe("poster_frame.png");
  });

  it("2. inserts audio shape with correct dimensions and format detection", () => {
    const audioShape = state.insertMedia("audio", "speech_intro.wav", 50, 80, 240, 48);
    expect(audioShape).toBeDefined();
    expect(audioShape.mediaType).toBe("audio");
    expect(audioShape.mediaUrl).toBe("speech_intro.wav");
    expect(audioShape.mediaFormat).toBe("wav");
    expect(audioShape.w).toBe(240);
    expect(audioShape.h).toBe(48);
    expect(audioShape.posterUrl).toBeUndefined();
  });

  it("3. handles InsertVideo and InsertAudio commands via execCommand", () => {
    const resVideo = state.execCommand("InsertVideo", "demo_clip.webm");
    expect(resVideo).toBe(true);
    const lastVideo = state.getCurrentSlide()?.shapes.slice(-1)[0];
    expect(lastVideo?.mediaType).toBe("video");
    expect(lastVideo?.mediaUrl).toBe("demo_clip.webm");
    expect(lastVideo?.mediaFormat).toBe("webm");

    const resAudio = state.execCommand("InsertAudio", "track.mp3");
    expect(resAudio).toBe(true);
    const lastAudio = state.getCurrentSlide()?.shapes.slice(-1)[0];
    expect(lastAudio?.mediaType).toBe("audio");
    expect(lastAudio?.mediaUrl).toBe("track.mp3");
    expect(lastAudio?.mediaFormat).toBe("mp3");
  });

  it("4. renders embedded video element and overlay controls in DOM", () => {
    const videoShape = state.insertMedia("video", "product_launch.mp4", 120, 90, 480, 270);
    overlay.refresh();

    const widget = container.querySelector(`[data-shape-id="${videoShape.id}"]`) as HTMLElement;
    expect(widget).not.toBeNull();
    expect(widget.classList.contains("slide-media-widget")).toBe(true);
    expect(widget.classList.contains("slide-media-video")).toBe(true);
    expect(widget.style.left).toBe("120px");
    expect(widget.style.top).toBe("90px");
    expect(widget.style.width).toBe("480px");
    expect(widget.style.height).toBe("270px");

    const videoEl = widget.querySelector("video") as HTMLVideoElement;
    expect(videoEl).not.toBeNull();
    expect(videoEl.src).toContain("product_launch.mp4");
    expect(videoEl.poster).toContain("poster_frame.png");

    const controls = widget.querySelector(".slide-media-controls");
    expect(controls).not.toBeNull();
    expect(controls?.querySelector(".media-btn-play")).not.toBeNull();
    expect(controls?.querySelector(".media-scrub-bar")).not.toBeNull();
    expect(controls?.querySelector(".media-btn-mute")).not.toBeNull();
    expect(controls?.querySelector(".media-btn-loop")).not.toBeNull();
    expect(controls?.querySelector(".media-btn-poster")).not.toBeNull();
  });

  it("5. renders audio card and controls for audio shape", () => {
    const audioShape = state.insertMedia("audio", "narration.mp3", 40, 60, 260, 50);
    overlay.refresh();

    const widget = container.querySelector(`[data-shape-id="${audioShape.id}"]`) as HTMLElement;
    expect(widget).not.toBeNull();
    expect(widget.classList.contains("slide-media-audio")).toBe(true);

    const audioEl = widget.querySelector("audio") as HTMLAudioElement;
    expect(audioEl).not.toBeNull();
    expect(audioEl.src).toContain("narration.mp3");

    // Audio widget should not show poster button
    const posterBtn = widget.querySelector(".media-btn-poster");
    expect(posterBtn).toBeNull();
  });

  it("6. toggles playback state and fires change callback", () => {
    const videoShape = state.insertMedia("video", "talk.mp4");
    overlay.refresh();

    let callbackFired = false;
    let callbackState = false;
    overlay.setOnMediaStateChange((id, isPlaying) => {
      if (id === videoShape.id) {
        callbackFired = true;
        callbackState = isPlaying;
      }
    });

    expect(videoShape.isPlaying).toBe(false);
    overlay.togglePlay(videoShape.id);
    expect(videoShape.isPlaying).toBe(true);
    expect(callbackFired).toBe(true);
    expect(callbackState).toBe(true);

    overlay.togglePlay(videoShape.id);
    expect(videoShape.isPlaying).toBe(false);
    expect(callbackState).toBe(false);
  });

  it("7. updates volume and mute state", () => {
    const audioShape = state.insertMedia("audio", "bgm.mp3");
    overlay.refresh();

    overlay.setVolume(audioShape.id, 0.65);
    expect(audioShape.volume).toBe(0.65);
    const mediaEl = overlay.getMediaElement(audioShape.id);
    expect(mediaEl?.volume).toBe(0.65);

    // Clamps to [0, 1]
    overlay.setVolume(audioShape.id, 1.5);
    expect(audioShape.volume).toBe(1.0);

    const widget = container.querySelector(`[data-shape-id="${audioShape.id}"]`)!;
    const muteBtn = widget.querySelector(".media-btn-mute") as HTMLButtonElement;
    expect(muteBtn.innerHTML).toBe("🔊");
    muteBtn.click();
    expect(audioShape.isMuted).toBe(true);
    expect(muteBtn.innerHTML).toBe("🔇");
  });

  it("8. toggles loop and updates poster frame", () => {
    const videoShape = state.insertMedia("video", "clip.mp4");
    overlay.refresh();

    expect(videoShape.loopPlayback).toBe(false);
    overlay.toggleLoop(videoShape.id);
    expect(videoShape.loopPlayback).toBe(true);

    overlay.setPosterFrame(videoShape.id, "new_poster.jpg");
    expect(videoShape.posterUrl).toBe("new_poster.jpg");
    const videoEl = overlay.getMediaElement(videoShape.id) as HTMLVideoElement;
    expect(videoEl.poster).toContain("new_poster.jpg");
  });

  it("9. integrates SlideMediaOverlay with UnifiedShellApp in PowerPoint mode", () => {
    const app = new UnifiedShellApp(false);
    const tabBar = app.getTabBar();
    tabBar.addTab({
      id: "tab-ppt-1",
      title: "Deck1.pptx",
      docType: "PowerPoint",
      isDirty: false,
    });

    const overlayInstance = app.getSlideMediaOverlay();
    expect(overlayInstance).toBeDefined();

    // Insert media via ribbon action emitter
    app.handleRibbonAction("InsertVideo", "keynote.mp4");
    const currentSlide = slidesState.getCurrentSlide();
    const videoShape = currentSlide?.shapes.find((s) => s.mediaUrl === "keynote.mp4");
    expect(videoShape).toBeDefined();
    expect(videoShape?.mediaType).toBe("video");

    const mediaEls = overlayInstance?.getMediaElements();
    expect(mediaEls?.size).toBeGreaterThan(0);
  });
});
