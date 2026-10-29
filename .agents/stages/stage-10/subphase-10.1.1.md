# Subphase 10.1.1: Editable Spreadsheet Grid, In-Cell Input, Multi-Sheet Tabs & Column Resizing
- **Stage:** 10 | **Phase:** 10.1
- **Target Subsystem:** `excel-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Full cell editing (double-click, F2, direct typing), Formula Bar sync, Sheet1/Sheet2/+ tab bar, and column/row drag-resizing.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

`	ext
/execute-subphase 10.1.1
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Fix the uneditable Excel grid seen in image_cb4d38.png:
   - **Direct Cell Editing:** Typing any printable character on a selected cell, pressing F2, or double-clicking a cell MUST open an active in-cell input editor synchronized in real time with the top Formula Bar (x).
   - **Commit & Navigation:** Pressing Enter commits the value/formula to excel-engine, triggers DAG recalculation, displays the computed result in the cell, and moves selection down (Shift+Enter up, Tab right, Shift+Tab left, Escape cancels edit).
   - **Multi-Sheet Management:** Populate the left Workbook Sheets panel AND a bottom sheet tab bar with Sheet1, a + New Sheet button, double-click to rename sheet, and delete sheet button.
   - **Row/Column Manipulation:** Drag column/row header dividers to resize width/height; right-click row/column headers to Insert Row/Column, Delete Row/Column, or Clear Contents.
2. Write automated DOM tests verifying cell typing, =SUM(A1:A3) evaluation, and creating Sheet2. Commit with [GEC-Verified].
`
