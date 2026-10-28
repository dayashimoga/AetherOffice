# Subphase 5.1.2: Interactive Hit-Testing, Selection Handles & Undo/Redo Command Bus
- **Stage:** 5 | **Phase:** 5.1
- **Target Crate:** `render-core`
- **Primary Agent Skill:** `@adaptive-ui-builder`
- **REA Binary Analysis Required:** NO (Pure ECMA-376 / Clean-Room)
- **Deliverable:** Unified cursor/touch hit-testing, multi-range selection, clipboard (HTML/OOXML/Plain), and atomic Undo/Redo transaction stack.
- **Validation Command:** `cargo test --package render-core --test subphase_5_1_2_test`

---

## Antigravity IDE Execution Prompt (Copy & Paste into Agent Manager)

```text
/execute-subphase 5.1.2
Invoke @adaptive-ui-builder under GEC-v1.0 compliance.
1. In `crates/render-core/src/interaction.rs`, build the cross-document interaction controller:
   - Sub-millisecond pointer/touch hit-testing mapping `(x, y, zoom, scroll)` coordinates to exact Word character offsets, Excel cell/fill-handle coordinates, or PowerPoint shape/rotation handles.
   - Unified transactional Command Bus with unlimited Undo/Redo history, pinch-to-zoom inertia physics, and multi-format clipboard serialization.
2. Validate with >= 25 interaction, selection, and undo/redo test vectors in `subphase_5_1_2_test.rs` and commit with `[GEC-Verified]`.
```
