export interface CellData {
  raw: string;
  computed: string | number;
  format?: string;
  bold?: boolean;
}

export interface GridRange {
  minRow: number;
  maxRow: number;
  minCol: number;
  maxCol: number;
}

export class CalcState {
  cells: Map<string, CellData> = new Map([
    ["A1", { raw: "Revenue", computed: "Revenue", bold: true }],
    ["A2", { raw: "15000", computed: 15000 }],
    ["A3", { raw: "25000", computed: 25000 }],
    ["A4", { raw: "=SUM(A2:A3)", computed: 40000, bold: true }],
    ["B1", { raw: "Q1", computed: "Q1", bold: true }],
    ["B2", { raw: "12000", computed: 12000 }],
    ["B3", { raw: "18000", computed: 18000 }],
    ["B4", { raw: "=SUM(B2:B3)", computed: 30000, bold: true }],
  ]);

  selectedCell: string = "A1";
  selectedRanges: GridRange[] = [{ minRow: 0, maxRow: 0, minCol: 0, maxCol: 0 }];
  formulaBarText: string = "Revenue";
  autoFillPreview: GridRange | null = null;
  formulaReferences: { range: GridRange; color: string }[] = [];

  selectCell(coord: string, extend = false, multi = false) {
    const parsed = this.coordToColRow(coord);
    this.selectedCell = coord;
    this.formulaBarText = this.cells.get(coord)?.raw || "";

    if (extend && this.selectedRanges.length > 0) {
      const cur = this.selectedRanges[this.selectedRanges.length - 1];
      cur.maxRow = parsed.row;
      cur.maxCol = parsed.col;
    } else if (multi) {
      this.selectedRanges.push({
        minRow: parsed.row,
        maxRow: parsed.row,
        minCol: parsed.col,
        maxCol: parsed.col,
      });
    } else {
      this.selectedRanges = [{
        minRow: parsed.row,
        maxRow: parsed.row,
        minCol: parsed.col,
        maxCol: parsed.col,
      }];
    }

    this.updateFormulaReferences();
  }

  commitFormula(val: string) {
    let computed: string | number = val;
    if (val.startsWith("=")) {
      computed = this.evaluateFormula(val);
    }
    this.cells.set(this.selectedCell, { raw: val, computed });
    this.formulaBarText = val;
    this.updateFormulaReferences();
  }

  evaluateFormula(expr: string): string | number {
    const upper = expr.toUpperCase().trim();
    if (upper.startsWith("=SUM(") && upper.endsWith(")")) {
      const inner = upper.slice(5, -1);
      const [start, end] = inner.split(":");
      if (start && end) {
        const c1 = this.coordToColRow(start);
        const c2 = this.coordToColRow(end);
        let sum = 0;
        for (let r = Math.min(c1.row, c2.row); r <= Math.max(c1.row, c2.row); r++) {
          for (let c = Math.min(c1.col, c2.col); c <= Math.max(c1.col, c2.col); c++) {
            const coord = `${this.colIndexToLetter(c)}${r + 1}`;
            const cell = this.cells.get(coord);
            if (cell) {
              const num = Number(cell.computed);
              if (!isNaN(num)) sum += num;
            }
          }
        }
        return sum;
      }
    }
    return expr;
  }

  updateFormulaReferences() {
    this.formulaReferences = [];
    const text = this.formulaBarText.trim();
    if (!text.startsWith("=")) return;

    const palette = ["#1a73e8", "#1e8e3e", "#9334e6", "#e37400", "#d93025"];
    const matches = text.matchAll(/(\$?[A-Z]+\$?[0-9]+(?::\$?[A-Z]+\$?[0-9]+)?)/gi);
    let colorIdx = 0;

    for (const match of matches) {
      const refStr = match[1].replace(/\$/g, "").toUpperCase();
      if (refStr.includes(":")) {
        const [s, e] = refStr.split(":");
        const p1 = this.coordToColRow(s);
        const p2 = this.coordToColRow(e);
        this.formulaReferences.push({
          range: {
            minRow: Math.min(p1.row, p2.row),
            maxRow: Math.max(p1.row, p2.row),
            minCol: Math.min(p1.col, p2.col),
            maxCol: Math.max(p1.col, p2.col),
          },
          color: palette[colorIdx % palette.len],
        });
        colorIdx++;
      } else {
        const p = this.coordToColRow(refStr);
        this.formulaReferences.push({
          range: { minRow: p.row, maxRow: p.row, minCol: p.col, maxCol: p.col },
          color: palette[colorIdx % palette.len],
        });
        colorIdx++;
      }
    }
  }

  applyAutoFill(source: GridRange, target: GridRange) {
    const isDown = target.maxRow > source.maxRow;
    const isRight = target.maxCol > source.maxCol;

    if (isDown) {
      for (let c = source.minCol; c <= source.maxCol; c++) {
        const samples: string[] = [];
        for (let r = source.minRow; r <= source.maxRow; r++) {
          const coord = `${this.colIndexToLetter(c)}${r + 1}`;
          samples.push(this.cells.get(coord)?.raw || "");
        }
        const count = target.maxRow - source.maxRow;
        const predicted = this.predictSeries(samples, count);
        for (let i = 0; i < predicted.length; i++) {
          const r = source.maxRow + 1 + i;
          const coord = `${this.colIndexToLetter(c)}${r + 1}`;
          const val = predicted[i];
          const computed = val.startsWith("=") ? this.evaluateFormula(val) : val;
          this.cells.set(coord, { raw: val, computed });
        }
      }
    } else if (isRight) {
      for (let r = source.minRow; r <= source.maxRow; r++) {
        const samples: string[] = [];
        for (let c = source.minCol; c <= source.maxCol; c++) {
          const coord = `${this.colIndexToLetter(c)}${r + 1}`;
          samples.push(this.cells.get(coord)?.raw || "");
        }
        const count = target.maxCol - source.maxCol;
        const predicted = this.predictSeries(samples, count);
        for (let i = 0; i < predicted.length; i++) {
          const c = source.maxCol + 1 + i;
          const coord = `${this.colIndexToLetter(c)}${r + 1}`;
          const val = predicted[i];
          const computed = val.startsWith("=") ? this.evaluateFormula(val) : val;
          this.cells.set(coord, { raw: val, computed });
        }
      }
    }
  }

  predictSeries(samples: string[], count: number): string[] {
    if (samples.length === 0) return [];
    const nums = samples.map((s) => Number(s));
    const allNums = nums.every((n) => !isNaN(n) && typeof n === "number");

    if (allNums) {
      if (nums.length === 1) {
        return Array(count).fill(String(nums[0]));
      }
      const step = (nums[nums.length - 1] - nums[0]) / (nums.length - 1);
      const res: string[] = [];
      for (let k = 1; k <= count; k++) {
        res.push(String(nums[nums.length - 1] + step * k));
      }
      return res;
    }

    // Default: repeat pattern
    const res: string[] = [];
    for (let k = 0; k < count; k++) {
      res.push(samples[k % samples.length]);
    }
    return res;
  }

  colIndexToLetter(idx: number): string {
    let letter = "";
    idx++;
    while (idx > 0) {
      const rem = (idx - 1) % 26;
      letter = String.fromCharCode(65 + rem) + letter;
      idx = Math.floor((idx - 1) / 26);
    }
    return letter;
  }

  coordToColRow(coord: string): { col: number; row: number } {
    const match = coord.match(/^([A-Z]+)([0-9]+)$/);
    if (!match) return { col: 0, row: 0 };
    let col = 0;
    for (let i = 0; i < match[1].length; i++) {
      col = col * 26 + (match[1].charCodeAt(i) - 64);
    }
    return { col: col - 1, row: parseInt(match[2], 10) - 1 };
  }
}

export const calcState = new CalcState();
