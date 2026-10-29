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

export interface RangeStats {
  count: number;
  numbersCount: number;
  sum: number;
  average: number;
  min: number;
  max: number;
}

export interface ClipboardPayload {
  range: GridRange;
  isCut: boolean;
  sheetId: string;
  data: string[][];
}

export function shiftFormula(formula: string, deltaRow: number, deltaCol: number): string {
  if (!formula.startsWith("=")) return formula;

  // Match cell references like $A$1, A$1, $A1, A1
  return formula.replace(/(\$?)([A-Z]+)(\$?)([0-9]+)/gi, (_match, colPinned, colLetters, rowPinned, rowDigits) => {
    let newCol = colLetters.toUpperCase();
    if (!colPinned && deltaCol !== 0) {
      let colIdx = 0;
      for (let i = 0; i < newCol.length; i++) {
        colIdx = colIdx * 26 + (newCol.charCodeAt(i) - 64);
      }
      colIdx = Math.max(1, colIdx + deltaCol);
      let shiftedCol = "";
      while (colIdx > 0) {
        const rem = (colIdx - 1) % 26;
        shiftedCol = String.fromCharCode(65 + rem) + shiftedCol;
        colIdx = Math.floor((colIdx - 1) / 26);
      }
      newCol = shiftedCol;
    }

    let newRow = parseInt(rowDigits, 10);
    if (!rowPinned && deltaRow !== 0) {
      newRow = Math.max(1, newRow + deltaRow);
    }

    return `${colPinned}${newCol}${rowPinned}${newRow}`;
  });
}

const SHORT_DAYS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const FULL_DAYS = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
const SHORT_MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
const FULL_MONTHS = [
  "January", "February", "March", "April", "May", "June",
  "July", "August", "September", "October", "November", "December"
];

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
  clipboardData: ClipboardPayload | null = null;
  internalClipboardText: string = "";

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
      cells: new Map<string, CellData>([["A1", { raw: "", computed: "" }]]),
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
    if (this.sheets.length <= 1) return false;

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

    const exists = this.sheets.some(
      (s) =>
        s.id !== idOrName &&
        s.name.toLowerCase() !== idOrName.toLowerCase() &&
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

  public getSelectedRangeStats(): RangeStats {
    const primary = this.selectedRanges[this.selectedRanges.length - 1];
    if (!primary) {
      return { count: 0, numbersCount: 0, sum: 0, average: 0, min: 0, max: 0 };
    }

    const minCol = Math.min(primary.minCol, primary.maxCol);
    const maxCol = Math.max(primary.minCol, primary.maxCol);
    const minRow = Math.min(primary.minRow, primary.maxRow);
    const maxRow = Math.max(primary.minRow, primary.maxRow);

    let count = 0;
    let numbersCount = 0;
    let sum = 0;
    let min = Infinity;
    let max = -Infinity;

    for (let r = minRow; r <= maxRow; r++) {
      for (let c = minCol; c <= maxCol; c++) {
        const coord = `${this.colIndexToLetter(c)}${r + 1}`;
        const cell = this.cells.get(coord);
        if (cell && cell.raw && cell.raw.trim().length > 0) {
          count++;
          const val = Number(cell.computed !== undefined ? cell.computed : cell.raw);
          if (!isNaN(val)) {
            numbersCount++;
            sum += val;
            if (val < min) min = val;
            if (val > max) max = val;
          }
        }
      }
    }

    const average = numbersCount > 0 ? Math.round((sum / numbersCount) * 1000) / 1000 : 0;
    return {
      count,
      numbersCount,
      sum: Math.round(sum * 1000) / 1000,
      average,
      min: min === Infinity ? 0 : min,
      max: max === -Infinity ? 0 : max,
    };
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

    // 1. Statistical & Math functions
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

    // 2. Arithmetic expressions with cell coordinates
    try {
      let evalExpr = content;
      evalExpr = evalExpr.replace(/(\$?[A-Z]+\$?[0-9]+)/gi, (coordMatch) => {
        const cleanCoord = coordMatch.replace(/\$/g, "").toUpperCase();
        const cell = this.cells.get(cleanCoord);
        if (cell) {
          const val = Number(cell.computed);
          return isNaN(val) ? "0" : String(val);
        }
        return "0";
      });

      if (/^[0-9+\-*/().\s^%]+$/.test(evalExpr)) {
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
        const predicted = this.predictSeries(samples, count, true);
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
        const predicted = this.predictSeries(samples, count, false);
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

  predictSeries(samples: string[], count: number, isRow: boolean = true): string[] {
    if (samples.length === 0) return [];

    // 1. Formulas shifting
    const allFormulas = samples.every((s) => s.startsWith("="));
    if (allFormulas) {
      const res: string[] = [];
      const sourceLen = samples.length;
      for (let k = 0; k < count; k++) {
        const baseFormula = samples[k % sourceLen];
        const cycle = Math.floor(k / sourceLen) + 1;
        const deltaRow = isRow ? cycle * sourceLen : 0;
        const deltaCol = isRow ? 0 : cycle * sourceLen;
        res.push(shiftFormula(baseFormula, deltaRow, deltaCol));
      }
      return res;
    }

    // 2. Days of week auto-fill
    const dayIndices = samples.map((s) => {
      const si = SHORT_DAYS.findIndex((d) => d.toLowerCase() === s.toLowerCase());
      if (si >= 0) return { idx: si, full: false };
      const fi = FULL_DAYS.findIndex((d) => d.toLowerCase() === s.toLowerCase());
      if (fi >= 0) return { idx: fi, full: true };
      return null;
    });

    if (dayIndices.every((d) => d !== null)) {
      const res: string[] = [];
      const last = dayIndices[dayIndices.length - 1]!;
      const isFull = last.full;
      const list = isFull ? FULL_DAYS : SHORT_DAYS;
      const step = dayIndices.length > 1 ? (last.idx - dayIndices[0]!.idx) / (dayIndices.length - 1) : 1;

      for (let k = 1; k <= count; k++) {
        const nextIdx = ((last.idx + Math.round(step * k)) % 7 + 7) % 7;
        res.push(list[nextIdx]);
      }
      return res;
    }

    // 3. Months auto-fill
    const monthIndices = samples.map((s) => {
      const si = SHORT_MONTHS.findIndex((m) => m.toLowerCase() === s.toLowerCase());
      if (si >= 0) return { idx: si, full: false };
      const fi = FULL_MONTHS.findIndex((m) => m.toLowerCase() === s.toLowerCase());
      if (fi >= 0) return { idx: fi, full: true };
      return null;
    });

    if (monthIndices.every((m) => m !== null)) {
      const res: string[] = [];
      const last = monthIndices[monthIndices.length - 1]!;
      const isFull = last.full;
      const list = isFull ? FULL_MONTHS : SHORT_MONTHS;
      const step = monthIndices.length > 1 ? (last.idx - monthIndices[0]!.idx) / (monthIndices.length - 1) : 1;

      for (let k = 1; k <= count; k++) {
        const nextIdx = ((last.idx + Math.round(step * k)) % 12 + 12) % 12;
        res.push(list[nextIdx]);
      }
      return res;
    }

    // 4. Linear numeric series
    const nums = samples.map((s) => Number(s));
    const allNums = nums.every((n) => !isNaN(n) && typeof n === "number");

    if (allNums) {
      if (nums.length === 1) {
        // Increment single number by 1
        const res: string[] = [];
        for (let k = 1; k <= count; k++) {
          res.push(String(nums[0] + k));
        }
        return res;
      }
      const step = (nums[nums.length - 1] - nums[0]) / (nums.length - 1);
      const res: string[] = [];
      for (let k = 1; k <= count; k++) {
        res.push(String(nums[nums.length - 1] + step * k));
      }
      return res;
    }

    // 5. Default pattern repetition
    const res: string[] = [];
    for (let k = 0; k < count; k++) {
      res.push(samples[k % samples.length]);
    }
    return res;
  }

  public copyRange(range?: GridRange): string {
    const targetRange = range || this.selectedRanges[this.selectedRanges.length - 1];
    if (!targetRange) return "";

    const minCol = Math.min(targetRange.minCol, targetRange.maxCol);
    const maxCol = Math.max(targetRange.minCol, targetRange.maxCol);
    const minRow = Math.min(targetRange.minRow, targetRange.maxRow);
    const maxRow = Math.max(targetRange.minRow, targetRange.maxRow);

    const data: string[][] = [];
    const lines: string[] = [];

    for (let r = minRow; r <= maxRow; r++) {
      const rowCells: string[] = [];
      for (let c = minCol; c <= maxCol; c++) {
        const coord = `${this.colIndexToLetter(c)}${r + 1}`;
        const cell = this.cells.get(coord);
        rowCells.push(cell?.raw || "");
      }
      data.push(rowCells);
      lines.push(rowCells.join("\t"));
    }

    const tsv = lines.join("\n");
    this.internalClipboardText = tsv;
    this.clipboardData = {
      range: { minRow, maxRow, minCol, maxCol },
      isCut: false,
      sheetId: this.activeSheetId,
      data,
    };
    return tsv;
  }

  public cutRange(range?: GridRange): string {
    const tsv = this.copyRange(range);
    if (this.clipboardData) {
      this.clipboardData.isCut = true;
    }
    return tsv;
  }

  public pasteRange(tsvText?: string, targetCoord?: string): void {
    const text = tsvText !== undefined ? tsvText : this.internalClipboardText;
    if (!text) return;

    const coord = targetCoord || this.selectedCell;
    const start = this.coordToColRow(coord);

    const rows = text.split(/\r?\n/).filter((line, idx, arr) => line.length > 0 || idx < arr.length - 1);

    rows.forEach((rowStr, rIdx) => {
      const cols = rowStr.split("\t");
      cols.forEach((val, cIdx) => {
        const targetCol = start.col + cIdx;
        const targetRow = start.row + rIdx;
        const cellCoord = `${this.colIndexToLetter(targetCol)}${targetRow + 1}`;

        let computed: string | number = val;
        if (val.startsWith("=")) {
          computed = this.evaluateFormula(val);
        } else {
          const n = Number(val);
          if (!isNaN(n) && val.trim() !== "") computed = n;
        }
        this.cells.set(cellCoord, { raw: val, computed });
      });
    });

    // If this was a cut operation, clear the original cells
    if (this.clipboardData?.isCut) {
      const src = this.clipboardData.range;
      const srcSheet =
        this.sheets.find((s) => s.id === this.clipboardData!.sheetId) || this.getActiveSheet();
      const numCols = rows[0]?.split("\t").length || 1;
      const numRows = rows.length;

      for (let r = src.minRow; r <= src.maxRow; r++) {
        for (let c = src.minCol; c <= src.maxCol; c++) {
          const isOverlapping =
            srcSheet.id === this.activeSheetId &&
            r >= start.row &&
            r < start.row + numRows &&
            c >= start.col &&
            c < start.col + numCols;

          if (!isOverlapping) {
            const srcCoord = `${this.colIndexToLetter(c)}${r + 1}`;
            srcSheet.cells.delete(srcCoord);
          }
        }
      }
      this.clipboardData = null;
    }

    this.recalculateAll();
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
