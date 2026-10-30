#![forbid(unsafe_code)]
//! 3D Shader Transitions & Keyframe Motion Paths Engine (Subphase 19.1.2)
//!
//! Provides:
//! - Hardware-accelerated 3D slide transitions (Cube, Flip, Vortex, Honeycomb, Origami, Ripple)
//! - 4x4 3D transform matrices with CSS `matrix3d(...)` projection
//! - Easing curves: Linear, EaseInOut, CubicBezier, Bounce, and Spring
//! - DrawingML `<animMotion>` Bezier motion path trajectory compiler with rotation tracking

use crate::animation::easing::solve_cubic_bezier_easing;
use serde::{Deserialize, Serialize};

/// 3D transition effects supported in slide decks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Transition3DEffect {
    Cube,
    Flip,
    Vortex,
    Honeycomb,
    Origami,
    Ripple,
}

impl Transition3DEffect {
    pub fn from_str_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "cube" => Some(Self::Cube),
            "flip" => Some(Self::Flip),
            "vortex" => Some(Self::Vortex),
            "honeycomb" => Some(Self::Honeycomb),
            "origami" => Some(Self::Origami),
            "ripple" => Some(Self::Ripple),
            _ => None,
        }
    }

    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Cube => "Cube",
            Self::Flip => "Flip",
            Self::Vortex => "Vortex",
            Self::Honeycomb => "Honeycomb",
            Self::Origami => "Origami",
            Self::Ripple => "Ripple",
        }
    }
}

/// Transition spatial direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Transition3DDirection {
    #[default]
    Right,
    Left,
    Up,
    Down,
}

impl Transition3DDirection {
    pub fn from_str_name(dir: &str) -> Self {
        match dir.to_ascii_lowercase().as_str() {
            "left" => Self::Left,
            "right" => Self::Right,
            "up" | "top" => Self::Up,
            "down" | "bottom" => Self::Down,
            _ => Self::Right,
        }
    }
}

/// 4x4 Matrix for 3D Affine Transformations (stored in column-major order for WebGL / CSS matrix3d).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform3DMatrix {
    pub m: [f64; 16],
}

impl Default for Transform3DMatrix {
    fn default() -> Self {
        Self::identity()
    }
}

impl Transform3DMatrix {
    /// Standard 4x4 Identity Matrix.
    pub fn identity() -> Self {
        Self {
            m: [
                1.0, 0.0, 0.0, 0.0, // Col 0
                0.0, 1.0, 0.0, 0.0, // Col 1
                0.0, 0.0, 1.0, 0.0, // Col 2
                0.0, 0.0, 0.0, 1.0, // Col 3
            ],
        }
    }

    /// Translation matrix.
    pub fn translation(x: f64, y: f64, z: f64) -> Self {
        Self {
            m: [
                1.0, 0.0, 0.0, 0.0, //
                0.0, 1.0, 0.0, 0.0, //
                0.0, 0.0, 1.0, 0.0, //
                x, y, z, 1.0, //
            ],
        }
    }

    /// Scale matrix.
    pub fn scale(sx: f64, sy: f64, sz: f64) -> Self {
        Self {
            m: [
                sx, 0.0, 0.0, 0.0, //
                0.0, sy, 0.0, 0.0, //
                0.0, 0.0, sz, 0.0, //
                0.0, 0.0, 0.0, 1.0, //
            ],
        }
    }

    /// Rotation around X-axis (in degrees).
    pub fn rotation_x(deg: f64) -> Self {
        let rad = deg.to_radians();
        let (s, c) = (rad.sin(), rad.cos());
        Self {
            m: [
                1.0, 0.0, 0.0, 0.0, //
                0.0, c, s, 0.0, //
                0.0, -s, c, 0.0, //
                0.0, 0.0, 0.0, 1.0, //
            ],
        }
    }

    /// Rotation around Y-axis (in degrees).
    pub fn rotation_y(deg: f64) -> Self {
        let rad = deg.to_radians();
        let (s, c) = (rad.sin(), rad.cos());
        Self {
            m: [
                c, 0.0, -s, 0.0, //
                0.0, 1.0, 0.0, 0.0, //
                s, 0.0, c, 0.0, //
                0.0, 0.0, 0.0, 1.0, //
            ],
        }
    }

    /// Rotation around Z-axis (in degrees).
    pub fn rotation_z(deg: f64) -> Self {
        let rad = deg.to_radians();
        let (s, c) = (rad.sin(), rad.cos());
        Self {
            m: [
                c, s, 0.0, 0.0, //
                -s, c, 0.0, 0.0, //
                0.0, 0.0, 1.0, 0.0, //
                0.0, 0.0, 0.0, 1.0, //
            ],
        }
    }

    /// Perspective projection matrix with focal distance `d`.
    pub fn perspective(d: f64) -> Self {
        let p = if d.abs() > 1e-4 { -1.0 / d } else { 0.0 };
        Self {
            m: [
                1.0, 0.0, 0.0, 0.0, //
                0.0, 1.0, 0.0, 0.0, //
                0.0, 0.0, 1.0, p, //
                0.0, 0.0, 0.0, 1.0, //
            ],
        }
    }

    /// Multiplies `self` with `rhs` (i.e. applies `rhs` then `self`).
    pub fn multiply(&self, rhs: &Self) -> Self {
        let mut out = [0.0; 16];
        for col in 0..4 {
            for row in 0..4 {
                let mut sum = 0.0;
                for k in 0..4 {
                    sum += self.m[k * 4 + row] * rhs.m[col * 4 + k];
                }
                out[col * 4 + row] = sum;
            }
        }
        Self { m: out }
    }

    /// Serializes matrix to standard CSS `matrix3d(m0, m1, ... m15)`.
    pub fn to_css_matrix3d(&self) -> String {
        let entries: Vec<String> = self
            .m
            .iter()
            .map(|&val| {
                if val.abs() < 1e-7 {
                    "0".to_string()
                } else {
                    format!("{:.6}", val)
                        .trim_end_matches('0')
                        .trim_end_matches('.')
                        .to_string()
                }
            })
            .collect();
        format!("matrix3d({})", entries.join(", "))
    }
}

/// Dynamic interpolated 3D transform pair for transition animation frame.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transition3DFrame {
    pub outgoing_transform: Transform3DMatrix,
    pub outgoing_opacity: f64,
    pub incoming_transform: Transform3DMatrix,
    pub incoming_opacity: f64,
}

/// Slide Transition 3D Engine computing transform matrices and opacities.
pub struct SlideTransition3DEngine;

impl SlideTransition3DEngine {
    /// Computes outgoing and incoming 3D matrices for any progress $t \in [0.0, 1.0]$.
    pub fn evaluate_transition(
        effect: Transition3DEffect,
        direction: Transition3DDirection,
        progress: f64,
        viewport_w: f64,
        viewport_h: f64,
    ) -> Transition3DFrame {
        let t = progress.clamp(0.0, 1.0);
        let half_w = viewport_w / 2.0;
        let half_h = viewport_h / 2.0;
        let pers = 1200.0;

        match effect {
            Transition3DEffect::Cube => {
                let (sign, is_horizontal) = match direction {
                    Transition3DDirection::Right => (-1.0, true),
                    Transition3DDirection::Left => (1.0, true),
                    Transition3DDirection::Down => (-1.0, false),
                    Transition3DDirection::Up => (1.0, false),
                };

                let p_mat = Transform3DMatrix::perspective(pers);
                let radius = if is_horizontal { half_w } else { half_h };

                if is_horizontal {
                    // Outgoing: 0 to sign * 90 deg
                    let out_rot = sign * 90.0 * t;
                    let out_mat = p_mat
                        .multiply(&Transform3DMatrix::translation(0.0, 0.0, -radius))
                        .multiply(&Transform3DMatrix::rotation_y(out_rot))
                        .multiply(&Transform3DMatrix::translation(0.0, 0.0, radius));

                    // Incoming: -sign * 90 to 0 deg
                    let in_rot = -sign * 90.0 * (1.0 - t);
                    let in_mat = p_mat
                        .multiply(&Transform3DMatrix::translation(0.0, 0.0, -radius))
                        .multiply(&Transform3DMatrix::rotation_y(in_rot))
                        .multiply(&Transform3DMatrix::translation(0.0, 0.0, radius));

                    Transition3DFrame {
                        outgoing_transform: out_mat,
                        outgoing_opacity: 1.0 - t * 0.2,
                        incoming_transform: in_mat,
                        incoming_opacity: 0.8 + t * 0.2,
                    }
                } else {
                    let out_rot = sign * 90.0 * t;
                    let out_mat = p_mat
                        .multiply(&Transform3DMatrix::translation(0.0, 0.0, -radius))
                        .multiply(&Transform3DMatrix::rotation_x(out_rot))
                        .multiply(&Transform3DMatrix::translation(0.0, 0.0, radius));

                    let in_rot = -sign * 90.0 * (1.0 - t);
                    let in_mat = p_mat
                        .multiply(&Transform3DMatrix::translation(0.0, 0.0, -radius))
                        .multiply(&Transform3DMatrix::rotation_x(in_rot))
                        .multiply(&Transform3DMatrix::translation(0.0, 0.0, radius));

                    Transition3DFrame {
                        outgoing_transform: out_mat,
                        outgoing_opacity: 1.0 - t * 0.2,
                        incoming_transform: in_mat,
                        incoming_opacity: 0.8 + t * 0.2,
                    }
                }
            }

            Transition3DEffect::Flip => {
                let sign = match direction {
                    Transition3DDirection::Left | Transition3DDirection::Up => -1.0,
                    _ => 1.0,
                };
                let p_mat = Transform3DMatrix::perspective(pers);
                let angle = sign * 180.0 * t;

                let out_mat = p_mat.multiply(&Transform3DMatrix::rotation_y(angle));
                let in_mat = p_mat.multiply(&Transform3DMatrix::rotation_y(angle - sign * 180.0));

                let out_opacity = if t < 0.5 { 1.0 } else { 0.0 };
                let in_opacity = if t >= 0.5 { 1.0 } else { 0.0 };

                Transition3DFrame {
                    outgoing_transform: out_mat,
                    outgoing_opacity: out_opacity,
                    incoming_transform: in_mat,
                    incoming_opacity: in_opacity,
                }
            }

            Transition3DEffect::Vortex => {
                let p_mat = Transform3DMatrix::perspective(pers);
                // Outgoing spirals back into depth
                let out_scale = (1.0 - t * 0.8).max(0.01);
                let out_rot = 720.0 * t;
                let out_z = -800.0 * t;
                let out_mat = p_mat
                    .multiply(&Transform3DMatrix::translation(0.0, 0.0, out_z))
                    .multiply(&Transform3DMatrix::rotation_z(out_rot))
                    .multiply(&Transform3DMatrix::scale(out_scale, out_scale, 1.0));

                // Incoming uncoils forward into view
                let in_scale = (t * 0.8 + 0.2).min(1.0);
                let in_rot = -720.0 * (1.0 - t);
                let in_z = -800.0 * (1.0 - t);
                let in_mat = p_mat
                    .multiply(&Transform3DMatrix::translation(0.0, 0.0, in_z))
                    .multiply(&Transform3DMatrix::rotation_z(in_rot))
                    .multiply(&Transform3DMatrix::scale(in_scale, in_scale, 1.0));

                Transition3DFrame {
                    outgoing_transform: out_mat,
                    outgoing_opacity: (1.0 - t).max(0.0),
                    incoming_transform: in_mat,
                    incoming_opacity: t,
                }
            }

            Transition3DEffect::Honeycomb => {
                let p_mat = Transform3DMatrix::perspective(pers);
                let out_scale = 1.0 + t * 0.6;
                let out_rot_y = 45.0 * t;
                let out_mat = p_mat
                    .multiply(&Transform3DMatrix::rotation_y(out_rot_y))
                    .multiply(&Transform3DMatrix::scale(out_scale, out_scale, 1.0));

                let in_scale = 0.7 + t * 0.3;
                let in_rot_y = -45.0 * (1.0 - t);
                let in_mat = p_mat
                    .multiply(&Transform3DMatrix::rotation_y(in_rot_y))
                    .multiply(&Transform3DMatrix::scale(in_scale, in_scale, 1.0));

                Transition3DFrame {
                    outgoing_transform: out_mat,
                    outgoing_opacity: (1.0 - t).max(0.0),
                    incoming_transform: in_mat,
                    incoming_opacity: t,
                }
            }

            Transition3DEffect::Origami => {
                let p_mat = Transform3DMatrix::perspective(pers);
                // Diagonal fold rotation
                let out_rot = 90.0 * t;
                let out_mat = p_mat
                    .multiply(&Transform3DMatrix::translation(
                        -half_w * t,
                        -half_h * t,
                        -200.0 * t,
                    ))
                    .multiply(&Transform3DMatrix::rotation_z(30.0 * t))
                    .multiply(&Transform3DMatrix::rotation_y(out_rot));

                let in_mat =
                    p_mat.multiply(&Transform3DMatrix::scale(0.8 + 0.2 * t, 0.8 + 0.2 * t, 1.0));

                Transition3DFrame {
                    outgoing_transform: out_mat,
                    outgoing_opacity: (1.0 - t).max(0.0),
                    incoming_transform: in_mat,
                    incoming_opacity: t,
                }
            }

            Transition3DEffect::Ripple => {
                let p_mat = Transform3DMatrix::perspective(pers);
                let wave = (t * std::f64::consts::PI * 4.0).sin() * 0.08 * (1.0 - t);
                let out_scale = 1.0 + wave;
                let out_mat = p_mat.multiply(&Transform3DMatrix::scale(out_scale, out_scale, 1.0));

                let in_scale = 0.95 + 0.05 * t - wave;
                let in_mat = p_mat.multiply(&Transform3DMatrix::scale(in_scale, in_scale, 1.0));

                Transition3DFrame {
                    outgoing_transform: out_mat,
                    outgoing_opacity: (1.0 - t).max(0.0),
                    incoming_transform: in_mat,
                    incoming_opacity: t,
                }
            }
        }
    }
}

/// Easing dynamics for animations and transitions.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum EasingCurve {
    Linear,
    #[default]
    EaseInOut,
    CubicBezier {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
    },
    Bounce,
    Spring {
        stiffness: f64,
        damping: f64,
    },
}

impl EasingCurve {
    /// Evaluates easing curve at progress $t \in [0.0, 1.0]$.
    pub fn evaluate(&self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::EaseInOut => solve_cubic_bezier_easing(0.42, 0.0, 0.58, 1.0, t),
            Self::CubicBezier { x1, y1, x2, y2 } => {
                solve_cubic_bezier_easing(*x1, *y1, *x2, *y2, t)
            }
            Self::Bounce => {
                let n1 = 7.5625;
                let d1 = 2.75;
                if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    let t2 = t - 1.5 / d1;
                    n1 * t2 * t2 + 0.75
                } else if t < 2.5 / d1 {
                    let t3 = t - 2.25 / d1;
                    n1 * t3 * t3 + 0.9375
                } else {
                    let t4 = t - 2.625 / d1;
                    n1 * t4 * t4 + 0.984375
                }
            }
            Self::Spring { stiffness, damping } => {
                let omega = (*stiffness).max(1.0).sqrt();
                let gamma = (*damping).max(0.1);
                // Damped oscillation formula approaching 1.0
                let decay = (-gamma * t * 4.0).exp();
                let osc = (omega * t * 2.0 * std::f64::consts::PI).cos();
                1.0 - decay * osc
            }
        }
    }
}

/// A discrete trajectory point sampled along a motion path.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MotionTrajectoryPoint {
    pub progress: f64,
    pub x: f64,
    pub y: f64,
    pub tangent_angle_deg: f64,
}

/// Motion Path Bezier Segment representation.
#[derive(Debug, Clone, PartialEq)]
enum PathSegment {
    Line {
        p0: (f64, f64),
        p1: (f64, f64),
    },
    Quadratic {
        p0: (f64, f64),
        ctrl: (f64, f64),
        p1: (f64, f64),
    },
    Cubic {
        p0: (f64, f64),
        c1: (f64, f64),
        c2: (f64, f64),
        p1: (f64, f64),
    },
}

impl PathSegment {
    fn eval(&self, u: f64) -> ((f64, f64), (f64, f64)) {
        let u = u.clamp(0.0, 1.0);
        match self {
            Self::Line { p0, p1 } => {
                let x = p0.0 + (p1.0 - p0.0) * u;
                let y = p0.1 + (p1.1 - p0.1) * u;
                let dx = p1.0 - p0.0;
                let dy = p1.1 - p0.1;
                ((x, y), (dx, dy))
            }
            Self::Quadratic { p0, ctrl, p1 } => {
                let inv = 1.0 - u;
                let x = inv * inv * p0.0 + 2.0 * inv * u * ctrl.0 + u * u * p1.0;
                let y = inv * inv * p0.1 + 2.0 * inv * u * ctrl.1 + u * u * p1.1;
                let dx = 2.0 * inv * (ctrl.0 - p0.0) + 2.0 * u * (p1.0 - ctrl.0);
                let dy = 2.0 * inv * (ctrl.1 - p0.1) + 2.0 * u * (p1.1 - ctrl.1);
                ((x, y), (dx, dy))
            }
            Self::Cubic { p0, c1, c2, p1 } => {
                let inv = 1.0 - u;
                let x = inv * inv * inv * p0.0
                    + 3.0 * inv * inv * u * c1.0
                    + 3.0 * inv * u * u * c2.0
                    + u * u * u * p1.0;
                let y = inv * inv * inv * p0.1
                    + 3.0 * inv * inv * u * c1.1
                    + 3.0 * inv * u * u * c2.1
                    + u * u * u * p1.1;
                let dx = 3.0 * inv * inv * (c1.0 - p0.0)
                    + 6.0 * inv * u * (c2.0 - c1.0)
                    + 3.0 * u * u * (p1.0 - c2.0);
                let dy = 3.0 * inv * inv * (c1.1 - p0.1)
                    + 6.0 * inv * u * (c2.1 - c1.1)
                    + 3.0 * u * u * (p1.1 - c2.1);
                ((x, y), (dx, dy))
            }
        }
    }

    fn approx_length(&self) -> f64 {
        let (p_start, _) = self.eval(0.0);
        let (p_mid, _) = self.eval(0.5);
        let (p_end, _) = self.eval(1.0);
        let d1 = ((p_mid.0 - p_start.0).powi(2) + (p_mid.1 - p_start.1).powi(2)).sqrt();
        let d2 = ((p_end.0 - p_mid.0).powi(2) + (p_end.1 - p_mid.1).powi(2)).sqrt();
        d1 + d2
    }
}

/// Motion Path Compiler compiling DrawingML `<animMotion>` path syntax into smooth keyframes.
pub struct MotionPathCompiler;

impl MotionPathCompiler {
    /// Compiles an SVG/DrawingML motion path string into a discrete trajectory with easing.
    pub fn compile_trajectory(
        path_str: &str,
        easing: &EasingCurve,
        samples: usize,
        rotate_to_path: bool,
    ) -> Vec<MotionTrajectoryPoint> {
        let segments = Self::parse_segments(path_str);
        if segments.is_empty() {
            return vec![MotionTrajectoryPoint {
                progress: 0.0,
                x: 0.0,
                y: 0.0,
                tangent_angle_deg: 0.0,
            }];
        }

        let mut seg_lengths = Vec::with_capacity(segments.len());
        let mut total_length = 0.0;
        for s in &segments {
            let l = s.approx_length();
            seg_lengths.push(l);
            total_length += l;
        }

        let n = samples.max(2);
        let mut trajectory = Vec::with_capacity(n);

        for step in 0..n {
            let norm_t = step as f64 / (n - 1) as f64;
            let eased_t = easing.evaluate(norm_t);

            let (x, y, angle) =
                Self::sample_at_distance(&segments, &seg_lengths, total_length, eased_t);

            trajectory.push(MotionTrajectoryPoint {
                progress: norm_t,
                x,
                y,
                tangent_angle_deg: if rotate_to_path { angle } else { 0.0 },
            });
        }

        trajectory
    }

    fn sample_at_distance(
        segments: &[PathSegment],
        lengths: &[f64],
        total_length: f64,
        t: f64,
    ) -> (f64, f64, f64) {
        if total_length <= 1e-6 {
            let (p, d) = segments[0].eval(0.0);
            let angle = d.1.atan2(d.0).to_degrees();
            return (p.0, p.1, angle);
        }

        let target_dist = (t * total_length).clamp(0.0, total_length);
        let mut acc = 0.0;

        for (i, seg) in segments.iter().enumerate() {
            let seg_len = lengths[i];
            if acc + seg_len >= target_dist || i == segments.len() - 1 {
                let u = if seg_len > 1e-6 {
                    (target_dist - acc) / seg_len
                } else {
                    0.0
                };
                let (pt, deriv) = seg.eval(u);
                let angle = deriv.1.atan2(deriv.0).to_degrees();
                return (pt.0, pt.1, angle);
            }
            acc += seg_len;
        }

        let (last_pt, last_deriv) = segments[segments.len() - 1].eval(1.0);
        (
            last_pt.0,
            last_pt.1,
            last_deriv.1.atan2(last_deriv.0).to_degrees(),
        )
    }

    /// Parses tokens into line, quadratic, and cubic segments.
    fn parse_segments(path_str: &str) -> Vec<PathSegment> {
        let mut segments = Vec::new();
        let tokens: Vec<&str> = path_str
            .split(|c: char| c.is_whitespace() || c == ',')
            .filter(|s| !s.is_empty())
            .collect();

        let mut i = 0;
        let mut curr = (0.0, 0.0);
        let mut start = (0.0, 0.0);

        while i < tokens.len() {
            let cmd = tokens[i];
            match cmd {
                "M" | "m" => {
                    if i + 2 < tokens.len() {
                        let x = tokens[i + 1].parse().unwrap_or(curr.0);
                        let y = tokens[i + 2].parse().unwrap_or(curr.1);
                        curr = (x, y);
                        start = curr;
                        i += 3;
                    } else {
                        break;
                    }
                }
                "L" | "l" => {
                    if i + 2 < tokens.len() {
                        let x = tokens[i + 1].parse().unwrap_or(curr.0);
                        let y = tokens[i + 2].parse().unwrap_or(curr.1);
                        let next = (x, y);
                        segments.push(PathSegment::Line { p0: curr, p1: next });
                        curr = next;
                        i += 3;
                    } else {
                        break;
                    }
                }
                "Q" | "q" => {
                    if i + 4 < tokens.len() {
                        let cx = tokens[i + 1].parse().unwrap_or(curr.0);
                        let cy = tokens[i + 2].parse().unwrap_or(curr.1);
                        let x = tokens[i + 3].parse().unwrap_or(curr.0);
                        let y = tokens[i + 4].parse().unwrap_or(curr.1);
                        let ctrl = (cx, cy);
                        let next = (x, y);
                        segments.push(PathSegment::Quadratic {
                            p0: curr,
                            ctrl,
                            p1: next,
                        });
                        curr = next;
                        i += 5;
                    } else {
                        break;
                    }
                }
                "C" | "c" => {
                    if i + 6 < tokens.len() {
                        let c1x = tokens[i + 1].parse().unwrap_or(curr.0);
                        let c1y = tokens[i + 2].parse().unwrap_or(curr.1);
                        let c2x = tokens[i + 3].parse().unwrap_or(curr.0);
                        let c2y = tokens[i + 4].parse().unwrap_or(curr.1);
                        let x = tokens[i + 5].parse().unwrap_or(curr.0);
                        let y = tokens[i + 6].parse().unwrap_or(curr.1);
                        let c1 = (c1x, c1y);
                        let c2 = (c2x, c2y);
                        let next = (x, y);
                        segments.push(PathSegment::Cubic {
                            p0: curr,
                            c1,
                            c2,
                            p1: next,
                        });
                        curr = next;
                        i += 7;
                    } else {
                        break;
                    }
                }
                "Z" | "z" => {
                    if curr != start {
                        segments.push(PathSegment::Line {
                            p0: curr,
                            p1: start,
                        });
                        curr = start;
                    }
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        }

        segments
    }

    /// Generates CSS `@keyframes` block string.
    pub fn to_css_keyframes(name: &str, points: &[MotionTrajectoryPoint]) -> String {
        let mut rules = Vec::new();
        for pt in points {
            let pct = (pt.progress * 100.0).round() as u32;
            let rule = format!(
                "  {}% {{ transform: translate({:.1}px, {:.1}px) rotate({:.1}deg); }}",
                pct, pt.x, pt.y, pt.tangent_angle_deg
            );
            rules.push(rule);
        }
        format!("@keyframes {} {{\n{}\n}}", name, rules.join("\n"))
    }
}
