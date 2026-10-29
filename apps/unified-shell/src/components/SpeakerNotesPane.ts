import { SlidesState, slidesState } from "../state/slidesState";

export class SpeakerNotesPane {
  private container: HTMLElement;
  private rootEl: HTMLElement;
  private headerEl: HTMLElement;
  private chevronEl: HTMLElement;
  private bodyEl: HTMLElement;
  private textarea: HTMLTextAreaElement;
  private state: SlidesState;
  private collapsed = false;
  private onDirtyCallback?: () => void;

  constructor(container: HTMLElement, state: SlidesState = slidesState) {
    this.container = container;
    this.state = state;

    this.rootEl = document.createElement("div");
    this.rootEl.className = "speaker-notes-pane";
    this.rootEl.style.width = "960px";
    this.rootEl.style.margin = "0 auto 24px auto";
    this.rootEl.style.backgroundColor = "#ffffff";
    this.rootEl.style.border = "1px solid #cbd5e1";
    this.rootEl.style.borderRadius = "6px";
    this.rootEl.style.boxShadow = "0 4px 12px rgba(0, 0, 0, 0.08)";
    this.rootEl.style.overflow = "hidden";
    this.rootEl.style.fontFamily = "Segoe UI, sans-serif";
    this.rootEl.style.transition = "all 0.2s ease";

    // Header bar
    this.headerEl = document.createElement("div");
    this.headerEl.className = "speaker-notes-header";
    this.headerEl.style.display = "flex";
    this.headerEl.style.justifyContent = "space-between";
    this.headerEl.style.alignItems = "center";
    this.headerEl.style.padding = "6px 12px";
    this.headerEl.style.backgroundColor = "#f8fafc";
    this.headerEl.style.borderBottom = "1px solid #e2e8f0";
    this.headerEl.style.cursor = "pointer";
    this.headerEl.style.userSelect = "none";

    const titleWrap = document.createElement("div");
    titleWrap.style.display = "flex";
    titleWrap.style.alignItems = "center";
    titleWrap.style.gap = "6px";
    titleWrap.style.fontSize = "12px";
    titleWrap.style.fontWeight = "600";
    titleWrap.style.color = "#334155";
    titleWrap.innerHTML = `<span>📝</span><span>Speaker Notes</span>`;
    this.headerEl.appendChild(titleWrap);

    this.chevronEl = document.createElement("span");
    this.chevronEl.className = "notes-chevron";
    this.chevronEl.textContent = "▼";
    this.chevronEl.style.fontSize = "10px";
    this.chevronEl.style.color = "#64748b";
    this.headerEl.appendChild(this.chevronEl);

    this.headerEl.onclick = () => this.toggleCollapse();
    this.rootEl.appendChild(this.headerEl);

    // Body & Textarea
    this.bodyEl = document.createElement("div");
    this.bodyEl.className = "speaker-notes-body";
    this.bodyEl.style.padding = "8px 12px";

    this.textarea = document.createElement("textarea");
    this.textarea.className = "speaker-notes-textarea";
    this.textarea.placeholder = "Click to add speaker notes for presenter view...";
    this.textarea.style.width = "100%";
    this.textarea.style.height = "70px";
    this.textarea.style.boxSizing = "border-box";
    this.textarea.style.border = "none";
    this.textarea.style.outline = "none";
    this.textarea.style.resize = "vertical";
    this.textarea.style.fontSize = "13px";
    this.textarea.style.fontFamily = "Segoe UI, sans-serif";
    this.textarea.style.color = "#1e293b";
    this.textarea.style.background = "transparent";

    this.textarea.addEventListener("input", () => {
      this.state.setSlideNotes(this.textarea.value);
      if (this.onDirtyCallback) this.onDirtyCallback();
    });

    this.bodyEl.appendChild(this.textarea);
    this.rootEl.appendChild(this.bodyEl);
    this.container.appendChild(this.rootEl);

    this.refresh();
  }

  public setOnDirty(cb: () => void) {
    this.onDirtyCallback = cb;
  }

  public refresh() {
    this.textarea.value = this.state.getSlideNotes();
  }

  public toggleCollapse() {
    this.collapsed = !this.collapsed;
    if (this.collapsed) {
      this.bodyEl.style.display = "none";
      this.chevronEl.textContent = "▲";
    } else {
      this.bodyEl.style.display = "block";
      this.chevronEl.textContent = "▼";
    }
  }

  public isCollapsed(): boolean {
    return this.collapsed;
  }

  public getTextarea(): HTMLTextAreaElement {
    return this.textarea;
  }

  public getRoot(): HTMLElement {
    return this.rootEl;
  }
}
