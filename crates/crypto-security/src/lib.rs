//! `crypto-security`
//!
//! Enterprise Cryptographic Security Engine for AetherOffice.
//! Implements X.509 certificate validation, RFC 3161 cryptographic timestamping,
//! PAdES-LTV digital signatures for PDF, and XMLDSIG for OOXML packages.

#![forbid(unsafe_code)]

pub mod aes;
pub mod asn1;
pub mod base64;
pub mod digest;
pub mod error;
pub mod kdf;
pub mod ooxml_crypto;
pub mod pades;
pub mod pdf_crypto;
pub mod pii;
pub mod pkcs7;
pub mod redact;
pub mod sanitize;
pub mod timestamp;
pub mod x509;
pub mod xmldsig;
pub mod zero_knowledge;

pub use aes::{
    aes256_cbc_decrypt, aes256_cbc_encrypt, aes256_gcm_decrypt, aes256_gcm_encrypt, Aes256,
};
pub use asn1::DerValue;
pub use digest::{compute_digest, verify_digest, HashAlgorithm};
pub use error::CryptoSecurityError;
pub use kdf::{hmac, pbkdf2};
pub use ooxml_crypto::{AgileEncryptionInfo, OoxmlAgileCrypto};
pub use pades::{PadesSigner, PadesValidationReport, PadesValidator, PdfByteRange};
pub use pdf_crypto::{PdfAccessSession, PdfEncryptDict, PdfPermissions, PdfSecurityEngine};
pub use pii::{PiiKind, PiiMatch, PiiScanner};
pub use pkcs7::{Pkcs7SignedData, SignatureVerificationResult, SignerInfo};
pub use redact::{PdfRedactor, RedactionBox};
pub use sanitize::{DocumentSanitizer, PrivacyInspectionReport, SanitizeOptions};
pub use timestamp::{TimeStampToken, TstInfo};
pub use x509::{CertificateChainValidator, DistinguishedName, ValidityPeriod, X509Certificate};
pub use xmldsig::{
    OoxmlSignatureReport, XmlDsigBuilder, XmlDsigReference, XmlDsigSignature, XmlDsigValidator,
};
pub use zero_knowledge::ProtectedKey;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check() {
        let data = b"AetherOffice Security Engine";
        let digest = compute_digest(HashAlgorithm::Sha256, data);
        assert_eq!(digest.len(), 32);
        assert!(verify_digest(HashAlgorithm::Sha256, data, &digest).is_ok());
    }
}
