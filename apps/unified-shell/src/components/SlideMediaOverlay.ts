/**
 * DrawingML Embedded Audio/Video Media Engine.
 * Renders embedded HTML5 media elements overlaying the vector slide canvas
 * with precise bounding box coordinates, scrub bars, playback state, and poster frame caching.
 */

import { slidesState, SlidesState, SlideShape } from "../state/slidesState";

export class SlideMediaOverlay {
  private container: HTMLElement;
  private overlayEl: HTMLElement;
  private state: SlidesState;
  private mediaElements: Map<string, HTMLMediaElement> = new Map();
  private onMediaStateChange?: (shapeId: string, isPlaying: boolean) => void;

  constructor(containerOrId: HTMLElement | string = "viewport-container", state: SlidesState = slidesState) {
    this.state = state;
    if (typeof containerOrId === "string") {
      let el = document.getElementById(containerOrId);
      if (!el) {
        el = document.createElement("div");
        el.id = containerOrId;
        document.body.appendChild(el);
      }
      this.container = el;
    } else {
      this.container = containerOrId;
    }

    this.overlayEl = document.createElement("div");
    this.overlayEl.className = "slide-media-overlay";
    this.overlayEl.style.position = "absolute";
    this.overlayEl.style.top = "0";
    this.overlayEl.style.left = "0";
    this.overlayEl.style.width = "100%";
    this.overlayEl.style.height = "100%";
    this.overlayEl.style.pointerEvents = "none";
    this.overlayEl.style.zIndex = "25";
    this.container.appendChild(this.overlayEl);

    this.refresh();
  }

  public setOnMediaStateChange(cb: (shapeId: string, isPlaying: boolean) => void): void {
    this.onMediaStateChange = cb;
  }

  public getMediaElements(): Map<string, HTMLMediaElement> {
    return this.mediaElements;
  }

  public getMediaElement(shapeId: string): HTMLMediaElement | undefined {
    return this.mediaElements.get(shapeId);
  }

  /**
   * Re-renders media widgets for all media shapes on the current slide.
   */
  public refresh(): void {
    this.overlayEl.innerHTML = "";
    this.mediaElements.clear();

    const slide = this.state.getCurrentSlide();
    if (!slide || !slide.shapes) return;

    for (const shape of slide.shapes) {
      if (shape.mediaType === "video" || shape.mediaType === "audio") {
        this.renderMediaWidget(shape);
      }
    }
  }

  private renderMediaWidget(shape: SlideShape): void {
    const isVideo = shape.mediaType === "video";
    const widget = document.createElement("div");
    widget.className = `slide-media-widget slide-media-${shape.mediaType}`;
    widget.dataset.shapeId = shape.id;
    widget.style.position = "absolute";
    widget.style.left = `${shape.x}px`;
    widget.style.top = `${shape.y}px`;
    widget.style.width = `${shape.w}px`;
    widget.style.height = `${shape.h}px`;
    widget.style.pointerEvents = "auto";
    widget.style.display = "flex";
    widget.style.flexDirection = "column";
    widget.style.borderRadius = "6px";
    widget.style.overflow = "hidden";
    widget.style.boxShadow = "0 4px 12px rgba(0, 0, 0, 0.15)";
    widget.style.backgroundColor = isVideo ? "#000000" : "#f8fafc";
    widget.style.border = "1px solid #64748b";

    let mediaEl: HTMLMediaElement;

    if (isVideo) {
      const videoEl = document.createElement("video");
      videoEl.className = "slide-video-element";
      videoEl.src = shape.mediaUrl || "";
      if (shape.posterUrl) {
        videoEl.poster = shape.posterUrl;
      }
      videoEl.style.width = "100%";
      videoEl.style.flex = "1";
      videoEl.style.objectFit = "contain";
      videoEl.style.backgroundColor = "#000000";
      videoEl.loop = !!shape.loopPlayback;
      videoEl.volume = shape.volume ?? 1.0;
      videoEl.muted = !!shape.isMuted;
      videoEl.autoplay = !!shape.autoPlay;

      widget.appendChild(videoEl);
      mediaEl = videoEl;
    } else {
      // Audio display card
      const audioCard = document.createElement("div");
      audioCard.style.display = "flex";
      audioCard.style.alignItems = "center";
      audioCard.style.gap = "8px";
      audioCard.style.padding = "6px 10px";
      audioCard.style.flex = "1";

      const icon = document.createElement("span");
      icon.innerHTML = "🔊";
      icon.style.fontSize = "16px";
      audioCard.appendChild(icon);

      const title = document.createElement("span");
      title.innerText = shape.mediaUrl?.split("/").pop() || "Embedded Audio";
      title.style.fontSize = "12px";
      title.style.fontWeight = "600";
      title.style.color = "#1e293b";
      audioCard.appendChild(title);

      const audioEl = document.createElement("audio");
      audioEl.className = "slide-audio-element";
      audioEl.src = shape.mediaUrl || "";
      audioEl.loop = !!shape.loopPlayback;
      audioEl.volume = shape.volume ?? 1.0;
      audioEl.muted = !!shape.isMuted;
      audioEl.autoplay = !!shape.autoPlay;

      widget.appendChild(audioCard);
      widget.appendChild(audioEl);
      mediaEl = audioEl;
    }

    this.mediaElements.set(shape.id, mediaEl);

    // Controls Bar
    const controls = document.createElement("div");
    controls.className = "slide-media-controls";
    controls.style.display = "flex";
    controls.style.alignItems = "center";
    controls.style.gap = "6px";
    controls.style.padding = "4px 8px";
    controls.style.backgroundColor = isVideo ? "rgba(15, 23, 42, 0.85)" : "#e2e8f0";
    controls.style.color = isVideo ? "#ffffff" : "#0f172a";
    controls.style.fontSize = "11px";

    // Play / Pause Button
    const playBtn = document.createElement("button");
    playBtn.className = "media-btn-play";
    playBtn.innerHTML = shape.isPlaying ? "⏸" : "▶";
    playBtn.style.background = "transparent";
    playBtn.style.border = "none";
    playBtn.style.color = "inherit";
    playBtn.style.cursor = "pointer";
    playBtn.style.fontSize = "13px";
    playBtn.onclick = () => {
      this.togglePlay(shape.id);
      playBtn.innerHTML = shape.isPlaying ? "⏸" : "▶";
    };
    controls.appendChild(playBtn);

    // Time scrub slider
    const scrub = document.createElement("input");
    scrub.type = "range";
    scrub.className = "media-scrub-bar";
    scrub.min = "0";
    scrub.max = "100";
    scrub.value = "0";
    scrub.style.flex = "1";
    scrub.style.cursor = "pointer";
    scrub.oninput = (e) => {
      const val = parseFloat((e.target as HTMLInputElement).value);
      if (mediaEl.duration) {
        mediaEl.currentTime = (val / 100) * mediaEl.duration;
      }
    };
    controls.appendChild(scrub);

    // Update scrub on timeupdate
    mediaEl.ontimeupdate = () => {
      if (mediaEl.duration) {
        scrub.value = String((mediaEl.currentTime / mediaEl.duration) * 100);
      }
    };

    // Volume / Mute
    const muteBtn = document.createElement("button");
    muteBtn.className = "media-btn-mute";
    muteBtn.innerHTML = shape.isMuted ? "🔇" : "🔊";
    muteBtn.style.background = "transparent";
    muteBtn.style.border = "none";
    muteBtn.style.color = "inherit";
    muteBtn.style.cursor = "pointer";
    muteBtn.onclick = () => {
      shape.isMuted = !shape.isMuted;
      mediaEl.muted = shape.isMuted;
      muteBtn.innerHTML = shape.isMuted ? "🔇" : "🔊";
    };
    controls.appendChild(muteBtn);

    // Loop Toggle
    const loopBtn = document.createElement("button");
    loopBtn.className = "media-btn-loop";
    loopBtn.innerHTML = "🔁";
    loopBtn.title = "Toggle Loop";
    loopBtn.style.background = "transparent";
    loopBtn.style.border = "none";
    loopBtn.style.color = shape.loopPlayback ? "#38bdf8" : "inherit";
    loopBtn.style.cursor = "pointer";
    loopBtn.onclick = () => {
      shape.loopPlayback = !shape.loopPlayback;
      mediaEl.loop = shape.loopPlayback;
      loopBtn.style.color = shape.loopPlayback ? "#38bdf8" : "inherit";
    };
    controls.appendChild(loopBtn);

    // Poster Frame Button (for Video)
    if (isVideo) {
      const posterBtn = document.createElement("button");
      posterBtn.className = "media-btn-poster";
      posterBtn.innerHTML = "🖼️";
      posterBtn.title = "Set Poster Frame";
      posterBtn.style.background = "transparent";
      posterBtn.style.border = "none";
      posterBtn.style.color = "inherit";
      posterBtn.style.cursor = "pointer";
      posterBtn.onclick = () => {
        const newPoster = "custom_poster_frame.png";
        this.setPosterFrame(shape.id, newPoster);
      };
      controls.appendChild(posterBtn);
    }

    widget.appendChild(controls);
    this.overlayEl.appendChild(widget);
  }

  public togglePlay(shapeId: string): void {
    const slide = this.state.getCurrentSlide();
    const shape = slide?.shapes.find((s) => s.id === shapeId);
    const mediaEl = this.mediaElements.get(shapeId);
    if (!shape || !mediaEl) return;

    if (shape.isPlaying) {
      shape.isPlaying = false;
      if (typeof mediaEl.pause === "function") {
        mediaEl.pause();
      }
    } else {
      shape.isPlaying = true;
      if (typeof mediaEl.play === "function") {
        mediaEl.play().catch(() => {});
      }
    }

    if (this.onMediaStateChange) {
      this.onMediaStateChange(shapeId, shape.isPlaying);
    }
  }

  public setVolume(shapeId: string, volume: number): void {
    const slide = this.state.getCurrentSlide();
    const shape = slide?.shapes.find((s) => s.id === shapeId);
    const mediaEl = this.mediaElements.get(shapeId);
    if (shape && mediaEl) {
      shape.volume = Math.max(0, Math.min(1, volume));
      mediaEl.volume = shape.volume;
    }
  }

  public toggleLoop(shapeId: string): void {
    const slide = this.state.getCurrentSlide();
    const shape = slide?.shapes.find((s) => s.id === shapeId);
    const mediaEl = this.mediaElements.get(shapeId);
    if (shape && mediaEl) {
      shape.loopPlayback = !shape.loopPlayback;
      mediaEl.loop = shape.loopPlayback;
    }
  }

  public setPosterFrame(shapeId: string, posterUrl: string): void {
    const slide = this.state.getCurrentSlide();
    const shape = slide?.shapes.find((s) => s.id === shapeId);
    const mediaEl = this.mediaElements.get(shapeId);
    if (shape) {
      shape.posterUrl = posterUrl;
      if (mediaEl instanceof HTMLVideoElement) {
        mediaEl.poster = posterUrl;
      }
    }
  }

  public clear(): void {
    this.overlayEl.innerHTML = "";
    this.mediaElements.clear();
  }
}

