use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum CrdtError {
    #[error("Invalid character offset: {offset}, visible length is {len}")]
    InvalidOffset { offset: usize, len: usize },

    #[error("Node ID not found: {0:?}")]
    NodeNotFound(crate::id::OpId),

    #[error("Invalid formatting range: start {start} > end {end}")]
    InvalidRange { start: usize, end: usize },

    #[error("Serialization / Deserialization error: {0}")]
    SerializationError(String),

    #[error("PieceTree conversion error: {0}")]
    PieceTreeError(String),
}
