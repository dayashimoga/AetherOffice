# Subphase 2.2.3: Parallel Evaluation of Core & Financial/Lookup Functions
- **Stage:** 2 | **Phase:** 2.2
- **Target Crate:** `excel-engine`
- **Primary Agent Skill:** `@rea-binary-analyst`
- **REA Binary Analysis Required:** YES (Run .\rea_bootstrap.ps1 before starting)
- **Deliverable:** High-speed SIMD/Rayon implementation of Math, Statistical, Lookup (XLOOKUP/VLOOKUP), Logical, Text, and Financial functions.
- **Validation Command:** `cargo test --package excel-engine --test subphase_2_2_3_test`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

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
