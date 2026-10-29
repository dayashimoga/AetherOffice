export interface FindReplaceMatch {
  pageIndex: number;
  paragraphIndex: number;
  runIndex: number;
  offset: number;
  length: number;
  text: string;
}

export interface FindReplaceCallbacks {
  onFind: (query: string, matchCase: boolean) => FindReplaceMatch[];
  onSelectMatch: (match: FindReplaceMatch) => void;
  onReplace: (match: FindReplaceMatch, replacement: string) => void;
  onReplaceAll: (query: string, replacement: string, matchCase: boolean) => number;
  onClose: () => void;
}

export class FindReplaceDialog {
  private element: HTMLElement;
  private isVisible = false;
  private mode: "find" | "replace" = "find";
  private matches: FindReplaceMatch[] = [];
  private currentMatchIndex = -1;
  private callbacks?: FindReplaceCallbacks;

  constructor() {
    this.element = document.createElement("div");
    this.element.className = "find-replace-dialog";
    this.element.id = "find-replace-dialog";
    this.applyStyles();
    document.body.appendChild(this.element);
    this.render();
  }

  public setCallbacks(cb: FindReplaceCallbacks) {
    this.callbacks = cb;
  }

  public show(mode: "find" | "replace" = "find", initialQuery: string = "") {
    this.mode = mode;
    this.isVisible = true;
    this.element.style.display = "flex";
    this.render();

    const input = this.element.querySelector<HTMLInputElement>(".find-query-input");
    if (input) {
      if (initialQuery) input.value = initialQuery;
      input.focus();
      input.select();
      if (input.value) this.executeFind();
    }
  }

  public hide() {
    this.isVisible = false;
    this.element.style.display = "none";
    this.matches = [];
    this.currentMatchIndex = -1;
    if (this.callbacks?.onClose) {
      this.callbacks.onClose();
    }
  }

  public isOpen(): boolean {
    return this.isVisible;
  }

  public getMatchesCount(): number {
    return this.matches.length;
  }

  public getCurrentMatchIndex(): number {
    return this.currentMatchIndex;
  }

  private applyStyles() {
    this.element.style.display = "none";
    this.element.style.position = "fixed";
    this.element.style.top = "120px";
    this.element.style.right = "40px";
    this.element.style.zIndex = "1000";
    this.element.style.backgroundColor = "#ffffff";
    this.element.style.border = "1px solid #d1d5db";
    this.element.style.borderRadius = "6px";
    this.element.style.boxShadow = "0 8px 24px rgba(0, 0, 0, 0.15)";
    this.element.style.padding = "16px";
    this.element.style.width = "380px";
    this.element.style.flexDirection = "column";
    this.element.style.gap = "10px";
    this.element.style.fontFamily = "Segoe UI, -apple-system, sans-serif";
    this.element.style.fontSize = "13px";
    this.element.style.color = "#1f2937";
  }

  private executeFind() {
    if (!this.callbacks) return;
    const findInput = this.element.querySelector<HTMLInputElement>(".find-query-input");
    const caseCheckbox = this.element.querySelector<HTMLInputElement>(".find-case-checkbox");
    const query = findInput?.value || "";
    const matchCase = caseCheckbox?.checked || false;

    if (!query) {
      this.matches = [];
      this.currentMatchIndex = -1;
      this.updateMatchDisplay();
      return;
    }

    this.matches = this.callbacks.onFind(query, matchCase);
    if (this.matches.length > 0) {
      this.currentMatchIndex = 0;
      this.callbacks.onSelectMatch(this.matches[0]);
    } else {
      this.currentMatchIndex = -1;
    }
    this.updateMatchDisplay();
  }

  private nextMatch() {
    if (this.matches.length === 0 || !this.callbacks) return;
    this.currentMatchIndex = (this.currentMatchIndex + 1) % this.matches.length;
    this.callbacks.onSelectMatch(this.matches[this.currentMatchIndex]);
    this.updateMatchDisplay();
  }

  private prevMatch() {
    if (this.matches.length === 0 || !this.callbacks) return;
    this.currentMatchIndex = (this.currentMatchIndex - 1 + this.matches.length) % this.matches.length;
    this.callbacks.onSelectMatch(this.matches[this.currentMatchIndex]);
    this.updateMatchDisplay();
  }

  private replaceCurrent() {
    if (this.matches.length === 0 || this.currentMatchIndex < 0 || !this.callbacks) return;
    const replaceInput = this.element.querySelector<HTMLInputElement>(".replace-text-input");
    const replacement = replaceInput?.value || "";
    const match = this.matches[this.currentMatchIndex];
    this.callbacks.onReplace(match, replacement);
    this.executeFind();
  }

  private replaceAll() {
    if (!this.callbacks) return;
    const findInput = this.element.querySelector<HTMLInputElement>(".find-query-input");
    const replaceInput = this.element.querySelector<HTMLInputElement>(".replace-text-input");
    const caseCheckbox = this.element.querySelector<HTMLInputElement>(".find-case-checkbox");
    const query = findInput?.value || "";
    const replacement = replaceInput?.value || "";
    const matchCase = caseCheckbox?.checked || false;

    if (!query) return;
    const count = this.callbacks.onReplaceAll(query, replacement, matchCase);
    const countDisplay = this.element.querySelector(".match-count-display");
    if (countDisplay) {
      countDisplay.textContent = `Replaced ${count} occurrence(s).`;
    }
    this.matches = [];
    this.currentMatchIndex = -1;
  }

  private updateMatchDisplay() {
    const countDisplay = this.element.querySelector(".match-count-display");
    if (!countDisplay) return;
    if (this.matches.length === 0) {
      countDisplay.textContent = "No matches";
    } else {
      countDisplay.textContent = `${this.currentMatchIndex + 1} of ${this.matches.length}`;
    }
  }

  private render() {
    this.element.innerHTML = `
      <div style="display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid #e5e7eb; padding-bottom: 8px;">
        <div style="display: flex; gap: 8px;">
          <button class="find-tab-btn ${this.mode === "find" ? "active" : ""}" style="background: none; border: none; font-weight: 600; cursor: pointer; color: ${this.mode === "find" ? "#2563eb" : "#6b7280"};">Find</button>
          <button class="replace-tab-btn ${this.mode === "replace" ? "active" : ""}" style="background: none; border: none; font-weight: 600; cursor: pointer; color: ${this.mode === "replace" ? "#2563eb" : "#6b7280"};">Replace</button>
        </div>
        <button class="btn-close-find" title="Close" style="background: none; border: none; font-size: 16px; cursor: pointer; color: #9ca3af;">✕</button>
      </div>

      <div style="display: flex; flex-direction: column; gap: 8px;">
        <div style="display: flex; align-items: center; gap: 6px;">
          <input type="text" class="find-query-input" placeholder="Find what..." style="flex: 1; padding: 6px 10px; border: 1px solid #d1d5db; borderRadius: 4px; font-size: 13px;" />
          <span class="match-count-display" style="font-size: 11px; color: #6b7280; min-width: 60px; text-align: right;"></span>
        </div>

        <div class="replace-row" style="display: ${this.mode === "replace" ? "flex" : "none"}; align-items: center; gap: 6px;">
          <input type="text" class="replace-text-input" placeholder="Replace with..." style="flex: 1; padding: 6px 10px; border: 1px solid #d1d5db; borderRadius: 4px; font-size: 13px;" />
        </div>

        <div style="display: flex; align-items: center; gap: 6px; font-size: 12px; color: #4b5563;">
          <label style="display: flex; align-items: center; gap: 4px; cursor: pointer;">
            <input type="checkbox" class="find-case-checkbox" /> Match case
          </label>
        </div>

        <div style="display: flex; justify-content: flex-end; gap: 6px; margin-top: 4px;">
          <button class="btn-find-prev" style="padding: 4px 10px; border: 1px solid #d1d5db; border-radius: 4px; background: #ffffff; cursor: pointer;">▲ Prev</button>
          <button class="btn-find-next" style="padding: 4px 10px; border: 1px solid #d1d5db; border-radius: 4px; background: #ffffff; cursor: pointer;">▼ Next</button>
          <button class="btn-replace" style="display: ${this.mode === "replace" ? "inline-block" : "none"}; padding: 4px 10px; border: 1px solid #d1d5db; border-radius: 4px; background: #ffffff; cursor: pointer;">Replace</button>
          <button class="btn-replace-all" style="display: ${this.mode === "replace" ? "inline-block" : "none"}; padding: 4px 10px; border: 1px solid #2563eb; border-radius: 4px; background: #2563eb; color: #ffffff; cursor: pointer;">Replace All</button>
        </div>
      </div>
    `;

    // Event attachments
    const findTab = this.element.querySelector<HTMLButtonElement>(".find-tab-btn")!;
    const replaceTab = this.element.querySelector<HTMLButtonElement>(".replace-tab-btn")!;
    const closeBtn = this.element.querySelector<HTMLButtonElement>(".btn-close-find")!;
    const findInput = this.element.querySelector<HTMLInputElement>(".find-query-input")!;
    const prevBtn = this.element.querySelector<HTMLButtonElement>(".btn-find-prev")!;
    const nextBtn = this.element.querySelector<HTMLButtonElement>(".btn-find-next")!;
    const replaceBtn = this.element.querySelector<HTMLButtonElement>(".btn-replace")!;
    const replaceAllBtn = this.element.querySelector<HTMLButtonElement>(".btn-replace-all")!;
    const caseCheckbox = this.element.querySelector<HTMLInputElement>(".find-case-checkbox")!;

    findTab.onclick = () => {
      this.mode = "find";
      this.render();
    };

    replaceTab.onclick = () => {
      this.mode = "replace";
      this.render();
    };

    closeBtn.onclick = () => this.hide();

    findInput.oninput = () => this.executeFind();
    findInput.onkeydown = (e) => {
      if (e.key === "Enter") {
        e.preventDefault();
        if (e.shiftKey) this.prevMatch();
        else this.nextMatch();
      } else if (e.key === "Escape") {
        this.hide();
      }
    };

    caseCheckbox.onchange = () => this.executeFind();
    prevBtn.onclick = () => this.prevMatch();
    nextBtn.onclick = () => this.nextMatch();
    replaceBtn.onclick = () => this.replaceCurrent();
    replaceAllBtn.onclick = () => this.replaceAll();
  }
}

