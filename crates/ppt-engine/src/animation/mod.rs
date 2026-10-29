#![forbid(unsafe_code)]
//! PresentationML SMIL Animation Timing & 120Hz Evaluator (ECMA-376 Part 1, Section 19.5)
//!
//! Provides:
//! - Full PresentationML SMIL timing DOM (`SlideTiming`, `TimeNode`, `CommonTimeNode`)
//! - Trigger types (`OnClick`, `WithPrev`, `AfterPrev`)
//! - Piecewise acceleration / deceleration easing curves and cubic Bezier easing
//! - Property interpolators (motion paths, rotation, scale, color, effects, opacity)
//! - 120Hz deterministic timeline evaluator (`AnimationTimeline`, `AnimatedSceneState`)

pub mod easing;
pub mod interpolator;
pub mod model;
pub mod timeline;

pub use easing::{apply_accel_decel, compute_node_progress, solve_cubic_bezier_easing};
pub use interpolator::{
    interpolate_color, interpolate_effect, interpolate_motion, interpolate_rotation,
    interpolate_scale,
};
pub use model::*;
pub use timeline::{AnimatedSceneState, AnimatedShapeState, AnimationTimeline};
