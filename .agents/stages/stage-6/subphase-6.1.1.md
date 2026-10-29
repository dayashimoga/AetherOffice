# Subphase 6.1.1: Transactional Command Bus with Global Undo/Redo & Clipboard
- **Target Subsystem:** `render-core`
- **Skill:** `@adaptive-ui-builder`
- **REA Binary Analysis:** NO (ECMA-376 / ISO Standard)

## Instructions for Antigravity Agent
1. Implement the subsystem in `crates/render-core` and `apps/` according to GEC-v1.0.
2. Ensure every UI button, input, or formula field dispatches through the Command Bus.
3. Validate with unit tests in `crates/render-core/tests/subphase_6_1_1_test.rs`.
4. Run `powershell -ExecutionPolicy Bypass -File tools/verify-contract.ps1` and commit with `[GEC-Verified]`.
