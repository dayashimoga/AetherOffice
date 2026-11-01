//! Cryptographic PII Sanitization Certificate.
//!
//! Produces and validates tamper-proof digital receipts certifying that a document
//! has undergone mandatory PII sanitization in compliance with enterprise DLP policies.

use crate::digest::{compute_digest, HashAlgorithm};
use crate::kdf::hmac;
use serde::{Deserialize, Serialize};

/// Cryptographically verifiable sanitization certificate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SanitizationCertificate {
    pub certificate_id: String,
    pub doc_id: String,
    pub original_digest: String,
    pub sanitized_digest: String,
    pub policy_id: String,
    pub sanitized_at_ms: u64,
    pub redactions_count: Vec<(String, usize)>,
    pub signature: String,
}

impl SanitizationCertificate {
    /// Constructs canonical byte representation used for digital signature verification.
    pub fn canonical_payload(&self) -> String {
        let mut redaction_summary = String::new();
        for (kind, count) in &self.redactions_count {
            redaction_summary.push_str(&format!("{kind}:{count};"));
        }

        format!(
            "AETHER-SAN-CERT:v1|doc={}|orig={}|san={}|policy={}|ts={}|redactions={}",
            self.doc_id,
            self.original_digest,
            self.sanitized_digest,
            self.policy_id,
            self.sanitized_at_ms,
            redaction_summary
        )
    }

    /// Total count of redacted entities recorded by this certificate.
    pub fn total_redactions(&self) -> usize {
        self.redactions_count.iter().map(|(_, c)| *c).sum()
    }
}

/// Helper converting byte slices into lowercase hex strings without external hex crate.
fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Generator for PII sanitization certificates.
pub struct SanitizationCertificateGenerator;

impl SanitizationCertificateGenerator {
    /// Generates a signed sanitization certificate from pre- and post-sanitization document bytes.
    pub fn generate(
        doc_id: &str,
        original_bytes: &[u8],
        sanitized_bytes: &[u8],
        policy_id: &str,
        sanitized_at_ms: u64,
        redactions_count: Vec<(String, usize)>,
        signing_key: &[u8],
    ) -> SanitizationCertificate {
        let orig_hash = compute_digest(HashAlgorithm::Sha256, original_bytes);
        let san_hash = compute_digest(HashAlgorithm::Sha256, sanitized_bytes);

        let orig_hex = bytes_to_hex(&orig_hash);
        let san_hex = bytes_to_hex(&san_hash);

        let cert_id = format!(
            "cert-{}-{}",
            sanitized_at_ms,
            &san_hex[..8.min(san_hex.len())]
        );

        let mut cert = SanitizationCertificate {
            certificate_id: cert_id,
            doc_id: doc_id.to_string(),
            original_digest: orig_hex,
            sanitized_digest: san_hex,
            policy_id: policy_id.to_string(),
            sanitized_at_ms,
            redactions_count,
            signature: String::new(),
        };

        let payload = cert.canonical_payload();
        let sig_bytes = hmac(HashAlgorithm::Sha256, signing_key, payload.as_bytes());
        cert.signature = bytes_to_hex(&sig_bytes);

        cert
    }
}

/// Verifier for PII sanitization certificates.
pub struct SanitizationCertificateValidator;

impl SanitizationCertificateValidator {
    /// Validates the HMAC-SHA256 signature on a sanitization certificate.
    pub fn verify_signature(
        cert: &SanitizationCertificate,
        signing_key: &[u8],
    ) -> Result<(), String> {
        let payload = cert.canonical_payload();
        let expected_sig = hmac(HashAlgorithm::Sha256, signing_key, payload.as_bytes());
        let expected_hex = bytes_to_hex(&expected_sig);

        if cert.signature.eq_ignore_ascii_case(&expected_hex) {
            Ok(())
        } else {
            Err(format!(
                "Invalid sanitization certificate signature for cert {}",
                cert.certificate_id
            ))
        }
    }

    /// Verifies both certificate signature and that sanitized document bytes match the certified digest.
    pub fn verify_document_integrity(
        cert: &SanitizationCertificate,
        sanitized_bytes: &[u8],
        signing_key: &[u8],
    ) -> Result<(), String> {
        Self::verify_signature(cert, signing_key)?;

        let computed_hash = compute_digest(HashAlgorithm::Sha256, sanitized_bytes);
        let computed_hex = bytes_to_hex(&computed_hash);

        if cert.sanitized_digest.eq_ignore_ascii_case(&computed_hex) {
            Ok(())
        } else {
            Err(format!(
                "Document digest mismatch: expected {} but computed {}",
                cert.sanitized_digest, computed_hex
            ))
        }
    }
}
