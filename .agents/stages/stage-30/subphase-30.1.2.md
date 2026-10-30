# Subphase 30.1.2: Excel Formulas, Data, Review & View Ribbons (100% Working Controls)
- **Stage:** 30 | **Phase:** 30.1
- **Target Subsystem:** `excel-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Complete working Excel Formulas library, Data tools (Sort, Filter, Validation, Goal Seek), Review comments/protection, and View freeze panes.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 30.1.2
Invoke @adaptive-ui-builder under GEC-v4.0.
1. Wire **100% of the buttons** in Excel's Formulas, Data, Review, and View ribbons:
   - **Formulas Tab:** Insert Function (fx) searchable wizard, AutoSum (Sum, Average, Count, Max, Min), Category Dropdowns that insert & evaluate functions (Financial: PMT,FV,PV,NPV,IRR, Logical: IF,AND,OR,NOT,IFERROR,IFS, Text: CONCAT,LEFT,RIGHT,MID,LEN,UPPER,LOWER,TRIM,SUBSTITUTE, Date & Time: TODAY,NOW,YEAR,MONTH,DAY,DATEDIF, Lookup & Ref: VLOOKUP,XLOOKUP,INDEX,MATCH, Math & Stat: ROUND,SUMIF,SUMIFS,COUNTIF,AVERAGEIF,STDEV,MEDIAN,RAND,RANDBETWEEN), Name Manager (define named ranges like TaxRate = 0.18 usable in formulas!), Show Formulas toggle, Trace Precedents / Dependents (highlights referenced cells), and Calculate Now (F9).
   - **Data Tab:** From CSV / JSON Import, Sort A â†’ Z, Sort Z â†’ A, Custom Multi-Column Sort, Filter (interactive dropdown filter on column headers), Clear Filter, Text to Columns, Flash Fill, Remove Duplicates, Data Validation (restrict cells to Dropdown List, Whole Number range, or Date!), Subtotals, and What-If Goal Seek solver!
   - **Review & View Tabs:** New Comment / Note (adds red corner indicator and hover tooltip on cell), Protect Sheet (locks cells from editing with optional password), Workbook Statistics, Normal / Page Break Preview, Formula Bar checkbox, Gridlines checkbox, Headings checkbox, Freeze Panes (Freeze Top Row, Freeze First Column, Unfreeze), and Zoom controls.
2. Verify all Formulas, Data, Review, and View actions with automated DOM tests. Commit with [GEC-Verified].
```
