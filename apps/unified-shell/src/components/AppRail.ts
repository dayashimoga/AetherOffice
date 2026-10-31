export type RailAppMode = "home" | "word" | "excel" | "powerpoint" | "pdf" | "notes";

export interface AppRailCallbacks {
  onSelectApp: (mode: RailAppMode) => void;
  onNewAppDocument?: (mode: RailAppMode) => void;
}

export class AppRail {
  private container: HTMLElement;
  private activeApp: RailAppMode = "word";
  private counts: Record<RailAppMode, number> = {
    home: 0,
    word: 0,
    excel: 0,
    powerpoint: 0,
    pdf: 0,
    notes: 0,
  };
  private callbacks: AppRailCallbacks = { onSelectApp: () => {} };

  constructor(containerIdOrElement: string | HTMLElement, callbacks?: AppRailCallbacks) {
    if (typeof containerIdOrElement === "string") {
      const el = document.getElementById(containerIdOrElement);
      if (!el) throw new Error(`AppRail container #${containerIdOrElement} not found`);
      this.container = el;
    } else {
      this.container = containerIdOrElement;
    }
    if (callbacks) {
      this.callbacks = callbacks;
    }
    this.render();
  }

  public setActiveApp(mode: RailAppMode) {
    this.activeApp = mode;
    this.render();
  }

  public getActiveApp(): RailAppMode {
    return this.activeApp;
  }

  public setCounts(counts: Partial<Record<RailAppMode, number>>) {
    Object.assign(this.counts, counts);
    this.render();
  }

  public getCounts(): Record<RailAppMode, number> {
    return { ...this.counts };
  }

  public setCallbacks(callbacks: AppRailCallbacks) {
    this.callbacks = callbacks;
  }

  public render() {
    this.container.innerHTML = "";
    this.container.className = "app-rail";
    this.container.setAttribute("data-testid", "app-rail");

    const apps: { id: RailAppMode; label: string; icon: string; iconClass: string; color: string }[] = [
      { id: "home", label: "Home Hub", icon: "🏠", iconClass: "rail-icon-home", color: "#f59e0b" },
      { id: "word", label: "Word (.docx)", icon: "W", iconClass: "rail-icon-word", color: "#3b82f6" },
      { id: "excel", label: "Excel (.xlsx)", icon: "X", iconClass: "rail-icon-excel", color: "#22c55e" },
      { id: "powerpoint", label: "PowerPoint (.pptx)", icon: "P", iconClass: "rail-icon-ppt", color: "#f97316" },
      { id: "pdf", label: "PDF Studio (.pdf)", icon: "PDF", iconClass: "rail-icon-pdf", color: "#ef4444" },
      { id: "notes", label: "Notepad / Notes (.txt/.md)", icon: "N", iconClass: "rail-icon-notes", color: "#a855f7" },
    ];

    apps.forEach((app) => {
      const count = this.counts[app.id] || 0;
      const btn = document.createElement("button");
      btn.className = `rail-btn ${this.activeApp === app.id ? "active" : ""}`;
      btn.dataset.railApp = app.id;
      btn.dataset.testid = `rail-btn-${app.id}`;
      btn.title = count > 0 ? `${app.label} (${count} open)` : app.label;
      btn.setAttribute("aria-label", btn.title);
      btn.style.position = "relative";

      const iconSpan = document.createElement("span");
      iconSpan.className = `rail-icon ${app.iconClass}`;
      iconSpan.textContent = app.icon;
      btn.appendChild(iconSpan);

      if (count > 0 && app.id !== "home") {
        const badge = document.createElement("span");
        badge.className = "rail-badge";
        badge.dataset.testid = `rail-badge-${app.id}`;
        badge.textContent = String(count);
        btn.appendChild(badge);
      }

      btn.onclick = () => {
        const prevApp = this.activeApp;
        this.setActiveApp(app.id);
        this.callbacks.onSelectApp(app.id);
        if (prevApp === app.id && count > 0 && this.callbacks.onNewAppDocument) {
          // If already viewing and user clicks again, allow spawning new document
          this.callbacks.onNewAppDocument(app.id);
        }
      };

      this.container.appendChild(btn);
    });
  }
}
