# Sub-Phase 49.2.1: Side-by-Side PDF Visual & Text Diff Engine

## Focus
`pdf-engine`, `render-core`, `unified-shell`

## Goals
1. Implement pure safe-Rust PDF comparison engine in `pdf-engine::diff`:
   - Side-by-side comparison of Document A (original) and Document B (revised).
   - Text diffing using word-level alignment, LCS / Myers diff, and spatial bounding box extraction.
   - Classification of diffs: `Inserted`, `Deleted`, `Modified`, `Moved`.
   - Structural diffing: page count delta, MediaBox dimension changes, orientation changes.
   - Visual overlay generator producing SVG highlight bounding boxes (red/green/amber) mapped precisely to page coordinates.
2. Register Tauri IPC commands:
   - `pdf_compare_documents(doc_a_bytes: Vec<u8>, doc_b_bytes: Vec<u8>, options_json: Option<String>) -> Result<PdfDiffResultDto, String>`
   - `pdf_generate_diff_overlay_svg(doc_bytes: Vec<u8>, page_index: usize, diff_items_json: String, is_revised: bool) -> Result<String, String>`
3. Build presentation shell coordinator:
   - `PdfDiffCoordinator.ts`:
     - Dual-document state management (`docA`, `docB`).
     - Synchronized split-pane viewport scrolling.
     - Diff navigation (jump to next / previous diff).
     - Diff filtering (insertions, deletions, modifications).
     - Reactive state distribution to UI split views.
4. Comprehensive test coverage:
   - 10 Rust unit/integration tests in `crates/pdf-engine/tests/subphase_49_2_1_test.rs`.
   - 10 Vitest tests in `apps/unified-shell/src/__tests__/pdf_diff_coordinator.test.ts`.
5. Pass `tools/verify-contract.ps1` with exit code 0.

