# Sub-Phase 45.1.1 Specification: R-Tree Viewport Culling & Tile Virtualization

## 1. Context & Objectives
In compliance with GEC-v7.0 Article I, III, IV, and VI:
- 120fps Rendering Frame Budget: $\le 8.33\text{ ms}$ per frame under continuous scrolling.
- Display list spatial query SLA: $< 1.0\text{ ms}$ across 10,000+ indexed elements.
- Spatial index for arbitrary 2D bounding boxes using hierarchical R-Tree partitioning.
- Dynamic Tile Virtualizer supporting zoom levels, LOD, viewports, overscan margins, and LRU cache eviction.
- Clean-room safe Rust in `crates/render-core/src/rtree.rs` under `#![forbid(unsafe_code)]` with zero unhandled error states.

## 2. Technical Scope

### 2.1 Spatial R-Tree Engine (`crates/render-core/src/rtree.rs`)
- `SpatialRTree<T>`:
  - Branching constants: `MIN_ENTRIES = 4`, `MAX_ENTRIES = 16`.
  - Node representation:
    - `Leaf { entries: Vec<(Rect2D, T)> }`
    - `Internal { entries: Vec<(Rect2D, RTreeNode<T>)> }`
  - Insertion with quadratic split:
    - `insert(bounds: Rect2D, item: T)`
    - `choose_subtree`: picks child with minimal MBR area enlargement.
    - `split_node`: partitions overflowing node into two balanced sub-nodes.
  - Queries:
    - `query(&self, viewport: &Rect2D) -> Vec<&T>`
    - `query_with_bounds(&self, viewport: &Rect2D) -> Vec<(&Rect2D, &T)>`
    - `query_indices(&self, viewport: &Rect2D) -> Vec<usize>`
  - Statistics: `len()`, `is_empty()`, `depth()`, `bounds()`.

### 2.2 Tile Virtualization (`crates/render-core/src/tile_virtualization.rs` or `rtree.rs`)
- `TileCoord`: `col`, `row`, `lod`.
- `VirtualTile`: coordinates, bounds, status.
- `TileVirtualizer`:
  - `tile_size: f64` (default 512.0 DIPs).
  - `overscan: i32` (1 or 2 tile margin for smooth scrolling).
  - `visible_tile_coords(viewport, lod) -> Vec<TileCoord>`.
  - `buffered_tile_coords(viewport, lod) -> Vec<TileCoord>`.
  - `evict_tiles(active_coords, max_cached) -> Vec<TileCoord>`.
- Display list integration: `cull_display_list(tree, viewport) -> DisplayList`.

### 2.3 Verification Gates
- 100% Rust unit tests in `crates/render-core/tests/subphase_45_1_1_test.rs`.
- Frontend Vitest suite in `apps/unified-shell/src/__tests__/rtree_viewport_virtualization.test.ts`.
- `tools/verify-contract.ps1` exit code 0.

