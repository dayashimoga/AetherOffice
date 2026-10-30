---
description: Automated execution workflow for Stage 30: Excel (.xlsx) Complete Fix - All 7 Ribbons, Formulas, Charts, Sort/Filter and Page Layout
---

# Workflow: Execute Stage 30 (/stage-30)

## Objective: Stage 30: Excel (.xlsx) Complete Fix - All 7 Ribbons, Formulas, Charts, Sort/Filter and Page Layout
Execute every subphase sequentially under strict Global Engineering Contract (GEC-v4.0) compliance. Every button and feature must be implemented and verified via automated Vitest DOM tests before marking complete:

### Subphase 30.1.1: Excel Home, Insert & Page Layout Ribbons (100% Working Controls)
- **Spec File:** `.agents/stages/stage-30/subphase-30.1.1.md`
/execute-subphase 30.1.1
Invoke @adaptive-ui-builder under GEC-v4.0.
1. Audit and wire **100% of the buttons** in Excel's Home, Insert, and Page Layout ribbons:
   - **Home Tab:** Cut/Copy/Paste, Font Family & Size, Bold/Italic/Underline, Borders (All, Outside, Thick Bottom, No Border), Fill Color, Font Color, Align Top/Middle/Bottom & Left/Center/Right, Wrap Text, Merge & Center, Number Formats (General, Number, Currency $, Accounting, Short Date, Percentage %, Fraction, Comma, Increase/Decrease Decimal), Conditional Formatting (Highlight >, Highlight <, Top 10%, Data Bars, Color Scales), Format as Table (applies alternating banded row styling to selected range), Insert/Delete/Format Rows & Columns (including AutoFit Row Height & Column Width), AutoSum, Fill Down/Right/Series, Clear (All / Formats / Contents), and Find & Select.
   - **Insert Tab:** PivotTable Summary (generates an automatic grouped summary table from selected data onto a new or current sheet), Table, Picture, Shapes, Column Chart, Line Chart, Pie/Doughnut Chart, Scatter Plot (draggable, resizable, live-updating SVG charts on the sheet canvas), Sparklines (renders mini inline SVG line/bar sparklines inside cells!), and Hyperlink / Comment.
   - **Page Layout Tab:** Fix the non-working buttons: Themes (applies color/font palette to grid & charts), Colors, Fonts, Margins (Normal, Narrow, Wide), Orientation (Portrait, Landscape â€” updates dashed print-boundary lines on grid), Print Area (Set Print Area, Clear Print Area), Scale to Fit, View Grid (toggles cell gridlines on/off!), Print Grid, View Headings (toggles A, B, C and 1, 2, 3 headers on/off!), and Print Headings.
2. Write automated tests verifying every control in Home, Insert, and Page Layout. Commit with [GEC-Verified].
- **Completion Gate:** `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 30.1.1`

### Subphase 30.1.2: Excel Formulas, Data, Review & View Ribbons (100% Working Controls)
- **Spec File:** `.agents/stages/stage-30/subphase-30.1.2.md`
/execute-subphase 30.1.2
Invoke @adaptive-ui-builder under GEC-v4.0.
1. Wire **100% of the buttons** in Excel's Formulas, Data, Review, and View ribbons:
   - **Formulas Tab:** Insert Function (fx) searchable wizard, AutoSum (Sum, Average, Count, Max, Min), Category Dropdowns that insert & evaluate functions (Financial: PMT,FV,PV,NPV,IRR, Logical: IF,AND,OR,NOT,IFERROR,IFS, Text: CONCAT,LEFT,RIGHT,MID,LEN,UPPER,LOWER,TRIM,SUBSTITUTE, Date & Time: TODAY,NOW,YEAR,MONTH,DAY,DATEDIF, Lookup & Ref: VLOOKUP,XLOOKUP,INDEX,MATCH, Math & Stat: ROUND,SUMIF,SUMIFS,COUNTIF,AVERAGEIF,STDEV,MEDIAN,RAND,RANDBETWEEN), Name Manager (define named ranges like TaxRate = 0.18 usable in formulas!), Show Formulas toggle, Trace Precedents / Dependents (highlights referenced cells), and Calculate Now (F9).
   - **Data Tab:** From CSV / JSON Import, Sort A â†’ Z, Sort Z â†’ A, Custom Multi-Column Sort, Filter (interactive dropdown filter on column headers), Clear Filter, Text to Columns, Flash Fill, Remove Duplicates, Data Validation (restrict cells to Dropdown List, Whole Number range, or Date!), Subtotals, and What-If Goal Seek solver!
   - **Review & View Tabs:** New Comment / Note (adds red corner indicator and hover tooltip on cell), Protect Sheet (locks cells from editing with optional password), Workbook Statistics, Normal / Page Break Preview, Formula Bar checkbox, Gridlines checkbox, Headings checkbox, Freeze Panes (Freeze Top Row, Freeze First Column, Unfreeze), and Zoom controls.
2. Verify all Formulas, Data, Review, and View actions with automated DOM tests. Commit with [GEC-Verified].
- **Completion Gate:** `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 30.1.2`

