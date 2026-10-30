# Subphase 23.1.1: Blank Virtualized Excel Grid, Instant Typing, Formula Engine & Multi-Sheet Bar
- **Stage:** 23 | **Phase:** 23.1
- **Target Subsystem:** `excel-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** 100% blank spreadsheet grid with direct cell typing, working Formula Bar (=SUM, =AVERAGE, etc.), and + New Sheet tabs.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 23.1.1
Invoke @adaptive-ui-builder under GEC-v3.0.
1. Rebuild the Excel (Calc) workspace (pps/unified-shell/src/apps/calc/) so it opens to a **100% blank, editable grid** (Book1.xlsx -> Sheet1 with zero fake numbers):
   - **Lightweight Virtualized Grid:** Render a smooth scrollable grid where clicking any cell selects it, and **typing any key immediately starts editing that cell** (or double-clicking / pressing F2).
   - **Navigation Keys:** Enter commits and moves down, Tab commits and moves right, Arrow keys move selection when not editing, Delete/Backspace clears selected cell(s), and Escape cancels editing.
   - **Live Formula Evaluator:** Typing =SUM(A1:B5), =AVERAGE(...), =MIN(...), =MAX(...), =COUNT(...), =IF(A1>10, "Yes", "No"), =VLOOKUP(...), or math expressions (=A1*B1+100) evaluates immediately and updates dependent cells automatically when source cells change.
   - **Working Multi-Sheet Management:** Add a bottom Sheet Tab Bar (Sheet1, Sheet2, ...) with a working **+ button to create new sheets**, double-click to rename a sheet, right-click/button to delete a sheet, and sync with the left Workbook Sheets list.
2. Write automated DOM tests verifying blank boot, typing numbers into A1 and A2, evaluating =SUM(A1:A2) in A3, and clicking + to create Sheet2. Commit with [GEC-Verified].
```
