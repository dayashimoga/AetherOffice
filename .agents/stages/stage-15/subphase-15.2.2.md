# Subphase 15.2.2: User-Defined Functions (UDFs) & Developer Macro Ribbon
- **Stage:** 15 | **Phase:** 15.2
- **Target Subsystem:** `unified-shell` & `excel-engine`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Execution of custom VBA functions in formula calculations, Developer Ribbon tab with Macro Record/Run dialogs and Code Inspector.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 15.2.2
Invoke @adaptive-ui-builder under GEC-v2.0:
1. In `excel-engine`, integrate `macro-engine` User-Defined Functions (UDFs):
   - When a formula calls a non-built-in function name (e.g. `=CALCULATETAX(A1, 0.2)`), invoke the registered sandboxed UDF handler.
2. In `apps/unified-shell/src/components/ribbon/DeveloperRibbon.ts`:
   - Add Developer tab to Ribbon: Macros button, Visual Basic Editor launcher, Record Macro toggle, Add-ins button.
3. In `apps/unified-shell/src/components/MacroEditorModal.ts`:
   - Interactive code editor dialog with syntax highlighting, line numbers, Run, Step-Into, and Security Permission toggles.
4. Add automated Vitest integration tests in `apps/unified-shell/src/__tests__/macro_developer.test.ts`.
5. Run `powershell -File tools/verify-contract.ps1` and ensure all gates pass. Commit with [GEC-Verified].
```

