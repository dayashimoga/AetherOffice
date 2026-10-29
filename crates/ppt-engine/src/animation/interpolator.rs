#![forbid(unsafe_code)]
//! Property Interpolators for SMIL PresentationML Animations
//!
//! Provides interpolation for:
//! - Motion paths (`interpolate_motion`)
//! - Rotations (`interpolate_rotation`)
//! - Scales (`interpolate_scale`)
//! - Colors in RGB and HSL color spaces (`interpolate_color`)
//! - Transition effects and reveal masks (`interpolate_effect`)

use super::model::{ColorDirection, ColorSpace, EffectAnimation, EffectTransition};

/// Sample a 2D motion path at progress $t \in [0.0, 1.0]$.
///
/// Returns `(x, y, tangent_angle_deg)`.
/// Parses standard PresentationML motion path syntax (e.g. `"M 0 0 L 200 100 C ..."`).
pub fn interpolate_motion(path_str: &str, progress: f64) -> (f64, f64, f64) {
    let progress = progress.clamp(0.0, 1.0);
    let points = parse_motion_path_points(path_str);
    if points.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    if points.len() == 1 {
        return (points[0].0, points[0].1, 0.0);
    }

    // Compute cumulative segment lengths
    let mut lengths = Vec::with_capacity(points.len() - 1);
    let mut total_length = 0.0;
    for i in 0..points.len() - 1 {
        let dx = points[i + 1].0 - points[i].0;
        let dy = points[i + 1].1 - points[i].1;
        let dist = (dx * dx + dy * dy).sqrt();
        lengths.push(dist);
        total_length += dist;
    }

    if total_length <= 1e-6 {
        return (points[0].0, points[0].1, 0.0);
    }

    let target_dist = progress * total_length;
    let mut accumulated = 0.0;

    for i in 0..lengths.len() {
        let seg_len = lengths[i];
        if accumulated + seg_len >= target_dist || i == lengths.len() - 1 {
            let seg_t = if seg_len > 1e-6 {
                (target_dist - accumulated) / seg_len
            } else {
                0.0
            };
            let p0 = points[i];
            let p1 = points[i + 1];
            let x = p0.0 + (p1.0 - p0.0) * seg_t;
            let y = p0.1 + (p1.1 - p0.1) * seg_t;
            let angle = (p1.1 - p0.1).atan2(p1.0 - p0.0).to_degrees();
            return (x, y, angle);
        }
        accumulated += seg_len;
    }

    let last = points[points.len() - 1];
    (last.0, last.1, 0.0)
}

/// Helper to parse points from motion path commands.
fn parse_motion_path_points(path_str: &str) -> Vec<(f64, f64)> {
    let mut points = Vec::new();
    let tokens: Vec<&str> = path_str
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|s| !s.is_empty())
        .collect();

    let mut i = 0;
    let mut curr_x = 0.0;
    let mut curr_y = 0.0;

    while i < tokens.len() {
        let cmd = tokens[i];
        match cmd {
            "M" | "m" => {
                if i + 2 < tokens.len() {
                    let x: f64 = tokens[i + 1].parse().unwrap_or(curr_x);
                    let y: f64 = tokens[i + 2].parse().unwrap_or(curr_y);
                    curr_x = x;
                    curr_y = y;
                    points.push((curr_x, curr_y));
                    i += 3;
                } else {
                    break;
                }
            }
            "L" | "l" => {
                if i + 2 < tokens.len() {
                    let x: f64 = tokens[i + 1].parse().unwrap_or(curr_x);
                    let y: f64 = tokens[i + 2].parse().unwrap_or(curr_y);
                    curr_x = x;
                    curr_y = y;
                    points.push((curr_x, curr_y));
                    i += 3;
                } else {
                    break;
                }
            }
            "C" | "c" => {
                // Approximate cubic bezier with start, control points, and end point
                if i + 6 < tokens.len() {
                    let end_x: f64 = tokens[i + 5].parse().unwrap_or(curr_x);
                    let end_y: f64 = tokens[i + 6].parse().unwrap_or(curr_y);
                    curr_x = end_x;
                    curr_y = end_y;
                    points.push((curr_x, curr_y));
                    i += 7;
                } else {
                    break;
                }
            }
            "Z" | "z" => {
                if let Some(&first) = points.first() {
                    points.push(first);
                }
                i += 1;
            }
            _ => {
                // If it's a number, try reading as coordinate pair
                if let (Ok(x), Some(y)) = (
                    cmd.parse::<f64>(),
                    tokens.get(i + 1).and_then(|t| t.parse::<f64>().ok()),
                ) {
                    curr_x = x;
                    curr_y = y;
                    points.push((curr_x, curr_y));
                    i += 2;
                } else {
                    i += 1;
                }
            }
        }
    }

    if points.is_empty() {
        points.push((0.0, 0.0));
    }
    points
}

/// Interpolate rotation angle in degrees at progress $t \in [0.0, 1.0]$.
pub fn interpolate_rotation(
    from_deg: Option<f64>,
    to_deg: Option<f64>,
    by_deg: Option<f64>,
    progress: f64,
) -> f64 {
    let start = from_deg.unwrap_or(0.0);
    let end = if let Some(to) = to_deg {
        to
    } else if let Some(by) = by_deg {
        start + by
    } else {
        start + 360.0
    };

    start + (end - start) * progress
}

/// Interpolate scale factor $(s_x, s_y)$ at progress $t \in [0.0, 1.0]$.
pub fn interpolate_scale(
    from: Option<(f64, f64)>,
    to: Option<(f64, f64)>,
    by: Option<(f64, f64)>,
    progress: f64,
) -> (f64, f64) {
    let (sx0, sy0) = from.unwrap_or((1.0, 1.0));
    let (sx1, sy1) = if let Some(target) = to {
        target
    } else if let Some(delta) = by {
        (sx0 * delta.0, sy0 * delta.1)
    } else {
        (sx0, sy0)
    };

    (sx0 + (sx1 - sx0) * progress, sy0 + (sy1 - sy0) * progress)
}

/// Interpolate RGB color in either RGB or HSL space at progress $t \in [0.0, 1.0]$.
pub fn interpolate_color(
    from_rgb: [u8; 3],
    to_rgb: [u8; 3],
    progress: f64,
    space: ColorSpace,
    direction: ColorDirection,
) -> [u8; 3] {
    let t = progress.clamp(0.0, 1.0);
    match space {
        ColorSpace::Rgb => {
            let r = from_rgb[0] as f64 + (to_rgb[0] as f64 - from_rgb[0] as f64) * t;
            let g = from_rgb[1] as f64 + (to_rgb[1] as f64 - from_rgb[1] as f64) * t;
            let b = from_rgb[2] as f64 + (to_rgb[2] as f64 - from_rgb[2] as f64) * t;
            [
                r.round().clamp(0.0, 255.0) as u8,
                g.round().clamp(0.0, 255.0) as u8,
                b.round().clamp(0.0, 255.0) as u8,
            ]
        }
        ColorSpace::Hsl => {
            let (h0, s0, l0) = rgb_to_hsl(from_rgb);
            let (h1, s1, l1) = rgb_to_hsl(to_rgb);

            // Interpolate hue according to direction
            let mut diff = h1 - h0;
            match direction {
                ColorDirection::Clockwise => {
                    if diff < 0.0 {
                        diff += 360.0;
                    }
                }
                ColorDirection::CounterClockwise => {
                    if diff > 0.0 {
                        diff -= 360.0;
                    }
                }
            }

            let h = (h0 + diff * t).rem_euclid(360.0);
            let s = s0 + (s1 - s0) * t;
            let l = l0 + (l1 - l0) * t;

            hsl_to_rgb(h, s, l)
        }
    }
}

/// Interpolate visual transition effect: returns `(opacity, clip_progress, is_visible)`.
pub fn interpolate_effect(effect: &EffectAnimation, progress: f64) -> (f32, f32, bool) {
    let p = progress.clamp(0.0, 1.0) as f32;
    match effect.transition {
        EffectTransition::In => {
            // Entrance: starts hidden, fades/wipes in
            let opacity = p;
            let clip_progress = p;
            let is_visible = p > 0.0;
            (opacity, clip_progress, is_visible)
        }
        EffectTransition::Out => {
            // Exit: starts visible, fades/wipes out
            let opacity = 1.0 - p;
            let clip_progress = 1.0 - p;
            let is_visible = p < 1.0;
            (opacity, clip_progress, is_visible)
        }
    }
}

// =========================================================================
// Color Space Conversions
// =========================================================================

fn rgb_to_hsl(rgb: [u8; 3]) -> (f64, f64, f64) {
    let r = rgb[0] as f64 / 255.0;
    let g = rgb[1] as f64 / 255.0;
    let b = rgb[2] as f64 / 255.0;

    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let delta = max - min;

    let l = (max + min) / 2.0;

    if delta.abs() < 1e-6 {
        return (0.0, 0.0, l);
    }

    let s = if l > 0.5 {
        delta / (2.0 - max - min)
    } else {
        delta / (max + min)
    };

    let h = if (max - r).abs() < 1e-6 {
        let mut val = (g - b) / delta;
        if g < b {
            val += 6.0;
        }
        val * 60.0
    } else if (max - g).abs() < 1e-6 {
        ((b - r) / delta + 2.0) * 60.0
    } else {
        ((r - g) / delta + 4.0) * 60.0
    };

    (h, s, l)
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> [u8; 3] {
    if s.abs() < 1e-6 {
        let val = (l * 255.0).round().clamp(0.0, 255.0) as u8;
        return [val, val, val];
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let hk = h / 360.0;

    let tr = (hk + 1.0 / 3.0).rem_euclid(1.0);
    let tg = hk.rem_euclid(1.0);
    let tb = (hk - 1.0 / 3.0).rem_euclid(1.0);

    let color_calc = |tc: f64| -> f64 {
        if tc < 1.0 / 6.0 {
            p + ((q - p) * 6.0 * tc)
        } else if tc < 1.0 / 2.0 {
            q
        } else if tc < 2.0 / 3.0 {
            p + ((q - p) * 6.0 * (2.0 / 3.0 - tc))
        } else {
            p
        }
    };

    [
        (color_calc(tr) * 255.0).round().clamp(0.0, 255.0) as u8,
        (color_calc(tg) * 255.0).round().clamp(0.0, 255.0) as u8,
        (color_calc(tb) * 255.0).round().clamp(0.0, 255.0) as u8,
    ]
}
