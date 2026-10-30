# Subphase 23.1.2: Working Excel Ribbon: Formatting, Sort/Filter, Row/Col Ops & Live Charts
- **Stage:** 23 | **Phase:** 23.1
- **Target Subsystem:** `excel-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** 100% working Excel ribbon across Home, Insert (Charts), Page Layout, Formulas, Data (Sort/Filter), and View tabs.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 23.1.2
Invoke @adaptive-ui-builder under GEC-v3.0.
1. Wire every button in the Excel Ribbon to operate on the selected cell or cell range:
   - **Home Tab:** Font family/size, Bold/Italic/Underline, Cell Fill Color, Font Color, Cell Borders (All, Outside, None), Align Left/Center/Right, Number Format (General, Currency $, Percentage %, Comma ,, + .00, - .00), Insert Row Above, Insert Column Left, Delete Row, Delete Column, Clear All.
   - **Insert Tab:** Bar Chart, Line Chart, Pie Chart (creates a floating, draggable, live-updating SVG chart from the currently selected cells), Insert Comment.
   - **Formulas Tab:** AutoSum (Î£) (automatically inserts =SUM(...) for the cells above/left), AVERAGE, COUNT, MAX, MIN, Insert Function (fx) dialog, and Show Formulas toggle (displays raw formulas vs computed values across the sheet).
   - **Data Tab:** Working Sort A -> Z, Sort Z -> A, Filter toggle, Remove Duplicates, and Text to Columns (splits comma-separated cell text across adjacent columns).
   - **Status Bar:** Live display of SUM, AVERAGE, COUNT, MIN, MAX for any multi-cell selection, plus Zoom slider.
2. Verify via automated DOM tests and commit with [GEC-Verified].
```
