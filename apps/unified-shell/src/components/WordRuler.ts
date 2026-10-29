export class WordRuler {
  private container: HTMLElement;
  private marginLeft = 96; // 1 inch default (96px)
  private marginRight = 96;
  private firstLineIndent = 0;
  private onMarginsChangeCallback?: (left: number, right: number, indent: number) => void;

  private isDragging = false;
  private dragTarget: "left" | "right" | "indent" | null = null;
  private startX = 0;
  private startVal = 0;

  constructor(containerId: string) {
    const el = document.getElementById(containerId);
    if (!el) throw new Error(`WordRuler container #${containerId} not found`);
    this.container = el;
    this.render();
    this.attachEvents();
  }

  public setOnMarginsChange(cb: (left: number, right: number, indent: number) => void) {
    this.onMarginsChangeCallback = cb;
  }

  public setMargins(left: number, right: number, indent = 0) {
    this.marginLeft = Math.max(24, Math.min(300, left));
    this.marginRight = Math.max(24, Math.min(300, right));
    this.firstLineIndent = Math.max(-48, Math.min(144, indent));
    this.updatePositions();
    if (this.onMarginsChangeCallback) {
      this.onMarginsChangeCallback(this.marginLeft, this.marginRight, this.firstLineIndent);
    }
  }

  public getMargins(): { left: number; right: number; indent: number } {
    return {
      left: this.marginLeft,
      right: this.marginRight,
      indent: this.firstLineIndent,
    };
  }

  private attachEvents() {
    window.addEventListener("mousemove", (e: MouseEvent) => {
      if (!this.isDragging || !this.dragTarget) return;

      const deltaX = e.clientX - this.startX;
      if (this.dragTarget === "left") {
        this.marginLeft = Math.max(24, Math.min(300, this.startVal + deltaX));
      } else if (this.dragTarget === "right") {
        this.marginRight = Math.max(24, Math.min(300, this.startVal - deltaX));
      } else if (this.dragTarget === "indent") {
        this.firstLineIndent = Math.max(-48, Math.min(144, this.startVal + deltaX));
      }

      this.updatePositions();
      if (this.onMarginsChangeCallback) {
        this.onMarginsChangeCallback(this.marginLeft, this.marginRight, this.firstLineIndent);
      }
    });

    window.addEventListener("mouseup", () => {
      if (this.isDragging) {
        this.isDragging = false;
        this.dragTarget = null;
      }
    });
  }

  private startDrag(target: "left" | "right" | "indent", clientX: number) {
    this.isDragging = true;
    this.dragTarget = target;
    this.startX = clientX;
    if (target === "left") {
      this.startVal = this.marginLeft;
    } else if (target === "right") {
      this.startVal = this.marginRight;
    } else {
      this.startVal = this.firstLineIndent;
    }
  }

  private updatePositions() {
    const leftMargin = this.container.querySelector<HTMLElement>(".ruler-left-margin");
    const rightMargin = this.container.querySelector<HTMLElement>(".ruler-right-margin");
    const indentSlider = this.container.querySelector<HTMLElement>(".ruler-indent-slider");

    if (leftMargin) {
      leftMargin.style.width = `${this.marginLeft}px`;
    }
    if (rightMargin) {
      rightMargin.style.width = `${this.marginRight}px`;
    }
    if (indentSlider) {
      indentSlider.style.left = `${this.marginLeft + this.firstLineIndent}px`;
    }
  }

  private render() {
    this.container.innerHTML = "";

    const rulerBox = document.createElement("div");
    rulerBox.className = "word-ruler-container";

    const track = document.createElement("div");
    track.className = "ruler-track";

    // Left margin block (draggable handle)
    const leftMargin = document.createElement("div");
    leftMargin.className = "ruler-margin-box ruler-left-margin";
    leftMargin.style.left = "0px";
    leftMargin.style.width = `${this.marginLeft}px`;
    leftMargin.title = "Drag to adjust Left Margin";
    leftMargin.onmousedown = (e) => {
      e.preventDefault();
      this.startDrag("left", e.clientX);
    };

    // Right margin block (draggable handle)
    const rightMargin = document.createElement("div");
    rightMargin.className = "ruler-margin-box ruler-right-margin";
    rightMargin.style.right = "0px";
    rightMargin.style.width = `${this.marginRight}px`;
    rightMargin.title = "Drag to adjust Right Margin";
    rightMargin.onmousedown = (e) => {
      e.preventDefault();
      this.startDrag("right", e.clientX);
    };

    // First line indent slider marker
    const indentSlider = document.createElement("div");
    indentSlider.className = "ruler-indent-slider";
    indentSlider.style.left = `${this.marginLeft + this.firstLineIndent}px`;
    indentSlider.title = "Drag to adjust First Line Indent";
    indentSlider.onmousedown = (e) => {
      e.preventDefault();
      e.stopPropagation();
      this.startDrag("indent", e.clientX);
    };

    track.appendChild(leftMargin);
    track.appendChild(rightMargin);
    track.appendChild(indentSlider);
    rulerBox.appendChild(track);

    this.container.appendChild(rulerBox);
  }
}
