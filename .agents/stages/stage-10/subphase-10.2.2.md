# Subphase 10.2.2: Calc Insert & View Ribbons: Live Charts (Bar, Line, Pie), Conditional Formatting & Freeze Panes
- **Stage:** 10 | **Phase:** 10.2
- **Target Subsystem:** `excel-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Interactive floating SVG/Canvas Charts bound to cell ranges, Conditional Formatting rules, and Freeze Panes/Gridlines toggles.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

`	ext
/execute-subphase 10.2.2
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Complete the Insert, Page Layout, and View ribbons in Calc:
   - **Insert Charts:** Selecting a data range (e.g., A1:B4) and clicking Insert -> Column Chart, Line Chart, or Pie Chart renders an interactive, draggable, resizable chart overlay on the sheet that automatically updates whenever cell values change!
   - **Conditional Formatting:** Highlight Cells Rules (Greater Than, Less Than, Duplicate Values) and Data Bars / Color Scales applied dynamically to selected ranges.
   - **View Tab:** Toggle Gridlines, Toggle Formula Bar, Toggle Row/Column Headings, and **Freeze Panes** (Freeze Top Row, Freeze First Column, Freeze Panes at selection).
2. Verify chart generation and conditional formatting via automated DOM tests. Commit with [GEC-Verified].
`
