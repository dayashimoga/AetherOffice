# Subphase 15.2.1: WebAssembly Isolation Sandbox & Capability Gatekeeper
- **Stage:** 15 | **Phase:** 15.2
- **Target Subsystem:** `macro-engine`
- **Primary Agent Skill:** `@security-engineer`
- **Deliverable:** Strict memory-isolated Wasm bytecode execution, permission prompt gate for external capabilities, zero host OS escape.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 15.2.1
Invoke @security-engineer under GEC-v2.0:
1. In `crates/macro-engine`, implement `MacroSandbox`:
   - Isolated bytecode VM with strict execution step limit (gas meter) to prevent infinite loops.
   - Capability-based permission system: File I/O, Network, and System Command access require explicit user authorization.
   - Static analysis scanner detecting suspicious API invocations (`Shell`, `CreateObject("WScript.Shell")`, `Environ`).
2. Implement sandbox security audit logging and automated exploit isolation tests.
3. Run `powershell -File tools/verify-contract.ps1` and ensure all gates pass. Commit with [GEC-Verified].
```

