# Subphase 5.2.1: Incremental Update Writer & Digital Signature Preservation
- **Stage:** 5 | **Phase:** 5.2
- **Target Crate:** `pdf-engine`
- **Primary Agent Skill:** `@ecma376-architect`
- **REA Binary Analysis Required:** NO (ISO 32000-1 / ISO 32000-2 Clean-Room Standard)
- **Deliverable:** Non-destructive incremental update serializer, multi-revision history inspector, and digital signature ByteRange validator.
- **Validation Command:** `cargo test --package pdf-engine --test subphase_5_2_1_test`

---

## Architecture & Requirements (ISO 32000-1 Clause 7.5.6, 12.8)
1. **Incremental Update Serializer (`crates/pdf-engine/src/writer/`):**
   - Appends new and modified objects to original document bytes without altering previous content.
   - Generates compact incremental `xref` tables and trailer dictionaries containing `/Prev` pointers to earlier revisions.
   - Serializes PDF objects (`Null`, `Boolean`, `Integer`, `Real`, `Name`, `LiteralString`, `HexString`, `Array`, `Dictionary`, `Stream`, `Reference`) to standards-compliant bytes.
2. **Digital Signature Preservation & ByteRange Verification (`crates/pdf-engine/src/signature.rs`):**
   - Detection of signature dictionaries (`/Type /Sig`) in form fields (`/FT /Sig`).
   - Parsing and validation of `/ByteRange [offset1 length1 offset2 length2]`.
   - Verification of byte integrity: ensures incremental modifications never alter bytes within signed regions.
   - SHA-256 digest computation over the exact disjoint `/ByteRange` byte spans.
3. **Multi-Revision Document Inspection (`crates/pdf-engine/src/revision.rs`):**
   - Traverses chained `/Prev` pointers to enumerate all historic document revisions.
   - Extracts exact file bytes at any historical revision stage for rollback and signing audit.
4. **Clean-Room Invariants:**
   - Enforce `#[forbid(unsafe_code)]` in `crates/pdf-engine/src/lib.rs`.
   - Zero `.unwrap()`, `.expect()`, `panic!()`, `todo!()`, or `unimplemented!()` in production paths. Return typed `Result<T, PdfError>`.
