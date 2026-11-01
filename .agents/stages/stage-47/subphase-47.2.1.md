# Sub-Phase 47.2.1: End-to-End Rust OLAP Cube to Pivot Table UI Binding

## Objective
Establish an end-to-end, high-performance binding between the authoritative Rust OLAP Cube engine (`crates/data-connectors/src/olap.rs`) and the unified Calc spreadsheet UI (`PivotFieldListSidebar`, `CalcPivotCoordinator`). Eliminate mock datasets and client-side split-brain calculations, enabling real-time multi-dimensional pivot aggregations directly over spreadsheet grid data and external datasets.

## Scope of Features

### 1. Grid Source Ingestion & Dataset Extraction (`crates/data-connectors/src/olap.rs`)
- **Grid Data Extraction:**
  - `extract_dataset_from_grid(grid: &CompressedSparsePageGrid, start_row: u32, start_col: u32, end_row: u32, end_col: u32, has_headers: bool) -> ConnectorResult<(Schema, Vec<Vec<DataValue>>)>`.
  - Type inference and coercion from `CellValue` (text, number, date, boolean, blank) to `DataValue` and strongly-typed `Schema`.
  - Automatic column header detection and deduplication.
- **Unique Field Values Collector:**
  - `OlapCube::collect_unique_values(records: &[Vec<DataValue>], schema: &Schema) -> BTreeMap<String, Vec<String>>`.
  - Powers interactive slicers and filter dropdowns with real distinct values.

### 2. Evaluated Pivot Table View Model & Reporting (`crates/data-connectors/src/olap.rs`)
- **Structured Pivot View Result:**
  - `PivotViewCell`: `row`, `col`, `value`, `formatted_value`, `is_subtotal`, `is_grand_total`.
  - `PivotViewResult`: `report`, `row_headers`, `col_headers`, `cells`, `row_totals`, `col_totals`, `grand_totals`, `unique_field_values`.
  - `OlapCube::to_view_result(&self, start_row: u32, start_col: u32) -> PivotViewResult`.
- **Advanced OLAP Features:**
  - Dynamic aggregations: `Sum`, `Count`, `CountNumbers`, `Average`, `Min`, `Max`, `Product`, `Variance`, `StdDev`, `DistinctCount`, `PercentOfTotal`, `PercentOfRow`, `PercentOfCol`.
  - Multi-level hierarchical row and column grouping.
  - Date hierarchy grouping (`Year`, `Quarter`, `Month`, `Day`).
  - Interactive slicer multi-value filtering.
  - Subtotals and Grand Totals calculation.
  - Simulation of Excel's `GETPIVOTDATA` function.

### 3. Tauri Desktop IPC Commands (`apps/unified-shell/src-tauri/`)
- Strongly typed Tauri commands for desktop execution:
  - `compute_pivot_table(source_data, config, start_row, start_col)` -> `Result<PivotViewResult, String>`.
  - `render_pivot_table_to_grid(session_id, sheet_name, source_range, target_cell, config)` -> `Result<PivotTableLayoutReport, String>`.
  - `get_pivot_unique_values(source_data, field_name)` -> `Result<Vec<String>, String>`.

### 4. Presentation Shell Coordinator (`apps/unified-shell/src/state/CalcPivotCoordinator.ts`)
- Reactive coordinator managing active pivot definitions per sheet/document.
- Extracts source data range from active `calcState` grid.
- Dispatches pivot calculation to Rust OLAP engine (with deterministic TypeScript fallback for browser/unit test environments).
- Manages interactive slicers with real unique values and multi-select filtering.
- Renders evaluated headers and cells into `calcState` grid.
- Synchronizes with linked PivotCharts (`SheetChart`).

### 5. UI Integration (`apps/unified-shell/src/components/PivotFieldListSidebar.ts`)
- Connects directly to `CalcPivotCoordinator`.
- Replaces hardcoded mock datasets with real grid source data.
- Dynamically populates available fields from source table.
- Dynamically populates slicer buttons from actual unique column values.
- Dark/Light semantic theme styling and full keyboard accessibility.

## Engineering Invariants (GEC-v7.0)
- `#![forbid(unsafe_code)]` enforced across all Rust crates.
- Zero panics, unwrap, or expect in non-test library code.
- Clean-room implementation of ECMA-376 and open spreadsheet standards.
- 100% test pass rate across Rust workspace and frontend Vitest suites.

