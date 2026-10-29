import { CalcState, SheetData } from "../state/calcState";

export interface SheetTabBarCallbacks {
  onSheetSwitched?: (sheetId: string) => void;
  onSheetAdded?: (sheet: SheetData) => void;
  onSheetRenamed?: (sheetId: string, newName: string) => void;
  onSheetDeleted?: (sheetId: string) => void;
}

export class SheetTabBar {
  private container: HTMLElement;
  private element: HTMLElement;
  private state: CalcState;
  private callbacks?: SheetTabBarCallbacks;

  constructor(container: HTMLElement, state: CalcState) {
    this.container = container;
    this.state = state;

    this.element = document.createElement("div");
    this.element.className = "calc-sheet-tab-bar";
    this.element.id = "calc-sheet-tab-bar";
    this.container.appendChild(this.element);

    this.render();
  }

  public setCallbacks(cb: SheetTabBarCallbacks) {
    this.callbacks = cb;
  }

  public refresh() {
    this.render();
  }

  public destroy() {
    this.element.remove();
  }

  private render() {
    this.element.innerHTML = "";

    // 1. Navigation buttons (◀, ▶)
    const navBox = document.createElement("div");
    navBox.className = "calc-sheet-nav-btns";

    const prevBtn = document.createElement("button");
    prevBtn.className = "sheet-nav-btn btn-sheet-prev";
    prevBtn.textContent = "◀";
    prevBtn.title = "Previous sheet";
    prevBtn.onclick = () => {
      const idx = this.state.getActiveSheetIndex();
      if (idx > 0) {
        const prevSheet = this.state.sheets[idx - 1];
        this.state.switchSheet(prevSheet.id);
        this.render();
        this.callbacks?.onSheetSwitched?.(prevSheet.id);
      }
    };

    const nextBtn = document.createElement("button");
    nextBtn.className = "sheet-nav-btn btn-sheet-next";
    nextBtn.textContent = "▶";
    nextBtn.title = "Next sheet";
    nextBtn.onclick = () => {
      const idx = this.state.getActiveSheetIndex();
      if (idx < this.state.sheets.length - 1) {
        const nextSheet = this.state.sheets[idx + 1];
        this.state.switchSheet(nextSheet.id);
        this.render();
        this.callbacks?.onSheetSwitched?.(nextSheet.id);
      }
    };

    navBox.appendChild(prevBtn);
    navBox.appendChild(nextBtn);
    this.element.appendChild(navBox);

    // 2. Sheet Tabs List
    const tabsList = document.createElement("div");
    tabsList.className = "calc-sheet-tabs-list";

    this.state.sheets.forEach((sheet) => {
      const isActive = sheet.id === this.state.activeSheetId;
      const tabEl = document.createElement("div");
      tabEl.className = `calc-sheet-tab ${isActive ? "active" : ""}`;
      tabEl.dataset.sheetId = sheet.id;
      tabEl.title = `${sheet.name} (Double-click to rename)`;

      const nameSpan = document.createElement("span");
      nameSpan.className = "sheet-tab-name";
      nameSpan.textContent = sheet.name;

      tabEl.appendChild(nameSpan);

      // Delete button if more than 1 sheet
      if (this.state.sheets.length > 1) {
        const delBtn = document.createElement("button");
        delBtn.className = "btn-delete-sheet";
        delBtn.textContent = "×";
        delBtn.title = `Delete ${sheet.name}`;
        delBtn.onclick = (e) => {
          e.stopPropagation();
          const ok = this.state.deleteSheet(sheet.id);
          if (ok) {
            this.render();
            this.callbacks?.onSheetDeleted?.(sheet.id);
          }
        };
        tabEl.appendChild(delBtn);
      }

      // Click to switch
      tabEl.onclick = () => {
        if (this.state.activeSheetId !== sheet.id) {
          this.state.switchSheet(sheet.id);
          this.render();
          this.callbacks?.onSheetSwitched?.(sheet.id);
        }
      };

      // Double-click to inline rename
      tabEl.ondblclick = (e) => {
        e.stopPropagation();
        this.startInlineRename(tabEl, sheet);
      };

      tabsList.appendChild(tabEl);
    });

    this.element.appendChild(tabsList);

    // 3. Add Sheet (+) Button
    const addBtn = document.createElement("button");
    addBtn.className = "btn-add-sheet";
    addBtn.textContent = "+";
    addBtn.title = "New Sheet";
    addBtn.onclick = () => {
      const newSheet = this.state.addSheet();
      this.render();
      this.callbacks?.onSheetAdded?.(newSheet);
      this.callbacks?.onSheetSwitched?.(newSheet.id);
    };

    this.element.appendChild(addBtn);
  }

  private startInlineRename(tabEl: HTMLElement, sheet: SheetData) {
    const nameSpan = tabEl.querySelector(".sheet-tab-name") as HTMLElement;
    if (!nameSpan) return;

    const input = document.createElement("input");
    input.type = "text";
    input.className = "sheet-rename-input";
    input.value = sheet.name;
    input.style.width = `${Math.max(60, sheet.name.length * 9)}px`;

    const commit = () => {
      const newName = input.value.trim();
      if (newName && newName !== sheet.name) {
        const ok = this.state.renameSheet(sheet.id, newName);
        if (ok) {
          this.callbacks?.onSheetRenamed?.(sheet.id, newName);
        }
      }
      this.render();
    };

    input.onkeydown = (e) => {
      if (e.key === "Enter") {
        e.preventDefault();
        commit();
      } else if (e.key === "Escape") {
        this.render();
      }
    };
    input.onblur = () => commit();

    nameSpan.replaceWith(input);
    input.focus();
    input.select();
  }
}

