use thiserror::Error;

/// Errors arising in the WebAssembly plugin runtime and host contract.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PluginError {
    #[error("Wasm binary format error: {0}")]
    WasmParseError(String),

    #[error("Linear memory access out of bounds: offset {offset}, length {len}, limit {limit}")]
    MemoryOutOfBounds {
        offset: usize,
        len: usize,
        limit: usize,
    },

    #[error("Execution fuel exhausted ({limit} units consumed)")]
    FuelExhausted { limit: u64 },

    #[error("Permission denied: required capability '{required_capability}' is not granted")]
    PermissionDenied { required_capability: String },

    #[error("Plugin not found: '{0}'")]
    PluginNotFound(String),

    #[error(
        "Invalid plugin lifecycle state: cannot perform '{action}' while in state '{current}'"
    )]
    LifecycleStateError { current: String, action: String },

    #[error("Invalid plugin manifest: {0}")]
    InvalidManifest(String),

    #[error("Host interface call error: {0}")]
    HostCallError(String),

    #[error("Document query error: {0}")]
    DocumentQueryError(String),

    #[error("Document mutation error: {0}")]
    DocumentMutationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Runtime fault: {0}")]
    RuntimeFault(String),
}
