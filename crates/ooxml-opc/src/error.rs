//! Shared Engine Error Model for OpenOfficeRS
//! Strictly complies with GEC-v1.0 Article IV: Zero panics, typed Result<T, EngineError>.

use thiserror::Error;

/// Core error enumeration for all OpenOfficeRS engine operations.
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ZIP package error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("XML parsing error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("OPC Part not found: {0}")]
    PartNotFound(String),

    #[error("Invalid or missing content type: {0}")]
    InvalidContentType(String),

    #[error("Relationship not found: {0}")]
    RelationshipNotFound(String),

    #[error("Malformed ECMA-376 package: {0}")]
    MalformedPackage(String),

    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    #[error("Formula parsing or evaluation error: {0}")]
    FormulaError(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

/// Convenience alias for `std::result::Result<T, EngineError>`.
pub type Result<T> = std::result::Result<T, EngineError>;
