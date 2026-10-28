# Subphase 1.2.2: UnparsedPartStore & Zero-Loss Round-Trip Engine
- **Stage:** 1 | **Phase:** 1.2
- **Target Crate:** `ooxml-opc`
- **Primary Agent Skill:** `@ecma376-architect`
- **REA Binary Analysis Required:** NO (Pure ECMA-376 / Clean-Room)
- **Deliverable:** 100% lossless round-trip preservation of unknown XML nodes, macros (vbaProject.bin), and custom parts.
- **Validation Command:** `cargo test --package ooxml-opc --test subphase_1_2_2_test`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 1.2.2
Invoke @ecma376-architect under GEC-v1.0 Article II (Zero-Loss Round-Trip Fidelity).
1. In `crates/ooxml-opc/src/unparsed.rs`, implement `UnparsedPartStore` and `RawXmlNodeBuffer` to capture:
   - Unmodeled ZIP parts (`vbaProject.bin`, digital signatures `_xmlsignatures/`, ActiveX controls, embedded OLE binaries, customXML).
   - Unknown XML elements and `mc:AlternateContent` / `mc:Ignorable` markup compatibility blocks inside modeled XML streams.
2. Prove deterministic round-trip preservation: opening any `.docx`, `.xlsx`, or `.pptx` and saving it back must retain 100% of relationships, binary parts, and unknown XML attributes.
3. Write >= 25 round-trip test vectors in `crates/ooxml-opc/tests/subphase_1_2_2_test.rs` and commit with `[GEC-Verified]`.
```
