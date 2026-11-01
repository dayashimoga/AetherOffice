# Sub-Phase 49.1.1 Prompt Specification: ISO 32000-2 Vector Path, Image XObject & Font CMap Canvas Renderer

## Mission & Architectural Scope
Sub-Phase 49.1.1 implements the commercial-grade ISO 32000-2 vector path, image XObject, and font CMap canvas renderer for AetherPDF (`pdf-engine`, `render-core`, `unified-shell`).

### Architectural Invariants (GEC-v7.0):
1. `#![forbid(unsafe_code)]` strictly enforced across `pdf-engine` and `render-core`.
2. Zero panics, unhandled errors, or `.unwrap()` / `.expect()` in non-test library code.
3. Clean-room implementation of ISO 32000-2 Clause 8 (Graphics) & Clause 9 (Text).
4. Direct high-performance vector path rendering, image XObject extraction, and font glyph mapping.
5. Automated Gatekeeper: `tools/verify-contract.ps1` must exit `0`.

---

## Deliverables & Modules

### 1. `crates/pdf-engine/src/renderer.rs`
- `PdfPageRenderer`: Interprets content stream operators for a `PdfPage` and produces:
  - `RenderedVectorPathDto`: Segment commands (M, L, C, Z), fill/stroke styling, line cap/join, transforms.
  - `RenderedTextElementDto`: Extracted text, positioning, glyph bounds, font size, color, transform.
  - `RenderedImageElementDto`: Image XObjects, width, height, coordinates, format/base64 payload.
  - `RenderedPageCanvasDto`: Complete page canvas description with standalone SVG serialization.
- `AffineMatrix`: 2D transformation matrix compositor for graphics state CTM stack.
- Color conversion: DeviceRGB, DeviceGray, DeviceCMYK to 8-bit hex/sRGB.

### 2. Tauri IPC Commands & Signatures
- `pdf_render_page_canvas`
- `pdf_render_page_svg`
- `pdf_extract_page_images`

### 3. Frontend Presentation Shell Coordinator (`apps/unified-shell/src/state/PdfCanvasRendererCoordinator.ts`)
- High-DPI canvas coordination, page caching, zoom/DPI scaling, and vector overlay synchronization.
- Export in `apps/unified-shell/src/state/index.ts`.

### 4. Integration & Regression Test Suites
- Rust integration tests in `crates/pdf-engine/tests/subphase_49_1_1_test.rs` (10 tests).
- Vitest tests in `apps/unified-shell/src/__tests__/pdf_canvas_renderer.test.ts` (10 tests).
- Automated Gatekeeper `tools/verify-contract.ps1`.

