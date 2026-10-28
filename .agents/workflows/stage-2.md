---
description: Automated execution workflow for Stage 2: Excel Engine (SpreadsheetML Parser, CSPG Sparse Grid, DAG Calc & 450+ Functions)
---
# Workflow: Execute Stage 2 (/stage-2)
## Objective: Stage 2: Excel Engine (SpreadsheetML Parser, CSPG Sparse Grid, DAG Calc & 450+ Functions)

When invoked via `/stage-2`, execute the following subphases sequentially. Do NOT advance to the next subphase until `tools/verify-contract.ps1` exits with code 0 and `specs/stage-tracker.json` is updated:

### Subphase 2.1.1: SharedStringTable, Styles & Parallel Workbook Parser
- **Prompt Spec File:** `.agents/stages/stage-2/subphase-2.1.1.md`

**Instructions:**
```text
/execute-subphase 2.1.1
Invoke @ecma376-architect under GEC-v1.0 compliance.
1. In `crates/excel-engine`, implement streaming `quick-xml` parsers and serializers for:
   - `xl/workbook.xml` (sheets, definedNames, calcPr, 1900 vs 1904 date system flag).
   - `xl/sharedStrings.xml` using deduplicated string interning (`Arc<str>` or arena table) capable of parsing 1,000,000 shared strings in `< 120 ms`.
   - `xl/styles.xml` (`numFmts`, `fonts`, `fills`, `borders`, `cellXfs`).
2. Preserve all unmodeled workbook extensions (`extLst`) via `ooxml-opc::RawXmlNodeBuffer`.
3. Create 25 test vectors in `specs/evidence/excel/2.1.1_vectors.json`, verify via `subphase_2_1_1_test.rs`, and commit with `[GEC-Verified]`.
```

**Post-Subphase Gate:**
Run `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 2.1.1` to verify contract compliance and advance the state tracker.

### Subphase 2.1.2: Compressed Sparse Page Grid (CSPG) Storage Engine
- **Prompt Spec File:** `.agents/stages/stage-2/subphase-2.1.2.md`

**Instructions:**
```text
/execute-subphase 2.1.2
Invoke @ecma376-architect under GEC-v1.0 Article III (Memory & Performance SLAs).
1. In `crates/excel-engine/src/grid.rs`, implement the Compressed Sparse Page Grid (CSPG) supporting full Excel sheet bounds (`1,048,576` rows by `16,384` columns, `A1:XFD1048576`):
   - Partition each worksheet into 64x64 cell pages indexed by a sparse `BTreeMap<(u16, u8), PageChunk>`.
   - Store cell payloads in Structure-of-Arrays (SoA) layout (`f64` values, string IDs, formula AST IDs, style indices) for cache-optimal SIMD scanning.
2. Implement parallel `rayon` worksheet XML loading (`xl/worksheets/sheet*.xml`) and serializer.
3. Verify memory footprint and O(1) lookup with >= 25 sparse/dense boundary vectors in `subphase_2_1_2_test.rs` and commit with `[GEC-Verified]`.
```

**Post-Subphase Gate:**
Run `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 2.1.2` to verify contract compliance and advance the state tracker.

### Subphase 2.2.1: Pratt Formula Parser, 3D Ranges & Dynamic Array Spill AST
- **Prompt Spec File:** `.agents/stages/stage-2/subphase-2.2.1.md`

**Instructions:**
```text
/execute-subphase 2.2.1
Invoke @ecma376-architect under GEC-v1.0 compliance.
1. In `crates/excel-engine/src/formula/`, build a zero-panic Pratt parser and bytecode compiler for ECMA-376 formulas:
   - Cell references (`A1`, `$A$1`, `R1C1`), 3D sheet spans (`Sheet1:Sheet3!A1:B10`), union/intersection operators (`,`, ` `, `:`), structured table refs (`Table1[[#Headers],[Sales]]`), and dynamic array spill refs (`A1#`, `@`).
   - Exact operator precedence (`%`, `^`, `*`/`/`, `+`/`-`, `&`, comparison operators).
2. Store compiled bytecode alongside raw formula strings for lossless XML serialization.
3. Validate against >= 50 complex formula syntax trees in `subphase_2_2_1_test.rs` and commit with `[GEC-Verified]`.
```

**Post-Subphase Gate:**
Run `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 2.2.1` to verify contract compliance and advance the state tracker.

### Subphase 2.2.2: REA Calc Chain DAG & IEEE-754 Snap-to-Zero Parity
- **Prompt Spec File:** `.agents/stages/stage-2/subphase-2.2.2.md`

**Instructions:**
```text
/execute-subphase 2.2.2
NOTE: Ensure `.\rea_bootstrap.ps1` is active if inspecting local `EXCEL.EXE`.
1. Invoke @rea-binary-analyst (and `tools/com-ground-truth/generate_truth.py`) to document how `EXCEL.EXE` handles:
   - Dirty-flag propagation, circular reference detection, and iterative calculation convergence (`iterate`, `iterateCount`, `iterateDelta`).
   - Undocumented IEEE-754 subnormal / snap-to-zero rounding where `=1.333-1.225-0.108` vs `=(1.333-1.225-0.108)` diverges due to top-level parenthesis suppression of the 15-significant-digit snap heuristic.
2. Save the clean-room mathematical proof in `specs/evidence/excel/2.2.2.md` and >= 35 COM-verified JSON test vectors in `specs/evidence/excel/2.2.2_vectors.json`.
3. Implement the clean-room DAG evaluator and IEEE-754 snap arithmetic in `crates/excel-engine/src/calc_dag.rs`.
4. Pass 100% of `subphase_2_2_2_test.rs` and commit with `[GEC-Verified]`.
```

**Post-Subphase Gate:**
Run `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 2.2.2` to verify contract compliance and advance the state tracker.

### Subphase 2.2.3: Parallel Evaluation of Core & Financial/Lookup Functions
- **Prompt Spec File:** `.agents/stages/stage-2/subphase-2.2.3.md`

**Instructions:**
```text
/execute-subphase 2.2.3
1. Invoke @rea-binary-analyst and COM ground-truth scripts to capture exact boundary/convergence rules for:
   - Lookup & Reference: `XLOOKUP`, `XMATCH`, `VLOOKUP`, `HLOOKUP`, `INDEX`, `MATCH`, `OFFSET`, `INDIRECT`, `FILTER`, `UNIQUE`, `SORT`.
   - Math & Statistical: `SUMIFS`, `COUNTIFS`, `AVERAGEIFS`, `SUBTOTAL`, `AGGREGATE`, `ROUND`, `ROUNDUP`, `ROUNDDOWN`, `MROUND`, `STDEV.S`, `PERCENTILE.INC`.
   - Financial & Date: `PMT`, `IPMT`, `PPMT`, `NPV`, `XNPV`, `IRR`, `XIRR` (Newton-Raphson secant fallback bounds), `WORKDAY.INTL`, `NETWORKDAYS.INTL`, `EDATE`, `EOMONTH` (including Excel's intentional 1900 leap-year bug `1900-02-29` = serial `60`).
   - Modern Lambda/Let: `LET`, `LAMBDA`, `MAP`, `REDUCE`, `SCAN`.
2. Write the mathematical specs and >= 50 test vectors to `specs/evidence/excel/2.2.3_vectors.json`.
3. Implement the function registry in `crates/excel-engine/src/functions/` with `rayon` parallel evaluation across independent DAG levels.
4. Pass `subphase_2_2_3_test.rs` and commit with `[GEC-Verified]`.
```

**Post-Subphase Gate:**
Run `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 2.2.3` to verify contract compliance and advance the state tracker.

