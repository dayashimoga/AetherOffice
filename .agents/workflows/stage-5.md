---
description: Automated execution workflow for Stage 5: Unified 120fps WebGPU Render Core, Adaptive Desktop/Tablet/Mobile Shell & 5-OS CI/CD
---
# Workflow: Execute Stage 5 (/stage-5)
## Objective: Stage 5: Unified 120fps WebGPU Render Core, Adaptive Desktop/Tablet/Mobile Shell & 5-OS CI/CD

When invoked via `/stage-5`, execute the following subphases sequentially. Do NOT advance to the next subphase until `tools/verify-contract.ps1` exits with code 0 and `specs/stage-tracker.json` is updated:

### Subphase 5.1.1: Unified 120fps WebGPU / Cosmic-Text Render Core
- **Prompt Spec File:** `.agents/stages/stage-5/subphase-5.1.1.md`

**Instructions:**
```text
/execute-subphase 5.1.1
Invoke @adaptive-ui-builder under GEC-v1.0 Article III (120fps / < 2.5ms GPU raster budget).
1. In `crates/render-core`, implement a unified scene-graph and tile-virtualized display list compiler bridging `word-engine`, `excel-engine`, and `ppt-engine`:
   - Viewport culling & spatial R-Tree / grid indexing so only visible Excel cells, Word page slices, or PowerPoint slide layers generate GPU draw primitives (`QuadInstance`, `GlyphRun`, `BezierPathBatch`).
   - High-DPI subpixel glyph shaping and bi-directional (BiDi / RTL / CJK) text layout integration, plus deterministic SVG/PNG/PDF software rasterizer fallback for headless testing and print export.
2. Benchmark and verify `< 2.0 ms` display-list compilation for a 4K viewport in `crates/render-core/tests/subphase_5_1_1_test.rs` and commit with `[GEC-Verified]`.
```

**Post-Subphase Gate:**
Run `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 5.1.1` to verify contract compliance and advance the state tracker.

### Subphase 5.1.2: Interactive Hit-Testing, Selection Handles & Undo/Redo Command Bus
- **Prompt Spec File:** `.agents/stages/stage-5/subphase-5.1.2.md`

**Instructions:**
```text
/execute-subphase 5.1.2
Invoke @adaptive-ui-builder under GEC-v1.0 compliance.
1. In `crates/render-core/src/interaction.rs`, build the cross-document interaction controller:
   - Sub-millisecond pointer/touch hit-testing mapping `(x, y, zoom, scroll)` coordinates to exact Word character offsets, Excel cell/fill-handle coordinates, or PowerPoint shape/rotation handles.
   - Unified transactional Command Bus with unlimited Undo/Redo history, pinch-to-zoom inertia physics, and multi-format clipboard serialization.
2. Validate with >= 25 interaction, selection, and undo/redo test vectors in `subphase_5_1_2_test.rs` and commit with `[GEC-Verified]`.
```

**Post-Subphase Gate:**
Run `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 5.1.2` to verify contract compliance and advance the state tracker.

### Subphase 5.2.1: Tauri v2 Desktop & Laptop Shell (Ribbon, Multi-Tab, Formula Bar)
- **Prompt Spec File:** `.agents/stages/stage-5/subphase-5.2.1.md`

**Instructions:**
```text
/execute-subphase 5.2.1
Invoke @adaptive-ui-builder under GEC-v1.0 Article V (Desktop/Laptop Form Factor).
1. Scaffold the Tauri v2 application inside `apps/unified-shell/` (`package.json`, `vite.config.ts`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`) linking `ooxml-opc`, `word-engine`, `excel-engine`, `ppt-engine`, and `render-core`.
2. Implement the Desktop/Laptop UI (`>= 1024px`, fine pointer):
   - Multi-document workspace tab bar allowing `.docx`, `.xlsx`, and `.pptx` files to be edited side-by-side in a single lightweight window.
   - Contextual Ribbon toolbar (Home, Insert, Layout, Formulas, Slide Show, Review, View), Excel Formula Bar with live syntax highlighting, Word Ruler & Navigation Pane, and PowerPoint Slide Sorter filmstrip.
   - Full keyboard shortcut engine (`Ctrl/Cmd+C/V/Z/Y/S/F`, Excel `F2`/`F4`/Arrow navigation, Alt keytips).
3. Verify Tauri IPC commands and workspace tests via `tools/verify-contract.ps1` and commit with `[GEC-Verified]`.
```

**Post-Subphase Gate:**
Run `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 5.2.1` to verify contract compliance and advance the state tracker.

### Subphase 5.2.2: Adaptive Tablet & Mobile UI (Touch Drawers, Stylus Ink, Reflow Mode)
- **Prompt Spec File:** `.agents/stages/stage-5/subphase-5.2.2.md`

**Instructions:**
```text
/execute-subphase 5.2.2
Invoke @adaptive-ui-builder under GEC-v1.0 Article V (Tablet & Mobile Adaptation).
1. In `apps/unified-shell/src/`, implement automatic responsive layout switching based on viewport breakpoints, safe-area insets (`env(safe-area-inset-*)`), and input modality (`pointer: coarse`):
   - **Tablet Mode (`768px - 1023px`):** Collapsible touch-friendly Ribbon, direct stylus/Apple Pencil/S-Pen freehand ink annotation layer (`DrawingML ink`), and split-screen multitasking support.
   - **Mobile Mode (`< 768px`):** Replace top Ribbon with thumb-zone swipeable bottom command sheets, virtual-keyboard viewport height compensation, one-tap **Mobile Reflow Reader Mode** for Word documents, **Row Card Inspector** for rapid mobile Excel data entry, and touch laser-pointer mode for PowerPoint presentations.
2. Verify responsive state transitions and commit with `[GEC-Verified]`.
```

**Post-Subphase Gate:**
Run `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 5.2.2` to verify contract compliance and advance the state tracker.

### Subphase 5.3.1: End-to-End 5-Platform Release Audit & GitHub Artifact Publication
- **Prompt Spec File:** `.agents/stages/stage-5/subphase-5.3.1.md`

**Instructions:**
```text
/execute-subphase 5.3.1
Execute final Stage 5 Release Gatekeeping under GEC-v1.0:
1. Run `powershell -ExecutionPolicy Bypass -File .\removeRea_bootstrap.ps1 -KeepGlobalNpmPackage` to verify zero proprietary binaries or temp files exist anywhere in the repository.
2. Run `powershell -ExecutionPolicy Bypass -File .\tools\verify-contract.ps1` and verify 100% pass rate across all 22 subphase test suites (`ooxml-opc`, `excel-engine`, `word-engine`, `ppt-engine`, `render-core`) and `apps/unified-shell` frontend build.
3. Commit final release metadata, tag `v1.0.0`, push to GitHub to trigger `.github/workflows/build-all-platforms.yml`, and verify that all 5 platform artifacts (`Windows-x64`, `Linux-x64`, `macOS-Universal`, `Android-Universal-APK`, `iOS-IPA`) build and upload cleanly.
```

**Post-Subphase Gate:**
Run `powershell -ExecutionPolicy Bypass -File tools/stage-runner.ps1 -Complete 5.3.1` to verify contract compliance and advance the state tracker.

