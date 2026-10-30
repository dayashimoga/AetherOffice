use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum CopilotError {
    #[error("Empty document or text provided")]
    EmptyDocument,

    #[error("Dimension mismatch: expected {expected}, found {found}")]
    DimensionMismatch { expected: usize, found: usize },

    #[error("Index is empty")]
    EmptyIndex,

    #[error("Chunk not found with id: {0}")]
    ChunkNotFound(String),

    #[error("Document not found with id: {0}")]
    DocumentNotFound(String),

    #[error("Invalid token or subword sequence: {0}")]
    InvalidToken(String),

    #[error("Token budget exceeded: limit was {limit}, actual {actual}")]
    TokenBudgetExceeded { limit: usize, actual: usize },

    #[error("Serialization failure: {0}")]
    SerializationError(String),

    #[error("Deserialization failure: {0}")]
    DeserializationError(String),

    #[error("Semantic chunking error: {0}")]
    ChunkingError(String),
}
