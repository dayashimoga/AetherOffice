# Subphase 5.3.1: Tile-Virtualized Multi-Page PDF Canvas with Zoom & Text Select
- **Target Subsystem:** `render-core`
- **Skill:** `@adaptive-ui-builder`
- **REA Binary Analysis:** NO (ISO 32000-1 / Clean-Room)

## Objectives
1. Implement virtualized PDF multi-page canvas in `crates/render-core` (`pdf_viewport.rs`).
2. Continuous vertical scrolling, tile-based on-demand page rendering, and 25% - 500% zoom scaling.
3. Multi-page layout calculations with page gaps, margins, and viewport culling.
4. Interactive text selection overlay with bounding box hit-testing and copy-to-clipboard support.
5. Unit and integration tests in `crates/render-core/tests/subphase_5_3_1_test.rs`.
6. Enforce hard SLAs (< 8.33 ms frame budget, 120 fps scrolling).
