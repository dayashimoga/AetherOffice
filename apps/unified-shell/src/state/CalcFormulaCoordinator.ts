/**
 * CalcFormulaCoordinator.ts
 *
 * Implements Sub-Phase 47.1.1: Modern Dynamic Array & Lambda Functions presentation coordinator.
 * Strictly conforms to GEC-v7.0:
 * - Instance-scoped, zero global singletons.
 * - Reactive state management for dynamic array spill zones, #SPILL! collision detection,
 *   formula autocomplete, parameter hints, and client-side fallback array evaluation.
 * - Full parity with commercial Excel / ECMA-376 dynamic array standards.
 */

export interface ParameterMetadata {
  name: string;
  optional?: boolean;
  description: string;
}

export interface FunctionMetadata {
  name: string;
  syntax: string;
  description: string;
  category: "Dynamic Array" | "Lambda & Closures" | "Lookup & Reference" | "Math & Statistical" | "Financial" | "Logical";
  minArgs: number;
  maxArgs: number;
  parameters: ParameterMetadata[];
}

export interface SpillZone {
  rootCell: string;
  endCell: string;
  rows: number;
  cols: number;
  spilledCells: string[];
  hasCollision: boolean;
  collisionCells: string[];
}

export const MODERN_FUNCTIONS_CATALOG: FunctionMetadata[] = [
  {
    name: "SEQUENCE",
    syntax: "SEQUENCE(rows, [columns], [start], [step])",
    description: "Generates an array of sequential numbers in a dynamic matrix.",
    category: "Dynamic Array",
    minArgs: 1,
    maxArgs: 4,
    parameters: [
      { name: "rows", description: "The number of rows to return." },
      { name: "columns", optional: true, description: "The number of columns to return (default 1)." },
      { name: "start", optional: true, description: "The starting number in the sequence (default 1)." },
      { name: "step", optional: true, description: "The amount to increment each value (default 1)." },
    ],
  },
  {
    name: "RANDARRAY",
    syntax: "RANDARRAY([rows], [columns], [min], [max], [whole_number])",
    description: "Generates an array of random numbers between min and max.",
    category: "Dynamic Array",
    minArgs: 0,
    maxArgs: 5,
    parameters: [
      { name: "rows", optional: true, description: "The number of rows to return (default 1)." },
      { name: "columns", optional: true, description: "The number of columns to return (default 1)." },
      { name: "min", optional: true, description: "The minimum value (default 0)." },
      { name: "max", optional: true, description: "The maximum value (default 1)." },
      { name: "whole_number", optional: true, description: "TRUE for integers, FALSE for decimals (default FALSE)." },
    ],
  },
  {
    name: "CHOOSEROWS",
    syntax: "CHOOSEROWS(array, row_num1, [row_num2], ...)",
    description: "Extracts specific rows from an array, supporting negative indices from the end.",
    category: "Dynamic Array",
    minArgs: 2,
    maxArgs: 255,
    parameters: [
      { name: "array", description: "The input array or range." },
      { name: "row_num1", description: "The first 1-based or negative row index to extract." },
      { name: "row_num2", optional: true, description: "Additional row indices to extract." },
    ],
  },
  {
    name: "CHOOSECOLS",
    syntax: "CHOOSECOLS(array, col_num1, [col_num2], ...)",
    description: "Extracts specific columns from an array, supporting negative indices from the end.",
    category: "Dynamic Array",
    minArgs: 2,
    maxArgs: 255,
    parameters: [
      { name: "array", description: "The input array or range." },
      { name: "col_num1", description: "The first 1-based or negative column index to extract." },
      { name: "col_num2", optional: true, description: "Additional column indices to extract." },
    ],
  },
  {
    name: "TOCOL",
    syntax: "TOCOL(array, [ignore], [scan_by_column])",
    description: "Transforms a 2D array or range into a single vertical column.",
    category: "Dynamic Array",
    minArgs: 1,
    maxArgs: 3,
    parameters: [
      { name: "array", description: "The array or range to transform." },
      { name: "ignore", optional: true, description: "0: keep all, 1: ignore blanks, 2: ignore errors, 3: ignore blanks and errors." },
      { name: "scan_by_column", optional: true, description: "FALSE: scan row by row (default), TRUE: scan column by column." },
    ],
  },
  {
    name: "TOROW",
    syntax: "TOROW(array, [ignore], [scan_by_column])",
    description: "Transforms a 2D array or range into a single horizontal row.",
    category: "Dynamic Array",
    minArgs: 1,
    maxArgs: 3,
    parameters: [
      { name: "array", description: "The array or range to transform." },
      { name: "ignore", optional: true, description: "0: keep all, 1: ignore blanks, 2: ignore errors, 3: ignore blanks and errors." },
      { name: "scan_by_column", optional: true, description: "FALSE: scan row by row (default), TRUE: scan column by column." },
    ],
  },
  {
    name: "TAKE",
    syntax: "TAKE(array, rows, [columns])",
    description: "Returns a contiguous subset of rows and/or columns from the start or end.",
    category: "Dynamic Array",
    minArgs: 2,
    maxArgs: 3,
    parameters: [
      { name: "array", description: "The array or range from which to take." },
      { name: "rows", description: "The number of rows to take. Positive takes from top, negative from bottom." },
      { name: "columns", optional: true, description: "The number of columns to take. Positive takes from left, negative from right." },
    ],
  },
  {
    name: "DROP",
    syntax: "DROP(array, rows, [columns])",
    description: "Excludes a specified number of rows and/or columns from the start or end of an array.",
    category: "Dynamic Array",
    minArgs: 2,
    maxArgs: 3,
    parameters: [
      { name: "array", description: "The array or range from which to drop." },
      { name: "rows", description: "The number of rows to drop. Positive drops from top, negative from bottom." },
      { name: "columns", optional: true, description: "The number of columns to drop. Positive drops from left, negative from right." },
    ],
  },
  {
    name: "EXPAND",
    syntax: "EXPAND(array, rows, [columns], [pad_with])",
    description: "Expands or pads an array to specified row and column dimensions.",
    category: "Dynamic Array",
    minArgs: 2,
    maxArgs: 4,
    parameters: [
      { name: "array", description: "The array or range to expand." },
      { name: "rows", description: "Target number of rows (must be >= current rows)." },
      { name: "columns", optional: true, description: "Target number of columns (must be >= current columns)." },
      { name: "pad_with", optional: true, description: "Value to pad empty cells with (default #N/A)." },
    ],
  },
  {
    name: "SORTBY",
    syntax: "SORTBY(array, by_array1, [order1], ...)",
    description: "Sorts the contents of a range or array based on the values in one or more corresponding arrays.",
    category: "Dynamic Array",
    minArgs: 2,
    maxArgs: 255,
    parameters: [
      { name: "array", description: "The range or array to sort." },
      { name: "by_array1", description: "The array or range to sort by." },
      { name: "order1", optional: true, description: "1 for ascending (default), -1 for descending." },
    ],
  },
  {
    name: "FILTER",
    syntax: "FILTER(array, include, [if_empty])",
    description: "Filters a range or array based on a boolean criteria array.",
    category: "Dynamic Array",
    minArgs: 2,
    maxArgs: 3,
    parameters: [
      { name: "array", description: "The range or array to filter." },
      { name: "include", description: "A boolean array of matching dimension indicating items to include." },
      { name: "if_empty", optional: true, description: "Value to return if no entries meet the criteria (default #CALC!)." },
    ],
  },
  {
    name: "UNIQUE",
    syntax: "UNIQUE(array, [by_col], [exactly_once])",
    description: "Returns unique values from a range or array across rows or columns.",
    category: "Dynamic Array",
    minArgs: 1,
    maxArgs: 3,
    parameters: [
      { name: "array", description: "The range or array to deduplicate." },
      { name: "by_col", optional: true, description: "FALSE to compare rows (default), TRUE to compare columns." },
      { name: "exactly_once", optional: true, description: "TRUE to return only items appearing exactly once, FALSE for all distinct items." },
    ],
  },
  {
    name: "SORT",
    syntax: "SORT(array, [sort_index], [sort_order], [by_col])",
    description: "Sorts the contents of a range or array.",
    category: "Dynamic Array",
    minArgs: 1,
    maxArgs: 4,
    parameters: [
      { name: "array", description: "The range or array to sort." },
      { name: "sort_index", optional: true, description: "1-based column or row index to sort by (default 1)." },
      { name: "sort_order", optional: true, description: "1 for ascending (default), -1 for descending." },
      { name: "by_col", optional: true, description: "FALSE to sort rows (default), TRUE to sort columns." },
    ],
  },
  {
    name: "XLOOKUP",
    syntax: "XLOOKUP(lookup_value, lookup_array, return_array, [if_not_found], [match_mode], [search_mode])",
    description: "Searches a range or array and returns an item or row/column corresponding to the first match.",
    category: "Lookup & Reference",
    minArgs: 3,
    maxArgs: 6,
    parameters: [
      { name: "lookup_value", description: "The value to search for." },
      { name: "lookup_array", description: "The array or range to search." },
      { name: "return_array", description: "The array or range to return values from." },
      { name: "if_not_found", optional: true, description: "Returned if no match found (default #N/A)." },
      { name: "match_mode", optional: true, description: "0: exact, -1: exact or next smaller, 1: exact or next larger, 2: wildcard." },
      { name: "search_mode", optional: true, description: "1: first to last, -1: last to first, 2: binary asc, -2: binary desc." },
    ],
  },
  {
    name: "XMATCH",
    syntax: "XMATCH(lookup_value, lookup_array, [match_mode], [search_mode])",
    description: "Returns the relative 1-based position of an item in an array or range.",
    category: "Lookup & Reference",
    minArgs: 2,
    maxArgs: 4,
    parameters: [
      { name: "lookup_value", description: "The value to search for." },
      { name: "lookup_array", description: "The array or range to search." },
      { name: "match_mode", optional: true, description: "0: exact, -1: next smaller, 1: next larger, 2: wildcard." },
      { name: "search_mode", optional: true, description: "1: first to last, -1: last to first, 2: binary asc, -2: binary desc." },
    ],
  },
  {
    name: "LET",
    syntax: "LET(name1, val1, [name2, val2], ..., calculation)",
    description: "Assigns names to calculation results, storing intermediate variables or closures.",
    category: "Lambda & Closures",
    minArgs: 3,
    maxArgs: 255,
    parameters: [
      { name: "name1", description: "The first variable name." },
      { name: "val1", description: "The value or lambda bound to name1." },
      { name: "calculation", description: "The final expression evaluated using the defined variables." },
    ],
  },
  {
    name: "LAMBDA",
    syntax: "LAMBDA([parameter1, parameter2, ...], calculation)",
    description: "Creates custom, reusable functions that can be called with arguments.",
    category: "Lambda & Closures",
    minArgs: 1,
    maxArgs: 255,
    parameters: [
      { name: "parameter1", optional: true, description: "Input parameter name passed into the function." },
      { name: "calculation", description: "The calculation executed and returned by the lambda." },
    ],
  },
  {
    name: "MAKEARRAY",
    syntax: "MAKEARRAY(rows, cols, lambda)",
    description: "Returns a calculated array of a specified row and column size by applying a LAMBDA.",
    category: "Lambda & Closures",
    minArgs: 3,
    maxArgs: 3,
    parameters: [
      { name: "rows", description: "The number of rows in the array." },
      { name: "cols", description: "The number of columns in the array." },
      { name: "lambda", description: "A LAMBDA(row_idx, col_idx) that calculates each cell's value." },
    ],
  },
  {
    name: "BYROW",
    syntax: "BYROW(array, lambda)",
    description: "Applies a LAMBDA to each row of an array and returns an array of the results.",
    category: "Lambda & Closures",
    minArgs: 2,
    maxArgs: 2,
    parameters: [
      { name: "array", description: "The array to separate into rows." },
      { name: "lambda", description: "A LAMBDA(row) applied to each row." },
    ],
  },
  {
    name: "BYCOL",
    syntax: "BYCOL(array, lambda)",
    description: "Applies a LAMBDA to each column of an array and returns an array of the results.",
    category: "Lambda & Closures",
    minArgs: 2,
    maxArgs: 2,
    parameters: [
      { name: "array", description: "The array to separate into columns." },
      { name: "lambda", description: "A LAMBDA(col) applied to each column." },
    ],
  },
  {
    name: "MAP",
    syntax: "MAP(array1, [array2], ..., lambda)",
    description: "Applies a LAMBDA to create an array formed by mapping each value in the array(s) to a new value.",
    category: "Lambda & Closures",
    minArgs: 2,
    maxArgs: 255,
    parameters: [
      { name: "array1", description: "The array to map." },
      { name: "lambda", description: "A LAMBDA that takes an element from each array and returns a result." },
    ],
  },
  {
    name: "REDUCE",
    syntax: "REDUCE(initial_value, array, lambda)",
    description: "Reduces an array to an accumulated value by applying a LAMBDA to each element.",
    category: "Lambda & Closures",
    minArgs: 3,
    maxArgs: 3,
    parameters: [
      { name: "initial_value", description: "The initial accumulator value." },
      { name: "array", description: "The array to reduce." },
      { name: "lambda", description: "A LAMBDA(accumulator, current_value) returning the next accumulated value." },
    ],
  },
  {
    name: "SCAN",
    syntax: "SCAN(initial_value, array, lambda)",
    description: "Scans an array by applying a LAMBDA to each element and returns an array of intermediate values.",
    category: "Lambda & Closures",
    minArgs: 3,
    maxArgs: 3,
    parameters: [
      { name: "initial_value", description: "The initial accumulator value." },
      { name: "array", description: "The array to scan." },
      { name: "lambda", description: "A LAMBDA(accumulator, current_value) returning each intermediate value." },
    ],
  },
  {
    name: "ISOMITTED",
    syntax: "ISOMITTED(parameter)",
    description: "Checks whether the value in a LAMBDA parameter is missing or omitted.",
    category: "Lambda & Closures",
    minArgs: 1,
    maxArgs: 1,
    parameters: [
      { name: "parameter", description: "The LAMBDA parameter to test." },
    ],
  },
];

export class CalcFormulaCoordinator {
  public activeSpillZones: Map<string, SpillZone> = new Map();
  private listeners: Set<() => void> = new Set();

  public subscribe(listener: () => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private notify(): void {
    for (const listener of this.listeners) {
      listener();
    }
  }

  /**
   * Returns autocomplete function suggestions matching the typed prefix.
   */
  public getSuggestions(inputPrefix: string): FunctionMetadata[] {
    const clean = inputPrefix.trim().replace(/^=/, "").toUpperCase();
    if (!clean) return [];
    return MODERN_FUNCTIONS_CATALOG.filter((f) =>
      f.name.toUpperCase().startsWith(clean)
    );
  }

  /**
   * Parses active function and parameter index for contextual signature tooltips.
   */
  public getActiveParameterHint(
    formula: string,
    cursorOffset: number
  ): { func: FunctionMetadata; activeParamIndex: number } | null {
    if (!formula.startsWith("=")) return null;
    const textBeforeCursor = formula.slice(1, cursorOffset);
    const lastOpenParen = textBeforeCursor.lastIndexOf("(");
    if (lastOpenParen === -1) return null;

    // Extract function name preceding '('
    const preText = textBeforeCursor.slice(0, lastOpenParen).trim();
    const match = preText.match(/([A-Z0-9_\.]+)$/i);
    if (!match) return null;

    const funcName = match[1].replace(/^_xlfn\./i, "").toUpperCase();
    const func = MODERN_FUNCTIONS_CATALOG.find((f) => f.name === funcName);
    if (!func) return null;

    // Count commas inside current paren level
    const insideArgs = textBeforeCursor.slice(lastOpenParen + 1);
    let commaCount = 0;
    let depth = 0;
    for (const ch of insideArgs) {
      if (ch === "(" || ch === "{" || ch === "[") depth++;
      else if (ch === ")" || ch === "}" || ch === "]") depth--;
      else if (ch === "," && depth === 0) commaCount++;
    }

    return { func, activeParamIndex: commaCount };
  }

  /**
   * Helper parsing cell coordinate into 0-based { row, col }.
   */
  public parseCoord(coord: string): { row: number; col: number } {
    const match = coord.trim().toUpperCase().match(/^([A-Z]+)([0-9]+)$/);
    if (!match) return { row: 0, col: 0 };
    const colStr = match[1];
    let col = 0;
    for (let i = 0; i < colStr.length; i++) {
      col = col * 26 + (colStr.charCodeAt(i) - 64);
    }
    const row = parseInt(match[2], 10) - 1;
    return { row, col: col - 1 };
  }

  /**
   * Helper formatting 0-based { row, col } into cell coordinate.
   */
  public formatCoord(row: number, col: number): string {
    let colStr = "";
    let c = col + 1;
    while (c > 0) {
      const rem = (c - 1) % 26;
      colStr = String.fromCharCode(65 + rem) + colStr;
      c = Math.floor((c - 1) / 26);
    }
    return `${colStr}${row + 1}`;
  }

  /**
   * Computes dynamic array spill range and validates against occupied cells.
   */
  public calculateSpillRange(
    rootCoord: string,
    rows: number,
    cols: number,
    occupiedCells?: Set<string>
  ): SpillZone {
    const { row: r0, col: c0 } = this.parseCoord(rootCoord);
    const spilledCells: string[] = [];
    const collisionCells: string[] = [];

    for (let r = 0; r < rows; r++) {
      for (let c = 0; c < cols; c++) {
        if (r === 0 && c === 0) continue;
        const cell = this.formatCoord(r0 + r, c0 + c);
        spilledCells.push(cell);
        if (occupiedCells && occupiedCells.has(cell)) {
          collisionCells.push(cell);
        }
      }
    }

    const endCell = this.formatCoord(r0 + rows - 1, c0 + cols - 1);
    const hasCollision = collisionCells.length > 0;

    const zone: SpillZone = {
      rootCell: rootCoord.toUpperCase(),
      endCell,
      rows,
      cols,
      spilledCells,
      hasCollision,
      collisionCells,
    };

    this.activeSpillZones.set(rootCoord.toUpperCase(), zone);
    this.notify();
    return zone;
  }

  /**
   * Clears spill zone associated with root cell.
   */
  public clearSpillZone(rootCoord: string): void {
    if (this.activeSpillZones.delete(rootCoord.toUpperCase())) {
      this.notify();
    }
  }

  /**
   * Evaluates dynamic array formulas client-side in fallback or browser mode.
   */
  public evalSequence(rows: number, cols = 1, start = 1, step = 1): number[][] {
    if (rows < 1 || cols < 1) throw new Error("#VALUE!");
    const matrix: number[][] = [];
    let cur = start;
    for (let r = 0; r < rows; r++) {
      const row: number[] = [];
      for (let c = 0; c < cols; c++) {
        row.push(cur);
        cur += step;
      }
      matrix.push(row);
    }
    return matrix;
  }

  public evalToCol(matrix: any[][], ignore = 0, scanByCol = false): any[][] {
    if (!matrix || matrix.length === 0) return [];
    const flat: any[] = [];
    const rows = matrix.length;
    const cols = matrix[0].length;

    const shouldKeep = (val: any) => {
      if (ignore === 1 && (val === null || val === undefined || val === "")) return false;
      if (ignore === 2 && typeof val === "string" && val.startsWith("#")) return false;
      if (ignore === 3 && ((val === null || val === undefined || val === "") || (typeof val === "string" && val.startsWith("#")))) return false;
      return true;
    };

    if (scanByCol) {
      for (let c = 0; c < cols; c++) {
        for (let r = 0; r < rows; r++) {
          if (shouldKeep(matrix[r][c])) flat.push(matrix[r][c]);
        }
      }
    } else {
      for (let r = 0; r < rows; r++) {
        for (let c = 0; c < cols; c++) {
          if (shouldKeep(matrix[r][c])) flat.push(matrix[r][c]);
        }
      }
    }

    return flat.map((item) => [item]);
  }
}

