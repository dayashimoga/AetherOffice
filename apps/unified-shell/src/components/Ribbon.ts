import { DocumentType, RibbonLayoutMode } from "../ipc";
import { AppRibbonComponent } from "./ribbon/types";
import { WriterRibbon } from "./ribbon/WriterRibbon";
import { CalcRibbon } from "./ribbon/CalcRibbon";
import { SlidesRibbon } from "./ribbon/SlidesRibbon";
import { PdfRibbon } from "./ribbon/PdfRibbon";
import { NotesRibbon } from "./ribbon/NotesRibbon";
import { ContextualRibbonRouter } from "./ribbon/ContextualRibbonRouter";
import { pluginHost, PluginHost } from "../plugins/pluginHost";

export type RibbonTabId = string;

export class Ribbon {
  private container: HTMLElement;
  private docType: DocumentType = "Word";
  private layoutMode: RibbonLayoutMode = "classic";
  private activeRibbonTabs: Record<DocumentType, string> = {
    Word: "Home",
    Excel: "Home",
    PowerPoint: "Home",
    PDF: "Home",
    Notes: "Home",
  };
  private keytipsActive = false;
  private backstageActive = false;
  private onActionCallback?: (action: string, payload?: unknown) => void;

  private writerRibbon = new WriterRibbon();
  private calcRibbon = new CalcRibbon();
  private slidesRibbon = new SlidesRibbon();
  private pdfRibbon = new PdfRibbon();
  private notesRibbon = new NotesRibbon();
  private router: ContextualRibbonRouter;

  constructor(containerId: string, host: PluginHost = pluginHost) {
    const el = document.getElementById(containerId);
    if (!el) throw new Error(`Ribbon container #${containerId} not found`);
    this.container = el;
    this.router = new ContextualRibbonRouter(host);

    host.onLifecycle(() => {
      this.render();
    });

    this.render();
  }

  public getRouter(): ContextualRibbonRouter {
    return this.router;
  }

  public getWriterRibbon(): WriterRibbon {
    return this.writerRibbon;
  }

  public getCalcRibbon(): CalcRibbon {
    return this.calcRibbon;
  }

  public getSlidesRibbon(): SlidesRibbon {
    return this.slidesRibbon;
  }

  public getPdfRibbon(): PdfRibbon {
    return this.pdfRibbon;
  }

  public getNotesRibbon(): NotesRibbon {
    return this.notesRibbon;
  }

  public getActiveAppRibbon(): AppRibbonComponent {
    switch (this.docType) {
      case "Word":
        return this.writerRibbon;
      case "Excel":
        return this.calcRibbon;
      case "PowerPoint":
        return this.slidesRibbon;
      case "PDF":
        return this.pdfRibbon;
      case "Notes":
        return this.notesRibbon;
      default:
        return this.writerRibbon;
    }
  }

  public getDocAccentColor(): string {
    switch (this.docType) {
      case "Word":
        return "#2b579a";
      case "Excel":
        return "#107c41";
      case "PowerPoint":
        return "#c43e1c";
      case "PDF":
        return "#d9381e";
      case "Notes":
      default:
        return "#7c3aed";
    }
  }

  public setDocumentType(docType: DocumentType) {
    this.docType = docType;
    const validTabs = this.getVisibleTabs();
    if (!validTabs.includes(this.activeRibbonTabs[docType])) {
      this.activeRibbonTabs[docType] = "Home";
    }
    this.container.style.setProperty("--primary-accent", this.getDocAccentColor());
    this.render();
  }

  public getLayoutMode(): RibbonLayoutMode {
    return this.layoutMode;
  }

  public setLayoutMode(mode: RibbonLayoutMode) {
    this.layoutMode = mode;
    this.render();
  }

  public setOnAction(callback: (action: string, payload?: unknown) => void) {
    this.onActionCallback = callback;
  }

  public getActiveTab(): string {
    return this.activeRibbonTabs[this.docType] || "Home";
  }

  public setActiveTab(tab: RibbonTabId) {
    const validTabs = this.getVisibleTabs();
    if (validTabs.includes(tab)) {
      this.activeRibbonTabs[this.docType] = tab;
      this.render();
    }
  }

  public selectTab(tab: RibbonTabId) {
    this.setActiveTab(tab);
  }

  public toggleKeytips(active?: boolean): boolean {
    this.keytipsActive = active !== undefined ? active : !this.keytipsActive;
    if (typeof document !== "undefined") {
      if (this.keytipsActive) {
        document.body.classList.add("alt-keytips-active");
      } else {
        document.body.classList.remove("alt-keytips-active");
      }
    }
    this.render();
    return this.keytipsActive;
  }

  public setBackstageActive(isOpen: boolean) {
    this.backstageActive = isOpen;
    this.render();
  }

  public isBackstageActive(): boolean {
    return this.backstageActive;
  }

  public getVisibleTabs(): string[] {
    return this.router.getVisibleTabs(this.docType, this.getActiveAppRibbon().getTabs());
  }

  private render() {
    this.container.innerHTML = "";
    this.container.style.setProperty("--primary-accent", this.getDocAccentColor());
    this.container.setAttribute("data-ribbon-mode", this.layoutMode);

    if (this.layoutMode === "compact") {
      this.container.classList.add("ribbon-compact");
      this.container.classList.remove("ribbon-collapsed");
    } else if (this.layoutMode === "collapsed") {
      this.container.classList.add("ribbon-collapsed");
      this.container.classList.remove("ribbon-compact");
    } else {
      this.container.classList.remove("ribbon-compact", "ribbon-collapsed");
    }

    // 1. Ribbon Tabs Header
    const header = document.createElement("div");
    header.className = `ribbon-tabs-header ${this.keytipsActive ? "keytips-active" : ""}`;

    // 1.1 Dedicated File Tab / Backstage Trigger
    // STRICT CONTRACT: File is NEVER an active tab, never underlined. It is a Backstage overlay trigger.
    const fileBtn = document.createElement("button");
    fileBtn.className = `ribbon-tab-btn ribbon-file-tab ${this.backstageActive ? "backstage-active active" : ""}`;
    fileBtn.setAttribute("data-testid", "ribbon-tab-file");
    fileBtn.textContent = "File";
    fileBtn.style.backgroundColor = this.getDocAccentColor();
    fileBtn.onclick = () => {
      if (this.onActionCallback) {
        this.onActionCallback("ToggleBackstage");
      }
    };
    if (this.keytipsActive) {
      const badge = document.createElement("span");
      badge.className = "ribbon-keytip-badge";
      badge.textContent = "F";
      fileBtn.appendChild(badge);
    }
    header.appendChild(fileBtn);

    // 1.2 Contextual App Tabs (Home, Insert, Layout, etc.)
    const activeTab = this.getActiveTab();
    const appRibbon = this.getActiveAppRibbon();

    this.getVisibleTabs().forEach((tab) => {
      const tabBtn = document.createElement("button");
      const isActive = tab === activeTab;
      tabBtn.className = `ribbon-tab-btn ${isActive ? "active" : ""}`;
      tabBtn.setAttribute(
        "data-testid",
        `ribbon-tab-${tab.toLowerCase().replace(/\s+/g, "-")}`
      );
      tabBtn.textContent = tab;
      tabBtn.onclick = () => this.setActiveTab(tab);

      if (this.keytipsActive) {
        const badge = document.createElement("span");
        badge.className = "ribbon-keytip-badge";
        badge.textContent = this.router.getKeytip(this.docType, tab, appRibbon);
        tabBtn.appendChild(badge);
      }

      header.appendChild(tabBtn);
    });

    // 1.3 Ribbon Layout Mode Switcher Toggle Button
    const layoutToggleBtn = document.createElement("button");
    layoutToggleBtn.className = "ribbon-layout-toggle-btn";
    layoutToggleBtn.setAttribute("data-testid", "ribbon-layout-toggle-btn");
    layoutToggleBtn.setAttribute("aria-label", "Toggle Ribbon Layout Mode");
    layoutToggleBtn.title = this.layoutMode === "compact"
      ? "Switch to Classic Ribbon (Multi-row)"
      : this.layoutMode === "collapsed"
      ? "Expand Ribbon"
      : "Switch to Compact Ribbon (Single-row)";
    layoutToggleBtn.innerHTML = this.layoutMode === "compact"
      ? `<span aria-hidden="true">⊞</span> Classic`
      : this.layoutMode === "collapsed"
      ? `<span aria-hidden="true">▼</span> Expand`
      : `<span aria-hidden="true">⊟</span> Compact`;
    layoutToggleBtn.onclick = () => {
      const nextMode: RibbonLayoutMode = this.layoutMode === "classic"
        ? "compact"
        : this.layoutMode === "compact"
        ? "collapsed"
        : "classic";
      this.setLayoutMode(nextMode);
      if (this.onActionCallback) {
        this.onActionCallback("RibbonLayoutModeChanged", nextMode);
      }
    };
    header.appendChild(layoutToggleBtn);

    this.container.appendChild(header);

    // 2. Ribbon Tools Body (rendered only if not collapsed)
    if (this.layoutMode !== "collapsed") {
      const body = document.createElement("div");
      body.className = `ribbon-tools-body ribbon-content ${this.layoutMode === "compact" ? "ribbon-compact" : ""}`;
      body.id = "ribbon-tools-body";

      this.router.renderTab(
        this.docType,
        activeTab,
        body,
        appRibbon,
        (action, payload) => this.emit(action, payload)
      );

      this.container.appendChild(body);
    }
  }

  private emit(action: string, payload?: unknown) {
    if (this.onActionCallback) {
      this.onActionCallback(action, payload);
    }
  }
}
