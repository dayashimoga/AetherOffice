# Subphase 5.2.2: Interactive AcroForms Engine, Annotations & Ink Path Layers
- **Stage:** 5 | **Phase:** 5.2
- **Target Crate:** `pdf-engine`
- **Primary Agent Skill:** `@ecma376-architect`
- **REA Binary Analysis Required:** NO (ISO 32000-1 / ISO 32000-2 Clean-Room Standard)
- **Deliverable:** Complete interactive AcroForms engine (Text, Checkbox, Radio, Dropdown), annotation model (Text, Highlight, FreeText, Shapes), and freehand stylus Ink path layer compiler.
- **Validation Command:** `cargo test --package pdf-engine --test subphase_5_2_2_test`

---

## Architecture & Requirements (ISO 32000-1 Clause 12.5, 12.7)
1. **Interactive AcroForms Engine (`crates/pdf-engine/src/acroform.rs`):**
   - Document AcroForm catalog traversal (`/Root -> /AcroForm -> /Fields`).
   - Field types: Text fields (`/Tx`), Buttons/Checkboxes/Radios (`/Btn`), and Choice dropdown/list boxes (`/Ch`).
   - Fully qualified hierarchical field name resolution (dot notation).
   - Form field value mutation and automatic appearance stream (`/AP /N`) regeneration.
2. **Annotation Models & CRUD (`crates/pdf-engine/src/annotation.rs`):**
   - Page `/Annots` array parsing and indirect reference resolution.
   - Subtypes: `/Text` (Sticky notes), `/Highlight` (with QuadPoints), `/FreeText`, `/Square`, `/Circle`, `/Line`, and `/Widget`.
   - Flags handling (Print, Hidden, ReadOnly, Locked).
3. **Stylus Freehand Ink Path Layers (`crates/pdf-engine/src/ink.rs`):**
   - `/Ink` annotation with coordinate stroke paths (`/InkList`).
   - Vector path appearance stream compiler generating PDF path drawing operators (`w`, `RG`, `m`, `l`, `c`, `S`).
4. **Incremental Persistence Bridge:**
   - Form fill changes and newly drawn ink annotations serialized seamlessly via `PdfIncrementalWriter`.
5. **Clean-Room Invariants:**
   - `#[forbid(unsafe_code)]` enforced in `crates/pdf-engine/src/lib.rs`.
   - Zero `.unwrap()`, `.expect()`, `panic!()`, `todo!()`, or `unimplemented!()` in production paths. Return typed `Result<T, PdfError>`.
