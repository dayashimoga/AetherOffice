# Sub-Phase 47.1.2: Data Validation, Conditional Formatting Rules Manager & Sparklines

## Objective
Implement industrial-grade Data Validation (dropdown lists, numeric ranges, custom formulas, alert styles), multi-rule Conditional Formatting engine (color scales, data bars, icon sets, highlight rules, top/bottom N, priority manager), and compact micro-chart Sparklines (Line, Column, Win/Loss) in `excel-engine` (AetherCalc) and the unified presentation shell.

## Scope of Features

### 1. Data Validation Engine (`validation.rs`)
- **Data Validation Types:**
  - `Whole`, `Decimal`, `List`, `Date`, `Time`, `TextLength`, `Custom`.
- **Operators:**
  - `Between`, `NotBetween`, `Equal`, `NotEqual`, `GreaterThan`, `LessThan`, `GreaterEqual`, `LessEqual`.
- **Alert Styles & Messages:**
  - Alert styles: `Stop`, `Warning`, `Information`.
  - Configurable prompt title/message and error title/message.
  - In-cell dropdown flag (`show_dropdown`).
  - Blank values allowance (`allow_blank`).
- **Validation Engine:**
  - `validate_cell(val: &CellValue, rule: &DataValidation, grid: Option<&CompressedSparsePageGrid>) -> ValidationResult`:
    - Evaluates list options (comma-separated literal `"A,B,C"` or range references `$A$1:$A$5`).
    - Numeric bounds and text lengths.
    - Custom formula evaluation.
- **XML Codec:**
  - ECMA-376 Part 1 §18.3.1.32 `<dataValidations>` and `<dataValidation>`.

### 2. Conditional Formatting Rules Manager (`conditional_format.rs`)
- **Rule Types:**
  - `CellIs`: comparison operators with 1 or 2 formulas/values.
  - `Expression`: custom boolean formula evaluation.
  - `ColorScale`: 2-color or 3-color scales with min/mid/max thresholds and RGB color interpolation.
  - `DataBar`: min/max thresholds, gradient/solid fill color, show value or bar only.
  - `IconSet`: 3-arrow, 3-traffic-lights, 3-flags, 5-ratings sets with threshold percentiles.
  - `Top10`: top/bottom N items or percent.
  - `DuplicateValues`, `UniqueValues`.
  - `TextContains`, `Blanks`, `Errors`.
- **Priority & Multi-Rule Resolution:**
  - Priority ordering (lower priority number takes precedence).
  - `stop_if_true` short-circuiting.
  - Style merging: font color, bold, italic, cell fill, borders.
- **XML Codec:**
  - ECMA-376 §18.3.1.18 `<conditionalFormatting>` and `<cfRule>`.

### 3. Compact Sparklines Engine (`conditional_format.rs` / `sparklines`)
- **Sparkline Types:**
  - `Line`, `Column`, `Stacked` (Win/Loss).
- **Point Formatting & Highlighting:**
  - Series color, high point color, low point color, first point color, last point color, negative values color.
  - Line markers option.
- **Mini-Chart Rendering:**
  - Extracts numeric data from range.
  - Generates lightweight SVG `<svg>` polyline and rect paths for instant embedding in cell canvas.
- **OpenXML ExtLst Codec:**
  - `<x14:sparklineGroups>` and `<x14:sparklineGroup>` schema conformant.

### 4. Presentation Shell Coordinator (`CalcValidationAndFormatCoordinator.ts`)
- Reactive state manager for active cell validation rules, dropdown list options, and input/error alerts.
- Conditional Formatting Rules Manager dialog state (add, edit, remove, reorder rules, test evaluation).
- Sparkline group management and live SVG preview generator.

## Engineering Invariants (GEC-v7.0)
- `#![forbid(unsafe_code)]` enforced across all Rust crates.
- Zero panics, unwrap, or expect in non-test library code.
- Clean-room implementation of ECMA-376 and open spreadsheet standards.
- 100% test pass rate across Rust workspace and frontend Vitest suites.

