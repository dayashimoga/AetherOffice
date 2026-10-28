# Subphase 1.2.1: Streaming ZIP & [Content_Types].xml / .rels Parser
- **Stage:** 1 | **Phase:** 1.2
- **Target Crate:** `ooxml-opc`
- **Primary Agent Skill:** `@ecma376-architect`
- **REA Binary Analysis Required:** NO (Pure ECMA-376 / Clean-Room)
- **Deliverable:** Zero-copy ECMA-376 Part 2 Open Packaging Conventions (OPC) reader/writer in crates/ooxml-opc.
- **Validation Command:** `cargo test --package ooxml-opc --test subphase_1_2_1_test`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

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
