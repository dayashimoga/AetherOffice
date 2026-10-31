# Subphase 30.1.1: Excel Multi-Row & Multi-Column Header Selection + Batch Row/Col Formatting & Editing
- **Stage:** 30 | **Phase:** 30.1
- **Target Subsystem:** `excel-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Click/drag row headers (1..N) and column headers (A..Z) to select whole rows/columns and apply batch formatting, formulas, resize, insert, or delete.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 30.1.1
Invoke @adaptive-ui-builder under GEC-v5.0 Article III.2.
1. **Implement Full Row, Multi-Row, Column, Multi-Column & Rectangular Range Selection in Excel:**
   - **Column Selection:** Clicking column header B selects the entire Column B (highlighting all cells in Column B and updating the Name Box to B:B). Clicking B and dragging across to E (or Shift+Click on E) selects Columns B:E. Ctrl+Click adds non-contiguous columns.
   - **Row Selection:** Clicking row header 3 selects the entire Row 3 (3:3). Dragging 3 to 10 (or Shift+Click on 10) selects Rows 3:10.
   - **Select All Sheet Corner (â—¢):** Clicking the top-left corner cell above 1 and left of A selects the entire worksheet!
   - **Cell Range Drag Selection:** Clicking cell B2 and dragging to D8 (or Shift+Arrow) selects the 2D range B2:D8.
2. **Row-Wise & Column-Wise Batch Operations:**
   - When multiple rows, columns, or cell ranges are selected, clicking ANY formatting or editing command (Bold, Italic, Underline, Font Family, Font Size, Text Color, Fill Color, Borders, Align Left/Center/Right, Number Format $ / % / , / .00, Wrap Text, Clear Contents, Delete, Insert Rows/Columns, Resize Column Width / Row Height) MUST apply simultaneously to **every cell in the selected rows/columns/range**!
3. Write automated DOM tests verifying: clicking Column B and dragging to D, applying Bold & Currency $ format, and asserting cells B1..D20 all receive Bold & Currency formatting; doing the same for Rows 2..5. Commit with [GEC-Verified].
```
