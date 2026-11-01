//! gesture.rs
//! GEC-v7.0 Stage 52 (Sub-Phase 52.2.2)
//!
//! Authoritative touch gesture recognizer, mathematical stylus palm rejection filter,
//! and adaptive multi-pane split grid layout compiler.

#![forbid(unsafe_code)]

use crate::adaptive::SafeAreaInsets;
use crate::primitive::Rect2D;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Individual touch or stylus contact point reported from pointer event streams.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TouchPoint {
    pub id: u64,
    pub x: f64,
    pub y: f64,
    pub timestamp_ms: u64,
    pub radius_x: f64,
    pub radius_y: f64,
    pub pressure: f64,
    pub is_stylus: bool,
}

impl TouchPoint {
    pub fn new(id: u64, x: f64, y: f64, timestamp_ms: u64) -> Self {
        Self {
            id,
            x,
            y,
            timestamp_ms,
            radius_x: 8.0,
            radius_y: 8.0,
            pressure: 0.5,
            is_stylus: false,
        }
    }

    pub fn with_geometry(
        mut self,
        radius_x: f64,
        radius_y: f64,
        pressure: f64,
        is_stylus: bool,
    ) -> Self {
        self.radius_x = radius_x;
        self.radius_y = radius_y;
        self.pressure = pressure;
        self.is_stylus = is_stylus;
        self
    }

    /// Computes elliptical contact area $\pi \cdot r_x \cdot r_y$.
    pub fn contact_area(&self) -> f64 {
        std::f64::consts::PI * self.radius_x * self.radius_y
    }
}

/// Cardinal direction for two-finger swipe gestures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SwipeDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Classified high-level multi-touch gesture events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "kebab-case")]
pub enum TouchGesture {
    None,
    PinchZoom {
        scale: f64,
        focal_x: f64,
        focal_y: f64,
    },
    PanScroll {
        delta_x: f64,
        delta_y: f64,
    },
    DoubleTap {
        tap_x: f64,
        tap_y: f64,
    },
    LongPress {
        press_x: f64,
        press_y: f64,
    },
    TwoFingerSwipe {
        direction: SwipeDirection,
        distance: f64,
    },
}

/// Mathematical palm rejection filter distinguishing intended stylus writing from hand rests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PalmRejectionFilter {
    pub max_palm_radius: f64,
    pub max_palm_area: f64,
    pub stylus_proximity_mask_radius: f64,
    pub edge_exclusion_margin: f64,
}

impl Default for PalmRejectionFilter {
    fn default() -> Self {
        Self {
            max_palm_radius: 22.0,
            max_palm_area: 450.0,
            stylus_proximity_mask_radius: 120.0,
            edge_exclusion_margin: 15.0,
        }
    }
}

impl PalmRejectionFilter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates whether a touch contact constitutes palm resting contact or edge interference.
    pub fn is_palm(
        &self,
        touch: &TouchPoint,
        active_stylus: Option<&TouchPoint>,
        screen_width: f64,
        screen_height: f64,
    ) -> bool {
        // 1. Stylus contact is never rejected as palm
        if touch.is_stylus {
            return false;
        }

        // 2. Contact geometry check (large touch ellipse = resting palm / ball of hand)
        let major_radius = touch.radius_x.max(touch.radius_y);
        if major_radius > self.max_palm_radius || touch.contact_area() > self.max_palm_area {
            return true;
        }

        // 3. Stylus proximity masking: if an active stylus is on or near the screen,
        // coarse touch points in its vicinity are classified as the supporting palm.
        if let Some(stylus) = active_stylus {
            let dx = touch.x - stylus.x;
            let dy = touch.y - stylus.y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq <= self.stylus_proximity_mask_radius * self.stylus_proximity_mask_radius {
                return true;
            }
        }

        // 4. Edge exclusion rejection (fingers wrapped around tablet edges)
        if screen_width > 0.0
            && screen_height > 0.0
            && (touch.x < self.edge_exclusion_margin
                || touch.x > screen_width - self.edge_exclusion_margin
                || touch.y < self.edge_exclusion_margin
                || touch.y > screen_height - self.edge_exclusion_margin)
        {
            return true;
        }

        false
    }
}

/// Multi-touch stream processor and gesture classifier.
#[derive(Debug, Clone, Default)]
pub struct TouchGestureRecognizer {
    active_touches: HashMap<u64, TouchPoint>,
    initial_touch_positions: HashMap<u64, (f64, f64)>,
    initial_pinch_distance: Option<f64>,
    last_pinch_distance: Option<f64>,
    last_tap: Option<(f64, f64, u64)>,
    start_timestamp: Option<u64>,
}

impl TouchGestureRecognizer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn active_touch_count(&self) -> usize {
        self.active_touches.len()
    }

    pub fn touch_start(&mut self, point: TouchPoint) -> TouchGesture {
        self.initial_touch_positions
            .insert(point.id, (point.x, point.y));
        self.active_touches.insert(point.id, point);
        self.start_timestamp = self
            .start_timestamp
            .or_else(|| self.active_touches.values().map(|p| p.timestamp_ms).min());

        if self.active_touches.len() == 2 {
            let pts: Vec<&TouchPoint> = self.active_touches.values().collect();
            let dist = ((pts[0].x - pts[1].x).powi(2) + (pts[0].y - pts[1].y).powi(2)).sqrt();
            self.initial_pinch_distance = Some(dist);
            self.last_pinch_distance = Some(dist);
        } else {
            self.initial_pinch_distance = None;
            self.last_pinch_distance = None;
        }

        TouchGesture::None
    }

    pub fn touch_move(&mut self, point: TouchPoint) -> TouchGesture {
        let prev_pt = match self.active_touches.get(&point.id) {
            Some(p) => *p,
            None => {
                self.active_touches.insert(point.id, point);
                return TouchGesture::None;
            }
        };

        self.active_touches.insert(point.id, point);

        // 1. Two-finger Pinch-to-Zoom or Two-finger Scroll
        if self.active_touches.len() == 2 {
            let pts: Vec<&TouchPoint> = self.active_touches.values().collect();
            let current_dist =
                ((pts[0].x - pts[1].x).powi(2) + (pts[0].y - pts[1].y).powi(2)).sqrt();
            let focal_x = (pts[0].x + pts[1].x) / 2.0;
            let focal_y = (pts[0].y + pts[1].y) / 2.0;

            if let Some(last_dist) = self.last_pinch_distance {
                if last_dist > 1.0 {
                    let scale = current_dist / last_dist;
                    self.last_pinch_distance = Some(current_dist);
                    if (scale - 1.0).abs() > 0.015 {
                        return TouchGesture::PinchZoom {
                            scale,
                            focal_x,
                            focal_y,
                        };
                    }
                }
            }
            self.last_pinch_distance = Some(current_dist);
        }

        // 2. Single-finger Pan / Scroll
        if self.active_touches.len() == 1 {
            let dx = point.x - prev_pt.x;
            let dy = point.y - prev_pt.y;
            if dx.abs() > 1.5 || dy.abs() > 1.5 {
                return TouchGesture::PanScroll {
                    delta_x: dx,
                    delta_y: dy,
                };
            }
        }

        TouchGesture::None
    }

    pub fn touch_end(&mut self, id: u64, timestamp_ms: u64) -> TouchGesture {
        let ending_pt = self.active_touches.remove(&id);
        let initial_pos = self.initial_touch_positions.remove(&id);

        if self.active_touches.is_empty() {
            self.initial_pinch_distance = None;
            self.last_pinch_distance = None;
            self.start_timestamp = None;
        }

        if let (Some(end), Some((start_x, start_y))) = (ending_pt, initial_pos) {
            let dist = ((end.x - start_x).powi(2) + (end.y - start_y).powi(2)).sqrt();
            let duration = timestamp_ms.saturating_sub(end.timestamp_ms);

            // Double tap detection (2 taps within 300ms and 25px)
            if dist < 12.0 && duration < 350 {
                if let Some((lx, ly, lt)) = self.last_tap {
                    let tap_dist = ((end.x - lx).powi(2) + (end.y - ly).powi(2)).sqrt();
                    if tap_dist < 25.0 && timestamp_ms.saturating_sub(lt) < 320 {
                        self.last_tap = None;
                        return TouchGesture::DoubleTap {
                            tap_x: end.x,
                            tap_y: end.y,
                        };
                    }
                }
                self.last_tap = Some((end.x, end.y, timestamp_ms));
            } else if dist < 15.0 && duration >= 500 {
                return TouchGesture::LongPress {
                    press_x: end.x,
                    press_y: end.y,
                };
            }
        }

        TouchGesture::None
    }

    pub fn touch_cancel(&mut self, id: u64) {
        self.active_touches.remove(&id);
        self.initial_touch_positions.remove(&id);
        if self.active_touches.is_empty() {
            self.initial_pinch_distance = None;
            self.last_pinch_distance = None;
            self.start_timestamp = None;
        }
    }
}

/// Adaptive multi-pane split grid layout configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SplitGridMode {
    Single,
    TwoWayHorizontal,
    TwoWayVertical,
    FourWayGrid,
}

/// Adaptive split-grid manager computing responsive sub-pane rectangles and divider handles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdaptiveSplitGrid {
    pub mode: SplitGridMode,
    pub ratio_x: f64,
    pub ratio_y: f64,
    pub snap_threshold: f64,
}

impl Default for AdaptiveSplitGrid {
    fn default() -> Self {
        Self {
            mode: SplitGridMode::Single,
            ratio_x: 0.5,
            ratio_y: 0.5,
            snap_threshold: 0.04,
        }
    }
}

impl AdaptiveSplitGrid {
    pub fn new(mode: SplitGridMode) -> Self {
        Self {
            mode,
            ..Default::default()
        }
    }

    /// Sets division ratios with magnetic snap to common proportions (1/3, 1/2, 2/3).
    pub fn set_ratios_with_snap(&mut self, rx: f64, ry: f64) {
        self.ratio_x = Self::snap_ratio(rx.clamp(0.15, 0.85), self.snap_threshold);
        self.ratio_y = Self::snap_ratio(ry.clamp(0.15, 0.85), self.snap_threshold);
    }

    fn snap_ratio(val: f64, threshold: f64) -> f64 {
        const SNAP_POINTS: [f64; 3] = [0.333, 0.500, 0.667];
        for &snap in &SNAP_POINTS {
            if (val - snap).abs() <= threshold {
                return snap;
            }
        }
        val
    }

    /// Computes layout bounding boxes for all active sub-panes within total viewport rect.
    pub fn compute_subpane_rects(
        &self,
        total_rect: &Rect2D,
        insets: &SafeAreaInsets,
    ) -> Vec<Rect2D> {
        let safe_x = total_rect.x + insets.left;
        let safe_y = total_rect.y + insets.top;
        let safe_w = (total_rect.width - insets.left - insets.right).max(1.0);
        let safe_h = (total_rect.height - insets.top - insets.bottom).max(1.0);

        match self.mode {
            SplitGridMode::Single => {
                vec![Rect2D::new(safe_x, safe_y, safe_w, safe_h)]
            }
            SplitGridMode::TwoWayHorizontal => {
                let w1 = safe_w * self.ratio_x;
                let w2 = safe_w - w1;
                vec![
                    Rect2D::new(safe_x, safe_y, w1, safe_h),
                    Rect2D::new(safe_x + w1, safe_y, w2, safe_h),
                ]
            }
            SplitGridMode::TwoWayVertical => {
                let h1 = safe_h * self.ratio_y;
                let h2 = safe_h - h1;
                vec![
                    Rect2D::new(safe_x, safe_y, safe_w, h1),
                    Rect2D::new(safe_x, safe_y + h1, safe_w, h2),
                ]
            }
            SplitGridMode::FourWayGrid => {
                let w1 = safe_w * self.ratio_x;
                let w2 = safe_w - w1;
                let h1 = safe_h * self.ratio_y;
                let h2 = safe_h - h1;
                vec![
                    Rect2D::new(safe_x, safe_y, w1, h1),           // Top-Left
                    Rect2D::new(safe_x + w1, safe_y, w2, h1),      // Top-Right
                    Rect2D::new(safe_x, safe_y + h1, w1, h2),      // Bottom-Left
                    Rect2D::new(safe_x + w1, safe_y + h1, w2, h2), // Bottom-Right
                ]
            }
        }
    }
}
