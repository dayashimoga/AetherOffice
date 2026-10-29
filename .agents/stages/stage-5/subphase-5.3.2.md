# Subphase 5.3.2: In-Place PDF Text/Image Editor, Page Organiser & Form Filler UI
- **Target Subsystem:** `render-core`
- **Skill:** `@adaptive-ui-builder`
- **REA Binary Analysis:** NO (ECMA-376 / ISO Standard)

## Instructions for Antigravity Agent
1. Implement the subsystem in `crates/render-core` and `apps/` according to GEC-v1.0.
2. Ensure every UI button, input, or formula field dispatches through the Command Bus.
3. Validate with unit tests in `crates/render-core/tests/subphase_5_3_2_test.rs`.
4. Run `powershell -ExecutionPolicy Bypass -File tools/verify-contract.ps1` and commit with `[GEC-Verified]`.
