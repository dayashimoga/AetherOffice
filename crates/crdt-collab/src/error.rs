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

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Decryption error: {0}")]
    DecryptionError(String),

    #[error("Transport error: {0}")]
    TransportError(String),

    #[error("Handshake rejected: {0}")]
    HandshakeRejected(String),

    #[error("Peer not found: {0}")]
    PeerNotFound(u64),

    #[error("Session mismatch: expected '{expected}', got '{actual}'")]
    SessionMismatch { expected: String, actual: String },

    #[error("Replay packet detected: sender {sender_id}, seq {seq}")]
    ReplayDetected { sender_id: u64, seq: u64 },
}
