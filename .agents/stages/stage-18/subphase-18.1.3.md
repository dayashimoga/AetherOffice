# Subphase 18.1.3: Interactive PivotTable & PivotChart Studio UI
- **Stage:** 18 | **Phase:** 18.1
- **Target Subsystem:** `unified-shell`
- **Primary Agent Skill:** `@frontend-architect`
- **Deliverable:** PivotTable Field List sidebar with drag-and-drop quadrant areas (Filters, Columns, Rows, Values), real-time slicers, dynamic filtering, and interactive PivotChart synchronization.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 18.1.3
Invoke @frontend-architect under GEC-v2.0:
1. Implement `PivotFieldListSidebar.ts` in `unified-shell` with drag-and-drop zones for Rows, Columns, Values, and Filters.
2. Implement dynamic pivot layout renderer writing summary tables directly into the active Calc sheet grid.
3. Add interactive Slicers UI component for visual point-and-click data segment filtering.
4. Wire PivotTable changes to automatically update linked PivotCharts (Column, Bar, Line, Pie).
5. Add Calc Insert Ribbon command "PivotTable" and "PivotChart" opening the Pivot Studio.
6. Add comprehensive Vitest integration tests for field drag-and-drop, aggregation changes, and slicer state updates.
7. Run `powershell -File tools/verify-contract.ps1` and ensure all gates pass. Commit with [GEC-Verified].
```

