//! Spatial Indexing & Viewport Culling
//!
//! Provides spatial partitioning via uniform 2D grid tiles to cull non-visible
//! primitives with $O(1)$ spatial queries during 120Hz display list compilation.

use crate::primitive::Rect2D;
use std::collections::{HashMap, HashSet};

/// Uniform 2D grid index for fast spatial queries and viewport culling.
#[derive(Debug, Clone)]
pub struct SpatialGrid<T> {
    tile_size: f64,
    tiles: HashMap<(i32, i32), Vec<usize>>,
    items: Vec<(Rect2D, T)>,
}

impl<T> Default for SpatialGrid<T> {
    fn default() -> Self {
        Self::new(256.0)
    }
}

impl<T> SpatialGrid<T> {
    /// Create a new spatial grid with uniform tile size (default 256.0 DIPs).
    pub fn new(tile_size: f64) -> Self {
        let tile_size = if tile_size > 0.0 { tile_size } else { 256.0 };
        Self {
            tile_size,
            tiles: HashMap::new(),
            items: Vec::new(),
        }
    }

    pub fn tile_size(&self) -> f64 {
        self.tile_size
    }

    /// Compute tile coordinate range $[(c_{\min}, r_{\min}), (c_{\max}, r_{\max})]$ for a given bounding box.
    pub fn tile_range(&self, rect: &Rect2D) -> ((i32, i32), (i32, i32)) {
        let c_min = (rect.x / self.tile_size).floor() as i32;
        let c_max = (rect.right() / self.tile_size).floor() as i32;
        let r_min = (rect.y / self.tile_size).floor() as i32;
        let r_max = (rect.bottom() / self.tile_size).floor() as i32;
        ((c_min, r_min), (c_max, r_max))
    }

    /// Insert an item with its 2D bounding box into the spatial grid.
    pub fn insert(&mut self, bounds: Rect2D, item: T) -> usize {
        let idx = self.items.len();
        let ((c_min, r_min), (c_max, r_max)) = self.tile_range(&bounds);

        for c in c_min..=c_max {
            for r in r_min..=r_max {
                self.tiles.entry((c, r)).or_default().push(idx);
            }
        }

        self.items.push((bounds, item));
        idx
    }

    /// Query unique items that intersect the given viewport rectangle.
    pub fn query_indices(&self, viewport: &Rect2D) -> HashSet<usize> {
        let mut result = HashSet::new();
        let ((c_min, r_min), (c_max, r_max)) = self.tile_range(viewport);

        for c in c_min..=c_max {
            for r in r_min..=r_max {
                if let Some(indices) = self.tiles.get(&(c, r)) {
                    for &idx in indices {
                        let (ref bounds, _) = self.items[idx];
                        if bounds.intersects(viewport) {
                            result.insert(idx);
                        }
                    }
                }
            }
        }

        result
    }

    /// Query and collect references to all items intersecting the viewport.
    pub fn query(&self, viewport: &Rect2D) -> Vec<&T> {
        let indices = self.query_indices(viewport);
        let mut list = Vec::with_capacity(indices.len());
        for idx in indices {
            list.push(&self.items[idx].1);
        }
        list
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn clear(&mut self) {
        self.tiles.clear();
        self.items.clear();
    }
}
