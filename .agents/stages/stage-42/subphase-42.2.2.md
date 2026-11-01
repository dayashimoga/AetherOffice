# Subphase 42.2.2: Dockable 2-Way / 4-Way Split-Pane Editor Grid

- **Stage:** 42 | **Phase:** 42.2
- **Target Subsystem:** `render-core` / `unified-shell`
- **Primary Agent Skill:** `@workspace-controller-architect`
- **Deliverable:**
  1. Implement `SplitPaneGrid.ts` supporting 1-way (single), 2-way horizontal (side-by-side), 2-way vertical (top-bottom), 3-way, and 4-way (quad 2x2) dockable pane grid layouts.
  2. Integrate into `WorkspaceLayoutManager.ts` and `UnifiedShellApp` (`main.ts`).
  3. Support independent active documents per pane slot with focus tracking and contextual ribbon synchronization when focus shifts between panes.
  4. Comprehensive automated test suite in `apps/unified-shell/src/__tests__/dockable_split_pane_grid.test.ts`.
- **Validation Command:** `powershell -ExecutionPolicy Bypass -File tools/verify-contract.ps1`

