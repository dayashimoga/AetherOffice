# Subphase 3.2.1: REA Line-Breaking Knuth-Plass/Greedy Hybrid & Pagination Solver
- **Stage:** 3 | **Phase:** 3.2
- **Target Crate:** `word-engine`
- **Primary Agent Skill:** `@rea-binary-analyst`
- **REA Binary Analysis Required:** YES (Run .\rea_bootstrap.ps1 before starting)
- **Deliverable:** Exact Word line-breaking, kerning, widow/orphan control, and multi-column page-break solver.
- **Validation Command:** `cargo test --package word-engine --test subphase_3_2_1_test`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

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
