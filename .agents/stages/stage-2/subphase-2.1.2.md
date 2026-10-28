# Subphase 2.1.2: Compressed Sparse Page Grid (CSPG) Storage Engine
- **Stage:** 2 | **Phase:** 2.1
- **Target Crate:** `excel-engine`
- **Primary Agent Skill:** `@ecma376-architect`
- **REA Binary Analysis Required:** NO (Pure ECMA-376 / Clean-Room)
- **Deliverable:** SIMD-aligned 64x64 chunked sparse grid supporting 1,048,576 rows x 16,384 columns under 180MB RSS.
- **Validation Command:** `cargo test --package excel-engine --test subphase_2_1_2_test`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 2.1.2
Invoke @ecma376-architect under GEC-v1.0 Article III (Memory & Performance SLAs).
1. In `crates/excel-engine/src/grid.rs`, implement the Compressed Sparse Page Grid (CSPG) supporting full Excel sheet bounds (`1,048,576` rows by `16,384` columns, `A1:XFD1048576`):
   - Partition each worksheet into 64x64 cell pages indexed by a sparse `BTreeMap<(u16, u8), PageChunk>`.
   - Store cell payloads in Structure-of-Arrays (SoA) layout (`f64` values, string IDs, formula AST IDs, style indices) for cache-optimal SIMD scanning.
2. Implement parallel `rayon` worksheet XML loading (`xl/worksheets/sheet*.xml`) and serializer.
3. Verify memory footprint and O(1) lookup with >= 25 sparse/dense boundary vectors in `subphase_2_1_2_test.rs` and commit with `[GEC-Verified]`.
```
