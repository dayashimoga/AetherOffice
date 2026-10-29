//! Error types for the PDF engine (ISO 32000-1 / ISO 32000-2).

use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum PdfError {
    #[error("Invalid PDF header: {0}")]
    InvalidHeader(String),

    #[error("PDF syntax error at byte {offset}: {message}")]
    SyntaxError { offset: usize, message: String },

    #[error("Unexpected end of file while reading PDF data")]
    UnexpectedEof,

    #[error("Invalid cross-reference table or stream: {0}")]
    InvalidXref(String),

    #[error("Invalid PDF trailer: {0}")]
    InvalidTrailer(String),

    #[error("Indirect object not found: {number} {generation} R")]
    ObjectNotFound { number: u32, generation: u16 },

    #[error("Invalid object type: expected {expected}, found {found}")]
    InvalidType {
        expected: &'static str,
        found: &'static str,
    },

    #[error("Decompression failed: {0}")]
    DecompressionError(String),

    #[error("Malformed page tree: {0}")]
    MalformedPageTree(String),

    #[error("Page index {index} out of range (total pages: {count})")]
    PageOutOfRange { index: usize, count: usize },

    #[error("Circular reference detected while resolving object {number}")]
    CircularReference { number: u32 },

    #[error("Linearization hint error: {0}")]
    LinearizationError(String),

    #[error("I/O error: {0}")]
    Io(String),
}
