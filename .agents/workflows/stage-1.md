---
description: Automated execution workflow for Stage 1: Foundation, ECMA-376 Open Packaging Conventions (OPC) & Differential Test Harness
---
# Workflow: Execute Stage 1 (/stage-1)
## Objective: Stage 1: Foundation, ECMA-376 Open Packaging Conventions (OPC) & Differential Test Harness

When invoked via `/stage-1`, execute the following subphases sequentially. Do NOT advance to the next subphase until `tools/verify-contract.ps1` exits with code 0 and `specs/stage-tracker.json` is updated:

### Subphase 1.1.1: Workspace, GitHub Repo & CI/CD Initialization
- **Prompt Spec File:** `.agents/stages/stage-1/subphase-1.1.1.md`

**Instructions:**
```text
/execute-subphase 1.1.1
Invoke @ecma376-architect under strict Global Engineering Contract (GEC-v1.0) compliance.
1. Verify the Cargo workspace structure across `crates/ooxml-opc`, `crates/word-engine`, `crates/excel-engine`, `crates/ppt-engine`, and `crates/render-core`.
2. Define the shared `EngineError` enum using `thiserror` in `crates/ooxml-opc/src/error.rs` with zero `.unwrap()` or `.expect()` calls.
3. Verify `.github/workflows/build-all-platforms.yml` syntax and initialize the GitHub remote repository using `gh repo create` if not yet linked.
4. Run `powershell -ExecutionPolicy Bypass -File tools/verify-contract.ps1` and commit with `feat(subphase-1.1.1): initialize workspace error model and CI/CD [GEC-Verified]`.
```

**Post-Subphase Gate:**
Run `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 1.1.1` to verify contract compliance and advance the state tracker.

### Subphase 1.1.2: COM Ground-Truth Differential Harness
- **Prompt Spec File:** `.agents/stages/stage-1/subphase-1.1.2.md`

**Instructions:**
```text
/execute-subphase 1.1.2
Invoke @ecma376-architect under GEC-v1.0 compliance.
1. Enhance `tools/com-ground-truth/generate_truth.py` to support headless generation of baseline `.docx`, `.xlsx`, and `.pptx` fixtures alongside calculated ground-truth JSON files in `specs/evidence/{word,excel,ppt}/`.
2. Create a fallback deterministic synthetic fixture generator in Rust (`crates/ooxml-opc/src/fixtures.rs`) so CI/CD runners without Microsoft Office installed can still generate and validate all ECMA-376 test packages.
3. Write 25 baseline vector tests in `crates/ooxml-opc/tests/subphase_1_1_2_test.rs`.
4. Run `tools/verify-contract.ps1` and commit with `feat(subphase-1.1.2): complete COM ground-truth and synthetic fixture harness [GEC-Verified]`.
```

**Post-Subphase Gate:**
Run `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 1.1.2` to verify contract compliance and advance the state tracker.

### Subphase 1.2.1: Streaming ZIP & [Content_Types].xml / .rels Parser
- **Prompt Spec File:** `.agents/stages/stage-1/subphase-1.2.1.md`

**Instructions:**
```text
/execute-subphase 1.2.1
Invoke @ecma376-architect under GEC-v1.0 compliance.
1. In `crates/ooxml-opc`, implement the ECMA-376 Part 2 (ISO/IEC 29500-2) Open Packaging Conventions engine:
   - Streaming parser and serializer for `[Content_Types].xml` (`Types`, `Default`, `Override`).
   - Hierarchical relationship graph resolver for `_rels/.rels` and part-level `_rels/*.rels` (`Id`, `Type`, `Target`, `TargetMode`).
   - Lazy ZIP entry stream reader so a 100MB `.xlsx` or `.pptx` package parses relationships in `< 15 ms` without decompressing all streams upfront.
2. Document the OPC state machine in `specs/evidence/word/1.2.1.md` and write >= 25 edge-case test vectors in `specs/evidence/word/1.2.1_vectors.json`.
3. Validate with `crates/ooxml-opc/tests/subphase_1_2_1_test.rs` and commit with `[GEC-Verified]`.
```

**Post-Subphase Gate:**
Run `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 1.2.1` to verify contract compliance and advance the state tracker.

### Subphase 1.2.2: UnparsedPartStore & Zero-Loss Round-Trip Engine
- **Prompt Spec File:** `.agents/stages/stage-1/subphase-1.2.2.md`

**Instructions:**
```text
/execute-subphase 1.2.2
Invoke @ecma376-architect under GEC-v1.0 Article II (Zero-Loss Round-Trip Fidelity).
1. In `crates/ooxml-opc/src/unparsed.rs`, implement `UnparsedPartStore` and `RawXmlNodeBuffer` to capture:
   - Unmodeled ZIP parts (`vbaProject.bin`, digital signatures `_xmlsignatures/`, ActiveX controls, embedded OLE binaries, customXML).
   - Unknown XML elements and `mc:AlternateContent` / `mc:Ignorable` markup compatibility blocks inside modeled XML streams.
2. Prove deterministic round-trip preservation: opening any `.docx`, `.xlsx`, or `.pptx` and saving it back must retain 100% of relationships, binary parts, and unknown XML attributes.
3. Write >= 25 round-trip test vectors in `crates/ooxml-opc/tests/subphase_1_2_2_test.rs` and commit with `[GEC-Verified]`.
```

**Post-Subphase Gate:**
Run `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 1.2.2` to verify contract compliance and advance the state tracker.

