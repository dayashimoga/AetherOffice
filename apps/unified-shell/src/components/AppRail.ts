export type RailAppMode = "home" | "word" | "excel" | "powerpoint" | "pdf" | "notes";

export interface AppRailCallbacks {
  onSelectApp: (mode: RailAppMode) => void;
}

export class AppRail {
  private container: HTMLElement;
  private activeApp: RailAppMode = "word";
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
      const btn = document.createElement("button");
      btn.className = `rail-btn ${this.activeApp === app.id ? "active" : ""}`;
      btn.dataset.railApp = app.id;
      btn.dataset.testid = `rail-btn-${app.id}`;
      btn.title = app.label;
      btn.setAttribute("aria-label", app.label);

      const iconSpan = document.createElement("span");
      iconSpan.className = `rail-icon ${app.iconClass}`;
      iconSpan.textContent = app.icon;
      btn.appendChild(iconSpan);

      btn.onclick = () => {
        this.setActiveApp(app.id);
        this.callbacks.onSelectApp(app.id);
      };

      this.container.appendChild(btn);
    });
  }
}

