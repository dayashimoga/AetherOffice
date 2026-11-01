# Subphase 42.1.1: Instance-Scoped Document Session Manager

- **Stage:** 42 | **Phase:** 42.1
- **Target Subsystem:** `render-core` / `unified-shell`
- **Primary Agent Skill:** `@session-architect`
- **Deliverable:** Strongly-typed, instance-scoped `DocumentSessionRegistry` managing multiple concurrent document sessions without global singleton state collision.
- **Validation Command:** `powershell -ExecutionPolicy Bypass -File tools/verify-contract.ps1`

