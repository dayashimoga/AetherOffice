# Subphase 30.1.2: Excel Complete Ribbon Overhaul: Home, Insert, Page Layout, Formulas, Data, Review, View
- **Stage:** 30 | **Phase:** 30.1
- **Target Subsystem:** `excel-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** 100% working Excel ribbon options across all 7 tabs with zero overflow and automated verification.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
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
```
