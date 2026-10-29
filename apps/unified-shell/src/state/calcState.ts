export interface CellData {
  raw: string;
  computed: string | number;
  format?: string;
  bold?: boolean;
  italic?: boolean;
  underline?: boolean;
  align?: "left" | "center" | "right";
  color?: string;
  bgColor?: string;
}

export interface GridRange {
  minRow: number;
  maxRow: number;
  minCol: number;
  maxCol: number;
}

export interface SheetData {
  id: string;
  name: string;
  cells: Map<string, CellData>;
  colWidths: Map<number, number>;
  rowHeights: Map<number, number>;
  selectedCell: string;
  selectedRanges: GridRange[];
}

export class CalcState {
  sheets: SheetData[] = [
    {
      id: "sheet-1",
      name: "Sheet1",
      cells: new Map<string, CellData>([
        ["A1", { raw: "Revenue", computed: "Revenue", bold: true }],
        ["A2", { raw: "15000", computed: 15000 }],
        ["A3", { raw: "25000", computed: 25000 }],
        ["A4", { raw: "=SUM(A2:A3)", computed: 40000, bold: true }],
        ["B1", { raw: "Q1", computed: "Q1", bold: true }],
        ["B2", { raw: "12000", computed: 12000 }],
        ["B3", { raw: "18000", computed: 18000 }],
        ["B4", { raw: "=SUM(B2:B3)", computed: 30000, bold: true }],
      ]),
      colWidths: new Map<number, number>(),
      rowHeights: new Map<number, number>(),
      selectedCell: "A1",
      selectedRanges: [{ minRow: 0, maxRow: 0, minCol: 0, maxCol: 0 }],
    },
  ];

  activeSheetId: string = "sheet-1";
  formulaBarText: string = "Revenue";
  autoFillPreview: GridRange | null = null;
  formulaReferences: { range: GridRange; color: string }[] = [];

  // Backward compatibility getters & setters delegating to active sheet
  get cells(): Map<string, CellData> {
    return this.getActiveSheet().cells;
  }
  set cells(val: Map<string, CellData>) {
    this.getActiveSheet().cells = val;
  }

  get selectedCell(): string {
    return this.getActiveSheet().selectedCell;
  }
  set selectedCell(val: string) {
    this.getActiveSheet().selectedCell = val;
  }

  get selectedRanges(): GridRange[] {
    return this.getActiveSheet().selectedRanges;
  }
  set selectedRanges(val: GridRange[]) {
    this.getActiveSheet().selectedRanges = val;
  }

  public getActiveSheet(): SheetData {
    const s = this.sheets.find((sh) => sh.id === this.activeSheetId);
    if (s) return s;
    if (this.sheets.length > 0) return this.sheets[0];
    const fallback: SheetData = {
      id: "sheet-1",
      name: "Sheet1",
      cells: new Map(),
      colWidths: new Map(),
      rowHeights: new Map(),
      selectedCell: "A1",
      selectedRanges: [{ minRow: 0, maxRow: 0, minCol: 0, maxCol: 0 }],
    };
    this.sheets.push(fallback);
    return fallback;
  }

  public getActiveSheetIndex(): number {
    const idx = this.sheets.findIndex((sh) => sh.id === this.activeSheetId);
    return idx >= 0 ? idx : 0;
  }

  public addSheet(name?: string): SheetData {
    let sheetName = name?.trim();
    if (!sheetName) {
      let counter = this.sheets.length + 1;
      while (this.sheets.some((s) => s.name.toLowerCase() === `sheet${counter}`.toLowerCase())) {
        counter++;
      }
      sheetName = `Sheet${counter}`;
    }

    const newSheet: SheetData = {
      id: `sheet-${Date.now()}-${Math.random().toString(36).substr(2, 4)}`,
      name: sheetName,
      cells: new Map<string, CellData>([
        ["A1", { raw: "", computed: "" }],
      ]),
      colWidths: new Map(),
      rowHeights: new Map(),
      selectedCell: "A1",
      selectedRanges: [{ minRow: 0, maxRow: 0, minCol: 0, maxCol: 0 }],
    };

    this.sheets.push(newSheet);
    this.switchSheet(newSheet.id);
    return newSheet;
  }

  public deleteSheet(idOrName: string): boolean {
    if (this.sheets.length <= 1) return false; // Prevent deleting only sheet

    const idx = this.sheets.findIndex(
      (s) => s.id === idOrName || s.name.toLowerCase() === idOrName.toLowerCase()
    );
    if (idx < 0) return false;

    const toDelete = this.sheets[idx];
    this.sheets.splice(idx, 1);

    if (this.activeSheetId === toDelete.id) {
      const nextIdx = Math.max(0, idx - 1);
      this.switchSheet(this.sheets[nextIdx].id);
    }
    return true;
  }

  public renameSheet(idOrName: string, newName: string): boolean {
    const clean = newName.trim();
    if (!clean) return false;

    // Check duplicate
    const exists = this.sheets.some(
      (s) => (s.id !== idOrName && s.name.toLowerCase() !== idOrName.toLowerCase()) &&
             s.name.toLowerCase() === clean.toLowerCase()
    );
    if (exists) return false;

    const target = this.sheets.find(
      (s) => s.id === idOrName || s.name.toLowerCase() === idOrName.toLowerCase()
    );
    if (!target) return false;

    target.name = clean;
    return true;
  }

  public switchSheet(idOrName: string): boolean {
    const s = this.sheets.find(
      (sh) => sh.id === idOrName || sh.name.toLowerCase() === idOrName.toLowerCase()
    );
    if (!s) return false;

    this.activeSheetId = s.id;
    this.formulaBarText = s.cells.get(s.selectedCell)?.raw || "";
    this.updateFormulaReferences();
    return true;
  }

  public getColWidth(col: number): number {
    return this.getActiveSheet().colWidths.get(col) || 96;
  }

  public setColWidth(col: number, width: number): void {
    this.getActiveSheet().colWidths.set(col, Math.max(24, width));
  }

  public getRowHeight(row: number): number {
    return this.getActiveSheet().rowHeights.get(row) || 24;
  }

  public setRowHeight(row: number, height: number): void {
    this.getActiveSheet().rowHeights.set(row, Math.max(16, height));
  }

  public getColOffset(col: number): number {
    let offset = 0;
    for (let c = 0; c < col; c++) {
      offset += this.getColWidth(c);
    }
    return offset;
  }

  public getRowOffset(row: number): number {
    let offset = 0;
    for (let r = 0; r < row; r++) {
      offset += this.getRowHeight(r);
    }
    return offset;
  }

  public insertRow(atRow: number): void {
    const sheet = this.getActiveSheet();
    const newCells = new Map<string, CellData>();

    sheet.cells.forEach((cellData, coord) => {
      const cr = this.coordToColRow(coord);
      if (cr.row >= atRow) {
        const newCoord = `${this.colIndexToLetter(cr.col)}${cr.row + 2}`;
        newCells.set(newCoord, cellData);
      } else {
        newCells.set(coord, cellData);
      }
    });

    sheet.cells = newCells;
    this.recalculateAll();
  }

  public deleteRow(atRow: number): void {
    const sheet = this.getActiveSheet();
    const newCells = new Map<string, CellData>();

    sheet.cells.forEach((cellData, coord) => {
      const cr = this.coordToColRow(coord);
      if (cr.row === atRow) {
        // dropped
      } else if (cr.row > atRow) {
        const newCoord = `${this.colIndexToLetter(cr.col)}${cr.row}`;
        newCells.set(newCoord, cellData);
      } else {
        newCells.set(coord, cellData);
      }
    });

    sheet.cells = newCells;
    this.recalculateAll();
  }

  public clearRow(atRow: number): void {
    const sheet = this.getActiveSheet();
    sheet.cells.forEach((cellData, coord) => {
      const cr = this.coordToColRow(coord);
      if (cr.row === atRow) {
        cellData.raw = "";
        cellData.computed = "";
      }
    });
    this.recalculateAll();
  }

  public insertColumn(atCol: number): void {
    const sheet = this.getActiveSheet();
    const newCells = new Map<string, CellData>();

    sheet.cells.forEach((cellData, coord) => {
      const cr = this.coordToColRow(coord);
      if (cr.col >= atCol) {
        const newCoord = `${this.colIndexToLetter(cr.col + 1)}${cr.row + 1}`;
        newCells.set(newCoord, cellData);
      } else {
        newCells.set(coord, cellData);
      }
    });

    sheet.cells = newCells;
    this.recalculateAll();
  }

  public deleteColumn(atCol: number): void {
    const sheet = this.getActiveSheet();
    const newCells = new Map<string, CellData>();

    sheet.cells.forEach((cellData, coord) => {
      const cr = this.coordToColRow(coord);
      if (cr.col === atCol) {
        // dropped
      } else if (cr.col > atCol) {
        const newCoord = `${this.colIndexToLetter(cr.col - 1)}${cr.row + 1}`;
        newCells.set(newCoord, cellData);
      } else {
        newCells.set(coord, cellData);
      }
    });

    sheet.cells = newCells;
    this.recalculateAll();
  }

  public clearColumn(atCol: number): void {
    const sheet = this.getActiveSheet();
    sheet.cells.forEach((cellData, coord) => {
      const cr = this.coordToColRow(coord);
      if (cr.col === atCol) {
        cellData.raw = "";
        cellData.computed = "";
      }
    });
    this.recalculateAll();
  }

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
      this.selectedRanges = [
        {
          minRow: parsed.row,
          maxRow: parsed.row,
          minCol: parsed.col,
          maxCol: parsed.col,
        },
      ];
    }

    this.updateFormulaReferences();
  }

  commitFormula(val: string) {
    let computed: string | number = val;
    if (val.startsWith("=")) {
      computed = this.evaluateFormula(val);
    } else {
      const num = Number(val);
      if (!isNaN(num) && val.trim() !== "") {
        computed = num;
      }
    }
    const current = this.cells.get(this.selectedCell);
    this.cells.set(this.selectedCell, {
      raw: val,
      computed,
      bold: current?.bold,
      italic: current?.italic,
      underline: current?.underline,
      format: current?.format,
      color: current?.color,
      bgColor: current?.bgColor,
    });
    this.formulaBarText = val;
    this.recalculateAll();
    this.updateFormulaReferences();
  }

  recalculateAll() {
    // Up to 5 passes for DAG dependency propagation
    for (let pass = 0; pass < 5; pass++) {
      let changed = false;
      this.cells.forEach((cell) => {
        if (cell.raw && cell.raw.startsWith("=")) {
          const res = this.evaluateFormula(cell.raw);
          if (res !== cell.computed) {
            cell.computed = res;
            changed = true;
          }
        }
      });
      if (!changed) break;
    }
  }

  evaluateFormula(expr: string): string | number {
    const raw = expr.trim();
    if (!raw.startsWith("=")) return raw;

    const content = raw.slice(1).trim();
    const upper = content.toUpperCase();

    // 1. =SUM(range), =AVERAGE(range), =MIN(range), =MAX(range), =COUNT(range)
    const fnMatch = upper.match(/^(SUM|AVERAGE|AVG|MIN|MAX|COUNT)\(([^)]+)\)$/);
    if (fnMatch) {
      const fnName = fnMatch[1];
      const rangeArg = fnMatch[2].trim();
      const numbers = this.getNumbersFromRangeArg(rangeArg);

      if (fnName === "SUM") {
        return numbers.reduce((a, b) => a + b, 0);
      }
      if (fnName === "AVERAGE" || fnName === "AVG") {
        return numbers.length > 0 ? numbers.reduce((a, b) => a + b, 0) / numbers.length : 0;
      }
      if (fnName === "MIN") {
        return numbers.length > 0 ? Math.min(...numbers) : 0;
      }
      if (fnName === "MAX") {
        return numbers.length > 0 ? Math.max(...numbers) : 0;
      }
      if (fnName === "COUNT") {
        return numbers.length;
      }
    }

    // 2. Simple Arithmetic with Cell References, e.g. A1+A2, A1*2, B2/B1 - 1
    try {
      let evalExpr = content;
      // Replace cell coordinates (e.g. A1, $B$2) with numeric values
      evalExpr = evalExpr.replace(/(\$?[A-Z]+\$?[0-9]+)/gi, (coordMatch) => {
        const cleanCoord = coordMatch.replace(/\$/g, "").toUpperCase();
        const cell = this.cells.get(cleanCoord);
        if (cell) {
          const val = Number(cell.computed);
          return isNaN(val) ? "0" : String(val);
        }
        return "0";
      });

      // Sanitized math evaluation
      if (/^[0-9+\-*/().\s^%]+$/.test(evalExpr)) {
        // Safe evaluation
        const fn = new Function(`return (${evalExpr});`);
        const result = fn();
        if (typeof result === "number" && !isNaN(result) && isFinite(result)) {
          return Math.round(result * 10000) / 10000;
        }
      }
    } catch {
      // Ignore evaluation errors
    }

    // 3. Single Cell Reference, e.g. =A1
    if (/^[A-Z]+[0-9]+$/i.test(content)) {
      const cell = this.cells.get(content.toUpperCase());
      return cell ? cell.computed : "";
    }

    return expr;
  }

  private getNumbersFromRangeArg(arg: string): number[] {
    const nums: number[] = [];
    const parts = arg.split(",");

    for (const part of parts) {
      const clean = part.trim().replace(/\$/g, "").toUpperCase();
      if (clean.includes(":")) {
        const [start, end] = clean.split(":");
        if (start && end) {
          const c1 = this.coordToColRow(start);
          const c2 = this.coordToColRow(end);
          for (let r = Math.min(c1.row, c2.row); r <= Math.max(c1.row, c2.row); r++) {
            for (let c = Math.min(c1.col, c2.col); c <= Math.max(c1.col, c2.col); c++) {
              const coord = `${this.colIndexToLetter(c)}${r + 1}`;
              const cell = this.cells.get(coord);
              if (cell && cell.computed !== undefined && cell.computed !== "") {
                const n = Number(cell.computed);
                if (!isNaN(n)) nums.push(n);
              }
            }
          }
        }
      } else {
        const cell = this.cells.get(clean);
        if (cell && cell.computed !== undefined && cell.computed !== "") {
          const n = Number(cell.computed);
          if (!isNaN(n)) nums.push(n);
        } else {
          const direct = Number(clean);
          if (!isNaN(direct)) nums.push(direct);
        }
      }
    }
    return nums;
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
          color: palette[colorIdx % palette.length],
        });
        colorIdx++;
      } else {
        const p = this.coordToColRow(refStr);
        this.formulaReferences.push({
          range: { minRow: p.row, maxRow: p.row, minCol: p.col, maxCol: p.col },
          color: palette[colorIdx % palette.length],
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
    this.recalculateAll();
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
