# Subphase 18.1.1: Streaming Ingestion Connectors (Apache Arrow, Parquet, CSV, SQLite & REST/JSON)
- **Stage:** 18 | **Phase:** 18.1
- **Target Subsystem:** `data-connectors`
- **Primary Agent Skill:** `@data-engineer`
- **Deliverable:** High-throughput streaming data connectors importing structured datasets into Calc's Compressed Sparse Page Grid (CSPG), automatic schema inference, and data type coercion.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 18.1.1
Invoke @data-engineer under GEC-v2.0:
1. Create new crate `crates/data-connectors` with strict `#![forbid(unsafe_code)]` and zero panic macros.
2. Implement tabular stream readers for Delimited CSV/TSV, JSON arrays, SQLite tables, and Apache Arrow/Parquet record batches.
3. Implement zero-copy columnar ingestion mapping external records directly into `excel_engine::grid::CompressedSparsePageGrid`.
4. Provide schema inference engine detecting numbers, dates, booleans, and strings with configurable null/NaN handling.
5. Add unit and integration tests verifying 100k-row ingestion within SLA budget (< 200ms) and type fidelity.
6. Run `powershell -File tools/verify-contract.ps1` and ensure all gates pass. Commit with [GEC-Verified].
```

