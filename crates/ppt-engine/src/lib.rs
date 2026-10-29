#![forbid(unsafe_code)]
//! PresentationML (PowerPoint `.pptx`) Engine
//!
//! Provides:
//! - Presentation, SlideMaster, SlideLayout, Slide, NotesSlide, Theme models
//! - Streaming fast XML parser & deterministic serializer
//! - 3-tier property, placeholder, and theme cascade engine (SlideMaster -> SlideLayout -> Slide)
//! - Zero-loss OPC package round-trip fidelity

pub mod cascade;
pub mod model;
pub mod package;
pub mod parser;
pub mod serializer;

pub use cascade::{InheritanceResolver, ResolvedParagraph, ResolvedRun, ResolvedShape};
pub use model::*;
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
