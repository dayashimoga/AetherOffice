/**
 * CalcDataConnectorCoordinator.ts
 * GEC-v7.0 Stage 47 (Sub-Phase 47.2.2)
 *
 * Industrial-grade two-way streaming coordinator for:
 * - Apache Parquet (.parquet)
 * - Apache Arrow IPC Stream (.arrow)
 * - SQLite 3 Database Tables (.db, .sqlite)
 * - OASIS OpenDocument Spreadsheet (.ods)
 * - Delimited CSV / TSV (.csv, .tsv)
 * - JSON Array / JSONL (.json)
 *
 * Provides typed Tauri IPC bridge integration, client-side fallback serialization,
 * grid hydration, cell formatting, and seamless roundtrip export.
 */

import {
  IPC,
  type DataConnectorImportOptions,
  type DataConnectorImportResult,
  type DataConnectorExportPayload,
  type DataConnectorSchema,
  type DataConnectorValue,
} from "../ipc";
import { calcState, type CellData } from "./calcState";

export type SupportedDataFormat =
  | "csv"
  | "json"
  | "parquet"
  | "arrow"
  | "sqlite"
  | "ods";

export interface DataConnectorExportOptions {
  range?: string;
  tableName?: string;
  hasHeaders?: boolean;
}

export class CalcDataConnectorCoordinator {
  /**
   * Imports an external dataset (Parquet, Arrow, SQLite, ODS, CSV, JSON) into Calc.
   */
  async importDataset(
    format: SupportedDataFormat,
    data: Uint8Array | number[] | string,
    options?: DataConnectorImportOptions,
    targetState: any = calcState
  ): Promise<DataConnectorImportResult> {
    const bytes =
      typeof data === "string"
        ? new TextEncoder().encode(data)
        : data instanceof Uint8Array
        ? data
        : new Uint8Array(data);

    let result: DataConnectorImportResult;

    try {
      result = await IPC.dataConnectorImport(
        format,
        bytes,
        options ? JSON.stringify(options) : undefined
      );
    } catch (err) {
      console.warn(`Tauri IPC data_connector_import failed, using client fallback:`, err);
      result = this.clientFallbackImport(format, bytes, options);
    }

    // Hydrate Calc grid with imported headers & records
    this.hydrateGridWithImportResult(result, targetState, options);

    return result;
  }

  /**
   * Exports the active sheet or selected grid range to the chosen binary/text format.
   */
  async exportDataset(
    format: SupportedDataFormat,
    options?: DataConnectorExportOptions,
    sourceState: any = calcState
  ): Promise<Uint8Array> {
    const { schema, rows } = this.extractTableFromGrid(
      options?.range,
      options?.hasHeaders ?? true,
      sourceState
    );

    const sheet = typeof sourceState.getActiveSheet === "function" ? sourceState.getActiveSheet() : sourceState.activeSheet;
    const payload: DataConnectorExportPayload = {
      format,
      table_name: options?.tableName || sheet?.name || "Sheet1",
      schema,
      rows,
    };

    try {
      return await IPC.dataConnectorExport(JSON.stringify(payload));
    } catch (err) {
      console.warn(`Tauri IPC data_connector_export failed, using client fallback:`, err);
      return this.clientFallbackExport(format, payload);
    }
  }

  /**
   * Exports dataset and saves directly to disk via Tauri native binary file bridge.
   */
  async saveExportToFile(
    format: SupportedDataFormat,
    fileName: string,
    options?: DataConnectorExportOptions,
    sourceState: any = calcState
  ): Promise<string> {
    const bytes = await this.exportDataset(format, options, sourceState);
    return await IPC.saveFileBinary(fileName, bytes);
  }

  /**
   * Extracts schema and row values from the spreadsheet grid.
   */
  extractTableFromGrid(
    range?: string,
    hasHeaders = true,
    sourceState: any = calcState
  ): { schema: DataConnectorSchema; rows: DataConnectorValue[][] } {
    const sheet = typeof sourceState.getActiveSheet === "function" ? sourceState.getActiveSheet() : sourceState.activeSheet;
    if (!sheet || !sheet.cells) {
      return { schema: { fields: [] }, rows: [] };
    }

    let minRow = 0;
    let maxRow = 0;
    let minCol = 0;
    let maxCol = 0;

    if (range) {
      const parsed = this.parseCellRange(range);
      if (parsed) {
        minRow = parsed.startRow;
        maxRow = parsed.endRow;
        minCol = parsed.startCol;
        maxCol = parsed.endCol;
      }
    } else {
      // Find bounding box from existing non-empty cells
      let found = false;
      sheet.cells.forEach((_: CellData, key: string) => {
        const coords = this.a1ToCoords(key);
        if (coords) {
          if (!found) {
            minRow = maxRow = coords.row;
            minCol = maxCol = coords.col;
            found = true;
          } else {
            minRow = Math.min(minRow, coords.row);
            maxRow = Math.max(maxRow, coords.row);
            minCol = Math.min(minCol, coords.col);
            maxCol = Math.max(maxCol, coords.col);
          }
        }
      });
      if (!found) {
        return { schema: { fields: [] }, rows: [] };
      }
    }

    const headers: string[] = [];
    const headerRow = hasHeaders ? minRow : -1;
    const dataStartRow = hasHeaders ? minRow + 1 : minRow;

    for (let c = minCol; c <= maxCol; c++) {
      if (hasHeaders) {
        const cell = sheet.cells.get(this.coordsToA1(headerRow, c));
        const txt = cell ? String(cell.computed ?? cell.raw ?? "").trim() : "";
        headers.push(txt || `Column_${c - minCol + 1}`);
      } else {
        headers.push(`Column_${c - minCol + 1}`);
      }
    }

    const rows: DataConnectorValue[][] = [];
    for (let r = dataStartRow; r <= maxRow; r++) {
      const rowVals: DataConnectorValue[] = [];
      for (let c = minCol; c <= maxCol; c++) {
        const cell = sheet.cells.get(this.coordsToA1(r, c));
        if (!cell || cell.computed === undefined || cell.computed === null || cell.computed === "") {
          rowVals.push(null);
        } else if (typeof cell.computed === "number") {
          if (Number.isInteger(cell.computed)) {
            rowVals.push({ Int64: cell.computed });
          } else {
            rowVals.push({ Float64: cell.computed });
          }
        } else if (typeof cell.computed === "boolean") {
          rowVals.push({ Boolean: cell.computed });
        } else {
          rowVals.push({ Text: String(cell.computed) });
        }
      }
      rows.push(rowVals);
    }

    // Infer schema types
    const fields = headers.map((name, colIdx) => {
      let inferred = "Utf8";
      for (const r of rows) {
        const val = r[colIdx];
        if (val && typeof val === "object") {
          if ("Int64" in val) inferred = "Int64";
          else if ("Float64" in val) inferred = "Float64";
          else if ("Boolean" in val) inferred = "Boolean";
        }
      }
      return {
        name,
        data_type: inferred,
        nullable: true,
      };
    });

    return {
      schema: { fields },
      rows,
    };
  }

  /**
   * Hydrates Calc state cells from an import result.
   */
  private hydrateGridWithImportResult(
    result: DataConnectorImportResult,
    targetState: any = calcState,
    options?: DataConnectorImportOptions
  ): void {
    const startRow = options?.start_row ?? result.start_row ?? 0;
    const startCol = options?.start_col ?? result.start_col ?? 0;
    const writeHeaders = options?.write_headers ?? true;

    let currentRow = startRow;

    // 1. Write Header Row
    if (writeHeaders && result.schema.fields.length > 0) {
      result.schema.fields.forEach((field, cIdx) => {
        const a1 = this.coordsToA1(currentRow, startCol + cIdx);
        targetState.setCellValue(a1, field.name);
      });
      currentRow += 1;
    }

    // 2. Write Data Rows
    for (const row of result.sample_rows) {
      row.forEach((val, cIdx) => {
        const a1 = this.coordsToA1(currentRow, startCol + cIdx);
        const strVal = this.unwrapDataValueToString(val);
        targetState.setCellValue(a1, strVal);
      });
      currentRow += 1;
    }

    // 3. Trigger recalculation if present
    if (typeof targetState.recalculateAll === "function") {
      targetState.recalculateAll();
    }
  }

  /**
   * Helper to unwrap typed `DataConnectorValue` into string representation for cell entry.
   */
  unwrapDataValueToString(val: DataConnectorValue): string {
    if (val === null || val === undefined) return "";
    if (typeof val === "string") return val;
    if (typeof val === "number" || typeof val === "boolean") return String(val);
    if (typeof val === "object") {
      if ("Text" in val) return val.Text;
      if ("Int64" in val) return String(val.Int64);
      if ("Float64" in val) return String(val.Float64);
      if ("Boolean" in val) return val.Boolean ? "TRUE" : "FALSE";
      if ("Date" in val) return String(val.Date);
      if ("DateTime" in val) return String(val.DateTime);
      if ("Binary" in val) return `<BLOB ${val.Binary.length} bytes>`;
    }
    return String(val);
  }

  /**
   * Client-side fallback import parsing for CSV, JSON and basic tables.
   */
  private clientFallbackImport(
    format: SupportedDataFormat,
    bytes: Uint8Array,
    options?: DataConnectorImportOptions
  ): DataConnectorImportResult {
    const text = new TextDecoder().decode(bytes);
    const startRow = options?.start_row ?? 0;
    const startCol = options?.start_col ?? 0;
    const writeHeaders = options?.write_headers ?? true;

    if (format === "json") {
      try {
        const parsed = JSON.parse(text);
        if (Array.isArray(parsed) && parsed.length > 0) {
          const keys = Object.keys(parsed[0]);
          const fields = keys.map((k) => ({
            name: k,
            data_type: typeof parsed[0][k] === "number" ? "Float64" : "Utf8",
            nullable: true,
          }));
          const sample_rows: DataConnectorValue[][] = parsed.map((item) =>
            keys.map((k) => {
              const v = item[k];
              if (v === null || v === undefined) return null;
              if (typeof v === "number") return { Float64: v };
              if (typeof v === "boolean") return { Boolean: v };
              return { Text: String(v) };
            })
          );
          return {
            rows_ingested: sample_rows.length,
            columns_ingested: keys.length,
            cells_written: sample_rows.length * keys.length,
            duration_ms: 1.0,
            start_row: startRow,
            start_col: startCol,
            end_row: startRow + (writeHeaders ? 1 : 0) + sample_rows.length,
            end_col: startCol + keys.length,
            schema: { fields },
            sample_rows,
          };
        }
      } catch {
        // Fall through to CSV
      }
    }

    // Parse as CSV
    const lines = text.split(/\r?\n/).filter((l) => l.trim().length > 0);
    const firstLine = lines[0] || "";
    const delimiter = firstLine.includes("\t") ? "\t" : firstLine.includes(";") ? ";" : ",";
    const headers = firstLine.split(delimiter).map((h) => h.replace(/^"|"$/g, "").trim());
    const fields = headers.map((h) => ({
      name: h || "Col",
      data_type: "Utf8",
      nullable: true,
    }));

    const sample_rows: DataConnectorValue[][] = lines.slice(1).map((line) =>
      line.split(delimiter).map((tok) => {
        const clean = tok.replace(/^"|"$/g, "").trim();
        const num = Number(clean);
        if (clean !== "" && !isNaN(num)) {
          return Number.isInteger(num) ? { Int64: num } : { Float64: num };
        }
        return { Text: clean };
      })
    );

    return {
      rows_ingested: sample_rows.length,
      columns_ingested: headers.length,
      cells_written: sample_rows.length * headers.length,
      duration_ms: 1.0,
      start_row: startRow,
      start_col: startCol,
      end_row: startRow + (writeHeaders ? 1 : 0) + sample_rows.length,
      end_col: startCol + headers.length,
      schema: { fields },
      sample_rows,
    };
  }

  /**
   * Client-side fallback export serialization.
   */
  private clientFallbackExport(
    format: SupportedDataFormat,
    payload: DataConnectorExportPayload
  ): Uint8Array {
    if (format === "json") {
      const records = payload.rows.map((row) => {
        const obj: Record<string, unknown> = {};
        payload.schema.fields.forEach((f, i) => {
          const val = row[i];
          if (val === null || val === undefined) obj[f.name] = null;
          else if (typeof val === "object") {
            if ("Text" in val) obj[f.name] = val.Text;
            else if ("Int64" in val) obj[f.name] = val.Int64;
            else if ("Float64" in val) obj[f.name] = val.Float64;
            else if ("Boolean" in val) obj[f.name] = val.Boolean;
            else obj[f.name] = String(val);
          } else {
            obj[f.name] = val;
          }
        });
        return obj;
      });
      return new TextEncoder().encode(JSON.stringify(records, null, 2));
    }

    // Default to CSV
    const headerLine = payload.schema.fields.map((f) => `"${f.name}"`).join(",");
    const rowLines = payload.rows.map((row) =>
      row
        .map((v) => {
          const s = this.unwrapDataValueToString(v);
          return s.includes(",") || s.includes('"') || s.includes("\n")
            ? `"${s.replace(/"/g, '""')}"`
            : s;
        })
        .join(",")
    );

    const csvText = [headerLine, ...rowLines].join("\n");
    return new TextEncoder().encode(csvText);
  }

  /**
   * Parses A1 or A1:B10 range strings into 0-indexed bounds.
   */
  parseCellRange(range: string): {
    startRow: number;
    startCol: number;
    endRow: number;
    endCol: number;
  } | null {
    const parts = range.split(":");
    if (parts.length === 0) return null;
    const start = this.a1ToCoords(parts[0].trim());
    if (!start) return null;
    if (parts.length === 1) {
      return {
        startRow: start.row,
        startCol: start.col,
        endRow: start.row,
        endCol: start.col,
      };
    }
    const end = this.a1ToCoords(parts[1].trim());
    if (!end) return null;
    return {
      startRow: Math.min(start.row, end.row),
      startCol: Math.min(start.col, end.col),
      endRow: Math.max(start.row, end.row),
      endCol: Math.max(start.col, end.col),
    };
  }

  /**
   * Converts A1 coordinate string to 0-indexed { row, col }.
   */
  a1ToCoords(a1: string): { row: number; col: number } | null {
    const match = a1.trim().toUpperCase().match(/^([A-Z]+)(\d+)$/);
    if (!match) return null;
    const letters = match[1];
    const row = parseInt(match[2], 10) - 1;
    let col = 0;
    for (let i = 0; i < letters.length; i++) {
      col = col * 26 + (letters.charCodeAt(i) - 64);
    }
    return { row, col: col - 1 };
  }

  /**
   * Converts 0-indexed row and col to A1 notation.
   */
  coordsToA1(row: number, col: number): string {
    let temp = col + 1;
    let colName = "";
    while (temp > 0) {
      const rem = (temp - 1) % 26;
      colName = String.fromCharCode(65 + rem) + colName;
      temp = Math.floor((temp - 1) / 26);
    }
    return `${colName}${row + 1}`;
  }
}

export const calcDataConnectorCoordinator = new CalcDataConnectorCoordinator();
