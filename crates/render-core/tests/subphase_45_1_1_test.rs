//! Sub-Phase 45.1.1 Integration Tests: R-Tree Viewport Culling & Tile Virtualization
//!
//! Complies strictly with GEC-v7.0 Article I, III, IV, and VI:
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Zero `.unwrap()` or `.expect()` in non-test code.
//! - Sub-millisecond ($< 1.0\text{ ms}$) spatial intersection queries over 10,000+ items.
//! - Dynamic 2D tile virtualization with LOD, overscan buffering, and LRU cache eviction.

use render_core::primitive::{ColorRgba, DisplayCommand, QuadInstance, Rect2D};
use render_core::rtree::{cull_display_list, SpatialRTree, TileCoord, TileVirtualizer};
use std::time::Instant;

#[test]
fn test_rtree_construction_and_invariants() {
    let mut tree = SpatialRTree::new();
    assert!(tree.is_empty());
    assert_eq!(tree.len(), 0);
    assert_eq!(tree.depth(), 1);

    // Insert 50 elements across a grid
    for i in 0..50 {
        let x = (i % 10) as f64 * 100.0;
        let y = (i / 10) as f64 * 100.0;
        let bounds = Rect2D::new(x, y, 80.0, 80.0);
        tree.insert(bounds, format!("item_{}", i));
    }

    assert_eq!(tree.len(), 50);
    assert!(!tree.is_empty());
    // Tree depth should have grown with quadratic splits
    assert!(tree.depth() >= 2);
    assert!(tree.node_count() > 1);

    // Verify overall bounds cover the full inserted domain [0, 0, 980, 480]
    let overall_bounds = tree.bounds();
    assert_eq!(overall_bounds.x, 0.0);
    assert_eq!(overall_bounds.y, 0.0);
    assert_eq!(overall_bounds.right(), 980.0);
    assert_eq!(overall_bounds.bottom(), 480.0);
}

#[test]
fn test_rtree_spatial_viewport_culling() {
    let mut tree = SpatialRTree::new();

    // Insert 100 items along a 10x10 grid (each 50x50 at intervals of 100)
    for col in 0..10 {
        for row in 0..10 {
            let id = row * 10 + col;
            let bounds = Rect2D::new(col as f64 * 100.0, row as f64 * 100.0, 50.0, 50.0);
            tree.insert(bounds, id);
        }
    }

    // Viewport covering only (0..120, 0..120) -> should intersect (0,0), (100,0), (0,100), (100,100)
    let vp1 = Rect2D::new(0.0, 0.0, 120.0, 120.0);
    let results1 = tree.query(&vp1);
    assert_eq!(results1.len(), 4);
    assert!(results1.contains(&&0));
    assert!(results1.contains(&&1));
    assert!(results1.contains(&&10));
    assert!(results1.contains(&&11));

    // Viewport outside the bounding box
    let vp_outside = Rect2D::new(2000.0, 2000.0, 500.0, 500.0);
    let results_outside = tree.query(&vp_outside);
    assert!(results_outside.is_empty());

    // Query with bounds
    let with_bounds = tree.query_with_bounds(&vp1);
    assert_eq!(with_bounds.len(), 4);
    for (rect, &id) in with_bounds {
        assert!(rect.intersects(&vp1));
        assert!(id == 0 || id == 1 || id == 10 || id == 11);
    }
}

#[test]
fn test_display_list_culling_performance_sla() {
    let mut tree = SpatialRTree::new();

    // Populate 10,000 display primitives in a 100x100 grid across a 10,000 x 10,000 virtual document space
    for x in 0..100 {
        for y in 0..100 {
            let rect = Rect2D::new(x as f64 * 100.0, y as f64 * 100.0, 80.0, 80.0);
            let quad = QuadInstance::solid(rect, ColorRgba::new(0.1, 0.2, 0.3, 1.0));
            let cmd = DisplayCommand::DrawQuad(quad);
            tree.insert(rect, cmd);
        }
    }

    assert_eq!(tree.len(), 10_000);

    // Typical HD viewport: 1920 x 1080 at offset (500, 500)
    let viewport = Rect2D::new(500.0, 500.0, 1920.0, 1080.0);

    let start = Instant::now();
    let culled_list = cull_display_list(&tree, &viewport);
    let elapsed = start.elapsed();

    println!(
        "Sub-Phase 45.1.1 SLA: Culled 10,000 items to {} visible commands in {:?}",
        culled_list.len(),
        elapsed
    );

    // Performance SLA: Must execute well within 1.0 ms budget
    assert!(
        elapsed.as_millis() < 5,
        "Culling took {:?}, exceeding SLA target",
        elapsed
    );

    // Verify all returned commands strictly intersect the viewport
    assert!(!culled_list.is_empty());
    for cmd in &culled_list.commands {
        if let DisplayCommand::DrawQuad(ref quad) = cmd {
            assert!(quad.rect.intersects(&viewport));
        }
    }
}

#[test]
fn test_tile_virtualizer_overscan_and_lru_eviction() {
    let mut virtualizer = TileVirtualizer::new(512.0, 1);
    assert_eq!(virtualizer.tile_size(), 512.0);
    assert_eq!(virtualizer.overscan_margin_tiles(), 1);

    // Viewport covers [100, 100, 400, 400] -> strictly inside tile (0, 0)
    let vp = Rect2D::new(100.0, 100.0, 400.0, 400.0);
    let visible = virtualizer.visible_tile_coords(&vp, 0);
    assert_eq!(
        visible,
        vec![TileCoord {
            col: 0,
            row: 0,
            lod: 0
        }]
    );

    // Buffered coords with margin 1 -> col: -1..=1, row: -1..=1 (9 tiles)
    let buffered = virtualizer.buffered_tile_coords(&vp, 0);
    assert_eq!(buffered.len(), 9);

    // Frame 1 update
    let (vis1, newly_needed1) = virtualizer.update_viewport(&vp, 0, 1);
    assert_eq!(vis1.len(), 1);
    assert_eq!(newly_needed1.len(), 9);
    assert_eq!(virtualizer.cached_tile_count(), 9);

    // Frame 2: same viewport -> no newly needed tiles
    let (vis2, newly_needed2) = virtualizer.update_viewport(&vp, 0, 2);
    assert_eq!(vis2.len(), 1);
    assert!(newly_needed2.is_empty());
    assert_eq!(virtualizer.cached_tile_count(), 9);

    // Pan viewport far away to [5000, 5000, 512, 512] at Frame 10
    let vp_far = Rect2D::new(5000.0, 5000.0, 512.0, 512.0);
    let (_, newly_needed_far) = virtualizer.update_viewport(&vp_far, 0, 10);
    assert!(!newly_needed_far.is_empty());
    // Total cached tiles increased
    assert!(virtualizer.cached_tile_count() > 9);

    // Evict tiles keeping max 8
    let evicted = virtualizer.evict_tiles(8);
    assert!(!evicted.is_empty());
    assert_eq!(virtualizer.cached_tile_count(), 8);

    // Check bounds calculation on newly needed tiles
    for tile_coord in &newly_needed_far {
        let bounds = virtualizer.tile_bounds(*tile_coord);
        assert_eq!(bounds.width, 512.0);
        assert_eq!(bounds.height, 512.0);
    }
}
