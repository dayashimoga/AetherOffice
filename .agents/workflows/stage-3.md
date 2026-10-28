---
description: Automated execution workflow for Stage 3: Word Engine (WordprocessingML Piece-Tree DOM, Styles, Pagination & Tables)
---
# Workflow: Execute Stage 3 (/stage-3)
## Objective: Stage 3: Word Engine (WordprocessingML Piece-Tree DOM, Styles, Pagination & Tables)

When invoked via `/stage-3`, execute the following subphases sequentially. Do NOT advance to the next subphase until `tools/verify-contract.ps1` exits with code 0 and `specs/stage-tracker.json` is updated:

### Subphase 3.1.1: WordprocessingML Piece-Tree Rope DOM & Sub-Document Parser
- **Prompt Spec File:** `.agents/stages/stage-3/subphase-3.1.1.md`

**Instructions:**
```text
/execute-subphase 3.1.1
Invoke @ecma376-architect under GEC-v1.0 compliance.
1. In `crates/word-engine`, implement parsers and lossless serializers for `word/document.xml`, `word/header*.xml`, `word/footer*.xml`, `word/footnotes.xml`, `word/endnotes.xml`, and `word/comments.xml`.
2. Model paragraphs (`w:p`), runs (`w:r`), bookmarks (`w:bookmarkStart`/`End`), hyperlinks, and revision tracking (`w:ins`, `w:del`) over an immutable B-Tree Piece-Tree / Rope so keystroke insertion/deletion in a 1,000-page document completes in `< 0.5 ms`.
3. Write >= 25 round-trip and edit-latency test vectors in `crates/word-engine/tests/subphase_3_1_1_test.rs` and commit with `[GEC-Verified]`.
```

**Post-Subphase Gate:**
Run `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 3.1.1` to verify contract compliance and advance the state tracker.

### Subphase 3.1.2: ECMA-376 Style Cascade, Numbering Lists & Section Geometry
- **Prompt Spec File:** `.agents/stages/stage-3/subphase-3.1.2.md`

**Instructions:**
```text
/execute-subphase 3.1.2
Invoke @ecma376-architect under GEC-v1.0 compliance.
1. In `crates/word-engine/src/styles.rs` and `numbering.rs`, implement the ECMA-376 Part 1 style hierarchy resolver:
   - `w:docDefaults` -> `w:style` (`basedOn` chain, `next`, `link`) -> conditional `w:tblStylePr` (firstRow, lastRow, band1Horz, etc.) -> direct run/paragraph properties (`w:rPr`, `w:pPr`) including toggle property XOR rules (`w:b`, `w:i`, `w:strike`).
   - Multi-level list numbering engine (`word/numbering.xml`: `w:abstractNum`, `w:num`, `w:lvlOverride`, `w:lvlRestart`) computing exact prefix strings (`1.a.i`, Roman, bullet glyphs).
2. Validate against >= 30 complex nested style and list vectors in `subphase_3_1_2_test.rs` and commit with `[GEC-Verified]`.
```

**Post-Subphase Gate:**
Run `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 3.1.2` to verify contract compliance and advance the state tracker.

### Subphase 3.2.1: REA Line-Breaking Knuth-Plass/Greedy Hybrid & Pagination Solver
- **Prompt Spec File:** `.agents/stages/stage-3/subphase-3.2.1.md`

**Instructions:**
```text
/execute-subphase 3.2.1
NOTE: Ensure `.\rea_bootstrap.ps1` is active if inspecting `wwlib.dll`.
1. Invoke @rea-binary-analyst on staged `wwlib.dll` (and COM ground-truth bounding boxes) to document:
   - Word's exact line-breaking whitespace compression, CJK/Latin kinsoku shori rules, and soft-hyphen penalty thresholds.
   - Pagination constraints: `w:widowControl` (minimum 2 lines at top/bottom of page), `w:keepNext`, `w:keepLines`, `w:pageBreakBefore`, and continuous vs next-page section breaks (`w:sectPr`).
2. Save the clean-room mathematical spec to `specs/evidence/word/3.2.1.md` and >= 25 pagination bounding-box vectors to `specs/evidence/word/3.2.1_vectors.json`.
3. Implement the deterministic pagination and line-layout engine in `crates/word-engine/src/layout.rs`.
4. Pass `subphase_3_2_1_test.rs` and commit with `[GEC-Verified]`.
```

**Post-Subphase Gate:**
Run `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 3.2.1` to verify contract compliance and advance the state tracker.

### Subphase 3.2.2: REA Table Auto-Fit Widths, Row Splitting & Floating Wrap Polygons
- **Prompt Spec File:** `.agents/stages/stage-3/subphase-3.2.2.md`

**Instructions:**
```text
/execute-subphase 3.2.2
1. Invoke @rea-binary-analyst on `wwlib.dll` to extract the mathematical constraints for:
   - Table Auto-Fit (`w:tblLayout w:type="autofit"`): minimum unbroken word width (`minContentWidth`) vs maximum single-line paragraph width (`maxContentWidth`) proportional distribution across `w:gridSpan` merged cells.
   - Cross-page table row breaking (`w:cantSplit`, `w:tblHeader` repeating header rows) and floating DrawingML anchor (`wp:anchor`) exclusion zones (`wrapSquare`, `wrapTight` polygon intersection with line segments).
2. Document formulas in `specs/evidence/word/3.2.2.md` and >= 25 test vectors in `specs/evidence/word/3.2.2_vectors.json`.
3. Implement the table and float layout solver in `crates/word-engine/src/table_layout.rs`.
4. Pass `subphase_3_2_2_test.rs` and commit with `[GEC-Verified]`.
```

**Post-Subphase Gate:**
Run `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 3.2.2` to verify contract compliance and advance the state tracker.

