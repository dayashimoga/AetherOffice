#![forbid(unsafe_code)]
//! 120Hz Deterministic Animation Timeline Evaluator & Click State Machine
//!
//! Provides:
//! - Frame-accurate deterministic evaluation (`sample_at(time_ms)`)
//! - Interactive click progression (`next_click()`, `seek_click(step)`)
//! - Evaluation of parallel/sequential time containers, delays, easing, and property interpolators
//! - Multi-shape animated scene state accumulation

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::easing::compute_node_progress;
use super::interpolator::{
    interpolate_color, interpolate_effect, interpolate_motion, interpolate_rotation,
    interpolate_scale,
};
use super::model::*;

/// Animated state of an individual shape at a specific point in time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimatedShapeState {
    pub shape_id: u32,
    pub visible: bool,
    pub opacity: f32,
    pub translation: (f64, f64),
    pub rotation_deg: f64,
    pub scale: (f64, f64),
    pub fill_color: Option<[u8; 3]>,
    pub clip_progress: f32,
}

impl AnimatedShapeState {
    pub fn new(shape_id: u32) -> Self {
        Self {
            shape_id,
            visible: true,
            opacity: 1.0,
            translation: (0.0, 0.0),
            rotation_deg: 0.0,
            scale: (1.0, 1.0),
            fill_color: None,
            clip_progress: 1.0,
        }
    }
}

/// Full animated scene snapshot at a specific point in time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimatedSceneState {
    pub time_ms: f64,
    pub click_index: usize,
    pub shapes: BTreeMap<u32, AnimatedShapeState>,
}

impl AnimatedSceneState {
    pub fn get_shape(&self, shape_id: u32) -> Option<&AnimatedShapeState> {
        self.shapes.get(&shape_id)
    }
}

/// A scheduled animation step within the click sequence.
#[derive(Debug, Clone)]
struct ScheduledAnimation {
    click_step: usize,
    start_time_ms: f64,
    duration_ms: f64,
    node: TimeNode,
}

/// Interactive Timeline and 120Hz deterministic state machine evaluator.
#[derive(Debug, Clone)]
pub struct AnimationTimeline {
    scheduled: Vec<ScheduledAnimation>,
    total_clicks: usize,
    current_click: usize,
}

impl AnimationTimeline {
    /// Construct a timeline from a `SlideTiming` model.
    pub fn new(timing: &SlideTiming) -> Self {
        let mut timeline = Self {
            scheduled: Vec::new(),
            total_clicks: 0,
            current_click: 0,
        };

        if let Some(ref root) = timing.root_node {
            timeline.compile_timing_tree(root);
        }

        timeline
    }

    /// Construct an empty timeline.
    pub fn empty() -> Self {
        Self {
            scheduled: Vec::new(),
            total_clicks: 0,
            current_click: 0,
        }
    }

    /// Schedule an animation node directly into a specific click step.
    pub fn schedule_node(&mut self, click_step: usize, delay_ms: f64, node: TimeNode) {
        let dur = node.common.duration_ms.unwrap_or(1000) as f64;
        self.scheduled.push(ScheduledAnimation {
            click_step,
            start_time_ms: delay_ms,
            duration_ms: dur,
            node,
        });

        if click_step >= self.total_clicks {
            self.total_clicks = click_step + 1;
        }
    }

    /// Total number of click steps in this slide animation.
    pub fn total_clicks(&self) -> usize {
        self.total_clicks
    }

    /// Currently active click step (0-indexed).
    pub fn current_click(&self) -> usize {
        self.current_click
    }

    /// Advance to the next click step. Returns `true` if advanced, `false` if already at end.
    pub fn next_click(&mut self) -> bool {
        if self.current_click + 1 < self.total_clicks {
            self.current_click += 1;
            true
        } else {
            false
        }
    }

    /// Return to the previous click step.
    pub fn prev_click(&mut self) -> bool {
        if self.current_click > 0 {
            self.current_click -= 1;
            true
        } else {
            false
        }
    }

    /// Seek directly to a specific click step.
    pub fn seek_click(&mut self, step: usize) {
        if step < self.total_clicks {
            self.current_click = step;
        } else if self.total_clicks > 0 {
            self.current_click = self.total_clicks - 1;
        }
    }

    /// Total duration in milliseconds for the current click step.
    pub fn step_duration_ms(&self, step: usize) -> f64 {
        let mut max_end = 0.0;
        for s in &self.scheduled {
            if s.click_step == step {
                let end = s.start_time_ms + s.duration_ms;
                if end > max_end {
                    max_end = end;
                }
            }
        }
        max_end
    }

    /// Total overall duration of all scheduled animations combined.
    pub fn total_duration_ms(&self) -> f64 {
        let mut total = 0.0;
        for step in 0..self.total_clicks {
            total += self.step_duration_ms(step);
        }
        total
    }

    /// Sample the animation state at an exact timestamp within the current click step.
    /// Frame-accurate at 120Hz (8.333ms resolution).
    pub fn sample_at(&self, time_ms: f64) -> AnimatedSceneState {
        let mut shapes: BTreeMap<u32, AnimatedShapeState> = BTreeMap::new();

        // 1. Process previous click steps: apply final state (hold)
        for s in &self.scheduled {
            if s.click_step < self.current_click {
                self.apply_node_to_scene(
                    &s.node,
                    1.0, // Completed (progress 1.0)
                    &mut shapes,
                );
            }
        }

        // 2. Process current click step: evaluate at time_ms
        for s in &self.scheduled {
            if s.click_step == self.current_click {
                let elapsed = time_ms - s.start_time_ms;
                if let Some(progress) = compute_node_progress(
                    elapsed,
                    s.node.common.delay_ms,
                    s.node.common.duration_ms,
                    s.node.common.repeat_count,
                    s.node.common.auto_reverse,
                    s.node.common.accel_ratio,
                    s.node.common.decel_ratio,
                ) {
                    self.apply_node_to_scene(&s.node, progress, &mut shapes);
                }
            }
        }

        AnimatedSceneState {
            time_ms,
            click_index: self.current_click,
            shapes,
        }
    }

    /// Sample at a specific frame index given frame rate (e.g. 120 fps).
    pub fn sample_frame(&self, frame_index: u64, fps: u32) -> AnimatedSceneState {
        let frame_duration_ms = 1000.0 / fps as f64;
        let time_ms = frame_index as f64 * frame_duration_ms;
        self.sample_at(time_ms)
    }

    fn apply_node_to_scene(
        &self,
        node: &TimeNode,
        progress: f64,
        shapes: &mut BTreeMap<u32, AnimatedShapeState>,
    ) {
        match &node.kind {
            TimeNodeKind::Motion(motion) => {
                let entry = shapes
                    .entry(motion.target.shape_id)
                    .or_insert_with(|| AnimatedShapeState::new(motion.target.shape_id));
                let (dx, dy, angle) = interpolate_motion(&motion.path, progress);
                entry.translation = (dx, dy);
                if motion.rotate_to_path {
                    entry.rotation_deg = angle;
                }
            }
            TimeNodeKind::Rotation(rot) => {
                let entry = shapes
                    .entry(rot.target.shape_id)
                    .or_insert_with(|| AnimatedShapeState::new(rot.target.shape_id));
                entry.rotation_deg =
                    interpolate_rotation(rot.from_deg, rot.to_deg, rot.by_deg, progress);
            }
            TimeNodeKind::Scale(scale) => {
                let entry = shapes
                    .entry(scale.target.shape_id)
                    .or_insert_with(|| AnimatedShapeState::new(scale.target.shape_id));
                entry.scale =
                    interpolate_scale(scale.from_scale, scale.to_scale, scale.by_scale, progress);
            }
            TimeNodeKind::Color(color) => {
                let entry = shapes
                    .entry(color.target.shape_id)
                    .or_insert_with(|| AnimatedShapeState::new(color.target.shape_id));
                if let (Some(from), Some(to)) = (color.from_rgb, color.to_rgb) {
                    entry.fill_color = Some(interpolate_color(
                        from,
                        to,
                        progress,
                        color.color_space,
                        color.direction,
                    ));
                }
            }
            TimeNodeKind::Effect(effect) => {
                let entry = shapes
                    .entry(effect.target.shape_id)
                    .or_insert_with(|| AnimatedShapeState::new(effect.target.shape_id));
                let (opacity, clip_progress, visible) = interpolate_effect(effect, progress);
                entry.opacity = opacity;
                entry.clip_progress = clip_progress;
                entry.visible = visible;
            }
            TimeNodeKind::Generic(generic) => {
                let entry = shapes
                    .entry(generic.target.shape_id)
                    .or_insert_with(|| AnimatedShapeState::new(generic.target.shape_id));
                if generic.attribute_name == "opacity" || generic.attribute_name == "alpha" {
                    let from = generic.from_val.unwrap_or(0.0);
                    let to = generic.to_val.unwrap_or(1.0);
                    entry.opacity = (from + (to - from) * progress).clamp(0.0, 1.0) as f32;
                }
            }
            TimeNodeKind::Parallel(children) | TimeNodeKind::Sequential(children) => {
                for child in children {
                    self.apply_node_to_scene(child, progress, shapes);
                }
            }
            TimeNodeKind::Set(set) => {
                let entry = shapes
                    .entry(set.target.shape_id)
                    .or_insert_with(|| AnimatedShapeState::new(set.target.shape_id));
                if set.attribute_name == "visible" {
                    entry.visible = set.value == "1" || set.value == "true";
                }
            }
        }
    }

    fn compile_timing_tree(&mut self, root: &TimeNode) {
        let mut click_step = 0;
        self.compile_node(root, &mut click_step, 0.0);
        self.total_clicks = click_step.max(1);
    }

    fn compile_node(&mut self, node: &TimeNode, click_step: &mut usize, current_time: f64) {
        match &node.kind {
            TimeNodeKind::Sequential(children) => {
                let mut seq_time = current_time;
                for child in children {
                    if child.common.trigger_type == TriggerType::OnClick {
                        *click_step += 1;
                        seq_time = 0.0;
                    }
                    self.compile_node(child, click_step, seq_time);
                    if child.common.trigger_type == TriggerType::AfterPrev {
                        seq_time += child.common.duration_ms.unwrap_or(1000) as f64;
                    }
                }
            }
            TimeNodeKind::Parallel(children) => {
                for child in children {
                    self.compile_node(child, click_step, current_time);
                }
            }
            _ => {
                let dur = node.common.duration_ms.unwrap_or(1000) as f64;
                let start_time = current_time + node.common.delay_ms as f64;
                self.scheduled.push(ScheduledAnimation {
                    click_step: *click_step,
                    start_time_ms: start_time,
                    duration_ms: dur,
                    node: node.clone(),
                });
            }
        }
    }
}
