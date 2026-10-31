export interface CellSparkline {
  type: "line" | "column" | "winloss";
  data?: number[];
  color?: string;
}

export interface DataValidationRule {
  type: "list" | "whole" | "date";
  allow?: string[];
  min?: number;
  max?: number;
}

export interface CellData {
  raw: string;
  computed: string | number | boolean;
  format?: string;
  decimals?: number;
  bold?: boolean;
  italic?: boolean;
  underline?: boolean;
  fontFamily?: string;
  fontSize?: number;
  align?: "left" | "center" | "right";
  valign?: "top" | "middle" | "bottom";
  wrapText?: boolean;
  borders?: string;
  color?: string;
  bgColor?: string;
  comment?: string;
  sparkline?: CellSparkline;
  validation?: DataValidationRule;
}

export interface GridRange {
  minRow: number;
  maxRow: number;
  minCol: number;
  maxCol: number;
}

export interface SheetChart {
  id: string;
  type: "column" | "bar" | "line" | "pie" | "scatter" | "doughnut" | "area";
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
  theme?: string;
  themeColors?: string;
  themeFonts?: string;
  margins?: "normal" | "narrow" | "wide";
  pageSize?: "letter" | "a4" | "legal";
  orientation?: "portrait" | "landscape";
  printArea?: GridRange | null;
  printGridlines?: boolean;
  printHeadings?: boolean;
  tableRange?: GridRange | null;
  isProtected?: boolean;
  password?: string;
  viewMode?: "normal" | "pagebreak" | "pagelayout";
  rowGroups?: { startRow: number; endRow: number }[];
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
    case "Fraction": {
      const whole = Math.floor(num);
      const frac = num - whole;
      if (frac === 0) return String(whole);
      const denominator = 16;
      const numerator = Math.round(frac * denominator);
      if (numerator === 0) return String(whole);
      if (numerator === denominator) return String(whole + 1);
      const gcd = (a: number, b: number): number => (b === 0 ? a : gcd(b, a % b));
      const g = gcd(numerator, denominator);
      const simpNum = numerator / g;
      const simpDen = denominator / g;
      return whole > 0 ? `${whole} ${simpNum}/${simpDen}` : `${simpNum}/${simpDen}`;
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
      cells: new Map<string, CellData>(),
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
  formulaBarText: string = "";
  autoFillPreview: GridRange | null = null;
  formulaReferences: { range: GridRange; color: string }[] = [];
  clipboardData: ClipboardPayload | null = null;
  internalClipboardText: string = "";
  showFormulas: boolean = false;
  zoom: number = 1.0;
  workbookProtected: boolean = false;
  workbookPassword?: string;
  viewMode: "normal" | "pagebreak" | "pagelayout" = "normal";
  isSplitWindow: boolean = false;
  showAllNotes: boolean = false;
  definedNames: Map<string, string | number> = new Map();

  public resetToBlank(sheetName = "Sheet1"): void {
    this.sheets = [
      {
        id: "sheet-1",
        name: sheetName,
        cells: new Map<string, CellData>(),
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
        isProtected: false,
        viewMode: "normal",
      },
    ];
    this.activeSheetId = "sheet-1";
    this.formulaBarText = "";
    this.autoFillPreview = null;
    this.formulaReferences = [];
    this.clipboardData = null;
    this.internalClipboardText = "";
    this.undoStack = [];
    this.redoStack = [];
    this.showFormulas = false;
    this.zoom = 1.0;
    this.workbookProtected = false;
    this.workbookPassword = undefined;
    this.viewMode = "normal";
    this.isSplitWindow = false;
    this.showAllNotes = false;
    this.definedNames = new Map();
  }

  public exportCSV(): string {
    const sheet = this.getActiveSheet();
    if (sheet.cells.size === 0) return "";

    let maxRow = 0;
    let maxCol = 0;
    sheet.cells.forEach((_, coord) => {
      const cr = this.coordToColRow(coord);
      if (cr.row > maxRow) maxRow = cr.row;
      if (cr.col > maxCol) maxCol = cr.col;
    });

    const lines: string[] = [];
    for (let r = 0; r <= maxRow; r++) {
      const rowValues: string[] = [];
      for (let c = 0; c <= maxCol; c++) {
        const coord = `${this.colIndexToLetter(c)}${r + 1}`;
        const cell = sheet.cells.get(coord);
        let val = cell ? (cell.computed !== undefined && cell.computed !== "" ? String(cell.computed) : cell.raw || "") : "";
        if (val.includes(",") || val.includes('"') || val.includes("\n")) {
          val = `"${val.replace(/"/g, '""')}"`;
        }
        rowValues.push(val);
      }
      lines.push(rowValues.join(","));
    }
    return lines.join("\n");
  }

  public loadCSV(csvText: string, sheetName = "Sheet1"): void {
    this.resetToBlank(sheetName);
    const lines = csvText.split(/\r?\n/);
    lines.forEach((line, rIdx) => {
      if (line.trim().length === 0 && rIdx === lines.length - 1) return;
      const values: string[] = [];
      let inQuote = false;
      let curr = "";
      for (let i = 0; i < line.length; i++) {
        const char = line[i];
        if (char === '"') {
          if (inQuote && line[i + 1] === '"') {
            curr += '"';
            i++;
          } else {
            inQuote = !inQuote;
          }
        } else if (char === "," && !inQuote) {
          values.push(curr);
          curr = "";
        } else {
          curr += char;
        }
      }
      values.push(curr);

      values.forEach((val, cIdx) => {
        if (val.trim().length > 0) {
          const coord = `${this.colIndexToLetter(cIdx)}${rIdx + 1}`;
          this.setCellValue(coord, val.trim());
        }
      });
    });
    this.recalculateAll();
  }

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

  setCellValue(coord: string, val: string, force = false): boolean {
    if (this.isSheetProtected() && !force) {
      return false;
    }
    this.selectedCell = coord;
    this.commitFormula(val);
    return true;
  }

  getCellValue(coord: string): string | number | boolean | undefined {
    return this.cells.get(coord)?.computed;
  }

  public getCell(coord: string): CellData {
    const sheet = this.getActiveSheet();
    let cell = sheet.cells.get(coord);
    if (!cell) {
      cell = { raw: "", computed: "" };
      sheet.cells.set(coord, cell);
    }
    return cell;
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
      cells: new Map<string, CellData>(),
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

  public selectColumn(colIndex: number, extend = false, multi = false, maxRow = 999): void {
    const colLetter = this.colIndexToLetter(colIndex);
    this.selectedCell = `${colLetter}1`;
    this.formulaBarText = this.cells.get(this.selectedCell)?.raw || "";

    if (extend && this.selectedRanges.length > 0) {
      const cur = this.selectedRanges[this.selectedRanges.length - 1];
      cur.maxCol = colIndex;
      cur.minRow = 0;
      cur.maxRow = maxRow;
    } else if (multi) {
      this.selectedRanges.push({
        minRow: 0,
        maxRow,
        minCol: colIndex,
        maxCol: colIndex,
      });
    } else {
      this.selectedRanges = [
        {
          minRow: 0,
          maxRow,
          minCol: colIndex,
          maxCol: colIndex,
        },
      ];
    }
    this.updateFormulaReferences();
  }

  public selectColumnRange(startCol: number, endCol: number, multi = false, maxRow = 999): void {
    const minCol = Math.min(startCol, endCol);
    const maxCol = Math.max(startCol, endCol);
    this.selectedCell = `${this.colIndexToLetter(minCol)}1`;
    this.formulaBarText = this.cells.get(this.selectedCell)?.raw || "";
    const range: GridRange = {
      minRow: 0,
      maxRow,
      minCol,
      maxCol,
    };
    if (multi) {
      this.selectedRanges.push(range);
    } else {
      this.selectedRanges = [range];
    }
    this.updateFormulaReferences();
  }

  public selectRow(rowIndex: number, extend = false, multi = false, maxCol = 99): void {
    this.selectedCell = `A${rowIndex + 1}`;
    this.formulaBarText = this.cells.get(this.selectedCell)?.raw || "";

    if (extend && this.selectedRanges.length > 0) {
      const cur = this.selectedRanges[this.selectedRanges.length - 1];
      cur.maxRow = rowIndex;
      cur.minCol = 0;
      cur.maxCol = maxCol;
    } else if (multi) {
      this.selectedRanges.push({
        minRow: rowIndex,
        maxRow: rowIndex,
        minCol: 0,
        maxCol,
      });
    } else {
      this.selectedRanges = [
        {
          minRow: rowIndex,
          maxRow: rowIndex,
          minCol: 0,
          maxCol,
        },
      ];
    }
    this.updateFormulaReferences();
  }

  public selectRowRange(startRow: number, endRow: number, multi = false, maxCol = 99): void {
    const minRow = Math.min(startRow, endRow);
    const maxRow = Math.max(startRow, endRow);
    this.selectedCell = `A${minRow + 1}`;
    this.formulaBarText = this.cells.get(this.selectedCell)?.raw || "";
    const range: GridRange = {
      minRow,
      maxRow,
      minCol: 0,
      maxCol,
    };
    if (multi) {
      this.selectedRanges.push(range);
    } else {
      this.selectedRanges = [range];
    }
    this.updateFormulaReferences();
  }

  public selectAllCells(maxRow = 999, maxCol = 99): void {
    this.selectedCell = "A1";
    this.formulaBarText = this.cells.get("A1")?.raw || "";
    this.selectedRanges = [
      {
        minRow: 0,
        maxRow,
        minCol: 0,
        maxCol,
      },
    ];
    this.updateFormulaReferences();
  }

  public selectCellRange(startCoord: string, endCoord: string, multi = false): void {
    const start = this.coordToColRow(startCoord);
    const end = this.coordToColRow(endCoord);
    this.selectedCell = startCoord;
    this.formulaBarText = this.cells.get(startCoord)?.raw || "";
    const range: GridRange = {
      minRow: Math.min(start.row, end.row),
      maxRow: Math.max(start.row, end.row),
      minCol: Math.min(start.col, end.col),
      maxCol: Math.max(start.col, end.col),
    };
    if (multi) {
      this.selectedRanges.push(range);
    } else {
      this.selectedRanges = [range];
    }
    this.updateFormulaReferences();
  }

  public getSelectionDescriptor(): string {
    if (this.selectedRanges.length === 0) {
      return this.selectedCell || "A1";
    }
    const r = this.selectedRanges[this.selectedRanges.length - 1];
    const minRow = Math.min(r.minRow, r.maxRow);
    const maxRow = Math.max(r.minRow, r.maxRow);
    const minCol = Math.min(r.minCol, r.maxCol);
    const maxCol = Math.max(r.minCol, r.maxCol);

    const isFullCol = minRow === 0 && maxRow >= 199;
    const isFullRow = minCol === 0 && maxCol >= 99;

    if (isFullCol && isFullRow) {
      return "1:1000";
    }
    if (isFullCol) {
      const colStart = this.colIndexToLetter(minCol);
      const colEnd = this.colIndexToLetter(maxCol);
      return `${colStart}:${colEnd}`;
    }
    if (isFullRow) {
      return `${minRow + 1}:${maxRow + 1}`;
    }
    if (minRow === maxRow && minCol === maxCol) {
      return `${this.colIndexToLetter(minCol)}${minRow + 1}`;
    }
    return `${this.colIndexToLetter(minCol)}${minRow + 1}:${this.colIndexToLetter(maxCol)}${maxRow + 1}`;
  }

  public selectRangeByName(ref: string): boolean {
    const trimmed = ref.trim().toUpperCase();
    if (!trimmed) return false;

    // Check Col:Col e.g. B:E or B:B
    const colRangeMatch = trimmed.match(/^([A-Z]+):([A-Z]+)$/);
    if (colRangeMatch) {
      let col1 = 0;
      for (let i = 0; i < colRangeMatch[1].length; i++) {
        col1 = col1 * 26 + (colRangeMatch[1].charCodeAt(i) - 64);
      }
      let col2 = 0;
      for (let i = 0; i < colRangeMatch[2].length; i++) {
        col2 = col2 * 26 + (colRangeMatch[2].charCodeAt(i) - 64);
      }
      this.selectColumnRange(col1 - 1, col2 - 1);
      return true;
    }

    // Check Row:Row e.g. 3:10 or 3:3 or 1:1000
    const rowRangeMatch = trimmed.match(/^([0-9]+):([0-9]+)$/);
    if (rowRangeMatch) {
      const r1 = parseInt(rowRangeMatch[1], 10) - 1;
      const r2 = parseInt(rowRangeMatch[2], 10) - 1;
      if (r1 === 0 && r2 >= 999) {
        this.selectAllCells();
        return true;
      }
      this.selectRowRange(r1, r2);
      return true;
    }

    // Check Cell:Cell e.g. B2:D8
    const cellRangeMatch = trimmed.match(/^([A-Z]+[0-9]+):([A-Z]+[0-9]+)$/);
    if (cellRangeMatch) {
      this.selectCellRange(cellRangeMatch[1], cellRangeMatch[2]);
      return true;
    }

    // Check single cell e.g. B2
    if (/^[A-Z]+[0-9]+$/.test(trimmed)) {
      this.selectCell(trimmed);
      return true;
    }

    return false;
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

    const totalCells = (maxRow - minRow + 1) * (maxCol - minCol + 1);
    if (totalCells > this.cells.size * 2 && this.cells.size > 0) {
      this.cells.forEach((cell, coord) => {
        const cr = this.coordToColRow(coord);
        if (cr.row >= minRow && cr.row <= maxRow && cr.col >= minCol && cr.col <= maxCol) {
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
      });
    } else {
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
    let computed: string | number | boolean = val;
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
      fontFamily: current?.fontFamily,
      fontSize: current?.fontSize,
      format: current?.format,
      decimals: current?.decimals,
      align: current?.align,
      valign: current?.valign,
      wrapText: current?.wrapText,
      borders: current?.borders,
      color: current?.color,
      bgColor: current?.bgColor,
      comment: current?.comment,
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

  evaluateFormula(expr: string): string | number | boolean {
    const raw = expr.trim();
    if (!raw.startsWith("=")) return raw;

    let content = raw.slice(1).trim();

    // 0. Defined Names substitution (e.g. TaxRate -> 0.18)
    if (this.definedNames && this.definedNames.size > 0) {
      this.definedNames.forEach((val, name) => {
        const reg = new RegExp(`\\b${name}\\b`, "gi");
        content = content.replace(reg, String(val));
      });
    }

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

    // ROUND(number, [digits])
    if (upper.startsWith("ROUND(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const args = this.splitFormulaArgs(inner);
      const n = Number(this.resolveFormulaValue(args[0]));
      const d = args.length > 1 ? Number(this.resolveFormulaValue(args[1])) : 0;
      if (!isNaN(n) && !isNaN(d)) {
        const factor = Math.pow(10, d);
        return Math.round(n * factor) / factor;
      }
    }

    // RAND()
    if (upper === "RAND()") {
      return Math.round(Math.random() * 10000) / 10000;
    }

    // RANDBETWEEN(min, max)
    if (upper.startsWith("RANDBETWEEN(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const args = this.splitFormulaArgs(inner);
      const min = Math.ceil(Number(this.resolveFormulaValue(args[0])));
      const max = Math.floor(Number(this.resolveFormulaValue(args[1])));
      if (!isNaN(min) && !isNaN(max)) {
        return Math.floor(Math.random() * (max - min + 1)) + min;
      }
    }

    // MEDIAN(range)
    if (upper.startsWith("MEDIAN(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const nums = this.getNumbersFromRangeArg(inner).sort((a, b) => a - b);
      if (nums.length === 0) return 0;
      const mid = Math.floor(nums.length / 2);
      return nums.length % 2 !== 0 ? nums[mid] : (nums[mid - 1] + nums[mid]) / 2;
    }

    // STDEV(range)
    if (upper.startsWith("STDEV(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const nums = this.getNumbersFromRangeArg(inner);
      if (nums.length <= 1) return 0;
      const avg = nums.reduce((a, b) => a + b, 0) / nums.length;
      const sumSq = nums.reduce((a, b) => a + Math.pow(b - avg, 2), 0);
      return Math.round(Math.sqrt(sumSq / (nums.length - 1)) * 10000) / 10000;
    }

    // COUNTIF(range, criteria)
    if (upper.startsWith("COUNTIF(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const args = this.splitFormulaArgs(inner);
      if (args.length >= 2) {
        const rangeCoords = this.getCoordsFromRange(args[0]);
        const criteria = this.resolveFormulaValue(args[1]);
        let count = 0;
        for (const coord of rangeCoords) {
          const cell = this.cells.get(coord);
          const val = cell ? (cell.computed !== undefined ? cell.computed : cell.raw) : "";
          if (this.matchesCriteria(val, criteria)) count++;
        }
        return count;
      }
    }

    // SUMIF(range, criteria, [sum_range])
    if (upper.startsWith("SUMIF(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const args = this.splitFormulaArgs(inner);
      if (args.length >= 2) {
        const rangeCoords = this.getCoordsFromRange(args[0]);
        const sumCoords = args.length > 2 ? this.getCoordsFromRange(args[2]) : rangeCoords;
        const criteria = this.resolveFormulaValue(args[1]);
        let sum = 0;
        for (let i = 0; i < rangeCoords.length; i++) {
          const coord = rangeCoords[i];
          const cell = this.cells.get(coord);
          const val = cell ? (cell.computed !== undefined ? cell.computed : cell.raw) : "";
          if (this.matchesCriteria(val, criteria)) {
            const sumCoord = sumCoords[i] || coord;
            const sumCell = this.cells.get(sumCoord);
            const n = Number(sumCell ? (sumCell.computed !== undefined ? sumCell.computed : sumCell.raw) : 0);
            if (!isNaN(n)) sum += n;
          }
        }
        return sum;
      }
    }

    // SUMIFS(sum_range, crit_range1, crit1, ...)
    if (upper.startsWith("SUMIFS(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const args = this.splitFormulaArgs(inner);
      if (args.length >= 3 && args.length % 2 === 1) {
        const sumCoords = this.getCoordsFromRange(args[0]);
        const pairs: { coords: string[]; crit: string | number | boolean }[] = [];
        for (let i = 1; i < args.length; i += 2) {
          pairs.push({
            coords: this.getCoordsFromRange(args[i]),
            crit: this.resolveFormulaValue(args[i + 1]),
          });
        }
        let total = 0;
        for (let idx = 0; idx < sumCoords.length; idx++) {
          let allMatch = true;
          for (const p of pairs) {
            const checkCoord = p.coords[idx];
            const cell = checkCoord ? this.cells.get(checkCoord) : null;
            const val = cell ? (cell.computed !== undefined ? cell.computed : cell.raw) : "";
            if (!this.matchesCriteria(val, p.crit)) {
              allMatch = false;
              break;
            }
          }
          if (allMatch) {
            const sc = sumCoords[idx];
            const sCell = this.cells.get(sc);
            const num = Number(sCell ? (sCell.computed !== undefined ? sCell.computed : sCell.raw) : 0);
            if (!isNaN(num)) total += num;
          }
        }
        return total;
      }
    }

    // AVERAGEIF(range, criteria, [avg_range])
    if (upper.startsWith("AVERAGEIF(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const args = this.splitFormulaArgs(inner);
      if (args.length >= 2) {
        const rangeCoords = this.getCoordsFromRange(args[0]);
        const avgCoords = args.length > 2 ? this.getCoordsFromRange(args[2]) : rangeCoords;
        const criteria = this.resolveFormulaValue(args[1]);
        let sum = 0;
        let count = 0;
        for (let i = 0; i < rangeCoords.length; i++) {
          const coord = rangeCoords[i];
          const cell = this.cells.get(coord);
          const val = cell ? (cell.computed !== undefined ? cell.computed : cell.raw) : "";
          if (this.matchesCriteria(val, criteria)) {
            const avgCoord = avgCoords[i] || coord;
            const avgCell = this.cells.get(avgCoord);
            const n = Number(avgCell ? (avgCell.computed !== undefined ? avgCell.computed : avgCell.raw) : 0);
            if (!isNaN(n)) {
              sum += n;
              count++;
            }
          }
        }
        return count > 0 ? Math.round((sum / count) * 10000) / 10000 : 0;
      }
    }

    // 2. Logical IF function: =IF(condition, true_value, false_value)
    if (upper.startsWith("IF(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const args = this.splitFormulaArgs(inner);
      if (args.length >= 2) {
        const condStr = args[0];
        const trueExpr = args[1];
        const falseExpr = args.length > 2 ? args[2] : '""';
        const isTrue = this.evaluateCondition(condStr);
        const chosen = isTrue ? trueExpr : falseExpr;
        return this.resolveFormulaValue(chosen);
      }
    }

    if (upper.startsWith("AND(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const args = this.splitFormulaArgs(inner);
      return args.every((a) => this.evaluateCondition(a));
    }

    if (upper.startsWith("OR(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const args = this.splitFormulaArgs(inner);
      return args.some((a) => this.evaluateCondition(a));
    }

    if (upper.startsWith("NOT(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      return !this.evaluateCondition(inner);
    }

    if (upper.startsWith("IFERROR(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const args = this.splitFormulaArgs(inner);
      if (args.length >= 2) {
        try {
          const res = this.resolveFormulaValue(args[0]);
          const str = String(res);
          if (str.startsWith("#") || str === "NaN" || res === undefined || res === null) {
            return this.resolveFormulaValue(args[1]);
          }
          return res;
        } catch {
          return this.resolveFormulaValue(args[1]);
        }
      }
    }

    if (upper.startsWith("IFS(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const args = this.splitFormulaArgs(inner);
      for (let i = 0; i < args.length; i += 2) {
        if (this.evaluateCondition(args[i])) {
          return this.resolveFormulaValue(args[i + 1]);
        }
      }
      return "#N/A";
    }

    // 3. Text Functions
    if ((upper.startsWith("CONCAT(") || upper.startsWith("CONCATENATE(")) && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const args = this.splitFormulaArgs(inner);
      let out = "";
      for (const a of args) {
        if (a.includes(":")) {
          const coords = this.getCoordsFromRange(a);
          for (const c of coords) {
            const cell = this.cells.get(c);
            out += cell ? String(cell.computed !== undefined ? cell.computed : cell.raw) : "";
          }
        } else {
          out += String(this.resolveFormulaValue(a));
        }
      }
      return out;
    }

    if (upper.startsWith("LEFT(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const args = this.splitFormulaArgs(inner);
      const str = String(this.resolveFormulaValue(args[0]));
      const n = args.length > 1 ? Number(this.resolveFormulaValue(args[1])) : 1;
      return str.slice(0, n);
    }

    if (upper.startsWith("RIGHT(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const args = this.splitFormulaArgs(inner);
      const str = String(this.resolveFormulaValue(args[0]));
      const n = args.length > 1 ? Number(this.resolveFormulaValue(args[1])) : 1;
      return n >= str.length ? str : str.slice(str.length - n);
    }

    if (upper.startsWith("MID(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const args = this.splitFormulaArgs(inner);
      const str = String(this.resolveFormulaValue(args[0]));
      const start = Math.max(1, Number(this.resolveFormulaValue(args[1]))) - 1;
      const len = Number(this.resolveFormulaValue(args[2]));
      return str.substr(start, len);
    }

    if (upper.startsWith("LEN(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const str = String(this.resolveFormulaValue(inner));
      return str.length;
    }

    if (upper.startsWith("UPPER(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      return String(this.resolveFormulaValue(inner)).toUpperCase();
    }

    if (upper.startsWith("LOWER(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      return String(this.resolveFormulaValue(inner)).toLowerCase();
    }

    if (upper.startsWith("TRIM(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      return String(this.resolveFormulaValue(inner)).trim();
    }

    if (upper.startsWith("SUBSTITUTE(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const args = this.splitFormulaArgs(inner);
      const text = String(this.resolveFormulaValue(args[0]));
      const oldT = String(this.resolveFormulaValue(args[1]));
      const newT = String(this.resolveFormulaValue(args[2]));
      return text.split(oldT).join(newT);
    }

    // 4. Date & Time Functions
    if (upper === "TODAY()") {
      const d = new Date();
      return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
    }

    if (upper === "NOW()") {
      const d = new Date();
      return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")} ${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
    }

    if (upper.startsWith("YEAR(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const val = String(this.resolveFormulaValue(inner));
      const parsed = new Date(val);
      return !isNaN(parsed.getFullYear()) ? parsed.getFullYear() : 0;
    }

    if (upper.startsWith("MONTH(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const val = String(this.resolveFormulaValue(inner));
      const parsed = new Date(val);
      return !isNaN(parsed.getMonth()) ? parsed.getMonth() + 1 : 0;
    }

    if (upper.startsWith("DAY(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const val = String(this.resolveFormulaValue(inner));
      const parsed = new Date(val);
      return !isNaN(parsed.getDate()) ? parsed.getDate() : 0;
    }

    if (upper.startsWith("DATEDIF(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const args = this.splitFormulaArgs(inner);
      const d1 = new Date(String(this.resolveFormulaValue(args[0])));
      const d2 = new Date(String(this.resolveFormulaValue(args[1])));
      const unit = String(this.resolveFormulaValue(args[2])).toUpperCase();
      const diffMs = d2.getTime() - d1.getTime();
      const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
      if (unit === "D") return diffDays;
      if (unit === "M") return (d2.getFullYear() - d1.getFullYear()) * 12 + (d2.getMonth() - d1.getMonth());
      if (unit === "Y") return d2.getFullYear() - d1.getFullYear();
      return diffDays;
    }

    // 5. Financial Functions
    if (upper.startsWith("PMT(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const args = this.splitFormulaArgs(inner);
      const rate = Number(this.resolveFormulaValue(args[0]));
      const nper = Number(this.resolveFormulaValue(args[1]));
      const pv = Number(this.resolveFormulaValue(args[2]));
      if (rate === 0) return Math.round((-pv / nper) * 100) / 100;
      const pmt = (rate * pv) / (1 - Math.pow(1 + rate, -nper));
      return -Math.round(pmt * 100) / 100;
    }

    if (upper.startsWith("FV(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const args = this.splitFormulaArgs(inner);
      const rate = Number(this.resolveFormulaValue(args[0]));
      const nper = Number(this.resolveFormulaValue(args[1]));
      const pmt = Number(this.resolveFormulaValue(args[2]));
      if (rate === 0) return -(pmt * nper);
      const fv = -(pmt * (Math.pow(1 + rate, nper) - 1)) / rate;
      return Math.round(fv * 100) / 100;
    }

    if (upper.startsWith("PV(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const args = this.splitFormulaArgs(inner);
      const rate = Number(this.resolveFormulaValue(args[0]));
      const nper = Number(this.resolveFormulaValue(args[1]));
      const pmt = Number(this.resolveFormulaValue(args[2]));
      if (rate === 0) return -(pmt * nper);
      const pv = -(pmt * (1 - Math.pow(1 + rate, -nper))) / rate;
      return Math.round(pv * 100) / 100;
    }

    if (upper.startsWith("NPV(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const args = this.splitFormulaArgs(inner);
      const rate = Number(this.resolveFormulaValue(args[0]));
      const values: number[] = [];
      for (let i = 1; i < args.length; i++) {
        if (args[i].includes(":")) {
          values.push(...this.getNumbersFromRangeArg(args[i]));
        } else {
          const n = Number(this.resolveFormulaValue(args[i]));
          if (!isNaN(n)) values.push(n);
        }
      }
      let npv = 0;
      for (let t = 0; t < values.length; t++) {
        npv += values[t] / Math.pow(1 + rate, t + 1);
      }
      return Math.round(npv * 100) / 100;
    }

    if (upper.startsWith("IRR(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const args = this.splitFormulaArgs(inner);
      const values = this.getNumbersFromRangeArg(args[0]);
      if (values.length < 2) return "#VALUE!";
      let rate = args.length > 1 ? Number(this.resolveFormulaValue(args[1])) : 0.1;
      for (let iter = 0; iter < 100; iter++) {
        let npv = 0;
        let dNpv = 0;
        for (let t = 0; t < values.length; t++) {
          npv += values[t] / Math.pow(1 + rate, t);
          dNpv -= (t * values[t]) / Math.pow(1 + rate, t + 1);
        }
        if (Math.abs(dNpv) < 1e-10) break;
        const newRate = rate - npv / dNpv;
        if (Math.abs(newRate - rate) < 1e-6) {
          rate = newRate;
          break;
        }
        rate = newRate;
      }
      return Math.round(rate * 10000) / 10000;
    }

    // 6. Lookup & Reference
    if (upper.startsWith("XLOOKUP(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const args = this.splitFormulaArgs(inner);
      if (args.length >= 3) {
        const lookupVal = this.resolveFormulaValue(args[0]);
        const lookCoords = this.getCoordsFromRange(args[1]);
        const retCoords = this.getCoordsFromRange(args[2]);
        const notFound = args.length > 3 ? this.resolveFormulaValue(args[3]) : "#N/A";

        const searchStr = String(lookupVal).trim().toLowerCase();
        let matchedIdx = -1;
        for (let i = 0; i < lookCoords.length; i++) {
          const cell = this.cells.get(lookCoords[i]);
          const val = cell ? (cell.computed !== undefined ? cell.computed : cell.raw) : "";
          if (String(val).trim().toLowerCase() === searchStr) {
            matchedIdx = i;
            break;
          }
        }
        if (matchedIdx >= 0 && matchedIdx < retCoords.length) {
          const retCell = this.cells.get(retCoords[matchedIdx]);
          return retCell ? (retCell.computed !== undefined ? retCell.computed : retCell.raw) : "";
        }
        return notFound;
      }
    }

    if (upper.startsWith("INDEX(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const args = this.splitFormulaArgs(inner);
      if (args.length >= 2) {
        const rangeStr = args[0].replace(/\$/g, "");
        const rowNum = Number(this.resolveFormulaValue(args[1]));
        const colNum = args.length > 2 ? Number(this.resolveFormulaValue(args[2])) : 1;
        const [start, end] = rangeStr.split(":");
        const c1 = this.coordToColRow(start);
        const c2 = end ? this.coordToColRow(end) : c1;
        const targetRow = Math.min(c1.row, c2.row) + (rowNum - 1);
        const targetCol = Math.min(c1.col, c2.col) + (colNum - 1);
        const coord = `${this.colIndexToLetter(targetCol)}${targetRow + 1}`;
        const cell = this.cells.get(coord);
        return cell ? (cell.computed !== undefined ? cell.computed : cell.raw) : "";
      }
    }

    if (upper.startsWith("MATCH(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const args = this.splitFormulaArgs(inner);
      if (args.length >= 2) {
        const lookupVal = this.resolveFormulaValue(args[0]);
        const coords = this.getCoordsFromRange(args[1]);
        const searchStr = String(lookupVal).trim().toLowerCase();
        for (let i = 0; i < coords.length; i++) {
          const cell = this.cells.get(coords[i]);
          const val = cell ? (cell.computed !== undefined ? cell.computed : cell.raw) : "";
          if (String(val).trim().toLowerCase() === searchStr) {
            return i + 1;
          }
        }
        return "#N/A";
      }
    }

    // 7. Lookup function: =VLOOKUP(lookup_value, table_range, col_index, [range_lookup])
    if (upper.startsWith("VLOOKUP(") && upper.endsWith(")")) {
      const inner = content.slice(content.indexOf("(") + 1, content.lastIndexOf(")")).trim();
      const args = this.splitFormulaArgs(inner);
      if (args.length >= 3) {
        const lookupVal = this.resolveFormulaValue(args[0]);
        const rangeStr = args[1].trim().replace(/\$/g, "");
        const colIdx = parseInt(args[2].trim(), 10);
        if (isNaN(colIdx) || colIdx < 1) return "#VALUE!";
        if (!rangeStr.includes(":")) return "#REF!";

        const [startCoord, endCoord] = rangeStr.split(":");
        const start = this.coordToColRow(startCoord);
        const end = this.coordToColRow(endCoord);

        const minCol = Math.min(start.col, end.col);
        const maxCol = Math.max(start.col, end.col);
        const minRow = Math.min(start.row, end.row);
        const maxRow = Math.max(start.row, end.row);

        const targetCol = minCol + (colIdx - 1);
        if (targetCol > maxCol) return "#REF!";

        const searchStr = String(lookupVal).trim().toLowerCase();
        const searchNum = typeof lookupVal === "number" ? lookupVal : Number(lookupVal);
        const isSearchNumeric = !isNaN(searchNum) && String(lookupVal).trim() !== "";

        let matchedRow = -1;
        for (let r = minRow; r <= maxRow; r++) {
          const firstCoord = `${this.colIndexToLetter(minCol)}${r + 1}`;
          const cell = this.cells.get(firstCoord);
          if (!cell) continue;
          const val = cell.computed !== undefined ? cell.computed : cell.raw;
          const cellStr = String(val).trim().toLowerCase();
          const cellNum = typeof val === "number" ? val : Number(val);

          if (isSearchNumeric && !isNaN(cellNum) && cellNum === searchNum) {
            matchedRow = r;
            break;
          } else if (cellStr === searchStr) {
            matchedRow = r;
            break;
          }
        }

        if (matchedRow !== -1) {
          const resultCoord = `${this.colIndexToLetter(targetCol)}${matchedRow + 1}`;
          const resultCell = this.cells.get(resultCoord);
          return resultCell ? (resultCell.computed !== undefined ? resultCell.computed : resultCell.raw) : "";
        }

        return "#N/A";
      }
    }

    // 4. Arithmetic expressions with cell coordinates
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
        const jsExpr = evalExpr.replace(/\^/g, "**");
        const fn = new Function(`return (${jsExpr});`);
        const result = fn();
        if (typeof result === "number" && !isNaN(result) && isFinite(result)) {
          return Math.round(result * 10000) / 10000;
        }
      }
    } catch {
      // Ignore evaluation errors
    }

    // 5. Single Cell Reference, e.g. =A1
    if (/^\$?[A-Z]+\$?[0-9]+$/i.test(content)) {
      const cell = this.cells.get(content.replace(/\$/g, "").toUpperCase());
      return cell ? cell.computed : "";
    }

    return expr;
  }

  private splitFormulaArgs(argStr: string): string[] {
    const args: string[] = [];
    let cur = "";
    let depth = 0;
    let inQuote = false;
    let quoteChar = "";

    for (let i = 0; i < argStr.length; i++) {
      const ch = argStr[i];
      if ((ch === '"' || ch === "'") && !inQuote) {
        inQuote = true;
        quoteChar = ch;
        cur += ch;
      } else if (ch === quoteChar && inQuote) {
        inQuote = false;
        cur += ch;
      } else if (!inQuote && (ch === "(" || ch === "[")) {
        depth++;
        cur += ch;
      } else if (!inQuote && (ch === ")" || ch === "]")) {
        depth--;
        cur += ch;
      } else if (!inQuote && depth === 0 && ch === ",") {
        args.push(cur.trim());
        cur = "";
      } else {
        cur += ch;
      }
    }
    if (cur.trim().length > 0) {
      args.push(cur.trim());
    }
    return args;
  }

  private evaluateCondition(condStr: string): boolean {
    const s = condStr.trim();
    const ops = [">=", "<=", "<>", "!=", "==", "=", ">", "<"];
    for (const op of ops) {
      const idx = s.indexOf(op);
      if (idx !== -1) {
        const leftRaw = s.slice(0, idx).trim();
        const rightRaw = s.slice(idx + op.length).trim();
        const leftVal = this.resolveFormulaValue(leftRaw);
        const rightVal = this.resolveFormulaValue(rightRaw);

        const leftNum = typeof leftVal === "number" ? leftVal : Number(leftVal);
        const rightNum = typeof rightVal === "number" ? rightVal : Number(rightVal);
        const bothNumbers = !isNaN(leftNum) && !isNaN(rightNum) && typeof leftVal !== "boolean" && typeof rightVal !== "boolean";

        switch (op) {
          case ">=":
            return bothNumbers ? leftNum >= rightNum : String(leftVal) >= String(rightVal);
          case "<=":
            return bothNumbers ? leftNum <= rightNum : String(leftVal) <= String(rightVal);
          case "<>":
          case "!=":
            return bothNumbers ? leftNum !== rightNum : String(leftVal) !== String(rightVal);
          case "==":
          case "=":
            return bothNumbers ? leftNum === rightNum : String(leftVal).toLowerCase() === String(rightVal).toLowerCase();
          case ">":
            return bothNumbers ? leftNum > rightNum : String(leftVal) > String(rightVal);
          case "<":
            return bothNumbers ? leftNum < rightNum : String(leftVal) < String(rightVal);
        }
      }
    }

    const single = this.resolveFormulaValue(s);
    if (typeof single === "boolean") return single;
    if (typeof single === "number") return single !== 0;
    if (typeof single === "string") {
      const lower = single.trim().toLowerCase();
      if (lower === "true") return true;
      if (lower === "false" || lower === "0" || lower === "") return false;
      return true;
    }
    return Boolean(single);
  }

  private resolveFormulaValue(valStr: string): string | number | boolean {
    const s = valStr.trim();
    if (!s) return "";
    if ((s.startsWith('"') && s.endsWith('"')) || (s.startsWith("'") && s.endsWith("'"))) {
      return s.slice(1, -1);
    }
    if (s.startsWith("=")) {
      return this.evaluateFormula(s);
    }
    if (/^\$?[A-Z]+\$?[0-9]+$/i.test(s)) {
      const clean = s.replace(/\$/g, "").toUpperCase();
      const cell = this.cells.get(clean);
      return cell ? cell.computed : "";
    }
    const n = Number(s);
    if (!isNaN(n)) {
      return n;
    }
    if (/^[0-9+\-*/().\s^%]+$/.test(s)) {
      try {
        const jsExpr = s.replace(/\^/g, "**");
        const res = new Function(`return (${jsExpr});`)();
        if (typeof res === "number" && !isNaN(res)) return res;
      } catch {}
    }
    return s;
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

  public getCoordsFromRange(rangeStr: string): string[] {
    const clean = rangeStr.trim().replace(/\$/g, "").toUpperCase();
    const coords: string[] = [];
    if (clean.includes(":")) {
      const [start, end] = clean.split(":");
      const c1 = this.coordToColRow(start);
      const c2 = this.coordToColRow(end);
      for (let r = Math.min(c1.row, c2.row); r <= Math.max(c1.row, c2.row); r++) {
        for (let c = Math.min(c1.col, c2.col); c <= Math.max(c1.col, c2.col); c++) {
          coords.push(`${this.colIndexToLetter(c)}${r + 1}`);
        }
      }
    } else {
      coords.push(clean);
    }
    return coords;
  }

  public matchesCriteria(cellVal: unknown, criteria: unknown): boolean {
    const cStr = String(criteria).trim();
    const vNum = Number(cellVal);
    const isVNum = !isNaN(vNum) && cellVal !== "" && cellVal !== null && cellVal !== undefined;

    if (cStr.startsWith(">=")) {
      const num = Number(cStr.slice(2).trim());
      return isVNum && vNum >= num;
    }
    if (cStr.startsWith("<=")) {
      const num = Number(cStr.slice(2).trim());
      return isVNum && vNum <= num;
    }
    if (cStr.startsWith("<>")) {
      const num = Number(cStr.slice(2).trim());
      return isVNum ? vNum !== num : String(cellVal).toLowerCase() !== cStr.slice(2).trim().toLowerCase();
    }
    if (cStr.startsWith(">")) {
      const num = Number(cStr.slice(1).trim());
      return isVNum && vNum > num;
    }
    if (cStr.startsWith("<")) {
      const num = Number(cStr.slice(1).trim());
      return isVNum && vNum < num;
    }
    if (cStr.startsWith("=")) {
      const target = cStr.slice(1).trim();
      return String(cellVal).toLowerCase() === target.toLowerCase();
    }
    return String(cellVal).toLowerCase() === cStr.toLowerCase();
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

        let computed: string | number | boolean = val;
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

  public pasteValues(tsvText?: string, targetCoord?: string): void {
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

        let computed: string | number | boolean = val;
        const n = Number(val);
        if (!isNaN(n) && val.trim() !== "") computed = n;

        const existing = this.cells.get(cellCoord);
        if (existing) {
          existing.raw = val;
          existing.computed = computed;
        } else {
          this.cells.set(cellCoord, { raw: val, computed });
        }
      });
    });

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

  public getAllTargetRanges(range?: GridRange): GridRange[] {
    if (range) {
      return [
        {
          minRow: Math.min(range.minRow, range.maxRow),
          maxRow: Math.max(range.minRow, range.maxRow),
          minCol: Math.min(range.minCol, range.maxCol),
          maxCol: Math.max(range.minCol, range.maxCol),
        },
      ];
    }
    if (this.selectedRanges.length === 0) {
      const cr = this.coordToColRow(this.selectedCell);
      return [{ minRow: cr.row, maxRow: cr.row, minCol: cr.col, maxCol: cr.col }];
    }
    return this.selectedRanges.map((r) => ({
      minRow: Math.min(r.minRow, r.maxRow),
      maxRow: Math.max(r.minRow, r.maxRow),
      minCol: Math.min(r.minCol, r.maxCol),
      maxCol: Math.max(r.minCol, r.maxCol),
    }));
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
    const targets = this.getAllTargetRanges(range);
    const visited = new Set<string>();
    for (const target of targets) {
      for (let r = target.minRow; r <= target.maxRow; r++) {
        for (let c = target.minCol; c <= target.maxCol; c++) {
          const coord = `${this.colIndexToLetter(c)}${r + 1}`;
          if (!visited.has(coord)) {
            visited.add(coord);
            const cell = this.getOrCreateCell(coord);
            fn(cell, coord, r, c);
          }
        }
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

  public setFontFamily(font: string, range?: GridRange): void {
    this.forEachCellInRange(range, (cell) => {
      cell.fontFamily = font;
    });
  }

  public setFontSize(size: number, range?: GridRange): void {
    this.forEachCellInRange(range, (cell) => {
      cell.fontSize = size;
    });
  }

  public clearSelection(range?: GridRange): void {
    this.forEachCellInRange(range, (cell) => {
      cell.raw = "";
      cell.computed = "";
    });
  }

  public increaseFontSize(range?: GridRange): void {
    this.forEachCellInRange(range, (cell) => {
      const cur = cell.fontSize || 11;
      cell.fontSize = Math.min(72, cur + (cur >= 14 ? 2 : 1));
    });
  }

  public decreaseFontSize(range?: GridRange): void {
    this.forEachCellInRange(range, (cell) => {
      const cur = cell.fontSize || 11;
      cell.fontSize = Math.max(6, cur - (cur > 14 ? 2 : 1));
    });
  }

  public deleteComment(range?: GridRange): void {
    this.forEachCellInRange(range, (cell) => {
      delete cell.comment;
    });
  }

  public setPageSize(size: "letter" | "a4" | "legal"): void {
    this.getActiveSheet().pageSize = size;
  }

  private savedFormat: Partial<CellData> | null = null;
  public formatPainterActive = false;

  public copyFormat(coord?: string): void {
    const c = coord || this.selectedCell;
    const cell = this.getCell(c);
    this.savedFormat = {
      bold: cell.bold,
      italic: cell.italic,
      underline: cell.underline,
      fontFamily: cell.fontFamily,
      fontSize: cell.fontSize,
      color: cell.color,
      bgColor: cell.bgColor,
      format: cell.format,
      decimals: cell.decimals,
      align: cell.align,
      valign: cell.valign,
      wrapText: cell.wrapText,
      borders: cell.borders,
    };
    this.formatPainterActive = true;
  }

  public applyFormat(range?: GridRange): void {
    if (!this.savedFormat) return;
    const fmt = this.savedFormat;
    this.forEachCellInRange(range, (cell) => {
      if (fmt.bold !== undefined) cell.bold = fmt.bold;
      if (fmt.italic !== undefined) cell.italic = fmt.italic;
      if (fmt.underline !== undefined) cell.underline = fmt.underline;
      if (fmt.fontFamily !== undefined) cell.fontFamily = fmt.fontFamily;
      if (fmt.fontSize !== undefined) cell.fontSize = fmt.fontSize;
      if (fmt.color !== undefined) cell.color = fmt.color;
      if (fmt.bgColor !== undefined) cell.bgColor = fmt.bgColor;
      if (fmt.format !== undefined) cell.format = fmt.format;
      if (fmt.decimals !== undefined) cell.decimals = fmt.decimals;
      if (fmt.align !== undefined) cell.align = fmt.align;
      if (fmt.valign !== undefined) cell.valign = fmt.valign;
      if (fmt.wrapText !== undefined) cell.wrapText = fmt.wrapText;
      if (fmt.borders !== undefined) cell.borders = fmt.borders;
    });
  }

  public toggleFormatPainter(): void {
    if (this.formatPainterActive && this.savedFormat) {
      this.applyFormat();
      this.formatPainterActive = false;
    } else {
      this.copyFormat();
    }
  }

  public insertComment(commentText: string): void {
    const cell = this.getOrCreateCell(this.selectedCell);
    cell.comment = commentText;
  }

  public toggleShowFormulas(): boolean {
    this.showFormulas = !this.showFormulas;
    return this.showFormulas;
  }

  public setZoom(zoom: number): void {
    this.zoom = Math.max(0.5, Math.min(3.0, zoom));
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
    const rowRecords: { rowIdx: number; cells: Map<number, CellData>; key: string | number | boolean }[] = [];

    for (let r = target.minRow; r <= target.maxRow; r++) {
      const rowMap = new Map<number, CellData>();
      let sortKey: string | number | boolean = "";
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
    type: "column" | "bar" | "line" | "pie" | "scatter" | "doughnut" | "area" = "column",
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

  public insertSparkline(type: "line" | "column" | "winloss" = "line", data?: number[], color?: string): void {
    const sheet = this.getActiveSheet();
    const cell = this.getCell(this.selectedCell);
    cell.sparkline = {
      type,
      data: data || (type === "winloss" ? [1, -1, 1, 1, -1, 1, -1] : [12, 19, 8, 25, 32, 28, 45]),
      color: color || (type === "winloss" ? "#107c41" : "#2563eb"),
    };
    sheet.cells.set(this.selectedCell, cell);
  }

  public formatAsTable(range?: GridRange): void {
    const target = this.getTargetRange(range);
    const sheet = this.getActiveSheet();
    sheet.tableRange = target;
    for (let r = target.minRow; r <= target.maxRow; r++) {
      for (let c = target.minCol; c <= target.maxCol; c++) {
        const coord = `${this.colIndexToLetter(c)}${r + 1}`;
        let cell = sheet.cells.get(coord);
        if (!cell) {
          cell = { raw: "", computed: "" };
          sheet.cells.set(coord, cell);
        }
        cell.borders = "all";
        if (r === target.minRow) {
          cell.bgColor = "#107c41";
          cell.color = "#ffffff";
          cell.bold = true;
          if (!cell.raw) {
            cell.raw = `Column ${c + 1}`;
            cell.computed = cell.raw;
          }
        } else {
          cell.bgColor = r % 2 === 0 ? "#f8fafc" : "#ffffff";
          cell.color = "#1b1b1b";
        }
      }
    }
  }

  public generatePivotSummary(targetCoord = "G1"): void {
    const sheet = this.getActiveSheet();
    const sourceRange = this.getTargetRange();
    const groupSums = new Map<string, number>();
    let grandTotal = 0;

    for (let r = sourceRange.minRow; r <= sourceRange.maxRow; r++) {
      const catCoord = `${this.colIndexToLetter(sourceRange.minCol)}${r + 1}`;
      const valCoord = `${this.colIndexToLetter(sourceRange.minCol + 1)}${r + 1}`;
      const catVal = String(sheet.cells.get(catCoord)?.computed ?? sheet.cells.get(catCoord)?.raw ?? "").trim();
      const numVal = Number(sheet.cells.get(valCoord)?.computed ?? sheet.cells.get(valCoord)?.raw ?? 0) || 0;
      if (catVal) {
        groupSums.set(catVal, (groupSums.get(catVal) || 0) + numVal);
        grandTotal += numVal;
      }
    }

    if (groupSums.size === 0) {
      groupSums.set("North", 15200);
      groupSums.set("South", 9800);
      groupSums.set("East", 12400);
      groupSums.set("West", 18100);
      grandTotal = 55500;
    }

    const tcr = this.coordToColRow(targetCoord);
    let curRow = tcr.row;

    const h1 = `${this.colIndexToLetter(tcr.col)}${curRow + 1}`;
    const h2 = `${this.colIndexToLetter(tcr.col + 1)}${curRow + 1}`;
    sheet.cells.set(h1, { raw: "Row Labels", computed: "Row Labels", bold: true, bgColor: "#1f497d", color: "#ffffff", borders: "all" });
    sheet.cells.set(h2, { raw: "Sum of Value", computed: "Sum of Value", bold: true, bgColor: "#1f497d", color: "#ffffff", borders: "all" });
    curRow++;

    groupSums.forEach((sum, cat) => {
      const c1 = `${this.colIndexToLetter(tcr.col)}${curRow + 1}`;
      const c2 = `${this.colIndexToLetter(tcr.col + 1)}${curRow + 1}`;
      sheet.cells.set(c1, { raw: cat, computed: cat, borders: "all" });
      sheet.cells.set(c2, { raw: String(sum), computed: sum, format: "Currency", borders: "all" });
      curRow++;
    });

    const gt1 = `${this.colIndexToLetter(tcr.col)}${curRow + 1}`;
    const gt2 = `${this.colIndexToLetter(tcr.col + 1)}${curRow + 1}`;
    sheet.cells.set(gt1, { raw: "Grand Total", computed: "Grand Total", bold: true, bgColor: "#f1f5f9", borders: "bottom" });
    sheet.cells.set(gt2, { raw: String(grandTotal), computed: grandTotal, bold: true, format: "Currency", bgColor: "#f1f5f9", borders: "bottom" });
  }

  public setTheme(theme: string): void {
    const sheet = this.getActiveSheet();
    sheet.theme = theme;
  }

  public setThemeColors(colors = "Office"): void {
    const sheet = this.getActiveSheet();
    sheet.themeColors = colors;
  }

  public setThemeFonts(fonts = "Calibri"): void {
    const sheet = this.getActiveSheet();
    sheet.themeFonts = fonts;
  }

  public setMargins(margins: "normal" | "narrow" | "wide"): void {
    const sheet = this.getActiveSheet();
    sheet.margins = margins;
  }

  public setOrientation(orientation: "portrait" | "landscape"): void {
    const sheet = this.getActiveSheet();
    sheet.orientation = orientation;
  }

  public setPrintArea(range?: GridRange): void {
    const sheet = this.getActiveSheet();
    sheet.printArea = this.getTargetRange(range);
  }

  public clearPrintArea(): void {
    const sheet = this.getActiveSheet();
    sheet.printArea = null;
  }

  public togglePrintGrid(): boolean {
    const sheet = this.getActiveSheet();
    sheet.printGridlines = sheet.printGridlines !== undefined ? !sheet.printGridlines : true;
    return sheet.printGridlines;
  }

  public togglePrintHeadings(): boolean {
    const sheet = this.getActiveSheet();
    sheet.printHeadings = sheet.printHeadings !== undefined ? !sheet.printHeadings : true;
    return sheet.printHeadings;
  }

  public autoFitColumnWidth(col?: number): void {
    const targetCol = col !== undefined ? col : this.coordToColRow(this.selectedCell).col;
    let maxLen = 8;
    const sheet = this.getActiveSheet();
    sheet.cells.forEach((cell, coord) => {
      const cr = this.coordToColRow(coord);
      if (cr.col === targetCol) {
        const text = String(cell.computed ?? cell.raw ?? "");
        if (text.length > maxLen) maxLen = text.length;
      }
    });
    const newWidth = Math.min(300, Math.max(40, maxLen * 9 + 20));
    sheet.colWidths.set(targetCol, newWidth);
  }

  public autoFitRowHeight(row?: number): void {
    const targetRow = row !== undefined ? row : this.coordToColRow(this.selectedCell).row;
    let maxFontSize = 12;
    const sheet = this.getActiveSheet();
    sheet.cells.forEach((cell, coord) => {
      const cr = this.coordToColRow(coord);
      if (cr.row === targetRow) {
        if (cell.fontSize && cell.fontSize > maxFontSize) maxFontSize = cell.fontSize;
      }
    });
    const newHeight = Math.max(22, maxFontSize * 1.5);
    sheet.rowHeights.set(targetRow, newHeight);
  }

  public fillDown(range?: GridRange): void {
    const target = this.getTargetRange(range);
    if (target.maxRow > target.minRow) {
      const source: GridRange = {
        minRow: target.minRow,
        maxRow: target.minRow,
        minCol: target.minCol,
        maxCol: target.maxCol,
      };
      this.applyAutoFill(source, target);
    }
  }

  public fillRight(range?: GridRange): void {
    const target = this.getTargetRange(range);
    if (target.maxCol > target.minCol) {
      const source: GridRange = {
        minRow: target.minRow,
        maxRow: target.maxRow,
        minCol: target.minCol,
        maxCol: target.minCol,
      };
      this.applyAutoFill(source, target);
    }
  }

  public fillSeries(range?: GridRange): void {
    this.fillDown(range);
  }

  public clearFormats(range?: GridRange): void {
    this.forEachCellInRange(range, (cell) => {
      delete cell.bold;
      delete cell.italic;
      delete cell.underline;
      delete cell.fontFamily;
      delete cell.fontSize;
      delete cell.align;
      delete cell.valign;
      delete cell.wrapText;
      delete cell.borders;
      delete cell.color;
      delete cell.bgColor;
      delete cell.format;
      delete cell.decimals;
    });
  }

  public clearContents(range?: GridRange): void {
    this.forEachCellInRange(range, (cell) => {
      cell.raw = "";
      cell.computed = "";
    });
  }

  public defineName(name: string, value: string | number): void {
    this.definedNames.set(name, value);
    this.recalculateAll();
  }

  public getName(name: string): string | number | undefined {
    return this.definedNames.get(name);
  }

  public getAllNames(): { name: string; value: string | number }[] {
    const list: { name: string; value: string | number }[] = [];
    this.definedNames.forEach((val, name) => {
      list.push({ name, value: val });
    });
    return list;
  }

  public deleteName(name: string): boolean {
    const res = this.definedNames.delete(name);
    if (res) this.recalculateAll();
    return res;
  }

  public tracePrecedents(coord?: string): { range: GridRange; color: string }[] {
    const target = coord || this.selectedCell;
    const cell = this.cells.get(target);
    this.formulaReferences = [];
    if (!cell || !cell.raw || !cell.raw.startsWith("=")) return [];

    const text = cell.raw.slice(1);
    const regex = /(\$?[A-Z]+\$?[0-9]+)(?::(\$?[A-Z]+\$?[0-9]+))?/gi;
    const palette = ["#2563eb", "#059669", "#d97706", "#7c3aed", "#e11d48"];
    let match: RegExpExecArray | null;
    let idx = 0;

    while ((match = regex.exec(text)) !== null) {
      const s = match[1].replace(/\$/g, "");
      const e = match[2] ? match[2].replace(/\$/g, "") : s;
      const p1 = this.coordToColRow(s);
      const p2 = this.coordToColRow(e);
      this.formulaReferences.push({
        range: {
          minRow: Math.min(p1.row, p2.row),
          maxRow: Math.max(p1.row, p2.row),
          minCol: Math.min(p1.col, p2.col),
          maxCol: Math.max(p1.col, p2.col),
        },
        color: palette[idx % palette.length],
      });
      idx++;
    }
    return this.formulaReferences;
  }

  public traceDependents(coord?: string): { range: GridRange; color: string }[] {
    const target = (coord || this.selectedCell).toUpperCase().replace(/\$/g, "");
    this.formulaReferences = [];

    this.cells.forEach((cell, cellCoord) => {
      if (cell.raw && cell.raw.startsWith("=")) {
        const text = cell.raw.toUpperCase().replace(/\$/g, "");
        const regex = new RegExp(`\\b${target}\\b`, "i");
        if (regex.test(text)) {
          const cr = this.coordToColRow(cellCoord);
          this.formulaReferences.push({
            range: { minRow: cr.row, maxRow: cr.row, minCol: cr.col, maxCol: cr.col },
            color: "#dc2626",
          });
        }
      }
    });
    return this.formulaReferences;
  }

  public removeArrows(): void {
    this.formulaReferences = [];
  }

  public importFromCsv(csvText: string, startCoord = "A1"): { rows: number; cols: number } {
    const start = this.coordToColRow(startCoord);
    const lines = csvText.split(/\r?\n/).filter((l) => l.trim().length > 0);
    let maxCols = 0;

    lines.forEach((line, rIdx) => {
      const tokens: string[] = [];
      let cur = "";
      let inQuote = false;
      for (let i = 0; i < line.length; i++) {
        const ch = line[i];
        if (ch === '"') {
          inQuote = !inQuote;
        } else if (ch === ',' && !inQuote) {
          tokens.push(cur.trim());
          cur = "";
        } else {
          cur += ch;
        }
      }
      tokens.push(cur.trim());

      if (tokens.length > maxCols) maxCols = tokens.length;

      tokens.forEach((tok, cIdx) => {
        const coord = `${this.colIndexToLetter(start.col + cIdx)}${start.row + rIdx + 1}`;
        this.setCellValue(coord, tok);
      });
    });

    this.recalculateAll();
    return { rows: lines.length, cols: maxCols };
  }

  public importFromJson(jsonText: string, startCoord = "A1"): { rows: number; cols: number } {
    const start = this.coordToColRow(startCoord);
    let data: unknown;
    try {
      data = JSON.parse(jsonText);
    } catch {
      return { rows: 0, cols: 0 };
    }

    if (Array.isArray(data)) {
      if (data.length === 0) return { rows: 0, cols: 0 };
      if (typeof data[0] === "object" && data[0] !== null && !Array.isArray(data[0])) {
        const headers = Object.keys(data[0]);
        headers.forEach((h, cIdx) => {
          const coord = `${this.colIndexToLetter(start.col + cIdx)}${start.row + 1}`;
          this.setCellValue(coord, h);
          const cell = this.getCell(coord);
          cell.bold = true;
          cell.bgColor = "#107c41";
          cell.color = "#ffffff";
        });

        data.forEach((item: Record<string, unknown>, rIdx) => {
          headers.forEach((h, cIdx) => {
            const coord = `${this.colIndexToLetter(start.col + cIdx)}${start.row + rIdx + 2}`;
            const val = item[h] !== undefined ? String(item[h]) : "";
            this.setCellValue(coord, val);
          });
        });
        this.recalculateAll();
        return { rows: data.length + 1, cols: headers.length };
      } else if (Array.isArray(data[0])) {
        let maxCols = 0;
        data.forEach((rowArr: unknown[], rIdx) => {
          if (rowArr.length > maxCols) maxCols = rowArr.length;
          rowArr.forEach((val, cIdx) => {
            const coord = `${this.colIndexToLetter(start.col + cIdx)}${start.row + rIdx + 1}`;
            this.setCellValue(coord, String(val ?? ""));
          });
        });
        this.recalculateAll();
        return { rows: data.length, cols: maxCols };
      }
    }
    return { rows: 0, cols: 0 };
  }

  public customSort(sortSpecs: { col: number; ascending: boolean }[], range?: GridRange): void {
    const target = this.getTargetRange(range);
    const rowRecords: { rowIdx: number; cells: Map<number, CellData> }[] = [];

    for (let r = target.minRow; r <= target.maxRow; r++) {
      const rowMap = new Map<number, CellData>();
      for (let c = target.minCol; c <= target.maxCol; c++) {
        const coord = `${this.colIndexToLetter(c)}${r + 1}`;
        const cell = this.cells.get(coord);
        if (cell) rowMap.set(c, { ...cell });
      }
      rowRecords.push({ rowIdx: r, cells: rowMap });
    }

    rowRecords.sort((a, b) => {
      for (const spec of sortSpecs) {
        const cellA = a.cells.get(spec.col);
        const cellB = b.cells.get(spec.col);
        const valA = cellA ? (cellA.computed !== undefined ? cellA.computed : cellA.raw) : "";
        const valB = cellB ? (cellB.computed !== undefined ? cellB.computed : cellB.raw) : "";

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
        if (cmp !== 0) {
          return spec.ascending ? cmp : -cmp;
        }
      }
      return 0;
    });

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

  public setDataValidation(rule: DataValidationRule, range?: GridRange): void {
    this.forEachCellInRange(range, (cell) => {
      cell.validation = { ...rule };
    });
  }

  public validateCellValue(coord: string, val: unknown): { valid: boolean; error?: string } {
    const cell = this.cells.get(coord);
    if (!cell || !cell.validation) return { valid: true };

    const { type, allow, min, max } = cell.validation;
    const strVal = String(val).trim();

    if (type === "list") {
      if (allow && allow.length > 0) {
        const match = allow.some((item) => item.trim().toLowerCase() === strVal.toLowerCase());
        if (!match) {
          return { valid: false, error: `Value must be one of: ${allow.join(", ")}` };
        }
      }
    } else if (type === "whole") {
      const num = Number(strVal);
      if (isNaN(num) || !Number.isInteger(num)) {
        return { valid: false, error: "Value must be a whole number" };
      }
      if (min !== undefined && num < min) {
        return { valid: false, error: `Value must be at least ${min}` };
      }
      if (max !== undefined && num > max) {
        return { valid: false, error: `Value must be at most ${max}` };
      }
    } else if (type === "date") {
      const d = new Date(strVal);
      if (isNaN(d.getTime())) {
        return { valid: false, error: "Value must be a valid date (YYYY-MM-DD)" };
      }
    }
    return { valid: true };
  }

  public groupRows(range?: GridRange): void {
    const target = this.getTargetRange(range);
    const sheet = this.getActiveSheet();
    sheet.rowGroups = sheet.rowGroups || [];
    sheet.rowGroups.push({ startRow: target.minRow, endRow: target.maxRow });
  }

  public ungroupRows(range?: GridRange): void {
    const target = this.getTargetRange(range);
    const sheet = this.getActiveSheet();
    if (!sheet.rowGroups) return;
    sheet.rowGroups = sheet.rowGroups.filter(
      (g) => !(g.startRow >= target.minRow && g.endRow <= target.maxRow)
    );
  }

  public insertSubtotals(groupByCol = 0, sumCol = 1, range?: GridRange): void {
    const target = this.getTargetRange(range);
    const sheet = this.getActiveSheet();

    const groups: { key: string; rows: number[] }[] = [];
    let currentKey = "";
    let currentRows: number[] = [];

    for (let r = target.minRow; r <= target.maxRow; r++) {
      const keyCoord = `${this.colIndexToLetter(groupByCol)}${r + 1}`;
      const cell = sheet.cells.get(keyCoord);
      const key = String(cell ? (cell.computed !== undefined ? cell.computed : cell.raw) : "").trim();
      if (key !== currentKey) {
        if (currentRows.length > 0) {
          groups.push({ key: currentKey, rows: currentRows });
        }
        currentKey = key;
        currentRows = [r];
      } else {
        currentRows.push(r);
      }
    }
    if (currentRows.length > 0) {
      groups.push({ key: currentKey, rows: currentRows });
    }

    let grandTotal = 0;
    for (let g = groups.length - 1; g >= 0; g--) {
      const grp = groups[g];
      let groupSum = 0;
      for (const r of grp.rows) {
        const sumCoord = `${this.colIndexToLetter(sumCol)}${r + 1}`;
        const c = sheet.cells.get(sumCoord);
        const val = Number(c ? (c.computed !== undefined ? c.computed : c.raw) : 0);
        if (!isNaN(val)) groupSum += val;
      }
      grandTotal += groupSum;

      const insertAt = grp.rows[grp.rows.length - 1] + 1;
      this.insertRow(insertAt);
      const labelCoord = `${this.colIndexToLetter(groupByCol)}${insertAt + 1}`;
      const totalCoord = `${this.colIndexToLetter(sumCol)}${insertAt + 1}`;
      sheet.cells.set(labelCoord, {
        raw: `${grp.key} Total`,
        computed: `${grp.key} Total`,
        bold: true,
        bgColor: "#f1f5f9",
        borders: "top,bottom",
      });
      sheet.cells.set(totalCoord, {
        raw: String(groupSum),
        computed: groupSum,
        bold: true,
        bgColor: "#f1f5f9",
        format: "Currency",
        borders: "top,bottom",
      });
    }

    const grandRow = target.maxRow + groups.length + 1;
    const gLabelCoord = `${this.colIndexToLetter(groupByCol)}${grandRow + 1}`;
    const gTotalCoord = `${this.colIndexToLetter(sumCol)}${grandRow + 1}`;
    sheet.cells.set(gLabelCoord, {
      raw: "Grand Total",
      computed: "Grand Total",
      bold: true,
      bgColor: "#e2e8f0",
      borders: "bottom",
    });
    sheet.cells.set(gTotalCoord, {
      raw: String(grandTotal),
      computed: grandTotal,
      bold: true,
      bgColor: "#e2e8f0",
      format: "Currency",
      borders: "bottom",
    });

    this.recalculateAll();
  }

  public goalSeek(
    targetCoord: string,
    targetVal: number,
    changingCoord: string,
    maxIterations = 100,
    tolerance = 0.001
  ): { success: boolean; iterations: number; finalValue: number } {
    let x0 = Number(this.getCellValue(changingCoord)) || 0;
    if (x0 === 0) x0 = 1;
    this.setCellValue(changingCoord, String(x0));
    let y0 = Number(this.getCellValue(targetCoord)) - targetVal;
    if (Math.abs(y0) <= tolerance) {
      return { success: true, iterations: 0, finalValue: x0 };
    }

    let x1 = x0 + 1;
    this.setCellValue(changingCoord, String(x1));
    let y1 = Number(this.getCellValue(targetCoord)) - targetVal;

    for (let i = 0; i < maxIterations; i++) {
      if (Math.abs(y1) <= tolerance) {
        return { success: true, iterations: i + 1, finalValue: Math.round(x1 * 10000) / 10000 };
      }
      const denom = y1 - y0;
      if (Math.abs(denom) < 1e-12) break;
      const xNext = x1 - (y1 * (x1 - x0)) / denom;
      x0 = x1;
      y0 = y1;
      x1 = xNext;
      this.setCellValue(changingCoord, String(x1));
      y1 = Number(this.getCellValue(targetCoord)) - targetVal;
    }

    const success = Math.abs(y1) <= tolerance * 10;
    return { success, iterations: maxIterations, finalValue: Math.round(x1 * 10000) / 10000 };
  }

  public isSheetProtected(): boolean {
    return Boolean(this.getActiveSheet().isProtected);
  }

  public protectSheet(password?: string): void {
    const sheet = this.getActiveSheet();
    sheet.isProtected = true;
    sheet.password = password;
  }

  public unprotectSheet(password?: string): boolean {
    const sheet = this.getActiveSheet();
    if (sheet.password && sheet.password !== password) return false;
    sheet.isProtected = false;
    sheet.password = undefined;
    return true;
  }

  public isWorkbookProtected(): boolean {
    return this.workbookProtected;
  }

  public protectWorkbook(password?: string): void {
    this.workbookProtected = true;
    this.workbookPassword = password;
  }

  public unprotectWorkbook(password?: string): boolean {
    if (this.workbookPassword && this.workbookPassword !== password) return false;
    this.workbookProtected = false;
    this.workbookPassword = undefined;
    return true;
  }

  public getWorkbookStatistics(): {
    sheetCount: number;
    cellCount: number;
    formulasCount: number;
    chartsCount: number;
    tablesCount: number;
  } {
    let cellCount = 0;
    let formulasCount = 0;
    let chartsCount = 0;
    let tablesCount = 0;

    this.sheets.forEach((s) => {
      cellCount += s.cells.size;
      s.cells.forEach((c) => {
        if (c.raw && c.raw.startsWith("=")) formulasCount++;
      });
      if (s.charts) chartsCount += s.charts.length;
      if (s.tableRange) tablesCount++;
    });

    return {
      sheetCount: this.sheets.length,
      cellCount,
      formulasCount,
      chartsCount,
      tablesCount,
    };
  }

  public setViewMode(mode: "normal" | "pagebreak" | "pagelayout"): void {
    this.viewMode = mode;
    this.getActiveSheet().viewMode = mode;
  }

  public zoomToSelection(range?: GridRange): void {
    const target = this.getTargetRange(range);
    const rows = target.maxRow - target.minRow + 1;
    const cols = target.maxCol - target.minCol + 1;
    if (rows > 10 || cols > 10) {
      this.setZoom(0.75);
    } else if (rows <= 3 && cols <= 3) {
      this.setZoom(1.25);
    } else {
      this.setZoom(1.0);
    }
  }

  public toggleSplitWindow(): boolean {
    this.isSplitWindow = !this.isSplitWindow;
    return this.isSplitWindow;
  }

  public toggleShowAllNotes(): boolean {
    this.showAllNotes = !this.showAllNotes;
    return this.showAllNotes;
  }
}

export const calcState = new CalcState();
