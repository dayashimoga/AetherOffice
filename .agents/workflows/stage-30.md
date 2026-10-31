description: Automated execution workflow for Stage 30: Excel (.xlsx) Fix - Multi-Row/Column Selection, Batch Row/Col Formatting and All Ribbons
Workflow: Execute Stage 30 (/stage-30)
Objective: Stage 30: Excel (.xlsx) Fix - Multi-Row/Column Selection, Batch Row/Col Formatting and All Ribbons

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v5.0) compliance:

### Subphase 30.1.1: Excel Multi-Row & Multi-Column Header Selection + Batch Row/Col Formatting & Editing
- Spec File: .agents/stages/stage-30/subphase-30.1.1.md
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

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 30.1.1

### Subphase 30.1.2: Excel Complete Ribbon Overhaul: Home, Insert, Page Layout, Formulas, Data, Review, View
- Spec File: .agents/stages/stage-30/subphase-30.1.2.md
/execute-subphase 30.1.2
Invoke @adaptive-ui-builder under GEC-v5.0.
1. Ensure **100% of Excel's Ribbon controls** across all 7 tabs are accessible (zero right-edge clipping) and fully functional:
   - **Home Tab:** Clipboard (Cut, Copy, Paste, Paste Values), Font (Family, Size, A+, A-, Bold, Italic, Underline, Borders, Fill Color, Text Color), Alignment (Top/Mid/Bottom, Left/Center/Right, Wrap Text, Merge & Center), Number (Format Dropdown, $, %, ,, .00, .0), Styles (Conditional Formatting rules + Format as Table banded themes), Cells (Insert Row, Insert Col, Delete Row, Delete Col, AutoFit Row Height, AutoFit Col Width), Editing (AutoSum Î£, Fill Down, Fill Right, Fill Series, Clear All/Formats/Contents, Sort & Filter, Find & Replace).
   - **Insert Tab:** PivotTable (opens interactive Pivot Table builder grouping selected range by Row/Column/Sum/Average!), Table, Picture, Shapes, Column Chart, Line Chart, Pie Chart, Area/Scatter Chart (interactive floating charts on grid), Sparklines (Line/Column), Link, Comment.
   - **Page Layout Tab:** Themes, Colors, Fonts, Margins, Orientation, Size, Set Print Area, Clear Print Area, Gridlines (View / Print), Headings (View / Print).
   - **Formulas Tab:** Insert Function (fx) modal, AutoSum (Sum, Average, Count, Max, Min), Financial, Logical, Text, Date & Time, Lookup & Reference, Math & Trig, Name Manager, Show Formulas, Trace Precedents/Dependents, Evaluate Formula.
   - **Data Tab:** From CSV/JSON, Sort A-Z, Sort Z-A, Custom Multi-Column Sort, Filter (interactive column header filter checkboxes), Clear Filter, Text to Columns, Flash Fill, Remove Duplicates, Data Validation, Group / Ungroup Rows & Cols, Subtotal, Goal Seek.
   - **Review & View Tabs:** New Comment, Delete Comment, Show All Comments, Protect Sheet, Workbook Stats, Normal / Page Break View, Formula Bar, Gridlines, Headings, Freeze Top Row / First Column / Selected Cell, Split Sheet, Zoom.
2. Verify all Excel ribbon buttons via automated DOM tests. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 30.1.2

