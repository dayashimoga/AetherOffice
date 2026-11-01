# Sub-Phase 49.1.2 Prompt Specification: Page Split, Merge, Extract, Bates Numbering & Optical Character Recognition (OCR)

## Mission & Architectural Scope
Sub-Phase 49.1.2 implements commercial-grade PDF document assembly, page extraction, legal Bates numbering, and searchable OCR text layer injection for AetherPDF (`pdf-engine`, `render-core`, `unified-shell`).

### Architectural Invariants (GEC-v7.0):
1. `#![forbid(unsafe_code)]` strictly enforced across `pdf-engine`.
2. Zero panics, unhandled errors, or `.unwrap()` / `.expect()` in non-test library code.
3. Robust object ID re-indexing during document merge to prevent indirect reference collisions.
4. Clean ISO 32000-1 content stream injection for Bates numbering and invisible text OCR rendering mode (`3 Tr`).
5. Automated Gatekeeper: `tools/verify-contract.ps1` must exit `0`.

---

## Deliverables & Modules

### 1. `crates/pdf-engine/src/page_ops.rs`
- `PdfPageOps`:
  - `split_document(data: &[u8], split_indices: &[usize]) -> Result<Vec<Vec<u8>>, PdfError>`
  - `extract_pages(data: &[u8], page_indices: &[usize]) -> Result<Vec<u8>, PdfError>`
  - `merge_documents(docs_data: &[&[u8]]) -> Result<Vec<u8>, PdfError>`
  - `apply_bates_numbering(data: &[u8], config: &BatesNumberingConfig) -> Result<Vec<u8>, PdfError>`
  - `inject_ocr_layer(data: &[u8], page_index: usize, ocr_data: &OcrPageData) -> Result<Vec<u8>, PdfError>`
- Models:
  - `BatesNumberingConfig`: prefix, suffix, start_number, num_digits, position (`TopLeft`, `TopCenter`, `TopRight`, `BottomLeft`, `BottomCenter`, `BottomRight`), font_size, color_rgb.
  - `OcrPageData`, `OcrWord`, `OcrBoundingBox`.

### 2. Tauri IPC Commands & Signatures
- `pdf_split_document`
- `pdf_merge_documents`
- `pdf_extract_pages`
- `pdf_apply_bates_numbering`
- `pdf_inject_ocr_layer`

### 3. Frontend Presentation Shell Coordinator (`apps/unified-shell/src/state/PdfPageOpsCoordinator.ts`)
- Manages assembly operations, page range extraction, multi-file merging, Bates stamping, and OCR text overlay coordination.
- Export in `apps/unified-shell/src/state/index.ts`.

### 4. Integration & Regression Test Suites
- Rust integration tests in `crates/pdf-engine/tests/subphase_49_1_2_test.rs` (10 tests).
- Vitest tests in `apps/unified-shell/src/__tests__/pdf_page_ops.test.ts` (10 tests).
- Automated Gatekeeper `tools/verify-contract.ps1`.

