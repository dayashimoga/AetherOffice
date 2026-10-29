import { CalcState, calcState } from "../state/calcState";

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

  public selectCell(coord: string) {
    this.state.selectCell(coord);
    this.render();
    if (this.onCellSelected) {
      this.onCellSelected(coord, this.state.formulaBarText);
    }
  }

  private colIndexToLetter(idx: number): string {
    let letter = "";
    idx++;
    while (idx > 0) {
      const rem = (idx - 1) % 26;
      letter = String.fromCharCode(65 + rem) + letter;
      idx = Math.floor((idx - 1) / 26);
    }
    return letter;
  }

  private coordToColRow(coord: string): { col: number; row: number } {
    const match = coord.match(/^([A-Z]+)([0-9]+)$/);
    if (!match) return { col: 0, row: 0 };
    let col = 0;
    for (let i = 0; i < match[1].length; i++) {
      col = col * 26 + (match[1].charCodeAt(i) - 64);
    }
    return { col: col - 1, row: parseInt(match[2], 10) - 1 };
  }

  private attachEvents() {
    window.addEventListener("resize", () => this.resize());

    this.canvas.addEventListener("mousedown", (e: MouseEvent) => {
      const rect = this.canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;

      if (x > this.headerWidth && y > this.headerHeight) {
        const colIdx = Math.floor((x - this.headerWidth + this.scrollX) / this.colWidth);
        const rowIdx = Math.floor((y - this.headerHeight + this.scrollY) / this.rowHeight);
        const coord = `${this.colIndexToLetter(colIdx)}${rowIdx + 1}`;
        this.selectCell(coord);
      }
    });

    this.canvas.addEventListener("keydown", (e: KeyboardEvent) => {
      const cur = this.coordToColRow(this.state.selectedCell);
      let changed = false;

      if (e.key === "ArrowDown") { cur.row++; changed = true; }
      else if (e.key === "ArrowUp") { if (cur.row > 0) cur.row--; changed = true; }
      else if (e.key === "ArrowRight") { cur.col++; changed = true; }
      else if (e.key === "ArrowLeft") { if (cur.col > 0) cur.col--; changed = true; }

      if (changed) {
        e.preventDefault();
        const coord = `${this.colIndexToLetter(cur.col)}${cur.row + 1}`;
        this.selectCell(coord);
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
        const colLetter = this.colIndexToLetter(c);
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

    // 3. Active Cell Selection Box
    const sel = this.coordToColRow(this.state.selectedCell);
    const selX = this.headerWidth + sel.col * this.colWidth;
    const selY = this.headerHeight + sel.row * this.rowHeight;

    ctx.strokeStyle = "#107c41"; // Excel forest green
    ctx.lineWidth = 2;
    ctx.strokeRect(selX, selY, this.colWidth, this.rowHeight);

    // Fill handle (bottom-right handle)
    ctx.fillStyle = "#107c41";
    ctx.fillRect(selX + this.colWidth - 3, selY + this.rowHeight - 3, 6, 6);

    // 4. Column Headers
    ctx.fillStyle = "#f3f2f1";
    ctx.fillRect(0, 0, w, this.headerHeight);

    for (let c = 0; c < colsInView; c++) {
      const x = this.headerWidth + c * this.colWidth;
      const isColActive = c === sel.col;

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
      ctx.fillText(this.colIndexToLetter(c), x + this.colWidth / 2, 17);
    }

    // 5. Row Headers
    ctx.fillStyle = "#f3f2f1";
    ctx.fillRect(0, 0, this.headerWidth, h);

    for (let r = 0; r < rowsInView; r++) {
      const y = this.headerHeight + r * this.rowHeight;
      const isRowActive = r === sel.row;

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

    // 6. Top-left Select-all box
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

