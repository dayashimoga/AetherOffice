export type SheetState = "Collapsed" | "HalfExpanded" | "FullExpanded";

export class BottomSheet {
  private el: HTMLElement;
  private state: SheetState = "Collapsed";
  private startY = 0;
  private currentY = 0;
  private onActionCallback?: (action: string) => void;

  constructor(containerId = "bottom-sheet") {
    let element = document.getElementById(containerId);
    if (!element) {
      element = document.createElement("div");
      element.id = containerId;
      element.className = "bottom-sheet";
      document.body.appendChild(element);
    }
    this.el = element;
    this.render();
    this.attachEvents();
  }

  public setOnAction(callback: (action: string) => void) {
    this.onActionCallback = callback;
  }

  public setState(state: SheetState) {
    this.state = state;
    this.updatePosition();
  }

  private updatePosition() {
    switch (this.state) {
      case "Collapsed":
        this.el.style.transform = "translateY(calc(100% - var(--bottom-sheet-collapsed-h)))";
        break;
      case "HalfExpanded":
        this.el.style.transform = "translateY(calc(100% - var(--bottom-sheet-half-h)))";
        break;
      case "FullExpanded":
        this.el.style.transform = "translateY(0%)";
        break;
    }
  }

  private attachEvents() {
    const handle = this.el.querySelector(".bottom-sheet-handle") as HTMLElement | null;
    if (!handle) return;

    handle.addEventListener("touchstart", ((e: TouchEvent) => {
      this.startY = e.touches[0].clientY;
      this.currentY = this.startY;
    }) as EventListener);

    handle.addEventListener("touchmove", ((e: TouchEvent) => {
      this.currentY = e.touches[0].clientY;
    }) as EventListener);

    handle.addEventListener("touchend", () => {
      const deltaY = this.currentY - this.startY;
      if (deltaY < -60) {
        // Swiped up
        if (this.state === "Collapsed") this.setState("HalfExpanded");
        else if (this.state === "HalfExpanded") this.setState("FullExpanded");
      } else if (deltaY > 60) {
        // Swiped down
        if (this.state === "FullExpanded") this.setState("HalfExpanded");
        else if (this.state === "HalfExpanded") this.setState("Collapsed");
      }
    });

    // Tap handle toggles between collapsed and half expanded
    handle.addEventListener("click", () => {
      if (this.state === "Collapsed") {
        this.setState("HalfExpanded");
      } else {
        this.setState("Collapsed");
      }
    });
  }

  private render() {
    this.el.innerHTML = `
      <div class="bottom-sheet-handle"></div>
      <div class="bottom-sheet-quick-actions">
        <button class="ribbon-btn" data-action="Undo" title="Undo">↩️</button>
        <button class="ribbon-btn" data-action="ToggleBold" title="Bold"><b>B</b></button>
        <button class="ribbon-btn" data-action="ToggleItalic" title="Italic"><i>I</i></button>
        <button class="ribbon-btn" data-action="ToggleInk" title="Ink">✏️</button>
        <button class="ribbon-btn" data-action="ToggleReflow" title="Reader View">📖</button>
      </div>
      <div class="bottom-sheet-content">
        <div style="font-weight: 600; margin-bottom: 8px;">Commands</div>
        <div style="display: flex; flex-direction: column; gap: 8px;">
          <button class="ribbon-btn" data-action="Save">💾 Save Document</button>
          <button class="ribbon-btn" data-action="ExportPDF">📄 Export to PDF</button>
          <button class="ribbon-btn" data-action="Find">🔍 Find in Document</button>
        </div>
      </div>
    `;

    this.el.querySelectorAll("[data-action]").forEach((btn) => {
      btn.addEventListener("click", (e) => {
        const action = (e.currentTarget as HTMLElement).getAttribute("data-action");
        if (action && this.onActionCallback) {
          this.onActionCallback(action);
        }
      });
    });

    this.updatePosition();
  }
}

