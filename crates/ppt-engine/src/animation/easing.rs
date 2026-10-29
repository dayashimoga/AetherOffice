#![forbid(unsafe_code)]
//! SMIL Animation Timing, Repetition & Easing Calculations
//!
//! Evaluates SMIL timing dynamics:
//! - Start delay and active interval calculation
//! - Repeat counts (finite and indefinite)
//! - Auto-reverse (ping-pong animation)
//! - Acceleration and deceleration profile (ECMA-376 Section 19.5.48)
//! - Cubic Bezier easing curves

/// Computes the normalized active progress $p \in [0.0, 1.0]$ for a time node given elapsed time.
///
/// Returns:
/// - `None` if the animation has not started yet (`elapsed_ms < delay_ms`).
/// - `Some(progress)` if active, or after completion if `fill == Hold`.
pub fn compute_node_progress(
    elapsed_ms: f64,
    delay_ms: i64,
    duration_ms: Option<u64>,
    repeat_count: Option<f32>,
    auto_reverse: bool,
    accel_ratio: f32,
    decel_ratio: f32,
) -> Option<f64> {
    let t_active = elapsed_ms - delay_ms as f64;
    if t_active < 0.0 {
        return None;
    }

    let dur = match duration_ms {
        Some(d) if d > 0 => d as f64,
        _ => return Some(0.0), // Indefinite or 0
    };

    let total_repeats = repeat_count.unwrap_or(1.0) as f64;
    let total_active_dur = dur * total_repeats;

    if t_active >= total_active_dur {
        // Animation finished: determine final position
        let frac = total_repeats - total_repeats.floor();
        let final_linear = if frac > f64::EPSILON {
            let last_cycle = total_repeats.floor() as u64;
            if auto_reverse && (last_cycle % 2 == 1) {
                1.0 - frac
            } else {
                frac
            }
        } else {
            // Exactly finished integer cycles
            let full_cycles = total_repeats.round() as u64;
            if auto_reverse && full_cycles.is_multiple_of(2) {
                // E.g., 2 full cycles: cycle 0 (0->1), cycle 1 (1->0). Finished at start (0.0).
                0.0
            } else {
                // E.g., 1 full cycle: cycle 0 (0->1). Finished at end (1.0).
                1.0
            }
        };
        return Some(apply_accel_decel(
            final_linear,
            accel_ratio as f64,
            decel_ratio as f64,
        ));
    }

    // Determine current iteration progress
    let cycle_index = (t_active / dur).floor();
    let current_cycle_t = t_active - cycle_index * dur;
    let mut linear_progress = (current_cycle_t / dur).clamp(0.0, 1.0);

    // Apply auto-reverse if needed
    if auto_reverse && (cycle_index as u64 % 2 == 1) {
        linear_progress = 1.0 - linear_progress;
    }

    // Apply acceleration and deceleration easing
    Some(apply_accel_decel(
        linear_progress,
        accel_ratio as f64,
        decel_ratio as f64,
    ))
}

/// Applies SMIL piecewise acceleration and deceleration easing to linear progress $t \in [0, 1]$.
///
/// Formulation according to SMIL 2.0 / ECMA-376 Part 1 Section 19.5:
/// - Acceleration ratio $a \in [0, 1]$
/// - Deceleration ratio $d \in [0, 1]$ with $a + d \le 1.0$.
pub fn apply_accel_decel(t: f64, a: f64, d: f64) -> f64 {
    let a = a.clamp(0.0, 1.0);
    let d = d.clamp(0.0, 1.0);

    if a + d > 1.0 {
        // Normalize if sum exceeds 1.0
        let sum = a + d;
        return apply_accel_decel(t, a / sum, d / sum);
    }

    if a <= 1e-6 && d <= 1e-6 {
        return t;
    }

    // Maximum velocity v_max = 2 / (2 - a - d)
    let v_max = 2.0 / (2.0 - a - d);

    if t < a && a > 1e-6 {
        // Acceleration phase: velocity increases linearly from 0 to v_max
        (v_max / (2.0 * a)) * t * t
    } else if t < (1.0 - d) {
        // Constant velocity phase
        (a * v_max / 2.0) + v_max * (t - a)
    } else if d > 1e-6 {
        // Deceleration phase: velocity decreases linearly from v_max to 0
        let t_rem = 1.0 - t;
        1.0 - (v_max / (2.0 * d)) * t_rem * t_rem
    } else {
        1.0
    }
}

/// Cubic Bezier easing evaluator (like CSS cubic-bezier or PresentationML `keySplines`).
/// Control points $P_1 = (x_1, y_1)$ and $P_2 = (x_2, y_2)$ with $P_0 = (0, 0)$ and $P_3 = (1, 1)$.
pub fn solve_cubic_bezier_easing(x1: f64, y1: f64, x2: f64, y2: f64, target_x: f64) -> f64 {
    let target_x = target_x.clamp(0.0, 1.0);
    if target_x <= 1e-6 {
        return 0.0;
    }
    if target_x >= 1.0 - 1e-6 {
        return 1.0;
    }

    // Solve for parameter t such that B_x(t) == target_x using bisection
    let mut low = 0.0;
    let mut high = 1.0;
    let mut t = target_x;

    for _ in 0..16 {
        let current_x = eval_bezier_coord(x1, x2, t);
        if (current_x - target_x).abs() < 1e-5 {
            break;
        }
        if current_x < target_x {
            low = t;
        } else {
            high = t;
        }
        t = (low + high) / 2.0;
    }

    eval_bezier_coord(y1, y2, t)
}

#[inline]
fn eval_bezier_coord(c1: f64, c2: f64, t: f64) -> f64 {
    let inv = 1.0 - t;
    3.0 * inv * inv * t * c1 + 3.0 * inv * t * t * c2 + t * t * t
}
