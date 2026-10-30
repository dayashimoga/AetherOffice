//! Error types for the crypto-security crate.

use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CryptoSecurityError {
    #[error("ASN.1 DER decode error: {0}")]
    DerDecodeError(String),

    #[error("ASN.1 DER encode error: {0}")]
    DerEncodeError(String),

    #[error("Base64 error: {0}")]
    Base64Error(String),

    #[error("Invalid certificate: {0}")]
    CertificateError(String),

    #[error("Certificate validity error: valid from {not_before} to {not_after}, evaluated at {evaluated_at}")]
    CertificateValidityError {
        not_before: u64,
        not_after: u64,
        evaluated_at: u64,
    },

    #[error("Certificate chain verification failed: {0}")]
    ChainVerificationError(String),

    #[error("Digest mismatch: expected {expected}, calculated {calculated}")]
    DigestMismatch {
        expected: String,
        calculated: String,
    },

    #[error("PKCS#7 signature verification error: {0}")]
    Pkcs7Error(String),

    #[error("RFC 3161 timestamp verification error: {0}")]
    TimestampError(String),

    #[error("PDF ByteRange error: {0}")]
    PdfByteRangeError(String),

    #[error("PAdES validation error: {0}")]
    PadesValidationError(String),

    #[error("XMLDSIG verification error: {0}")]
    XmlDsigError(String),

    #[error("Invalid password provided for document decryption")]
    InvalidPassword,

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Invalid cipher padding: {0}")]
    InvalidPadding(String),

    #[error("Document permission denied: {0}")]
    PermissionDenied(String),

    #[error("OOXML Agile Encryption error: {0}")]
    OoxmlEncryptionError(String),

    #[error("PDF Standard Security error: {0}")]
    PdfSecurityError(String),

    #[error("IO/Buffer error: {0}")]
    BufferError(String),
}
