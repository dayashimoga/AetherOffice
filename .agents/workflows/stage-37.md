description: Automated execution workflow for Stage 37: Excel (.xlsx) Fix - Working File Save/Open (.xlsx/.csv) & Complete 7-Tab Ribbon
Workflow: Execute Stage 37 (/stage-37)
Objective: Stage 37: Excel (.xlsx) Fix - Working File Save/Open (.xlsx/.csv) & Complete 7-Tab Ribbon

Execute every subphase sequentially under strict Global Engineering Contract (GEC-v6.0) compliance:

Subphase 37.1.1: Excel Save/Open (.xlsx & .csv) Fix & Complete 7-Tab Ribbon Functionality
Spec File: .agents/stages/stage-37/subphase-37.1.1.md
/execute-subphase 37.1.1
Invoke @adaptive-ui-builder under GEC-v6.0 Article II.
1. **Fix Excel Save & Open Existing `.xlsx` / `.csv` Files:**
   - When the user clicks `Save` (`Ctrl+S`), `File -> Save`, or `File -> Export CSV` in Excel, serialize all sheets (`Sheet1`, `Sheet2`, ...), cell values, formulas, and styles into a real `.xlsx` (OOXML ZIP package containing `[Content_Types].xml`, `_rels/.rels`, `xl/workbook.xml`, `xl/worksheets/sheet1.xml`, `xl/sharedStrings.xml`) OR `.csv`, write it via `save_file_bytes` / download, and display the `"✅ Saved <filename>"` confirmation toast!
   - When the user clicks `Open` (`Ctrl+O`) and selects any `.xlsx` or `.csv` file, parse all sheets and cells and render them immediately on the grid (fixing the bug where `test.xlsx` opened as an empty sheet in `image_975970.jpg`)!
2. **Verify & Complete All 7 Excel Ribbon Tabs (`Home`, `Insert`, `Page Layout`, `Formulas`, `Data`, `Review`, `View`):**
   - Ensure every button works on single cells, cell ranges (`A1:D10`), whole selected columns (`A:C`), and whole selected rows (`1:5`).
   - Verify `PivotTable`, `Column/Line/Pie Charts`, `Sparklines`, `Conditional Formatting`, `Sort A-Z / Z-A`, `Filter` dropdowns on column headers, `Text to Columns`, `Flash Fill`, `Remove Duplicates`, `Data Validation`, `Subtotals`, `Goal Seek`, `Name Manager`, `Show Formulas`, `Freeze Panes`, `View Grid`, and `View Headings`.
3. Write automated tests verifying `.xlsx` and `.csv` Save -> Open round-tripping and all 7 Excel ribbon tabs. Commit with `[GEC-Verified]`.

Completion Gate: powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 37.1.1
