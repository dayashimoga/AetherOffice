# Sub-Phase 48.1.2: Interactive SmartArt Builder, Grouping/Z-Order Layer Tree & Smart Guides

## Objective
Implement commercial-grade SmartArt generation, hierarchical shape grouping and ungrouping, Z-order layer tree management, and real-time magnetic smart alignment guides in `ppt-engine`, `render-core`, and the unified presentation shell (`apps/unified-shell`). Enable professional slide composition with zero data loss, `#![forbid(unsafe_code)]`, and clean-room provenance conforming to ECMA-376 Part 1 §19 & §20.

## Scope of Features

### 1. SmartArt Generation & Layout Engine (`crates/ppt-engine/src/smartart.rs`)
- **SmartArt Categories & Canonical Layouts:**
  - `Process`: Horizontal Process, Chevron List, Step-Up Process.
  - `Hierarchy`: Organization Chart, Top-Down Hierarchy Tree, Horizontal Tree.
  - `Cycle`: Basic Cycle, Radial Cycle, Segmented Gear.
  - `Matrix`: 2x2 Quadrant Matrix, Grid Matrix.
  - `Pyramid`: Basic Pyramid, Inverted Pyramid.
- **Data Model & Node Synthesis (`SmartArtDataTree`):**
  - Logical text/item hierarchy (`title`, `subitems`, `color_accent`, `level`).
  - Automated layout compilation into DrawingML shapes (`ShapeElement::Shape`) and connection arrows/lines with proper text formatting.
  - Theme color palette integration (accent 1 through accent 6 distribution across nodes).

### 2. Grouping, Ungrouping & Z-Order Layer Management (`crates/ppt-engine/src/layers.rs`)
- **Hierarchical Shape Grouping (`GroupShape`):**
  - Grouping selected shape IDs into a composite `GroupShape` with computed outer bounding box (`Transform2D`) and child coordinate mapping.
  - Ungrouping: Flattens `GroupShape` elements back to top-level slide shapes, accumulating relative transforms into absolute slide coordinates.
  - Nested group support: groups within groups handled deterministically.
- **Z-Order Manipulation:**
  - `bring_to_front(slide, shape_id)`
  - `send_to_back(slide, shape_id)`
  - `bring_forward(slide, shape_id)`
  - `send_backward(slide, shape_id)`
  - `reorder_shape_tree(slide, ordered_shape_ids)`
- **Serializable Layer Tree DTO (`SlideLayerTreeDto`, `LayerItemDto`):**
  - Exposes hierarchical shape and group structure to the presentation shell: shape ID, name, kind (`shape`, `group`, `text`, `media`, `smartArt`), z-index, visibility, and lock status.

### 3. Magnetic Smart Alignment Guides Engine (`crates/render-core/src/smart_guides.rs`)
- **Edge & Center Alignment Detection:**
  - Snaps dragged shape to left, center, right, top, middle, and bottom edges of peer shapes on the active slide.
  - Margin & slide center snapping (horizontal center, vertical center, 0.5-inch page margin).
- **Equidistant Spacing Detection:**
  - Detects when moving shape creates equal horizontal or vertical spacing between adjacent shapes.
- **Snapping Computation & Overlay Metadata (`SmartGuideResult`):**
  - Returns snapped coordinates (`snapped_x`, `snapped_y`) within snap threshold (default 5px / 45,000 EMUs).
  - Returns active guide lines with start/end points and guide type (`edge`, `center`, `spacing`) for high-DPI canvas overlay rendering.

### 4. Desktop Tauri IPC Commands (`apps/unified-shell/src-tauri/src/binary_commands.rs`)
- `ppt_build_smartart(package_bytes, slide_idx, layout_type, data_json, bounds)` -> updated package bytes.
- `ppt_group_shapes(package_bytes, slide_idx, shape_ids)` -> updated package bytes.
- `ppt_ungroup_shapes(package_bytes, slide_idx, group_id)` -> updated package bytes.
- `ppt_reorder_z_order(package_bytes, slide_idx, shape_id, action)` -> updated package bytes.
- `ppt_get_slide_layer_tree(package_bytes, slide_idx)` -> `SlideLayerTreeDto`.

### 5. Presentation Shell Layer & Guides Coordinator (`apps/unified-shell/src/state/SlideLayersCoordinator.ts`)
- Manages selection, grouping, z-order operations, and layer sidebar state.
- Smart guide calculation during canvas drag and shape transform interactions.
- Full undo/redo integration with `slidesState`.

## Engineering Invariants (GEC-v7.0)
- `#![forbid(unsafe_code)]` enforced across all domain crates.
- Zero panics, `.unwrap()`, or `.expect()` in non-test library code.
- Clean-room ECMA-376 Part 1 implementation.
- Zero compiler warnings under `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all -- --check`.
- 100% test pass rate across Rust workspace and frontend Vitest suites with zero regressions.

