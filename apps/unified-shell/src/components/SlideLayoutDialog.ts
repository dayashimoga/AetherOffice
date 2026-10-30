import { SlideLayoutType } from "../state/slidesState";

export interface SlideLayoutOption {
  type: SlideLayoutType;
  title: string;
  description: string;
  iconSvg: string;
}

export class SlideLayoutDialog {
  private overlay: HTMLElement;
  private dialog: HTMLElement;
  private onSelectLayoutCallback?: (layout: SlideLayoutType) => void;

  private layouts: SlideLayoutOption[] = [
    {
      type: "title",
      title: "Title Slide",
      description: "Opening slide with title and subtitle",
      iconSvg: `
        <svg width="64" height="40" viewBox="0 0 64 40" fill="none" xmlns="http://www.w3.org/2000/svg">
          <rect width="64" height="40" rx="3" fill="#ffffff" stroke="#cbd5e1" stroke-width="1.5"/>
          <rect x="12" y="12" width="40" height="7" rx="1.5" fill="#c43e1c"/>
          <rect x="18" y="23" width="28" height="4" rx="1" fill="#94a3b8"/>
        </svg>
      `,
    },
    {
      type: "titleAndContent",
      title: "Title and Content",
      description: "Standard slide with title and bulleted content",
      iconSvg: `
        <svg width="64" height="40" viewBox="0 0 64 40" fill="none" xmlns="http://www.w3.org/2000/svg">
          <rect width="64" height="40" rx="3" fill="#ffffff" stroke="#cbd5e1" stroke-width="1.5"/>
          <rect x="8" y="6" width="48" height="5" rx="1" fill="#c43e1c"/>
          <rect x="8" y="15" width="48" height="19" rx="1.5" fill="#f1f5f9" stroke="#e2e8f0"/>
          <circle cx="12" cy="19" r="1.5" fill="#64748b"/>
          <rect x="16" y="18" width="34" height="2" rx="0.5" fill="#64748b"/>
          <circle cx="12" cy="25" r="1.5" fill="#64748b"/>
          <rect x="16" y="24" width="30" height="2" rx="0.5" fill="#64748b"/>
          <circle cx="12" cy="30" r="1.5" fill="#64748b"/>
          <rect x="16" y="29" width="26" height="2" rx="0.5" fill="#64748b"/>
        </svg>
      `,
    },
    {
      type: "sectionHeader",
      title: "Section Header",
      description: "Category divider with header and text",
      iconSvg: `
        <svg width="64" height="40" viewBox="0 0 64 40" fill="none" xmlns="http://www.w3.org/2000/svg">
          <rect width="64" height="40" rx="3" fill="#ffffff" stroke="#cbd5e1" stroke-width="1.5"/>
          <rect x="10" y="14" width="44" height="6" rx="1" fill="#c43e1c"/>
          <rect x="16" y="23" width="32" height="3" rx="0.5" fill="#94a3b8"/>
        </svg>
      `,
    },
    {
      type: "twoContent",
      title: "Two Content",
      description: "Side-by-side comparison columns",
      iconSvg: `
        <svg width="64" height="40" viewBox="0 0 64 40" fill="none" xmlns="http://www.w3.org/2000/svg">
          <rect width="64" height="40" rx="3" fill="#ffffff" stroke="#cbd5e1" stroke-width="1.5"/>
          <rect x="8" y="6" width="48" height="5" rx="1" fill="#c43e1c"/>
          <rect x="8" y="15" width="22" height="19" rx="1.5" fill="#f1f5f9" stroke="#e2e8f0"/>
          <rect x="34" y="15" width="22" height="19" rx="1.5" fill="#f1f5f9" stroke="#e2e8f0"/>
          <rect x="11" y="18" width="16" height="2" rx="0.5" fill="#64748b"/>
          <rect x="11" y="23" width="13" height="2" rx="0.5" fill="#64748b"/>
          <rect x="37" y="18" width="16" height="2" rx="0.5" fill="#64748b"/>
          <rect x="37" y="23" width="13" height="2" rx="0.5" fill="#64748b"/>
        </svg>
      `,
    },
    {
      type: "titleOnly",
      title: "Title Only",
      description: "Header placeholder with open canvas area",
      iconSvg: `
        <svg width="64" height="40" viewBox="0 0 64 40" fill="none" xmlns="http://www.w3.org/2000/svg">
          <rect width="64" height="40" rx="3" fill="#ffffff" stroke="#cbd5e1" stroke-width="1.5"/>
          <rect x="8" y="6" width="48" height="5" rx="1" fill="#c43e1c"/>
        </svg>
      `,
    },
    {
      type: "blank",
      title: "Blank",
      description: "Completely empty canvas with no predefined shapes",
      iconSvg: `
        <svg width="64" height="40" viewBox="0 0 64 40" fill="none" xmlns="http://www.w3.org/2000/svg">
          <rect width="64" height="40" rx="3" fill="#ffffff" stroke="#cbd5e1" stroke-width="1.5"/>
          <rect x="6" y="6" width="52" height="28" rx="1" fill="#f8fafc" stroke="#e2e8f0" stroke-dasharray="2 2"/>
        </svg>
      `,
    },
  ];

  constructor() {
    this.overlay = document.createElement("div");
    this.overlay.className = "dialog-overlay slide-layout-dialog-overlay";
    this.overlay.style.position = "fixed";
    this.overlay.style.inset = "0";
    this.overlay.style.backgroundColor = "rgba(0, 0, 0, 0.45)";
    this.overlay.style.display = "none";
    this.overlay.style.justifyContent = "center";
    this.overlay.style.alignItems = "center";
    this.overlay.style.zIndex = "1000";

    this.dialog = document.createElement("div");
    this.dialog.className = "slide-layout-dialog";
    this.dialog.style.width = "480px";
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

  public open(onSelect?: (layout: SlideLayoutType) => void) {
    if (onSelect) {
      this.onSelectLayoutCallback = onSelect;
    }
    this.overlay.style.display = "flex";
  }

  public close() {
    this.overlay.style.display = "none";
  }

  public isOpen(): boolean {
    return this.overlay.style.display === "flex";
  }

  public getLayoutCards(): HTMLElement[] {
    return Array.from(this.dialog.querySelectorAll(".slide-layout-card"));
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
    titleEl.textContent = "Apply Slide Layout";
    titleEl.style.margin = "0";
    titleEl.style.fontSize = "16px";
    titleEl.style.fontWeight = "600";
    titleEl.style.color = "#1e293b";
    header.appendChild(titleEl);

    const closeBtn = document.createElement("button");
    closeBtn.className = "dialog-close-btn";
    closeBtn.textContent = "✕";
    closeBtn.style.border = "none";
    closeBtn.style.background = "transparent";
    closeBtn.style.fontSize = "16px";
    closeBtn.style.cursor = "pointer";
    closeBtn.style.color = "#64748b";
    closeBtn.onclick = () => this.close();
    header.appendChild(closeBtn);

    this.dialog.appendChild(header);

    // Layout Cards Grid (2x2)
    const grid = document.createElement("div");
    grid.className = "slide-layout-grid";
    grid.style.display = "grid";
    grid.style.gridTemplateColumns = "repeat(2, 1fr)";
    grid.style.gap = "14px";
    grid.style.marginBottom = "16px";

    this.layouts.forEach((opt) => {
      const card = document.createElement("div");
      card.className = "slide-layout-card";
      card.setAttribute("data-layout", opt.type);
      card.style.display = "flex";
      card.style.flexDirection = "column";
      card.style.alignItems = "center";
      card.style.padding = "12px 10px";
      card.style.border = "1.5px solid #e2e8f0";
      card.style.borderRadius = "6px";
      card.style.cursor = "pointer";
      card.style.transition = "all 0.15s ease";
      card.style.backgroundColor = "#f8fafc";

      card.onmouseenter = () => {
        card.style.borderColor = "#c43e1c";
        card.style.backgroundColor = "#fff5f2";
      };
      card.onmouseleave = () => {
        card.style.borderColor = "#e2e8f0";
        card.style.backgroundColor = "#f8fafc";
      };

      card.onclick = () => {
        if (this.onSelectLayoutCallback) {
          this.onSelectLayoutCallback(opt.type);
        }
        this.close();
      };

      const iconWrap = document.createElement("div");
      iconWrap.innerHTML = opt.iconSvg;
      card.appendChild(iconWrap);

      const label = document.createElement("div");
      label.textContent = opt.title;
      label.style.fontWeight = "600";
      label.style.fontSize = "13px";
      label.style.color = "#1e293b";
      label.style.marginTop = "8px";
      card.appendChild(label);

      const desc = document.createElement("div");
      desc.textContent = opt.description;
      desc.style.fontSize = "11px";
      desc.style.color = "#64748b";
      desc.style.textAlign = "center";
      desc.style.marginTop = "4px";
      card.appendChild(desc);

      grid.appendChild(card);
    });

    this.dialog.appendChild(grid);

    // Footer with Cancel button
    const footer = document.createElement("div");
    footer.style.display = "flex";
    footer.style.justifyContent = "flex-end";

    const cancelBtn = document.createElement("button");
    cancelBtn.className = "btn-dialog-cancel";
    cancelBtn.textContent = "Cancel";
    cancelBtn.style.padding = "6px 14px";
    cancelBtn.style.fontSize = "13px";
    cancelBtn.style.border = "1px solid #cbd5e1";
    cancelBtn.style.borderRadius = "4px";
    cancelBtn.style.background = "#f1f5f9";
    cancelBtn.style.cursor = "pointer";
    cancelBtn.onclick = () => this.close();
    footer.appendChild(cancelBtn);

    this.dialog.appendChild(footer);
  }
}

