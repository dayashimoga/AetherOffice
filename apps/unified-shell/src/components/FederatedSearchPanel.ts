import {
  IPC,
  FederatedSearchMatch,
  FederatedReplaceSummary,
  SearchOptions,
} from "../ipc";

export interface FederatedSearchCallbacks {
  onSelectMatch?: (match: FederatedSearchMatch) => void;
  onTabSwitchRequested?: (sessionId: string) => void;
  onClose?: () => void;
}

export class FederatedSearchPanel {
  private element: HTMLElement;
  private isVisible = false;
  private mode: "find" | "replace" = "find";
  private matches: FederatedSearchMatch[] = [];
  private selectedMatchId: string | null = null;
  private statusMessage = "";
  private callbacks?: FederatedSearchCallbacks;

  constructor() {
    this.element = document.createElement("div");
    this.element.className = "federated-search-panel";
    this.element.id = "federated-search-panel";
    this.element.setAttribute("role", "dialog");
    this.element.setAttribute("aria-label", "Cross-Document Federated Search & Replace");
    this.applyStyles();
    document.body.appendChild(this.element);
    this.render();
  }

  public setCallbacks(callbacks: FederatedSearchCallbacks): void {
    this.callbacks = callbacks;
  }

  public show(initialQuery = "", mode: "find" | "replace" = "find"): void {
    this.mode = mode;
    this.isVisible = true;
    this.element.style.display = "flex";
    this.render();

    const input = this.element.querySelector<HTMLInputElement>(".federated-query-input");
    if (input) {
      if (initialQuery) {
        input.value = initialQuery;
      }
      input.focus();
      input.select();
      if (input.value.trim().length > 0) {
        void this.executeSearch();
      }
    }
  }

  public hide(): void {
    this.isVisible = false;
    this.element.style.display = "none";
    this.matches = [];
    this.selectedMatchId = null;
    this.statusMessage = "";
    if (this.callbacks?.onClose) {
      this.callbacks.onClose();
    }
  }

  public toggle(initialQuery = "", mode: "find" | "replace" = "find"): void {
    if (this.isVisible) {
      this.hide();
    } else {
      this.show(initialQuery, mode);
    }
  }

  public isOpen(): boolean {
    return this.isVisible;
  }

  public getMatches(): FederatedSearchMatch[] {
    return this.matches;
  }

  public getSelectedMatch(): FederatedSearchMatch | null {
    if (!this.selectedMatchId) return null;
    return this.matches.find((m) => m.match_id === this.selectedMatchId) || null;
  }

  public getOptions(): SearchOptions {
    const caseCb = this.element.querySelector<HTMLInputElement>(".federated-case-checkbox");
    const wordCb = this.element.querySelector<HTMLInputElement>(".federated-word-checkbox");
    const regexCb = this.element.querySelector<HTMLInputElement>(".federated-regex-checkbox");

    return {
      case_sensitive: caseCb?.checked ?? false,
      whole_word: wordCb?.checked ?? false,
      use_regex: regexCb?.checked ?? false,
    };
  }

  public async executeSearch(customStatus?: string): Promise<void> {
    const queryInput = this.element.querySelector<HTMLInputElement>(".federated-query-input");
    const query = queryInput?.value || "";

    if (!query.trim()) {
      this.matches = [];
      this.selectedMatchId = null;
      this.statusMessage = "Please enter a search query.";
      this.render();
      return;
    }

    try {
      this.statusMessage = customStatus || "Searching across workspace tabs...";
      this.render();

      const options = this.getOptions();
      const hits = await IPC.federatedSearch(query, options);
      this.matches = hits;

      if (customStatus) {
        this.statusMessage = customStatus;
        this.selectedMatchId = hits.length > 0 ? hits[0].match_id : null;
      } else if (hits.length === 0) {
        this.statusMessage = `No matches found for "${query}".`;
        this.selectedMatchId = null;
      } else {
        const uniqueDocs = new Set(hits.map((h) => h.session_id)).size;
        this.statusMessage = `Found ${hits.length} match${hits.length === 1 ? "" : "es"} across ${uniqueDocs} document${uniqueDocs === 1 ? "" : "s"}.`;
        this.selectedMatchId = hits[0].match_id;
      }
      this.render();
    } catch (err) {
      this.statusMessage = `Search failed: ${err instanceof Error ? err.message : String(err)}`;
      this.render();
    }
  }

  public async executeReplaceSelected(): Promise<void> {
    const selected = this.getSelectedMatch();
    if (!selected) {
      this.statusMessage = "No match selected to replace.";
      this.render();
      return;
    }

    const queryInput = this.element.querySelector<HTMLInputElement>(".federated-query-input");
    const replaceInput = this.element.querySelector<HTMLInputElement>(".federated-replace-input");
    const query = queryInput?.value || selected.matched_text;
    const replacement = replaceInput?.value || "";

    try {
      const options = this.getOptions();
      const count = await IPC.federatedReplaceMatch(
        selected.session_id,
        query,
        replacement,
        options
      );

      const msg = `Replaced ${count} occurrence in "${selected.document_title}".`;
      await this.executeSearch(msg);
    } catch (err) {
      this.statusMessage = `Replace failed: ${err instanceof Error ? err.message : String(err)}`;
      this.render();
    }
  }

  public async executeReplaceAll(): Promise<void> {
    const queryInput = this.element.querySelector<HTMLInputElement>(".federated-query-input");
    const replaceInput = this.element.querySelector<HTMLInputElement>(".federated-replace-input");
    const query = queryInput?.value || "";
    const replacement = replaceInput?.value || "";

    if (!query.trim()) {
      this.statusMessage = "Enter search term to replace.";
      this.render();
      return;
    }

    try {
      this.statusMessage = "Replacing occurrences across documents...";
      this.render();

      const options = this.getOptions();
      const summary: FederatedReplaceSummary = await IPC.federatedReplaceAll(
        query,
        replacement,
        options
      );

      const msg = `Replaced ${summary.total_replacements} occurrence${summary.total_replacements === 1 ? "" : "s"} across ${summary.documents_affected} document${summary.documents_affected === 1 ? "" : "s"}.`;
      await this.executeSearch(msg);
    } catch (err) {
      this.statusMessage = `Replace all failed: ${err instanceof Error ? err.message : String(err)}`;
      this.render();
    }
  }

  public selectMatch(match: FederatedSearchMatch): void {
    this.selectedMatchId = match.match_id;
    if (this.callbacks?.onTabSwitchRequested) {
      this.callbacks.onTabSwitchRequested(match.session_id);
    }
    if (this.callbacks?.onSelectMatch) {
      this.callbacks.onSelectMatch(match);
    }
    this.render();
  }

  private applyStyles(): void {
    this.element.style.display = "none";
    this.element.style.position = "fixed";
    this.element.style.top = "100px";
    this.element.style.right = "32px";
    this.element.style.width = "480px";
    this.element.style.maxHeight = "calc(100vh - 140px)";
    this.element.style.zIndex = "2500";
    this.element.style.backgroundColor = "var(--bg-panel, #ffffff)";
    this.element.style.color = "var(--text-main, #1f2937)";
    this.element.style.border = "1px solid var(--border-panel, #d1d5db)";
    this.element.style.borderRadius = "8px";
    this.element.style.boxShadow = "0 12px 32px rgba(0, 0, 0, 0.2)";
    this.element.style.flexDirection = "column";
    this.element.style.overflow = "hidden";
    this.element.style.fontFamily = "Segoe UI, -apple-system, sans-serif";
    this.element.style.fontSize = "13px";
  }

  private formatLocation(match: FederatedSearchMatch): string {
    const loc = match.location;
    if (loc.type === "Word") {
      return `Line ${loc.details.line}, Col ${loc.details.char_offset + 1}`;
    }
    if (loc.type === "Excel") {
      return `${loc.details.sheet}: Cell ${loc.details.coord}`;
    }
    if (loc.type === "PowerPoint") {
      return `Slide ${loc.details.slide_index}`;
    }
    if (loc.type === "Pdf") {
      return `Page ${loc.details.page_number}`;
    }
    return `Offset ${loc.details.offset}`;
  }

  private highlightSnippet(snippet: string, matchedText: string): string {
    if (!matchedText) return snippet;
    const escaped = matchedText.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const regex = new RegExp(`(${escaped})`, "gi");
    return snippet.replace(
      regex,
      '<mark class="federated-highlight" style="background-color: #fef08a; color: #854d0e; padding: 1px 3px; border-radius: 2px; font-weight: 600;">$1</mark>'
    );
  }

  private render(): void {
    const isDark =
      typeof document !== "undefined" &&
      (document.documentElement.classList.contains("theme-dark") ||
        document.documentElement.dataset.theme === "dark");

    const bgPanel = isDark ? "#1f2937" : "#ffffff";
    const textMain = isDark ? "#f3f4f6" : "#111827";
    const textMuted = isDark ? "#9ca3af" : "#6b7280";
    const borderCol = isDark ? "#374151" : "#e5e7eb";
    const inputBg = isDark ? "#111827" : "#f9fafb";
    const activeItemBg = isDark ? "#374151" : "#e0e7ff";
    const hoverItemBg = isDark ? "#283548" : "#f3f4f6";

    // Preserve input values across renders
    const prevQuery = this.element.querySelector<HTMLInputElement>(".federated-query-input")?.value || "";
    const prevReplace = this.element.querySelector<HTMLInputElement>(".federated-replace-input")?.value || "";
    const prevCase = this.element.querySelector<HTMLInputElement>(".federated-case-checkbox")?.checked ?? false;
    const prevWord = this.element.querySelector<HTMLInputElement>(".federated-word-checkbox")?.checked ?? false;
    const prevRegex = this.element.querySelector<HTMLInputElement>(".federated-regex-checkbox")?.checked ?? false;

    // Group matches by session
    const groups: { sessionId: string; title: string; docType: string; items: FederatedSearchMatch[] }[] = [];
    const groupMap = new Map<string, { sessionId: string; title: string; docType: string; items: FederatedSearchMatch[] }>();

    for (const m of this.matches) {
      if (!groupMap.has(m.session_id)) {
        const g = { sessionId: m.session_id, title: m.document_title, docType: m.doc_type, items: [] };
        groupMap.set(m.session_id, g);
        groups.push(g);
      }
      groupMap.get(m.session_id)!.items.push(m);
    }

    this.element.innerHTML = `
      <div class="federated-header" style="display: flex; align-items: center; justify-content: space-between; padding: 12px 16px; border-bottom: 1px solid ${borderCol}; background: ${bgPanel};">
        <div style="display: flex; align-items: center; gap: 8px;">
          <span style="font-weight: 600; font-size: 14px; color: ${textMain};">Workspace Search & Replace</span>
          <span style="font-size: 11px; padding: 2px 6px; border-radius: 4px; background: #3b82f620; color: #3b82f6; font-weight: 500;">Ctrl+Shift+F</span>
        </div>
        <div style="display: flex; align-items: center; gap: 6px;">
          <div style="display: flex; background: ${inputBg}; border: 1px solid ${borderCol}; border-radius: 4px; overflow: hidden;">
            <button class="federated-tab-find" style="padding: 4px 10px; border: none; font-size: 12px; cursor: pointer; background: ${this.mode === "find" ? "#3b82f6" : "transparent"}; color: ${this.mode === "find" ? "#ffffff" : textMain}; font-weight: ${this.mode === "find" ? "600" : "normal"};">Find</button>
            <button class="federated-tab-replace" style="padding: 4px 10px; border: none; font-size: 12px; cursor: pointer; background: ${this.mode === "replace" ? "#3b82f6" : "transparent"}; color: ${this.mode === "replace" ? "#ffffff" : textMain}; font-weight: ${this.mode === "replace" ? "600" : "normal"};">Replace</button>
          </div>
          <button class="federated-btn-close" aria-label="Close" style="border: none; background: transparent; color: ${textMuted}; cursor: pointer; font-size: 16px; padding: 2px 6px; border-radius: 4px;">✕</button>
        </div>
      </div>

      <div class="federated-inputs-area" style="padding: 12px 16px; display: flex; flex-direction: column; gap: 8px; border-bottom: 1px solid ${borderCol}; background: ${bgPanel};">
        <div style="display: flex; gap: 6px; align-items: center;">
          <input type="text" class="federated-query-input" placeholder="Search across all open documents..." style="flex: 1; padding: 6px 10px; border: 1px solid ${borderCol}; border-radius: 4px; background: ${inputBg}; color: ${textMain}; font-size: 13px;" />
          <button class="federated-search-btn" style="padding: 6px 12px; border: 1px solid #2563eb; background: #2563eb; color: #ffffff; border-radius: 4px; font-weight: 500; cursor: pointer;">Find All</button>
        </div>

        <div class="federated-replace-row" style="display: ${this.mode === "replace" ? "flex" : "none"}; gap: 6px; align-items: center;">
          <input type="text" class="federated-replace-input" placeholder="Replace with..." style="flex: 1; padding: 6px 10px; border: 1px solid ${borderCol}; border-radius: 4px; background: ${inputBg}; color: ${textMain}; font-size: 13px;" />
          <button class="federated-replace-btn" style="padding: 6px 10px; border: 1px solid ${borderCol}; background: ${bgPanel}; color: ${textMain}; border-radius: 4px; cursor: pointer;">Replace</button>
          <button class="federated-replace-all-btn" style="padding: 6px 10px; border: 1px solid #16a34a; background: #16a34a; color: #ffffff; border-radius: 4px; font-weight: 500; cursor: pointer;">Replace All</button>
        </div>

        <div style="display: flex; align-items: center; gap: 14px; font-size: 12px; color: ${textMuted};">
          <label style="display: flex; align-items: center; gap: 4px; cursor: pointer;">
            <input type="checkbox" class="federated-case-checkbox" /> Match case
          </label>
          <label style="display: flex; align-items: center; gap: 4px; cursor: pointer;">
            <input type="checkbox" class="federated-word-checkbox" /> Whole word
          </label>
          <label style="display: flex; align-items: center; gap: 4px; cursor: pointer;">
            <input type="checkbox" class="federated-regex-checkbox" /> Regex
          </label>
        </div>
      </div>

      <div class="federated-status-bar" style="padding: 6px 16px; font-size: 11px; color: ${textMuted}; background: ${inputBg}; border-bottom: 1px solid ${borderCol}; display: flex; justify-content: space-between;">
        <span class="federated-status-text">${this.statusMessage || "Enter search term and press Find All."}</span>
        <span class="federated-matches-count">${this.matches.length > 0 ? `${this.matches.length} hits` : ""}</span>
      </div>

      <div class="federated-results-container" style="flex: 1; overflow-y: auto; max-height: 380px; padding: 8px 12px; display: flex; flex-direction: column; gap: 8px; background: ${bgPanel};">
        ${
          groups.length === 0
            ? `<div style="padding: 24px; text-align: center; color: ${textMuted}; font-size: 12px;">No document matches to display.</div>`
            : groups
                .map(
                  (g) => `
          <div class="federated-doc-group" data-session-id="${g.sessionId}" style="border: 1px solid ${borderCol}; border-radius: 6px; overflow: hidden;">
            <div class="federated-doc-header" style="display: flex; align-items: center; justify-content: space-between; padding: 6px 10px; background: ${inputBg}; border-bottom: 1px solid ${borderCol}; cursor: pointer;">
              <div style="display: flex; align-items: center; gap: 6px; font-weight: 600; font-size: 12px; color: ${textMain};">
                <span style="padding: 1px 5px; border-radius: 3px; font-size: 10px; background: #3b82f620; color: #3b82f6;">${g.docType}</span>
                <span>${g.title}</span>
              </div>
              <span style="font-size: 11px; color: ${textMuted};">${g.items.length} hit${g.items.length === 1 ? "" : "s"}</span>
            </div>
            <div class="federated-doc-matches" style="display: flex; flex-direction: column;">
              ${g.items
                .map((m) => {
                  const isSelected = m.match_id === this.selectedMatchId;
                  const itemBg = isSelected ? activeItemBg : "transparent";
                  return `
                <div class="federated-match-item" data-match-id="${m.match_id}" tabindex="0" role="button" aria-label="Match at ${this.formatLocation(m)}" style="padding: 6px 10px; border-bottom: 1px solid ${borderCol}20; background: ${itemBg}; cursor: pointer; display: flex; flex-direction: column; gap: 2px;">
                  <div style="display: flex; justify-content: space-between; font-size: 11px; color: ${textMuted};">
                    <span style="font-weight: 500;">${this.formatLocation(m)}</span>
                    <span style="font-family: monospace; font-size: 10px;">${m.matched_text}</span>
                  </div>
                  <div class="federated-snippet" style="font-size: 12px; color: ${textMain}; white-space: pre-wrap; word-break: break-word;">${this.highlightSnippet(m.snippet, m.matched_text)}</div>
                </div>
              `;
                })
                .join("")}
            </div>
          </div>
        `
                )
                .join("")
        }
      </div>

      <div class="federated-footer" style="padding: 8px 16px; border-top: 1px solid ${borderCol}; background: ${inputBg}; display: flex; justify-content: flex-end; align-items: center; gap: 8px;">
        <button class="federated-btn-dismiss" style="padding: 4px 12px; border: 1px solid ${borderCol}; background: ${bgPanel}; color: ${textMain}; border-radius: 4px; cursor: pointer; font-size: 12px;">Close (Esc)</button>
      </div>
    `;

    // Restore input states
    const queryInput = this.element.querySelector<HTMLInputElement>(".federated-query-input");
    const replaceInput = this.element.querySelector<HTMLInputElement>(".federated-replace-input");
    const caseCb = this.element.querySelector<HTMLInputElement>(".federated-case-checkbox");
    const wordCb = this.element.querySelector<HTMLInputElement>(".federated-word-checkbox");
    const regexCb = this.element.querySelector<HTMLInputElement>(".federated-regex-checkbox");

    if (queryInput && prevQuery) queryInput.value = prevQuery;
    if (replaceInput && prevReplace) replaceInput.value = prevReplace;
    if (caseCb) caseCb.checked = prevCase;
    if (wordCb) wordCb.checked = prevWord;
    if (regexCb) regexCb.checked = prevRegex;

    // Attach Event Listeners
    const tabFind = this.element.querySelector<HTMLButtonElement>(".federated-tab-find");
    const tabReplace = this.element.querySelector<HTMLButtonElement>(".federated-tab-replace");
    const closeBtn = this.element.querySelector<HTMLButtonElement>(".federated-btn-close");
    const dismissBtn = this.element.querySelector<HTMLButtonElement>(".federated-btn-dismiss");
    const searchBtn = this.element.querySelector<HTMLButtonElement>(".federated-search-btn");
    const replaceBtn = this.element.querySelector<HTMLButtonElement>(".federated-replace-btn");
    const replaceAllBtn = this.element.querySelector<HTMLButtonElement>(".federated-replace-all-btn");

    if (tabFind) {
      tabFind.onclick = () => {
        this.mode = "find";
        this.render();
      };
    }
    if (tabReplace) {
      tabReplace.onclick = () => {
        this.mode = "replace";
        this.render();
      };
    }
    if (closeBtn) closeBtn.onclick = () => this.hide();
    if (dismissBtn) dismissBtn.onclick = () => this.hide();

    if (searchBtn) {
      searchBtn.onclick = () => void this.executeSearch();
    }
    if (replaceBtn) {
      replaceBtn.onclick = () => void this.executeReplaceSelected();
    }
    if (replaceAllBtn) {
      replaceAllBtn.onclick = () => void this.executeReplaceAll();
    }

    if (queryInput) {
      queryInput.onkeydown = (e) => {
        if (e.key === "Enter") {
          e.preventDefault();
          void this.executeSearch();
        } else if (e.key === "Escape") {
          e.preventDefault();
          this.hide();
        }
      };
    }

    if (replaceInput) {
      replaceInput.onkeydown = (e) => {
        if (e.key === "Enter") {
          e.preventDefault();
          void this.executeReplaceSelected();
        } else if (e.key === "Escape") {
          e.preventDefault();
          this.hide();
        }
      };
    }

    // Match item clicks and doc header clicks
    const matchItems = this.element.querySelectorAll<HTMLElement>(".federated-match-item");
    matchItems.forEach((el) => {
      const matchId = el.getAttribute("data-match-id");
      if (!matchId) return;
      const match = this.matches.find((m) => m.match_id === matchId);
      if (!match) return;

      el.onclick = () => this.selectMatch(match);
      el.onkeydown = (e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          this.selectMatch(match);
        }
      };

      el.onmouseenter = () => {
        if (el.getAttribute("data-match-id") !== this.selectedMatchId) {
          el.style.backgroundColor = hoverItemBg;
        }
      };
      el.onmouseleave = () => {
        if (el.getAttribute("data-match-id") !== this.selectedMatchId) {
          el.style.backgroundColor = "transparent";
        }
      };
    });

    const docHeaders = this.element.querySelectorAll<HTMLElement>(".federated-doc-header");
    docHeaders.forEach((dh) => {
      const groupEl = dh.closest<HTMLElement>(".federated-doc-group");
      const sessionId = groupEl?.getAttribute("data-session-id");
      if (sessionId) {
        dh.onclick = () => {
          if (this.callbacks?.onTabSwitchRequested) {
            this.callbacks.onTabSwitchRequested(sessionId);
          }
        };
      }
    });
  }
}
