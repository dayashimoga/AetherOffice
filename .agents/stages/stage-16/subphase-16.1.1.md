# Subphase 16.1.1: WebAssembly Component Model Runtime & WIT Contract
- **Stage:** 16 | **Phase:** 16.1
- **Target Subsystem:** `plugin-runtime`
- **Primary Agent Skill:** `@wasm-architect`
- **Deliverable:** WebAssembly Component Model loader, host interface definition (WIT) for document inspection and mutation, plugin sandbox lifecycle.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 16.1.1
Invoke @wasm-architect under GEC-v2.0:
1. Create new crate `crates/plugin-runtime` with strict `#![forbid(unsafe_code)]` and zero panic macros.
2. Define WebAssembly Interface Types (`wit/aetheroffice-plugin.wit`):
   - Document lifecycle hooks (`on-init`, `on-document-opened`, `on-before-save`, `on-selection-changed`).
   - Query APIs (inspect active selection, cell values, paragraph runs, slide shapes).
   - Command dispatch APIs (insert text, execute ribbon command, set cell formula).
3. Implement `WasmPluginHost` with memory isolation and crash resilience.
4. Add unit tests loading sample Wasm plugin binaries and asserting bidirectional event dispatch.
5. Run `powershell -File tools/verify-contract.ps1` and ensure all gates pass. Commit with [GEC-Verified].
```
