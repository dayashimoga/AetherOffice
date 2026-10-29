import { CalcState, calcState } from "../../state/calcState";

export class CalcGridCanvas {
  private container: HTMLElement;
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private state: CalcState;
  private colWidth = 96;
  private rowHeight = 24;
  private headerWidth = 50;
  private headerHeight = 26;
  private scrollX = 0;
  private scrollY = 0;
  private isDraggingFillHandle = false;
  private isSelecting = false;
  private onCellSelected?: (coord: string, value: string) => void;

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

    this.container.innerHTML = "";
    this.container.appendChild(this.canvas);

    const context = this.canvas.getContext("2d");
    if (!context) throw new Error("Could not acquire 2D canvas context for Calc");
    this.ctx = context;

    this.attachEvents();
    this.resize();
    this.render();
  }

  public setOnCellSelected(cb: (coord: string, value: string) => void) {
    this.onCellSelected = cb;
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
    this.state.selectCell(coord, extend, multi);
    this.render();
    if (this.onCellSelected) {
      this.onCellSelected(coord, this.state.formulaBarText);
    }
  }

  private isNearFillHandle(x: number, y: number): boolean {
    const primary = this.state.selectedRanges[this.state.selectedRanges.length - 1];
    if (!primary) return false;

    const maxCol = Math.max(primary.minCol, primary.maxCol);
    const maxRow = Math.max(primary.minRow, primary.maxRow);

    const hx = this.headerWidth + (maxCol + 1) * this.colWidth - this.scrollX;
    const hy = this.headerHeight + (maxRow + 1) * this.rowHeight - this.scrollY;

    return Math.abs(x - hx) <= 6 && Math.abs(y - hy) <= 6;
  }

  private attachEvents() {
    window.addEventListener("resize", () => this.resize());

    this.canvas.addEventListener("mousedown", (e: MouseEvent) => {
      const rect = this.canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;

      if (this.isNearFillHandle(x, y)) {
        this.isDraggingFillHandle = true;
        return;
      }

      if (x > this.headerWidth && y > this.headerHeight) {
        this.isSelecting = true;
        const colIdx = Math.floor((x - this.headerWidth + this.scrollX) / this.colWidth);
        const rowIdx = Math.floor((y - this.headerHeight + this.scrollY) / this.rowHeight);
        const coord = `${this.state.colIndexToLetter(colIdx)}${rowIdx + 1}`;

        this.selectCell(coord, e.shiftKey, e.ctrlKey || e.metaKey);
      }
    });

    this.canvas.addEventListener("mousemove", (e: MouseEvent) => {
      const rect = this.canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;

      if (this.isNearFillHandle(x, y)) {
        this.canvas.style.cursor = "crosshair";
      } else {
        this.canvas.style.cursor = "cell";
      }

      if (this.isDraggingFillHandle) {
        const primary = this.state.selectedRanges[this.state.selectedRanges.length - 1];
        if (!primary) return;
        const colIdx = Math.max(primary.minCol, Math.floor((x - this.headerWidth + this.scrollX) / this.colWidth));
        const rowIdx = Math.max(primary.minRow, Math.floor((y - this.headerHeight + this.scrollY) / this.rowHeight));

        this.state.autoFillPreview = {
          minRow: primary.minRow,
          maxRow: rowIdx,
          minCol: primary.minCol,
          maxCol: colIdx,
        };
        this.render();
      } else if (this.isSelecting && (e.buttons & 1) === 1) {
        const colIdx = Math.floor((x - this.headerWidth + this.scrollX) / this.colWidth);
        const rowIdx = Math.floor((y - this.headerHeight + this.scrollY) / this.rowHeight);
        const coord = `${this.state.colIndexToLetter(colIdx)}${rowIdx + 1}`;
        this.selectCell(coord, true, false);
      }
    });

    window.addEventListener("mouseup", () => {
      if (this.isDraggingFillHandle && this.state.autoFillPreview) {
        const primary = this.state.selectedRanges[this.state.selectedRanges.length - 1];
        if (primary) {
          this.state.applyAutoFill(primary, this.state.autoFillPreview);
        }
        this.state.autoFillPreview = null;
        this.isDraggingFillHandle = false;
        this.render();
      }
      this.isSelecting = false;
    });

    this.canvas.addEventListener("keydown", (e: KeyboardEvent) => {
      const cur = this.state.coordToColRow(this.state.selectedCell);
      let changed = false;

      if (e.key === "ArrowDown") { cur.row++; changed = true; }
      else if (e.key === "ArrowUp") { if (cur.row > 0) cur.row--; changed = true; }
      else if (e.key === "ArrowRight") { cur.col++; changed = true; }
      else if (e.key === "ArrowLeft") { if (cur.col > 0) cur.col--; changed = true; }

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

    const colsInView = Math.ceil((w - this.headerWidth) / this.colWidth) + 1;
    const rowsInView = Math.ceil((h - this.headerHeight) / this.rowHeight) + 1;

    // 2. Grid lines and Cells
    ctx.font = "12px Calibri, sans-serif";
    for (let r = 0; r < rowsInView; r++) {
      const y = this.headerHeight + r * this.rowHeight;
      const rowNum = r + 1;

      for (let c = 0; c < colsInView; c++) {
        const x = this.headerWidth + c * this.colWidth;
        const colLetter = this.state.colIndexToLetter(c);
        const coord = `${colLetter}${rowNum}`;

        // Cell border
        ctx.strokeStyle = "#e1dfdd";
        ctx.lineWidth = 1;
        ctx.strokeRect(x, y, this.colWidth, this.rowHeight);

        // Cell content
        const cellData = this.state.cells.get(coord);
        if (cellData) {
          ctx.save();
          ctx.beginPath();
          ctx.rect(x + 2, y, this.colWidth - 4, this.rowHeight);
          ctx.clip();

          ctx.fillStyle = "#1b1b1b";
          if (cellData.bold) ctx.font = "bold 12px Calibri, sans-serif";
          else ctx.font = "12px Calibri, sans-serif";

          const text = String(cellData.computed !== undefined ? cellData.computed : cellData.raw);
          const isNum = typeof cellData.computed === "number" || (!isNaN(Number(text)) && text.trim() !== "");

          if (isNum) {
            ctx.textAlign = "right";
            ctx.fillText(text, x + this.colWidth - 8, y + 16);
          } else {
            ctx.textAlign = "left";
            ctx.fillText(text, x + 6, y + 16);
          }
          ctx.restore();
        }
      }
    }

    // 3. Render Formula References Highlights (when editing formula)
    this.state.formulaReferences.forEach((ref) => {
      const rx = this.headerWidth + Math.min(ref.range.minCol, ref.range.maxCol) * this.colWidth - this.scrollX;
      const ry = this.headerHeight + Math.min(ref.range.minRow, ref.range.maxRow) * this.rowHeight - this.scrollY;
      const rw = (Math.abs(ref.range.maxCol - ref.range.minCol) + 1) * this.colWidth;
      const rh = (Math.abs(ref.range.maxRow - ref.range.minRow) + 1) * this.rowHeight;

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

      const selX = this.headerWidth + minCol * this.colWidth - this.scrollX;
      const selY = this.headerHeight + minRow * this.rowHeight - this.scrollY;
      const selW = (maxCol - minCol + 1) * this.colWidth;
      const selH = (maxRow - minRow + 1) * this.rowHeight;

      ctx.fillStyle = "rgba(16, 124, 65, 0.12)";
      ctx.fillRect(selX, selY, selW, selH);

      ctx.strokeStyle = "#107c41"; // Excel forest green
      ctx.lineWidth = 2;
      ctx.strokeRect(selX, selY, selW, selH);

      // Fill handle on last (active) range
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

      const px = this.headerWidth + minCol * this.colWidth - this.scrollX;
      const py = this.headerHeight + minRow * this.rowHeight - this.scrollY;
      const pw = (maxCol - minCol + 1) * this.colWidth;
      const ph = (maxRow - minRow + 1) * this.rowHeight;

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
      const x = this.headerWidth + c * this.colWidth;
      const isColActive = c === activeCol;

      if (isColActive) {
        ctx.fillStyle = "#caead8";
        ctx.fillRect(x, 0, this.colWidth, this.headerHeight);
      }

      ctx.strokeStyle = "#d2d0ce";
      ctx.lineWidth = 1;
      ctx.strokeRect(x, 0, this.colWidth, this.headerHeight);

      ctx.fillStyle = isColActive ? "#107c41" : "#494745";
      ctx.font = isColActive ? "bold 11px Calibri, sans-serif" : "11px Calibri, sans-serif";
      ctx.textAlign = "center";
      ctx.fillText(this.state.colIndexToLetter(c), x + this.colWidth / 2, 17);
    }

    // 7. Row Headers
    ctx.fillStyle = "#f3f2f1";
    ctx.fillRect(0, 0, this.headerWidth, h);

    for (let r = 0; r < rowsInView; r++) {
      const y = this.headerHeight + r * this.rowHeight;
      const isRowActive = r === activeRow;

      if (isRowActive) {
        ctx.fillStyle = "#caead8";
        ctx.fillRect(0, y, this.headerWidth, this.rowHeight);
      }

      ctx.strokeStyle = "#d2d0ce";
      ctx.lineWidth = 1;
      ctx.strokeRect(0, y, this.headerWidth, this.rowHeight);

      ctx.fillStyle = isRowActive ? "#107c41" : "#494745";
      ctx.font = isRowActive ? "bold 11px Calibri, sans-serif" : "11px Calibri, sans-serif";
      ctx.textAlign = "center";
      ctx.fillText(String(r + 1), this.headerWidth / 2, y + 16);
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
