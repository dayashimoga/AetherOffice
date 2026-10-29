export class MobileReflowReader {
  private container: HTMLElement;
  private fontScale = 1.0;
  private paragraphs: string[] = [];

  constructor(containerId = "reflow-reader-container") {
    let el = document.getElementById(containerId);
    if (!el) {
      el = document.createElement("div");
      el.id = containerId;
      el.className = "reflow-reader-view";
      el.style.display = "none";
      const viewport = document.getElementById("viewport-container");
      if (viewport) viewport.appendChild(el);
      else document.body.appendChild(el);
    }
    this.container = el;
  }

  public setContent(paragraphs: string[]) {
    this.paragraphs = paragraphs;
    this.render();
  }

  public setFontScale(scale: number) {
    this.fontScale = Math.max(1.0, Math.min(2.5, scale));
    this.container.style.fontSize = `${16 * this.fontScale}px`;
  }

  public show() {
    this.container.style.display = "block";
    const surface = document.getElementById("render-surface");
    if (surface) surface.style.display = "none";
  }

  public hide() {
    this.container.style.display = "none";
    const surface = document.getElementById("render-surface");
    if (surface) surface.style.display = "block";
  }

  public toggle(): boolean {
    if (this.container.style.display === "none") {
      this.show();
      return true;
    } else {
      this.hide();
      return false;
    }
  }

  private render() {
    this.container.innerHTML = `
      <div class="reflow-controls-bar">
        <span><b>Reader Mode</b> (Mobile Reflow)</span>
        <div style="display: flex; gap: 8px;">
          <button id="reflow-font-down" class="ribbon-btn">A-</button>
          <button id="reflow-font-up" class="ribbon-btn">A+</button>
          <button id="reflow-close" class="ribbon-btn">✕</button>
        </div>
      </div>
      <div class="reflow-text-body" style="padding-top: 12px;">
        ${this.paragraphs.map((p) => `<p>${p}</p>`).join("")}
      </div>
    `;

    document.getElementById("reflow-font-up")?.addEventListener("click", () => {
      this.setFontScale(this.fontScale + 0.2);
    });

    document.getElementById("reflow-font-down")?.addEventListener("click", () => {
      this.setFontScale(this.fontScale - 0.2);
    });

    document.getElementById("reflow-close")?.addEventListener("click", () => {
      this.hide();
    });
  }
}
