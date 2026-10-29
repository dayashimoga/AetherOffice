# Subphase 13.1.2: Final 5-Binary Release Build & GitHub Actions Multi-Platform Verification
- **Stage:** 13 | **Phase:** 13.1
- **Target Subsystem:** `render-core`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **Deliverable:** Verified release builds of aether-hub, aether-writer, aether-calc, aether-slides, and aether-pdf pushed and packaged across CI/CD.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

`	ext
/execute-subphase 13.1.2
Execute final GEC-v2.0 Release Audit:
1. Update .github/workflows/build-all-platforms.yml so desktop builds package and upload all 5 executables (ether-hub, ether-writer, ether-calc, ether-slides, ether-pdf) alongside the unified installer bundle.
2. Run powershell -ExecutionPolicy Bypass -File .\tools\verify-contract.ps1 to verify all Rust crates, Clippy, Formatting, Anti-Placeholder scans, and Frontend E2E UI tests pass 100%.
3. Commit, tag 2.0.0, and push to origin.
`
