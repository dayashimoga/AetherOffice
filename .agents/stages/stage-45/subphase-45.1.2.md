# Stage 45: High-Performance Canvas Rendering & 120fps Layout Virtualization
## Sub-Phase 45.1.2: Subpixel Text Shaper & High-DPI Canvas Compositor

### 1. Context & Objectives
Under GEC-v7.0 Articles I, V, and VI:
- Text rendering must maintain crisp typographic fidelity across any High-DPI display scale factor (1.0x, 1.25x, 1.5x, 2.0x, 3.0x) with subpixel precision and fractional kerning.
- BiDi text (mixed Arabic/Hebrew and Latin/CJK) must segment into directional visual runs with correct horizontal ordering.
- High-DPI canvas compositor must calculate physical device pixel bounds, pixel-snapped baselines, and glyph atlas coordinates with zero panics and $< 1.0\text{ ms}$ processing budget per frame.

### 2. Architecture & Components
1. **`HighDpiTextCompositor` & Subpixel Positioning (`crates/render-core/src/text.rs` or `subpixel.rs`)**:
   - `SubpixelKerning`: Kerning pair lookups and fractional glyph advance adjustment.
   - `VisualTextRun`: Ordered directional run with script, direction (`Ltr` | `Rtl`), start index, glyphs, and pixel-snapped baseline.
   - `HighDpiTextCompositor`:
     - Fractional snapping: `snap_to_device_pixel(coord: f64, dpr: f64) -> f64`.
     - Multi-run BiDi layout: decomposes paragraphs into visual runs, computes advances, and positions glyphs.
     - Device-pixel-aware glyph run scaling with crisp integer pixel heights and subpixel horizontal offsets.
2. **DisplayList & Compositor Integration**:
   - Seamless compilation into `DisplayCommand::DrawText` with subpixel-accurate glyph coordinates.
3. **Frontend High-DPI Canvas Compositor Adapter (`apps/unified-shell/src/components/HighDpiCanvasCompositor.ts`)**:
   - Canvas backing store scaling (`canvas.width = cssWidth * dpr`, `ctx.scale(dpr, dpr)`).
   - Subpixel baseline alignment and crisp text drawing at 120fps.
4. **Testing & Invariants**:
   - Rust integration test suite `crates/render-core/tests/subphase_45_1_2_test.rs`.
   - Vitest test suite `apps/unified-shell/src/__tests__/high_dpi_text_compositor.test.ts`.
   - Full gatekeeper `tools/verify-contract.ps1` exit 0.

