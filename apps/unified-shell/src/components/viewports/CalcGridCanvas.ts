import { CalcState, calcState } from "../../state/calcState";

export class CalcGridCanvas {
  private container: HTMLElement;
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private state: CalcState;
  private editorInput: HTMLInputElement;
  private contextMenuEl: HTMLElement;
  private isEditing = false;
  private headerWidth = 50;
  private headerHeight = 26;
  private scrollX = 0;
  private scrollY = 0;
  private isDraggingFillHandle = false;
  private isSelecting = false;
  private isResizingCol = false;
  private resizingColIndex = -1;
  private colResizeStartX = 0;
  private colResizeInitialWidth = 0;
  private isResizingRow = false;
  private resizingRowIndex = -1;
  private rowResizeStartY = 0;
  private rowResizeInitialHeight = 0;

  private onCellSelected?: (coord: string, value: string) => void;
  private onCellEditing?: (coord: string, value: string) => void;
  private onCellModified?: (coord: string, value: string) => void;

  constructor(container: HTMLElement, state: CalcState = calcState) {
    this.container = container;
    this.state = state;

    this.canvas = document.createElement("canvas");
    this.canvas.className = "calc-grid-canvas";
    this.canvas.tabIndex = 0;
    this.canvas.style.display = "block";
    this.canvas.style.width = "100%";
    this.canvas.style.height = "100%";
    this.canvas.style.outline = "none";
    this.canvas.style.cursor = "cell";

    // Cell Editor Input Overlay
    this.editorInput = document.createElement("input");
    this.editorInput.type = "text";
    this.editorInput.className = "calc-cell-editor";
    this.editorInput.style.display = "none";
    this.editorInput.style.position = "absolute";
    this.editorInput.style.border = "2px solid #107c41";
    this.editorInput.style.outline = "none";
    this.editorInput.style.backgroundColor = "#ffffff";
    this.editorInput.style.padding = "0 4px";
    this.editorInput.style.font = "12px Calibri, sans-serif";
    this.editorInput.style.zIndex = "10";
    this.editorInput.style.boxSizing = "border-box";

    // Context Menu Overlay
    this.contextMenuEl = document.createElement("div");
    this.contextMenuEl.className = "calc-context-menu";
    this.contextMenuEl.style.display = "none";
    this.contextMenuEl.style.position = "absolute";
    this.contextMenuEl.style.zIndex = "100";
    this.contextMenuEl.style.backgroundColor = "#ffffff";
    this.contextMenuEl.style.border = "1px solid #d1d5db";
    this.contextMenuEl.style.borderRadius = "4px";
    this.contextMenuEl.style.boxShadow = "0 4px 12px rgba(0,0,0,0.15)";
    this.contextMenuEl.style.padding = "4px 0";
    this.contextMenuEl.style.fontSize = "12px";

    this.container.innerHTML = "";
    this.container.style.position = "relative";
    this.container.appendChild(this.canvas);
    this.container.appendChild(this.editorInput);
    this.container.appendChild(this.contextMenuEl);

    const context = this.canvas.getContext("2d");
    if (!context) throw new Error("Could not acquire 2D canvas context for Calc");
    this.ctx = context;

    this.attachEvents();
    this.attachEditorEvents();
    this.resize();
    this.render();
  }

  public setOnCellSelected(cb: (coord: string, value: string) => void) {
    this.onCellSelected = cb;
  }

  public setOnCellEditing(cb: (coord: string, value: string) => void) {
    this.onCellEditing = cb;
  }

  public setOnCellModified(cb: (coord: string, value: string) => void) {
    this.onCellModified = cb;
  }

  public getState(): CalcState {
    return this.state;
  }

  public getEditorInput(): HTMLInputElement {
    return this.editorInput;
  }

  public isCellEditing(): boolean {
    return this.isEditing;
  }

  public resize() {
    const rect = this.container.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    this.canvas.width = Math.max(rect.width, 300) * dpr;
    this.canvas.height = Math.max(rect.height, 200) * dpr;
    this.ctx.scale(dpr, dpr);
    this.render();
  }

  public selectCell(coord: string, extend = false, multi = false) {
    if (this.isEditing) {
      this.commitEditing("none");
    }
    this.state.selectCell(coord, extend, multi);
    this.render();
    if (this.onCellSelected) {
      this.onCellSelected(coord, this.state.formulaBarText);
    }
  }

  public startEditing(initialChar?: string) {
    if (this.isEditing) return;

    this.closeContextMenu();
    const cr = this.state.coordToColRow(this.state.selectedCell);
    const x = this.headerWidth + this.state.getColOffset(cr.col) - this.scrollX;
    const y = this.headerHeight + this.state.getRowOffset(cr.row) - this.scrollY;
    const w = this.state.getColWidth(cr.col);
    const h = this.state.getRowHeight(cr.row);

    this.isEditing = true;
    this.editorInput.style.display = "block";
    this.editorInput.style.left = `${x}px`;
    this.editorInput.style.top = `${y}px`;
    this.editorInput.style.width = `${w + 1}px`;
    this.editorInput.style.height = `${h + 1}px`;

    if (initialChar !== undefined) {
      this.editorInput.value = initialChar;
    } else {
      const cell = this.state.cells.get(this.state.selectedCell);
      this.editorInput.value = cell?.raw || "";
    }

    this.editorInput.focus();
    if (initialChar === undefined) {
      this.editorInput.select();
    }

    if (this.onCellEditing) {
      this.onCellEditing(this.state.selectedCell, this.editorInput.value);
    }
  }

  public setEditingValue(val: string) {
    if (this.isEditing) {
      this.editorInput.value = val;
    } else {
      this.startEditing(val);
    }
  }

  public commitEditing(nav: "down" | "up" | "right" | "left" | "none" = "down") {
    if (!this.isEditing) return;

    const val = this.editorInput.value;
    const currentCoord = this.state.selectedCell;

    this.isEditing = false;
    this.editorInput.style.display = "none";

    this.state.commitFormula(val);

    if (nav !== "none") {
      const cr = this.state.coordToColRow(currentCoord);
      if (nav === "down") cr.row++;
      else if (nav === "up" && cr.row > 0) cr.row--;
      else if (nav === "right") cr.col++;
      else if (nav === "left" && cr.col > 0) cr.col--;

      const nextCoord = `${this.state.colIndexToLetter(cr.col)}${cr.row + 1}`;
      this.selectCell(nextCoord);
    } else {
      this.render();
      if (this.onCellSelected) {
        this.onCellSelected(currentCoord, this.state.formulaBarText);
      }
    }

    if (this.onCellModified) {
      this.onCellModified(currentCoord, val);
    }

    this.canvas.focus();
  }

  public cancelEditing() {
    if (!this.isEditing) return;

    this.isEditing = false;
    this.editorInput.style.display = "none";
    const cell = this.state.cells.get(this.state.selectedCell);
    this.state.formulaBarText = cell?.raw || "";

    if (this.onCellSelected) {
      this.onCellSelected(this.state.selectedCell, this.state.formulaBarText);
    }

    this.render();
    this.canvas.focus();
  }

  private attachEditorEvents() {
    this.editorInput.addEventListener("input", () => {
      const val = this.editorInput.value;
      this.state.formulaBarText = val;
      if (this.onCellEditing) {
        this.onCellEditing(this.state.selectedCell, val);
      }
      this.state.updateFormulaReferences();
      this.render();
    });

    this.editorInput.addEventListener("keydown", (e: KeyboardEvent) => {
      if (e.key === "Enter") {
        e.preventDefault();
        this.commitEditing(e.shiftKey ? "up" : "down");
      } else if (e.key === "Tab") {
        e.preventDefault();
        this.commitEditing(e.shiftKey ? "left" : "right");
      } else if (e.key === "Escape") {
        e.preventDefault();
        this.cancelEditing();
      }
    });

    this.editorInput.addEventListener("blur", () => {
      if (this.isEditing) {
        this.commitEditing("none");
      }
    });
  }

  private isNearFillHandle(x: number, y: number): boolean {
    const primary = this.state.selectedRanges[this.state.selectedRanges.length - 1];
    if (!primary) return false;

    const maxCol = Math.max(primary.minCol, primary.maxCol);
    const maxRow = Math.max(primary.minRow, primary.maxRow);

    const hx = this.headerWidth + this.state.getColOffset(maxCol + 1) - this.scrollX;
    const hy = this.headerHeight + this.state.getRowOffset(maxRow + 1) - this.scrollY;

    return Math.abs(x - hx) <= 6 && Math.abs(y - hy) <= 6;
  }

  private findColDivider(x: number, y: number): number | null {
    if (y > this.headerHeight) return null;
    let accumulatedX = this.headerWidth - this.scrollX;
    for (let c = 0; c < 50; c++) {
      accumulatedX += this.state.getColWidth(c);
      if (Math.abs(x - accumulatedX) <= 4) {
        return c;
      }
      if (accumulatedX > this.canvas.width + 100) break;
    }
    return null;
  }

  private findRowDivider(x: number, y: number): number | null {
    if (x > this.headerWidth) return null;
    let accumulatedY = this.headerHeight - this.scrollY;
    for (let r = 0; r < 100; r++) {
      accumulatedY += this.state.getRowHeight(r);
      if (Math.abs(y - accumulatedY) <= 4) {
        return r;
      }
      if (accumulatedY > this.canvas.height + 100) break;
    }
    return null;
  }

  private getColAtX(x: number): number {
    let accumulatedX = this.headerWidth - this.scrollX;
    for (let c = 0; c < 100; c++) {
      const w = this.state.getColWidth(c);
      if (x >= accumulatedX && x < accumulatedX + w) {
        return c;
      }
      accumulatedX += w;
    }
    return 0;
  }

  private getRowAtY(y: number): number {
    let accumulatedY = this.headerHeight - this.scrollY;
    for (let r = 0; r < 200; r++) {
      const h = this.state.getRowHeight(r);
      if (y >= accumulatedY && y < accumulatedY + h) {
        return r;
      }
      accumulatedY += h;
    }
    return 0;
  }

  private closeContextMenu() {
    this.contextMenuEl.style.display = "none";
  }

  private showContextMenu(x: number, y: number, items: { label: string; action: () => void }[]) {
    this.contextMenuEl.innerHTML = "";
    items.forEach((item) => {
      const btn = document.createElement("div");
      btn.className = "calc-context-item";
      btn.textContent = item.label;
      btn.style.padding = "6px 14px";
      btn.style.cursor = "pointer";
      btn.style.whiteSpace = "nowrap";
      btn.onmouseenter = () => (btn.style.backgroundColor = "#f3f4f6");
      btn.onmouseleave = () => (btn.style.backgroundColor = "transparent");
      btn.onclick = () => {
        this.closeContextMenu();
        item.action();
      };
      this.contextMenuEl.appendChild(btn);
    });
    this.contextMenuEl.style.left = `${x}px`;
    this.contextMenuEl.style.top = `${y}px`;
    this.contextMenuEl.style.display = "block";
  }

  private attachEvents() {
    window.addEventListener("resize", () => this.resize());
    window.addEventListener("click", () => this.closeContextMenu());

    this.canvas.addEventListener("contextmenu", (e: MouseEvent) => {
      e.preventDefault();
      const rect = this.canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;

      if (y <= this.headerHeight && x > this.headerWidth) {
        // Column header right-click
        const col = this.getColAtX(x);
        this.showContextMenu(x, y, [
          {
            label: "Insert Column Left",
            action: () => {
              this.state.insertColumn(col);
              this.render();
              this.onCellModified?.(this.state.selectedCell, "insertColumn");
            },
          },
          {
            label: "Delete Column",
            action: () => {
              this.state.deleteColumn(col);
              this.render();
              this.onCellModified?.(this.state.selectedCell, "deleteColumn");
            },
          },
          {
            label: "Clear Column",
            action: () => {
              this.state.clearColumn(col);
              this.render();
              this.onCellModified?.(this.state.selectedCell, "clearColumn");
            },
          },
        ]);
      } else if (x <= this.headerWidth && y > this.headerHeight) {
        // Row header right-click
        const row = this.getRowAtY(y);
        this.showContextMenu(x, y, [
          {
            label: "Insert Row Above",
            action: () => {
              this.state.insertRow(row);
              this.render();
              this.onCellModified?.(this.state.selectedCell, "insertRow");
            },
          },
          {
            label: "Delete Row",
            action: () => {
              this.state.deleteRow(row);
              this.render();
              this.onCellModified?.(this.state.selectedCell, "deleteRow");
            },
          },
          {
            label: "Clear Row",
            action: () => {
              this.state.clearRow(row);
              this.render();
              this.onCellModified?.(this.state.selectedCell, "clearRow");
            },
          },
        ]);
      } else if (x > this.headerWidth && y > this.headerHeight) {
        // Cell right-click
        const col = this.getColAtX(x);
        const row = this.getRowAtY(y);
        const coord = `${this.state.colIndexToLetter(col)}${row + 1}`;
        this.selectCell(coord);
        this.showContextMenu(x, y, [
          {
            label: "Insert Row Above",
            action: () => {
              this.state.insertRow(row);
              this.render();
              this.onCellModified?.(coord, "insertRow");
            },
          },
          {
            label: "Insert Column Left",
            action: () => {
              this.state.insertColumn(col);
              this.render();
              this.onCellModified?.(coord, "insertColumn");
            },
          },
          {
            label: "Delete Row",
            action: () => {
              this.state.deleteRow(row);
              this.render();
              this.onCellModified?.(coord, "deleteRow");
            },
          },
          {
            label: "Delete Column",
            action: () => {
              this.state.deleteColumn(col);
              this.render();
              this.onCellModified?.(coord, "deleteColumn");
            },
          },
          {
            label: "Clear Contents",
            action: () => {
              this.state.commitFormula("");
              this.render();
              this.onCellModified?.(coord, "");
            },
          },
        ]);
      }
    });

    this.canvas.addEventListener("dblclick", (e: MouseEvent) => {
      const rect = this.canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;

      if (x > this.headerWidth && y > this.headerHeight) {
        this.startEditing();
      }
    });

    this.canvas.addEventListener("mousedown", (e: MouseEvent) => {
      if (e.button !== 0) return; // Only left click
      this.closeContextMenu();

      const rect = this.canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;

      // Check Column Divider Resize
      const colDiv = this.findColDivider(x, y);
      if (colDiv !== null) {
        this.isResizingCol = true;
        this.resizingColIndex = colDiv;
        this.colResizeStartX = e.clientX;
        this.colResizeInitialWidth = this.state.getColWidth(colDiv);
        return;
      }

      // Check Row Divider Resize
      const rowDiv = this.findRowDivider(x, y);
      if (rowDiv !== null) {
        this.isResizingRow = true;
        this.resizingRowIndex = rowDiv;
        this.rowResizeStartY = e.clientY;
        this.rowResizeInitialHeight = this.state.getRowHeight(rowDiv);
        return;
      }

      // Check Fill Handle
      if (this.isNearFillHandle(x, y)) {
        this.isDraggingFillHandle = true;
        return;
      }

      // Cell Selection
      if (x > this.headerWidth && y > this.headerHeight) {
        this.isSelecting = true;
        const colIdx = this.getColAtX(x);
        const rowIdx = this.getRowAtY(y);
        const coord = `${this.state.colIndexToLetter(colIdx)}${rowIdx + 1}`;
        this.selectCell(coord, e.shiftKey, e.ctrlKey || e.metaKey);
      }
    });

    this.canvas.addEventListener("mousemove", (e: MouseEvent) => {
      const rect = this.canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;

      if (this.isResizingCol) {
        const delta = e.clientX - this.colResizeStartX;
        this.state.setColWidth(this.resizingColIndex, this.colResizeInitialWidth + delta);
        this.render();
        return;
      }

      if (this.isResizingRow) {
        const delta = e.clientY - this.rowResizeStartY;
        this.state.setRowHeight(this.resizingRowIndex, this.rowResizeInitialHeight + delta);
        this.render();
        return;
      }

      // Cursor adjustments
      if (this.findColDivider(x, y) !== null) {
        this.canvas.style.cursor = "col-resize";
      } else if (this.findRowDivider(x, y) !== null) {
        this.canvas.style.cursor = "row-resize";
      } else if (this.isNearFillHandle(x, y)) {
        this.canvas.style.cursor = "crosshair";
      } else if (x <= this.headerWidth || y <= this.headerHeight) {
        this.canvas.style.cursor = "default";
      } else {
        this.canvas.style.cursor = "cell";
      }

      if (this.isDraggingFillHandle) {
        const primary = this.state.selectedRanges[this.state.selectedRanges.length - 1];
        if (!primary) return;
        const colIdx = Math.max(primary.minCol, this.getColAtX(x));
        const rowIdx = Math.max(primary.minRow, this.getRowAtY(y));

        this.state.autoFillPreview = {
          minRow: primary.minRow,
          maxRow: rowIdx,
          minCol: primary.minCol,
          maxCol: colIdx,
        };
        this.render();
      } else if (this.isSelecting && (e.buttons & 1) === 1) {
        const colIdx = this.getColAtX(x);
        const rowIdx = this.getRowAtY(y);
        const coord = `${this.state.colIndexToLetter(colIdx)}${rowIdx + 1}`;
        this.selectCell(coord, true, false);
      }
    });

    window.addEventListener("mouseup", () => {
      if (this.isResizingCol) {
        this.isResizingCol = false;
        this.onCellModified?.(this.state.selectedCell, "resizeColumn");
      }
      if (this.isResizingRow) {
        this.isResizingRow = false;
        this.onCellModified?.(this.state.selectedCell, "resizeRow");
      }

      if (this.isDraggingFillHandle && this.state.autoFillPreview) {
        const primary = this.state.selectedRanges[this.state.selectedRanges.length - 1];
        if (primary) {
          this.state.applyAutoFill(primary, this.state.autoFillPreview);
        }
        this.state.autoFillPreview = null;
        this.isDraggingFillHandle = false;
        this.render();
        this.onCellModified?.(this.state.selectedCell, "autoFill");
      }
      this.isSelecting = false;
    });

    this.canvas.addEventListener("keydown", (e: KeyboardEvent) => {
      if (this.isEditing) return;

      if (e.key === "F2") {
        e.preventDefault();
        this.startEditing();
        return;
      }

      if (e.key === "Enter") {
        e.preventDefault();
        this.startEditing();
        return;
      }

      if (e.key === "Delete" || e.key === "Backspace") {
        e.preventDefault();
        this.state.commitFormula("");
        this.render();
        this.onCellModified?.(this.state.selectedCell, "");
        if (this.onCellSelected) {
          this.onCellSelected(this.state.selectedCell, "");
        }
        return;
      }

      // Printable single characters trigger active in-cell editing
      if (e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey) {
        e.preventDefault();
        this.startEditing(e.key);
        return;
      }

      // Directional Navigation
      const cur = this.state.coordToColRow(this.state.selectedCell);
      let changed = false;

      if (e.key === "ArrowDown") {
        cur.row++;
        changed = true;
      } else if (e.key === "ArrowUp") {
        if (cur.row > 0) cur.row--;
        changed = true;
      } else if (e.key === "ArrowRight") {
        cur.col++;
        changed = true;
      } else if (e.key === "ArrowLeft") {
        if (cur.col > 0) cur.col--;
        changed = true;
      }

      if (changed) {
        e.preventDefault();
        const coord = `${this.state.colIndexToLetter(cur.col)}${cur.row + 1}`;
        this.selectCell(coord, e.shiftKey, false);
      }
    });
  }

  public render() {
    const rect = this.canvas.getBoundingClientRect();
    const w = rect.width;
    const h = rect.height;
    if (w <= 0 || h <= 0) return;

    const ctx = this.ctx;
    ctx.clearRect(0, 0, w, h);

    // 1. Grid Background
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(0, 0, w, h);

    // Determine visible cols and rows
    let colsInView = 0;
    let accumulatedX = this.headerWidth - this.scrollX;
    while (accumulatedX < w + 100 && colsInView < 100) {
      accumulatedX += this.state.getColWidth(colsInView);
      colsInView++;
    }

    let rowsInView = 0;
    let accumulatedY = this.headerHeight - this.scrollY;
    while (accumulatedY < h + 100 && rowsInView < 200) {
      accumulatedY += this.state.getRowHeight(rowsInView);
      rowsInView++;
    }

    // 2. Grid lines and Cells
    ctx.font = "12px Calibri, sans-serif";
    for (let r = 0; r < rowsInView; r++) {
      const y = this.headerHeight + this.state.getRowOffset(r) - this.scrollY;
      const rowHeight = this.state.getRowHeight(r);
      const rowNum = r + 1;

      for (let c = 0; c < colsInView; c++) {
        const x = this.headerWidth + this.state.getColOffset(c) - this.scrollX;
        const colWidth = this.state.getColWidth(c);
        const colLetter = this.state.colIndexToLetter(c);
        const coord = `${colLetter}${rowNum}`;

        // Cell border
        ctx.strokeStyle = "#e1dfdd";
        ctx.lineWidth = 1;
        ctx.strokeRect(x, y, colWidth, rowHeight);

        // Cell content
        const cellData = this.state.cells.get(coord);
        if (cellData && (!this.isEditing || coord !== this.state.selectedCell)) {
          ctx.save();
          ctx.beginPath();
          ctx.rect(x + 2, y, colWidth - 4, rowHeight);
          ctx.clip();

          ctx.fillStyle = cellData.color || "#1b1b1b";
          if (cellData.bold) ctx.font = "bold 12px Calibri, sans-serif";
          else ctx.font = "12px Calibri, sans-serif";

          const text = String(cellData.computed !== undefined ? cellData.computed : cellData.raw);
          const isNum = typeof cellData.computed === "number" || (!isNaN(Number(text)) && text.trim() !== "");

          if (isNum) {
            ctx.textAlign = "right";
            ctx.fillText(text, x + colWidth - 6, y + rowHeight / 2 + 4);
          } else {
            ctx.textAlign = "left";
            ctx.fillText(text, x + 6, y + rowHeight / 2 + 4);
          }
          ctx.restore();
        }
      }
    }

    // 3. Render Formula References Highlights (when editing formula)
    this.state.formulaReferences.forEach((ref) => {
      const minCol = Math.min(ref.range.minCol, ref.range.maxCol);
      const maxCol = Math.max(ref.range.minCol, ref.range.maxCol);
      const minRow = Math.min(ref.range.minRow, ref.range.maxRow);
      const maxRow = Math.max(ref.range.minRow, ref.range.maxRow);

      const rx = this.headerWidth + this.state.getColOffset(minCol) - this.scrollX;
      const ry = this.headerHeight + this.state.getRowOffset(minRow) - this.scrollY;
      let rw = 0;
      for (let c = minCol; c <= maxCol; c++) rw += this.state.getColWidth(c);
      let rh = 0;
      for (let r = minRow; r <= maxRow; r++) rh += this.state.getRowHeight(r);

      ctx.save();
      ctx.strokeStyle = ref.color;
      ctx.lineWidth = 2;
      ctx.fillStyle = ref.color + "18";
      ctx.fillRect(rx, ry, rw, rh);
      ctx.strokeRect(rx, ry, rw, rh);
      ctx.restore();
    });

    // 4. Render Multi-Range Selection Boxes
    this.state.selectedRanges.forEach((range, idx) => {
      const minCol = Math.min(range.minCol, range.maxCol);
      const maxCol = Math.max(range.minCol, range.maxCol);
      const minRow = Math.min(range.minRow, range.maxRow);
      const maxRow = Math.max(range.minRow, range.maxRow);

      const selX = this.headerWidth + this.state.getColOffset(minCol) - this.scrollX;
      const selY = this.headerHeight + this.state.getRowOffset(minRow) - this.scrollY;
      let selW = 0;
      for (let c = minCol; c <= maxCol; c++) selW += this.state.getColWidth(c);
      let selH = 0;
      for (let r = minRow; r <= maxRow; r++) selH += this.state.getRowHeight(r);

      ctx.fillStyle = "rgba(16, 124, 65, 0.12)";
      ctx.fillRect(selX, selY, selW, selH);

      ctx.strokeStyle = "#107c41"; // Excel forest green
      ctx.lineWidth = 2;
      ctx.strokeRect(selX, selY, selW, selH);

      // Fill handle on active selection
      if (idx === this.state.selectedRanges.length - 1) {
        ctx.fillStyle = "#107c41";
        ctx.fillRect(selX + selW - 3, selY + selH - 3, 6, 6);
      }
    });

    // 5. Render Auto-Fill Preview Box
    if (this.state.autoFillPreview) {
      const p = this.state.autoFillPreview;
      const minCol = Math.min(p.minCol, p.maxCol);
      const maxCol = Math.max(p.minCol, p.maxCol);
      const minRow = Math.min(p.minRow, p.maxRow);
      const maxRow = Math.max(p.minRow, p.maxRow);

      const px = this.headerWidth + this.state.getColOffset(minCol) - this.scrollX;
      const py = this.headerHeight + this.state.getRowOffset(minRow) - this.scrollY;
      let pw = 0;
      for (let c = minCol; c <= maxCol; c++) pw += this.state.getColWidth(c);
      let ph = 0;
      for (let r = minRow; r <= maxRow; r++) ph += this.state.getRowHeight(r);

      ctx.save();
      ctx.strokeStyle = "#666666";
      ctx.lineWidth = 1.5;
      ctx.setLineDash([4, 4]);
      ctx.strokeRect(px, py, pw, ph);
      ctx.restore();
    }

    // 6. Column Headers
    ctx.fillStyle = "#f3f2f1";
    ctx.fillRect(0, 0, w, this.headerHeight);

    const primaryRange = this.state.selectedRanges[this.state.selectedRanges.length - 1];
    const activeCol = primaryRange ? primaryRange.maxCol : 0;
    const activeRow = primaryRange ? primaryRange.maxRow : 0;

    for (let c = 0; c < colsInView; c++) {
      const x = this.headerWidth + this.state.getColOffset(c) - this.scrollX;
      const colWidth = this.state.getColWidth(c);
      const isColActive = c === activeCol;

      if (isColActive) {
        ctx.fillStyle = "#caead8";
        ctx.fillRect(x, 0, colWidth, this.headerHeight);
      }

      ctx.strokeStyle = "#d2d0ce";
      ctx.lineWidth = 1;
      ctx.strokeRect(x, 0, colWidth, this.headerHeight);

      ctx.fillStyle = isColActive ? "#107c41" : "#494745";
      ctx.font = isColActive ? "bold 11px Calibri, sans-serif" : "11px Calibri, sans-serif";
      ctx.textAlign = "center";
      ctx.fillText(this.state.colIndexToLetter(c), x + colWidth / 2, 17);
    }

    // 7. Row Headers
    ctx.fillStyle = "#f3f2f1";
    ctx.fillRect(0, 0, this.headerWidth, h);

    for (let r = 0; r < rowsInView; r++) {
      const y = this.headerHeight + this.state.getRowOffset(r) - this.scrollY;
      const rowHeight = this.state.getRowHeight(r);
      const isRowActive = r === activeRow;

      if (isRowActive) {
        ctx.fillStyle = "#caead8";
        ctx.fillRect(0, y, this.headerWidth, rowHeight);
      }

      ctx.strokeStyle = "#d2d0ce";
      ctx.lineWidth = 1;
      ctx.strokeRect(0, y, this.headerWidth, rowHeight);

      ctx.fillStyle = isRowActive ? "#107c41" : "#494745";
      ctx.font = isRowActive ? "bold 11px Calibri, sans-serif" : "11px Calibri, sans-serif";
      ctx.textAlign = "center";
      ctx.fillText(String(r + 1), this.headerWidth / 2, y + rowHeight / 2 + 4);
    }

    // 8. Top-left Select-all box
    ctx.fillStyle = "#e1dfdd";
    ctx.fillRect(0, 0, this.headerWidth, this.headerHeight);
    ctx.strokeStyle = "#c8c6c4";
    ctx.strokeRect(0, 0, this.headerWidth, this.headerHeight);
    ctx.beginPath();
    ctx.moveTo(this.headerWidth - 10, this.headerHeight - 2);
    ctx.lineTo(this.headerWidth - 2, this.headerHeight - 10);
    ctx.lineTo(this.headerWidth - 2, this.headerHeight - 2);
    ctx.fillStyle = "#979593";
    ctx.fill();
  }
}
