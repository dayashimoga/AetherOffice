export interface SlideThumbnail {
  id: number;
  slideNumber: number;
  title: string;
  shapeCount?: number;
  isHidden?: boolean;
}

export class SlideSorter {
  private container: HTMLElement;
  private slides: SlideThumbnail[] = [];
  private activeSlideId: number | null = null;
  private onSelectSlideCallback?: (id: number) => void;
  private onReorderCallback?: (fromIdx: number, toIdx: number) => void;
  private onAddSlideCallback?: () => void;
  private onDuplicateSlideCallback?: (idx: number) => void;
  private onDeleteSlideCallback?: (idx: number) => void;
  private onToggleHideCallback?: (idx: number) => void;
  private onMoveUpCallback?: (idx: number) => void;
  private onMoveDownCallback?: (idx: number) => void;

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
    onAddSlide?: () => void;
    onDuplicateSlide?: (idx: number) => void;
    onDeleteSlide?: (idx: number) => void;
    onToggleHide?: (idx: number) => void;
    onMoveUp?: (idx: number) => void;
    onMoveDown?: (idx: number) => void;
  }) {
    this.onSelectSlideCallback = callbacks.onSelectSlide;
    this.onReorderCallback = callbacks.onReorder;
    this.onAddSlideCallback = callbacks.onAddSlide;
    this.onDuplicateSlideCallback = callbacks.onDuplicateSlide;
    this.onDeleteSlideCallback = callbacks.onDeleteSlide;
    this.onToggleHideCallback = callbacks.onToggleHide;
    this.onMoveUpCallback = callbacks.onMoveUp;
    this.onMoveDownCallback = callbacks.onMoveDown;
  }

  private render() {
    this.container.innerHTML = "";

    // Header with Count & Add Slide button
    const header = document.createElement("div");
    header.className = "sidebar-heading";
    header.style.display = "flex";
    header.style.justifyContent = "space-between";
    header.style.alignItems = "center";

    const titleSpan = document.createElement("span");
    titleSpan.textContent = `Slides (${this.slides.length})`;
    header.appendChild(titleSpan);

    const addBtn = document.createElement("button");
    addBtn.className = "btn-sidebar-add";
    addBtn.style.padding = "2px 8px";
    addBtn.style.fontSize = "12px";
    addBtn.style.cursor = "pointer";
    addBtn.style.borderRadius = "4px";
    addBtn.style.border = "1px solid #cbd5e1";
    addBtn.style.background = "#f8fafc";
    addBtn.textContent = "+ Slide";
    addBtn.onclick = (e) => {
      e.stopPropagation();
      if (this.onAddSlideCallback) this.onAddSlideCallback();
    };
    header.appendChild(addBtn);

    this.container.appendChild(header);

    const filmstrip = document.createElement("div");
    filmstrip.className = "slide-sorter-filmstrip";

    let draggedIdx: number | null = null;

    this.slides.forEach((slide, idx) => {
      const card = document.createElement("div");
      card.className = `slide-thumbnail-card ${slide.id === this.activeSlideId ? "active" : ""} ${slide.isHidden ? "slide-hidden" : ""}`;
      card.draggable = true;
      card.style.position = "relative";
      if (slide.isHidden) {
        card.style.opacity = "0.55";
      }

      card.ondragstart = () => {
        draggedIdx = idx;
      };

      card.ondragover = (e) => {
        e.preventDefault();
        card.style.borderTop = "2px solid #c43e1c";
      };

      card.ondragleave = () => {
        card.style.borderTop = "";
      };

      card.ondrop = (e) => {
        e.preventDefault();
        card.style.borderTop = "";
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
      preview.style.display = "flex";
      preview.style.flexDirection = "column";
      preview.style.justifyContent = "center";
      preview.style.padding = "4px 8px";

      const titleEl = document.createElement("div");
      titleEl.style.fontWeight = "600";
      titleEl.style.fontSize = "12px";
      titleEl.style.overflow = "hidden";
      titleEl.style.textOverflow = "ellipsis";
      titleEl.style.whiteSpace = "nowrap";
      titleEl.textContent = slide.title || `Slide ${slide.slideNumber}`;
      preview.appendChild(titleEl);

      const metaRow = document.createElement("div");
      metaRow.style.display = "flex";
      metaRow.style.gap = "6px";
      metaRow.style.fontSize = "10px";
      metaRow.style.color = "#64748b";
      metaRow.style.marginTop = "2px";

      if (slide.shapeCount !== undefined) {
        const countSpan = document.createElement("span");
        countSpan.textContent = `${slide.shapeCount} shape${slide.shapeCount === 1 ? "" : "s"}`;
        metaRow.appendChild(countSpan);
      }

      if (slide.isHidden) {
        const hideBadge = document.createElement("span");
        hideBadge.style.color = "#ef4444";
        hideBadge.textContent = "[Hidden]";
        metaRow.appendChild(hideBadge);
      }

      preview.appendChild(metaRow);

      // Mini Quick Action Controls
      const actions = document.createElement("div");
      actions.className = "slide-thumb-actions";
      actions.style.position = "absolute";
      actions.style.top = "4px";
      actions.style.right = "4px";
      actions.style.display = "flex";
      actions.style.gap = "2px";

      const upBtn = document.createElement("button");
      upBtn.title = "Move Slide Up";
      upBtn.textContent = "▲";
      upBtn.style.fontSize = "9px";
      upBtn.style.cursor = "pointer";
      upBtn.style.background = "transparent";
      upBtn.style.border = "none";
      upBtn.onclick = (e) => {
        e.stopPropagation();
        if (this.onMoveUpCallback) this.onMoveUpCallback(idx);
      };
      actions.appendChild(upBtn);

      const downBtn = document.createElement("button");
      downBtn.title = "Move Slide Down";
      downBtn.textContent = "▼";
      downBtn.style.fontSize = "9px";
      downBtn.style.cursor = "pointer";
      downBtn.style.background = "transparent";
      downBtn.style.border = "none";
      downBtn.onclick = (e) => {
        e.stopPropagation();
        if (this.onMoveDownCallback) this.onMoveDownCallback(idx);
      };
      actions.appendChild(downBtn);

      const dupBtn = document.createElement("button");
      dupBtn.title = "Duplicate Slide";
      dupBtn.textContent = "📋";
      dupBtn.style.fontSize = "9px";
      dupBtn.style.cursor = "pointer";
      dupBtn.style.background = "transparent";
      dupBtn.style.border = "none";
      dupBtn.onclick = (e) => {
        e.stopPropagation();
        if (this.onDuplicateSlideCallback) this.onDuplicateSlideCallback(idx);
      };
      actions.appendChild(dupBtn);

      const hideBtn = document.createElement("button");
      hideBtn.title = slide.isHidden ? "Unhide Slide" : "Hide Slide";
      hideBtn.textContent = slide.isHidden ? "👁️‍🗨️" : "👁️";
      hideBtn.style.fontSize = "9px";
      hideBtn.style.cursor = "pointer";
      hideBtn.style.background = "transparent";
      hideBtn.style.border = "none";
      hideBtn.onclick = (e) => {
        e.stopPropagation();
        if (this.onToggleHideCallback) this.onToggleHideCallback(idx);
      };
      actions.appendChild(hideBtn);

      const delBtn = document.createElement("button");
      delBtn.title = "Delete Slide";
      delBtn.textContent = "🗑️";
      delBtn.style.fontSize = "9px";
      delBtn.style.cursor = "pointer";
      delBtn.style.background = "transparent";
      delBtn.style.border = "none";
      delBtn.onclick = (e) => {
        e.stopPropagation();
        if (this.onDeleteSlideCallback) this.onDeleteSlideCallback(idx);
      };
      actions.appendChild(delBtn);

      card.appendChild(num);
      card.appendChild(preview);
      card.appendChild(actions);
      filmstrip.appendChild(card);
    });

    this.container.appendChild(filmstrip);
  }
}
