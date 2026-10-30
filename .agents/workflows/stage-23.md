description: Automated execution workflow for Stage 23: Excel (Calc) - Blank Virtualized Grid, Formulas, Sheets & Charts
Workflow: Execute Stage 23 (/stage-23)
Objective: Stage 23: Excel (Calc) - Blank Virtualized Grid, Formulas, Sheets & Charts

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v3.0) compliance. You MUST implement real, interactive frontend DOM + Rust IPC functionality and pass tools/verify-contract.ps1 before marking any subphase complete:
Subphase 23.1.1: Blank Virtualized Excel Grid, Instant Typing, Formula Engine & Multi-Sheet Bar
Spec File: .agents/stages/stage-23/subphase-23.1.1.md
/execute-subphase 23.1.1
Invoke @adaptive-ui-builder under GEC-v3.0.
1. Rebuild the Excel (Calc) workspace (pps/unified-shell/src/apps/calc/) so it opens to a **100% blank, editable grid** (Book1.xlsx -> Sheet1 with zero fake numbers):
   - **Lightweight Virtualized Grid:** Render a smooth scrollable grid where clicking any cell selects it, and **typing any key immediately starts editing that cell** (or double-clicking / pressing F2).
   - **Navigation Keys:** Enter commits and moves down, Tab commits and moves right, Arrow keys move selection when not editing, Delete/Backspace clears selected cell(s), and Escape cancels editing.
   - **Live Formula Evaluator:** Typing =SUM(A1:B5), =AVERAGE(...), =MIN(...), =MAX(...), =COUNT(...), =IF(A1>10, "Yes", "No"), =VLOOKUP(...), or math expressions (=A1*B1+100) evaluates immediately and updates dependent cells automatically when source cells change.
   - **Working Multi-Sheet Management:** Add a bottom Sheet Tab Bar (Sheet1, Sheet2, ...) with a working **+ button to create new sheets**, double-click to rename a sheet, right-click/button to delete a sheet, and sync with the left Workbook Sheets list.
2. Write automated DOM tests verifying blank boot, typing numbers into A1 and A2, evaluating =SUM(A1:A2) in A3, and clicking + to create Sheet2. Commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 23.1.1
Subphase 23.1.2: Working Excel Ribbon: Formatting, Sort/Filter, Row/Col Ops & Live Charts
Spec File: .agents/stages/stage-23/subphase-23.1.2.md
/execute-subphase 23.1.2
Invoke @adaptive-ui-builder under GEC-v3.0.
1. Wire every button in the Excel Ribbon to operate on the selected cell or cell range:
   - **Home Tab:** Font family/size, Bold/Italic/Underline, Cell Fill Color, Font Color, Cell Borders (All, Outside, None), Align Left/Center/Right, Number Format (General, Currency $, Percentage %, Comma ,, + .00, - .00), Insert Row Above, Insert Column Left, Delete Row, Delete Column, Clear All.
   - **Insert Tab:** Bar Chart, Line Chart, Pie Chart (creates a floating, draggable, live-updating SVG chart from the currently selected cells), Insert Comment.
   - **Formulas Tab:** AutoSum (Î£) (automatically inserts =SUM(...) for the cells above/left), AVERAGE, COUNT, MAX, MIN, Insert Function (fx) dialog, and Show Formulas toggle (displays raw formulas vs computed values across the sheet).
   - **Data Tab:** Working Sort A -> Z, Sort Z -> A, Filter toggle, Remove Duplicates, and Text to Columns (splits comma-separated cell text across adjacent columns).
   - **Status Bar:** Live display of SUM, AVERAGE, COUNT, MIN, MAX for any multi-cell selection, plus Zoom slider.
2. Verify via automated DOM tests and commit with [GEC-Verified].

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 23.1.2

