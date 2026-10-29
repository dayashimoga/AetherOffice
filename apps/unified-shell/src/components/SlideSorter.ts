export interface SlideThumbnail {
  id: number;
  slideNumber: number;
  title: string;
}

export class SlideSorter {
  private container: HTMLElement;
  private slides: SlideThumbnail[] = [];
  private activeSlideId: number | null = null;
  private onSelectSlideCallback?: (id: number) => void;
  private onReorderCallback?: (fromIdx: number, toIdx: number) => void;

  constructor(containerId: string) {
    const el = document.getElementById(containerId);
    if (!el) throw new Error(`SlideSorter container #${containerId} not found`);
    this.container = el;
    this.render();
  }

  public setSlides(slides: SlideThumbnail[], activeId?: number) {
    this.slides = slides;
    this.activeSlideId = activeId !== undefined ? activeId : slides[0]?.id || null;
    this.render();
  }

  public setCallbacks(callbacks: {
    onSelectSlide?: (id: number) => void;
    onReorder?: (fromIdx: number, toIdx: number) => void;
  }) {
    this.onSelectSlideCallback = callbacks.onSelectSlide;
    this.onReorderCallback = callbacks.onReorder;
  }

  private render() {
    this.container.innerHTML = "";

    const header = document.createElement("div");
    header.className = "sidebar-heading";
    header.textContent = `Slides (${this.slides.length})`;
    this.container.appendChild(header);

    const filmstrip = document.createElement("div");
    filmstrip.className = "slide-sorter-filmstrip";

    let draggedIdx: number | null = null;

    this.slides.forEach((slide, idx) => {
      const card = document.createElement("div");
      card.className = `slide-thumbnail-card ${slide.id === this.activeSlideId ? "active" : ""}`;
      card.draggable = true;

      card.ondragstart = () => {
        draggedIdx = idx;
      };

      card.ondragover = (e) => {
        e.preventDefault();
      };

      card.ondrop = (e) => {
        e.preventDefault();
        if (draggedIdx !== null && draggedIdx !== idx) {
          if (this.onReorderCallback) {
            this.onReorderCallback(draggedIdx, idx);
          }
        }
      };

      card.onclick = () => {
        this.activeSlideId = slide.id;
        this.render();
        if (this.onSelectSlideCallback) {
          this.onSelectSlideCallback(slide.id);
        }
      };

      const num = document.createElement("span");
      num.className = "slide-thumb-num";
      num.textContent = String(slide.slideNumber);

      const preview = document.createElement("div");
      preview.className = "slide-thumb-preview";
      preview.textContent = slide.title || `Slide ${slide.slideNumber}`;

      card.appendChild(num);
      card.appendChild(preview);
      filmstrip.appendChild(card);
    });

    this.container.appendChild(filmstrip);
  }
}

