# Subphase 1.1.2: COM Ground-Truth Differential Harness
- **Stage:** 1 | **Phase:** 1.1
- **Target Crate:** `ooxml-opc`
- **Primary Agent Skill:** `@ecma376-architect`
- **REA Binary Analysis Required:** NO (Pure ECMA-376 / Clean-Room)
- **Deliverable:** Automated Python COM harness generating baseline JSON test vectors from local Office installation.
- **Validation Command:** `cargo test --package ooxml-opc`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 1.1.2
Invoke @ecma376-architect under GEC-v1.0 compliance.
1. Enhance `tools/com-ground-truth/generate_truth.py` to support headless generation of baseline `.docx`, `.xlsx`, and `.pptx` fixtures alongside calculated ground-truth JSON files in `specs/evidence/{word,excel,ppt}/`.
2. Create a fallback deterministic synthetic fixture generator in Rust (`crates/ooxml-opc/src/fixtures.rs`) so CI/CD runners without Microsoft Office installed can still generate and validate all ECMA-376 test packages.
3. Write 25 baseline vector tests in `crates/ooxml-opc/tests/subphase_1_1_2_test.rs`.
4. Run `tools/verify-contract.ps1` and commit with `feat(subphase-1.1.2): complete COM ground-truth and synthetic fixture harness [GEC-Verified]`.
```
