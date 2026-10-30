use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum MacroError {
    #[error("Invalid OLE compound file header: {0}")]
    InvalidOleHeader(String),

    #[error("Corrupted FAT/MiniFAT sector chain: {0}")]
    CorruptedFatChain(String),

    #[error("Stream not found: '{0}'")]
    StreamNotFound(String),

    #[error("Directory parse error: {0}")]
    DirectoryParseError(String),

    #[error("Invalid MS-OVBA compressed stream header: {0}")]
    InvalidCompressedStream(String),

    #[error("MS-OVBA decompression failed: {0}")]
    DecompressionError(String),

    #[error("VBA project parsing error: {0}")]
    ProjectParseError(String),

    #[error("VBA syntax lexer error: {0}")]
    LexError(String),

    #[error("VBA AST parser error: {0}")]
    ParseError(String),

    #[error("VBA evaluation error: {0}")]
    EvalError(String),

    #[error("Virtual Office DOM error: {0}")]
    DomError(String),

    #[error("Execution gas limit exhausted: {0}")]
    GasExhausted(String),

    #[error("Capability permission denied: {0}")]
    PermissionDenied(String),

    #[error("Security violation detected: {0}")]
    SecurityViolation(String),

    #[error("I/O error: {0}")]
    IoError(String),
}
