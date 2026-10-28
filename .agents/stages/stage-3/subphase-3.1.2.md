# Subphase 3.1.2: ECMA-376 Style Cascade, Numbering Lists & Section Geometry
- **Stage:** 3 | **Phase:** 3.1
- **Target Crate:** `word-engine`
- **Primary Agent Skill:** `@ecma376-architect`
- **REA Binary Analysis Required:** NO (Pure ECMA-376 / Clean-Room)
- **Deliverable:** Complete style inheritance resolver (docDefaults -> table -> paragraph -> character -> direct) and multi-level list state machine.
- **Validation Command:** `cargo test --package word-engine --test subphase_3_1_2_test`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 3.1.2
Invoke @ecma376-architect under GEC-v1.0 compliance.
1. In `crates/word-engine/src/styles.rs` and `numbering.rs`, implement the ECMA-376 Part 1 style hierarchy resolver:
   - `w:docDefaults` -> `w:style` (`basedOn` chain, `next`, `link`) -> conditional `w:tblStylePr` (firstRow, lastRow, band1Horz, etc.) -> direct run/paragraph properties (`w:rPr`, `w:pPr`) including toggle property XOR rules (`w:b`, `w:i`, `w:strike`).
   - Multi-level list numbering engine (`word/numbering.xml`: `w:abstractNum`, `w:num`, `w:lvlOverride`, `w:lvlRestart`) computing exact prefix strings (`1.a.i`, Roman, bullet glyphs).
2. Validate against >= 30 complex nested style and list vectors in `subphase_3_1_2_test.rs` and commit with `[GEC-Verified]`.
```
