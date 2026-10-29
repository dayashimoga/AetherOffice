#![forbid(unsafe_code)]
//! PresentationML SMIL Animation Timing DOM (ECMA-376 Part 1, Section 19.5)
//!
//! Provides typed data structures for PresentationML animation timing trees:
//! - `<p:timing>`: Slide animation timing root
//! - `<p:tnLst>`: Time node list
//! - `<p:cTn>`: Common time node properties (duration, repeat, accel/decel, auto-reverse)
//! - `<p:par>`, `<p:seq>`: Parallel and sequential time containers
//! - `<p:animMotion>`, `<p:animRot>`, `<p:animScale>`, `<p:animClr>`, `<p:anim>`, `<p:animEffect>`

use serde::{Deserialize, Serialize};

/// Slide animation timing root: `<p:timing>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SlideTiming {
    pub root_node: Option<TimeNode>,
}

/// A node in the SMIL animation timing tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeNode {
    pub common: CommonTimeNode,
    pub kind: TimeNodeKind,
}

/// Common properties of any time node: `<p:cTn>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommonTimeNode {
    pub id: u32,
    pub duration_ms: Option<u64>,  // None represents "indefinite"
    pub delay_ms: i64,             // Start delay relative to trigger
    pub repeat_count: Option<f32>, // 1.0 = once, None = indefinite
    pub auto_reverse: bool,
    pub accel_ratio: f32, // 0.0 to 1.0 (accel attribute is 0..100000)
    pub decel_ratio: f32, // 0.0 to 1.0 (decel attribute is 0..100000)
    pub trigger_type: TriggerType,
    pub fill: TimeNodeFill,
    pub restart: TimeNodeRestart,
}

impl Default for CommonTimeNode {
    fn default() -> Self {
        Self {
            id: 0,
            duration_ms: Some(1000), // Default 1 second
            delay_ms: 0,
            repeat_count: Some(1.0),
            auto_reverse: false,
            accel_ratio: 0.0,
            decel_ratio: 0.0,
            trigger_type: TriggerType::WithPrev,
            fill: TimeNodeFill::Hold,
            restart: TimeNodeRestart::Always,
        }
    }
}

/// How a time node is triggered relative to previous nodes or user interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TriggerType {
    /// Starts on user click / slide advance step (`nodeType="clickEffect"` or `<p:cond evt="onBegin"/>`)
    OnClick,
    /// Starts concurrently with previous sibling node (`nodeType="withEffect"`)
    #[default]
    WithPrev,
    /// Starts immediately after previous sibling node finishes (`nodeType="afterEffect"`)
    AfterPrev,
}

/// Fill behavior after animation completes: `<p:cTn fill="...">`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TimeNodeFill {
    #[default]
    Hold,
    Remove,
    Freeze,
}

/// Restart behavior: `<p:cTn restart="...">`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TimeNodeRestart {
    #[default]
    Always,
    WhenNotActive,
    Never,
}

/// Specific behavior or container variant of a time node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TimeNodeKind {
    /// Parallel container: runs child nodes concurrently: `<p:par>`
    Parallel(Vec<TimeNode>),
    /// Sequential container: runs child nodes in order: `<p:seq>`
    Sequential(Vec<TimeNode>),
    /// Motion path animation: `<p:animMotion>`
    Motion(MotionAnimation),
    /// Rotation animation: `<p:animRot>`
    Rotation(RotationAnimation),
    /// Scale animation: `<p:animScale>`
    Scale(ScaleAnimation),
    /// Color transition animation: `<p:animClr>`
    Color(ColorAnimation),
    /// Generic attribute animation: `<p:anim>`
    Generic(GenericAnimation),
    /// Reveal / Wipe / Filter effect animation: `<p:animEffect>`
    Effect(EffectAnimation),
    /// Discrete property set: `<p:set>`
    Set(SetAnimation),
}

/// Target shape identifier: `<p:tgtEl><p:spTgt spid="..."/></p:tgtEl>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TargetElement {
    pub shape_id: u32,
}

/// Motion path animation: `<p:animMotion>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct MotionAnimation {
    pub target: TargetElement,
    /// Path specification string in normalized SVG/DrawingML format (e.g. `"M 0 0 L 0.25 0.5"`)
    pub path: String,
    pub rotate_to_path: bool,
}

/// Rotation animation: `<p:animRot>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RotationAnimation {
    pub target: TargetElement,
    /// Start angle in degrees
    pub from_deg: Option<f64>,
    /// End angle in degrees
    pub to_deg: Option<f64>,
    /// Delta rotation angle in degrees (e.g. 360.0 for full spin)
    pub by_deg: Option<f64>,
}

/// Scale / Zoom animation: `<p:animScale>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScaleAnimation {
    pub target: TargetElement,
    /// Start scale factor (sx, sy), where 1.0 = 100%
    pub from_scale: Option<(f64, f64)>,
    /// End scale factor (sx, sy)
    pub to_scale: Option<(f64, f64)>,
    /// Delta scale factor (sx, sy)
    pub by_scale: Option<(f64, f64)>,
}

impl Default for ScaleAnimation {
    fn default() -> Self {
        Self {
            target: TargetElement::default(),
            from_scale: Some((1.0, 1.0)),
            to_scale: Some((1.0, 1.0)),
            by_scale: None,
        }
    }
}

/// Color transition animation: `<p:animClr>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorAnimation {
    pub target: TargetElement,
    pub color_space: ColorSpace,
    pub direction: ColorDirection,
    pub from_rgb: Option<[u8; 3]>,
    pub to_rgb: Option<[u8; 3]>,
}

impl Default for ColorAnimation {
    fn default() -> Self {
        Self {
            target: TargetElement::default(),
            color_space: ColorSpace::Rgb,
            direction: ColorDirection::Clockwise,
            from_rgb: None,
            to_rgb: None,
        }
    }
}

/// Color space for color interpolation: `clrSpc="rgb"` or `"hsl"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ColorSpace {
    #[default]
    Rgb,
    Hsl,
}

/// Direction on the HSL color wheel: `dir="cw"` or `"ccw"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ColorDirection {
    #[default]
    Clockwise,
    CounterClockwise,
}

/// Generic property animation: `<p:anim>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct GenericAnimation {
    pub target: TargetElement,
    pub attribute_name: String,
    pub from_val: Option<f64>,
    pub to_val: Option<f64>,
    pub by_val: Option<f64>,
}

/// Visual effect animation (fade, wipe, box, etc.): `<p:animEffect>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct EffectAnimation {
    pub target: TargetElement,
    pub effect_type: String, // e.g. "wipe", "fade", "box", "circle"
    pub transition: EffectTransition,
}

/// Transition direction: `transition="in"` (reveal/entrance) or `"out"` (exit).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum EffectTransition {
    #[default]
    In,
    Out,
}

/// Discrete property set: `<p:set>`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SetAnimation {
    pub target: TargetElement,
    pub attribute_name: String,
    pub value: String,
}
