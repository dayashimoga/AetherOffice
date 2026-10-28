# Subphase 2.1.1: SharedStringTable, Styles & Parallel Workbook Parser
- **Stage:** 2 | **Phase:** 2.1
- **Target Crate:** `excel-engine`
- **Primary Agent Skill:** `@ecma376-architect`
- **REA Binary Analysis Required:** NO (Pure ECMA-376 / Clean-Room)
- **Deliverable:** Multi-threaded parser for xl/workbook.xml, xl/sharedStrings.xml, and xl/styles.xml.
- **Validation Command:** `cargo test --package excel-engine --test subphase_2_1_1_test`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

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
