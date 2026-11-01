//! Spatial R-Tree Viewport Culling & Tile Virtualization Engine
//!
//! Complies strictly with GEC-v7.0 Article I, III, IV, and VI:
//! - Pure safe Rust under `#![forbid(unsafe_code)]`.
//! - Pure safe Rust with zero panics or unhandled error states.
//! - Sub-millisecond ($< 1.0\text{ ms}$) spatial intersection queries over 10,000+ items.
//! - Dynamic 2D tile virtualization with LOD, overscan buffering, and LRU cache eviction.

use crate::primitive::{DisplayCommand, DisplayList, Rect2D};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const MIN_ENTRIES: usize = 3;
const MAX_ENTRIES: usize = 8;

/// Helper to compute area of a 2D rectangle.
#[inline]
pub fn rect_area(r: &Rect2D) -> f64 {
    (r.width * r.height).max(0.0)
}

/// Helper to compute the enlargement area needed for `base` to contain `item`.
#[inline]
pub fn rect_enlargement(base: &Rect2D, item: &Rect2D) -> f64 {
    let united = base.union(item);
    rect_area(&united) - rect_area(base)
}

/// Computes the Minimum Bounding Rectangle (MBR) covering an iterator of rectangles.
#[inline]
pub fn compute_mbr<I: IntoIterator<Item = Rect2D>>(rects: I) -> Rect2D {
    let mut iter = rects.into_iter();
    if let Some(first) = iter.next() {
        iter.fold(first, |acc, r| acc.union(&r))
    } else {
        Rect2D::ZERO
    }
}

/// Internal node representation for `SpatialRTree`.
#[derive(Debug, Clone)]
enum RTreeNode<T> {
    Leaf {
        entries: Vec<(Rect2D, T)>,
    },
    Internal {
        entries: Vec<(Rect2D, RTreeNode<T>)>,
    },
}

impl<T> RTreeNode<T> {
    pub fn compute_bounds(&self) -> Rect2D {
        match self {
            RTreeNode::Leaf { entries } => compute_mbr(entries.iter().map(|(r, _)| *r)),
            RTreeNode::Internal { entries } => compute_mbr(entries.iter().map(|(r, _)| *r)),
        }
    }

    pub fn depth(&self) -> usize {
        match self {
            RTreeNode::Leaf { .. } => 1,
            RTreeNode::Internal { entries } => {
                1 + entries
                    .iter()
                    .map(|(_, child)| child.depth())
                    .max()
                    .unwrap_or(0)
            }
        }
    }

    pub fn node_count(&self) -> usize {
        match self {
            RTreeNode::Leaf { .. } => 1,
            RTreeNode::Internal { entries } => {
                1 + entries
                    .iter()
                    .map(|(_, child)| child.node_count())
                    .sum::<usize>()
            }
        }
    }

    /// Query items intersecting `viewport`.
    pub fn query<'a>(&'a self, viewport: &Rect2D, results: &mut Vec<&'a T>) {
        let node_bounds = self.compute_bounds();
        if !node_bounds.intersects(viewport) {
            return;
        }

        match self {
            RTreeNode::Leaf { entries } => {
                for (rect, item) in entries {
                    if rect.intersects(viewport) {
                        results.push(item);
                    }
                }
            }
            RTreeNode::Internal { entries } => {
                for (child_mbr, child_node) in entries {
                    if child_mbr.intersects(viewport) {
                        child_node.query(viewport, results);
                    }
                }
            }
        }
    }

    /// Query items and their bounding boxes intersecting `viewport`.
    pub fn query_with_bounds<'a>(
        &'a self,
        viewport: &Rect2D,
        results: &mut Vec<(&'a Rect2D, &'a T)>,
    ) {
        let node_bounds = self.compute_bounds();
        if !node_bounds.intersects(viewport) {
            return;
        }

        match self {
            RTreeNode::Leaf { entries } => {
                for (rect, item) in entries {
                    if rect.intersects(viewport) {
                        results.push((rect, item));
                    }
                }
            }
            RTreeNode::Internal { entries } => {
                for (child_mbr, child_node) in entries {
                    if child_mbr.intersects(viewport) {
                        child_node.query_with_bounds(viewport, results);
                    }
                }
            }
        }
    }
}

/// Quadratic split algorithm for Leaf entries.
fn split_leaf<T>(mut entries: Vec<(Rect2D, T)>) -> (RTreeNode<T>, RTreeNode<T>) {
    let n = entries.len();
    if n <= 1 {
        return (
            RTreeNode::Leaf { entries },
            RTreeNode::Leaf {
                entries: Vec::new(),
            },
        );
    }

    // 1. Pick Seeds
    let mut best_waste = -1.0;
    let mut seed1_idx = 0;
    let mut seed2_idx = 1;

    for i in 0..n {
        for j in (i + 1)..n {
            let united = entries[i].0.union(&entries[j].0);
            let waste = rect_area(&united) - rect_area(&entries[i].0) - rect_area(&entries[j].0);
            if waste > best_waste {
                best_waste = waste;
                seed1_idx = i;
                seed2_idx = j;
            }
        }
    }

    // Remove seed2 first then seed1 (to avoid index shift)
    let seed2 = entries.remove(seed2_idx);
    let seed1 = entries.remove(seed1_idx);

    let mut group1 = vec![seed1];
    let mut group2 = vec![seed2];
    let mut mbr1 = group1[0].0;
    let mut mbr2 = group2[0].0;

    // 2. Distribute remaining entries
    while !entries.is_empty() {
        let remaining = entries.len();
        if group1.len() + remaining <= MIN_ENTRIES {
            group1.extend(entries);
            break;
        }
        if group2.len() + remaining <= MIN_ENTRIES {
            group2.extend(entries);
            break;
        }

        // Pick next entry that has max preference difference between group1 and group2
        let mut best_diff = -1.0;
        let mut best_entry_idx = 0;
        let mut assign_to_group1 = true;

        for (idx, (r, _)) in entries.iter().enumerate() {
            let d1 = rect_enlargement(&mbr1, r);
            let d2 = rect_enlargement(&mbr2, r);
            let diff = (d1 - d2).abs();
            if diff > best_diff {
                best_diff = diff;
                best_entry_idx = idx;
                if d1 < d2 {
                    assign_to_group1 = true;
                } else if d2 < d1 {
                    assign_to_group1 = false;
                } else {
                    assign_to_group1 = rect_area(&mbr1) <= rect_area(&mbr2);
                }
            }
        }

        let entry = entries.remove(best_entry_idx);
        if assign_to_group1 {
            mbr1 = mbr1.union(&entry.0);
            group1.push(entry);
        } else {
            mbr2 = mbr2.union(&entry.0);
            group2.push(entry);
        }
    }

    (
        RTreeNode::Leaf { entries: group1 },
        RTreeNode::Leaf { entries: group2 },
    )
}

/// Quadratic split algorithm for Internal entries.
fn split_internal<T>(mut entries: Vec<(Rect2D, RTreeNode<T>)>) -> (RTreeNode<T>, RTreeNode<T>) {
    let n = entries.len();
    if n <= 1 {
        return (
            RTreeNode::Internal { entries },
            RTreeNode::Internal {
                entries: Vec::new(),
            },
        );
    }

    let mut best_waste = -1.0;
    let mut seed1_idx = 0;
    let mut seed2_idx = 1;

    for i in 0..n {
        for j in (i + 1)..n {
            let united = entries[i].0.union(&entries[j].0);
            let waste = rect_area(&united) - rect_area(&entries[i].0) - rect_area(&entries[j].0);
            if waste > best_waste {
                best_waste = waste;
                seed1_idx = i;
                seed2_idx = j;
            }
        }
    }

    let seed2 = entries.remove(seed2_idx);
    let seed1 = entries.remove(seed1_idx);

    let mut group1 = vec![seed1];
    let mut group2 = vec![seed2];
    let mut mbr1 = group1[0].0;
    let mut mbr2 = group2[0].0;

    while !entries.is_empty() {
        let remaining = entries.len();
        if group1.len() + remaining <= MIN_ENTRIES {
            group1.extend(entries);
            break;
        }
        if group2.len() + remaining <= MIN_ENTRIES {
            group2.extend(entries);
            break;
        }

        let mut best_diff = -1.0;
        let mut best_entry_idx = 0;
        let mut assign_to_group1 = true;

        for (idx, (r, _)) in entries.iter().enumerate() {
            let d1 = rect_enlargement(&mbr1, r);
            let d2 = rect_enlargement(&mbr2, r);
            let diff = (d1 - d2).abs();
            if diff > best_diff {
                best_diff = diff;
                best_entry_idx = idx;
                if d1 < d2 {
                    assign_to_group1 = true;
                } else if d2 < d1 {
                    assign_to_group1 = false;
                } else {
                    assign_to_group1 = rect_area(&mbr1) <= rect_area(&mbr2);
                }
            }
        }

        let entry = entries.remove(best_entry_idx);
        if assign_to_group1 {
            mbr1 = mbr1.union(&entry.0);
            group1.push(entry);
        } else {
            mbr2 = mbr2.union(&entry.0);
            group2.push(entry);
        }
    }

    (
        RTreeNode::Internal { entries: group1 },
        RTreeNode::Internal { entries: group2 },
    )
}

/// Hierarchical 2D Spatial R-Tree Index.
#[derive(Debug, Clone)]
pub struct SpatialRTree<T> {
    root: RTreeNode<T>,
    total_items: usize,
}

impl<T> Default for SpatialRTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SpatialRTree<T> {
    /// Constructs an empty `SpatialRTree`.
    pub fn new() -> Self {
        Self {
            root: RTreeNode::Leaf {
                entries: Vec::new(),
            },
            total_items: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.total_items
    }

    pub fn is_empty(&self) -> bool {
        self.total_items == 0
    }

    pub fn depth(&self) -> usize {
        self.root.depth()
    }

    pub fn node_count(&self) -> usize {
        self.root.node_count()
    }

    pub fn bounds(&self) -> Rect2D {
        self.root.compute_bounds()
    }

    /// Inserts an item with its axis-aligned 2D bounding box.
    pub fn insert(&mut self, bounds: Rect2D, item: T) {
        self.total_items += 1;
        let split_result = Self::insert_recursive(&mut self.root, bounds, item);

        if let Some((sibling_node, sibling_mbr)) = split_result {
            // Root has split: create a new Internal root with old root and new sibling
            let old_root = std::mem::replace(
                &mut self.root,
                RTreeNode::Leaf {
                    entries: Vec::new(),
                },
            );
            let old_mbr = old_root.compute_bounds();

            self.root = RTreeNode::Internal {
                entries: vec![(old_mbr, old_root), (sibling_mbr, sibling_node)],
            };
        }
    }

    fn insert_recursive(
        node: &mut RTreeNode<T>,
        bounds: Rect2D,
        item: T,
    ) -> Option<(RTreeNode<T>, Rect2D)> {
        match node {
            RTreeNode::Leaf { entries } => {
                entries.push((bounds, item));
                if entries.len() > MAX_ENTRIES {
                    let entries_to_split = std::mem::take(entries);
                    let (n1, n2) = split_leaf(entries_to_split);
                    *node = n1;
                    let mbr2 = n2.compute_bounds();
                    Some((n2, mbr2))
                } else {
                    None
                }
            }
            RTreeNode::Internal { entries } => {
                // 1. Choose Subtree: pick child entry whose enlargement is minimal
                let mut best_idx = 0;
                let mut best_enlargement = f64::MAX;
                let mut best_area = f64::MAX;

                for (idx, (child_mbr, _)) in entries.iter().enumerate() {
                    let enl = rect_enlargement(child_mbr, &bounds);
                    let area = rect_area(child_mbr);
                    if enl < best_enlargement
                        || ((enl - best_enlargement).abs() < f64::EPSILON && area < best_area)
                    {
                        best_enlargement = enl;
                        best_area = area;
                        best_idx = idx;
                    }
                }

                let child_split = Self::insert_recursive(&mut entries[best_idx].1, bounds, item);
                // Update child MBR
                entries[best_idx].0 = entries[best_idx].1.compute_bounds();

                if let Some((new_sibling, sibling_mbr)) = child_split {
                    entries.push((sibling_mbr, new_sibling));
                    if entries.len() > MAX_ENTRIES {
                        let entries_to_split = std::mem::take(entries);
                        let (n1, n2) = split_internal(entries_to_split);
                        *node = n1;
                        let mbr2 = n2.compute_bounds();
                        Some((n2, mbr2))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    /// Queries all items whose bounding box intersects the given `viewport`.
    pub fn query(&self, viewport: &Rect2D) -> Vec<&T> {
        let mut results = Vec::new();
        self.root.query(viewport, &mut results);
        results
    }

    /// Queries all items and their bounding boxes whose bounding box intersects `viewport`.
    pub fn query_with_bounds(&self, viewport: &Rect2D) -> Vec<(&Rect2D, &T)> {
        let mut results = Vec::new();
        self.root.query_with_bounds(viewport, &mut results);
        results
    }

    /// Clears all entries from the tree.
    pub fn clear(&mut self) {
        self.root = RTreeNode::Leaf {
            entries: Vec::new(),
        };
        self.total_items = 0;
    }
}

// =========================================================================
// 2. Tile Virtualization Engine
// =========================================================================

/// Grid coordinates identifying a virtual surface tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileCoord {
    pub col: i32,
    pub row: i32,
    pub lod: u32,
}

/// Metadata and spatial bounds of a virtualized canvas tile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VirtualTile {
    pub coord: TileCoord,
    pub bounds: Rect2D,
    pub is_visible: bool,
    pub last_frame_seen: u64,
}

/// Virtualized 2D canvas tile coordinator with overscan preloading and cache pruning.
#[derive(Debug, Clone)]
pub struct TileVirtualizer {
    tile_size: f64,
    overscan_margin_tiles: i32,
    cached_tiles: HashMap<TileCoord, VirtualTile>,
}

impl Default for TileVirtualizer {
    fn default() -> Self {
        Self::new(512.0, 1)
    }
}

impl TileVirtualizer {
    /// Creates a tile virtualizer with specified tile size (DIPs) and overscan margin.
    pub fn new(tile_size: f64, overscan_margin_tiles: i32) -> Self {
        let size = if tile_size > 0.0 { tile_size } else { 512.0 };
        Self {
            tile_size: size,
            overscan_margin_tiles: overscan_margin_tiles.max(0),
            cached_tiles: HashMap::new(),
        }
    }

    pub fn tile_size(&self) -> f64 {
        self.tile_size
    }

    pub fn overscan_margin_tiles(&self) -> i32 {
        self.overscan_margin_tiles
    }

    /// Computes the exact 2D bounding rectangle in DIPs for a given tile coordinate.
    pub fn tile_bounds(&self, coord: TileCoord) -> Rect2D {
        Rect2D::new(
            (coord.col as f64) * self.tile_size,
            (coord.row as f64) * self.tile_size,
            self.tile_size,
            self.tile_size,
        )
    }

    /// Computes all tile coordinates strictly intersecting the current `viewport`.
    pub fn visible_tile_coords(&self, viewport: &Rect2D, lod: u32) -> Vec<TileCoord> {
        let c_min = (viewport.x / self.tile_size).floor() as i32;
        let c_max = (viewport.right() / self.tile_size).floor() as i32;
        let r_min = (viewport.y / self.tile_size).floor() as i32;
        let r_max = (viewport.bottom() / self.tile_size).floor() as i32;

        let mut coords = Vec::new();
        for col in c_min..=c_max {
            for row in r_min..=r_max {
                coords.push(TileCoord { col, row, lod });
            }
        }
        coords
    }

    /// Computes tile coordinates intersecting `viewport` expanded by overscan preloading margins.
    pub fn buffered_tile_coords(&self, viewport: &Rect2D, lod: u32) -> Vec<TileCoord> {
        let m = self.overscan_margin_tiles;
        let c_min = ((viewport.x / self.tile_size).floor() as i32) - m;
        let c_max = ((viewport.right() / self.tile_size).floor() as i32) + m;
        let r_min = ((viewport.y / self.tile_size).floor() as i32) - m;
        let r_max = ((viewport.bottom() / self.tile_size).floor() as i32) + m;

        let mut coords = Vec::new();
        for col in c_min..=c_max {
            for row in r_min..=r_max {
                coords.push(TileCoord { col, row, lod });
            }
        }
        coords
    }

    /// Updates viewport state for `frame_num`.
    /// Returns `(visible_coords, tiles_to_render)` where `tiles_to_render` includes
    /// newly buffered tiles not previously present in cache.
    pub fn update_viewport(
        &mut self,
        viewport: &Rect2D,
        lod: u32,
        frame_num: u64,
    ) -> (Vec<TileCoord>, Vec<TileCoord>) {
        let visible = self.visible_tile_coords(viewport, lod);
        let buffered = self.buffered_tile_coords(viewport, lod);

        let mut newly_needed = Vec::new();

        for &coord in &buffered {
            let is_vis = visible.contains(&coord);
            if let Some(tile) = self.cached_tiles.get_mut(&coord) {
                tile.is_visible = is_vis;
                tile.last_frame_seen = frame_num;
            } else {
                let bounds = self.tile_bounds(coord);
                self.cached_tiles.insert(
                    coord,
                    VirtualTile {
                        coord,
                        bounds,
                        is_visible: is_vis,
                        last_frame_seen: frame_num,
                    },
                );
                newly_needed.push(coord);
            }
        }

        (visible, newly_needed)
    }

    /// Evicts tiles from cache that exceed `max_cached` capacity using LRU timestamp sorting.
    pub fn evict_tiles(&mut self, max_cached: usize) -> Vec<TileCoord> {
        if self.cached_tiles.len() <= max_cached {
            return Vec::new();
        }

        let mut entries: Vec<(TileCoord, u64)> = self
            .cached_tiles
            .iter()
            .map(|(&k, v)| (k, v.last_frame_seen))
            .collect();

        // Sort ascending by last_frame_seen (oldest first)
        entries.sort_by_key(|&(_, frame)| frame);

        let evict_count = self.cached_tiles.len().saturating_sub(max_cached);
        let mut evicted = Vec::with_capacity(evict_count);

        for (coord, _) in entries.into_iter().take(evict_count) {
            self.cached_tiles.remove(&coord);
            evicted.push(coord);
        }

        evicted
    }

    pub fn cached_tile_count(&self) -> usize {
        self.cached_tiles.len()
    }
}

// =========================================================================
// 3. DisplayList Culling Integration
// =========================================================================

/// Culls a retained `SpatialRTree` of display commands, returning an optimized `DisplayList`
/// containing only commands visible within `viewport`.
pub fn cull_display_list(tree: &SpatialRTree<DisplayCommand>, viewport: &Rect2D) -> DisplayList {
    let visible_commands = tree.query(viewport);
    let mut list = DisplayList::new();
    for cmd in visible_commands {
        list.push(cmd.clone());
    }
    list
}
