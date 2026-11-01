# Sub-Phase 48.2.2 Specification: OpenDocument Presentation (.odp) Support & High-DPI PDF/SVG Deck Export

## Objective
Establish clean-room OASIS OpenDocument Presentation (`.odp`) codec support and high-DPI vector slide deck export (standalone SVG and multi-page ISO 32000-2 PDF) within `ppt-engine`, `render-core`, and `unified-shell`.

## Architecture & Deliverables
1. **`ppt-engine` (ODP Codec & Deck Exporter)**:
   - `crates/ppt-engine/src/odp.rs`:
     - Clean-room parser & serializer for OASIS OpenDocument Presentation v1.3 (`mimetype`, `content.xml`, `meta.xml`, `styles.xml`).
     - Maps `office:presentation` and `draw:page` to internal PresentationML `Slide` and `Shape` models.
     - Preserves geometry, transforms, colors, and text hierarchies.
   - `crates/ppt-engine/src/deck_export.rs`:
     - `SlideSvgExporter`: Renders slide shapes, text runs, and fills into high-fidelity SVG 1.1/2.0 markup strings with proper viewBox and EMU-to-point scaling.
     - `DeckPdfExporter`: Bridges presentation slides into vector PDF streams via `pdf-engine` / vector primitives.

2. **Desktop Tauri IPC Bridge (`apps/unified-shell/src-tauri`)**:
   - `ppt_export_slide_svg(package_bytes, slide_idx) -> Result<String, String>`
   - `ppt_export_deck_pdf(package_bytes) -> Result<Vec<u8>, String>`
   - `ppt_convert_odp_to_pptx(odp_bytes) -> Result<Vec<u8>, String>`
   - `ppt_convert_pptx_to_odp(pptx_bytes) -> Result<Vec<u8>, String>`

3. **Presentation Shell Integration (`apps/unified-shell`)**:
   - `SlideExportCoordinator.ts`:
     - Manages SVG vector snapshot extraction, PDF download generation, and ODP format conversion.
   - Vitest test suite: `apps/unified-shell/src/__tests__/slide_odp_and_deck_export.test.ts`.

4. **Test & Verification**:
   - Rust integration test suite: `crates/ppt-engine/tests/subphase_48_2_2_test.rs`.
   - Vitest suite: 10/10 tests.
   - Master automated gatekeeper: `tools/verify-contract.ps1` exit code 0.

