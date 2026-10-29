import { CalcState, SheetData } from "../state/calcState";

export interface WorkbookSheetsPaneCallbacks {
  onSheetSwitched?: (sheetId: string) => void;
  onSheetAdded?: (sheet: SheetData) => void;
  onSheetRenamed?: (sheetId: string, newName: string) => void;
  onSheetDeleted?: (sheetId: string) => void;
}

export class WorkbookSheetsPane {
  private container: HTMLElement;
  private state: CalcState;
  private callbacks?: WorkbookSheetsPaneCallbacks;

  constructor(containerId: string, state: CalcState) {
    const el = document.getElementById(containerId);
    if (!el) throw new Error(`WorkbookSheetsPane container #${containerId} not found`);
    this.container = el;
    this.state = state;
    this.render();
  }

  public setCallbacks(cb: WorkbookSheetsPaneCallbacks) {
    this.callbacks = cb;
  }

  public refresh() {
    this.render();
  }

  private render() {
    this.container.innerHTML = "";

    const wrapper = document.createElement("div");
    wrapper.className = "workbook-sheets-pane";

    // Heading row
    const heading = document.createElement("div");
    heading.className = "sidebar-heading workbook-sheets-header";
    heading.style.display = "flex";
    heading.style.justifyContent = "space-between";
    heading.style.alignItems = "center";

    const titleSpan = document.createElement("span");
    titleSpan.textContent = "Workbook Sheets";
    heading.appendChild(titleSpan);

    const addBtn = document.createElement("button");
    addBtn.className = "btn-sidebar-add-sheet";
    addBtn.textContent = "+";
    addBtn.title = "Add New Sheet";
    addBtn.onclick = () => {
      const newSheet = this.state.addSheet();
      this.render();
      this.callbacks?.onSheetAdded?.(newSheet);
      this.callbacks?.onSheetSwitched?.(newSheet.id);
    };
    heading.appendChild(addBtn);

    wrapper.appendChild(heading);

    // List of sheets
    const list = document.createElement("div");
    list.className = "sheets-list";
    list.style.display = "flex";
    list.style.flexDirection = "column";
    list.style.gap = "4px";
    list.style.padding = "8px";

    this.state.sheets.forEach((sheet) => {
      const isActive = sheet.id === this.state.activeSheetId;
      const item = document.createElement("div");
      item.className = `sheet-item ${isActive ? "active" : ""}`;
      item.dataset.sheetId = sheet.id;
      item.style.display = "flex";
      item.style.alignItems = "center";
      item.style.justifyContent = "space-between";
      item.style.padding = "6px 10px";
      item.style.borderRadius = "4px";
      item.style.cursor = "pointer";
      item.style.backgroundColor = isActive ? "#caead8" : "transparent";
      item.style.fontWeight = isActive ? "600" : "normal";

      const left = document.createElement("div");
      left.style.display = "flex";
      left.style.alignItems = "center";
      left.style.gap = "8px";

      const icon = document.createElement("span");
      icon.textContent = "📊";
      left.appendChild(icon);

      const name = document.createElement("span");
      name.className = "sheet-name";
      name.textContent = sheet.name;
      left.appendChild(name);

      const count = document.createElement("span");
      count.className = "sheet-cell-count";
      count.style.fontSize = "11px";
      count.style.color = "#6b7280";
      const populated = Array.from(sheet.cells.values()).filter((c) => c.raw.trim().length > 0).length;
      count.textContent = `(${populated} cells)`;
      left.appendChild(count);

      item.appendChild(left);

      // Actions
      const actions = document.createElement("div");
      actions.className = "sheet-actions";
      actions.style.display = "flex";
      actions.style.gap = "4px";

      const renameBtn = document.createElement("button");
      renameBtn.className = "btn-sheet-rename";
      renameBtn.textContent = "✏️";
      renameBtn.title = "Rename sheet";
      renameBtn.style.background = "none";
      renameBtn.style.border = "none";
      renameBtn.style.cursor = "pointer";
      renameBtn.onclick = (e) => {
        e.stopPropagation();
        const newName = window.prompt("Enter new sheet name:", sheet.name);
        if (newName && newName.trim() && newName.trim() !== sheet.name) {
          const ok = this.state.renameSheet(sheet.id, newName.trim());
          if (ok) {
            this.render();
            this.callbacks?.onSheetRenamed?.(sheet.id, newName.trim());
          }
        }
      };
      actions.appendChild(renameBtn);

      if (this.state.sheets.length > 1) {
        const delBtn = document.createElement("button");
        delBtn.className = "btn-sheet-delete";
        delBtn.textContent = "🗑️";
        delBtn.title = "Delete sheet";
        delBtn.style.background = "none";
        delBtn.style.border = "none";
        delBtn.style.cursor = "pointer";
        delBtn.onclick = (e) => {
          e.stopPropagation();
          const ok = this.state.deleteSheet(sheet.id);
          if (ok) {
            this.render();
            this.callbacks?.onSheetDeleted?.(sheet.id);
          }
        };
        actions.appendChild(delBtn);
      }

      item.appendChild(actions);

      item.onclick = () => {
        if (this.state.activeSheetId !== sheet.id) {
          this.state.switchSheet(sheet.id);
          this.render();
          this.callbacks?.onSheetSwitched?.(sheet.id);
        }
      };

      list.appendChild(item);
    });

    wrapper.appendChild(list);
    this.container.appendChild(wrapper);
  }
}

