export class LaserPointer {
  private dotEl: HTMLElement;
  private isActive = false;

  constructor(containerId = "laser-pointer-dot") {
    let el = document.getElementById(containerId);
    if (!el) {
      el = document.createElement("div");
      el.id = containerId;
      el.className = "laser-pointer-dot";
      el.style.display = "none";
      const viewport = document.getElementById("viewport-container");
      if (viewport) viewport.appendChild(el);
      else document.body.appendChild(el);
    }
    this.dotEl = el;
    this.attachEvents();
  }

  public setActive(active: boolean) {
    this.isActive = active;
    this.dotEl.style.display = active ? "block" : "none";
  }

  private attachEvents() {
    const viewport = document.getElementById("viewport-container") || document.body;

    viewport.addEventListener("pointermove", (e: PointerEvent) => {
      if (!this.isActive) return;
      const rect = viewport.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      this.dotEl.style.left = `${x}px`;
      this.dotEl.style.top = `${y}px`;
    });
  }
}

