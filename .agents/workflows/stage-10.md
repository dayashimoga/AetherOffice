description: Automated execution workflow for Stage 10: Standalone Calc (Excel) - Live Cell Editing, Multi-Sheets, Formulas & Charts
Workflow: Execute Stage 10 (/stage-10)
Objective: Stage 10: Standalone Calc (Excel) - Live Cell Editing, Multi-Sheets, Formulas & Charts

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v2.0) compliance. You MUST implement real, interactive frontend DOM + Rust IPC functionality and pass tools/verify-contract.ps1 (including Frontend E2E tests) before marking any subphase complete:

Subphase 10.1.1: Editable Spreadsheet Grid, In-Cell Input, Multi-Sheet Tabs & Column Resizing
Spec File: .agents/stages/stage-10/subphase-10.1.1.md
/execute-subphase 10.1.1
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Fix the uneditable Excel grid seen in image_cb4d38.png:
   - **Direct Cell Editing:** Typing any printable character on a selected cell, pressing F2, or double-clicking a cell MUST open an active in-cell input editor synchronized in real time with the top Formula Bar (x).
   - **Commit & Navigation:** Pressing Enter commits the value/formula to excel-engine, triggers DAG recalculation, displays the computed result in the cell, and moves selection down (Shift+Enter up, Tab right, Shift+Tab left, Escape cancels edit).
   - **Multi-Sheet Management:** Populate the left Workbook Sheets panel AND a bottom sheet tab bar with Sheet1, a + New Sheet button, double-click to rename sheet, and delete sheet button.
   - **Row/Column Manipulation:** Drag column/row header dividers to resize width/height; right-click row/column headers to Insert Row/Column, Delete Row/Column, or Clear Contents.
2. Write automated DOM tests verifying cell typing, =SUM(A1:A3) evaluation, and creating Sheet2. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 10.1.1
Subphase 10.1.2: Range Selection, Fill Handle Auto-Series & Clipboard Cut/Copy/Paste
Spec File: .agents/stages/stage-10/subphase-10.1.2.md
/execute-subphase 10.1.2
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Implement full range mechanics in Calc:
   - **Range Selection:** Mouse click-and-drag or Shift + Arrow selects a rectangular cell range (A1:C5), updating the Name Box (A1) and displaying live Average, Count, and Sum in the bottom Status Bar.
   - **Fill Handle Drag:** Dragging the small green square in the bottom-right corner of the selection box auto-fills linear numeric series (1, 2 -> 3, 4, 5), months/days, and shifts relative formula references (=A1+B1 -> =A2+B2).
   - **Multi-Cell Clipboard:** Ctrl+C, Ctrl+X, and Ctrl+V copy and paste multi-cell blocks cleanly (compatible with system TSV/Excel clipboard).
2. Verify with automated DOM tests and commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 10.1.2
Subphase 10.2.1: Calc Home, Formulas & Data Ribbons: Number Formats, Function Wizard & Sort/Filter
Spec File: .agents/stages/stage-10/subphase-10.2.1.md
/execute-subphase 10.2.1
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Wire every button across the Home, Formulas, and Data ribbons in CalcRibbon.tsx:
   - **Home Tab:** Font/Size/Bold/Italic, Cell Background Color, Text Color, Borders (All Borders, Thick Box Border, Bottom Border), Alignment, Wrap Text, Merge & Center, and **Number Format** (General, Currency $, Accounting, Percentage %, Comma, Date, Increase/Decrease Decimal).
   - **Formulas Tab:** Replace the dead placeholder text with working buttons: **Insert Function (x modal with search across all 450+ functions)**, **AutoSum (Σ Sum, Average, Count, Max, Min)**, Financial dropdown, Logical dropdown, Text dropdown, Lookup & Reference dropdown, Name Manager, and Show Formulas toggle.
   - **Data Tab:** Make Sort A-Z, Sort Z-A, Filter (adds dropdown arrows to header cells to filter rows by value), Flash Fill, Remove Duplicates, and Text to Columns 100% functional on the selected grid range!
2. Write automated tests verifying Sort A-Z, AutoSum, and Currency formatting. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 10.2.1
Subphase 10.2.2: Calc Insert & View Ribbons: Live Charts (Bar, Line, Pie), Conditional Formatting & Freeze Panes
Spec File: .agents/stages/stage-10/subphase-10.2.2.md
/execute-subphase 10.2.2
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Complete the Insert, Page Layout, and View ribbons in Calc:
   - **Insert Charts:** Selecting a data range (e.g., A1:B4) and clicking Insert -> Column Chart, Line Chart, or Pie Chart renders an interactive, draggable, resizable chart overlay on the sheet that automatically updates whenever cell values change!
   - **Conditional Formatting:** Highlight Cells Rules (Greater Than, Less Than, Duplicate Values) and Data Bars / Color Scales applied dynamically to selected ranges.
   - **View Tab:** Toggle Gridlines, Toggle Formula Bar, Toggle Row/Column Headings, and **Freeze Panes** (Freeze Top Row, Freeze First Column, Freeze Panes at selection).
2. Verify chart generation and conditional formatting via automated DOM tests. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 10.2.2
