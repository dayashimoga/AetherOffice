# Subphase 30.1.1: Excel Home, Insert & Page Layout Ribbons (100% Working Controls)
- **Stage:** 30 | **Phase:** 30.1
- **Target Subsystem:** `excel-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Complete working Excel Home, Insert (Tables, Pivot Summary, Charts, Sparklines), and Page Layout ribbons.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 30.1.1
Invoke @adaptive-ui-builder under GEC-v4.0.
1. Audit and wire **100% of the buttons** in Excel's Home, Insert, and Page Layout ribbons:
   - **Home Tab:** Cut/Copy/Paste, Font Family & Size, Bold/Italic/Underline, Borders (All, Outside, Thick Bottom, No Border), Fill Color, Font Color, Align Top/Middle/Bottom & Left/Center/Right, Wrap Text, Merge & Center, Number Formats (General, Number, Currency $, Accounting, Short Date, Percentage %, Fraction, Comma, Increase/Decrease Decimal), Conditional Formatting (Highlight >, Highlight <, Top 10%, Data Bars, Color Scales), Format as Table (applies alternating banded row styling to selected range), Insert/Delete/Format Rows & Columns (including AutoFit Row Height & Column Width), AutoSum, Fill Down/Right/Series, Clear (All / Formats / Contents), and Find & Select.
   - **Insert Tab:** PivotTable Summary (generates an automatic grouped summary table from selected data onto a new or current sheet), Table, Picture, Shapes, Column Chart, Line Chart, Pie/Doughnut Chart, Scatter Plot (draggable, resizable, live-updating SVG charts on the sheet canvas), Sparklines (renders mini inline SVG line/bar sparklines inside cells!), and Hyperlink / Comment.
   - **Page Layout Tab:** Fix the non-working buttons: Themes (applies color/font palette to grid & charts), Colors, Fonts, Margins (Normal, Narrow, Wide), Orientation (Portrait, Landscape â€” updates dashed print-boundary lines on grid), Print Area (Set Print Area, Clear Print Area), Scale to Fit, View Grid (toggles cell gridlines on/off!), Print Grid, View Headings (toggles A, B, C and 1, 2, 3 headers on/off!), and Print Headings.
2. Write automated tests verifying every control in Home, Insert, and Page Layout. Commit with [GEC-Verified].
```
