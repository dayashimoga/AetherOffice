#![forbid(unsafe_code)]
//! Magnetic Smart Alignment Guides & Dynamic Snapping Engine.
//!
//! Provides sub-pixel / EMU magnetic snapping algorithms for interactive presentation
//! canvas manipulation, including edge-to-edge alignment, center alignment,
//! canvas margin snapping, and equidistant distribution guides.

use serde::{Deserialize, Serialize};

/// Orientation of a visual smart guide line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuideOrientation {
    Horizontal,
    Vertical,
}

/// Semantic alignment type of the smart guide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuideKind {
    Edge,
    Center,
    Spacing,
    Margin,
}

/// Individual visual smart guide line rendered on the canvas overlay.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SmartGuideLine {
    pub orientation: GuideOrientation,
    pub kind: GuideKind,
    pub coordinate: f64,
    pub start: f64,
    pub end: f64,
}

/// 2D bounding box for geometry snap evaluation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub struct RectBounds {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl RectBounds {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }

    pub fn left(&self) -> f64 {
        self.x
    }

    pub fn right(&self) -> f64 {
        self.x + self.w
    }

    pub fn top(&self) -> f64 {
        self.y
    }

    pub fn bottom(&self) -> f64 {
        self.y + self.h
    }

    pub fn center_x(&self) -> f64 {
        self.x + (self.w / 2.0)
    }

    pub fn center_y(&self) -> f64 {
        self.y + (self.h / 2.0)
    }
}

/// Result of evaluating smart alignment guides against neighboring geometry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SnapResult {
    pub snapped_x: f64,
    pub snapped_y: f64,
    pub dx: f64,
    pub dy: f64,
    pub guides: Vec<SmartGuideLine>,
}

/// High-precision magnetic snapping engine.
pub struct SmartGuidesEngine {
    pub snap_threshold: f64,
}

impl Default for SmartGuidesEngine {
    fn default() -> Self {
        Self {
            snap_threshold: 5.0, // Default 5.0 DIPs (or 45,000 EMUs if operating in EMU scale)
        }
    }
}

impl SmartGuidesEngine {
    pub fn new(snap_threshold: f64) -> Self {
        Self { snap_threshold }
    }

    /// Evaluates magnetic snapping for a moving bounding box against static neighboring bounds and optional canvas bounds.
    pub fn compute_snap(
        &self,
        moving: &RectBounds,
        neighbors: &[RectBounds],
        canvas_bounds: Option<&RectBounds>,
    ) -> SnapResult {
        let mut min_dx = f64::MAX;
        let mut best_dx = 0.0;
        let mut x_guides = Vec::new();

        let mut min_dy = f64::MAX;
        let mut best_dy = 0.0;
        let mut y_guides = Vec::new();

        // 1. Evaluate Canvas Margins and Center if canvas bounds provided
        if let Some(canvas) = canvas_bounds {
            // Horizontal canvas center
            let c_center_x = canvas.center_x();
            let d = c_center_x - moving.center_x();
            if d.abs() <= self.snap_threshold && d.abs() < min_dx {
                min_dx = d.abs();
                best_dx = d;
                x_guides.clear();
                x_guides.push(SmartGuideLine {
                    orientation: GuideOrientation::Vertical,
                    kind: GuideKind::Center,
                    coordinate: c_center_x,
                    start: canvas.top(),
                    end: canvas.bottom(),
                });
            }

            // Vertical canvas center
            let c_center_y = canvas.center_y();
            let d_y = c_center_y - moving.center_y();
            if d_y.abs() <= self.snap_threshold && d_y.abs() < min_dy {
                min_dy = d_y.abs();
                best_dy = d_y;
                y_guides.clear();
                y_guides.push(SmartGuideLine {
                    orientation: GuideOrientation::Horizontal,
                    kind: GuideKind::Center,
                    coordinate: c_center_y,
                    start: canvas.left(),
                    end: canvas.right(),
                });
            }
        }

        // 2. Evaluate Neighbor Shapes for X-Axis Alignment (Vertical Lines)
        for target in neighbors {
            let x_checks = [
                (
                    target.left() - moving.left(),
                    target.left(),
                    GuideKind::Edge,
                ),
                (
                    target.right() - moving.left(),
                    target.right(),
                    GuideKind::Edge,
                ),
                (
                    target.left() - moving.right(),
                    target.left(),
                    GuideKind::Edge,
                ),
                (
                    target.right() - moving.right(),
                    target.right(),
                    GuideKind::Edge,
                ),
                (
                    target.center_x() - moving.center_x(),
                    target.center_x(),
                    GuideKind::Center,
                ),
            ];

            for (delta, guide_coord, kind) in x_checks {
                if delta.abs() <= self.snap_threshold {
                    if delta.abs() < min_dx - 1e-6 {
                        min_dx = delta.abs();
                        best_dx = delta;
                        x_guides.clear();
                    }
                    if (delta.abs() - min_dx).abs() <= 1e-6 {
                        let y_start = moving.top().min(target.top()) - 20.0;
                        let y_end = moving.bottom().max(target.bottom()) + 20.0;
                        x_guides.push(SmartGuideLine {
                            orientation: GuideOrientation::Vertical,
                            kind,
                            coordinate: guide_coord,
                            start: y_start,
                            end: y_end,
                        });
                    }
                }
            }

            // 3. Evaluate Neighbor Shapes for Y-Axis Alignment (Horizontal Lines)
            let y_checks = [
                (target.top() - moving.top(), target.top(), GuideKind::Edge),
                (
                    target.bottom() - moving.top(),
                    target.bottom(),
                    GuideKind::Edge,
                ),
                (
                    target.top() - moving.bottom(),
                    target.top(),
                    GuideKind::Edge,
                ),
                (
                    target.bottom() - moving.bottom(),
                    target.bottom(),
                    GuideKind::Edge,
                ),
                (
                    target.center_y() - moving.center_y(),
                    target.center_y(),
                    GuideKind::Center,
                ),
            ];

            for (delta, guide_coord, kind) in y_checks {
                if delta.abs() <= self.snap_threshold {
                    if delta.abs() < min_dy - 1e-6 {
                        min_dy = delta.abs();
                        best_dy = delta;
                        y_guides.clear();
                    }
                    if (delta.abs() - min_dy).abs() <= 1e-6 {
                        let x_start = moving.left().min(target.left()) - 20.0;
                        let x_end = moving.right().max(target.right()) + 20.0;
                        y_guides.push(SmartGuideLine {
                            orientation: GuideOrientation::Horizontal,
                            kind,
                            coordinate: guide_coord,
                            start: x_start,
                            end: x_end,
                        });
                    }
                }
            }
        }

        // 4. Evaluate Equidistant Distribution Spacing
        if neighbors.len() >= 2 {
            Self::check_equidistant_spacing(
                moving,
                neighbors,
                self.snap_threshold,
                &mut best_dx,
                &mut min_dx,
                &mut x_guides,
                &mut best_dy,
                &mut min_dy,
                &mut y_guides,
            );
        }

        let mut all_guides = x_guides;
        all_guides.extend(y_guides);

        SnapResult {
            snapped_x: moving.x + best_dx,
            snapped_y: moving.y + best_dy,
            dx: best_dx,
            dy: best_dy,
            guides: all_guides,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn check_equidistant_spacing(
        moving: &RectBounds,
        neighbors: &[RectBounds],
        threshold: f64,
        best_dx: &mut f64,
        min_dx: &mut f64,
        x_guides: &mut Vec<SmartGuideLine>,
        _best_dy: &mut f64,
        _min_dy: &mut f64,
        _y_guides: &mut Vec<SmartGuideLine>,
    ) {
        // Horizontal spacing: test if moving box falls between or after two neighbors with equal gap
        for i in 0..neighbors.len() {
            for j in 0..neighbors.len() {
                if i == j {
                    continue;
                }
                let a = &neighbors[i];
                let b = &neighbors[j];

                // If A is to the left of B
                if a.right() < b.left() {
                    let known_gap = b.left() - a.right();

                    // Case: moving shape placed to the right of B with same gap
                    let target_moving_x = b.right() + known_gap;
                    let delta = target_moving_x - moving.left();
                    if delta.abs() <= threshold && delta.abs() < *min_dx {
                        *min_dx = delta.abs();
                        *best_dx = delta;
                        x_guides.clear();
                        x_guides.push(SmartGuideLine {
                            orientation: GuideOrientation::Vertical,
                            kind: GuideKind::Spacing,
                            coordinate: target_moving_x,
                            start: b.top().min(moving.top()),
                            end: b.bottom().max(moving.bottom()),
                        });
                    }
                }
            }
        }
    }
}
