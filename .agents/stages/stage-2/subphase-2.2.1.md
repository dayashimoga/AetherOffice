# Subphase 2.2.1: Pratt Formula Parser, 3D Ranges & Dynamic Array Spill AST
- **Stage:** 2 | **Phase:** 2.2
- **Target Crate:** `excel-engine`
- **Primary Agent Skill:** `@ecma376-architect`
- **REA Binary Analysis Required:** NO (Pure ECMA-376 / Clean-Room)
- **Deliverable:** Complete Excel formula lexer and AST compiler supporting R1C1, A1, structured tables, and spill (#) operators.
- **Validation Command:** `cargo test --package excel-engine --test subphase_2_2_1_test`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 2.2.1
Invoke @ecma376-architect under GEC-v1.0 compliance.
1. In `crates/excel-engine/src/formula/`, build a zero-panic Pratt parser and bytecode compiler for ECMA-376 formulas:
   - Cell references (`A1`, `$A$1`, `R1C1`), 3D sheet spans (`Sheet1:Sheet3!A1:B10`), union/intersection operators (`,`, ` `, `:`), structured table refs (`Table1[[#Headers],[Sales]]`), and dynamic array spill refs (`A1#`, `@`).
   - Exact operator precedence (`%`, `^`, `*`/`/`, `+`/`-`, `&`, comparison operators).
2. Store compiled bytecode alongside raw formula strings for lossless XML serialization.
3. Validate against >= 50 complex formula syntax trees in `subphase_2_2_1_test.rs` and commit with `[GEC-Verified]`.
```
