# Subphase 42.2.1: Unified Mixed-Format Tab Bar & Project Explorer Sidebar

- **Stage:** 42 | **Phase:** 42.2
- **Target Subsystem:** `render-core` / `unified-shell`
- **Primary Agent Skill:** `@workspace-controller-architect`
- **Deliverable:**
  1. Upgrade `TabBar.ts` to support unified mixed-format tabs side-by-side with color-coded format badges (`W`, `X`, `P`, `PDF`, `N`) and toggleable filtering mode (`all` vs `app-scoped`).
  2. Implement `ProjectExplorerSidebar.ts` in `apps/unified-shell/src/components/ProjectExplorerSidebar.ts` for mounting directory workspaces and displaying office documents in a collapsible tree.
  3. Integrate into `UnifiedShellApp` (`main.ts`) and `AppRail.ts`.
  4. Comprehensive automated test suite in `apps/unified-shell/src/__tests__/unified_mixed_tab_project_explorer.test.ts`.
- **Validation Command:** `powershell -ExecutionPolicy Bypass -File tools/verify-contract.ps1`

