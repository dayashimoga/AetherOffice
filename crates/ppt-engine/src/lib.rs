#![forbid(unsafe_code)]
//! PresentationML (PowerPoint `.pptx`) Engine
//!
//! Provides:
//! - Presentation, SlideMaster, SlideLayout, Slide, NotesSlide, Theme models
//! - Streaming fast XML parser & deterministic serializer
//! - 3-tier property, placeholder, and theme cascade engine (SlideMaster -> SlideLayout -> Slide)
//! - Zero-loss OPC package round-trip fidelity

pub mod animation;
pub mod cascade;
pub mod drawingml;
pub mod media;
pub mod model;
pub mod morph;
pub mod package;
pub mod parser;
pub mod serializer;

pub use animation::{
    apply_accel_decel, compute_node_progress, solve_cubic_bezier_easing, AnimatedSceneState,
    AnimatedShapeState, AnimationTimeline, ColorAnimation, ColorDirection, ColorSpace,
    CommonTimeNode, EffectAnimation, EffectTransition, GenericAnimation, MotionAnimation,
    RotationAnimation, ScaleAnimation, SetAnimation, SlideTiming, TargetElement, TimeNode,
    TimeNodeFill, TimeNodeKind, TimeNodeRestart, TriggerType,
};
pub use cascade::{InheritanceResolver, ResolvedParagraph, ResolvedRun, ResolvedShape};
pub use drawingml::{
    get_preset_geometry, list_preset_names, CustomGeometry, CustomPath, DrawingPath, GeomGuide,
    GeometryCompiler, GuideContext, PathCommand, PathFillMode, PathInstruction, Point2D,
    PresetDefinition,
};
pub use media::{
    MediaAsset, MediaBounds, MediaEngine, MediaFormat, MediaNode, MediaType,
    AUDIO_RELATIONSHIP_TYPE, MEDIA_RELATIONSHIP_TYPE, VIDEO_RELATIONSHIP_TYPE,
};
pub use model::*;
pub use morph::{
    CubicBezierSegment, DiagramConnection, DiagramDataModel, DiagramPoint, DiagramPointType,
    LinearDirection, MorphEvaluator, MorphMatcher, MorphPair, MorphedShapeState, MorphedSlideState,
    MorphedTextElement, MorphedTransform, PathMorpher, Rect, SmartArtLayoutEngine,
    SmartArtNodeLayout, TextMorpher,
};
pub use package::PresentationPackage;
pub use parser::{
    parse_notes_slide, parse_presentation, parse_slide, parse_slide_layout, parse_slide_master,
    parse_theme,
};
pub use serializer::{
    serialize_notes_slide, serialize_presentation, serialize_slide, serialize_slide_layout,
    serialize_slide_master, serialize_theme,
};

pub fn health_check() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check() {
        assert!(health_check());
    }
}
