#![forbid(unsafe_code)]
//! crdt-collab - Conflict-Free Replicated Data Types for Real-Time Office Collaboration
//!
//! Provides deterministic convergence, causal tree ordering (Fugue Tree-RGA),
//! Peritext rich-text formatting intervals, and multi-user awareness.

pub mod error;
pub mod fugue;
pub mod id;
pub mod peritext;

pub use error::CrdtError;
pub use fugue::{FugueDelta, FugueNode, FugueTreeRga, StateVector};
pub use id::OpId;
pub use peritext::{FormatAttribute, PeritextSpan, RichTextRun};

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
