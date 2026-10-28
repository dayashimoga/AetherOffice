# Subphase 3.2.2: REA Table Auto-Fit Widths, Row Splitting & Floating Wrap Polygons
- **Stage:** 3 | **Phase:** 3.2
- **Target Crate:** `word-engine`
- **Primary Agent Skill:** `@rea-binary-analyst`
- **REA Binary Analysis Required:** YES (Run .\rea_bootstrap.ps1 before starting)
- **Deliverable:** Table column auto-fit solver, cross-page repeating table headers, and tight polygon text wrapping around floating DrawingML anchors.
- **Validation Command:** `cargo test --package word-engine --test subphase_3_2_2_test`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 3.2.2
1. Invoke @rea-binary-analyst on `wwlib.dll` to extract the mathematical constraints for:
   - Table Auto-Fit (`w:tblLayout w:type="autofit"`): minimum unbroken word width (`minContentWidth`) vs maximum single-line paragraph width (`maxContentWidth`) proportional distribution across `w:gridSpan` merged cells.
   - Cross-page table row breaking (`w:cantSplit`, `w:tblHeader` repeating header rows) and floating DrawingML anchor (`wp:anchor`) exclusion zones (`wrapSquare`, `wrapTight` polygon intersection with line segments).
2. Document formulas in `specs/evidence/word/3.2.2.md` and >= 25 test vectors in `specs/evidence/word/3.2.2_vectors.json`.
3. Implement the table and float layout solver in `crates/word-engine/src/table_layout.rs`.
4. Pass `subphase_3_2_2_test.rs` and commit with `[GEC-Verified]`.
```
