export class FormatBackgroundDialog {
  private overlay: HTMLElement;
  private dialog: HTMLElement;
  private onApplyCallback?: (bg: string, applyToAll: boolean) => void;
  private selectedBackground = "#ffffff";

  private solidColors = [
    { name: "Pure White", color: "#ffffff" },
    { name: "Light Slate", color: "#f8fafc" },
    { name: "Mint Fresh", color: "#f0fdf4" },
    { name: "Soft Sky", color: "#f0f9ff" },
    { name: "Warm Cream", color: "#fff7ed" },
    { name: "Dark Slate", color: "#0f172a" },
    { name: "Pitch Black", color: "#000000" },
  ];

  private gradientPresets = [
    { name: "Soft Sky", gradient: "linear-gradient(135deg, #f0f9ff, #bae6fd)" },
    { name: "Sunset Warmth", gradient: "linear-gradient(135deg, #fff7ed, #fed7aa)" },
    { name: "Mint Meadow", gradient: "linear-gradient(135deg, #f0fdf4, #bbf7d0)" },
    { name: "Lavender Glow", gradient: "linear-gradient(135deg, #fdf2f8, #f5d0fe)" },
    { name: "Dark Modern", gradient: "linear-gradient(135deg, #1e293b, #0f172a)" },
  ];

  constructor() {
    this.overlay = document.createElement("div");
    this.overlay.className = "dialog-overlay format-background-dialog-overlay";
    this.overlay.style.position = "fixed";
    this.overlay.style.inset = "0";
    this.overlay.style.backgroundColor = "rgba(0, 0, 0, 0.45)";
    this.overlay.style.display = "none";
    this.overlay.style.justifyContent = "center";
    this.overlay.style.alignItems = "center";
    this.overlay.style.zIndex = "1000";

    this.dialog = document.createElement("div");
    this.dialog.className = "format-background-dialog";
    this.dialog.style.width = "460px";
    this.dialog.style.backgroundColor = "#ffffff";
    this.dialog.style.borderRadius = "8px";
    this.dialog.style.boxShadow = "0 20px 40px rgba(0, 0, 0, 0.25)";
    this.dialog.style.padding = "20px";
    this.dialog.style.fontFamily = "Segoe UI, sans-serif";

    this.render();
    this.overlay.appendChild(this.dialog);
    document.body.appendChild(this.overlay);

    this.overlay.onclick = (e) => {
      if (e.target === this.overlay) this.close();
    };

    window.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && this.isOpen()) {
        this.close();
      }
    });
  }

  public open(onApply?: (bg: string, applyToAll: boolean) => void) {
    if (onApply) this.onApplyCallback = onApply;
    this.overlay.style.display = "flex";
  }

  public close() {
    this.overlay.style.display = "none";
  }

  public isOpen(): boolean {
    return this.overlay.style.display === "flex";
  }

  public getSolidSwatches(): HTMLElement[] {
    return Array.from(this.dialog.querySelectorAll(".bg-swatch-solid"));
  }

  public getGradientSwatches(): HTMLElement[] {
    return Array.from(this.dialog.querySelectorAll(".bg-swatch-gradient"));
  }

  private render() {
    this.dialog.innerHTML = "";

    // Header
    const header = document.createElement("div");
    header.style.display = "flex";
    header.style.justifyContent = "space-between";
    header.style.alignItems = "center";
    header.style.marginBottom = "16px";

    const titleEl = document.createElement("h3");
    titleEl.textContent = "Format Slide Background";
    titleEl.style.margin = "0";
    titleEl.style.fontSize = "16px";
    titleEl.style.fontWeight = "600";
    titleEl.style.color = "#1e293b";
    header.appendChild(titleEl);

    const closeBtn = document.createElement("button");
    closeBtn.textContent = "✕";
    closeBtn.style.border = "none";
    closeBtn.style.background = "transparent";
    closeBtn.style.fontSize = "16px";
    closeBtn.style.cursor = "pointer";
    closeBtn.style.color = "#64748b";
    closeBtn.onclick = () => this.close();
    header.appendChild(closeBtn);
    this.dialog.appendChild(header);

    // Section 1: Solid Colors
    const solidSection = document.createElement("div");
    solidSection.style.marginBottom = "16px";

    const solidLabel = document.createElement("div");
    solidLabel.textContent = "Solid Color Fills";
    solidLabel.style.fontSize = "13px";
    solidLabel.style.fontWeight = "600";
    solidLabel.style.color = "#334155";
    solidLabel.style.marginBottom = "8px";
    solidSection.appendChild(solidLabel);

    const solidRow = document.createElement("div");
    solidRow.style.display = "flex";
    solidRow.style.gap = "8px";

    this.solidColors.forEach((sc) => {
      const swatch = document.createElement("button");
      swatch.className = "bg-swatch-solid";
      swatch.title = sc.name;
      swatch.setAttribute("data-color", sc.color);
      swatch.style.width = "32px";
      swatch.style.height = "32px";
      swatch.style.borderRadius = "4px";
      swatch.style.border = "1.5px solid #cbd5e1";
      swatch.style.backgroundColor = sc.color;
      swatch.style.cursor = "pointer";
      swatch.onclick = () => {
        this.selectedBackground = sc.color;
        this.highlightSelected(swatch);
      };
      solidRow.appendChild(swatch);
    });
    solidSection.appendChild(solidRow);
    this.dialog.appendChild(solidSection);

    // Section 2: Gradient Fills
    const gradSection = document.createElement("div");
    gradSection.style.marginBottom = "20px";

    const gradLabel = document.createElement("div");
    gradLabel.textContent = "Gradient Presets";
    gradLabel.style.fontSize = "13px";
    gradLabel.style.fontWeight = "600";
    gradLabel.style.color = "#334155";
    gradLabel.style.marginBottom = "8px";
    gradSection.appendChild(gradLabel);

    const gradRow = document.createElement("div");
    gradRow.style.display = "flex";
    gradRow.style.gap = "8px";

    this.gradientPresets.forEach((gp) => {
      const swatch = document.createElement("button");
      swatch.className = "bg-swatch-gradient";
      swatch.title = gp.name;
      swatch.setAttribute("data-gradient", gp.gradient);
      swatch.style.width = "48px";
      swatch.style.height = "32px";
      swatch.style.borderRadius = "4px";
      swatch.style.border = "1.5px solid #cbd5e1";
      swatch.style.background = gp.gradient;
      swatch.style.cursor = "pointer";
      swatch.onclick = () => {
        this.selectedBackground = gp.gradient;
        this.highlightSelected(swatch);
      };
      gradRow.appendChild(swatch);
    });
    gradSection.appendChild(gradRow);
    this.dialog.appendChild(gradSection);

    // Footer with Action Buttons
    const footer = document.createElement("div");
    footer.style.display = "flex";
    footer.style.justifyContent = "space-between";
    footer.style.alignItems = "center";

    const applyAllBtn = document.createElement("button");
    applyAllBtn.className = "btn-apply-all-bg";
    applyAllBtn.textContent = "Apply to All";
    applyAllBtn.style.padding = "6px 14px";
    applyAllBtn.style.fontSize = "12px";
    applyAllBtn.style.border = "1px solid #cbd5e1";
    applyAllBtn.style.borderRadius = "4px";
    applyAllBtn.style.background = "#f1f5f9";
    applyAllBtn.style.cursor = "pointer";
    applyAllBtn.onclick = () => {
      if (this.onApplyCallback) this.onApplyCallback(this.selectedBackground, true);
      this.close();
    };
    footer.appendChild(applyAllBtn);

    const rightBtns = document.createElement("div");
    rightBtns.style.display = "flex";
    rightBtns.style.gap = "8px";

    const cancelBtn = document.createElement("button");
    cancelBtn.textContent = "Cancel";
    cancelBtn.style.padding = "6px 12px";
    cancelBtn.style.fontSize = "12px";
    cancelBtn.style.border = "1px solid #cbd5e1";
    cancelBtn.style.borderRadius = "4px";
    cancelBtn.style.background = "#f1f5f9";
    cancelBtn.style.cursor = "pointer";
    cancelBtn.onclick = () => this.close();
    rightBtns.appendChild(cancelBtn);

    const applyBtn = document.createElement("button");
    applyBtn.className = "btn-apply-bg";
    applyBtn.textContent = "Apply";
    applyBtn.style.padding = "6px 14px";
    applyBtn.style.fontSize = "12px";
    applyBtn.style.fontWeight = "600";
    applyBtn.style.border = "none";
    applyBtn.style.borderRadius = "4px";
    applyBtn.style.background = "#c43e1c";
    applyBtn.style.color = "#ffffff";
    applyBtn.style.cursor = "pointer";
    applyBtn.onclick = () => {
      if (this.onApplyCallback) this.onApplyCallback(this.selectedBackground, false);
      this.close();
    };
    rightBtns.appendChild(applyBtn);

    footer.appendChild(rightBtns);
    this.dialog.appendChild(footer);
  }

  private highlightSelected(target: HTMLElement) {
    this.dialog.querySelectorAll(".bg-swatch-solid, .bg-swatch-gradient").forEach((el) => {
      (el as HTMLElement).style.borderColor = "#cbd5e1";
      (el as HTMLElement).style.outline = "none";
    });
    target.style.borderColor = "#c43e1c";
    target.style.outline = "2px solid #c43e1c";
  }
}
