//! `crypto-security`
//!
//! Enterprise Cryptographic Security Engine for AetherOffice.
//! Implements X.509 certificate validation, RFC 3161 cryptographic timestamping,
//! PAdES-LTV digital signatures for PDF, and XMLDSIG for OOXML packages.

#![forbid(unsafe_code)]

pub mod asn1;
pub mod base64;
pub mod digest;
pub mod error;
pub mod pades;
pub mod pkcs7;
pub mod timestamp;
pub mod x509;
pub mod xmldsig;

pub use asn1::DerValue;
pub use digest::{compute_digest, verify_digest, HashAlgorithm};
pub use error::CryptoSecurityError;
pub use pades::{PadesSigner, PadesValidationReport, PadesValidator, PdfByteRange};
pub use pkcs7::{Pkcs7SignedData, SignatureVerificationResult, SignerInfo};
pub use timestamp::{TimeStampToken, TstInfo};
pub use x509::{CertificateChainValidator, DistinguishedName, ValidityPeriod, X509Certificate};
pub use xmldsig::{
    OoxmlSignatureReport, XmlDsigBuilder, XmlDsigReference, XmlDsigSignature, XmlDsigValidator,
};

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
