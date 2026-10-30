//! Error types for data connectors and streaming ingestion.

use thiserror::Error;

/// Result alias for data connector operations.
pub type ConnectorResult<T> = Result<T, ConnectorError>;

/// Strongly typed error variants for all supported data sources and ingestion tasks.
#[derive(Debug, Error)]
pub enum ConnectorError {
    #[error("I/O error: {0}")]
    Io(String),

    #[error("Parse error at line {line}, col {col}: {message}")]
    Parse {
        line: usize,
        col: usize,
        message: String,
    },

    #[error("Schema error: {0}")]
    Schema(String),

    #[error("Type coercion error: cannot coerce '{value}' to type {target_type}")]
    Coercion { value: String, target_type: String },

    #[error("JSON error: {0}")]
    Json(String),

    #[error("SQLite format error: {0}")]
    Sqlite(String),

    #[error("Arrow IPC format error: {0}")]
    Arrow(String),

    #[error("Parquet format error: {0}")]
    Parquet(String),

    #[error("Grid bounds exceeded: {0}")]
    BoundsExceeded(String),

    #[error("Excel engine error: {0}")]
    Engine(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),
}

impl From<std::io::Error> for ConnectorError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<serde_json::Error> for ConnectorError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err.to_string())
    }
}

impl From<excel_engine::EngineError> for ConnectorError {
    fn from(err: excel_engine::EngineError) -> Self {
        Self::Engine(err.to_string())
    }
}
