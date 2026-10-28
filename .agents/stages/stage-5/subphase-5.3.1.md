# Subphase 5.3.1: End-to-End 5-Platform Release Audit & GitHub Artifact Publication
- **Stage:** 5 | **Phase:** 5.3
- **Target Crate:** `render-core`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **REA Binary Analysis Required:** NO (Pure ECMA-376 / Clean-Room)
- **Deliverable:** Verified Windows (.msi/.exe), Linux (.AppImage/.deb), macOS (.dmg), Android (.apk), and iOS (.ipa) release artifacts.
- **Validation Command:** `powershell -File tools/verify-contract.ps1`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 5.3.1
Execute final Stage 5 Release Gatekeeping under GEC-v1.0:
1. Run `powershell -ExecutionPolicy Bypass -File .\removeRea_bootstrap.ps1 -KeepGlobalNpmPackage` to verify zero proprietary binaries or temp files exist anywhere in the repository.
2. Run `powershell -ExecutionPolicy Bypass -File .\tools\verify-contract.ps1` and verify 100% pass rate across all 22 subphase test suites (`ooxml-opc`, `excel-engine`, `word-engine`, `ppt-engine`, `render-core`) and `apps/unified-shell` frontend build.
3. Commit final release metadata, tag `v1.0.0`, push to GitHub to trigger `.github/workflows/build-all-platforms.yml`, and verify that all 5 platform artifacts (`Windows-x64`, `Linux-x64`, `macOS-Universal`, `Android-Universal-APK`, `iOS-IPA`) build and upload cleanly.
```
