# Subphase 10.1.2: Range Selection, Fill Handle Auto-Series & Clipboard Cut/Copy/Paste
- **Stage:** 10 | **Phase:** 10.1
- **Target Subsystem:** `excel-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Click-drag rectangular range selection (A1:D10), Shift+Arrow selection, bottom-right Fill Handle auto-fill, and TSV/CSV clipboard paste.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

`	ext
/execute-subphase 10.1.2
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Implement full range mechanics in Calc:
   - **Range Selection:** Mouse click-and-drag or Shift + Arrow selects a rectangular cell range (A1:C5), updating the Name Box (A1) and displaying live Average, Count, and Sum in the bottom Status Bar.
   - **Fill Handle Drag:** Dragging the small green square in the bottom-right corner of the selection box auto-fills linear numeric series (1, 2 -> 3, 4, 5), months/days, and shifts relative formula references (=A1+B1 -> =A2+B2).
   - **Multi-Cell Clipboard:** Ctrl+C, Ctrl+X, and Ctrl+V copy and paste multi-cell blocks cleanly (compatible with system TSV/Excel clipboard).
2. Verify with automated DOM tests and commit with [GEC-Verified].
`
