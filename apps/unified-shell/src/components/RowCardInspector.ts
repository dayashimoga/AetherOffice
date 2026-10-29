export interface CardFieldData {
  label: string;
  value: string;
  colIndex: number;
}

export class RowCardInspector {
  private container: HTMLElement;
  private currentRow = 1;
  private totalRows = 100;
  private fields: CardFieldData[] = [];
  private onCommitFieldCallback?: (row: number, col: number, value: string) => void;
  private onNavigateRowCallback?: (row: number) => void;

  constructor(containerId = "row-card-inspector-container") {
    let el = document.getElementById(containerId);
    if (!el) {
      el = document.createElement("div");
      el.id = containerId;
      el.className = "row-card-inspector";
      el.style.display = "none";
      const viewport = document.getElementById("viewport-container");
      if (viewport) viewport.appendChild(el);
      else document.body.appendChild(el);
    }
    this.container = el;
  }

  public setCallbacks(callbacks: {
    onCommitField?: (row: number, col: number, value: string) => void;
    onNavigateRow?: (row: number) => void;
  }) {
    this.onCommitFieldCallback = callbacks.onCommitField;
    this.onNavigateRowCallback = callbacks.onNavigateRow;
  }

  public setRowData(row: number, totalRows: number, fields: CardFieldData[]) {
    this.currentRow = row;
    this.totalRows = totalRows;
    this.fields = fields;
    this.render();
  }

  public show() {
    this.container.style.display = "flex";
  }

  public hide() {
    this.container.style.display = "none";
  }

  private render() {
    this.container.innerHTML = `
      <div style="display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid var(--border-color); padding-bottom: 8px;">
        <span style="font-weight: 600;">Row Card Inspector</span>
        <span style="color: var(--text-secondary); font-size: 12px;">Row ${this.currentRow} of ${this.totalRows}</span>
      </div>
      <div class="card-fields-list" style="display: flex; flex-direction: column; gap: 8px; max-height: 40vh; overflow-y: auto;">
        ${this.fields
          .map(
            (f) => `
          <div class="card-field-row">
            <span class="card-field-label">${f.label}</span>
            <input class="card-field-input" data-col="${f.colIndex}" value="${f.value}" />
          </div>
        `
          )
          .join("")}
      </div>
      <div class="card-nav-bar">
        <button id="card-nav-prev" class="ribbon-btn">▲ Previous Row</button>
        <button id="card-nav-next" class="ribbon-btn">▼ Next Row</button>
      </div>
    `;

    this.container.querySelectorAll(".card-field-input").forEach((inp) => {
      inp.addEventListener("change", (e) => {
        const inputEl = e.currentTarget as HTMLInputElement;
        const col = parseInt(inputEl.getAttribute("data-col") || "0", 10);
        if (this.onCommitFieldCallback) {
          this.onCommitFieldCallback(this.currentRow, col, inputEl.value);
        }
      });
    });

    document.getElementById("card-nav-prev")?.addEventListener("click", () => {
      if (this.currentRow > 1 && this.onNavigateRowCallback) {
        this.onNavigateRowCallback(this.currentRow - 1);
      }
    });

    document.getElementById("card-nav-next")?.addEventListener("click", () => {
      if (this.currentRow < this.totalRows && this.onNavigateRowCallback) {
        this.onNavigateRowCallback(this.currentRow + 1);
      }
    });
  }
}

