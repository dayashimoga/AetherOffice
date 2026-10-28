# Subphase 5.1.1: Unified 120fps WebGPU / Cosmic-Text Render Core
- **Stage:** 5 | **Phase:** 5.1
- **Target Crate:** `render-core`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **REA Binary Analysis Required:** NO (Pure ECMA-376 / Clean-Room)
- **Deliverable:** Shared GPU display-list compositor and tile virtualizer for Word pages, Excel grids, and PowerPoint slides.
- **Validation Command:** `cargo test --package render-core --test subphase_5_1_1_test`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 5.1.1
Invoke @adaptive-ui-builder under GEC-v1.0 Article III (120fps / < 2.5ms GPU raster budget).
1. In `crates/render-core`, implement a unified scene-graph and tile-virtualized display list compiler bridging `word-engine`, `excel-engine`, and `ppt-engine`:
   - Viewport culling & spatial R-Tree / grid indexing so only visible Excel cells, Word page slices, or PowerPoint slide layers generate GPU draw primitives (`QuadInstance`, `GlyphRun`, `BezierPathBatch`).
   - High-DPI subpixel glyph shaping and bi-directional (BiDi / RTL / CJK) text layout integration, plus deterministic SVG/PNG/PDF software rasterizer fallback for headless testing and print export.
2. Benchmark and verify `< 2.0 ms` display-list compilation for a 4K viewport in `crates/render-core/tests/subphase_5_1_1_test.rs` and commit with `[GEC-Verified]`.
```
