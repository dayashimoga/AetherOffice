# Subphase 2.2.2: REA Calc Chain DAG & IEEE-754 Snap-to-Zero Parity
- **Stage:** 2 | **Phase:** 2.2
- **Target Crate:** `excel-engine`
- **Primary Agent Skill:** `@rea-binary-analyst`
- **REA Binary Analysis Required:** YES (Run .\rea_bootstrap.ps1 before starting)
- **Deliverable:** Clean-room topological dependency graph and exact Excel IEEE-754 floating-point subtraction rounding solver.
- **Validation Command:** `cargo test --package excel-engine --test subphase_2_2_2_test`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

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
