# Subphase 10.2.1: Calc Home, Formulas & Data Ribbons: Number Formats, Function Wizard & Sort/Filter
- **Stage:** 10 | **Phase:** 10.2
- **Target Subsystem:** `excel-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Working Number Formatting ($, %, Comma, Decimals), Cell Borders/Fill, AutoSum, Function Library inserter, Sort A-Z/Z-A, Filter, and Flash Fill.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

`	ext
/execute-subphase 10.2.1
Invoke @adaptive-ui-builder under GEC-v2.0.
1. Wire every button across the Home, Formulas, and Data ribbons in CalcRibbon.tsx:
   - **Home Tab:** Font/Size/Bold/Italic, Cell Background Color, Text Color, Borders (All Borders, Thick Box Border, Bottom Border), Alignment, Wrap Text, Merge & Center, and **Number Format** (General, Currency $, Accounting, Percentage %, Comma, Date, Increase/Decrease Decimal).
   - **Formulas Tab:** Replace the dead placeholder text with working buttons: **Insert Function (x modal with search across all 450+ functions)**, **AutoSum (Σ Sum, Average, Count, Max, Min)**, Financial dropdown, Logical dropdown, Text dropdown, Lookup & Reference dropdown, Name Manager, and Show Formulas toggle.
   - **Data Tab:** Make Sort A-Z, Sort Z-A, Filter (adds dropdown arrows to header cells to filter rows by value), Flash Fill, Remove Duplicates, and Text to Columns 100% functional on the selected grid range!
2. Write automated tests verifying Sort A-Z, AutoSum, and Currency formatting. Commit with [GEC-Verified].
`
