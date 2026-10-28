# Subphase 1.1.1: Workspace, GitHub Repo & CI/CD Initialization
- **Stage:** 1 | **Phase:** 1.1
- **Target Crate:** `ooxml-opc`
- **Primary Agent Skill:** `@ecma376-architect`
- **REA Binary Analysis Required:** NO (Pure ECMA-376 / Clean-Room)
- **Deliverable:** Verified Cargo workspace, public/private GitHub repository, and 5-platform GitHub Actions workflow.
- **Validation Command:** `cargo check --workspace && powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 1.1.1
Invoke @ecma376-architect under strict Global Engineering Contract (GEC-v1.0) compliance.
1. Verify the Cargo workspace structure across `crates/ooxml-opc`, `crates/word-engine`, `crates/excel-engine`, `crates/ppt-engine`, and `crates/render-core`.
2. Define the shared `EngineError` enum using `thiserror` in `crates/ooxml-opc/src/error.rs` with zero `.unwrap()` or `.expect()` calls.
3. Verify `.github/workflows/build-all-platforms.yml` syntax and initialize the GitHub remote repository using `gh repo create` if not yet linked.
4. Run `powershell -ExecutionPolicy Bypass -File tools/verify-contract.ps1` and commit with `feat(subphase-1.1.1): initialize workspace error model and CI/CD [GEC-Verified]`.
```
