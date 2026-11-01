#![forbid(unsafe_code)]
//! Industrial PresentationML Morph Transition & SMIL Animation Bridge
//!
//! Provides high-level serialization DTOs and evaluators for:
//! - Hardware-accelerated Morph transitions (continuous geometry, transform, and text interpolation)
//! - 120Hz frame-accurate SMIL animation scrubbing and click progression state machine
//! - Direct bidirectional bridge between presentation package slides and presentation shell

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::animation::{
    AnimatedSceneState, AnimatedShapeState, AnimationTimeline, CommonTimeNode, EffectAnimation,
    EffectTransition, MotionAnimation, RotationAnimation, ScaleAnimation, SlideTiming,
    TargetElement, TimeNode, TimeNodeFill, TimeNodeKind, TimeNodeRestart, TriggerType,
};
use crate::model::{MorphOption, Slide};
use crate::morph::{MorphEvaluator, MorphedSlideState, MorphedTransform};
use crate::package::PresentationPackage;

// =========================================================================
// 1. Serialization DTOs
// =========================================================================

/// Serializable transform of a morphed shape.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MorphedTransformDto {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation_deg: f64,
}

impl From<MorphedTransform> for MorphedTransformDto {
    fn from(t: MorphedTransform) -> Self {
        Self {
            x: t.x,
            y: t.y,
            width: t.width,
            height: t.height,
            rotation_deg: t.rotation_deg,
        }
    }
}

/// Serializable individual text element interpolated during morph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MorphedTextElementDto {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub font_size_pt: f64,
    pub opacity: f64,
}

/// Evaluated state of a morphed shape crossing the IPC boundary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MorphedShapeStateDto {
    pub shape_id: u32,
    pub name: String,
    pub transform: MorphedTransformDto,
    pub opacity: f64,
    pub is_matched: bool,
    pub text_elements: Vec<MorphedTextElementDto>,
}

/// Full evaluated morph transition scene state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MorphedSlideStateDto {
    pub t: f64,
    pub shapes: Vec<MorphedShapeStateDto>,
}

/// Scheduled animation track item for timeline UI visualization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnimationTrackItemDto {
    pub shape_id: u32,
    pub shape_name: String,
    pub effect_type: String,
    pub click_step: usize,
    pub start_time_ms: f64,
    pub duration_ms: f64,
}

/// Summary of all animation tracks present on a slide.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SlideAnimationTracksDto {
    pub slide_index: usize,
    pub total_clicks: usize,
    pub total_duration_ms: f64,
    pub tracks: Vec<AnimationTrackItemDto>,
}

/// Animated state of a single shape during timeline scrubbing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnimatedShapeStateDto {
    pub shape_id: u32,
    pub visible: bool,
    pub opacity: f64,
    pub translation_x: f64,
    pub translation_y: f64,
    pub rotation_deg: f64,
    pub scale_x: f64,
    pub scale_y: f64,
    pub fill_color_rgb: Option<[u8; 3]>,
}

impl From<&AnimatedShapeState> for AnimatedShapeStateDto {
    fn from(s: &AnimatedShapeState) -> Self {
        Self {
            shape_id: s.shape_id,
            visible: s.visible,
            opacity: s.opacity as f64,
            translation_x: s.translation.0,
            translation_y: s.translation.1,
            rotation_deg: s.rotation_deg,
            scale_x: s.scale.0,
            scale_y: s.scale.1,
            fill_color_rgb: s.fill_color,
        }
    }
}

/// Animated scene state at a scrubbed timestamp.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnimatedSceneStateDto {
    pub time_ms: f64,
    pub click_index: usize,
    pub shapes: Vec<AnimatedShapeStateDto>,
}

impl From<AnimatedSceneState> for AnimatedSceneStateDto {
    fn from(s: AnimatedSceneState) -> Self {
        let shapes = s.shapes.values().map(AnimatedShapeStateDto::from).collect();
        Self {
            time_ms: s.time_ms,
            click_index: s.click_index,
            shapes,
        }
    }
}

/// Result of navigating animation click steps.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnimationClickResultDto {
    pub click_index: usize,
    pub total_clicks: usize,
    pub time_ms: f64,
    pub scene: AnimatedSceneStateDto,
}

// =========================================================================
// 2. Morph Transition Bridge
// =========================================================================

pub struct MorphTransitionBridge;

impl MorphTransitionBridge {
    /// Evaluates morph transition between two arbitrary slides in a presentation package.
    pub fn evaluate_morph_transition(
        pkg: &PresentationPackage,
        source_slide_idx: usize,
        target_slide_idx: usize,
        progress: f64,
        morph_option: &str,
    ) -> Result<MorphedSlideStateDto, String> {
        if source_slide_idx >= pkg.slides.len() {
            return Err(format!(
                "Source slide index {source_slide_idx} out of bounds"
            ));
        }
        if target_slide_idx >= pkg.slides.len() {
            return Err(format!(
                "Target slide index {target_slide_idx} out of bounds"
            ));
        }

        let slide_a = &pkg.slides[source_slide_idx].1;
        let slide_b = &pkg.slides[target_slide_idx].1;

        let canvas_w = if pkg.presentation.slide_size.cx > 0 {
            pkg.presentation.slide_size.cx as f64
        } else {
            12_192_000.0 // 16:9 EMU default
        };
        let canvas_h = if pkg.presentation.slide_size.cy > 0 {
            pkg.presentation.slide_size.cy as f64
        } else {
            6_858_000.0 // 16:9 EMU default
        };

        let option = match morph_option {
            "byChar" => MorphOption::ByChar,
            "byWord" => MorphOption::ByWord,
            _ => MorphOption::ByObject,
        };

        let morphed = MorphEvaluator::evaluate_transition(
            slide_a, slide_b, option, progress, canvas_w, canvas_h,
        );

        Ok(Self::convert_morphed_state(morphed))
    }

    fn convert_morphed_state(morphed: MorphedSlideState) -> MorphedSlideStateDto {
        let shapes = morphed
            .shapes
            .into_iter()
            .map(|s| {
                let is_matched = s.opacity >= 0.99 && !s.name.is_empty();
                let text_elements = s
                    .text_elements
                    .into_iter()
                    .map(|te| MorphedTextElementDto {
                        text: te.text,
                        x: te.x,
                        y: te.y,
                        font_size_pt: te.font_size_pt,
                        opacity: te.opacity,
                    })
                    .collect();

                MorphedShapeStateDto {
                    shape_id: s.shape_id,
                    name: s.name,
                    transform: s.transform.into(),
                    opacity: s.opacity,
                    is_matched,
                    text_elements,
                }
            })
            .collect();

        MorphedSlideStateDto {
            t: morphed.t,
            shapes,
        }
    }
}

// =========================================================================
// 3. SMIL Animation Timeline Bridge
// =========================================================================

pub struct SmilAnimationBridge;

impl SmilAnimationBridge {
    /// Extracts shape names from a slide shape tree into an ID -> Name lookup map.
    fn extract_shape_names(slide: &Slide) -> HashMap<u32, String> {
        let mut map = HashMap::new();
        for elem in &slide.common_slide_data.shape_tree.shapes {
            map.insert(elem.id(), elem.name().to_string());
        }
        map
    }

    /// Extracts track summary information for a slide.
    pub fn extract_slide_animation_tracks(
        pkg: &PresentationPackage,
        slide_idx: usize,
    ) -> Result<SlideAnimationTracksDto, String> {
        if slide_idx >= pkg.slides.len() {
            return Err(format!("Slide index {slide_idx} out of bounds"));
        }

        let slide = &pkg.slides[slide_idx].1;
        let shape_names = Self::extract_shape_names(slide);

        if let Some(ref timing) = slide.timing {
            let timeline = AnimationTimeline::new(timing);
            let raw_tracks = timeline.extract_tracks();

            let tracks = raw_tracks
                .into_iter()
                .map(|t| {
                    let shape_name = shape_names
                        .get(&t.shape_id)
                        .cloned()
                        .unwrap_or_else(|| format!("Shape {}", t.shape_id));

                    AnimationTrackItemDto {
                        shape_id: t.shape_id,
                        shape_name,
                        effect_type: t.effect_type,
                        click_step: t.click_step,
                        start_time_ms: t.start_time_ms,
                        duration_ms: t.duration_ms,
                    }
                })
                .collect();

            Ok(SlideAnimationTracksDto {
                slide_index: slide_idx,
                total_clicks: timeline.total_clicks(),
                total_duration_ms: timeline.total_duration_ms(),
                tracks,
            })
        } else {
            Ok(SlideAnimationTracksDto {
                slide_index: slide_idx,
                total_clicks: 0,
                total_duration_ms: 0.0,
                tracks: Vec::new(),
            })
        }
    }

    /// Evaluates the slide animation state at an exact timestamp and click step.
    pub fn scrub_animation(
        pkg: &PresentationPackage,
        slide_idx: usize,
        time_ms: f64,
        click_index: Option<usize>,
    ) -> Result<AnimatedSceneStateDto, String> {
        if slide_idx >= pkg.slides.len() {
            return Err(format!("Slide index {slide_idx} out of bounds"));
        }

        let slide = &pkg.slides[slide_idx].1;

        if let Some(ref timing) = slide.timing {
            let mut timeline = AnimationTimeline::new(timing);
            if let Some(click) = click_index {
                timeline.seek_click(click);
            }
            let scene = timeline.sample_at(time_ms);
            Ok(scene.into())
        } else {
            // Default static scene when no animations are configured
            Ok(AnimatedSceneStateDto {
                time_ms,
                click_index: click_index.unwrap_or(0),
                shapes: Vec::new(),
            })
        }
    }

    /// Steps through click sequence (forward or backward) and returns evaluated start scene.
    pub fn advance_click(
        pkg: &PresentationPackage,
        slide_idx: usize,
        current_click: usize,
        forward: bool,
    ) -> Result<AnimationClickResultDto, String> {
        if slide_idx >= pkg.slides.len() {
            return Err(format!("Slide index {slide_idx} out of bounds"));
        }

        let slide = &pkg.slides[slide_idx].1;

        if let Some(ref timing) = slide.timing {
            let mut timeline = AnimationTimeline::new(timing);
            timeline.seek_click(current_click);

            let new_click = if forward {
                let _ = timeline.next_click();
                timeline.current_click()
            } else {
                let _ = timeline.prev_click();
                timeline.current_click()
            };

            let scene = timeline.sample_at(0.0);
            Ok(AnimationClickResultDto {
                click_index: new_click,
                total_clicks: timeline.total_clicks(),
                time_ms: 0.0,
                scene: scene.into(),
            })
        } else {
            Ok(AnimationClickResultDto {
                click_index: 0,
                total_clicks: 0,
                time_ms: 0.0,
                scene: AnimatedSceneStateDto {
                    time_ms: 0.0,
                    click_index: 0,
                    shapes: Vec::new(),
                },
            })
        }
    }

    /// Helper: Injects an animation effect node into a slide timing tree.
    pub fn add_animation_effect(
        slide: &mut Slide,
        shape_id: u32,
        effect_type: &str,
        trigger: TriggerType,
        duration_ms: u64,
    ) {
        let node_kind = match effect_type {
            "Motion" => TimeNodeKind::Motion(MotionAnimation {
                target: TargetElement { shape_id },
                path: "M 0 0 L 0.25 0.5".to_string(),
                rotate_to_path: false,
            }),
            "Rotation" => TimeNodeKind::Rotation(RotationAnimation {
                target: TargetElement { shape_id },
                from_deg: Some(0.0),
                to_deg: Some(360.0),
                by_deg: Some(360.0),
            }),
            "Scale" => TimeNodeKind::Scale(ScaleAnimation {
                target: TargetElement { shape_id },
                from_scale: Some((1.0, 1.0)),
                to_scale: Some((1.5, 1.5)),
                by_scale: None,
            }),
            _ => TimeNodeKind::Effect(EffectAnimation {
                target: TargetElement { shape_id },
                effect_type: effect_type.to_string(),
                transition: EffectTransition::In,
            }),
        };

        let common = CommonTimeNode {
            id: shape_id + 1000,
            duration_ms: Some(duration_ms),
            delay_ms: 0,
            repeat_count: Some(1.0),
            auto_reverse: false,
            accel_ratio: 0.0,
            decel_ratio: 0.0,
            trigger_type: trigger,
            fill: TimeNodeFill::Hold,
            restart: TimeNodeRestart::Always,
        };

        let anim_node = TimeNode {
            common,
            kind: node_kind,
        };

        if let Some(ref mut timing) = slide.timing {
            if let Some(ref mut root) = timing.root_node {
                if let TimeNodeKind::Sequential(ref mut seq) = root.kind {
                    seq.push(anim_node);
                } else {
                    root.kind = TimeNodeKind::Sequential(vec![anim_node]);
                }
            } else {
                timing.root_node = Some(TimeNode {
                    common: CommonTimeNode::default(),
                    kind: TimeNodeKind::Sequential(vec![anim_node]),
                });
            }
        } else {
            slide.timing = Some(SlideTiming {
                root_node: Some(TimeNode {
                    common: CommonTimeNode::default(),
                    kind: TimeNodeKind::Sequential(vec![anim_node]),
                }),
            });
        }
    }
}
