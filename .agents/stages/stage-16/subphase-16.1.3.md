# Subphase 16.1.3: Custom Calculation UDF Registry & Plugin Marketplace UI
- **Stage:** 16 | **Phase:** 16.1
- **Target Subsystem:** `unified-shell` & `excel-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Dynamic calculation formula functions loaded from plugins, Plugin Manager dialog with Enable/Disable/Install controls.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 16.1.3
Invoke @adaptive-ui-builder under GEC-v2.0:
1. In `excel-engine`:
   - Connect dynamic plugin UDF registry to the formula evaluation engine so plugin functions calculate natively.
2. In `apps/unified-shell/src/components/PluginManagerModal.ts`:
   - Modal dialog listing installed plugins, permissions granted, author, version, toggle switches, and "Add Plugin" file picker.
3. In Backstage Menu:
   - Add "Add-ins & Extensions" entrypoint opening Plugin Manager.
4. Add automated Vitest integration tests in `apps/unified-shell/src/__tests__/plugin_marketplace.test.ts`.
5. Run `powershell -File tools/verify-contract.ps1` and ensure all gates pass. Commit with [GEC-Verified].
```
