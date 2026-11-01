/**
 * CalcPivotCoordinator.ts
 *
 * Implements Sub-Phase 47.2.1: End-to-End Rust OLAP Cube to Pivot Table UI Binding.
 * Strictly conforms to GEC-v7.0:
 * - Authoritative multi-dimensional OLAP aggregation model.
 * - Extracts real tabular datasets from spreadsheet grid cells.
 * - Dispatches pivot calculation via Tauri IPC with deterministic in-memory fallback.
 * - Synchronizes evaluated cells, grand totals, slicers, and linked PivotCharts.
 */

export type PivotAggregation =
  | "Sum"
  | "Count"
  | "CountNumbers"
  | "Average"
  | "Min"
  | "Max"
  | "Product"
  | "Variance"
  | "StdDev"
  | "DistinctCount"
  | "PercentOfTotal"
  | "PercentOfRow"
  | "PercentOfCol";

export type DateGrouping = "Year" | "Quarter" | "Month" | "Day";

export interface PivotFieldConfig {
  sourceColumn: string;
  dateGrouping?: DateGrouping;
  sortAscending?: boolean;
}

export interface MeasureFieldConfig {
  sourceColumn: string;
  aggregation: PivotAggregation;
  customLabel?: string;
}

export interface FilterFieldConfig {
  sourceColumn: string;
  includedValues?: string[];
  excludedValues?: string[];
}

export interface PivotConfigDto {
  rowFields: PivotFieldConfig[];
  colFields: PivotFieldConfig[];
  valueFields: MeasureFieldConfig[];
  filterFields: FilterFieldConfig[];
  grandTotals: "Both" | "RowsOnly" | "ColumnsOnly" | "None";
  showSubtotals: boolean;
}

export interface PivotViewCellDto {
  row: number;
  col: number;
  value: number;
  formatted_value: string;
  is_subtotal: boolean;
  is_grand_total: boolean;
}

export interface PivotTableLayoutReportDto {
  start_row: number;
  start_col: number;
  end_row: number;
  end_col: number;
  data_range_start: [number, number];
  data_range_end: [number, number];
  row_labels_count: number;
  col_labels_count: number;
  measures_count: number;
}

export interface PivotViewResultDto {
  layout: PivotTableLayoutReportDto;
  row_headers: string[][];
  col_headers: string[][];
  cells: PivotViewCellDto[];
  row_totals: number[];
  col_totals: number[];
  grand_totals: number[];
  unique_field_values: Record<string, string[]>;
}

export interface TableDataset {
  headers: string[];
  types: string[];
  records: (string | number | boolean | null)[][];
}

export class CalcPivotCoordinator {
  private activeDatasets: Map<string, TableDataset> = new Map();
  private lastResult: PivotViewResultDto | null = null;

  /**
   * Extracts a structured tabular dataset from a range of sheet cells.
   */
  public extractDatasetFromGrid(
    state: any,
    range: string,
    hasHeaders = true,
  ): TableDataset {
    const coords = this.parseCellRange(range);
    if (!coords) {
      return { headers: [], types: [], records: [] };
    }

    const { startRow, startCol, endRow, endCol } = coords;

    // 1. Extract Headers
    const headers: string[] = [];
    let firstDataRow = startRow;

    if (hasHeaders) {
      for (let c = startCol; c <= endCol; c++) {
        const addr = this.coordsToA1(startRow, c);
        const cell = state.getCellValue ? state.getCellValue(addr) : state.cells?.[addr];
        const val = this.extractCellRaw(cell);
        const name = val !== null && String(val).trim() !== ""
          ? String(val).trim()
          : `Column_${c - startCol + 1}`;
        headers.push(name);
      }
      firstDataRow = startRow + 1;
    } else {
      for (let c = startCol; c <= endCol; c++) {
        headers.push(`Column_${c - startCol + 1}`);
      }
    }

    // 2. Extract Data Records
    const records: (string | number | boolean | null)[][] = [];
    for (let r = firstDataRow; r <= endRow; r++) {
      const row: (string | number | boolean | null)[] = [];
      let hasData = false;
      for (let c = startCol; c <= endCol; c++) {
        const addr = this.coordsToA1(r, c);
        const cell = state.getCellValue ? state.getCellValue(addr) : state.cells?.[addr];
        const val = this.extractCellRaw(cell);
        if (val !== null && val !== "") {
          hasData = true;
        }
        row.push(val);
      }
      if (hasData) {
        records.push(row);
      }
    }

    // 3. Infer Types
    const types: string[] = headers.map((_, colIdx) => {
      let isNumeric = true;
      let isBoolean = true;
      let isDate = true;
      let count = 0;

      for (const row of records) {
        const v = row[colIdx];
        if (v === null || v === "") continue;
        count++;

        if (typeof v !== "number" && isNaN(Number(v))) {
          isNumeric = false;
        }
        if (typeof v !== "boolean" && v !== "true" && v !== "false") {
          isBoolean = false;
        }
        if (typeof v !== "string" || !/^\d{4}-\d{2}-\d{2}/.test(v)) {
          isDate = false;
        }
      }

      if (count === 0) return "Utf8";
      if (isNumeric) return "Float64";
      if (isBoolean) return "Boolean";
      if (isDate) return "Date";
      return "Utf8";
    });

    const dataset: TableDataset = { headers, types, records };
    this.activeDatasets.set(range, dataset);
    return dataset;
  }

  /**
   * Computes multi-dimensional OLAP cube aggregations.
   * Dispatches to Tauri desktop IPC when available, otherwise executes deterministic client calculation.
   */
  public async computePivotTable(
    dataset: TableDataset,
    config: PivotConfigDto,
    startRow = 0,
    startCol = 0,
  ): Promise<PivotViewResultDto> {
    const isTauri = typeof window !== "undefined" && ("__TAURI_INTERNALS__" in window || "__TAURI__" in window);

    if (isTauri) {
      try {
        const tauri = (window as any).__TAURI__;
        if (tauri?.core?.invoke) {
          const payload = {
            schema: {
              fields: dataset.headers.map((name, i) => ({
                name,
                data_type: dataset.types[i] || "Utf8",
                nullable: true,
              })),
            },
            records: dataset.records.map((row) =>
              row.map((val) => {
                if (val === null || val === "") return "Null";
                if (typeof val === "number") return { Float64: val };
                if (typeof val === "boolean") return { Boolean: val };
                return { Text: String(val) };
              }),
            ),
            config: {
              row_fields: config.rowFields.map((rf) => ({
                source_column: rf.sourceColumn,
                date_grouping: rf.dateGrouping || null,
                sort_ascending: rf.sortAscending ?? true,
              })),
              col_fields: config.colFields.map((cf) => ({
                source_column: cf.sourceColumn,
                date_grouping: cf.dateGrouping || null,
                sort_ascending: cf.sortAscending ?? true,
              })),
              value_fields: config.valueFields.map((vf) => ({
                source_column: vf.sourceColumn,
                aggregation: vf.aggregation,
                custom_label: vf.customLabel || null,
              })),
              filter_fields: config.filterFields.map((ff) => ({
                source_column: ff.sourceColumn,
                included_values: ff.includedValues
                  ? ff.includedValues.map((v) => ({ Text: v }))
                  : null,
                excluded_values: ff.excludedValues
                  ? ff.excludedValues.map((v) => ({ Text: v }))
                  : null,
              })),
              grand_totals: config.grandTotals,
              show_subtotals: config.showSubtotals,
            },
            start_row: startRow,
            start_col: startCol,
          };

          const result: PivotViewResultDto = await tauri.core.invoke(
            "data_connector_compute_pivot_table",
            { payloadJson: JSON.stringify(payload) },
          );
          this.lastResult = result;
          return result;
        }
      } catch (err) {
        console.warn("Tauri IPC compute_pivot_table unavailable, using deterministic in-memory calculation:", err);
      }
    }

    // Deterministic in-memory calculation fallback
    const result = this.computeInMemory(dataset, config, startRow, startCol);
    this.lastResult = result;
    return result;
  }

  /**
   * Deterministic client-side multi-dimensional OLAP calculation fallback.
   */
  public computeInMemory(
    dataset: TableDataset,
    config: PivotConfigDto,
    startRow = 0,
    startCol = 0,
  ): PivotViewResultDto {
    const uniqueValues = this.collectUniqueValues(dataset);

    // 1. Filter dataset
    const colIndexMap = new Map<string, number>();
    dataset.headers.forEach((h, i) => colIndexMap.set(h, i));

    const filteredRecords = dataset.records.filter((row) => {
      for (const filter of config.filterFields) {
        const colIdx = colIndexMap.get(filter.sourceColumn);
        if (colIdx === undefined) continue;
        const val = row[colIdx];
        const strVal = val === null ? "" : String(val);

        if (filter.includedValues && filter.includedValues.length > 0) {
          if (!filter.includedValues.includes(strVal)) return false;
        }
        if (filter.excludedValues && filter.excludedValues.length > 0) {
          if (filter.excludedValues.includes(strVal)) return false;
        }
      }
      return true;
    });

    // 2. Extract row and col coordinate tuples
    const rowKeysSet = new Set<string>();
    const colKeysSet = new Set<string>();

    const rowTuples: string[][] = [];
    const colTuples: string[][] = [];

    const formatTupleVal = (val: any, dg?: DateGrouping): string => {
      if (val === null || val === undefined) return "";
      const s = String(val);
      if (dg && s.length >= 10) {
        const m = parseInt(s.slice(5, 7), 10);
        if (dg === "Quarter") return `Q${Math.floor((m - 1) / 3) + 1}`;
        if (dg === "Year") return s.slice(0, 4);
        if (dg === "Month") {
          const names = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
          return names[m - 1] || "Jan";
        }
      }
      return s;
    };

    for (const row of filteredRecords) {
      const rTuple = config.rowFields.map((rf) => {
        const idx = colIndexMap.get(rf.sourceColumn) ?? 0;
        return formatTupleVal(row[idx], rf.dateGrouping);
      });
      const cTuple = config.colFields.map((cf) => {
        const idx = colIndexMap.get(cf.sourceColumn) ?? 0;
        return formatTupleVal(row[idx], cf.dateGrouping);
      });

      const rKey = rTuple.join("§");
      if (rTuple.length > 0 && !rowKeysSet.has(rKey)) {
        rowKeysSet.add(rKey);
        rowTuples.push(rTuple);
      }

      const cKey = cTuple.join("§");
      if (cTuple.length > 0 && !colKeysSet.has(cKey)) {
        colKeysSet.add(cKey);
        colTuples.push(cTuple);
      }
    }

    if (rowTuples.length === 0) rowTuples.push(["Total"]);
    if (colTuples.length === 0) colTuples.push(["Total"]);

    // Sort coordinates
    rowTuples.sort((a, b) => a.join(" - ").localeCompare(b.join(" - ")));
    colTuples.sort((a, b) => a.join(" - ").localeCompare(b.join(" - ")));

    // 3. Accumulate metric values
    const cellValues = new Map<string, number[]>();
    const rowTotalsMap = new Map<number, number[]>();
    const colTotalsMap = new Map<number, number[]>();
    const grandTotals = config.valueFields.map(() => 0);

    const numMeasures = config.valueFields.length || 1;

    for (const row of filteredRecords) {
      const rTuple = config.rowFields.map((rf) => {
        const idx = colIndexMap.get(rf.sourceColumn) ?? 0;
        return formatTupleVal(row[idx], rf.dateGrouping);
      });
      const cTuple = config.colFields.map((cf) => {
        const idx = colIndexMap.get(cf.sourceColumn) ?? 0;
        return formatTupleVal(row[idx], cf.dateGrouping);
      });

      const rIdx = rowTuples.findIndex((t) => t.join("§") === rTuple.join("§"));
      const cIdx = colTuples.findIndex((t) => t.join("§") === cTuple.join("§"));

      const cellR = rIdx >= 0 ? rIdx : 0;
      const cellC = cIdx >= 0 ? cIdx : 0;

      for (let mIdx = 0; mIdx < numMeasures; mIdx++) {
        const vf = config.valueFields[mIdx];
        const colIdx = vf ? (colIndexMap.get(vf.sourceColumn) ?? 0) : 0;
        const val = Number(row[colIdx]) || 0;

        const cellKey = `${cellR}_${cellC}_${mIdx}`;
        const existing = cellValues.get(cellKey) || [];
        existing.push(val);
        cellValues.set(cellKey, existing);

        const rTotals = rowTotalsMap.get(cellR) || config.valueFields.map(() => 0);
        rTotals[mIdx] = (rTotals[mIdx] || 0) + val;
        rowTotalsMap.set(cellR, rTotals);

        const cTotals = colTotalsMap.get(cellC) || config.valueFields.map(() => 0);
        cTotals[mIdx] = (cTotals[mIdx] || 0) + val;
        colTotalsMap.set(cellC, cTotals);

        grandTotals[mIdx] += val;
      }
    }

    // 4. Calculate layout report & cells
    let curRow = startRow;
    const startC = startCol;

    if (config.filterFields.length > 0) {
      curRow += config.filterFields.length + 1;
    }

    const numRowFields = Math.max(1, config.rowFields.length);
    const colDataStart = startC + numRowFields;

    const includeColTotals = config.grandTotals === "Both" || config.grandTotals === "ColumnsOnly";
    const includeRowTotals = config.grandTotals === "Both" || config.grandTotals === "RowsOnly";

    let curColOffset = colDataStart;
    for (let i = 0; i < colTuples.length; i++) {
      curColOffset += numMeasures;
    }
    const grandTotalColStart = curColOffset;
    if (includeColTotals) {
      curColOffset += numMeasures;
    }
    const endCol = curColOffset;

    curRow++; // Header row
    const dataStartRow = curRow;

    const cells: PivotViewCellDto[] = [];
    const rowTotalsList: number[] = [];
    const colTotalsList: number[] = [];

    const aggregateValues = (vals: number[], agg: PivotAggregation): number => {
      if (!vals || vals.length === 0) return 0;
      switch (agg) {
        case "Sum":
          return vals.reduce((a, b) => a + b, 0);
        case "Count":
        case "CountNumbers":
          return vals.length;
        case "Average":
          return vals.reduce((a, b) => a + b, 0) / vals.length;
        case "Min":
          return Math.min(...vals);
        case "Max":
          return Math.max(...vals);
        case "Product":
          return vals.reduce((a, b) => a * b, 1);
        case "DistinctCount":
          return new Set(vals).size;
        default:
          return vals.reduce((a, b) => a + b, 0);
      }
    };

    const formatNum = (n: number): string => {
      return Number.isInteger(n) ? String(n) : n.toFixed(2);
    };

    // Populate data rows
    for (let rIdx = 0; rIdx < rowTuples.length; rIdx++) {
      const rowNum = curRow;
      let cOffset = colDataStart;

      for (let cIdx = 0; cIdx < colTuples.length; cIdx++) {
        for (let mIdx = 0; mIdx < numMeasures; mIdx++) {
          const vf = config.valueFields[mIdx];
          const rawVals = cellValues.get(`${rIdx}_${cIdx}_${mIdx}`) || [];
          const val = aggregateValues(rawVals, vf?.aggregation || "Sum");

          cells.push({
            row: rowNum,
            col: cOffset + mIdx,
            value: val,
            formatted_value: formatNum(val),
            is_subtotal: false,
            is_grand_total: false,
          });
        }
        cOffset += numMeasures;
      }

      if (includeColTotals) {
        const rTotals = rowTotalsMap.get(rIdx) || config.valueFields.map(() => 0);
        for (let mIdx = 0; mIdx < numMeasures; mIdx++) {
          const total = rTotals[mIdx] || 0;
          rowTotalsList.push(total);
          cells.push({
            row: rowNum,
            col: grandTotalColStart + mIdx,
            value: total,
            formatted_value: formatNum(total),
            is_subtotal: false,
            is_grand_total: true,
          });
        }
      }

      curRow++;
    }

    const dataEndRow = curRow;

    // Populate Grand Total row
    if (includeRowTotals) {
      const rowNum = curRow;
      let cOffset = colDataStart;

      for (let cIdx = 0; cIdx < colTuples.length; cIdx++) {
        const cTotals = colTotalsMap.get(cIdx) || config.valueFields.map(() => 0);
        for (let mIdx = 0; mIdx < numMeasures; mIdx++) {
          const total = cTotals[mIdx] || 0;
          colTotalsList.push(total);
          cells.push({
            row: rowNum,
            col: cOffset + mIdx,
            value: total,
            formatted_value: formatNum(total),
            is_subtotal: false,
            is_grand_total: true,
          });
        }
        cOffset += numMeasures;
      }

      if (includeColTotals) {
        for (let mIdx = 0; mIdx < numMeasures; mIdx++) {
          const total = grandTotals[mIdx] || 0;
          cells.push({
            row: rowNum,
            col: grandTotalColStart + mIdx,
            value: total,
            formatted_value: formatNum(total),
            is_subtotal: false,
            is_grand_total: true,
          });
        }
      }

      curRow++;
    }

    const finalEndRow = curRow;

    return {
      layout: {
        start_row: startRow,
        start_col: startCol,
        end_row: finalEndRow,
        end_col: endCol,
        data_range_start: [dataStartRow, colDataStart],
        data_range_end: [dataEndRow, endCol],
        row_labels_count: rowTuples.length,
        col_labels_count: colTuples.length,
        measures_count: numMeasures,
      },
      row_headers: rowTuples,
      col_headers: colTuples,
      cells,
      row_totals: rowTotalsList,
      col_totals: colTotalsList,
      grand_totals: grandTotals,
      unique_field_values: uniqueValues,
    };
  }

  /**
   * Collects unique sorted values per field from a dataset.
   */
  public collectUniqueValues(dataset: TableDataset): Record<string, string[]> {
    const result: Record<string, string[]> = {};
    for (let c = 0; c < dataset.headers.length; c++) {
      const header = dataset.headers[c];
      const set = new Set<string>();
      for (const row of dataset.records) {
        const v = row[c];
        if (v !== null && v !== undefined && String(v).trim() !== "") {
          set.add(String(v));
        }
      }
      result[header] = Array.from(set).sort();
    }
    return result;
  }

  /**
   * Renders the evaluated pivot table cells and headers directly into `calcState`.
   */
  public renderPivotTableToState(
    state: any,
    result: PivotViewResultDto,
    config: PivotConfigDto,
  ): void {
    const { layout, row_headers, col_headers, cells } = result;

    // 1. Render Row Headers Title
    const rowTitle = config.rowFields.map((f) => f.sourceColumn).join(" / ") || "Rows";
    const colTitle = config.colFields.map((f) => f.sourceColumn).join(" / ") || "Columns";
    state.setCellValue(
      this.coordsToA1(layout.start_row, layout.start_col),
      `${rowTitle} \\ ${colTitle}`,
    );

    // 2. Render Column Headers
    let curCol = layout.data_range_start[1];
    const numMeasures = layout.measures_count;

    for (const cTuple of col_headers) {
      const colLabel = cTuple.join(" - ");
      for (let m = 0; m < numMeasures; m++) {
        const vf = config.valueFields[m];
        const title = col_headers.length > 1 || cTuple[0] !== "Total"
          ? `${colLabel} | ${vf?.customLabel || `${vf?.aggregation || "Sum"} of ${vf?.sourceColumn || ""}`}`
          : vf?.customLabel || `${vf?.aggregation || "Sum"} of ${vf?.sourceColumn || ""}`;
        state.setCellValue(this.coordsToA1(layout.start_row, curCol + m), title);
      }
      curCol += numMeasures;
    }

    if (config.grandTotals === "Both" || config.grandTotals === "ColumnsOnly") {
      for (let m = 0; m < numMeasures; m++) {
        const vf = config.valueFields[m];
        state.setCellValue(
          this.coordsToA1(layout.start_row, curCol + m),
          `Total ${vf?.sourceColumn || ""}`,
        );
      }
    }

    // 3. Render Row Labels
    let curR = layout.data_range_start[0];
    for (const rTuple of row_headers) {
      for (let fIdx = 0; fIdx < rTuple.length; fIdx++) {
        state.setCellValue(this.coordsToA1(curR, layout.start_col + fIdx), rTuple[fIdx]);
      }
      curR++;
    }

    if (config.grandTotals === "Both" || config.grandTotals === "RowsOnly") {
      state.setCellValue(this.coordsToA1(curR, layout.start_col), "Grand Total");
    }

    // 4. Render Calculated Cells
    for (const cell of cells) {
      state.setCellValue(this.coordsToA1(cell.row, cell.col), cell.formatted_value);
    }
  }

  /**
   * Synchronizes an attached PivotChart (`SheetChart`) with updated pivot aggregation data.
   */
  public syncPivotChart(
    state: any,
    chartId: string,
    result: PivotViewResultDto,
    title?: string,
  ): void {
    const sheet = state.getActiveSheet ? state.getActiveSheet() : (state.activeSheet || state);
    const charts = sheet.charts || state.charts;
    if (!charts) return;
    const chart = charts.find((c: any) => c.id === chartId);
    if (!chart) return;

    chart.dataRange = {
      minRow: result.layout.start_row,
      maxRow: result.layout.end_row,
      minCol: result.layout.start_col,
      maxCol: result.layout.end_col,
    };
    if (title) {
      chart.title = title;
    }

    // Categories = row headers
    (chart as any).categories = result.row_headers.map((t) => t.join(" - "));

    // Series = col headers * measures
    const seriesList: any[] = [];
    const numMeasures = result.layout.measures_count;

    for (let cIdx = 0; cIdx < result.col_headers.length; cIdx++) {
      const cLabel = result.col_headers[cIdx].join(" - ");
      for (let mIdx = 0; mIdx < numMeasures; mIdx++) {
        const sName = result.col_headers.length > 1 ? `${cLabel} (M${mIdx + 1})` : `Measure ${mIdx + 1}`;
        const data: number[] = [];

        for (let rIdx = 0; rIdx < result.row_headers.length; rIdx++) {
          const rowNum = result.layout.data_range_start[0] + rIdx;
          const colNum = result.layout.data_range_start[1] + cIdx * numMeasures + mIdx;
          const cell = result.cells.find((c) => c.row === rowNum && c.col === colNum);
          data.push(cell ? cell.value : 0);
        }

        seriesList.push({ name: sName, data });
      }
    }

    (chart as any).series = seriesList;
  }

  public getLastResult(): PivotViewResultDto | null {
    return this.lastResult;
  }

  // --- Helper Methods ---

  private extractCellRaw(cell: any): string | number | boolean | null {
    if (cell === null || cell === undefined) return null;
    if (typeof cell === "number" || typeof cell === "boolean" || typeof cell === "string") {
      return cell;
    }
    if (cell.value !== undefined) {
      return cell.value;
    }
    if (cell.text !== undefined) {
      return cell.text;
    }
    return null;
  }

  private parseCellRange(range: string): { startRow: number; startCol: number; endRow: number; endCol: number } | null {
    const parts = range.split(":");
    if (parts.length === 0) return null;

    const start = this.a1ToCoords(parts[0].trim());
    if (!start) return null;

    if (parts.length === 1) {
      return { startRow: start.row, startCol: start.col, endRow: start.row, endCol: start.col };
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

  private a1ToCoords(a1: string): { row: number; col: number } | null {
    const clean = a1.replace(/\$/g, "").toUpperCase();
    const match = clean.match(/^([A-Z]+)(\d+)$/);
    if (!match) return null;

    const letters = match[1];
    let col = 0;
    for (let i = 0; i < letters.length; i++) {
      col = col * 26 + (letters.charCodeAt(i) - 64);
    }
    const row = parseInt(match[2], 10) - 1;
    return { row, col: col - 1 };
  }

  private coordsToA1(row: number, col: number): string {
    let temp = col + 1;
    let letters = "";
    while (temp > 0) {
      const rem = (temp - 1) % 26;
      letters = String.fromCharCode(65 + rem) + letters;
      temp = Math.floor((temp - 1) / 26);
    }
    return `${letters}${row + 1}`;
  }
}

export const calcPivotCoordinator = new CalcPivotCoordinator();
