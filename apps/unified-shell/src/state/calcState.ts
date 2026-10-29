export interface CellData {
  raw: string;
  computed: string | number;
  format?: string;
  decimals?: number;
  bold?: boolean;
  italic?: boolean;
  underline?: boolean;
  align?: "left" | "center" | "right";
  valign?: "top" | "middle" | "bottom";
  wrapText?: boolean;
  borders?: string;
  color?: string;
  bgColor?: string;
}

export interface GridRange {
  minRow: number;
  maxRow: number;
  minCol: number;
  maxCol: number;
}

export interface SheetChart {
  id: string;
  type: "column" | "bar" | "line" | "pie";
  title: string;
  dataRange: GridRange;
  x: number;
  y: number;
  width: number;
  height: number;
}

export type ConditionalFormatType =
  | "greaterThan"
  | "lessThan"
  | "between"
  | "equalTo"
  | "duplicateValues"
  | "dataBar"
  | "colorScale";

export interface ConditionalFormatRule {
  id: string;
  type: ConditionalFormatType;
  range: GridRange;
  value1?: number | string;
  value2?: number | string;
  bgColor?: string;
  color?: string;
  barColor?: string;
}

export interface SheetData {
  id: string;
  name: string;
  cells: Map<string, CellData>;
  colWidths: Map<number, number>;
  rowHeights: Map<number, number>;
  selectedCell: string;
  selectedRanges: GridRange[];
  filterActive?: boolean;
  filterRange?: GridRange | null;
  hiddenRows?: Set<number>;
  mergedRanges?: GridRange[];
  charts?: SheetChart[];
  conditionalFormats?: ConditionalFormatRule[];
  freezePanes?: { row: number; col: number };
  showGridlines?: boolean;
  showHeadings?: boolean;
  showFormulaBar?: boolean;
}

export function formatCellValue(cell: CellData): string {
  if (cell.computed === undefined || cell.computed === null || cell.computed === "") {
    return "";
  }
  const format = cell.format || "General";
  const num = typeof cell.computed === "number" ? cell.computed : Number(cell.computed);
  const isNumeric = !isNaN(num) && typeof cell.computed !== "boolean" && String(cell.raw).trim() !== "";

  if (!isNumeric) {
    return String(cell.computed);
  }

  const decimals = cell.decimals !== undefined ? cell.decimals : (format === "General" ? undefined : 2);

  switch (format) {
    case "Currency": {
      const d = decimals !== undefined ? decimals : 2;
      const abs = Math.abs(num).toLocaleString("en-US", {
        minimumFractionDigits: d,
        maximumFractionDigits: d,
      });
      return num < 0 ? `-$${abs}` : `$${abs}`;
    }
    case "Percentage": {
      const d = decimals !== undefined ? decimals : 2;
      const pctVal = (num * 100).toLocaleString("en-US", {
        minimumFractionDigits: d,
        maximumFractionDigits: d,
      });
      return `${pctVal}%`;
    }
    case "Comma":
    case "Number": {
      const d = decimals !== undefined ? decimals : 2;
      return num.toLocaleString("en-US", {
        minimumFractionDigits: d,
        maximumFractionDigits: d,
      });
    }
    case "Accounting": {
      const d = decimals !== undefined ? decimals : 2;
      const abs = Math.abs(num).toLocaleString("en-US", {
        minimumFractionDigits: d,
        maximumFractionDigits: d,
      });
      return num < 0 ? `($ ${abs})` : `$ ${abs}`;
    }
    case "Short Date":
    case "Date": {
      if (num > 0) {
        const date = new Date((num - (25567 + 2)) * 86400 * 1000);
        if (!isNaN(date.getTime()) && date.getFullYear() > 1900 && date.getFullYear() < 2100) {
          return date.toLocaleDateString("en-US");
        }
      }
      return String(cell.computed);
    }
    case "General":
    default:
      if (decimals !== undefined) {
        return num.toLocaleString("en-US", {
          minimumFractionDigits: decimals,
          maximumFractionDigits: decimals,
        });
      }
      return String(cell.computed);
  }
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
      charts: [],
      conditionalFormats: [],
      freezePanes: { row: 0, col: 0 },
      showGridlines: true,
      showHeadings: true,
      showFormulaBar: true,
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

  get charts(): SheetChart[] {
    return this.getActiveSheet().charts || [];
  }

  setCellValue(coord: string, val: string) {
    this.selectedCell = coord;
    this.commitFormula(val);
  }

  getCellValue(coord: string): string | number | undefined {
    return this.cells.get(coord)?.computed;
  }

  get frozenRows(): number {
    return this.getActiveSheet().freezePanes?.row || 0;
  }

  get frozenCols(): number {
    return this.getActiveSheet().freezePanes?.col || 0;
  }

  public selectSheet(idxOrId: number | string): boolean {
    if (typeof idxOrId === "number") {
      if (idxOrId >= 0 && idxOrId < this.sheets.length) {
        return this.switchSheet(this.sheets[idxOrId].id);
      }
      return false;
    }
    return this.switchSheet(idxOrId);
  }

  private undoStack: string[] = [];
  private redoStack: string[] = [];

  public commitSnapshot() {
    const snap = JSON.stringify(
      this.sheets.map((s) => ({
        id: s.id,
        name: s.name,
        cells: Array.from(s.cells.entries()),
        selectedCell: s.selectedCell,
        freezePanes: s.freezePanes,
      }))
    );
    this.undoStack.push(snap);
    this.redoStack = [];
    if (this.undoStack.length > 50) this.undoStack.shift();
  }

  public canUndo(): boolean {
    return this.undoStack.length > 0;
  }

  public canRedo(): boolean {
    return this.redoStack.length > 0;
  }

  public undo(): boolean {
    if (!this.canUndo()) return false;
    const currentSnap = JSON.stringify(
      this.sheets.map((s) => ({
        id: s.id,
        name: s.name,
        cells: Array.from(s.cells.entries()),
        selectedCell: s.selectedCell,
        freezePanes: s.freezePanes,
      }))
    );
    this.redoStack.push(currentSnap);
    const prev = JSON.parse(this.undoStack.pop()!);
    this.restoreSnapshot(prev);
    return true;
  }

  public redo(): boolean {
    if (!this.canRedo()) return false;
    const currentSnap = JSON.stringify(
      this.sheets.map((s) => ({
        id: s.id,
        name: s.name,
        cells: Array.from(s.cells.entries()),
        selectedCell: s.selectedCell,
        freezePanes: s.freezePanes,
      }))
    );
    this.undoStack.push(currentSnap);
    const next = JSON.parse(this.redoStack.pop()!);
    this.restoreSnapshot(next);
    return true;
  }

  private restoreSnapshot(saved: any[]) {
    this.sheets = saved.map((s) => ({
      ...s,
      cells: new Map(s.cells),
      colWidths: new Map(),
      rowHeights: new Map(),
      selectedRanges: [{ minRow: 0, maxRow: 0, minCol: 0, maxCol: 0 }],
      charts: [],
      conditionalFormats: [],
      showGridlines: true,
      showHeadings: true,
      showFormulaBar: true,
    }));
    this.recalculateAll();
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
      charts: [],
      conditionalFormats: [],
      freezePanes: { row: 0, col: 0 },
      showGridlines: true,
      showHeadings: true,
      showFormulaBar: true,
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
      charts: [],
      conditionalFormats: [],
      freezePanes: { row: 0, col: 0 },
      showGridlines: true,
      showHeadings: true,
      showFormulaBar: true,
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
    this.commitSnapshot();
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
      decimals: current?.decimals,
      align: current?.align,
      valign: current?.valign,
      wrapText: current?.wrapText,
      borders: current?.borders,
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

  public getTargetRange(range?: GridRange): GridRange {
    const raw = range || this.selectedRanges[this.selectedRanges.length - 1];
    if (!raw) {
      const cr = this.coordToColRow(this.selectedCell);
      return { minRow: cr.row, maxRow: cr.row, minCol: cr.col, maxCol: cr.col };
    }
    return {
      minRow: Math.min(raw.minRow, raw.maxRow),
      maxRow: Math.max(raw.minRow, raw.maxRow),
      minCol: Math.min(raw.minCol, raw.maxCol),
      maxCol: Math.max(raw.minCol, raw.maxCol),
    };
  }

  public getOrCreateCell(coord: string): CellData {
    let cell = this.cells.get(coord);
    if (!cell) {
      cell = { raw: "", computed: "" };
      this.cells.set(coord, cell);
    }
    return cell;
  }

  public forEachCellInRange(
    range: GridRange | undefined,
    fn: (cell: CellData, coord: string, r: number, c: number) => void
  ): void {
    const target = this.getTargetRange(range);
    for (let r = target.minRow; r <= target.maxRow; r++) {
      for (let c = target.minCol; c <= target.maxCol; c++) {
        const coord = `${this.colIndexToLetter(c)}${r + 1}`;
        const cell = this.getOrCreateCell(coord);
        fn(cell, coord, r, c);
      }
    }
  }

  public setNumberFormat(format: string, range?: GridRange): void {
    this.forEachCellInRange(range, (cell) => {
      cell.format = format;
      if (
        (format === "Currency" || format === "Percentage" || format === "Comma" || format === "Accounting") &&
        cell.decimals === undefined
      ) {
        cell.decimals = 2;
      }
    });
  }

  public increaseDecimal(range?: GridRange): void {
    this.forEachCellInRange(range, (cell) => {
      cell.decimals = Math.min(10, (cell.decimals !== undefined ? cell.decimals : 2) + 1);
    });
  }

  public decreaseDecimal(range?: GridRange): void {
    this.forEachCellInRange(range, (cell) => {
      cell.decimals = Math.max(0, (cell.decimals !== undefined ? cell.decimals : 2) - 1);
    });
  }

  public toggleBold(range?: GridRange): void {
    const target = this.getTargetRange(range);
    const primary = this.cells.get(`${this.colIndexToLetter(target.minCol)}${target.minRow + 1}`);
    const nextState = !primary?.bold;
    this.forEachCellInRange(range, (cell) => {
      cell.bold = nextState;
    });
  }

  public toggleItalic(range?: GridRange): void {
    const target = this.getTargetRange(range);
    const primary = this.cells.get(`${this.colIndexToLetter(target.minCol)}${target.minRow + 1}`);
    const nextState = !primary?.italic;
    this.forEachCellInRange(range, (cell) => {
      cell.italic = nextState;
    });
  }

  public toggleUnderline(range?: GridRange): void {
    const target = this.getTargetRange(range);
    const primary = this.cells.get(`${this.colIndexToLetter(target.minCol)}${target.minRow + 1}`);
    const nextState = !primary?.underline;
    this.forEachCellInRange(range, (cell) => {
      cell.underline = nextState;
    });
  }

  public setAlignment(align: "left" | "center" | "right", range?: GridRange): void {
    this.forEachCellInRange(range, (cell) => {
      cell.align = align;
    });
  }

  public setVerticalAlignment(valign: "top" | "middle" | "bottom", range?: GridRange): void {
    this.forEachCellInRange(range, (cell) => {
      cell.valign = valign;
    });
  }

  public toggleWrapText(range?: GridRange): void {
    const target = this.getTargetRange(range);
    const primary = this.cells.get(`${this.colIndexToLetter(target.minCol)}${target.minRow + 1}`);
    const nextState = !primary?.wrapText;
    this.forEachCellInRange(range, (cell) => {
      cell.wrapText = nextState;
    });
  }

  public setBorders(borderType: string = "all", range?: GridRange): void {
    this.forEachCellInRange(range, (cell) => {
      cell.borders = borderType;
    });
  }

  public setFillColor(color: string, range?: GridRange): void {
    this.forEachCellInRange(range, (cell) => {
      cell.bgColor = color;
    });
  }

  public setFontColor(color: string, range?: GridRange): void {
    this.forEachCellInRange(range, (cell) => {
      cell.color = color;
    });
  }

  public mergeAndCenter(range?: GridRange): void {
    const target = this.getTargetRange(range);
    const sheet = this.getActiveSheet();
    sheet.mergedRanges = sheet.mergedRanges || [];
    sheet.mergedRanges.push(target);

    const topLeftCoord = `${this.colIndexToLetter(target.minCol)}${target.minRow + 1}`;
    const topLeft = this.getOrCreateCell(topLeftCoord);
    topLeft.align = "center";

    // Clear other cells
    for (let r = target.minRow; r <= target.maxRow; r++) {
      for (let c = target.minCol; c <= target.maxCol; c++) {
        if (r === target.minRow && c === target.minCol) continue;
        const coord = `${this.colIndexToLetter(c)}${r + 1}`;
        this.cells.delete(coord);
      }
    }
  }

  public insertAutoSum(fnName: "SUM" | "AVERAGE" | "COUNT" | "MAX" | "MIN" = "SUM"): string {
    const target = this.getTargetRange();
    const isMultiCell = target.minRow !== target.maxRow || target.minCol !== target.maxCol;

    let formula = "";
    let destCoord = "";

    if (isMultiCell) {
      const minColLetter = this.colIndexToLetter(target.minCol);
      const maxColLetter = this.colIndexToLetter(target.maxCol);
      const rangeRef = `${minColLetter}${target.minRow + 1}:${maxColLetter}${target.maxRow + 1}`;
      formula = `=${fnName}(${rangeRef})`;
      destCoord = `${minColLetter}${target.maxRow + 2}`;
    } else {
      const curr = this.coordToColRow(this.selectedCell);
      let startRow = curr.row - 1;
      let aboveCount = 0;
      while (startRow >= 0) {
        const cCoord = `${this.colIndexToLetter(curr.col)}${startRow + 1}`;
        const cell = this.cells.get(cCoord);
        if (cell && cell.raw && cell.raw.trim().length > 0) {
          aboveCount++;
          startRow--;
        } else {
          break;
        }
      }

      if (aboveCount > 0) {
        const fromRow = startRow + 1;
        const toRow = curr.row - 1;
        const colLetter = this.colIndexToLetter(curr.col);
        const rangeRef = fromRow === toRow ? `${colLetter}${fromRow + 1}` : `${colLetter}${fromRow + 1}:${colLetter}${toRow + 1}`;
        formula = `=${fnName}(${rangeRef})`;
        destCoord = this.selectedCell;
      } else {
        let startCol = curr.col - 1;
        let leftCount = 0;
        while (startCol >= 0) {
          const cCoord = `${this.colIndexToLetter(startCol)}${curr.row + 1}`;
          const cell = this.cells.get(cCoord);
          if (cell && cell.raw && cell.raw.trim().length > 0) {
            leftCount++;
            startCol--;
          } else {
            break;
          }
        }

        if (leftCount > 0) {
          const fromCol = startCol + 1;
          const toCol = curr.col - 1;
          const rowNum = curr.row + 1;
          const fromColLetter = this.colIndexToLetter(fromCol);
          const toColLetter = this.colIndexToLetter(toCol);
          const rangeRef = fromCol === toCol ? `${fromColLetter}${rowNum}` : `${fromColLetter}${rowNum}:${toColLetter}${rowNum}`;
          formula = `=${fnName}(${rangeRef})`;
          destCoord = this.selectedCell;
        } else {
          formula = `=${fnName}()`;
          destCoord = this.selectedCell;
        }
      }
    }

    this.selectedCell = destCoord;
    this.commitFormula(formula);
    return formula;
  }

  public sortRange(ascending: boolean = true, sortColIndex?: number, range?: GridRange): void {
    let target = this.getTargetRange(range);
    // If only 1 cell selected, expand to contiguous used rows in this column block
    if (target.minRow === target.maxRow && target.minCol === target.maxCol) {
      let rMin = target.minRow;
      let rMax = target.maxRow;
      while (rMin > 0 && this.cells.has(`${this.colIndexToLetter(target.minCol)}${rMin}`)) {
        rMin--;
      }
      while (this.cells.has(`${this.colIndexToLetter(target.minCol)}${rMax + 2}`)) {
        rMax++;
      }
      target = { minRow: rMin, maxRow: rMax, minCol: target.minCol, maxCol: target.maxCol };
    }

    const sortCol = sortColIndex !== undefined ? sortColIndex : target.minCol;
    const rowRecords: { rowIdx: number; cells: Map<number, CellData>; key: string | number }[] = [];

    for (let r = target.minRow; r <= target.maxRow; r++) {
      const rowMap = new Map<number, CellData>();
      let sortKey: string | number = "";
      for (let c = target.minCol; c <= target.maxCol; c++) {
        const coord = `${this.colIndexToLetter(c)}${r + 1}`;
        const cell = this.cells.get(coord);
        if (cell) {
          rowMap.set(c, { ...cell });
          if (c === sortCol) {
            sortKey = cell.computed !== undefined ? cell.computed : cell.raw;
          }
        }
      }
      rowRecords.push({ rowIdx: r, cells: rowMap, key: sortKey });
    }

    rowRecords.sort((a, b) => {
      const valA = a.key;
      const valB = b.key;
      const numA = Number(valA);
      const numB = Number(valB);
      const isNumA = !isNaN(numA) && typeof valA !== "boolean" && String(valA).trim() !== "";
      const isNumB = !isNaN(numB) && typeof valB !== "boolean" && String(valB).trim() !== "";

      let cmp = 0;
      if (isNumA && isNumB) {
        cmp = numA - numB;
      } else {
        cmp = String(valA).localeCompare(String(valB), undefined, { numeric: true, sensitivity: "base" });
      }
      return ascending ? cmp : -cmp;
    });

    // Write back sorted records
    for (let i = 0; i < rowRecords.length; i++) {
      const destRow = target.minRow + i;
      const rec = rowRecords[i];
      for (let c = target.minCol; c <= target.maxCol; c++) {
        const destCoord = `${this.colIndexToLetter(c)}${destRow + 1}`;
        const cellData = rec.cells.get(c);
        if (cellData) {
          this.cells.set(destCoord, cellData);
        } else {
          this.cells.delete(destCoord);
        }
      }
    }

    this.recalculateAll();
  }

  public toggleFilter(range?: GridRange): boolean {
    const sheet = this.getActiveSheet();
    if (sheet.filterActive) {
      sheet.filterActive = false;
      sheet.filterRange = null;
      sheet.hiddenRows?.clear();
      return false;
    } else {
      sheet.filterActive = true;
      sheet.filterRange = this.getTargetRange(range);
      sheet.hiddenRows = sheet.hiddenRows || new Set<number>();
      return true;
    }
  }

  public clearFilter(): void {
    const sheet = this.getActiveSheet();
    sheet.hiddenRows?.clear();
  }

  public filterColumn(colIdx: number, criteria: (val: string) => boolean): void {
    const sheet = this.getActiveSheet();
    if (!sheet.filterRange) return;
    sheet.hiddenRows = sheet.hiddenRows || new Set<number>();
    sheet.hiddenRows.clear();
    const startRow = sheet.filterRange.maxRow + 1;
    for (let r = startRow; r < startRow + 200; r++) {
      const coord = `${this.colIndexToLetter(colIdx)}${r + 1}`;
      const cell = this.cells.get(coord);
      if (cell && cell.raw) {
        const text = String(cell.computed !== undefined ? cell.computed : cell.raw);
        if (!criteria(text)) {
          sheet.hiddenRows.add(r);
        }
      }
    }
  }

  public textToColumns(delimiter: string = ",", range?: GridRange): void {
    const target = this.getTargetRange(range);
    for (let r = target.minRow; r <= target.maxRow; r++) {
      const srcCoord = `${this.colIndexToLetter(target.minCol)}${r + 1}`;
      const cell = this.cells.get(srcCoord);
      if (!cell || !cell.raw) continue;

      const tokens = cell.raw.split(delimiter);
      tokens.forEach((tok, idx) => {
        const col = target.minCol + idx;
        const dest = `${this.colIndexToLetter(col)}${r + 1}`;
        const trimmed = tok.trim();
        const num = Number(trimmed);
        const computed = !isNaN(num) && trimmed !== "" ? num : trimmed;
        this.cells.set(dest, { raw: trimmed, computed });
      });
    }
    this.recalculateAll();
  }

  public flashFill(range?: GridRange): void {
    const target = this.getTargetRange(range);
    const targetCol = target.minCol;
    const sourceCol = Math.max(0, targetCol - 1);

    // Look at first row of selection
    const topSrcCoord = `${this.colIndexToLetter(sourceCol)}${target.minRow + 1}`;
    const topTgtCoord = `${this.colIndexToLetter(targetCol)}${target.minRow + 1}`;
    const srcCell = this.cells.get(topSrcCoord);
    const tgtCell = this.cells.get(topTgtCoord);

    if (!srcCell || !tgtCell) return;

    const sVal = String(srcCell.computed !== undefined ? srcCell.computed : srcCell.raw).trim();
    const tVal = String(tgtCell.computed !== undefined ? tgtCell.computed : tgtCell.raw).trim();

    let transform: (source: string) => string = (s) => s;

    const delimiters = ["@", "-", "_", "/", ":", ";", ","];
    let matchedDelim: { delim: string; partIdx: number } | null = null;
    for (const d of delimiters) {
      if (sVal.includes(d)) {
        const parts = sVal.split(d);
        const pIdx = parts.findIndex((p) => p === tVal);
        if (pIdx >= 0) {
          matchedDelim = { delim: d, partIdx: pIdx };
          break;
        }
      }
    }

    if (tVal.toLowerCase() === sVal.toLowerCase() && tVal !== sVal) {
      if (tVal === sVal.toUpperCase()) transform = (s) => s.toUpperCase();
      else if (tVal === sVal.toLowerCase()) transform = (s) => s.toLowerCase();
    } else if (sVal.split(/\s+/)[0] === tVal) {
      transform = (s) => s.split(/\s+/)[0] || "";
    } else if (sVal.split(/\s+/).slice(-1)[0] === tVal) {
      transform = (s) => s.split(/\s+/).slice(-1)[0] || "";
    } else if (matchedDelim) {
      const { delim, partIdx } = matchedDelim;
      transform = (s) => s.split(delim)[partIdx] || "";
    } else if (sVal.replace(/\D/g, "") === tVal) {
      transform = (s) => s.replace(/\D/g, "");
    } else {
      const idx = sVal.indexOf(tVal);
      if (idx >= 0) {
        const len = tVal.length;
        transform = (s) => s.substring(idx, idx + len);
      }
    }

    for (let r = target.minRow + 1; r <= target.maxRow; r++) {
      const srcRowCoord = `${this.colIndexToLetter(sourceCol)}${r + 1}`;
      const destRowCoord = `${this.colIndexToLetter(targetCol)}${r + 1}`;
      const rowSrc = this.cells.get(srcRowCoord);
      if (rowSrc) {
        const text = String(rowSrc.computed !== undefined ? rowSrc.computed : rowSrc.raw);
        const result = transform(text);
        const num = Number(result);
        const computed = !isNaN(num) && result !== "" ? num : result;
        this.cells.set(destRowCoord, { raw: result, computed });
      }
    }

    this.recalculateAll();
  }

  public removeDuplicates(range?: GridRange): number {
    const target = this.getTargetRange(range);
    const seen = new Set<string>();
    let removed = 0;

    for (let r = target.minRow; r <= target.maxRow; r++) {
      const rowTokens: string[] = [];
      for (let c = target.minCol; c <= target.maxCol; c++) {
        const coord = `${this.colIndexToLetter(c)}${r + 1}`;
        const cell = this.cells.get(coord);
        rowTokens.push(cell?.raw || "");
      }
      const fingerprint = rowTokens.join("|||");
      if (seen.has(fingerprint)) {
        for (let c = target.minCol; c <= target.maxCol; c++) {
          const coord = `${this.colIndexToLetter(c)}${r + 1}`;
          this.cells.delete(coord);
        }
        removed++;
      } else {
        seen.add(fingerprint);
      }
    }

    this.recalculateAll();
    return removed;
  }

  public insertChart(
    type: "column" | "bar" | "line" | "pie" = "column",
    range?: GridRange,
    title?: string
  ): SheetChart {
    const sheet = this.getActiveSheet();
    sheet.charts = sheet.charts || [];
    const targetRange = this.getTargetRange(range);

    let chartTitle = title;
    if (!chartTitle) {
      const topCell = this.cells.get(`${this.colIndexToLetter(targetRange.minCol)}${targetRange.minRow + 1}`);
      chartTitle = topCell && topCell.raw ? String(topCell.computed || topCell.raw) : `${type.toUpperCase()} Chart`;
    }

    const x = 40 + ((sheet.charts.length * 30) % 240);
    const y = 60 + ((sheet.charts.length * 30) % 240);

    const chart: SheetChart = {
      id: `chart-${Date.now()}-${Math.random().toString(36).substr(2, 4)}`,
      type,
      title: chartTitle,
      dataRange: targetRange,
      x,
      y,
      width: 360,
      height: 240,
    };

    sheet.charts.push(chart);
    return chart;
  }

  public deleteChart(chartId: string): boolean {
    const sheet = this.getActiveSheet();
    if (!sheet.charts) return false;
    const idx = sheet.charts.findIndex((c) => c.id === chartId);
    if (idx >= 0) {
      sheet.charts.splice(idx, 1);
      return true;
    }
    return false;
  }

  public getCharts(): SheetChart[] {
    const sheet = this.getActiveSheet();
    return sheet.charts || [];
  }

  public getChartData(chart: SheetChart): {
    categories: string[];
    series: { name: string; values: number[] }[];
  } {
    const range = chart.dataRange;
    const numCols = range.maxCol - range.minCol + 1;

    if (numCols === 1) {
      const values: number[] = [];
      const categories: string[] = [];
      for (let r = range.minRow; r <= range.maxRow; r++) {
        const coord = `${this.colIndexToLetter(range.minCol)}${r + 1}`;
        const cell = this.cells.get(coord);
        const val = Number(cell?.computed !== undefined ? cell.computed : cell?.raw);
        values.push(isNaN(val) ? 0 : val);
        categories.push(`Row ${r + 1}`);
      }
      return {
        categories,
        series: [{ name: chart.title, values }],
      };
    }

    const firstDataRowCell = this.cells.get(`${this.colIndexToLetter(range.minCol + 1)}${range.minRow + 1}`);
    const hasHeaderRow = isNaN(Number(firstDataRowCell?.raw)) && firstDataRowCell?.raw !== "";
    const startRow = hasHeaderRow ? range.minRow + 1 : range.minRow;

    const categories: string[] = [];
    for (let r = startRow; r <= range.maxRow; r++) {
      const coord = `${this.colIndexToLetter(range.minCol)}${r + 1}`;
      const cell = this.cells.get(coord);
      categories.push(cell?.raw ? String(cell.computed || cell.raw) : `R${r + 1}`);
    }

    const series: { name: string; values: number[] }[] = [];
    for (let c = range.minCol + 1; c <= range.maxCol; c++) {
      let seriesName = `Series ${c - range.minCol}`;
      if (hasHeaderRow) {
        const headerCoord = `${this.colIndexToLetter(c)}${range.minRow + 1}`;
        const headerCell = this.cells.get(headerCoord);
        if (headerCell?.raw) seriesName = String(headerCell.computed || headerCell.raw);
      }

      const values: number[] = [];
      for (let r = startRow; r <= range.maxRow; r++) {
        const coord = `${this.colIndexToLetter(c)}${r + 1}`;
        const cell = this.cells.get(coord);
        const num = Number(cell?.computed !== undefined ? cell.computed : cell?.raw);
        values.push(isNaN(num) ? 0 : num);
      }
      series.push({ name: seriesName, values });
    }

    return { categories, series };
  }

  public addConditionalFormat(rule: Omit<ConditionalFormatRule, "id">): ConditionalFormatRule {
    const sheet = this.getActiveSheet();
    sheet.conditionalFormats = sheet.conditionalFormats || [];
    const newRule: ConditionalFormatRule = {
      ...rule,
      id: `cf-${Date.now()}-${Math.random().toString(36).substr(2, 4)}`,
    };
    sheet.conditionalFormats.push(newRule);
    return newRule;
  }

  public clearConditionalFormats(range?: GridRange): void {
    const sheet = this.getActiveSheet();
    if (!range) {
      sheet.conditionalFormats = [];
    } else {
      const target = this.getTargetRange(range);
      sheet.conditionalFormats = (sheet.conditionalFormats || []).filter(
        (f) =>
          !(
            f.range.minRow === target.minRow &&
            f.range.maxRow === target.maxRow &&
            f.range.minCol === target.minCol &&
            f.range.maxCol === target.maxCol
          )
      );
    }
  }

  public getConditionalStyleForCell(
    coord: string,
    row: number,
    col: number
  ): { bgColor?: string; color?: string; dataBarRatio?: number; barColor?: string } | null {
    const sheet = this.getActiveSheet();
    if (!sheet.conditionalFormats || sheet.conditionalFormats.length === 0) return null;

    const cell = this.cells.get(coord);
    const val = cell?.computed !== undefined ? cell.computed : cell?.raw;
    const numVal = Number(val);
    const isNum = !isNaN(numVal) && val !== "" && val !== undefined;

    for (const rule of sheet.conditionalFormats) {
      const r = rule.range;
      if (row >= r.minRow && row <= r.maxRow && col >= r.minCol && col <= r.maxCol) {
        switch (rule.type) {
          case "greaterThan": {
            if (isNum && numVal > Number(rule.value1)) {
              return { bgColor: rule.bgColor || "#fee2e2", color: rule.color || "#991b1b" };
            }
            break;
          }
          case "lessThan": {
            if (isNum && numVal < Number(rule.value1)) {
              return { bgColor: rule.bgColor || "#dcfce7", color: rule.color || "#15803d" };
            }
            break;
          }
          case "between": {
            if (isNum && numVal >= Number(rule.value1) && numVal <= Number(rule.value2)) {
              return { bgColor: rule.bgColor || "#fef3c7", color: rule.color || "#92400e" };
            }
            break;
          }
          case "equalTo": {
            if (String(val) === String(rule.value1)) {
              return { bgColor: rule.bgColor || "#e0e7ff", color: rule.color || "#3730a3" };
            }
            break;
          }
          case "duplicateValues": {
            let count = 0;
            for (let checkR = r.minRow; checkR <= r.maxRow; checkR++) {
              for (let checkC = r.minCol; checkC <= r.maxCol; checkC++) {
                const checkCoord = `${this.colIndexToLetter(checkC)}${checkR + 1}`;
                const cCell = this.cells.get(checkCoord);
                const cVal = cCell?.computed !== undefined ? cCell.computed : cCell?.raw;
                if (cVal !== undefined && cVal !== "" && String(cVal) === String(val)) {
                  count++;
                }
              }
            }
            if (count > 1) {
              return { bgColor: rule.bgColor || "#fee2e2", color: rule.color || "#991b1b" };
            }
            break;
          }
          case "dataBar": {
            if (isNum) {
              let min = Infinity;
              let max = -Infinity;
              for (let checkR = r.minRow; checkR <= r.maxRow; checkR++) {
                for (let checkC = r.minCol; checkC <= r.maxCol; checkC++) {
                  const cCell = this.cells.get(`${this.colIndexToLetter(checkC)}${checkR + 1}`);
                  const n = Number(cCell?.computed !== undefined ? cCell.computed : cCell?.raw);
                  if (!isNaN(n)) {
                    if (n < min) min = n;
                    if (n > max) max = n;
                  }
                }
              }
              const ratio = max > min ? Math.max(0, Math.min(1, (numVal - min) / (max - min))) : 1;
              return { dataBarRatio: ratio, barColor: rule.barColor || "#60a5fa" };
            }
            break;
          }
          case "colorScale": {
            if (isNum) {
              let min = Infinity;
              let max = -Infinity;
              for (let checkR = r.minRow; checkR <= r.maxRow; checkR++) {
                for (let checkC = r.minCol; checkC <= r.maxCol; checkC++) {
                  const cCell = this.cells.get(`${this.colIndexToLetter(checkC)}${checkR + 1}`);
                  const n = Number(cCell?.computed !== undefined ? cCell.computed : cCell?.raw);
                  if (!isNaN(n)) {
                    if (n < min) min = n;
                    if (n > max) max = n;
                  }
                }
              }
              const ratio = max > min ? Math.max(0, Math.min(1, (numVal - min) / (max - min))) : 0.5;
              let rCol: number, gCol: number, bCol: number;
              if (ratio < 0.5) {
                const factor = ratio * 2;
                rCol = Math.round(99 + factor * (254 - 99));
                gCol = Math.round(190 + factor * (243 - 190));
                bCol = Math.round(123 + factor * (199 - 123));
              } else {
                const factor = (ratio - 0.5) * 2;
                rCol = Math.round(254 - factor * (254 - 248));
                gCol = Math.round(243 - factor * (243 - 113));
                bCol = Math.round(199 - factor * (199 - 113));
              }
              return { bgColor: `rgb(${rCol},${gCol},${bCol})` };
            }
            break;
          }
        }
      }
    }
    return null;
  }

  public freezeTopRow(): void {
    this.getActiveSheet().freezePanes = { row: 1, col: 0 };
  }

  public freezeFirstColumn(): void {
    this.getActiveSheet().freezePanes = { row: 0, col: 1 };
  }

  public freezePanesAtSelection(range?: GridRange): void {
    const target = this.getTargetRange(range);
    this.getActiveSheet().freezePanes = { row: target.minRow, col: target.minCol };
  }

  public unfreezePanes(): void {
    this.getActiveSheet().freezePanes = { row: 0, col: 0 };
  }

  public toggleGridlines(): boolean {
    const sheet = this.getActiveSheet();
    sheet.showGridlines = sheet.showGridlines !== undefined ? !sheet.showGridlines : false;
    return sheet.showGridlines;
  }

  public toggleHeadings(): boolean {
    const sheet = this.getActiveSheet();
    sheet.showHeadings = sheet.showHeadings !== undefined ? !sheet.showHeadings : false;
    return sheet.showHeadings;
  }

  public toggleFormulaBar(): boolean {
    const sheet = this.getActiveSheet();
    sheet.showFormulaBar = sheet.showFormulaBar !== undefined ? !sheet.showFormulaBar : false;
    return sheet.showFormulaBar;
  }
}

export const calcState = new CalcState();
