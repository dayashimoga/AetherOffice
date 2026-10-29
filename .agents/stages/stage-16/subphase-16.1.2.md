# Subphase 16.1.2: Dynamic UI Ribbon & Sidebar Injection API
- **Stage:** 16 | **Phase:** 16.1
- **Target Subsystem:** `unified-shell`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Plugin manifest parser, third-party Ribbon tab registration, custom iframe/web panel sidebars, event lifecycle hooks.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 16.1.2
Invoke @adaptive-ui-builder under GEC-v2.0:
1. In `apps/unified-shell/src/plugins/pluginHost.ts`:
   - Parse plugin manifests (`plugin.json`) declaring custom tabs, buttons, sidebars, and keyboard shortcuts.
2. In `ContextualRibbonRouter.ts`:
   - Support dynamic injection of third-party custom ribbon tabs and groups alongside standard tabs.
3. In `UnifiedShellApp`:
   - Allow plugins to register custom sidebar panes in `#sidebar-container` with isolated webview/iframe sandboxing.
4. Add automated Vitest integration tests in `apps/unified-shell/src/__tests__/plugin_ui_injection.test.ts`.
5. Run `powershell -File tools/verify-contract.ps1` and ensure all gates pass. Commit with [GEC-Verified].
```
