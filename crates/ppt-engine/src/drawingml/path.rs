#![forbid(unsafe_code)]
//! DrawingML Vector Path Model & Normalization
//!
//! Provides normalized vector path commands:
//! - `MoveTo`
//! - `LineTo`
//! - `QuadTo` (Quadratic Bezier)
//! - `CubicTo` (Cubic Bezier)
//! - `ArcTo` (DrawingML Elliptical Arc)
//! - `Close`
//!
//! Includes conversion from DrawingML elliptical arcs to cubic Beziers,
//! SVG path data serialization, bounding box evaluation, and hit testing.

use serde::{Deserialize, Serialize};

/// 2D Floating Point Coordinate
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// Fill mode for a DrawingML path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PathFillMode {
    #[default]
    Norm,
    None,
    Lighten,
    LightenLess,
    Darken,
    DarkenLess,
}

/// Normalized Path Command
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PathCommand {
    MoveTo(Point2D),
    LineTo(Point2D),
    QuadTo {
        ctrl: Point2D,
        to: Point2D,
    },
    CubicTo {
        ctrl1: Point2D,
        ctrl2: Point2D,
        to: Point2D,
    },
    ArcTo {
        w_r: f64,
        h_r: f64,
        start_angle_deg: f64,
        swing_angle_deg: f64,
    },
    Close,
}

/// Evaluated vector path ready for rasterization, tessellation, or SVG export.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DrawingPath {
    pub commands: Vec<PathCommand>,
    pub fill: PathFillMode,
    pub stroke: bool,
    pub width: Option<i64>,
    pub height: Option<i64>,
}

impl DrawingPath {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            fill: PathFillMode::Norm,
            stroke: true,
            width: None,
            height: None,
        }
    }

    pub fn move_to(&mut self, x: f64, y: f64) {
        self.commands.push(PathCommand::MoveTo(Point2D::new(x, y)));
    }

    pub fn line_to(&mut self, x: f64, y: f64) {
        self.commands.push(PathCommand::LineTo(Point2D::new(x, y)));
    }

    pub fn quad_to(&mut self, cx: f64, cy: f64, x: f64, y: f64) {
        self.commands.push(PathCommand::QuadTo {
            ctrl: Point2D::new(cx, cy),
            to: Point2D::new(x, y),
        });
    }

    pub fn cubic_to(&mut self, c1x: f64, c1y: f64, c2x: f64, c2y: f64, x: f64, y: f64) {
        self.commands.push(PathCommand::CubicTo {
            ctrl1: Point2D::new(c1x, c1y),
            ctrl2: Point2D::new(c2x, c2y),
            to: Point2D::new(x, y),
        });
    }

    pub fn arc_to(&mut self, w_r: f64, h_r: f64, start_angle_deg: f64, swing_angle_deg: f64) {
        self.commands.push(PathCommand::ArcTo {
            w_r,
            h_r,
            start_angle_deg,
            swing_angle_deg,
        });
    }

    pub fn close(&mut self) {
        self.commands.push(PathCommand::Close);
    }

    /// Convert all `ArcTo` commands into equivalent cubic Bezier curves.
    /// This produces a purely Bezier path (`MoveTo`, `LineTo`, `CubicTo`, `QuadTo`, `Close`).
    pub fn to_cubic_beziers(&self) -> DrawingPath {
        let mut new_path = DrawingPath {
            commands: Vec::with_capacity(self.commands.len()),
            fill: self.fill,
            stroke: self.stroke,
            width: self.width,
            height: self.height,
        };

        let mut current_point = Point2D::new(0.0, 0.0);

        for cmd in &self.commands {
            match *cmd {
                PathCommand::MoveTo(pt) => {
                    current_point = pt;
                    new_path.commands.push(PathCommand::MoveTo(pt));
                }
                PathCommand::LineTo(pt) => {
                    current_point = pt;
                    new_path.commands.push(PathCommand::LineTo(pt));
                }
                PathCommand::QuadTo { ctrl, to } => {
                    // Convert quadratic Bezier to cubic Bezier
                    // CP1 = current + 2/3 * (ctrl - current)
                    // CP2 = to + 2/3 * (ctrl - to)
                    let c1x = current_point.x + (2.0 / 3.0) * (ctrl.x - current_point.x);
                    let c1y = current_point.y + (2.0 / 3.0) * (ctrl.y - current_point.y);
                    let c2x = to.x + (2.0 / 3.0) * (ctrl.x - to.x);
                    let c2y = to.y + (2.0 / 3.0) * (ctrl.y - to.y);
                    current_point = to;
                    new_path.commands.push(PathCommand::CubicTo {
                        ctrl1: Point2D::new(c1x, c1y),
                        ctrl2: Point2D::new(c2x, c2y),
                        to,
                    });
                }
                PathCommand::CubicTo { ctrl1, ctrl2, to } => {
                    current_point = to;
                    new_path
                        .commands
                        .push(PathCommand::CubicTo { ctrl1, ctrl2, to });
                }
                PathCommand::ArcTo {
                    w_r,
                    h_r,
                    start_angle_deg,
                    swing_angle_deg,
                } => {
                    // In DrawingML, the current point is at start_angle on the ellipse:
                    // cx = current_point.x - w_r * cos(stAng)
                    // cy = current_point.y - h_r * sin(stAng)
                    let st_rad = start_angle_deg.to_radians();
                    let cx = current_point.x - w_r * st_rad.cos();
                    let cy = current_point.y - h_r * st_rad.sin();

                    let beziers =
                        arc_to_beziers(cx, cy, w_r, h_r, start_angle_deg, swing_angle_deg);
                    for b in beziers {
                        current_point = b.to;
                        new_path.commands.push(PathCommand::CubicTo {
                            ctrl1: b.ctrl1,
                            ctrl2: b.ctrl2,
                            to: b.to,
                        });
                    }
                }
                PathCommand::Close => {
                    new_path.commands.push(PathCommand::Close);
                }
            }
        }

        new_path
    }

    /// Compute axis-aligned bounding box: `(min_x, min_y, max_x, max_y)`.
    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        if self.commands.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let flattened = self.to_cubic_beziers();
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        let mut update = |pt: Point2D| {
            if pt.x < min_x {
                min_x = pt.x;
            }
            if pt.x > max_x {
                max_x = pt.x;
            }
            if pt.y < min_y {
                min_y = pt.y;
            }
            if pt.y > max_y {
                max_y = pt.y;
            }
        };

        for cmd in &flattened.commands {
            match *cmd {
                PathCommand::MoveTo(pt) | PathCommand::LineTo(pt) => update(pt),
                PathCommand::QuadTo { ctrl, to } => {
                    update(ctrl);
                    update(to);
                }
                PathCommand::CubicTo { ctrl1, ctrl2, to } => {
                    update(ctrl1);
                    update(ctrl2);
                    update(to);
                }
                _ => {}
            }
        }

        if min_x.is_infinite() {
            (0.0, 0.0, 0.0, 0.0)
        } else {
            (min_x, min_y, max_x, max_y)
        }
    }

    /// Serialize to standard SVG path data string (`d="..."`).
    pub fn to_svg_path_data(&self) -> String {
        let mut s = String::new();
        for cmd in &self.commands {
            match *cmd {
                PathCommand::MoveTo(pt) => {
                    s.push_str(&format!("M {:.2} {:.2} ", pt.x, pt.y));
                }
                PathCommand::LineTo(pt) => {
                    s.push_str(&format!("L {:.2} {:.2} ", pt.x, pt.y));
                }
                PathCommand::QuadTo { ctrl, to } => {
                    s.push_str(&format!(
                        "Q {:.2} {:.2} {:.2} {:.2} ",
                        ctrl.x, ctrl.y, to.x, to.y
                    ));
                }
                PathCommand::CubicTo { ctrl1, ctrl2, to } => {
                    s.push_str(&format!(
                        "C {:.2} {:.2} {:.2} {:.2} {:.2} {:.2} ",
                        ctrl1.x, ctrl1.y, ctrl2.x, ctrl2.y, to.x, to.y
                    ));
                }
                PathCommand::ArcTo {
                    w_r,
                    h_r,
                    start_angle_deg,
                    swing_angle_deg,
                } => {
                    // SVG Arc command format: A rx ry x-axis-rotation large-arc-flag sweep-flag x y
                    let st_rad = start_angle_deg.to_radians();
                    let end_deg = start_angle_deg + swing_angle_deg;
                    let end_rad = end_deg.to_radians();

                    // Calculate end point relative to center
                    // We assume last point is (cx + w_r * cos(st), cy + h_r * sin(st))
                    let dx = w_r * (end_rad.cos() - st_rad.cos());
                    let dy = h_r * (end_rad.sin() - st_rad.sin());

                    let large_arc = if swing_angle_deg.abs() > 180.0 { 1 } else { 0 };
                    let sweep = if swing_angle_deg >= 0.0 { 1 } else { 0 };

                    s.push_str(&format!(
                        "a {:.2} {:.2} 0 {} {} {:.2} {:.2} ",
                        w_r, h_r, large_arc, sweep, dx, dy
                    ));
                }
                PathCommand::Close => {
                    s.push_str("Z ");
                }
            }
        }
        s.trim_end().to_string()
    }

    /// Point-in-polygon hit test using the standard even-odd ray casting algorithm.
    pub fn contains_point(&self, target: Point2D) -> bool {
        let flattened = self.to_cubic_beziers();
        let mut segments: Vec<(Point2D, Point2D)> = Vec::new();
        let mut curr = Point2D::new(0.0, 0.0);
        let mut start = Point2D::new(0.0, 0.0);

        for cmd in &flattened.commands {
            match *cmd {
                PathCommand::MoveTo(pt) => {
                    curr = pt;
                    start = pt;
                }
                PathCommand::LineTo(pt) => {
                    segments.push((curr, pt));
                    curr = pt;
                }
                PathCommand::CubicTo { ctrl1, ctrl2, to } => {
                    // Approximate cubic with 8 linear segments for hit test
                    let n = 8;
                    let mut prev = curr;
                    for step in 1..=n {
                        let t = step as f64 / n as f64;
                        let inv_t = 1.0 - t;
                        let x = inv_t.powi(3) * curr.x
                            + 3.0 * inv_t.powi(2) * t * ctrl1.x
                            + 3.0 * inv_t * t.powi(2) * ctrl2.x
                            + t.powi(3) * to.x;
                        let y = inv_t.powi(3) * curr.y
                            + 3.0 * inv_t.powi(2) * t * ctrl1.y
                            + 3.0 * inv_t * t.powi(2) * ctrl2.y
                            + t.powi(3) * to.y;
                        let next = Point2D::new(x, y);
                        segments.push((prev, next));
                        prev = next;
                    }
                    curr = to;
                }
                PathCommand::Close => {
                    if (curr.x - start.x).abs() > 1e-6 || (curr.y - start.y).abs() > 1e-6 {
                        segments.push((curr, start));
                    }
                    curr = start;
                }
                _ => {}
            }
        }

        let mut inside = false;
        for (p1, p2) in segments {
            if (p1.y > target.y) != (p2.y > target.y) {
                let x_intersection = (p2.x - p1.x) * (target.y - p1.y) / (p2.y - p1.y) + p1.x;
                if target.x < x_intersection {
                    inside = !inside;
                }
            }
        }

        inside
    }
}

/// Helper struct representing a cubic Bezier segment for arc approximation.
struct CubicSegment {
    ctrl1: Point2D,
    ctrl2: Point2D,
    to: Point2D,
}

/// Subdivides an elliptical arc into one or more cubic Beziers (each <= 90 degrees).
fn arc_to_beziers(
    cx: f64,
    cy: f64,
    w_r: f64,
    h_r: f64,
    start_angle_deg: f64,
    swing_angle_deg: f64,
) -> Vec<CubicSegment> {
    let mut segments = Vec::new();
    let num_segments = (swing_angle_deg.abs() / 90.0).ceil() as usize;
    if num_segments == 0 {
        return segments;
    }

    let segment_angle = swing_angle_deg / num_segments as f64;
    let mut current_angle = start_angle_deg;

    for _ in 0..num_segments {
        let next_angle = current_angle + segment_angle;
        let seg = single_arc_to_bezier(cx, cy, w_r, h_r, current_angle, next_angle);
        segments.push(seg);
        current_angle = next_angle;
    }

    segments
}

/// Approximates a single arc segment (span <= 90 deg) with a single cubic Bezier.
fn single_arc_to_bezier(
    cx: f64,
    cy: f64,
    w_r: f64,
    h_r: f64,
    ang1_deg: f64,
    ang2_deg: f64,
) -> CubicSegment {
    let rad1 = ang1_deg.to_radians();
    let rad2 = ang2_deg.to_radians();
    let delta = rad2 - rad1;
    let alpha = delta / 2.0;

    let k = (4.0 / 3.0) * ((alpha / 2.0).tan());

    let p1x = cx + w_r * rad1.cos();
    let p1y = cy + h_r * rad1.sin();

    let p4x = cx + w_r * rad2.cos();
    let p4y = cy + h_r * rad2.sin();

    let c1x = p1x - k * w_r * rad1.sin();
    let c1y = p1y + k * h_r * rad1.cos();

    let c2x = p4x + k * w_r * rad2.sin();
    let c2y = p4y - k * h_r * rad2.cos();

    CubicSegment {
        ctrl1: Point2D::new(c1x, c1y),
        ctrl2: Point2D::new(c2x, c2y),
        to: Point2D::new(p4x, p4y),
    }
}
