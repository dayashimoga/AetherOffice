# Subphase 42.1.2: Modular Workspace Controller Extraction

- **Stage:** 42 | **Phase:** 42.1
- **Target Subsystem:** `render-core` / `unified-shell`
- **Primary Agent Skill:** `@workspace-controller-architect`
- **Deliverable:** Decompose monolithic `UnifiedShellApp` (`main.ts`, ~3,000 LOC) into modular controllers (< 600 LOC per module) with clean listener lifecycle management:
  1. `WorkspaceLayoutManager.ts` (manages container layouts, split-pane docking, responsive resizing)
  2. `PaneController.ts` (manages active/warm/cold viewport canvas mounting and event dispatch per pane)
  3. `FocusRouter.ts` (handles focus tracking, keyboard navigation and contextual ribbon updates)
- **Validation Command:** `powershell -ExecutionPolicy Bypass -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt

```text
/execute-subphase 42.1.2
Invoke @workspace-controller-architect under GEC-v7.0.
1. Extract modular workspace controllers into `apps/unified-shell/src/controllers/`:
   - `WorkspaceLayoutManager.ts`: container layouts, split-pane docking, responsive CSS variables, drawer/sidebar toggles.
   - `PaneController.ts`: viewport mounting, lifecycle state transitions (active, warm, cold), document rendering per pane.
   - `FocusRouter.ts`: tracking active focused pane, routing global shortcuts, syncing contextual ribbon tabs and actions.
2. Refactor `UnifiedShellApp` in `main.ts` to delegate to these controllers while preserving 100% of all public interfaces, properties, methods, event listeners, and data-testid DOM hooks.
3. Add unit test suite in `apps/unified-shell/src/__tests__/workspace_controllers.test.ts` verifying layout adjustments, focus routing, and pane management.
4. Run `powershell -ExecutionPolicy Bypass -File tools/verify-contract.ps1` to ensure all 77+ suites pass with zero regressions.
```

