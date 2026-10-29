# Subphase 14.1.2: 2D Sparse Grid Matrix CRDT & Coordinate Shifter
- **Stage:** 14 | **Phase:** 14.1
- **Target Subsystem:** `crdt-collab` & `excel-engine`
- **Primary Agent Skill:** `@crdt-architect`
- **Deliverable:** Cell-level LWW register with vector clocks, structural coordinate shifter (row/column insert/delete transform), formula ref updating under remote mutations.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 14.1.2
Invoke @crdt-architect under GEC-v2.0:
1. In `crates/crdt-collab`, implement `GridMatrixCrdt`:
   - LWW-Register per cell address `(sheet_id, row, col)` backed by Lamport vector clocks.
   - Support for cell values (Number, Text, Boolean, Error), formulas, styles, and format strings.
2. Implement `CoordinateShifter`:
   - Intent-preserving transforms for concurrent row/column insertions and deletions across remote peers.
   - Automatic rewriting of formula coordinates (e.g. updating `=SUM(A1:A10)` when a row is inserted at row 5 on a remote peer).
3. Connect `GridMatrixCrdt` with `excel-engine` Compressed Sparse Page Grid (CSPG) and calculation DAG.
4. Add automated unit tests verifying concurrent cell edits, conflicting cell updates with deterministic clock resolution, and multi-sheet row shifts.
5. Run `powershell -File tools/verify-contract.ps1` and ensure all gates pass. Commit with [GEC-Verified].
```

