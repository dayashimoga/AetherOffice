import { IPC, OmniboxItem, OmniboxCategory } from "../ipc";

export interface OmniboxCallbacks {
  onExecuteItem?: (item: OmniboxItem) => void;
  onTabSwitch?: (sessionId: string) => void;
  onClose?: () => void;
}

export class OmniboxLauncher {
  private overlay: HTMLElement;
  private modal: HTMLElement;
  private isVisible = false;
  private items: OmniboxItem[] = [];
  private selectedIndex = 0;
  private activeCategoryFilter: "All" | OmniboxCategory = "All";
  private callbacks?: OmniboxCallbacks;

  constructor() {
    this.overlay = document.createElement("div");
    this.overlay.className = "omnibox-launcher-overlay";
    this.overlay.id = "omnibox-launcher-overlay";
    this.overlay.setAttribute("role", "dialog");
    this.overlay.setAttribute("aria-label", "Command, File & Symbol Omnibox");

    this.modal = document.createElement("div");
    this.modal.className = "omnibox-launcher-modal";
    this.modal.id = "omnibox-launcher-modal";

    this.overlay.appendChild(this.modal);
    this.applyStyles();
    document.body.appendChild(this.overlay);
    this.render();
  }

  public setCallbacks(callbacks: OmniboxCallbacks): void {
    this.callbacks = callbacks;
  }

  public show(initialQuery = ""): void {
    this.isVisible = true;
    this.overlay.style.display = "flex";
    this.selectedIndex = 0;
    this.render();

    const input = this.modal.querySelector<HTMLInputElement>(".omnibox-input");
    if (input) {
      if (initialQuery) {
        input.value = initialQuery;
      }
      input.focus();
      input.select();
      void this.executeQuery(input.value);
    }
  }

  public hide(): void {
    this.isVisible = false;
    this.overlay.style.display = "none";
    this.items = [];
    this.selectedIndex = 0;
    if (this.callbacks?.onClose) {
      this.callbacks.onClose();
    }
  }

  public toggle(initialQuery = ""): void {
    if (this.isVisible) {
      this.hide();
    } else {
      this.show(initialQuery);
    }
  }

  public isOpen(): boolean {
    return this.isVisible;
  }

  public getItems(): OmniboxItem[] {
    return this.items;
  }

  public getSelectedIndex(): number {
    return this.selectedIndex;
  }

  public async executeQuery(query: string): Promise<void> {
    try {
      let finalQuery = query;
      if (this.activeCategoryFilter === "Command" && !query.startsWith(">")) {
        finalQuery = `>${query}`;
      } else if (this.activeCategoryFilter === "File" && !query.startsWith("@") && !query.startsWith("#")) {
        finalQuery = `@${query}`;
      } else if (this.activeCategoryFilter === "Symbol" && !query.startsWith(":") && !query.startsWith("$")) {
        finalQuery = `:${query}`;
      }

      const results = await IPC.omniboxQuery(finalQuery, 25);
      this.items = results;
      this.selectedIndex = 0;
      this.renderResults();
    } catch (err) {
      console.warn("Omnibox query failed:", err);
      this.items = [];
      this.renderResults();
    }
  }

  public selectNext(): void {
    if (this.items.length === 0) return;
    this.selectedIndex = (this.selectedIndex + 1) % this.items.length;
    this.updateSelection();
  }

  public selectPrev(): void {
    if (this.items.length === 0) return;
    this.selectedIndex = (this.selectedIndex - 1 + this.items.length) % this.items.length;
    this.updateSelection();
  }

  public executeSelected(): void {
    if (this.items.length === 0 || this.selectedIndex >= this.items.length) {
      return;
    }
    const item = this.items[this.selectedIndex];
    this.executeItem(item);
  }

  public executeItem(item: OmniboxItem): void {
    this.hide();

    if (item.category === "File") {
      if (this.callbacks?.onTabSwitch) {
        this.callbacks.onTabSwitch(item.payload);
      }
    }

    if (this.callbacks?.onExecuteItem) {
      this.callbacks.onExecuteItem(item);
    }
  }

  private applyStyles(): void {
    this.overlay.style.display = "none";
    this.overlay.style.position = "fixed";
    this.overlay.style.top = "0";
    this.overlay.style.left = "0";
    this.overlay.style.width = "100vw";
    this.overlay.style.height = "100vh";
    this.overlay.style.zIndex = "3000";
    this.overlay.style.backgroundColor = "rgba(0, 0, 0, 0.45)";
    this.overlay.style.backdropFilter = "blur(4px)";
    this.overlay.style.alignItems = "flex-start";
    this.overlay.style.justifyContent = "center";
    this.overlay.style.paddingTop = "15vh";

    this.modal.style.width = "620px";
    this.modal.style.maxWidth = "90vw";
    this.modal.style.maxHeight = "70vh";
    this.modal.style.backgroundColor = "var(--bg-panel, #ffffff)";
    this.modal.style.color = "var(--text-main, #1f2937)";
    this.modal.style.border = "1px solid var(--border-panel, #d1d5db)";
    this.modal.style.borderRadius = "10px";
    this.modal.style.boxShadow = "0 20px 48px rgba(0, 0, 0, 0.3)";
    this.modal.style.display = "flex";
    this.modal.style.flexDirection = "column";
    this.modal.style.overflow = "hidden";
    this.modal.style.fontFamily = "Segoe UI, -apple-system, sans-serif";
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

    const prevValue = this.modal.querySelector<HTMLInputElement>(".omnibox-input")?.value || "";

    this.modal.innerHTML = `
      <div class="omnibox-search-row" style="display: flex; align-items: center; gap: 10px; padding: 14px 18px; border-bottom: 1px solid ${borderCol}; background: ${inputBg};">
        <span style="font-size: 16px; color: ${textMuted};">🔍</span>
        <input type="text" class="omnibox-input" placeholder="Type a command or search... (> commands, @ files, : symbols)" style="flex: 1; border: none; background: transparent; color: ${textMain}; font-size: 15px; outline: none;" />
        <span class="omnibox-shortcut-hint" style="font-size: 11px; padding: 2px 6px; border-radius: 4px; background: ${bgPanel}; border: 1px solid ${borderCol}; color: ${textMuted}; font-family: monospace;">ESC</span>
      </div>

      <div class="omnibox-pill-filters" style="display: flex; align-items: center; gap: 6px; padding: 8px 18px; border-bottom: 1px solid ${borderCol}; background: ${bgPanel}; font-size: 12px;">
        <button class="omnibox-pill" data-cat="All" style="padding: 3px 10px; border-radius: 12px; border: 1px solid ${borderCol}; background: ${this.activeCategoryFilter === "All" ? "#3b82f6" : "transparent"}; color: ${this.activeCategoryFilter === "All" ? "#ffffff" : textMuted}; cursor: pointer;">All</button>
        <button class="omnibox-pill" data-cat="Command" style="padding: 3px 10px; border-radius: 12px; border: 1px solid ${borderCol}; background: ${this.activeCategoryFilter === "Command" ? "#3b82f6" : "transparent"}; color: ${this.activeCategoryFilter === "Command" ? "#ffffff" : textMuted}; cursor: pointer;">&gt; Commands</button>
        <button class="omnibox-pill" data-cat="File" style="padding: 3px 10px; border-radius: 12px; border: 1px solid ${borderCol}; background: ${this.activeCategoryFilter === "File" ? "#3b82f6" : "transparent"}; color: ${this.activeCategoryFilter === "File" ? "#ffffff" : textMuted}; cursor: pointer;">@ Open Tabs</button>
        <button class="omnibox-pill" data-cat="Symbol" style="padding: 3px 10px; border-radius: 12px; border: 1px solid ${borderCol}; background: ${this.activeCategoryFilter === "Symbol" ? "#3b82f6" : "transparent"}; color: ${this.activeCategoryFilter === "Symbol" ? "#ffffff" : textMuted}; cursor: pointer;">: Symbols</button>
      </div>

      <div class="omnibox-results-list" style="flex: 1; overflow-y: auto; max-height: 420px; display: flex; flex-direction: column; background: ${bgPanel};">
        <!-- Rendered dynamically -->
      </div>

      <div class="omnibox-footer" style="display: flex; align-items: center; justify-content: space-between; padding: 8px 18px; border-top: 1px solid ${borderCol}; background: ${inputBg}; font-size: 11px; color: ${textMuted};">
        <div style="display: flex; gap: 12px;">
          <span><kbd style="font-family: monospace; font-size: 10px; padding: 1px 4px; border: 1px solid ${borderCol}; border-radius: 3px;">↑↓</kbd> navigate</span>
          <span><kbd style="font-family: monospace; font-size: 10px; padding: 1px 4px; border: 1px solid ${borderCol}; border-radius: 3px;">↵</kbd> select</span>
          <span><kbd style="font-family: monospace; font-size: 10px; padding: 1px 4px; border: 1px solid ${borderCol}; border-radius: 3px;">esc</kbd> close</span>
        </div>
        <span class="omnibox-result-count">${this.items.length} items</span>
      </div>
    `;

    const input = this.modal.querySelector<HTMLInputElement>(".omnibox-input");
    if (input) {
      if (prevValue) input.value = prevValue;
      input.oninput = () => void this.executeQuery(input.value);
      input.onkeydown = (e) => {
        if (e.key === "ArrowDown") {
          e.preventDefault();
          this.selectNext();
        } else if (e.key === "ArrowUp") {
          e.preventDefault();
          this.selectPrev();
        } else if (e.key === "Enter") {
          e.preventDefault();
          this.executeSelected();
        } else if (e.key === "Escape") {
          e.preventDefault();
          this.hide();
        }
      };
    }

    const pills = this.modal.querySelectorAll<HTMLButtonElement>(".omnibox-pill");
    pills.forEach((p) => {
      p.onclick = () => {
        const cat = p.getAttribute("data-cat") as "All" | OmniboxCategory;
        if (cat) {
          this.activeCategoryFilter = cat;
          this.render();
          const currInput = this.modal.querySelector<HTMLInputElement>(".omnibox-input");
          void this.executeQuery(currInput?.value || "");
        }
      };
    });

    this.overlay.onclick = (e) => {
      if (e.target === this.overlay) {
        this.hide();
      }
    };

    this.renderResults();
  }

  private renderResults(): void {
    const list = this.modal.querySelector<HTMLElement>(".omnibox-results-list");
    if (!list) return;

    const countEl = this.modal.querySelector<HTMLElement>(".omnibox-result-count");
    if (countEl) {
      countEl.textContent = `${this.items.length} item${this.items.length === 1 ? "" : "s"}`;
    }

    const isDark =
      typeof document !== "undefined" &&
      (document.documentElement.classList.contains("theme-dark") ||
        document.documentElement.dataset.theme === "dark");

    const textMain = isDark ? "#f3f4f6" : "#111827";
    const textMuted = isDark ? "#9ca3af" : "#6b7280";
    const activeItemBg = isDark ? "#374151" : "#e0e7ff";
    const borderCol = isDark ? "#374151" : "#e5e7eb";

    if (this.items.length === 0) {
      list.innerHTML = `
        <div style="padding: 32px; text-align: center; color: ${textMuted}; font-size: 13px;">
          No matching commands, files, or symbols found.
        </div>
      `;
      return;
    }

    list.innerHTML = this.items
      .map((item, idx) => {
        const isSelected = idx === this.selectedIndex;
        const bg = isSelected ? activeItemBg : "transparent";

        const badgeBg =
          item.category === "Command"
            ? "#3b82f620"
            : item.category === "File"
            ? "#10b98120"
            : item.category === "Symbol"
            ? "#f59e0b20"
            : "#8b5cf620";

        const badgeColor =
          item.category === "Command"
            ? "#3b82f6"
            : item.category === "File"
            ? "#10b981"
            : item.category === "Symbol"
            ? "#d97706"
            : "#8b5cf6";

        return `
        <div class="omnibox-item ${isSelected ? "selected" : ""}" data-index="${idx}" tabindex="0" role="option" aria-selected="${isSelected}" style="padding: 10px 18px; display: flex; align-items: center; justify-content: space-between; background: ${bg}; cursor: pointer; border-bottom: 1px solid ${borderCol}20;">
          <div style="display: flex; align-items: center; gap: 10px; overflow: hidden;">
            <span style="font-size: 10px; font-weight: 600; padding: 2px 6px; border-radius: 4px; background: ${badgeBg}; color: ${badgeColor}; min-width: 58px; text-align: center;">${item.category}</span>
            <div style="display: flex; flex-direction: column; gap: 2px; overflow: hidden;">
              <span class="omnibox-item-title" style="font-size: 13px; font-weight: 500; color: ${textMain}; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${item.title}</span>
              <span class="omnibox-item-desc" style="font-size: 11px; color: ${textMuted}; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">${item.description}</span>
            </div>
          </div>
          ${
            item.shortcut
              ? `<span style="font-size: 11px; font-family: monospace; padding: 2px 6px; border-radius: 4px; border: 1px solid ${borderCol}; color: ${textMuted}; margin-left: 12px; white-space: nowrap;">${item.shortcut}</span>`
              : ""
          }
        </div>
      `;
      })
      .join("");

    const itemEls = list.querySelectorAll<HTMLElement>(".omnibox-item");
    itemEls.forEach((el) => {
      const idx = parseInt(el.getAttribute("data-index") || "0", 10);
      el.onclick = () => {
        this.selectedIndex = idx;
        this.executeSelected();
      };
      el.onmouseenter = () => {
        this.selectedIndex = idx;
        this.updateSelection();
      };
    });
  }

  private updateSelection(): void {
    const list = this.modal.querySelector<HTMLElement>(".omnibox-results-list");
    if (!list) return;

    const isDark =
      typeof document !== "undefined" &&
      (document.documentElement.classList.contains("theme-dark") ||
        document.documentElement.dataset.theme === "dark");
    const activeItemBg = isDark ? "#374151" : "#e0e7ff";

    const itemEls = list.querySelectorAll<HTMLElement>(".omnibox-item");
    itemEls.forEach((el, idx) => {
      const isSelected = idx === this.selectedIndex;
      el.style.backgroundColor = isSelected ? activeItemBg : "transparent";
      el.setAttribute("aria-selected", isSelected ? "true" : "false");
      if (isSelected) {
        el.classList.add("selected");
        el.scrollIntoView({ block: "nearest" });
      } else {
        el.classList.remove("selected");
      }
    });
  }
}

